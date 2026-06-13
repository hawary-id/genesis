//! ECS systems for the Genesis persistence boundary.
//!
//! All systems run under the [`PersistenceBoundary`] schedule in the following order:
//!
//! 1. [`detect_snapshot_due`] — emits [`SnapshotRequested`] on configured interval.
//! 2. [`handle_snapshot_requests`] — constructs and writes snapshots; emits [`SnapshotCompleted`].
//! 3. [`clear_persisted_dirty_markers`] — stub; reserved for future dirty-chunk cleanup.

use bevy_ecs::prelude::*;

use crate::app::events::{SnapshotCompleted, SnapshotRequested};
use crate::app::plugins::SnapshotConfig;
use crate::persistence::{build_world_snapshot, write_world_snapshot};
use crate::time::SimulationClock;
use crate::world::climate::ClimateChunk;
use crate::world::coord::ChunkCoord;
use crate::world::energy::EnergyAvailabilityChunk;
use crate::world::resource::ResourceChunk;
use crate::world::terrain::TerrainChunk;
use crate::{config::WorldConfig, rng::WorldSeed};

/// Output directory for snapshot files.
///
/// In Phase 1, this is a constant relative path. It is not configurable at runtime.
/// `write_world_snapshot` calls `create_dir_all` so the directory is created on demand.
const SNAPSHOT_OUTPUT_DIRECTORY: &str = "snapshots";

/// Emits [`SnapshotRequested`] when automatic snapshots are due.
///
/// Listed as system #1 in the `PersistenceBoundary` execution order
/// (from `PHASE1_WORLD_TECH_SPEC.md`).
///
/// Automatic snapshots fire when:
/// - `snapshot_interval_ticks > 0`
/// - `total_ticks % snapshot_interval_ticks == 0`
/// - `total_ticks > 0` (prevents a snapshot at the pre-tick world state)
pub fn detect_snapshot_due(
    config: Res<SnapshotConfig>,
    clock: Res<SimulationClock>,
    mut events: EventWriter<SnapshotRequested>,
) {
    let interval = config.snapshot_interval_ticks;
    let ticks = clock.total_ticks;

    if interval > 0 && ticks > 0 && ticks.is_multiple_of(interval) {
        events.send(SnapshotRequested);
    }
}

/// Reads [`SnapshotRequested`] events and writes snapshots to disk.
///
/// Listed as system #2 in the `PersistenceBoundary` execution order.
///
/// For each pending request:
/// 1. Collects chunk data from the ECS query, sorted by `(coord.y, coord.x)`.
/// 2. Calls [`build_world_snapshot`] to assemble a [`WorldSnapshot`].
/// 3. Calls [`write_world_snapshot`] to serialize to JSON.
/// 4. Emits [`SnapshotCompleted`] on success.
///
/// On I/O failure, logs the error. Does not panic. Does not mutate simulation state.
pub fn handle_snapshot_requests(
    mut events: EventReader<SnapshotRequested>,
    config: Res<WorldConfig>,
    seed: Res<WorldSeed>,
    clock: Res<SimulationClock>,
    snap_config: Res<SnapshotConfig>,
    chunk_query: Query<(
        &ChunkCoord,
        &TerrainChunk,
        &ClimateChunk,
        &ResourceChunk,
        &EnergyAvailabilityChunk,
    )>,
    mut completed: EventWriter<SnapshotCompleted>,
) {
    for _ in events.read() {
        let chunks: Vec<_> = chunk_query
            .iter()
            .map(|(coord, terrain, climate, resource, energy)| {
                (
                    *coord,
                    terrain.clone(),
                    climate.clone(),
                    resource.clone(),
                    energy.clone(),
                )
            })
            .collect();

        let snapshot =
            build_world_snapshot(&config, &seed, &clock, snap_config.schema_version, &chunks);

        let total_ticks = snapshot.total_ticks;

        match write_world_snapshot(&snapshot, SNAPSHOT_OUTPUT_DIRECTORY) {
            Ok(()) => {
                completed.send(SnapshotCompleted { total_ticks });
            }
            Err(e) => {
                eprintln!(
                    "[genesis::persistence] snapshot write failed at tick {total_ticks}: {e}"
                );
            }
        }
    }
}

/// Reserved stub for future dirty-chunk tracking cleanup.
///
/// Listed as system #6 in the `PersistenceBoundary` execution order
/// (from `PHASE1_WORLD_TECH_SPEC.md`). Empty in Phase 1.
pub fn clear_persisted_dirty_markers() {
    // Reserved for Phase 2+ dirty-chunk tracking.
    // No implementation required in Milestone 9.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::events::{SnapshotCompleted, SnapshotRequested};
    use crate::app::plugins::SnapshotConfig;
    use crate::app::{App, PersistenceBoundary, PostTickValidation};
    use crate::persistence::SNAPSHOT_SCHEMA_VERSION;
    use crate::testing::{create_test_config, create_test_seed};

    fn test_app() -> App {
        App::new(create_test_config(), create_test_seed())
    }

    #[test]
    fn app_registers_snapshot_config_resource() {
        let app = test_app();
        assert!(
            app.world().contains_resource::<SnapshotConfig>(),
            "SnapshotConfig resource must be registered"
        );
    }

    #[test]
    fn app_registers_snapshot_events() {
        let app = test_app();
        assert!(
            app.world().contains_resource::<Events<SnapshotRequested>>(),
            "SnapshotRequested event resource must be present"
        );
        assert!(
            app.world().contains_resource::<Events<SnapshotCompleted>>(),
            "SnapshotCompleted event resource must be present"
        );
    }

    #[test]
    fn detect_snapshot_due_disabled_when_interval_is_zero() {
        // Systems already registered by App::new(). Interval 0 is the default.
        let mut app = test_app();
        app.run_startup();

        // Default interval is 0 — no snapshots should fire
        for _ in 0..10 {
            app.world_mut()
                .run_schedule(crate::app::schedules::FixedSimulationTick);
            app.world_mut().run_schedule(PostTickValidation);
            app.world_mut().run_schedule(PersistenceBoundary);
        }

        let events = app.world().resource::<Events<SnapshotRequested>>();
        let count = events.get_reader().read(events).count();
        assert_eq!(
            count, 0,
            "no SnapshotRequested events should fire when interval = 0"
        );
    }

    #[test]
    fn detect_snapshot_due_does_not_emit_at_tick_zero() {
        // Use a minimal world to test detect_snapshot_due in isolation at tick 0.
        let mut world = World::new();
        world.insert_resource(SnapshotConfig {
            snapshot_interval_ticks: 1,
            schema_version: SNAPSHOT_SCHEMA_VERSION,
        });
        world.insert_resource(SimulationClock {
            total_ticks: 0, // tick 0
            tick_duration_hours: 1,
        });
        world.init_resource::<Events<SnapshotRequested>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(detect_snapshot_due);
        schedule.run(&mut world);

        let events = world.resource::<Events<SnapshotRequested>>();
        let count = events.get_reader().read(events).count();
        assert_eq!(count, 0, "no SnapshotRequested should emit at tick 0");
    }

    #[test]
    fn detect_snapshot_due_emits_at_interval() {
        // Test in isolation using a minimal world at tick 5 with interval 5.
        let mut world = World::new();
        world.insert_resource(SnapshotConfig {
            snapshot_interval_ticks: 5,
            schema_version: SNAPSHOT_SCHEMA_VERSION,
        });
        world.insert_resource(SimulationClock {
            total_ticks: 5,
            tick_duration_hours: 1,
        });
        world.init_resource::<Events<SnapshotRequested>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(detect_snapshot_due);
        schedule.run(&mut world);

        let events = world.resource::<Events<SnapshotRequested>>();
        let count = events.get_reader().read(events).count();
        assert_eq!(
            count, 1,
            "SnapshotRequested should fire at tick 5 with interval 5"
        );
    }

    #[test]
    fn persistence_does_not_mutate_simulation_state() {
        let mut app = test_app();
        app.run_startup();

        let ticks_before = app.world().resource::<SimulationClock>().total_ticks;

        // Manually trigger a snapshot
        app.world_mut()
            .resource_mut::<Events<SnapshotRequested>>()
            .send(SnapshotRequested);

        app.world_mut().run_schedule(PersistenceBoundary);

        let ticks_after = app.world().resource::<SimulationClock>().total_ticks;
        assert_eq!(
            ticks_before, ticks_after,
            "PersistenceBoundary must not mutate SimulationClock"
        );

        // Entity count unchanged
        let entity_count = app.world().entities().len() as usize;
        assert_eq!(
            entity_count, 64,
            "chunk entity count must not change after persistence"
        );
    }

    #[test]
    fn handle_snapshot_requests_emits_completed_on_success() {
        let mut app = test_app();
        app.run_startup();

        app.world_mut()
            .resource_mut::<Events<SnapshotRequested>>()
            .send(SnapshotRequested);

        app.world_mut().run_schedule(PersistenceBoundary);

        let events = app.world().resource::<Events<SnapshotCompleted>>();
        let count = events.get_reader().read(events).count();
        assert_eq!(
            count, 1,
            "SnapshotCompleted must be emitted after successful write"
        );

        // Clean up the snapshot_0000000000.json written by this test
        let filename = "snapshots/snapshot_0000000000.json";
        let path = std::path::Path::new(filename);
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn automatic_interval_30_ticks() {
        let mut app = test_app();

        // Configure snapshot config resource to trigger every 10 ticks
        app.world_mut()
            .resource_mut::<SnapshotConfig>()
            .snapshot_interval_ticks = 10;

        app.run_startup();

        let mut completed_ticks = Vec::new();

        for _ in 1..=30 {
            app.world_mut()
                .run_schedule(crate::app::schedules::FixedSimulationTick);
            app.world_mut().run_schedule(PostTickValidation);
            app.world_mut().run_schedule(PersistenceBoundary);

            // Read SnapshotCompleted events
            let mut events = app.world_mut().resource_mut::<Events<SnapshotCompleted>>();
            let mut reader = events.get_reader();
            for ev in reader.read(&events) {
                completed_ticks.push(ev.total_ticks);
            }
            events.clear();
        }

        assert_eq!(
            completed_ticks,
            vec![10, 20, 30],
            "should have snapshotted at ticks 10, 20, 30"
        );

        // Clean up written files.
        for tick in &[10, 20, 30] {
            let filename = format!("snapshots/snapshot_{:010}.json", tick);
            let path = std::path::Path::new(&filename);
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
        }

        // Try to remove the snapshots directory if empty
        let _ = std::fs::remove_dir("snapshots");
    }
}

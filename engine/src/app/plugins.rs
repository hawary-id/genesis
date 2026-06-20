//! ECS bootstrap helpers.
//!
//! Provides resource registration and schedule initialization for the Genesis engine.

use bevy_ecs::prelude::World;
use serde::{Deserialize, Serialize};

use crate::agent::StableIdGenerator;
use crate::config::{WorldBounds, WorldConfig};
use crate::rng::WorldSeed;
use crate::time::{SeasonState, SimulationClock};

use super::events::{SnapshotCompleted, SnapshotRequested, WorldGenerationCompleted};
use super::schedules::register_schedules;
use crate::persistence::SNAPSHOT_SCHEMA_VERSION;

/// Controls snapshot scheduling at the persistence boundary.
///
/// Defined in `PHASE1_WORLD_TECH_SPEC.md` as a required Phase 1 resource.
///
/// Setting `snapshot_interval_ticks = 0` disables automatic snapshots.
/// Manual triggers via [`SnapshotRequested`] still work regardless of this setting.
#[derive(bevy_ecs::prelude::Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Number of ticks between automatic snapshots. `0` disables automatic snapshots.
    pub snapshot_interval_ticks: u32,

    /// Schema version embedded in written snapshot files.
    ///
    /// Must equal [`crate::persistence::SNAPSHOT_SCHEMA_VERSION`] to load correctly.
    pub schema_version: u32,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            snapshot_interval_ticks: 0, // automatic snapshots disabled by default
            schema_version: SNAPSHOT_SCHEMA_VERSION,
        }
    }
}

/// Registers all foundational resources and schedules required by Genesis.
///
/// Resources registered:
///
/// - [`WorldConfig`] — immutable world parameters
/// - [`WorldSeed`] — root deterministic seed
/// - [`SimulationClock`] — canonical simulation time
/// - [`WorldBounds`] — validated coordinate boundaries derived from [`WorldConfig`]
/// - [`SeasonState`] — initial derived seasonal state
/// - [`SnapshotConfig`] — persistence boundary configuration
/// - [`StableIdGenerator`] — unique stable identifier generator
///
/// Also registers the five Phase 1 schedule labels in canonical execution order.
/// Also registers Phase 1 events: `WorldGenerationCompleted`, `SnapshotRequested`, `SnapshotCompleted`.
pub fn register_initial_resources(world: &mut World, config: WorldConfig, seed: WorldSeed) {
    let bounds = WorldBounds::from_config(&config);
    let initial_season = SeasonState::derive(0, &config);

    world.insert_resource(config);
    world.insert_resource(seed);
    world.insert_resource(SimulationClock::new());
    world.insert_resource(bounds);
    world.insert_resource(initial_season);
    world.insert_resource(SnapshotConfig::default());
    world.insert_resource(StableIdGenerator::new());
    world.insert_resource(crate::agent::GenomeConfig::default());
    world.insert_resource(crate::agent::diagnostics::PopulationStatistics::default());

    // Register events
    world.init_resource::<bevy_ecs::event::Events<WorldGenerationCompleted>>();
    world.init_resource::<bevy_ecs::event::Events<SnapshotRequested>>();
    world.init_resource::<bevy_ecs::event::Events<SnapshotCompleted>>();
    world.init_resource::<bevy_ecs::event::Events<crate::agent::ObservationEvent>>();
    world.init_resource::<bevy_ecs::event::Events<crate::agent::EventMemoryEvent>>();
    world.init_resource::<bevy_ecs::event::Events<crate::agent::SocialMemoryEvent>>();
    world.insert_resource(crate::agent::EventSequenceCounter::default());

    register_schedules(world);
}

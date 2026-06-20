//! Snapshot construction, serialization, and load path functions.

use std::path::Path;

use bevy_ecs::prelude::World;

use crate::config::{WorldBounds, WorldConfig};
use crate::persistence::{
    AgentSnapshot, ChunkSnapshot, SnapshotError, WorldSnapshot, SNAPSHOT_SCHEMA_VERSION,
};
use crate::rng::WorldSeed;
use crate::time::{SeasonState, SimulationClock};
use crate::world::climate::ClimateChunk;
use crate::world::coord::ChunkCoord;
use crate::world::energy::EnergyAvailabilityChunk;
use crate::world::generation::Generated;
use crate::world::resource::ResourceChunk;
use crate::world::terrain::TerrainChunk;

/// Assembles a [`WorldSnapshot`] from raw ECS data.
///
/// This is a pure function with no ECS access, no I/O, and no side effects.
/// It is called by `handle_snapshot_requests` after collecting data from the ECS world.
///
/// # Determinism
///
/// Chunks are sorted by `(coord.y, coord.x)` ascending using a stable sort
/// before being included in the snapshot. This satisfies ADR-002 stable
/// iteration requirements regardless of ECS query order.
pub fn build_world_snapshot(
    config: &WorldConfig,
    seed: &WorldSeed,
    clock: &SimulationClock,
    schema_version: u32,
    chunks: &[(
        ChunkCoord,
        TerrainChunk,
        ClimateChunk,
        ResourceChunk,
        EnergyAvailabilityChunk,
    )],
    id_generator: &crate::agent::StableIdGenerator,
    agents: &[AgentSnapshot],
) -> WorldSnapshot {
    let mut sorted = chunks.to_vec();
    sorted.sort_by(|a, b| {
        let (ay, ax) = (a.0.y, a.0.x);
        let (by, bx) = (b.0.y, b.0.x);
        (ay, ax).cmp(&(by, bx))
    });

    let chunk_records = sorted
        .into_iter()
        .map(
            |(coord, terrain, climate, resources, energy)| ChunkSnapshot {
                coord,
                terrain,
                climate,
                resources,
                energy,
            },
        )
        .collect();

    let mut sorted_agents = agents.to_vec();
    sorted_agents.sort_by_key(|a| a.metadata.id);

    WorldSnapshot {
        schema_version,
        total_ticks: clock.total_ticks,
        config: config.clone(),
        seed: *seed,
        id_generator: *id_generator,
        chunks: chunk_records,
        agents: sorted_agents,
    }
}

/// Serializes a [`WorldSnapshot`] to JSON and writes it to disk.
///
/// Filename format: `snapshot_{total_ticks:010}.json`
///
/// On success returns `Ok(())`.
/// On I/O or serialization failure returns the appropriate [`SnapshotError`] variant.
/// Does not panic. Does not mutate any simulation state.
pub fn write_world_snapshot(
    snapshot: &WorldSnapshot,
    output_directory: &str,
) -> Result<(), SnapshotError> {
    let dir = Path::new(output_directory);
    std::fs::create_dir_all(dir)?;

    let filename = format!("snapshot_{:010}.json", snapshot.total_ticks);
    let path = dir.join(filename);

    let json = serde_json::to_string_pretty(snapshot)?;
    std::fs::write(path, json)?;

    Ok(())
}

/// Reads and deserializes a [`WorldSnapshot`] from a JSON file.
///
/// Checks `schema_version` against [`SNAPSHOT_SCHEMA_VERSION`] and returns
/// [`SnapshotError::SchemaMismatch`] if they differ.
pub fn load_world_snapshot(path: &str) -> Result<WorldSnapshot, SnapshotError> {
    let contents = std::fs::read_to_string(path)?;
    let snapshot: WorldSnapshot = serde_json::from_str(&contents)?;

    if snapshot.schema_version != SNAPSHOT_SCHEMA_VERSION {
        return Err(SnapshotError::SchemaMismatch {
            found: snapshot.schema_version,
            expected: SNAPSHOT_SCHEMA_VERSION,
        });
    }

    Ok(snapshot)
}

/// Reconstructs an ECS world from a [`WorldSnapshot`].
///
/// This function is not an ECS system. It is called by test harnesses and
/// future runner infrastructure after loading a snapshot from disk.
///
/// After this function returns, the caller must run `validate_world_on_startup`
/// before enabling `FixedSimulationTick`.
///
/// # Reconstruction Order
///
/// 1. Insert `WorldConfig`.
/// 2. Insert `WorldSeed`.
/// 3. Insert `SimulationClock` with `total_ticks` from the snapshot.
/// 4. Derive and insert `WorldBounds`.
/// 5. Derive and insert `SeasonState`.
/// 6. Spawn one chunk entity per `ChunkSnapshot`.
/// 7. Mark all chunk entities `Generated`.
pub fn reconstruct_world_from_snapshot(world: &mut World, snapshot: WorldSnapshot) {
    let config = snapshot.config.clone();
    let total_ticks = snapshot.total_ticks;

    let bounds = WorldBounds::from_config(&config);
    let season_state = SeasonState::derive(total_ticks, &config);
    let clock = SimulationClock {
        total_ticks,
        tick_duration_hours: 1,
    };

    world.insert_resource(config.clone());
    world.insert_resource(snapshot.seed);
    world.insert_resource(clock);
    let spatial_map = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);
    world.insert_resource(bounds);
    world.insert_resource(spatial_map);
    world.insert_resource(season_state);
    world.insert_resource(snapshot.id_generator);

    for chunk in snapshot.chunks {
        let coord = chunk.coord;
        let e = world
            .spawn((
                chunk.coord,
                chunk.terrain,
                chunk.climate,
                chunk.resources,
                chunk.energy,
                Generated,
            ))
            .id();
        world
            .resource_mut::<crate::world::spatial::SpatialMap>()
            .set(coord, e);
    }

    let gen_config = crate::agent::GenomeConfig::default();
    for mut agent in snapshot.agents {
        // Dynamically pad undersized genomes to ensure forward compatibility
        if agent.genome.genes.len() < crate::agent::GENOME_SIZE {
            agent.genome.genes.resize(crate::agent::GENOME_SIZE, 0.5);
        }
        let phenotype =
            crate::agent::systems::derive_phenotype(&agent.genome, &gen_config, &config);
        world.spawn((
            agent.metadata,
            agent.position,
            agent.stock,
            crate::agent::ActionRequest::new(crate::agent::ActionIntent::None),
            agent.genome,
            agent.lineage,
            phenotype,
            agent.location_memory.unwrap_or_default(),
            agent.event_memory.unwrap_or_default(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::testing::{create_test_config, create_test_seed};
    use crate::time::SimulationClock;

    /// Builds a minimal world snapshot for testing.
    fn make_test_snapshot(total_ticks: u32, config: WorldConfig, seed: WorldSeed) -> WorldSnapshot {
        let clock = SimulationClock {
            total_ticks,
            tick_duration_hours: 1,
        };
        build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &[],
            &crate::agent::StableIdGenerator::new(),
            &[],
        )
    }

    // -------------------------------------------------------------------------
    // build_world_snapshot tests
    // -------------------------------------------------------------------------

    #[test]
    fn build_snapshot_contains_correct_tick() {
        let config = create_test_config();
        let seed = create_test_seed();
        let clock = SimulationClock {
            total_ticks: 42,
            tick_duration_hours: 1,
        };
        let snapshot = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &[],
            &crate::agent::StableIdGenerator::new(),
            &[],
        );
        assert_eq!(snapshot.total_ticks, 42);
    }

    #[test]
    fn build_snapshot_schema_version_propagated() {
        let snapshot = make_test_snapshot(0, create_test_config(), create_test_seed());
        assert_eq!(snapshot.schema_version, SNAPSHOT_SCHEMA_VERSION);
    }

    #[test]
    fn build_snapshot_chunk_count_matches_input() {
        let config = create_test_config();
        let seed = create_test_seed();

        // Generate a world to get real chunk data
        let mut app = App::new(config.clone(), seed);
        app.run_startup();
        let world = app.world_mut();

        let mut query = world.query::<(
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
            &EnergyAvailabilityChunk,
        )>();

        let chunks: Vec<_> = query
            .iter(world)
            .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
            .collect();

        let chunk_count = chunks.len();
        let clock = SimulationClock {
            total_ticks: 0,
            tick_duration_hours: 1,
        };
        let snapshot = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &chunks,
            &crate::agent::StableIdGenerator::new(),
            &[],
        );

        assert_eq!(snapshot.chunks.len(), chunk_count);
    }

    #[test]
    fn build_snapshot_sorts_chunks_by_y_then_x() {
        let config = create_test_config();
        let seed = create_test_seed();
        let mut app = App::new(config.clone(), seed);
        app.run_startup();
        let world = app.world_mut();

        let mut query = world.query::<(
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
            &EnergyAvailabilityChunk,
        )>();

        let mut chunks: Vec<_> = query
            .iter(world)
            .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
            .collect();

        // Reverse to ensure unsorted input
        chunks.reverse();

        let clock = SimulationClock {
            total_ticks: 0,
            tick_duration_hours: 1,
        };
        let snapshot = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &chunks,
            &crate::agent::StableIdGenerator::new(),
            &[],
        );

        // Assert sorted by (y, x) ascending
        for window in snapshot.chunks.windows(2) {
            let a = (window[0].coord.y, window[0].coord.x);
            let b = (window[1].coord.y, window[1].coord.x);
            assert!(a <= b, "chunk order violated: {:?} > {:?}", a, b);
        }
    }

    #[test]
    fn build_snapshot_is_deterministic() {
        let config = create_test_config();
        let seed = create_test_seed();
        let mut app = App::new(config.clone(), seed);
        app.run_startup();
        let world = app.world_mut();

        let mut query = world.query::<(
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
            &EnergyAvailabilityChunk,
        )>();

        let chunks: Vec<_> = query
            .iter(world)
            .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
            .collect();

        let clock = SimulationClock {
            total_ticks: 10,
            tick_duration_hours: 1,
        };
        let s1 = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &chunks,
            &crate::agent::StableIdGenerator::new(),
            &[],
        );
        let s2 = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &chunks,
            &crate::agent::StableIdGenerator::new(),
            &[],
        );

        assert_eq!(s1.total_ticks, s2.total_ticks);
        assert_eq!(s1.chunks.len(), s2.chunks.len());
        for (a, b) in s1.chunks.iter().zip(s2.chunks.iter()) {
            assert_eq!(a.coord, b.coord);
            assert_eq!(a.terrain.elevation, b.terrain.elevation);
        }
    }

    // -------------------------------------------------------------------------
    // write_world_snapshot + load_world_snapshot tests
    // -------------------------------------------------------------------------

    #[test]
    fn write_and_read_snapshot_roundtrip() {
        let config = create_test_config();
        let seed = create_test_seed();
        let snapshot = make_test_snapshot(100, config, seed);

        let dir = std::env::temp_dir().join(format!(
            "genesis_test_roundtrip_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        let dir_str = dir.to_string_lossy().to_string();

        write_world_snapshot(&snapshot, &dir_str).expect("write should succeed");

        let path = dir.join("snapshot_0000000100.json");
        let loaded = load_world_snapshot(&path.to_string_lossy()).expect("load should succeed");

        assert_eq!(loaded.schema_version, snapshot.schema_version);
        assert_eq!(loaded.total_ticks, snapshot.total_ticks);
        assert_eq!(loaded.seed.root_seed, snapshot.seed.root_seed);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_rejects_wrong_schema_version() {
        let config = create_test_config();
        let seed = create_test_seed();
        let snapshot = make_test_snapshot(100, config, seed);

        let dir = std::env::temp_dir().join(format!(
            "genesis_test_schema_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        let dir_str = dir.to_string_lossy().to_string();

        write_world_snapshot(&snapshot, &dir_str).expect("write should succeed");

        let path = dir.join("snapshot_0000000100.json");
        let path_str = path.to_string_lossy().to_string();

        // Modify the file to have a bad schema version
        let contents = std::fs::read_to_string(&path).expect("read should succeed");
        let modified = contents.replace(
            &format!("\"schema_version\": {}", SNAPSHOT_SCHEMA_VERSION),
            "\"schema_version\": 999",
        );
        std::fs::write(&path, modified).expect("write should succeed");

        let result = load_world_snapshot(&path_str);
        assert!(result.is_err());
        match result.unwrap_err() {
            SnapshotError::SchemaMismatch { found, expected } => {
                assert_eq!(found, 999);
                assert_eq!(expected, SNAPSHOT_SCHEMA_VERSION);
            }
            other => panic!("expected SchemaMismatch, got {:?}", other),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn written_snapshot_is_human_readable() {
        let snapshot = make_test_snapshot(5, create_test_config(), create_test_seed());

        let dir = std::env::temp_dir().join(format!(
            "genesis_test_readable_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        let dir_str = dir.to_string_lossy().to_string();

        write_world_snapshot(&snapshot, &dir_str).expect("write should succeed");

        let path = dir.join("snapshot_0000000005.json");
        let contents = std::fs::read_to_string(&path).expect("file should exist");

        assert!(
            contents.contains("schema_version"),
            "missing schema_version key"
        );
        assert!(contents.contains("total_ticks"), "missing total_ticks key");
        assert!(contents.contains("config"), "missing config key");
        assert!(contents.contains("chunks"), "missing chunks key");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_to_nonexistent_parent_creates_directory() {
        let dir = std::env::temp_dir().join(format!(
            "genesis_test_mkdir_{}/nested/path",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        let dir_str = dir.to_string_lossy().to_string();
        let snapshot = make_test_snapshot(0, create_test_config(), create_test_seed());

        let result = write_world_snapshot(&snapshot, &dir_str);
        assert!(result.is_ok(), "create_dir_all should handle nested paths");

        let _ = std::fs::remove_dir_all(std::env::temp_dir().join(format!(
                "genesis_test_mkdir_{}",
                dir_str
                    .split("genesis_test_mkdir_")
                    .nth(1)
                    .unwrap_or("")
                    .split('/')
                    .next()
                    .unwrap_or("")
            )));
    }

    // -------------------------------------------------------------------------
    // reconstruct_world_from_snapshot tests
    // -------------------------------------------------------------------------

    #[test]
    fn reconstruct_world_has_correct_chunk_count() {
        let config = create_test_config();
        let seed = create_test_seed();

        // Generate world to get snapshot data
        let mut src_app = App::new(config.clone(), seed);
        src_app.run_startup();
        let src_world = src_app.world_mut();

        let mut query = src_world.query::<(
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
            &EnergyAvailabilityChunk,
        )>();
        let chunks: Vec<_> = query
            .iter(src_world)
            .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
            .collect();
        let expected_count = chunks.len();

        let clock = SimulationClock {
            total_ticks: 0,
            tick_duration_hours: 1,
        };
        let snapshot = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &chunks,
            &crate::agent::StableIdGenerator::new(),
            &[],
        );

        let mut new_world = World::new();
        reconstruct_world_from_snapshot(&mut new_world, snapshot);

        assert_eq!(new_world.entities().len() as usize, expected_count);
    }

    #[test]
    fn reconstruct_world_clock_matches_snapshot_ticks() {
        let config = create_test_config();
        let seed = create_test_seed();
        let snapshot = make_test_snapshot(77, config, seed);

        let mut world = World::new();
        reconstruct_world_from_snapshot(&mut world, snapshot);

        let clock = world.resource::<SimulationClock>();
        assert_eq!(clock.total_ticks, 77);
    }

    #[test]
    fn reconstruct_world_season_state_matches_derived() {
        use crate::time::SeasonState;
        let config = create_test_config();
        let seed = create_test_seed();
        let snapshot = make_test_snapshot(2160, config.clone(), seed);

        let mut world = World::new();
        reconstruct_world_from_snapshot(&mut world, snapshot);

        let season = world.resource::<SeasonState>();
        let derived = SeasonState::derive(2160, &config);

        assert_eq!(season.season_index, derived.season_index);
        assert_eq!(season.tick_in_season, derived.tick_in_season);
        assert!((season.progress - derived.progress).abs() < 1e-6);
    }

    #[test]
    fn reconstruct_world_config_matches_snapshot() {
        let config = create_test_config();
        let seed = create_test_seed();
        let snapshot = make_test_snapshot(0, config.clone(), seed);

        let mut world = World::new();
        reconstruct_world_from_snapshot(&mut world, snapshot);

        let loaded_config = world.resource::<WorldConfig>();
        assert_eq!(loaded_config.world_width, config.world_width);
        assert_eq!(loaded_config.world_height, config.world_height);
        assert_eq!(loaded_config.chunk_size, config.chunk_size);
    }

    #[test]
    fn reconstruct_world_all_chunks_have_generated_marker() {
        let config = create_test_config();
        let seed = create_test_seed();

        let mut src_app = App::new(config.clone(), seed);
        src_app.run_startup();
        let src_world = src_app.world_mut();

        let mut query = src_world.query::<(
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
            &EnergyAvailabilityChunk,
        )>();
        let chunks: Vec<_> = query
            .iter(src_world)
            .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
            .collect();
        let expected_count = chunks.len();

        let clock = SimulationClock {
            total_ticks: 0,
            tick_duration_hours: 1,
        };
        let snapshot = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &chunks,
            &crate::agent::StableIdGenerator::new(),
            &[],
        );

        let mut new_world = World::new();
        reconstruct_world_from_snapshot(&mut new_world, snapshot);

        let mut gen_query = new_world.query::<&Generated>();
        let generated_count = gen_query.iter(&new_world).count();
        assert_eq!(generated_count, expected_count);
    }

    #[test]
    fn save_load_equivalence() {
        use crate::app::schedules::FixedSimulationTick;
        use crate::app::{App, PersistenceBoundary, PostTickValidation};

        let config = create_test_config();
        let seed = create_test_seed();

        // 1. Continuous Run (N = 50 ticks)
        let mut app_continuous = App::new(config.clone(), seed);
        app_continuous.run_startup();
        for _ in 0..50 {
            app_continuous.world_mut().run_schedule(FixedSimulationTick);
            app_continuous.world_mut().run_schedule(PostTickValidation);
            app_continuous.world_mut().run_schedule(PersistenceBoundary);
        }
        let world_continuous = app_continuous.world_mut();

        // 2. Split Run: Run A = 30 ticks
        let mut app_split = App::new(config.clone(), seed);
        app_split.run_startup();
        for _ in 0..30 {
            app_split.world_mut().run_schedule(FixedSimulationTick);
            app_split.world_mut().run_schedule(PostTickValidation);
            app_split.world_mut().run_schedule(PersistenceBoundary);
        }

        // Build snapshot from split run
        let world_split = app_split.world_mut();
        let mut query = world_split.query::<(
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
            &EnergyAvailabilityChunk,
        )>();
        let chunks: Vec<_> = query
            .iter(world_split)
            .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
            .collect();
        let clock = world_split.resource::<SimulationClock>().clone();
        let id_generator = *world_split.resource::<crate::agent::StableIdGenerator>();
        let mut agent_query = world_split.query::<(
            &crate::agent::AgentMetadata,
            &crate::agent::AgentPosition,
            &crate::agent::MetabolicStock,
            &crate::agent::Genome,
            &crate::agent::LineageMetadata,
            Option<&crate::agent::LocationMemory>,
            Option<&crate::agent::components::EventMemory>,
        )>();
        let agents: Vec<_> = agent_query
            .iter(world_split)
            .map(
                |(m, p, s, g, l, lm, em)| crate::persistence::AgentSnapshot {
                    metadata: *m,
                    position: *p,
                    stock: *s,
                    genome: g.clone(),
                    lineage: *l,
                    location_memory: lm.cloned(),
                    event_memory: em.cloned(),
                },
            )
            .collect();
        let snapshot = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &chunks,
            &id_generator,
            &agents,
        );

        // Serialize and Deserialize to simulate save/load boundary
        let json = serde_json::to_string(&snapshot).expect("serialize failed");
        let deserialized: WorldSnapshot = serde_json::from_str(&json).expect("deserialize failed");

        // 3. Reconstruct world from deserialized snapshot in a new App
        let mut app_loaded = App::new(config.clone(), seed);
        // Do NOT run app_loaded.run_startup() since that would regenerate new terrain chunk entities.
        reconstruct_world_from_snapshot(app_loaded.world_mut(), deserialized);

        // Note: validate_world_on_startup is intentionally not called here.
        // In Milestone 11, agents are not part of the snapshot format.
        // Running startup validation on a reconstructed world would incorrectly
        // report AgentCountMismatch because initial_agent_count > 0 but no
        // agents are present in the reconstructed state. Agent snapshot
        // support will be addressed in a future milestone.

        // Run remaining B = 20 ticks (A + B = 50)
        for _ in 0..20 {
            app_loaded.world_mut().run_schedule(FixedSimulationTick);
            app_loaded.world_mut().run_schedule(PostTickValidation);
            app_loaded.world_mut().run_schedule(PersistenceBoundary);
        }

        // 4. Assert equivalence
        crate::testing::assert_worlds_equivalent(world_continuous, app_loaded.world_mut());
    }

    #[test]
    fn test_snapshot_startup_validation() {
        use crate::app::App;
        use crate::validation::systems::validate_world_on_startup;

        let config = create_test_config();
        let seed = create_test_seed();
        let mut app = App::new(config.clone(), seed);
        app.run_startup();

        let world = app.world_mut();
        let mut query = world.query::<(
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
            &EnergyAvailabilityChunk,
        )>();
        let chunks: Vec<_> = query
            .iter(world)
            .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
            .collect();
        let clock = world.resource::<SimulationClock>().clone();
        let id_generator = *world.resource::<crate::agent::StableIdGenerator>();
        let mut agent_query = world.query::<(
            &crate::agent::AgentMetadata,
            &crate::agent::AgentPosition,
            &crate::agent::MetabolicStock,
            &crate::agent::Genome,
            &crate::agent::LineageMetadata,
            Option<&crate::agent::LocationMemory>,
            Option<&crate::agent::components::EventMemory>,
        )>();
        let agents: Vec<_> = agent_query
            .iter(world)
            .map(
                |(m, p, s, g, l, lm, em)| crate::persistence::AgentSnapshot {
                    metadata: *m,
                    position: *p,
                    stock: *s,
                    genome: g.clone(),
                    lineage: *l,
                    location_memory: lm.cloned(),
                    event_memory: em.cloned(),
                },
            )
            .collect();

        let snapshot = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &chunks,
            &id_generator,
            &agents,
        );

        let mut app_loaded = App::new(config.clone(), seed);
        reconstruct_world_from_snapshot(app_loaded.world_mut(), snapshot);

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(validate_world_on_startup);
        schedule.run(app_loaded.world_mut());
    }

    #[test]
    fn test_reconstruct_pads_undersized_genomes() {
        let config = create_test_config();
        let seed = create_test_seed();
        let mut snapshot = make_test_snapshot(0, config.clone(), seed);

        let agent_snap = crate::persistence::AgentSnapshot {
            metadata: crate::agent::AgentMetadata::new(1),
            position: crate::agent::AgentPosition::new(crate::world::coord::WorldCoord::new(0, 0)),
            stock: crate::agent::MetabolicStock::new(100.0, 0),
            genome: crate::agent::Genome::new(vec![0.1, 0.2]), // Undersized genome
            lineage: crate::agent::LineageMetadata::new(None, 0),
            location_memory: None,
            event_memory: None,
        };
        snapshot.agents.push(agent_snap);

        let mut world = World::new();
        reconstruct_world_from_snapshot(&mut world, snapshot);

        let mut query = world.query::<&crate::agent::Genome>();
        let genome = query.iter(&world).next().unwrap();

        assert_eq!(genome.genes.len(), crate::agent::GENOME_SIZE);
        assert_eq!(genome.genes[0], 0.1);
        assert_eq!(genome.genes[1], 0.2);
        assert_eq!(genome.genes[2], 0.5); // padded with 0.5
    }

    #[test]
    fn test_reconstruct_preserves_genome_length() {
        let config = create_test_config();
        let seed = create_test_seed();
        let mut snapshot = make_test_snapshot(0, config.clone(), seed);

        let genes = vec![0.5; crate::agent::GENOME_SIZE];
        let agent_snap = crate::persistence::AgentSnapshot {
            metadata: crate::agent::AgentMetadata::new(1),
            position: crate::agent::AgentPosition::new(crate::world::coord::WorldCoord::new(0, 0)),
            stock: crate::agent::MetabolicStock::new(100.0, 0),
            genome: crate::agent::Genome::new(genes),
            lineage: crate::agent::LineageMetadata::new(None, 0),
            location_memory: None,
            event_memory: None,
        };
        snapshot.agents.push(agent_snap);

        let mut world = World::new();
        reconstruct_world_from_snapshot(&mut world, snapshot);

        let mut query = world.query::<&crate::agent::Genome>();
        let genome = query.iter(&world).next().unwrap();

        assert_eq!(genome.genes.len(), crate::agent::GENOME_SIZE);
    }
}

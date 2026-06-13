//! Snapshot construction, serialization, and load path functions.

use std::path::Path;

use bevy_ecs::prelude::World;

use crate::config::{WorldBounds, WorldConfig};
use crate::persistence::{ChunkSnapshot, SnapshotError, WorldSnapshot, SNAPSHOT_SCHEMA_VERSION};
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

    WorldSnapshot {
        schema_version,
        total_ticks: clock.total_ticks,
        config: config.clone(),
        seed: *seed,
        chunks: chunk_records,
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
    world.insert_resource(bounds);
    world.insert_resource(season_state);

    for chunk in snapshot.chunks {
        world.spawn((
            chunk.coord,
            chunk.terrain,
            chunk.climate,
            chunk.resources,
            chunk.energy,
            Generated,
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
        build_world_snapshot(&config, &seed, &clock, SNAPSHOT_SCHEMA_VERSION, &[])
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
        let snapshot = build_world_snapshot(&config, &seed, &clock, SNAPSHOT_SCHEMA_VERSION, &[]);
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
        let snapshot =
            build_world_snapshot(&config, &seed, &clock, SNAPSHOT_SCHEMA_VERSION, &chunks);

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
        let snapshot =
            build_world_snapshot(&config, &seed, &clock, SNAPSHOT_SCHEMA_VERSION, &chunks);

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
        let s1 = build_world_snapshot(&config, &seed, &clock, SNAPSHOT_SCHEMA_VERSION, &chunks);
        let s2 = build_world_snapshot(&config, &seed, &clock, SNAPSHOT_SCHEMA_VERSION, &chunks);

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
        let snapshot =
            build_world_snapshot(&config, &seed, &clock, SNAPSHOT_SCHEMA_VERSION, &chunks);

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
        let snapshot =
            build_world_snapshot(&config, &seed, &clock, SNAPSHOT_SCHEMA_VERSION, &chunks);

        let mut new_world = World::new();
        reconstruct_world_from_snapshot(&mut new_world, snapshot);

        let mut gen_query = new_world.query::<&Generated>();
        let generated_count = gen_query.iter(&new_world).count();
        assert_eq!(generated_count, expected_count);
    }

    fn assert_worlds_equivalent(world_a: &mut World, world_b: &mut World) {
        use crate::config::WorldConfig;
        use crate::rng::WorldSeed;
        use crate::time::{SeasonState, SimulationClock};
        use crate::world::climate::ClimateChunk;
        use crate::world::coord::ChunkCoord;
        use crate::world::energy::EnergyAvailabilityChunk;
        use crate::world::resource::ResourceChunk;
        use crate::world::terrain::TerrainChunk;

        // 1. Compare simulation clock
        let clock_a = world_a.resource::<SimulationClock>();
        let clock_b = world_b.resource::<SimulationClock>();
        assert_eq!(
            clock_a.total_ticks, clock_b.total_ticks,
            "simulation clock ticks mismatch"
        );
        assert_eq!(
            clock_a.tick_duration_hours, clock_b.tick_duration_hours,
            "tick duration hours mismatch"
        );

        // 2. Compare SeasonState
        let season_a = world_a.resource::<SeasonState>();
        let season_b = world_b.resource::<SeasonState>();
        assert_eq!(
            season_a.season_index, season_b.season_index,
            "season index mismatch"
        );
        assert_eq!(
            season_a.tick_in_season, season_b.tick_in_season,
            "tick in season mismatch"
        );
        assert!(
            (season_a.progress - season_b.progress).abs() < 1e-5,
            "season progress mismatch"
        );

        // 3. Compare WorldSeed
        let seed_a = world_a.resource::<WorldSeed>();
        let seed_b = world_b.resource::<WorldSeed>();
        assert_eq!(seed_a.root_seed, seed_b.root_seed, "world seed mismatch");

        // 4. Compare WorldConfig
        let config_a = world_a.resource::<WorldConfig>();
        let config_b = world_b.resource::<WorldConfig>();
        assert_eq!(
            config_a.world_width, config_b.world_width,
            "config world_width mismatch"
        );
        assert_eq!(
            config_a.world_height, config_b.world_height,
            "config world_height mismatch"
        );
        assert_eq!(
            config_a.chunk_size, config_b.chunk_size,
            "config chunk_size mismatch"
        );

        // 5. Query and collect all chunk entities from both worlds
        let mut query_a = world_a.query::<(
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
            &EnergyAvailabilityChunk,
        )>();
        let mut chunks_a: Vec<_> = query_a
            .iter(world_a)
            .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
            .collect();
        chunks_a.sort_by_key(|c| (c.0.y, c.0.x));

        let mut query_b = world_b.query::<(
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
            &EnergyAvailabilityChunk,
        )>();
        let mut chunks_b: Vec<_> = query_b
            .iter(world_b)
            .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
            .collect();
        chunks_b.sort_by_key(|c| (c.0.y, c.0.x));

        assert_eq!(chunks_a.len(), chunks_b.len(), "chunk count mismatch");

        for (chunk_a, chunk_b) in chunks_a.iter().zip(chunks_b.iter()) {
            assert_eq!(chunk_a.0, chunk_b.0, "chunk coord mismatch");

            // Compare TerrainChunk fields
            let terrain_a = &chunk_a.1;
            let terrain_b = &chunk_b.1;
            assert_eq!(terrain_a.elevation.len(), terrain_b.elevation.len());
            for (va, vb) in terrain_a.elevation.iter().zip(terrain_b.elevation.iter()) {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "elevation mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
            assert_eq!(terrain_a.slope.len(), terrain_b.slope.len());
            for (va, vb) in terrain_a.slope.iter().zip(terrain_b.slope.iter()) {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "slope mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
            assert_eq!(terrain_a.water_depth.len(), terrain_b.water_depth.len());
            for (va, vb) in terrain_a
                .water_depth
                .iter()
                .zip(terrain_b.water_depth.iter())
            {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "water_depth mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
            assert_eq!(terrain_a.soil_depth.len(), terrain_b.soil_depth.len());
            for (va, vb) in terrain_a.soil_depth.iter().zip(terrain_b.soil_depth.iter()) {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "soil_depth mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
            assert_eq!(
                terrain_a.soil_fertility.len(),
                terrain_b.soil_fertility.len()
            );
            for (va, vb) in terrain_a
                .soil_fertility
                .iter()
                .zip(terrain_b.soil_fertility.iter())
            {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "soil_fertility mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }

            // Compare ClimateChunk fields
            let climate_a = &chunk_a.2;
            let climate_b = &chunk_b.2;
            assert_eq!(climate_a.temperature.len(), climate_b.temperature.len());
            for (va, vb) in climate_a
                .temperature
                .iter()
                .zip(climate_b.temperature.iter())
            {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "temperature mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
            assert_eq!(climate_a.moisture.len(), climate_b.moisture.len());
            for (va, vb) in climate_a.moisture.iter().zip(climate_b.moisture.iter()) {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "moisture mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
            assert_eq!(climate_a.rainfall.len(), climate_b.rainfall.len());
            for (va, vb) in climate_a.rainfall.iter().zip(climate_b.rainfall.iter()) {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "rainfall mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
            assert_eq!(
                climate_a.sunlight_factor.len(),
                climate_b.sunlight_factor.len()
            );
            for (va, vb) in climate_a
                .sunlight_factor
                .iter()
                .zip(climate_b.sunlight_factor.iter())
            {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "sunlight_factor mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }

            // Compare ResourceChunk fields
            let res_a = &chunk_a.3;
            let res_b = &chunk_b.3;
            assert_eq!(res_a.fresh_water.len(), res_b.fresh_water.len());
            for (va, vb) in res_a.fresh_water.iter().zip(res_b.fresh_water.iter()) {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "fresh_water mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
            assert_eq!(res_a.nutrients.len(), res_b.nutrients.len());
            for (va, vb) in res_a.nutrients.iter().zip(res_b.nutrients.iter()) {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "nutrients mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
            assert_eq!(res_a.minerals.len(), res_b.minerals.len());
            for (va, vb) in res_a.minerals.iter().zip(res_b.minerals.iter()) {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "minerals mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
            assert_eq!(res_a.biomass_potential.len(), res_b.biomass_potential.len());
            for (va, vb) in res_a
                .biomass_potential
                .iter()
                .zip(res_b.biomass_potential.iter())
            {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "biomass_potential mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }

            // Compare EnergyAvailabilityChunk fields
            let energy_a = &chunk_a.4;
            let energy_b = &chunk_b.4;
            assert_eq!(energy_a.solar_exposure.len(), energy_b.solar_exposure.len());
            for (va, vb) in energy_a
                .solar_exposure
                .iter()
                .zip(energy_b.solar_exposure.iter())
            {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "solar_exposure mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
            assert_eq!(
                energy_a.energy_availability.len(),
                energy_b.energy_availability.len()
            );
            for (va, vb) in energy_a
                .energy_availability
                .iter()
                .zip(energy_b.energy_availability.iter())
            {
                assert!(
                    (va - vb).abs() < 1e-5,
                    "energy_availability mismatch at {:?}: {} vs {}",
                    chunk_a.0,
                    va,
                    vb
                );
            }
        }
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
        let clock = world_split.resource::<SimulationClock>();
        let snapshot =
            build_world_snapshot(&config, &seed, clock, SNAPSHOT_SCHEMA_VERSION, &chunks);

        // Serialize and Deserialize to simulate save/load boundary
        let json = serde_json::to_string(&snapshot).expect("serialize failed");
        let deserialized: WorldSnapshot = serde_json::from_str(&json).expect("deserialize failed");

        // 3. Reconstruct world from deserialized snapshot in a new App
        let mut app_loaded = App::new(config.clone(), seed);
        // Do NOT run app_loaded.run_startup() since that would regenerate new terrain chunk entities.
        reconstruct_world_from_snapshot(app_loaded.world_mut(), deserialized);

        // Run validation on startup (from Milestone 8) before running simulation
        let mut startup_schedule = bevy_ecs::schedule::Schedule::default();
        startup_schedule.add_systems(crate::validation::systems::validate_world_on_startup);
        startup_schedule.run(app_loaded.world_mut());

        // Run remaining B = 20 ticks (A + B = 50)
        for _ in 0..20 {
            app_loaded.world_mut().run_schedule(FixedSimulationTick);
            app_loaded.world_mut().run_schedule(PostTickValidation);
            app_loaded.world_mut().run_schedule(PersistenceBoundary);
        }

        // 4. Assert equivalence
        assert_worlds_equivalent(world_continuous, app_loaded.world_mut());
    }
}

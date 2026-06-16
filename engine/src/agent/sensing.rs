//! Environmental sensing query API for biological agents.
//!
//! Provides read-only lookup functions to query nutrients and fresh water resources
//! from the environment for specific grid cells or neighborhoods.

use bevy_ecs::prelude::Query;
use bevy_ecs::query::QueryFilter;

use crate::config::WorldBounds;
use crate::world::coord::{world_to_chunk, world_to_local, WorldCoord};
use crate::world::resource::ResourceChunk;

/// Environmental resource values sensed at a single grid cell.
///
/// Milestone 12 only exposes nutrient and fresh water availability.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SensedResource {
    /// Nutrient availability in range [0.0, nutrients_max].
    pub nutrients: f32,
    /// Fresh water availability in range [0.0, fresh_water_max].
    pub fresh_water: f32,
}

/// Queries the environmental resources at a single world coordinate.
///
/// Returns `None` if the coordinate is out of bounds or if the containing chunk is not found.
pub fn query_cell<F: QueryFilter>(
    coord: WorldCoord,
    bounds: &WorldBounds,
    spatial_map: &crate::world::spatial::SpatialMap,
    chunks: &Query<&ResourceChunk, F>,
) -> Option<SensedResource> {
    if !bounds.contains_world_coord(coord) {
        return None;
    }

    let target_chunk = world_to_chunk(coord, bounds.chunk_size);
    if let Some(entity) = spatial_map.get(target_chunk) {
        if let Ok(resource_chunk) = chunks.get(entity) {
            let local_coord = world_to_local(coord, bounds.chunk_size);
            let index = (local_coord.y * bounds.chunk_size + local_coord.x) as usize;
            return Some(SensedResource {
                nutrients: resource_chunk.nutrients[index],
                fresh_water: resource_chunk.fresh_water[index],
            });
        }
    }

    None
}

/// Queries environmental resources in an 8-neighbor square neighborhood around the center coordinate.
///
/// Uses `query_cell` internally and returns all valid, in-bounds cells within `sensing_radius`
/// ordered deterministically in row-major order.
pub fn query_neighborhood<F: QueryFilter>(
    center: WorldCoord,
    radius: u32,
    bounds: &WorldBounds,
    spatial_map: &crate::world::spatial::SpatialMap,
    chunks: &Query<&ResourceChunk, F>,
) -> Vec<(WorldCoord, SensedResource)> {
    let mut results = Vec::new();

    // Determine query window bounds safely using saturating/checked arithmetic
    let min_x = center.x.saturating_sub(radius);
    let max_x = center
        .x
        .saturating_add(radius)
        .min(bounds.world_width.saturating_sub(1));
    let min_y = center.y.saturating_sub(radius);
    let max_y = center
        .y
        .saturating_add(radius)
        .min(bounds.world_height.saturating_sub(1));

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let coord = WorldCoord::new(x, y);
            if let Some(resource) = query_cell(coord, bounds, spatial_map, chunks) {
                results.push((coord, resource));
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::create_test_config;
    use crate::world::coord::ChunkCoord;
    use bevy_ecs::prelude::*;

    #[derive(Resource)]
    struct TestContext {
        bounds: WorldBounds,
        results_cell_00: Option<SensedResource>,
        results_cell_32: Option<SensedResource>,
        results_oob: Option<SensedResource>,
        results_neighborhood_1: Vec<(WorldCoord, SensedResource)>,
        results_neighborhood_oob: Vec<(WorldCoord, SensedResource)>,
    }

    fn test_sensing_system(
        chunks: Query<&ResourceChunk>,
        spatial_map: Res<crate::world::spatial::SpatialMap>,
        mut context: ResMut<TestContext>,
    ) {
        // Test single cell lookup inside bounds
        context.results_cell_00 = query_cell(
            WorldCoord::new(0, 0),
            &context.bounds,
            &spatial_map,
            &chunks,
        );
        context.results_cell_32 = query_cell(
            WorldCoord::new(32, 0),
            &context.bounds,
            &spatial_map,
            &chunks,
        );

        // Test single cell lookup out of bounds
        context.results_oob = query_cell(
            WorldCoord::new(256, 256),
            &context.bounds,
            &spatial_map,
            &chunks,
        );

        // Test neighborhood query (radius 1) centered at (1, 1) -> 9 cells
        context.results_neighborhood_1 = query_neighborhood(
            WorldCoord::new(1, 1),
            1,
            &context.bounds,
            &spatial_map,
            &chunks,
        );

        // Test neighborhood query (radius 1) centered at world boundary (0, 0) -> should clamp/contain 4 cells
        context.results_neighborhood_oob = query_neighborhood(
            WorldCoord::new(0, 0),
            1,
            &context.bounds,
            &spatial_map,
            &chunks,
        );
    }

    #[test]
    fn test_environmental_sensing() {
        let mut world = World::new();
        let config = create_test_config(); // world_width = 256, world_height = 256, chunk_size = 32
        let bounds = WorldBounds::from_config(&config);

        let chunk_size = bounds.chunk_size;
        let expected_len = (chunk_size * chunk_size) as usize;

        // Chunk (0, 0): fresh_water = 0.5, nutrients = 0.7
        let rc_00 = ResourceChunk {
            fresh_water: vec![0.5; expected_len],
            nutrients: vec![0.7; expected_len],
            minerals: vec![0.0; expected_len],
            biomass_potential: vec![0.0; expected_len],
        };

        // Chunk (1, 0): fresh_water = 0.1, nutrients = 0.2, except local (0, 0) i.e. world (32, 0)
        let mut fresh_water_10 = vec![0.1; expected_len];
        let mut nutrients_10 = vec![0.2; expected_len];
        fresh_water_10[0] = 0.85;
        nutrients_10[0] = 0.95;

        let rc_10 = ResourceChunk {
            fresh_water: fresh_water_10,
            nutrients: nutrients_10,
            minerals: vec![0.0; expected_len],
            biomass_potential: vec![0.0; expected_len],
        };

        let mut spatial_map =
            crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);
        let e1 = world.spawn((ChunkCoord::new(0, 0), rc_00)).id();
        let e2 = world.spawn((ChunkCoord::new(1, 0), rc_10)).id();
        spatial_map.set(ChunkCoord::new(0, 0), e1);
        spatial_map.set(ChunkCoord::new(1, 0), e2);
        world.insert_resource(spatial_map);

        world.insert_resource(TestContext {
            bounds,
            results_cell_00: None,
            results_cell_32: None,
            results_oob: None,
            results_neighborhood_1: Vec::new(),
            results_neighborhood_oob: Vec::new(),
        });

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(test_sensing_system);
        world.add_schedule(schedule);
        world.run_schedule(crate::app::FixedSimulationTick);

        let context = world.resource::<TestContext>();

        // Verify resource values exactly match ResourceChunk storage
        let r00 = context.results_cell_00.expect("Expected result for (0, 0)");
        assert_eq!(r00.fresh_water, 0.5);
        assert_eq!(r00.nutrients, 0.7);

        // Verify coordinate mapping and index translation works across chunks
        let r32 = context
            .results_cell_32
            .expect("Expected result for (32, 0)");
        assert_eq!(r32.fresh_water, 0.85);
        assert_eq!(r32.nutrients, 0.95);

        // Verify out of bounds coordinate lookup fails safely
        assert!(context.results_oob.is_none());

        // Verify neighborhood query retrieves correct count (radius 1 centered at (1, 1) -> 3x3 grid = 9 cells)
        assert_eq!(context.results_neighborhood_1.len(), 9);
        for &(coord, resource) in &context.results_neighborhood_1 {
            assert!(coord.x <= 2);
            assert!(coord.y <= 2);
            assert_eq!(resource.fresh_water, 0.5);
            assert_eq!(resource.nutrients, 0.7);
        }

        // Verify neighborhood query clamps at boundaries (radius 1 centered at (0, 0) -> x in [0, 1], y in [0, 1] = 4 cells)
        assert_eq!(context.results_neighborhood_oob.len(), 4);
        let coords_oob: Vec<WorldCoord> = context
            .results_neighborhood_oob
            .iter()
            .map(|(c, _)| *c)
            .collect();
        assert!(coords_oob.contains(&WorldCoord::new(0, 0)));
        assert!(coords_oob.contains(&WorldCoord::new(1, 0)));
        assert!(coords_oob.contains(&WorldCoord::new(0, 1)));
        assert!(coords_oob.contains(&WorldCoord::new(1, 1)));
    }

    #[test]
    fn test_determinism() {
        let mut world = World::new();
        let config = create_test_config();
        let bounds = WorldBounds::from_config(&config);

        let chunk_size = bounds.chunk_size;
        let expected_len = (chunk_size * chunk_size) as usize;

        let rc = ResourceChunk {
            fresh_water: vec![0.333; expected_len],
            nutrients: vec![0.666; expected_len],
            minerals: vec![0.0; expected_len],
            biomass_potential: vec![0.0; expected_len],
        };

        let mut spatial_map1 =
            crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);
        let e1 = world.spawn((ChunkCoord::new(0, 0), rc)).id();
        spatial_map1.set(ChunkCoord::new(0, 0), e1);
        world.insert_resource(spatial_map1);

        world.insert_resource(TestContext {
            bounds: bounds.clone(),
            results_cell_00: None,
            results_cell_32: None,
            results_oob: None,
            results_neighborhood_1: Vec::new(),
            results_neighborhood_oob: Vec::new(),
        });

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(test_sensing_system);
        world.add_schedule(schedule);
        world.run_schedule(crate::app::FixedSimulationTick);

        let context = world.resource::<TestContext>();

        // Run second time with identical inputs
        let mut world_b = World::new();
        let rc_b = ResourceChunk {
            fresh_water: vec![0.333; expected_len],
            nutrients: vec![0.666; expected_len],
            minerals: vec![0.0; expected_len],
            biomass_potential: vec![0.0; expected_len],
        };
        let mut spatial_map2 =
            crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);
        let e2 = world_b.spawn((ChunkCoord::new(0, 0), rc_b)).id();
        spatial_map2.set(ChunkCoord::new(0, 0), e2);
        world_b.insert_resource(spatial_map2);
        world_b.insert_resource(TestContext {
            bounds: bounds.clone(),
            results_cell_00: None,
            results_cell_32: None,
            results_oob: None,
            results_neighborhood_1: Vec::new(),
            results_neighborhood_oob: Vec::new(),
        });
        let mut schedule_b = Schedule::new(crate::app::FixedSimulationTick);
        schedule_b.add_systems(test_sensing_system);
        world_b.add_schedule(schedule_b);
        world_b.run_schedule(crate::app::FixedSimulationTick);

        let context_b = world_b.resource::<TestContext>();

        // Verify bit-perfect identity
        assert_eq!(context.results_cell_00, context_b.results_cell_00);
        assert_eq!(
            context.results_neighborhood_1,
            context_b.results_neighborhood_1
        );
        assert_eq!(
            context.results_neighborhood_oob,
            context_b.results_neighborhood_oob
        );
    }

    #[derive(Resource)]
    struct StrongDeterminismContext {
        neighborhood: Vec<(WorldCoord, SensedResource)>,
    }

    fn strong_determinism_system(
        chunks: Query<&ResourceChunk>,
        bounds: Res<WorldBounds>,
        spatial_map: Res<crate::world::spatial::SpatialMap>,
        mut context: ResMut<StrongDeterminismContext>,
    ) {
        context.neighborhood =
            query_neighborhood(WorldCoord::new(31, 31), 2, &bounds, &spatial_map, &chunks);
    }

    #[test]
    fn test_sensing_strong_determinism() {
        let config = create_test_config();
        let chunk_size = config.chunk_size;
        let expected_len = (chunk_size * chunk_size) as usize;

        let rc_00 = ResourceChunk {
            fresh_water: vec![0.1; expected_len],
            nutrients: vec![0.2; expected_len],
            minerals: vec![0.0; expected_len],
            biomass_potential: vec![0.0; expected_len],
        };
        let rc_10 = ResourceChunk {
            fresh_water: vec![0.3; expected_len],
            nutrients: vec![0.4; expected_len],
            minerals: vec![0.0; expected_len],
            biomass_potential: vec![0.0; expected_len],
        };
        let rc_01 = ResourceChunk {
            fresh_water: vec![0.5; expected_len],
            nutrients: vec![0.6; expected_len],
            minerals: vec![0.0; expected_len],
            biomass_potential: vec![0.0; expected_len],
        };
        let rc_11 = ResourceChunk {
            fresh_water: vec![0.7; expected_len],
            nutrients: vec![0.8; expected_len],
            minerals: vec![0.0; expected_len],
            biomass_potential: vec![0.0; expected_len],
        };

        // World A: Spawning chunks in order (0,0), (1,0), (0,1), (1,1)
        let mut world_a = World::new();
        let bounds_a = WorldBounds::from_config(&config);
        world_a.insert_resource(bounds_a.clone());
        world_a.insert_resource(StrongDeterminismContext {
            neighborhood: Vec::new(),
        });
        let mut spatial_map_a =
            crate::world::spatial::SpatialMap::new(bounds_a.chunks_x, bounds_a.chunks_y);
        spatial_map_a.set(
            ChunkCoord::new(0, 0),
            world_a.spawn((ChunkCoord::new(0, 0), rc_00.clone())).id(),
        );
        spatial_map_a.set(
            ChunkCoord::new(1, 0),
            world_a.spawn((ChunkCoord::new(1, 0), rc_10.clone())).id(),
        );
        spatial_map_a.set(
            ChunkCoord::new(0, 1),
            world_a.spawn((ChunkCoord::new(0, 1), rc_01.clone())).id(),
        );
        spatial_map_a.set(
            ChunkCoord::new(1, 1),
            world_a.spawn((ChunkCoord::new(1, 1), rc_11.clone())).id(),
        );
        world_a.insert_resource(spatial_map_a);

        let mut schedule_a = Schedule::new(crate::app::FixedSimulationTick);
        schedule_a.add_systems(strong_determinism_system);
        world_a.add_schedule(schedule_a);
        world_a.run_schedule(crate::app::FixedSimulationTick);
        let results_a = world_a
            .resource::<StrongDeterminismContext>()
            .neighborhood
            .clone();

        // World B: Spawning chunks in order (1,1), (0,1), (1,0), (0,0) (completely reversed)
        let mut world_b = World::new();
        let bounds_b = WorldBounds::from_config(&config);
        world_b.insert_resource(bounds_b.clone());
        world_b.insert_resource(StrongDeterminismContext {
            neighborhood: Vec::new(),
        });
        let mut spatial_map_b =
            crate::world::spatial::SpatialMap::new(bounds_b.chunks_x, bounds_b.chunks_y);
        spatial_map_b.set(
            ChunkCoord::new(1, 1),
            world_b.spawn((ChunkCoord::new(1, 1), rc_11)).id(),
        );
        spatial_map_b.set(
            ChunkCoord::new(0, 1),
            world_b.spawn((ChunkCoord::new(0, 1), rc_01)).id(),
        );
        spatial_map_b.set(
            ChunkCoord::new(1, 0),
            world_b.spawn((ChunkCoord::new(1, 0), rc_10)).id(),
        );
        spatial_map_b.set(
            ChunkCoord::new(0, 0),
            world_b.spawn((ChunkCoord::new(0, 0), rc_00)).id(),
        );
        world_b.insert_resource(spatial_map_b);

        let mut schedule_b = Schedule::new(crate::app::FixedSimulationTick);
        schedule_b.add_systems(strong_determinism_system);
        world_b.add_schedule(schedule_b);
        world_b.run_schedule(crate::app::FixedSimulationTick);
        let results_b = world_b
            .resource::<StrongDeterminismContext>()
            .neighborhood
            .clone();

        // 1. Verify entity spawn order independence: output results must be bit-perfect identical
        assert_eq!(results_a, results_b);

        // 2. Verify neighborhood ordering stability: output results must be sorted deterministically in row-major order
        for i in 0..results_a.len().saturating_sub(1) {
            let coord_curr = results_a[i].0;
            let coord_next = results_a[i + 1].0;
            assert!(
                coord_curr.y < coord_next.y || (coord_curr.y == coord_next.y && coord_curr.x < coord_next.x),
                "Neighborhood output sorting is not stable row-major: index {} at {:?} vs index {} at {:?}",
                i, coord_curr, i + 1, coord_next
            );
        }
    }
}

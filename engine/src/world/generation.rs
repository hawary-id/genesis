//! ECS systems for deterministic world generation and validation.

use bevy_ecs::prelude::*;

use crate::app::WorldGenerationCompleted;
use crate::config::{WorldBounds, WorldConfig};
use crate::rng::WorldSeed;
use crate::world::climate::ClimateChunk;
use crate::world::coord::ChunkCoord;
use crate::world::energy::EnergyAvailabilityChunk;
use crate::world::resource::ResourceChunk;
use crate::world::terrain::TerrainChunk;

/// Marker component indicating that a chunk has completed its initial generation
/// and passed post-generation validation.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Generated;

/// Verifies that the world configuration parameters satisfy essential invariants.
///
/// # Panics
///
/// Panics if the configuration is invalid (e.g. non-divisible dimensions, chunk size is 0,
/// or non-positive day length).
pub fn validate_world_config(config: Res<WorldConfig>) {
    assert!(
        config.chunk_size > 0,
        "WorldConfig: chunk_size must be greater than zero"
    );
    assert!(
        config.world_width > 0,
        "WorldConfig: world_width must be greater than zero"
    );
    assert!(
        config.world_height > 0,
        "WorldConfig: world_height must be greater than zero"
    );
    assert!(
        config.world_width % config.chunk_size == 0,
        "WorldConfig: world_width ({}) must be perfectly divisible by chunk_size ({})",
        config.world_width,
        config.chunk_size
    );
    assert!(
        config.world_height % config.chunk_size == 0,
        "WorldConfig: world_height ({}) must be perfectly divisible by chunk_size ({})",
        config.world_height,
        config.chunk_size
    );
    assert!(
        config.day_length_ticks > 0,
        "WorldConfig: day_length_ticks must be greater than zero"
    );
}

/// Spawns chunk entities covering the entire map grid coordinates.
///
/// Each spawned entity receives a [`ChunkCoord`] component representing its location.
pub fn spawn_chunk_entities(mut commands: Commands, bounds: Res<WorldBounds>) {
    for cy in 0..bounds.chunks_y {
        for cx in 0..bounds.chunks_x {
            commands.spawn(ChunkCoord::new(cx, cy));
        }
    }
}

/// Generates terrain fields for all spawned chunk entities.
///
/// Iterates over entities carrying a [`ChunkCoord`] and attaches a generated
/// [`TerrainChunk`] component containing the terrain field vectors.
pub fn generate_terrain_chunks(
    mut commands: Commands,
    config: Res<WorldConfig>,
    seed: Res<WorldSeed>,
    query: Query<(Entity, &ChunkCoord), Without<TerrainChunk>>,
) {
    let terrain_seed = crate::rng::derive_terrain_seed(seed.root_seed);
    for (entity, coord) in &query {
        let chunk = crate::world::terrain::generate_terrain_chunk(*coord, terrain_seed, &config);
        commands.entity(entity).insert(chunk);
    }
}

/// Generates climate fields for all chunk entities populated with terrain data.
///
/// Iterates over entities carrying terrain information and attaches the derived
/// [`ClimateChunk`] component containing the base climate conditions.
pub fn generate_climate_chunks(
    mut commands: Commands,
    config: Res<WorldConfig>,
    query: Query<(Entity, &ChunkCoord, &TerrainChunk), Without<ClimateChunk>>,
) {
    let seasonal_modifier = crate::world::climate::calculate_seasonal_modifier(0, &config);
    let chunk_size = config.chunk_size;
    let world_height = config.world_height;

    for (entity, coord, terrain) in &query {
        let n = (chunk_size * chunk_size) as usize;
        let mut temperature = vec![0.0f32; n];
        let mut moisture = vec![0.0f32; n];
        let mut rainfall = vec![0.0f32; n];
        let mut sunlight_factor = vec![0.0f32; n];

        for idx in 0..n {
            let ly = (idx as u32) / chunk_size;
            let gy = coord.y * chunk_size + ly;

            let sf = crate::world::climate::calculate_sunlight_factor(gy, world_height);
            sunlight_factor[idx] = sf;

            let elev = terrain.elevation[idx];
            let temp =
                crate::world::climate::calculate_temperature(elev, sf, seasonal_modifier, &config);
            temperature[idx] = temp;

            let wd = terrain.water_depth[idx];
            let moist = crate::world::climate::calculate_moisture(wd, elev, &config);
            moisture[idx] = moist;

            let rain = crate::world::climate::calculate_rainfall(moist, temp, &config);
            rainfall[idx] = rain;
        }

        commands.entity(entity).insert(ClimateChunk {
            temperature,
            moisture,
            rainfall,
            sunlight_factor,
        });
    }
}

/// Generates resource fields for all chunk entities populated with terrain and climate data.
///
/// Iterates over entities carrying terrain and climate information and attaches the derived
/// [`ResourceChunk`] component containing the base environmental materials.
pub fn generate_resource_chunks(
    mut commands: Commands,
    config: Res<WorldConfig>,
    seed: Res<WorldSeed>,
    query: Query<(Entity, &ChunkCoord, &TerrainChunk, &ClimateChunk), Without<ResourceChunk>>,
) {
    let resource_seed = crate::rng::derive_resource_seed(seed.root_seed);
    let chunk_size = config.chunk_size;

    for (entity, coord, terrain, climate) in &query {
        let n = (chunk_size * chunk_size) as usize;
        let mut fresh_water = vec![0.0f32; n];
        let mut nutrients = vec![0.0f32; n];
        let mut minerals = vec![0.0f32; n];
        let mut biomass_potential = vec![0.0f32; n];

        for idx in 0..n {
            let lx = (idx as u32) % chunk_size;
            let ly = (idx as u32) / chunk_size;

            let gx = coord.x * chunk_size + lx;
            let gy = coord.y * chunk_size + ly;

            // Minerals noise
            let min_val = crate::world::resource::value_noise_sample(
                gx,
                gy,
                resource_seed,
                16,
                0.0,
                config.minerals_max,
            );
            minerals[idx] = min_val;

            // Fresh Water: terrain water_depth, climate moisture/rainfall
            let wd = terrain.water_depth[idx];
            let moist = climate.moisture[idx];
            let norm_wd = wd / config.water_depth_max.max(f32::EPSILON);
            let fw = (norm_wd * 0.8 + moist * 0.2).clamp(0.0, 1.0) * config.fresh_water_max;
            fresh_water[idx] = fw;

            // Nutrients: soil fertility and soil depth
            let sd = terrain.soil_depth[idx];
            let sf = terrain.soil_fertility[idx];
            let norm_sd = sd / config.soil_depth_max.max(f32::EPSILON);
            let norm_sf = sf / config.soil_fertility_max.max(f32::EPSILON);
            let nut = (norm_sd * norm_sf).clamp(0.0, 1.0) * config.nutrients_max;
            nutrients[idx] = nut;

            // Biomass carrying potential: derived from temp, moist, and soil nutrients
            let temp = climate.temperature[idx];
            let norm_nut = nut / config.nutrients_max.max(f32::EPSILON);
            let bp = (temp * moist * norm_nut).clamp(0.0, 1.0) * config.biomass_potential_max;
            biomass_potential[idx] = bp;
        }

        commands.entity(entity).insert(ResourceChunk {
            fresh_water,
            nutrients,
            minerals,
            biomass_potential,
        });
    }
}

/// Validates that all generated terrain, climate, and resource field values reside within configured bounds.
///
/// # Panics
///
/// Panics if any cell values violate `WorldConfig` ranges.
pub fn validate_generated_world(
    config: Res<WorldConfig>,
    query: Query<(
        &TerrainChunk,
        &ClimateChunk,
        &ResourceChunk,
        &EnergyAvailabilityChunk,
    )>,
) {
    let chunk_size = config.chunk_size as usize;
    let expected_len = chunk_size * chunk_size;

    for (terrain, climate, resource, energy) in &query {
        // Terrain validations
        assert_eq!(
            terrain.elevation.len(),
            expected_len,
            "Terrain validation: elevation array length mismatch"
        );
        assert_eq!(
            terrain.slope.len(),
            expected_len,
            "Terrain validation: slope array length mismatch"
        );
        assert_eq!(
            terrain.water_depth.len(),
            expected_len,
            "Terrain validation: water_depth array length mismatch"
        );
        assert_eq!(
            terrain.soil_depth.len(),
            expected_len,
            "Terrain validation: soil_depth array length mismatch"
        );
        assert_eq!(
            terrain.soil_fertility.len(),
            expected_len,
            "Terrain validation: soil_fertility array length mismatch"
        );

        for &val in &terrain.elevation {
            assert!(
                val >= config.elevation_min && val <= config.elevation_max,
                "Terrain validation: elevation value {} out of configured bounds [{}, {}]",
                val,
                config.elevation_min,
                config.elevation_max
            );
        }

        for &val in &terrain.slope {
            assert!(
                val >= 0.0 && val <= config.slope_max,
                "Terrain validation: slope value {} out of configured bounds [0.0, {}]",
                val,
                config.slope_max
            );
        }

        for &val in &terrain.water_depth {
            assert!(
                val >= 0.0 && val <= config.water_depth_max,
                "Terrain validation: water_depth value {} out of configured bounds [0.0, {}]",
                val,
                config.water_depth_max
            );
        }

        for &val in &terrain.soil_depth {
            assert!(
                val >= 0.0 && val <= config.soil_depth_max,
                "Terrain validation: soil_depth value {} out of configured bounds [0.0, {}]",
                val,
                config.soil_depth_max
            );
        }

        for &val in &terrain.soil_fertility {
            assert!(
                val >= 0.0 && val <= config.soil_fertility_max,
                "Terrain validation: soil_fertility value {} out of configured bounds [0.0, {}]",
                val,
                config.soil_fertility_max
            );
        }

        // Climate validations
        assert_eq!(
            climate.temperature.len(),
            expected_len,
            "Climate validation: temperature array length mismatch"
        );
        assert_eq!(
            climate.moisture.len(),
            expected_len,
            "Climate validation: moisture array length mismatch"
        );
        assert_eq!(
            climate.rainfall.len(),
            expected_len,
            "Climate validation: rainfall array length mismatch"
        );
        assert_eq!(
            climate.sunlight_factor.len(),
            expected_len,
            "Climate validation: sunlight_factor array length mismatch"
        );

        for &val in &climate.temperature {
            assert!(
                val >= config.temperature_min && val <= config.temperature_max,
                "Climate validation: temperature value {} out of configured bounds [{}, {}]",
                val,
                config.temperature_min,
                config.temperature_max
            );
        }

        for &val in &climate.moisture {
            assert!(
                val >= config.moisture_min && val <= config.moisture_max,
                "Climate validation: moisture value {} out of configured bounds [{}, {}]",
                val,
                config.moisture_min,
                config.moisture_max
            );
        }

        for &val in &climate.rainfall {
            assert!(
                val >= config.rainfall_min && val <= config.rainfall_max,
                "Climate validation: rainfall value {} out of configured bounds [{}, {}]",
                val,
                config.rainfall_min,
                config.rainfall_max
            );
        }

        for &val in &climate.sunlight_factor {
            assert!(
                val >= config.sunlight_factor_min && val <= config.sunlight_factor_max,
                "Climate validation: sunlight_factor value {} out of configured bounds [{}, {}]",
                val,
                config.sunlight_factor_min,
                config.sunlight_factor_max
            );
        }

        // Resource validations
        assert_eq!(
            resource.fresh_water.len(),
            expected_len,
            "Resource validation: fresh_water array length mismatch"
        );
        assert_eq!(
            resource.nutrients.len(),
            expected_len,
            "Resource validation: nutrients array length mismatch"
        );
        assert_eq!(
            resource.minerals.len(),
            expected_len,
            "Resource validation: minerals array length mismatch"
        );
        assert_eq!(
            resource.biomass_potential.len(),
            expected_len,
            "Resource validation: biomass_potential array length mismatch"
        );

        for &val in &resource.fresh_water {
            assert!(
                val >= 0.0 && val <= config.fresh_water_max,
                "Resource validation: fresh_water value {} out of configured bounds [0.0, {}]",
                val,
                config.fresh_water_max
            );
        }

        for &val in &resource.nutrients {
            assert!(
                val >= 0.0 && val <= config.nutrients_max,
                "Resource validation: nutrients value {} out of configured bounds [0.0, {}]",
                val,
                config.nutrients_max
            );
        }

        for &val in &resource.minerals {
            assert!(
                val >= 0.0 && val <= config.minerals_max,
                "Resource validation: minerals value {} out of configured bounds [0.0, {}]",
                val,
                config.minerals_max
            );
        }

        for &val in &resource.biomass_potential {
            assert!(
                val >= 0.0 && val <= config.biomass_potential_max,
                "Resource validation: biomass_potential value {} out of configured bounds [0.0, {}]",
                val,
                config.biomass_potential_max
            );
        }

        // Energy validations
        assert_eq!(
            energy.solar_exposure.len(),
            expected_len,
            "Energy validation: solar_exposure array length mismatch"
        );
        assert_eq!(
            energy.energy_availability.len(),
            expected_len,
            "Energy validation: energy_availability array length mismatch"
        );

        for &val in &energy.solar_exposure {
            assert!(
                val >= 0.0 && val <= config.solar_exposure_max,
                "Energy validation: solar_exposure value {} out of configured bounds [0.0, {}]",
                val,
                config.solar_exposure_max
            );
        }

        for &val in &energy.energy_availability {
            assert!(
                val >= 0.0 && val <= config.energy_availability_max,
                "Energy validation: energy_availability value {} out of configured bounds [0.0, {}]",
                val,
                config.energy_availability_max
            );
        }
    }
}

/// Marks chunk entities as generated.
///
/// Attaches the [`Generated`] component to chunk entities containing terrain, climate, resource, and energy data.
pub fn mark_chunks_generated(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            With<ChunkCoord>,
            With<TerrainChunk>,
            With<ClimateChunk>,
            With<ResourceChunk>,
            With<EnergyAvailabilityChunk>,
            Without<Generated>,
        ),
    >,
) {
    for entity in &query {
        commands.entity(entity).insert(Generated);
    }
}

/// Emits the [`WorldGenerationCompleted`] event indicating simulation start readiness.
pub fn emit_world_generation_completed(mut writer: EventWriter<WorldGenerationCompleted>) {
    writer.send(WorldGenerationCompleted);
}

/// Registers the startup generation systems to the `StartupGeneration` schedule.
pub fn register_generation_systems(world: &mut World) {
    use crate::app::StartupGeneration;
    use bevy_ecs::schedule::Schedules;

    let mut schedules = world.resource_mut::<Schedules>();
    if let Some(schedule) = schedules.get_mut(StartupGeneration) {
        schedule.add_systems((
            validate_world_config,
            spawn_chunk_entities.after(validate_world_config),
            generate_terrain_chunks.after(spawn_chunk_entities),
            generate_climate_chunks.after(generate_terrain_chunks),
            generate_resource_chunks.after(generate_climate_chunks),
            crate::world::energy::generate_energy_availability_chunks
                .after(generate_resource_chunks),
            validate_generated_world
                .after(crate::world::energy::generate_energy_availability_chunks),
            mark_chunks_generated.after(validate_generated_world),
            emit_world_generation_completed.after(mark_chunks_generated),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::schedules::register_schedules;
    use crate::config::WorldBounds;

    fn test_world() -> World {
        let mut world = World::new();
        let config = WorldConfig {
            world_width: 64,
            world_height: 64,
            chunk_size: 16,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        let seed = WorldSeed::new(12345);

        world.insert_resource(config);
        world.insert_resource(bounds);
        world.insert_resource(seed);
        world.init_resource::<Events<WorldGenerationCompleted>>();

        register_schedules(&mut world);
        register_generation_systems(&mut world);

        world
    }

    #[test]
    fn full_generation_flow_success() {
        let mut world = test_world();

        // Execute the StartupGeneration schedule
        world.run_schedule(crate::app::StartupGeneration);

        // Check chunk count: bounds.chunks_x = 4, chunks_y = 4 => 16 chunks
        let mut query = world.query::<(
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
            &EnergyAvailabilityChunk,
            &Generated,
        )>();
        let chunks: Vec<_> = query.iter(&world).collect();
        assert_eq!(chunks.len(), 16);

        // Verify that the WorldGenerationCompleted event was sent exactly once
        let events = world.resource::<Events<WorldGenerationCompleted>>();
        let mut reader = events.get_reader();
        let event_count = reader.read(events).count();
        assert_eq!(event_count, 1);
    }

    #[test]
    #[should_panic(
        expected = "WorldConfig: world_width (64) must be perfectly divisible by chunk_size (10)"
    )]
    fn invalid_config_panics() {
        let mut world = World::new();
        let config = WorldConfig {
            world_width: 64,
            world_height: 64,
            chunk_size: 10, // 64 is not divisible by 10
            ..WorldConfig::default()
        };
        world.insert_resource(config);

        let mut schedule = Schedule::new(crate::app::StartupGeneration);
        schedule.add_systems(validate_world_config);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::StartupGeneration);
    }
}

//! Resource component, deterministic generation, and updates for the Genesis world simulation.

use bevy_ecs::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::config::WorldConfig;
use crate::time::SimulationClock;
use crate::world::climate::ClimateChunk;
use crate::world::coord::ChunkCoord;
use crate::world::terrain::TerrainChunk;

/// Material environmental resource fields for all cells in one chunk.
///
/// Each field is a `Vec<f32>` of length `chunk_size * chunk_size`.
/// Cell index: `local_y * chunk_size + local_x`.
///
/// All values must remain within the range `[0.0, config.*_max]`.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ResourceChunk {
    /// Fresh water availability. Range: `[0.0, fresh_water_max]`.
    pub fresh_water: Vec<f32>,

    /// Nutrient availability. Range: `[0.0, nutrients_max]`.
    pub nutrients: Vec<f32>,

    /// Mineral concentration. Range: `[0.0, minerals_max]`.
    pub minerals: Vec<f32>,

    /// Biomass carrying potential. Range: `[0.0, biomass_potential_max]`.
    pub biomass_potential: Vec<f32>,
}

// ---------------------------------------------------------------------------
// Value Noise sampling functions for Minerals
// ---------------------------------------------------------------------------

/// Samples a value-noise field at global coordinate `(gx, gy)`.
///
/// Divides the world into a lattice with spacing `scale`. Each lattice point
/// is seeded deterministically and produces a height. Cell values are
/// bilinearly interpolated from the four surrounding lattice points.
pub fn value_noise_sample(
    gx: u32,
    gy: u32,
    seed: u64,
    scale: u32,
    min_val: f32,
    max_val: f32,
) -> f32 {
    if scale == 0 {
        return min_val;
    }
    // Lattice coordinates.
    let lx0 = gx / scale;
    let ly0 = gy / scale;
    let lx1 = lx0 + 1;
    let ly1 = ly0 + 1;

    // Fractional position within the lattice cell.
    let fx = (gx % scale) as f32 / scale as f32;
    let fy = (gy % scale) as f32 / scale as f32;

    // Values at the four surrounding lattice points.
    let h00 = lattice_value(lx0, ly0, seed, min_val, max_val);
    let h10 = lattice_value(lx1, ly0, seed, min_val, max_val);
    let h01 = lattice_value(lx0, ly1, seed, min_val, max_val);
    let h11 = lattice_value(lx1, ly1, seed, min_val, max_val);

    // Bilinear interpolation.
    let h0 = h00 + fx * (h10 - h00);
    let h1 = h01 + fx * (h11 - h01);
    h0 + fy * (h1 - h0)
}

fn lattice_value(lx: u32, ly: u32, seed: u64, min_val: f32, max_val: f32) -> f32 {
    let point_seed = crate::rng::derive_chunk_seed(seed, lx, ly);
    let mut rng = ChaCha8Rng::seed_from_u64(point_seed);
    let t: f32 = rng.gen(); // uniform [0.0, 1.0)
    min_val + t * (max_val - min_val)
}

// ---------------------------------------------------------------------------
// ECS Update and Validation Systems
// ---------------------------------------------------------------------------

/// Recalculates resource fields for all chunks once per simulation day.
///
/// Checks clock ticks to enforce daily updates, returning early otherwise.
pub fn update_resource_fields(
    clock: Res<SimulationClock>,
    config: Res<WorldConfig>,
    mut query: Query<(
        &ChunkCoord,
        &TerrainChunk,
        &ClimateChunk,
        &mut ResourceChunk,
    )>,
) {
    // Perform updates only on daily ticks
    if clock.total_ticks % config.day_length_ticks != 0 {
        return;
    }

    let chunk_size = config.chunk_size;

    for (_chunk_coord, terrain, climate, mut resource) in &mut query {
        let n = (chunk_size * chunk_size) as usize;

        for idx in 0..n {
            let slope = terrain.slope[idx];
            let rainfall = climate.rainfall[idx];
            let temperature = climate.temperature[idx];
            let moisture = climate.moisture[idx];
            let soil_fertility = terrain.soil_fertility[idx];

            // 1. Water update: increases with rainfall, decays due to temperature-evaporation and slope-runoff
            let fw = resource.fresh_water[idx];
            let evap = temperature * 0.1 * fw;
            let runoff = slope * 0.05 * fw;
            let rain_add = rainfall * 0.2;
            let new_fw = (fw + rain_add - evap - runoff).clamp(0.0, config.fresh_water_max);
            resource.fresh_water[idx] = new_fw;

            // 2. Nutrient update: depletion via leaching runoff and slow replenishment based on soil fertility and temperature
            let nut = resource.nutrients[idx];
            let leaching = slope * rainfall * 0.05 * nut;
            let replenishment = soil_fertility * temperature * 0.01;
            let new_nut = (nut - leaching + replenishment).clamp(0.0, config.nutrients_max);
            resource.nutrients[idx] = new_nut;

            // 3. Minerals remain static

            // 4. Biomass potential update: recalculated dynamically based on climate/resource parameters
            let norm_nut = new_nut / config.nutrients_max.max(f32::EPSILON);
            let bp =
                (temperature * moisture * norm_nut).clamp(0.0, 1.0) * config.biomass_potential_max;
            resource.biomass_potential[idx] = bp;
        }
    }
}

use crate::validation::ValidationError;

/// Validates all cell values inside the resource chunk against limits specified in [`WorldConfig`].
pub fn validate_resource_chunk(
    coord: &ChunkCoord,
    resource: &ResourceChunk,
    config: &WorldConfig,
) -> Result<(), ValidationError> {
    let chunk_size = config.chunk_size as usize;
    let expected_len = chunk_size * chunk_size;

    if resource.fresh_water.len() != expected_len {
        return Err(ValidationError::ChunkInconsistency {
            coord: *coord,
            detail: "fresh_water array length mismatch",
        });
    }
    if resource.nutrients.len() != expected_len {
        return Err(ValidationError::ChunkInconsistency {
            coord: *coord,
            detail: "nutrients array length mismatch",
        });
    }
    if resource.minerals.len() != expected_len {
        return Err(ValidationError::ChunkInconsistency {
            coord: *coord,
            detail: "minerals array length mismatch",
        });
    }
    if resource.biomass_potential.len() != expected_len {
        return Err(ValidationError::ChunkInconsistency {
            coord: *coord,
            detail: "biomass_potential array length mismatch",
        });
    }

    for &val in &resource.fresh_water {
        if val < 0.0 || val > config.fresh_water_max {
            return Err(ValidationError::ResourceOutOfBounds {
                coord: *coord,
                field: "fresh_water",
                value: val,
            });
        }
    }

    for &val in &resource.nutrients {
        if val < 0.0 || val > config.nutrients_max {
            return Err(ValidationError::ResourceOutOfBounds {
                coord: *coord,
                field: "nutrients",
                value: val,
            });
        }
    }

    for &val in &resource.minerals {
        if val < 0.0 || val > config.minerals_max {
            return Err(ValidationError::ResourceOutOfBounds {
                coord: *coord,
                field: "minerals",
                value: val,
            });
        }
    }

    for &val in &resource.biomass_potential {
        if val < 0.0 || val > config.biomass_potential_max {
            return Err(ValidationError::ResourceOutOfBounds {
                coord: *coord,
                field: "biomass_potential",
                value: val,
            });
        }
    }

    Ok(())
}

/// Validates that all resource fields reside within configured bounds and are non-negative.
///
/// # Panics (debug/test)
///
/// Panics if any cell values violate `WorldConfig` limits or are negative.
pub fn validate_resource_fields(
    config: Res<WorldConfig>,
    query: Query<(&ChunkCoord, &ResourceChunk)>,
) {
    for (coord, resource) in &query {
        if let Err(err) = validate_resource_chunk(coord, resource, &config) {
            panic!("Resource validation failed: {:?}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::derive_resource_seed;

    fn test_config() -> WorldConfig {
        WorldConfig {
            world_width: 64,
            world_height: 64,
            chunk_size: 16,
            ..WorldConfig::default()
        }
    }

    #[test]
    fn value_noise_is_deterministic() {
        let root_seed = 98765;
        let res_seed = derive_resource_seed(root_seed);
        let v1 = value_noise_sample(5, 12, res_seed, 8, 0.0, 1.0);
        let v2 = value_noise_sample(5, 12, res_seed, 8, 0.0, 1.0);
        assert_eq!(v1, v2);

        let v3 = value_noise_sample(6, 12, res_seed, 8, 0.0, 1.0);
        assert_ne!(v1, v3);
    }

    #[test]
    fn resource_updates_only_on_daily_boundaries() {
        let mut world = World::new();

        let config = test_config();
        let chunk_size = config.chunk_size;
        let mut clock = SimulationClock::default();
        clock.total_ticks = 1; // Not a daily boundary

        world.insert_resource(config.clone());
        world.insert_resource(clock);

        let entity = world
            .spawn((
                ChunkCoord::new(0, 0),
                TerrainChunk {
                    elevation: vec![0.5; (chunk_size * chunk_size) as usize],
                    slope: vec![0.1; (chunk_size * chunk_size) as usize],
                    water_depth: vec![0.2; (chunk_size * chunk_size) as usize],
                    soil_depth: vec![0.8; (chunk_size * chunk_size) as usize],
                    soil_fertility: vec![0.6; (chunk_size * chunk_size) as usize],
                },
                ClimateChunk {
                    temperature: vec![0.7; (chunk_size * chunk_size) as usize],
                    moisture: vec![0.5; (chunk_size * chunk_size) as usize],
                    rainfall: vec![0.35; (chunk_size * chunk_size) as usize],
                    sunlight_factor: vec![0.5; (chunk_size * chunk_size) as usize],
                },
                ResourceChunk {
                    fresh_water: vec![0.5; (chunk_size * chunk_size) as usize],
                    nutrients: vec![0.5; (chunk_size * chunk_size) as usize],
                    minerals: vec![0.5; (chunk_size * chunk_size) as usize],
                    biomass_potential: vec![0.5; (chunk_size * chunk_size) as usize],
                },
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(update_resource_fields);
        world.add_schedule(schedule);

        // Run on tick 1 (sub-daily boundary)
        world.run_schedule(crate::app::FixedSimulationTick);

        // Verify no changes occurred
        let rc = world.entity(entity).get::<ResourceChunk>().unwrap();
        assert_eq!(rc.fresh_water[0], 0.5);

        // Advance to tick 24 (daily boundary)
        let mut clock = world.resource_mut::<SimulationClock>();
        clock.total_ticks = 24;

        // Run updates
        world.run_schedule(crate::app::FixedSimulationTick);

        // Verify updates applied
        let rc = world.entity(entity).get::<ResourceChunk>().unwrap();
        assert_ne!(rc.fresh_water[0], 0.5);
        assert_ne!(rc.nutrients[0], 0.5);
        assert_ne!(rc.biomass_potential[0], 0.5);
        // Minerals must remain static
        assert_eq!(rc.minerals[0], 0.5);
    }

    #[test]
    fn resource_validation_passes_valid_and_catches_invalid() {
        let config = test_config();
        let chunk_size = config.chunk_size;
        let mut world = World::new();
        world.insert_resource(config.clone());

        let entity = world
            .spawn((
                ChunkCoord::new(0, 0),
                ResourceChunk {
                    fresh_water: vec![0.5; (chunk_size * chunk_size) as usize],
                    nutrients: vec![0.5; (chunk_size * chunk_size) as usize],
                    minerals: vec![0.5; (chunk_size * chunk_size) as usize],
                    biomass_potential: vec![0.5; (chunk_size * chunk_size) as usize],
                },
            ))
            .id();

        // Run validation schedule
        let mut schedule = Schedule::new(crate::app::PostTickValidation);
        schedule.add_systems(validate_resource_fields);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::PostTickValidation);

        // Intentionally corrupt the state by exceeding max fresh_water
        world.entity_mut(entity).insert(ResourceChunk {
            fresh_water: vec![2.5; (chunk_size * chunk_size) as usize], // limit is 1.0
            nutrients: vec![0.5; (chunk_size * chunk_size) as usize],
            minerals: vec![0.5; (chunk_size * chunk_size) as usize],
            biomass_potential: vec![0.5; (chunk_size * chunk_size) as usize],
        });

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            world.run_schedule(crate::app::PostTickValidation);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn save_load_compatibility() {
        let chunk_size = 16;
        let chunk = ResourceChunk {
            fresh_water: vec![0.2; (chunk_size * chunk_size) as usize],
            nutrients: vec![0.4; (chunk_size * chunk_size) as usize],
            minerals: vec![0.6; (chunk_size * chunk_size) as usize],
            biomass_potential: vec![0.8; (chunk_size * chunk_size) as usize],
        };

        let serialized = serde_json::to_string(&chunk).unwrap();
        let deserialized: ResourceChunk = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chunk.fresh_water, deserialized.fresh_water);
        assert_eq!(chunk.nutrients, deserialized.nutrients);
        assert_eq!(chunk.minerals, deserialized.minerals);
        assert_eq!(chunk.biomass_potential, deserialized.biomass_potential);
    }
}

//! Energy availability component, deterministic generation, and updates for the Genesis world simulation.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::WorldConfig;
use crate::time::SimulationClock;
use crate::world::climate::ClimateChunk;
use crate::world::coord::ChunkCoord;
use crate::world::resource::ResourceChunk;
use crate::world::terrain::TerrainChunk;

/// Usable environmental energy potential fields for all cells in one chunk.
///
/// Each field is a `Vec<f32>` of length `chunk_size * chunk_size`.
/// Cell index: `local_y * chunk_size + local_x`.
///
/// All values must remain within the range `[0.0, config.*_max]`.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EnergyAvailabilityChunk {
    /// Solar exposure potential. Range: `[0.0, solar_exposure_max]`.
    pub solar_exposure: Vec<f32>,

    /// Usable environmental energy potential. Range: `[0.0, energy_availability_max]`.
    pub energy_availability: Vec<f32>,
}

// ---------------------------------------------------------------------------
// ECS Update and Validation Systems
// ---------------------------------------------------------------------------

/// Pure helper function to compute solar exposure for a single cell.
#[inline]
pub fn calculate_solar_exposure(
    elevation: f32,
    slope: f32,
    sunlight_factor: f32,
    config: &WorldConfig,
) -> f32 {
    let base_solar = (sunlight_factor
        * (1.0 + elevation * config.solar_elevation_coeff)
        * (1.0 - slope * config.solar_slope_coeff))
        .clamp(0.0, 1.0);
    base_solar * config.solar_exposure_max
}

/// Pure helper function to compute aggregate energy availability for a single cell.
#[inline]
pub fn calculate_energy_availability(
    solar_exposure: f32,
    temperature: f32,
    biomass_potential: f32,
    nutrients: f32,
    config: &WorldConfig,
) -> f32 {
    let norm_solar = solar_exposure / config.solar_exposure_max.max(f32::EPSILON);
    let norm_biomass = biomass_potential / config.biomass_potential_max.max(f32::EPSILON);
    let norm_nut = nutrients / config.nutrients_max.max(f32::EPSILON);
    let aggregate = (norm_solar * config.energy_solar_weight
        + temperature * config.energy_temp_weight
        + norm_biomass * config.energy_biomass_weight
        + norm_nut * config.energy_nutrient_weight)
        .clamp(0.0, 1.0);
    aggregate * config.energy_availability_max
}

/// Generates initial energy availability fields for all chunk entities populated with terrain, climate, and resource data.
pub fn generate_energy_availability_chunks(
    mut commands: Commands,
    config: Res<WorldConfig>,
    query: Query<
        (
            Entity,
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
        ),
        Without<EnergyAvailabilityChunk>,
    >,
) {
    let chunk_size = config.chunk_size;

    for (entity, _coord, terrain, climate, resource) in &query {
        let n = (chunk_size * chunk_size) as usize;
        let mut solar_exposure = vec![0.0f32; n];
        let mut energy_availability = vec![0.0f32; n];

        for idx in 0..n {
            let elevation = terrain.elevation[idx];
            let slope = terrain.slope[idx];
            let sunlight_factor = climate.sunlight_factor[idx];
            let temp = climate.temperature[idx];
            let biomass_pot = resource.biomass_potential[idx];
            let nutrients = resource.nutrients[idx];

            solar_exposure[idx] =
                calculate_solar_exposure(elevation, slope, sunlight_factor, &config);
            energy_availability[idx] = calculate_energy_availability(
                solar_exposure[idx],
                temp,
                biomass_pot,
                nutrients,
                &config,
            );
        }

        commands.entity(entity).insert(EnergyAvailabilityChunk {
            solar_exposure,
            energy_availability,
        });
    }
}

/// Recalculates energy availability fields for all chunks once per simulation day.
///
/// Update cadence is driven by the simulation clock.
pub fn update_energy_availability_fields(
    clock: Res<SimulationClock>,
    config: Res<WorldConfig>,
    mut query: Query<(
        &ChunkCoord,
        &TerrainChunk,
        &ClimateChunk,
        &ResourceChunk,
        &mut EnergyAvailabilityChunk,
    )>,
) {
    // Perform updates only on daily ticks
    if !clock.total_ticks.is_multiple_of(config.day_length_ticks) {
        return;
    }

    let chunk_size = config.chunk_size;

    for (_coord, terrain, climate, resource, mut energy) in &mut query {
        let n = (chunk_size * chunk_size) as usize;

        for idx in 0..n {
            let elevation = terrain.elevation[idx];
            let slope = terrain.slope[idx];
            let sunlight_factor = climate.sunlight_factor[idx];
            let temp = climate.temperature[idx];
            let biomass_pot = resource.biomass_potential[idx];
            let nutrients = resource.nutrients[idx];

            energy.solar_exposure[idx] =
                calculate_solar_exposure(elevation, slope, sunlight_factor, &config);
            energy.energy_availability[idx] = calculate_energy_availability(
                energy.solar_exposure[idx],
                temp,
                biomass_pot,
                nutrients,
                &config,
            );
        }
    }
}

use crate::validation::ValidationError;

/// Validates all cell values inside the energy chunk against limits specified in [`WorldConfig`].
pub fn validate_energy_chunk(
    coord: &ChunkCoord,
    energy: &EnergyAvailabilityChunk,
    config: &WorldConfig,
) -> Result<(), ValidationError> {
    let chunk_size = config.chunk_size as usize;
    let expected_len = chunk_size * chunk_size;

    if energy.solar_exposure.len() != expected_len {
        return Err(ValidationError::ChunkInconsistency {
            coord: *coord,
            detail: "solar_exposure array length mismatch",
        });
    }
    if energy.energy_availability.len() != expected_len {
        return Err(ValidationError::ChunkInconsistency {
            coord: *coord,
            detail: "energy_availability array length mismatch",
        });
    }

    for &val in &energy.solar_exposure {
        if val < 0.0 || val > config.solar_exposure_max {
            return Err(ValidationError::EnergyOutOfBounds {
                coord: *coord,
                field: "solar_exposure",
                value: val,
            });
        }
    }

    for &val in &energy.energy_availability {
        if val < 0.0 || val > config.energy_availability_max {
            return Err(ValidationError::EnergyOutOfBounds {
                coord: *coord,
                field: "energy_availability",
                value: val,
            });
        }
    }

    Ok(())
}

/// Validates that all energy availability fields reside within configured bounds and are non-negative.
///
/// # Panics (debug/test)
///
/// Panics if any cell values violate `WorldConfig` limits or are negative.
pub fn validate_energy_fields(
    config: Res<WorldConfig>,
    query: Query<(&ChunkCoord, &EnergyAvailabilityChunk)>,
) {
    for (coord, energy) in &query {
        if let Err(err) = validate_energy_chunk(coord, energy, &config) {
            panic!("Energy validation failed: {:?}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> WorldConfig {
        WorldConfig {
            world_width: 64,
            world_height: 64,
            chunk_size: 16,
            ..WorldConfig::default()
        }
    }

    #[test]
    fn energy_updates_only_on_daily_boundaries() {
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
                EnergyAvailabilityChunk {
                    solar_exposure: vec![0.5; (chunk_size * chunk_size) as usize],
                    energy_availability: vec![0.5; (chunk_size * chunk_size) as usize],
                },
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(update_energy_availability_fields);
        world.add_schedule(schedule);

        // Run on tick 1 (sub-daily boundary)
        world.run_schedule(crate::app::FixedSimulationTick);

        // Verify no changes occurred
        let ec = world
            .entity(entity)
            .get::<EnergyAvailabilityChunk>()
            .unwrap();
        assert_eq!(ec.solar_exposure[0], 0.5);
        assert_eq!(ec.energy_availability[0], 0.5);

        // Advance to tick 24 (daily boundary)
        let mut clock = world.resource_mut::<SimulationClock>();
        clock.total_ticks = 24;

        // Run updates
        world.run_schedule(crate::app::FixedSimulationTick);

        // Verify updates applied
        let ec = world
            .entity(entity)
            .get::<EnergyAvailabilityChunk>()
            .unwrap();
        assert_ne!(ec.solar_exposure[0], 0.5);
        assert_ne!(ec.energy_availability[0], 0.5);
    }

    #[test]
    fn energy_validation_passes_valid_and_catches_invalid() {
        let config = test_config();
        let chunk_size = config.chunk_size;
        let mut world = World::new();
        world.insert_resource(config.clone());

        let entity = world
            .spawn((
                ChunkCoord::new(0, 0),
                EnergyAvailabilityChunk {
                    solar_exposure: vec![0.5; (chunk_size * chunk_size) as usize],
                    energy_availability: vec![0.5; (chunk_size * chunk_size) as usize],
                },
            ))
            .id();

        // Run validation schedule
        let mut schedule = Schedule::new(crate::app::PostTickValidation);
        schedule.add_systems(validate_energy_fields);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::PostTickValidation);

        // Corrupt by exceeding limits
        world.entity_mut(entity).insert(EnergyAvailabilityChunk {
            solar_exposure: vec![2.5; (chunk_size * chunk_size) as usize], // exceeds limit
            energy_availability: vec![0.5; (chunk_size * chunk_size) as usize],
        });

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            world.run_schedule(crate::app::PostTickValidation);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn save_load_compatibility() {
        let chunk_size = 16;
        let chunk = EnergyAvailabilityChunk {
            solar_exposure: vec![0.2; (chunk_size * chunk_size) as usize],
            energy_availability: vec![0.7; (chunk_size * chunk_size) as usize],
        };

        let serialized = serde_json::to_string(&chunk).unwrap();
        let deserialized: EnergyAvailabilityChunk = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chunk.solar_exposure, deserialized.solar_exposure);
        assert_eq!(chunk.energy_availability, deserialized.energy_availability);
    }
}

//! Climate component and deterministic updates for the Genesis world simulation.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::WorldConfig;
use crate::time::SimulationClock;
use crate::world::coord::ChunkCoord;
use crate::world::terrain::TerrainChunk;

/// Climate fields for all cells in one chunk.
///
/// Each field is a `Vec<f32>` of length `chunk_size * chunk_size`.
/// Cell index: `local_y * chunk_size + local_x`.
///
/// All values must remain within the ranges defined in [`WorldConfig`].
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ClimateChunk {
    /// Temperature of each cell. Range: `[temperature_min, temperature_max]`.
    pub temperature: Vec<f32>,

    /// Moisture of each cell. Range: `[moisture_min, moisture_max]`.
    pub moisture: Vec<f32>,

    /// Rainfall of each cell. Range: `[rainfall_min, rainfall_max]`.
    pub rainfall: Vec<f32>,

    /// Sunlight factor of each cell (latitude dependent). Range: `[sunlight_factor_min, sunlight_factor_max]`.
    pub sunlight_factor: Vec<f32>,
}

// ---------------------------------------------------------------------------
// Pure climate calculation functions
// ---------------------------------------------------------------------------

/// Computes the seasonal modifier dynamically using a deterministic triangle wave.
///
/// Oscillates between `-1.0` and `1.0` over the course of a simulation year.
/// Pure function.
pub fn calculate_seasonal_modifier(total_ticks: u32, config: &WorldConfig) -> f32 {
    let year_length_ticks =
        config.day_length_ticks * config.season_length_days * config.seasons_per_year;
    if year_length_ticks == 0 {
        return 0.0;
    }
    let progress = (total_ticks % year_length_ticks) as f32 / year_length_ticks as f32;

    // Triangle wave
    if progress < 0.25 {
        4.0 * progress
    } else if progress < 0.75 {
        2.0 - 4.0 * progress
    } else {
        4.0 * progress - 4.0
    }
}

/// Computes the static sunlight (latitude) factor for global y coordinate.
///
/// Gradient ranges from `0.0` (pole, north/top) to `1.0` (equator, south/bottom).
/// Pure function.
pub fn calculate_sunlight_factor(gy: u32, world_height: u32) -> f32 {
    if world_height == 0 {
        return 0.0;
    }
    (gy as f32 / world_height as f32).clamp(0.0, 1.0)
}

/// Computes cell temperature from latitude, elevation, and seasonal modifiers.
///
/// Base temperature uses sea level base, lapse rate, and sunlight factors.
/// Pure function.
pub fn calculate_temperature(
    elevation: f32,
    sunlight_factor: f32,
    seasonal_modifier: f32,
    config: &WorldConfig,
) -> f32 {
    let base_temp = config.sea_level_temperature_base - (elevation * config.temperature_lapse_rate)
        + (sunlight_factor * 0.4);
    let temp = base_temp + seasonal_modifier * config.seasonal_temperature_amplitude;
    temp.clamp(config.temperature_min, config.temperature_max)
}

/// Computes moisture based on elevation and water presence.
///
/// Pure function.
pub fn calculate_moisture(water_depth: f32, elevation: f32, config: &WorldConfig) -> f32 {
    let base_moisture = if config.sea_level > 0.0 {
        (water_depth / config.sea_level) * 0.8 + (1.0 - elevation) * 0.2
    } else {
        (1.0 - elevation) * 0.2
    };
    base_moisture.clamp(config.moisture_min, config.moisture_max)
}

/// Computes rainfall from moisture and temperature.
///
/// Pure function.
pub fn calculate_rainfall(moisture: f32, temperature: f32, config: &WorldConfig) -> f32 {
    let rainfall = moisture * temperature;
    rainfall.clamp(config.rainfall_min, config.rainfall_max)
}

// ---------------------------------------------------------------------------
// ECS Update Systems
// ---------------------------------------------------------------------------

/// Recalculates climate fields for all chunks once per simulation day.
///
/// Checks clock ticks to enforce daily updates, returning early otherwise.
pub fn update_climate_fields(
    clock: Res<SimulationClock>,
    config: Res<WorldConfig>,
    mut query: Query<(&ChunkCoord, &TerrainChunk, &mut ClimateChunk)>,
) {
    // Perform updates only on daily ticks
    if clock.total_ticks % config.day_length_ticks != 0 {
        return;
    }

    let seasonal_modifier = calculate_seasonal_modifier(clock.total_ticks, &config);
    let chunk_size = config.chunk_size;
    let world_height = config.world_height;

    for (chunk_coord, terrain, mut climate) in &mut query {
        let n = (chunk_size * chunk_size) as usize;

        for idx in 0..n {
            let _lx = (idx as u32) % chunk_size;
            let ly = (idx as u32) / chunk_size;

            let gy = chunk_coord.y * chunk_size + ly;
            let sunlight_factor = calculate_sunlight_factor(gy, world_height);
            climate.sunlight_factor[idx] = sunlight_factor;

            let elevation = terrain.elevation[idx];
            let water_depth = terrain.water_depth[idx];

            let temp =
                calculate_temperature(elevation, sunlight_factor, seasonal_modifier, &config);
            climate.temperature[idx] = temp;

            let moisture = calculate_moisture(water_depth, elevation, &config);
            climate.moisture[idx] = moisture;

            let rainfall = calculate_rainfall(moisture, temp, &config);
            climate.rainfall[idx] = rainfall;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seasonal_modifier_bounds() {
        let config = WorldConfig::default();

        let m0 = calculate_seasonal_modifier(0, &config);
        assert_eq!(m0, 0.0);

        let m_peak = calculate_seasonal_modifier(2160, &config); // 1 season = 90 days = 2160 ticks
        assert!(m_peak > 0.9 && m_peak <= 1.0);

        let m_trough = calculate_seasonal_modifier(6480, &config); // 3 seasons = 6480 ticks
        assert!(m_trough < -0.9 && m_trough >= -1.0);
    }

    #[test]
    fn sunlight_factor_scaling() {
        let f0 = calculate_sunlight_factor(0, 512);
        assert_eq!(f0, 0.0);

        let f_mid = calculate_sunlight_factor(256, 512);
        assert_eq!(f_mid, 0.5);

        let f_max = calculate_sunlight_factor(512, 512);
        assert_eq!(f_max, 1.0);
    }

    #[test]
    fn climate_updates_only_on_daily_boundaries() {
        let mut world = World::new();

        let config = WorldConfig::default();
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
                    slope: vec![0.0; (chunk_size * chunk_size) as usize],
                    water_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                    soil_depth: vec![0.5; (chunk_size * chunk_size) as usize],
                    soil_fertility: vec![0.5; (chunk_size * chunk_size) as usize],
                },
                ClimateChunk {
                    temperature: vec![0.0; (chunk_size * chunk_size) as usize],
                    moisture: vec![0.0; (chunk_size * chunk_size) as usize],
                    rainfall: vec![0.0; (chunk_size * chunk_size) as usize],
                    sunlight_factor: vec![0.0; (chunk_size * chunk_size) as usize],
                },
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(update_climate_fields);
        world.add_schedule(schedule);

        // Run on tick 1 (sub-daily boundary)
        world.run_schedule(crate::app::FixedSimulationTick);

        // Verify no changes (temperature remains 0.0)
        let climate = world.entity(entity).get::<ClimateChunk>().unwrap();
        assert_eq!(climate.temperature[0], 0.0);

        // Advance clock to tick 24 (daily boundary)
        let mut clock = world.resource_mut::<SimulationClock>();
        clock.total_ticks = 24;

        // Run again
        world.run_schedule(crate::app::FixedSimulationTick);

        // Verify updates applied (temperature should now be calculated)
        let climate = world.entity(entity).get::<ClimateChunk>().unwrap();
        assert!(climate.temperature[0] > 0.0);
    }
}

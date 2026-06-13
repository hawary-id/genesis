//! Test fixtures and helpers for Genesis engine tests.

use crate::config::WorldConfig;
use crate::rng::WorldSeed;

/// Returns a deterministic test configuration.
///
/// Uses a smaller world size to keep future tests fast.
pub fn create_test_config() -> WorldConfig {
    WorldConfig {
        world_width: 256,
        world_height: 256,
        chunk_size: 32,

        day_length_ticks: 24,
        season_length_days: 90,
        seasons_per_year: 4,

        generation_version: 1,

        elevation_min: 0.0,
        elevation_max: 1.0,
        slope_max: 1.0,
        water_depth_max: 1.0,
        soil_depth_max: 1.0,
        soil_fertility_max: 1.0,
        sea_level: 0.35,

        temperature_min: 0.0,
        temperature_max: 1.0,
        moisture_min: 0.0,
        moisture_max: 1.0,
        rainfall_min: 0.0,
        rainfall_max: 1.0,
        sunlight_factor_min: 0.0,
        sunlight_factor_max: 1.0,

        fresh_water_max: 1.0,
        nutrients_max: 1.0,
        minerals_max: 1.0,
        biomass_potential_max: 1.0,
        solar_exposure_max: 1.0,
        energy_availability_max: 1.0,
        solar_elevation_coeff: 0.2,
        solar_slope_coeff: 0.3,
        energy_solar_weight: 0.4,
        energy_temp_weight: 0.3,
        energy_biomass_weight: 0.2,
        energy_nutrient_weight: 0.1,

        sea_level_temperature_base: 0.6,
        temperature_lapse_rate: 0.4,
        seasonal_temperature_amplitude: 0.15,
    }
}

/// Returns a deterministic test seed.
pub fn create_test_seed() -> WorldSeed {
    WorldSeed::new(987_654_321)
}

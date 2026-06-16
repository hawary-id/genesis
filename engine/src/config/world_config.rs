//! World configuration data structures.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Configuration parameters for the Genesis world simulation.
///
/// Immutable after startup generation begins.
///
/// All world dimensions, time constants, and terrain validation ranges
/// are stored here. Systems that need world-wide parameters read this resource.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Width of the world in cells.
    pub world_width: u32,

    /// Height of the world in cells.
    pub world_height: u32,

    /// Width and height of a single square chunk in cells.
    pub chunk_size: u32,

    /// Number of simulation ticks in one day.
    pub day_length_ticks: u32,

    /// Number of simulation days in one season.
    pub season_length_days: u32,

    /// Number of seasons in one year.
    pub seasons_per_year: u32,

    /// Version of the world generation pipeline.
    /// Incremented when the generation algorithm changes in a breaking way.
    pub generation_version: u32,

    // -------------------------------------------------------------------------
    // Terrain validation ranges.
    // These bounds define what values are considered valid after generation.
    // All terrain fields are normalized to [0.0, 1.0] by default.
    // -------------------------------------------------------------------------
    /// Minimum valid elevation value.
    pub elevation_min: f32,

    /// Maximum valid elevation value.
    pub elevation_max: f32,

    /// Maximum valid slope value.
    /// Slope is always non-negative, so the minimum is implicitly 0.0.
    pub slope_max: f32,

    /// Maximum valid water depth value.
    /// Water depth is always non-negative, so the minimum is implicitly 0.0.
    pub water_depth_max: f32,

    /// Maximum valid soil depth value.
    /// Soil depth is always non-negative, so the minimum is implicitly 0.0.
    pub soil_depth_max: f32,

    /// Maximum valid soil fertility value.
    /// Soil fertility is always non-negative, so the minimum is implicitly 0.0.
    pub soil_fertility_max: f32,

    /// Elevation threshold below which water is present.
    /// Cells at or below this elevation receive water depth proportional
    /// to the distance below the threshold.
    pub sea_level: f32,

    // -------------------------------------------------------------------------
    // Climate validation ranges.
    // All climate fields are normalized to [0.0, 1.0] by default.
    // -------------------------------------------------------------------------
    /// Minimum valid temperature.
    pub temperature_min: f32,

    /// Maximum valid temperature.
    pub temperature_max: f32,

    /// Minimum valid moisture.
    pub moisture_min: f32,

    /// Maximum valid moisture.
    pub moisture_max: f32,

    /// Minimum valid rainfall.
    pub rainfall_min: f32,

    /// Maximum valid rainfall.
    pub rainfall_max: f32,

    /// Minimum valid sunlight factor.
    pub sunlight_factor_min: f32,

    /// Maximum valid sunlight factor.
    pub sunlight_factor_max: f32,

    // -------------------------------------------------------------------------
    // Resource validation ranges.
    // -------------------------------------------------------------------------
    /// Maximum valid fresh water availability.
    pub fresh_water_max: f32,

    /// Maximum valid nutrients availability.
    pub nutrients_max: f32,

    /// Maximum valid minerals concentration.
    pub minerals_max: f32,

    /// Maximum valid biomass carrying potential.
    pub biomass_potential_max: f32,

    /// Maximum valid solar exposure.
    pub solar_exposure_max: f32,

    /// Maximum valid energy availability.
    pub energy_availability_max: f32,

    /// Coefficient for terrain elevation impact on solar exposure.
    pub solar_elevation_coeff: f32,

    /// Coefficient for terrain slope impact on solar exposure.
    pub solar_slope_coeff: f32,

    /// Weight of solar exposure in aggregate energy availability.
    pub energy_solar_weight: f32,

    /// Weight of climate temperature in aggregate energy availability.
    pub energy_temp_weight: f32,

    /// Weight of biomass potential in aggregate energy availability.
    pub energy_biomass_weight: f32,

    /// Weight of soil nutrients in aggregate energy availability.
    pub energy_nutrient_weight: f32,

    // -------------------------------------------------------------------------
    // Climate generation settings.
    // -------------------------------------------------------------------------
    /// Baseline temperature at sea level.
    pub sea_level_temperature_base: f32,

    /// Rate of temperature decrease per unit of elevation.
    pub temperature_lapse_rate: f32,

    /// Amplitude of seasonal temperature variance.
    pub seasonal_temperature_amplitude: f32,

    // -------------------------------------------------------------------------
    // Agent spawning and metadata settings.
    // -------------------------------------------------------------------------
    /// Initial number of agents spawned at startup.
    pub initial_agent_count: u32,

    /// Initial energy given to spawned agents.
    pub initial_agent_energy: f32,

    /// Maximum energy metabolic capacity for agents.
    pub agent_energy_max: f32,

    /// Chronological age limit for agents.
    pub agent_age_limit: u32,

    /// Maximum density cap for the agent population.
    pub agent_density_cap: u32,

    /// Spatial cell distance defining neighborhood sensing bounds.
    pub sensing_radius: u32,

    /// Base metabolic energy decay rate per tick.
    pub agent_base_decay_rate: f32,

    /// Target temperature at which agent energy decay is minimized.
    pub agent_thermal_optimum: f32,

    /// Multiplier applied to absolute temperature difference (Delta-T) to calculate metabolic penalty.
    pub thermal_penalty_multiplier: f32,

    /// Maximum slope threshold that an agent can traverse.
    pub agent_movement_max_slope: f32,

    /// Maximum water depth threshold that an agent can enter.
    pub agent_movement_max_water_depth: f32,

    /// Energy cost of a movement step.
    pub agent_movement_cost: f32,

    /// Maximum resource intake rate per cell per tick.
    /// Used to cap nutrient and fresh water consumption.
    pub max_harvest_rate: f32,

    /// Efficiency factor for converting consumed resource mass to metabolic energy stock.
    /// Represents digestion/assimilation efficiency in range [0.0, 1.0].
    pub consumption_efficiency: f32,

    /// Probability of mutation per gene index during asexual reproduction.
    #[serde(default = "default_mutation_rate")]
    pub mutation_rate: f32,

    /// Standard deviation of Gaussian mutation displacement offsets.
    #[serde(default = "default_mutation_step_size")]
    pub mutation_step_size: f32,
}

fn default_mutation_rate() -> f32 {
    0.05
}

fn default_mutation_step_size() -> f32 {
    0.05
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            world_width: 512,
            world_height: 512,
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

            initial_agent_count: 10,
            initial_agent_energy: 100.0,
            agent_energy_max: 1000.0,
            agent_age_limit: 1000,
            agent_density_cap: 1000,
            sensing_radius: 1,
            agent_base_decay_rate: 1.0,
            agent_thermal_optimum: 0.5,
            thermal_penalty_multiplier: 2.0,
            agent_movement_max_slope: 0.40,
            agent_movement_max_water_depth: 0.30,
            agent_movement_cost: 1.0,
            max_harvest_rate: 10.0,
            consumption_efficiency: 0.8,
            mutation_rate: 0.05,
            mutation_step_size: 0.05,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = WorldConfig::default();

        assert_eq!(config.world_width, 512);
        assert_eq!(config.world_height, 512);
        assert_eq!(config.chunk_size, 32);

        assert_eq!(config.day_length_ticks, 24);
        assert_eq!(config.season_length_days, 90);
        assert_eq!(config.seasons_per_year, 4);

        assert_eq!(config.generation_version, 1);
        assert_eq!(config.mutation_rate, 0.05);
        assert_eq!(config.mutation_step_size, 0.05);
    }

    #[test]
    fn default_terrain_ranges_are_normalized() {
        let config = WorldConfig::default();

        assert_eq!(config.elevation_min, 0.0);
        assert_eq!(config.elevation_max, 1.0);
        assert!(config.slope_max > 0.0);
        assert!(config.water_depth_max > 0.0);
        assert!(config.soil_depth_max > 0.0);
        assert!(config.soil_fertility_max > 0.0);

        assert_eq!(config.temperature_min, 0.0);
        assert_eq!(config.temperature_max, 1.0);
        assert_eq!(config.moisture_min, 0.0);
        assert_eq!(config.moisture_max, 1.0);
        assert_eq!(config.rainfall_min, 0.0);
        assert_eq!(config.rainfall_max, 1.0);
        assert_eq!(config.sunlight_factor_min, 0.0);
        assert_eq!(config.sunlight_factor_max, 1.0);

        assert!(config.sea_level_temperature_base > 0.0);
        assert!(config.temperature_lapse_rate > 0.0);
        assert!(config.seasonal_temperature_amplitude > 0.0);
    }

    #[test]
    fn elevation_range_is_valid() {
        let config = WorldConfig::default();
        assert!(config.elevation_min < config.elevation_max);
    }

    #[test]
    fn sea_level_is_within_elevation_range() {
        let config = WorldConfig::default();
        assert!(config.sea_level >= config.elevation_min);
        assert!(config.sea_level <= config.elevation_max);
    }
}

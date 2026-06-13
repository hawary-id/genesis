//! World configuration data structures.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Configuration parameters for the Genesis world simulation.
///
/// This resource stores immutable settings determined at startup.
///
/// Milestone 1 only defines configuration data.
/// No simulation logic or derived calculations belong here yet.
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
    pub generation_version: u32,
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
    }
}

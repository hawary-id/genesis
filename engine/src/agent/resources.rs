//! ECS resources for agents in Genesis.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

/// State-tracking stable identifier generator resource.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StableIdGenerator {
    next_id: u64,
}

impl StableIdGenerator {
    /// Creates a new stable identifier generator starting from `1`.
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    /// Allocates and returns the next sequential unique stable ID.
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }
}

impl Default for StableIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration ranges for mapping genome values to concrete agent traits.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GenomeConfig {
    /// Range of temperature values for thermal optimum.
    pub thermal_optimum_range: (f32, f32),
    /// Range of dietary preference (0.0 = nutrient, 1.0 = water).
    pub diet_preference_range: (f32, f32),
    /// Range of maximum slope tolerance.
    pub max_slope_range: (f32, f32),
    /// Range of maximum water depth tolerance.
    pub max_water_depth_range: (f32, f32),
    /// Range of sensing cell radius values.
    pub sensing_radius_range: (u32, u32),
    /// Range of energy thresholds to trigger reproduction.
    pub reproduction_threshold_range: (f32, f32),
    /// Range of maturity age limits.
    pub maturity_age_range: (u32, u32),
    /// Range of physical sizes.
    pub physical_size_range: (f32, f32),
}

impl Default for GenomeConfig {
    fn default() -> Self {
        Self {
            thermal_optimum_range: (0.0, 1.0),
            diet_preference_range: (0.0, 1.0),
            max_slope_range: (0.10, 0.60),
            max_water_depth_range: (0.10, 0.50),
            sensing_radius_range: (1, 4),
            reproduction_threshold_range: (150.0, 500.0),
            maturity_age_range: (20, 200),
            physical_size_range: (0.5, 2.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_allocates_sequentially() {
        let mut gen = StableIdGenerator::new();
        assert_eq!(gen.next_id(), 1);
        assert_eq!(gen.next_id(), 2);
        assert_eq!(gen.next_id(), 3);
    }
}

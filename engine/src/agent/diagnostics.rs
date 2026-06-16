//! Diagnostics and telemetry for agent populations.
//!
//! Exposes population-wide statistics computed dynamically from active ECS entities.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::agent::components::{AgentMetadata, Phenotype};

/// Diagnostic metrics representing the evolutionary health and trait distribution
/// of the current live population.
///
/// This data is transient telemetry. It is fully derived from the current ECS world state
/// and does not require snapshot serialization to maintain determinism.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PopulationStatistics {
    /// Total number of active agents in the world.
    pub total_population: usize,
    /// Arithmetic mean of `thermal_optimum` across all live agents.
    pub mean_thermal_optimum: f32,
    /// Arithmetic mean of `diet_preference` across all live agents.
    pub mean_diet_preference: f32,
    /// Arithmetic mean of `max_slope` across all live agents.
    pub mean_max_slope: f32,
    /// Arithmetic mean of `max_water_depth` across all live agents.
    pub mean_max_water_depth: f32,
    /// Arithmetic mean of `sensing_radius` across all live agents.
    pub mean_sensing_radius: f32,
    /// Arithmetic mean of `physical_size` across all live agents.
    pub mean_physical_size: f32,
    /// Population standard deviation of `thermal_optimum` across all live agents.
    pub standard_deviation_thermal_optimum: f32,
}

/// System computing the population metrics dynamically from live agent entities.
///
/// Executes strictly under the `ObservationBoundary` schedule to maintain
/// separation of concerns and avoid introducing side-effects into simulation ticks.
pub fn compute_population_statistics(
    mut stats: ResMut<PopulationStatistics>,
    agents: Query<&Phenotype, With<AgentMetadata>>,
) {
    let count = agents.iter().count();
    stats.total_population = count;

    if count == 0 {
        stats.mean_thermal_optimum = 0.0;
        stats.mean_diet_preference = 0.0;
        stats.mean_max_slope = 0.0;
        stats.mean_max_water_depth = 0.0;
        stats.mean_sensing_radius = 0.0;
        stats.mean_physical_size = 0.0;
        stats.standard_deviation_thermal_optimum = 0.0;
        return;
    }

    let mut sum_thermal_optimum = 0.0;
    let mut sum_diet_preference = 0.0;
    let mut sum_max_slope = 0.0;
    let mut sum_max_water_depth = 0.0;
    let mut sum_sensing_radius = 0.0;
    let mut sum_physical_size = 0.0;

    for phenotype in &agents {
        sum_thermal_optimum += phenotype.thermal_optimum;
        sum_diet_preference += phenotype.diet_preference;
        sum_max_slope += phenotype.max_slope;
        sum_max_water_depth += phenotype.max_water_depth;
        sum_sensing_radius += phenotype.sensing_radius as f32;
        sum_physical_size += phenotype.physical_size;
    }

    let n = count as f32;
    stats.mean_thermal_optimum = sum_thermal_optimum / n;
    stats.mean_diet_preference = sum_diet_preference / n;
    stats.mean_max_slope = sum_max_slope / n;
    stats.mean_max_water_depth = sum_max_water_depth / n;
    stats.mean_sensing_radius = sum_sensing_radius / n;
    stats.mean_physical_size = sum_physical_size / n;

    let mut variance_sum = 0.0;
    for phenotype in &agents {
        let diff = phenotype.thermal_optimum - stats.mean_thermal_optimum;
        variance_sum += diff * diff;
    }

    stats.standard_deviation_thermal_optimum = (variance_sum / n).sqrt();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_population_statistics_empty() {
        let mut world = World::new();
        world.insert_resource(PopulationStatistics::default());

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(compute_population_statistics);
        schedule.run(&mut world);

        let stats = world.resource::<PopulationStatistics>();
        assert_eq!(stats.total_population, 0);
        assert_eq!(stats.mean_thermal_optimum, 0.0);
    }

    #[test]
    fn compute_population_statistics_calculates_means_and_stddev() {
        let mut world = World::new();
        world.insert_resource(PopulationStatistics::default());

        // Spawn 3 agents to check logic
        world.spawn((
            AgentMetadata::new(1),
            Phenotype::new(10.0, 0.2, 0.5, 0.1, 2, 100.0, 10, 1.0, 1.0, 1.0),
        ));
        world.spawn((
            AgentMetadata::new(2),
            Phenotype::new(20.0, 0.4, 0.7, 0.3, 3, 100.0, 10, 1.5, 1.0, 1.0),
        ));
        world.spawn((
            AgentMetadata::new(3),
            Phenotype::new(30.0, 0.6, 0.9, 0.5, 4, 100.0, 10, 2.0, 1.0, 1.0),
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(compute_population_statistics);
        schedule.run(&mut world);

        let stats = world.resource::<PopulationStatistics>();
        assert_eq!(stats.total_population, 3);
        assert_eq!(stats.mean_thermal_optimum, 20.0);
        assert!((stats.mean_diet_preference - 0.4).abs() < 1e-5);
        assert!((stats.mean_max_slope - 0.7).abs() < 1e-5);
        assert!((stats.mean_max_water_depth - 0.3).abs() < 1e-5);
        assert_eq!(stats.mean_sensing_radius, 3.0);
        assert_eq!(stats.mean_physical_size, 1.5);

        // Std dev of [10, 20, 30] where mean is 20:
        // Variance = ((-10)^2 + 0^2 + 10^2) / 3 = 200 / 3 = 66.666...
        // StdDev = sqrt(66.666) ~ 8.1649658
        let expected_stddev = (200.0f32 / 3.0).sqrt();
        assert!((stats.standard_deviation_thermal_optimum - expected_stddev).abs() < 1e-5);
    }
}

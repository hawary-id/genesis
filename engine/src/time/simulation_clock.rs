//! Simulation clock foundations.
//!
//! Milestone 1 provides only the canonical simulation clock.
//! No simulation systems, seasonal logic, or time progression
//! behavior are implemented yet.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Canonical simulation clock.
///
/// Genesis uses a deterministic fixed-timestep model.
///
/// Under the current design:
///
/// - 1 tick = 1 simulation hour
///
/// Higher-level concepts such as days, seasons, and years
/// will be derived in later milestones.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SimulationClock {
    /// Total ticks elapsed since simulation start.
    pub total_ticks: u32,

    /// Duration represented by a single tick.
    pub tick_duration_hours: u32,
}

impl Default for SimulationClock {
    fn default() -> Self {
        Self {
            total_ticks: 0,
            tick_duration_hours: 1,
        }
    }
}

impl SimulationClock {
    /// Creates a new simulation clock.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_clock_defaults_are_valid() {
        let clock = SimulationClock::new();

        assert_eq!(clock.total_ticks, 0);
        assert_eq!(clock.tick_duration_hours, 1);
    }
}

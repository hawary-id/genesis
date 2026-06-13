//! Simulation clock foundations.
//!
//! Canonical simulation clock and deterministic time derivation helpers.
//! No simulation systems, seasonal logic, or time progression
//! behavior are implemented yet.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::config::WorldConfig;

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

    /// Derives the current simulation day count (0-indexed) based on `total_ticks`.
    pub fn current_day(&self, config: &WorldConfig) -> u32 {
        self.total_ticks
            .checked_div(config.day_length_ticks)
            .unwrap_or(0)
    }

    /// Derives the current simulation season index (0-indexed) based on `total_ticks`.
    pub fn current_season(&self, config: &WorldConfig) -> u32 {
        let ticks_per_season = config.day_length_ticks * config.season_length_days;
        if ticks_per_season == 0 || config.seasons_per_year == 0 {
            0
        } else {
            (self.total_ticks / ticks_per_season) % config.seasons_per_year
        }
    }

    /// Derives the current simulation year count (0-indexed) based on `total_ticks`.
    pub fn current_year(&self, config: &WorldConfig) -> u32 {
        let ticks_per_season = config.day_length_ticks * config.season_length_days;
        let ticks_per_year = ticks_per_season * config.seasons_per_year;
        self.total_ticks.checked_div(ticks_per_year).unwrap_or(0)
    }

    /// Derives the progress [0.0, 1.0) within the current season.
    pub fn season_progress(&self, config: &WorldConfig) -> f32 {
        let ticks_per_season = config.day_length_ticks * config.season_length_days;
        if ticks_per_season == 0 {
            0.0
        } else {
            let tick_in_season = self.total_ticks % ticks_per_season;
            tick_in_season as f32 / ticks_per_season as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::SeasonState;
    use bevy_ecs::prelude::*;

    fn test_config() -> WorldConfig {
        WorldConfig {
            day_length_ticks: 24,
            season_length_days: 90,
            seasons_per_year: 4,
            ..WorldConfig::default()
        }
    }

    #[test]
    fn simulation_clock_defaults_are_valid() {
        let clock = SimulationClock::new();

        assert_eq!(clock.total_ticks, 0);
        assert_eq!(clock.tick_duration_hours, 1);
    }

    #[test]
    fn clock_advancement() {
        let mut clock = SimulationClock::new();
        assert_eq!(clock.total_ticks, 0);
        clock.total_ticks += 1;
        assert_eq!(clock.total_ticks, 1);
        clock.total_ticks += 1;
        assert_eq!(clock.total_ticks, 2);
    }

    #[test]
    fn day_derivation() {
        let config = test_config();
        let mut clock = SimulationClock::new();
        assert_eq!(clock.current_day(&config), 0);
        clock.total_ticks = 23;
        assert_eq!(clock.current_day(&config), 0);
        clock.total_ticks = 24;
        assert_eq!(clock.current_day(&config), 1);
        clock.total_ticks = 48;
        assert_eq!(clock.current_day(&config), 2);
    }

    #[test]
    fn season_derivation() {
        let config = test_config();
        let mut clock = SimulationClock::new();
        // 1 season = 90 days = 2160 ticks
        assert_eq!(clock.current_season(&config), 0);
        clock.total_ticks = 2159;
        assert_eq!(clock.current_season(&config), 0);
        clock.total_ticks = 2160;
        assert_eq!(clock.current_season(&config), 1);
        clock.total_ticks = 4320;
        assert_eq!(clock.current_season(&config), 2);
    }

    #[test]
    fn season_rollover_derivation() {
        let config = test_config();
        let mut clock = SimulationClock::new();

        clock.total_ticks = 8640;
        assert_eq!(clock.current_season(&config), 0);

        clock.total_ticks = 10800;
        assert_eq!(clock.current_season(&config), 1);

        clock.total_ticks = 12960;
        assert_eq!(clock.current_season(&config), 2);

        clock.total_ticks = 15120;
        assert_eq!(clock.current_season(&config), 3);

        clock.total_ticks = 17280;
        assert_eq!(clock.current_season(&config), 0);
    }

    #[test]
    fn year_derivation() {
        let config = test_config();
        let mut clock = SimulationClock::new();
        // 1 year = 4 seasons = 360 days = 8640 ticks
        assert_eq!(clock.current_year(&config), 0);
        clock.total_ticks = 8639;
        assert_eq!(clock.current_year(&config), 0);
        clock.total_ticks = 8640;
        assert_eq!(clock.current_year(&config), 1);
        clock.total_ticks = 17280;
        assert_eq!(clock.current_year(&config), 2);
    }

    #[test]
    fn season_boundary_transitions() {
        let config = test_config();

        // Season 0 start
        let s0 = SeasonState::derive(0, &config);
        assert_eq!(s0.season_index, 0);
        assert_eq!(s0.tick_in_season, 0);
        assert_eq!(s0.progress, 0.0);
        assert_eq!(s0.seasonal_modifier, 0.0);

        // Season 1 start (midpoint peak of triangle wave)
        let s_peak = SeasonState::derive(2160, &config);
        assert_eq!(s_peak.season_index, 1);
        assert_eq!(s_peak.tick_in_season, 0);
        assert_eq!(s_peak.progress, 0.0);
        assert!((s_peak.seasonal_modifier - 1.0).abs() < 1e-5);

        // Season 2 start (zero-crossing going down)
        let s_mid = SeasonState::derive(4320, &config);
        assert_eq!(s_mid.season_index, 2);
        assert_eq!(s_mid.tick_in_season, 0);
        assert_eq!(s_mid.progress, 0.0);
        assert!(s_mid.seasonal_modifier.abs() < 1e-5);

        // Season 3 start (trough of triangle wave)
        let s_trough = SeasonState::derive(6480, &config);
        assert_eq!(s_trough.season_index, 3);
        assert_eq!(s_trough.tick_in_season, 0);
        assert_eq!(s_trough.progress, 0.0);
        assert!((s_trough.seasonal_modifier - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn season_state_reconstruction() {
        let config = test_config();
        let ticks = 5432;
        let s_orig = SeasonState::derive(ticks, &config);

        // Reconstruct from SimulationClock tick count
        let clock = SimulationClock {
            total_ticks: ticks,
            tick_duration_hours: 1,
        };
        let s_reconstructed = SeasonState::derive(clock.total_ticks, &config);

        assert_eq!(s_orig.season_index, s_reconstructed.season_index);
        assert_eq!(s_orig.tick_in_season, s_reconstructed.tick_in_season);
        assert_eq!(s_orig.progress, s_reconstructed.progress);
        assert_eq!(s_orig.seasonal_modifier, s_reconstructed.seasonal_modifier);
    }

    #[test]
    fn deterministic_season_state_generation() {
        let mut world = World::new();
        let config = test_config();
        let clock = SimulationClock::default();
        let initial_season = SeasonState::derive(clock.total_ticks, &config);

        world.insert_resource(config.clone());
        world.insert_resource(clock);
        world.insert_resource(initial_season);

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems((
            crate::time::advance_simulation_clock,
            crate::time::update_season_state.after(crate::time::advance_simulation_clock),
        ));
        world.add_schedule(schedule);

        // Run tick steps
        for _ in 0..100 {
            world.run_schedule(crate::app::FixedSimulationTick);
        }

        let final_clock = world.resource::<SimulationClock>();
        let final_season = world.resource::<SeasonState>();

        assert_eq!(final_clock.total_ticks, 100);

        let derived = SeasonState::derive(100, &config);
        assert_eq!(final_season.season_index, derived.season_index);
        assert_eq!(final_season.tick_in_season, derived.tick_in_season);
        assert_eq!(final_season.progress, derived.progress);
        assert_eq!(final_season.seasonal_modifier, derived.seasonal_modifier);
    }
}

//! Seasonal state structure and derivation logic for the Genesis world simulation.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::config::WorldConfig;

/// Canonical derived seasonal state.
///
/// Under the current design, `SeasonState` is computed entirely as a pure
/// derivation from `SimulationClock.total_ticks` and `WorldConfig`.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SeasonState {
    /// Index of the current season (0-indexed).
    pub season_index: u32,

    /// Tick position within the current season.
    pub tick_in_season: u32,

    /// Normalized progress within the current season [0.0, 1.0].
    pub progress: f32,

    /// Seasonal modifier [-1.0, 1.0] used for climate and temperature scaling.
    pub seasonal_modifier: f32,
}

impl SeasonState {
    /// Derives the SeasonState from the current clock tick count and configuration.
    pub fn derive(total_ticks: u32, config: &WorldConfig) -> Self {
        let day_length = config.day_length_ticks;
        let season_length = config.season_length_days;
        let seasons_per_year = config.seasons_per_year;

        let ticks_per_season = day_length * season_length;
        let ticks_per_year = ticks_per_season * seasons_per_year;

        let (season_index, tick_in_season, progress, seasonal_modifier) =
            if ticks_per_season == 0 || seasons_per_year == 0 || ticks_per_year == 0 {
                (0, 0, 0.0, 0.0)
            } else {
                let tick_in_season = total_ticks % ticks_per_season;
                let season_index = (total_ticks / ticks_per_season) % seasons_per_year;
                let progress = tick_in_season as f32 / ticks_per_season as f32;

                let tick_in_year = total_ticks % ticks_per_year;
                let year_progress = tick_in_year as f32 / ticks_per_year as f32;
                let seasonal_modifier = if year_progress < 0.25 {
                    4.0 * year_progress
                } else if year_progress < 0.75 {
                    2.0 - 4.0 * year_progress
                } else {
                    4.0 * year_progress - 4.0
                };

                (season_index, tick_in_season, progress, seasonal_modifier)
            };

        Self {
            season_index,
            tick_in_season,
            progress,
            seasonal_modifier,
        }
    }
}

use crate::time::SimulationClock;
use crate::validation::ValidationError;

/// Validates that the active season state resource matches the derived state from simulation clock.
pub fn validate_season_state(
    clock: &SimulationClock,
    season_state: &SeasonState,
    config: &WorldConfig,
) -> Result<(), ValidationError> {
    let derived = SeasonState::derive(clock.total_ticks, config);

    if season_state.season_index != derived.season_index
        || season_state.tick_in_season != derived.tick_in_season
        || (season_state.progress - derived.progress).abs() > 1e-5
        || (season_state.seasonal_modifier - derived.seasonal_modifier).abs() > 1e-5
    {
        return Err(ValidationError::SeasonStateMismatch {
            total_ticks: clock.total_ticks,
        });
    }

    Ok(())
}

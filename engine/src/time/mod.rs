//! Module for simulation time and clock mechanics.

pub mod season_state;
pub mod simulation_clock;

pub use season_state::SeasonState;
pub use simulation_clock::SimulationClock;

use crate::config::WorldConfig;
use bevy_ecs::prelude::*;

/// Advances the canonical simulation clock by exactly 1 tick.
///
/// This is the sole mutator of the simulation clock.
pub fn advance_simulation_clock(mut clock: ResMut<SimulationClock>) {
    clock.total_ticks += 1;
}

/// Updates the SeasonState resource based on the SimulationClock and WorldConfig.
pub fn update_season_state(
    clock: Res<SimulationClock>,
    config: Res<WorldConfig>,
    mut season_state: ResMut<SeasonState>,
) {
    *season_state = SeasonState::derive(clock.total_ticks, &config);
}

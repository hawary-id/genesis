//! ECS events for agents.

use crate::agent::components::LocationCategory;
use crate::world::coord::WorldCoord;
use bevy_ecs::event::Event;

/// Emitted by sensing and movement systems when an agent observes a notable location.
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ObservationEvent {
    /// The agent that observed the location.
    pub agent_id: u64,
    /// The coordinate that was observed.
    pub coord: WorldCoord,
    /// The category of the observation.
    pub category: LocationCategory,
}

impl ObservationEvent {
    /// Creates a new `ObservationEvent`.
    pub fn new(agent_id: u64, coord: WorldCoord, category: LocationCategory) -> Self {
        Self {
            agent_id,
            coord,
            category,
        }
    }
}

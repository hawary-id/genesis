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

/// Emitted by agent interaction systems when a notable subjective episodic event occurs.
#[derive(Event, Debug, Clone, PartialEq)]
pub struct EventMemoryEvent {
    /// The agent that experienced the event.
    pub agent_id: u64,
    /// The category of the event.
    pub category: crate::agent::components::EventCategory,
    /// The simulation tick when the event occurred.
    pub tick: u32,
    /// The deterministic sequence order of the event within the tick.
    pub sequence_in_tick: u32,
}

impl EventMemoryEvent {
    /// Creates a new `EventMemoryEvent`.
    pub fn new(
        agent_id: u64,
        category: crate::agent::components::EventCategory,
        tick: u32,
        sequence_in_tick: u32,
    ) -> Self {
        Self {
            agent_id,
            category,
            tick,
            sequence_in_tick,
        }
    }
}

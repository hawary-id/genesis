//! ECS components for agents in Genesis.

use crate::world::coord::WorldCoord;
use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

/// Intent of the agent behavior action request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionIntent {
    /// Do not move.
    None,
    /// Move one step North.
    MoveNorth,
    /// Move one step South.
    MoveSouth,
    /// Move one step East.
    MoveEast,
    /// Move one step West.
    MoveWest,
}

/// Identifies an agent entity with a unique stable identifier.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Stable identifier.
    pub id: u64,
}

impl AgentMetadata {
    /// Creates a new `AgentMetadata`.
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// Spatial location of an agent in the world grid.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPosition {
    /// Global grid coordinates of the agent.
    pub coord: WorldCoord,
}

impl AgentPosition {
    /// Creates a new `AgentPosition`.
    pub fn new(coord: WorldCoord) -> Self {
        Self { coord }
    }
}

/// Metabolic stock of energy and lifespan tracking for an agent.
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MetabolicStock {
    /// Current remaining energy stock.
    pub energy: f32,
    /// Chronological age in simulation ticks.
    pub age: u32,
}

impl MetabolicStock {
    /// Creates a new `MetabolicStock`.
    pub fn new(energy: f32, age: u32) -> Self {
        Self { energy, age }
    }
}

/// Active behavior action intent request of an agent.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionRequest {
    /// Requested behavioral action intent.
    pub intent: ActionIntent,
}

impl ActionRequest {
    /// Creates a new `ActionRequest`.
    pub fn new(intent: ActionIntent) -> Self {
        Self { intent }
    }
}

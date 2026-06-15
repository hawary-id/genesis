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

/// Genetic representation of an agent.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Genome {
    /// Vector of gene values in the range [0.0, 1.0].
    pub genes: Vec<f32>,
}

impl Genome {
    /// Creates a new `Genome` from a vector of gene values.
    pub fn new(genes: Vec<f32>) -> Self {
        Self { genes }
    }
}

/// Concrete traits derived from agent's genome.
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Phenotype {
    /// Thermal optimum temperature.
    pub thermal_optimum: f32,
    /// Diet preference (0.0 = pure nutrient consumer, 1.0 = pure fresh water consumer).
    pub diet_preference: f32,
    /// Max elevation slope traversed.
    pub max_slope: f32,
    /// Max water depth traversed.
    pub max_water_depth: f32,
    /// Sensing cell radius.
    pub sensing_radius: u32,
    /// Energy threshold to reproduce.
    pub reproduction_threshold: f32,
    /// Maturity age in simulation ticks.
    pub maturity_age: u32,
    /// Physical size multiplier.
    pub physical_size: f32,
    /// Derived base decay rate.
    pub derived_base_decay: f32,
    /// Derived step movement energy cost.
    pub derived_movement_cost: f32,
}

impl Phenotype {
    /// Creates a new `Phenotype` with the given values.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        thermal_optimum: f32,
        diet_preference: f32,
        max_slope: f32,
        max_water_depth: f32,
        sensing_radius: u32,
        reproduction_threshold: f32,
        maturity_age: u32,
        physical_size: f32,
        derived_base_decay: f32,
        derived_movement_cost: f32,
    ) -> Self {
        Self {
            thermal_optimum,
            diet_preference,
            max_slope,
            max_water_depth,
            sensing_radius,
            reproduction_threshold,
            maturity_age,
            physical_size,
            derived_base_decay,
            derived_movement_cost,
        }
    }
}

/// Family lineage and generational depth metadata.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageMetadata {
    /// Unique stable identifier of the direct parent.
    pub parent_id: Option<u64>,
    /// Generation count.
    pub generation: u32,
}

impl LineageMetadata {
    /// Creates a new `LineageMetadata`.
    pub fn new(parent_id: Option<u64>, generation: u32) -> Self {
        Self {
            parent_id,
            generation,
        }
    }
}

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

/// Canonical length of the genome vector.
pub const GENOME_SIZE: usize = 8;

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

/// Category of a remembered location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationCategory {
    /// Location with harvestable nutrients.
    Nutrient,
    /// Location with harvestable fresh water.
    FreshWater,
    /// Location with dangerous conditions.
    Hazard,
}

/// A single remembered location with temporal context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationMemoryNode {
    /// The spatial coordinate.
    pub coord: WorldCoord,
    /// The category of the location.
    pub category: LocationCategory,
    /// The chronological timestamp of the observation.
    pub last_observed_tick: u32,
}

impl LocationMemoryNode {
    /// Creates a new `LocationMemoryNode`.
    pub fn new(coord: WorldCoord, category: LocationCategory, last_observed_tick: u32) -> Self {
        Self {
            coord,
            category,
            last_observed_tick,
        }
    }
}

/// Component representing an agent's subjective spatial memory.
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LocationMemory {
    /// Fixed-capacity chronological list of remembered locations.
    pub nodes: Vec<LocationMemoryNode>,
}

impl LocationMemory {
    /// Creates a new empty `LocationMemory`.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}

/// Category of an episodic experiential event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCategory {
    /// Agent successfully consumed resources.
    ResourceConsumed,
    /// Agent attempted to move but failed due to terrain constraints.
    FailedMovement,
    /// Agent successfully reproduced.
    Reproduced,
    /// Agent encountered a hazard.
    HazardEncountered,
}

/// A single recorded episodic memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMemoryNode {
    /// The category of the event.
    pub category: EventCategory,
    /// The simulation tick when the event occurred.
    pub tick: u32,
    /// The deterministic sequence order of the event within the tick.
    pub sequence_in_tick: u32,
}

impl EventMemoryNode {
    /// Creates a new `EventMemoryNode`.
    pub fn new(category: EventCategory, tick: u32, sequence_in_tick: u32) -> Self {
        Self {
            category,
            tick,
            sequence_in_tick,
        }
    }
}

/// The maximum number of episodic events an agent can remember.
pub const MAX_EVENT_MEMORY_CAPACITY: usize = 10;

/// Component representing an agent's subjective episodic memory.
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EventMemory {
    /// Fixed-capacity chronological list of remembered events.
    pub nodes: Vec<EventMemoryNode>,
}

impl EventMemory {
    /// Creates a new empty `EventMemory`.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}

/// Category of a social relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocialRelationCategory {
    /// Agent is the parent of the remembered agent.
    Parent,
    /// Agent is the child of the remembered agent.
    Child,
}

/// A single recorded subjective social relationship memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocialMemoryNode {
    /// The stable identifier of the remembered agent.
    pub agent_id: u64,
    /// The category of the social relationship.
    pub relation: SocialRelationCategory,
    /// The simulation tick when the relationship was created/remembered.
    pub created_tick: u32,
}

impl SocialMemoryNode {
    /// Creates a new `SocialMemoryNode`.
    pub fn new(agent_id: u64, relation: SocialRelationCategory, created_tick: u32) -> Self {
        Self {
            agent_id,
            relation,
            created_tick,
        }
    }
}

/// The maximum number of social relationships an agent can remember.
pub const MAX_SOCIAL_MEMORY_CAPACITY: usize = 10;

/// Component representing an agent's subjective social memory graph.
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SocialMemory {
    /// Fixed-capacity chronological list of remembered relationships.
    pub nodes: Vec<SocialMemoryNode>,
}

impl SocialMemory {
    /// Creates a new empty `SocialMemory`.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}

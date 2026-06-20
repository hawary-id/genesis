//! Agent subcrate module declarations and public API exposures.

pub mod components;
pub mod diagnostics;
pub mod events;
pub mod resources;
pub mod sensing;
pub mod systems;

pub use components::{
    ActionIntent, ActionRequest, AgentMetadata, AgentPosition, EventCategory, EventMemory,
    EventMemoryNode, Genome, LineageMetadata, LocationCategory, LocationMemory, LocationMemoryNode,
    MetabolicStock, Phenotype, SocialMemory, SocialMemoryNode, SocialRelationCategory, GENOME_SIZE,
    MAX_EVENT_MEMORY_CAPACITY, MAX_SOCIAL_MEMORY_CAPACITY,
};
pub use events::{EventMemoryEvent, ObservationEvent, SocialMemoryEvent};
pub use resources::{EventSequenceCounter, GenomeConfig, StableIdGenerator};
pub use sensing::{query_cell, query_neighborhood, SensedResource};
pub use systems::{
    derive_phenotype_on_spawn, process_agent_consumption, process_agent_deaths,
    process_agent_movement, process_agent_reproduction, process_agent_sensing,
    process_event_memory_consolidation, process_memory_consolidation,
    process_social_memory_consolidation, reset_event_sequence, spawn_initial_agents,
    update_agent_metabolism,
};

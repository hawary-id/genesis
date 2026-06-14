//! Agent subcrate module declarations and public API exposures.

pub mod components;
pub mod resources;
pub mod sensing;
pub mod systems;

pub use components::{ActionIntent, ActionRequest, AgentMetadata, AgentPosition, MetabolicStock};
pub use resources::StableIdGenerator;
pub use sensing::{query_cell, query_neighborhood, SensedResource};
pub use systems::{
    process_agent_deaths, process_agent_movement, spawn_initial_agents, update_agent_metabolism,
};

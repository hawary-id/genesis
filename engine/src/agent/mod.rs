//! Agent subcrate module declarations and public API exposures.

pub mod components;
pub mod resources;
pub mod systems;

pub use components::{ActionIntent, ActionRequest, AgentMetadata, AgentPosition, MetabolicStock};
pub use resources::StableIdGenerator;
pub use systems::spawn_initial_agents;

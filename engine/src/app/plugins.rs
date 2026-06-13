//! ECS bootstrap helpers.
//!
//! Provides resource registration and schedule initialization for the Genesis engine.
//! No simulation systems, ticking, persistence, or observation pipelines exist yet.

use bevy_ecs::prelude::World;

use crate::config::{WorldBounds, WorldConfig};
use crate::rng::WorldSeed;
use crate::time::SimulationClock;

use super::schedules::register_schedules;

/// Registers all foundational resources and schedules required by Genesis.
///
/// Milestone 2 resources:
///
/// - [`WorldConfig`] — immutable world parameters
/// - [`WorldSeed`] — root deterministic seed
/// - [`SimulationClock`] — canonical simulation time
/// - [`WorldBounds`] — validated coordinate boundaries derived from [`WorldConfig`]
///
/// Also registers the five Phase 1 schedule labels in canonical execution order.
/// Schedule registration does not imply execution. No systems are added.
pub fn register_initial_resources(world: &mut World, config: WorldConfig, seed: WorldSeed) {
    let bounds = WorldBounds::from_config(&config);

    world.insert_resource(config);
    world.insert_resource(seed);
    world.insert_resource(SimulationClock::new());
    world.insert_resource(bounds);

    register_schedules(world);
}

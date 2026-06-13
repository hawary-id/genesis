//! Application bootstrap container for Genesis.
//!
//! Milestone 2 extends the foundational ECS setup with:
//! - [`WorldBounds`] resource
//! - [`WorldGenerationCompleted`] event type definition
//! - Phase 1 schedule label definitions
//!
//! No simulation schedules are executed, no world generation exists,
//! no ticking, persistence, or observation systems exist yet.

pub mod events;
pub mod plugins;
pub mod schedules;

use bevy_ecs::prelude::World;

use crate::config::WorldConfig;
use crate::rng::WorldSeed;

pub use events::WorldGenerationCompleted;
pub use schedules::{
    FixedSimulationTick, ObservationBoundary, PersistenceBoundary, PostTickValidation,
    StartupGeneration,
};

/// Primary application container.
///
/// Owns the ECS world and registers the foundational
/// resources and schedule labels required by Genesis.
pub struct App {
    world: World,
}

impl App {
    /// Creates a new Genesis application instance.
    ///
    /// Registers all Milestone 2 resources and schedule labels:
    /// - [`crate::config::WorldConfig`]
    /// - [`crate::rng::WorldSeed`]
    /// - [`crate::time::SimulationClock`]
    /// - [`crate::config::WorldBounds`]
    /// - All five Phase 1 schedule labels
    pub fn new(config: WorldConfig, seed: WorldSeed) -> Self {
        let mut world = World::new();

        plugins::register_initial_resources(&mut world, config, seed);

        Self { world }
    }

    /// Returns an immutable reference to the ECS world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Returns a mutable reference to the ECS world.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WorldBounds;
    use crate::testing::{create_test_config, create_test_seed};
    use crate::time::SimulationClock;
    use bevy_ecs::prelude::Schedules;

    fn test_app() -> App {
        App::new(create_test_config(), create_test_seed())
    }

    #[test]
    fn app_initializes_world_config() {
        let app = test_app();
        assert!(app.world().contains_resource::<WorldConfig>());
    }

    #[test]
    fn app_initializes_world_seed() {
        let app = test_app();
        assert!(app.world().contains_resource::<crate::rng::WorldSeed>());
    }

    #[test]
    fn app_initializes_simulation_clock() {
        let app = test_app();
        assert!(app.world().contains_resource::<SimulationClock>());
    }

    #[test]
    fn app_initializes_world_bounds() {
        let app = test_app();
        assert!(app.world().contains_resource::<WorldBounds>());
    }

    #[test]
    fn app_registers_all_phase1_schedules() {
        let app = test_app();
        let schedules = app.world().resource::<Schedules>();

        assert!(schedules.contains(StartupGeneration));
        assert!(schedules.contains(FixedSimulationTick));
        assert!(schedules.contains(PostTickValidation));
        assert!(schedules.contains(PersistenceBoundary));
        assert!(schedules.contains(ObservationBoundary));
    }

    #[test]
    fn world_bounds_consistent_with_world_config() {
        let app = test_app();
        let config = app.world().resource::<WorldConfig>();
        let bounds = app.world().resource::<WorldBounds>();

        assert_eq!(bounds.world_width, config.world_width);
        assert_eq!(bounds.world_height, config.world_height);
        assert_eq!(bounds.chunk_size, config.chunk_size);
        assert_eq!(bounds.chunks_x, config.world_width / config.chunk_size);
        assert_eq!(bounds.chunks_y, config.world_height / config.chunk_size);
    }
}

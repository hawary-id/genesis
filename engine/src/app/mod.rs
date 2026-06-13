//! Genesis application bootstrap and schedule orchestration.
//!
//! Owns the ECS world and registers the resources,
//! events, and schedules required by the simulation.

pub mod events;
pub mod plugins;
pub mod schedules;

use bevy_ecs::prelude::World;
use bevy_ecs::schedule::IntoSystemConfigs;

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
    /// Initializes the ECS world and registers
    /// the resources, events, and schedules
    /// required by Genesis.
    pub fn new(config: WorldConfig, seed: WorldSeed) -> Self {
        let mut world = World::new();

        plugins::register_initial_resources(&mut world, config, seed);

        // Register the generation completed event resource
        world.init_resource::<bevy_ecs::event::Events<WorldGenerationCompleted>>();

        // Bind the generation systems to the StartupGeneration schedule
        crate::world::generation::register_generation_systems(&mut world);

        // Bind the climate and resource update systems to the FixedSimulationTick schedule
        let mut schedules = world.resource_mut::<bevy_ecs::schedule::Schedules>();
        if let Some(schedule) = schedules.get_mut(FixedSimulationTick) {
            schedule.add_systems((
                crate::world::climate::update_climate_fields,
                crate::world::resource::update_resource_fields
                    .after(crate::world::climate::update_climate_fields),
            ));
        }

        // Bind the resource validation system to the PostTickValidation schedule
        if let Some(schedule) = schedules.get_mut(PostTickValidation) {
            schedule.add_systems(crate::world::resource::validate_resource_fields);
        }

        Self { world }
    }

    /// Runs the startup generation schedule.
    pub fn run_startup(&mut self) {
        self.world.run_schedule(StartupGeneration);
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

    #[test]
    fn app_run_startup_executes_terrain_generation() {
        let mut app = test_app();

        // Before startup, no entities or completed events
        assert_eq!(app.world().entities().len(), 0);
        let events = app
            .world()
            .resource::<bevy_ecs::event::Events<WorldGenerationCompleted>>();
        assert_eq!(events.get_reader().read(events).count(), 0);

        // Run startup generation
        app.run_startup();

        // Chunks generated (test configuration: 256x256 / 32 = 8x8 = 64 chunks)
        assert_eq!(app.world().entities().len(), 64);

        // Assert event was emitted
        let events = app
            .world()
            .resource::<bevy_ecs::event::Events<WorldGenerationCompleted>>();
        assert_eq!(events.get_reader().read(events).count(), 1);
    }
}

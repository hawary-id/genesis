//! Phase 1 schedule label definitions.
//!
//! Schedules define the execution pipeline for Genesis simulation systems.
//! Milestone 2 defines schedule labels and registers them in the ECS world.
//! No systems are added to any schedule in this milestone.
//! Empty schedules are correct at this stage.
//!
//! # Canonical Execution Order
//!
//! ```text
//! StartupGeneration       (runs once before ticking begins)
//!
//! FixedSimulationTick
//! PostTickValidation
//! PersistenceBoundary
//! ObservationBoundary     (repeats each tick)
//! ```
//!
//! This ordering is a determinism requirement.
//! Persistence and observation must not interleave with simulation mutation.
//! Validation must occur before persistence so that only valid state is saved.

use bevy_ecs::prelude::World;
use bevy_ecs::schedule::{Schedule, ScheduleLabel};

/// Runs once at startup to build the initial deterministic world
/// from configuration and seed.
///
/// `FixedSimulationTick` must not run until this schedule completes
/// and `WorldGenerationCompleted` has been emitted.
///
/// Systems added in Milestone 3:
/// configuration loading, seed initialization, world bounds derivation,
/// chunk spawning, terrain generation, climate generation, resource generation,
/// energy availability generation, generated chunk marking, world validation,
/// and world generation completed event emission.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct StartupGeneration;

/// Advances the world by exactly one deterministic simulation tick.
///
/// Invariants:
/// - No database writes occur in this schedule.
/// - No snapshot or observation export occurs in this schedule.
/// - No history or event archive is written in this schedule.
///
/// Systems added in Milestone 4–7:
/// clock advancement, season state update, climate field updates,
/// resource field updates, energy availability updates, dirty chunk marking.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixedSimulationTick;

/// Detects invalid world state immediately after deterministic mutation.
///
/// Runs after `FixedSimulationTick`, before any persistence or observation.
///
/// Invariant: debug and test builds must run full validation.
/// Release builds may reduce validation only after correctness is established.
///
/// Systems added in Milestone 8:
/// clock monotonicity validation, chunk coordinate validation,
/// chunk dimension validation, terrain range validation, climate range validation,
/// resource range validation, energy availability range validation, season state validation.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostTickValidation;

/// Saves stable world state without affecting simulation outcomes.
///
/// Runs after `PostTickValidation`.
///
/// Invariants:
/// - Persistence failure must not mutate simulation state.
/// - PostgreSQL integration belongs behind this boundary.
///
/// Systems added in Milestone 9:
/// snapshot due detection, snapshot request handling, world snapshot construction,
/// snapshot write, snapshot completed event emission, dirty marker cleanup.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PersistenceBoundary;

/// Produces read-only summaries for debugging, metrics, and future
/// dashboard and API layers.
///
/// Runs after `PersistenceBoundary`.
///
/// Invariant: observation must not mutate simulation state.
/// The dashboard must observe the world, not shape it.
///
/// Systems added in future milestones:
/// world metrics collection, generation metrics collection,
/// validation metrics collection, observation snapshot export.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObservationBoundary;

/// Registers all Phase 1 schedule labels in the ECS world.
///
/// Schedules are registered in canonical execution order:
///
/// 1. [`StartupGeneration`]
/// 2. [`FixedSimulationTick`]
/// 3. [`PostTickValidation`]
/// 4. [`PersistenceBoundary`]
/// 5. [`ObservationBoundary`]
///
/// Registration does not imply execution. No systems are assigned to any
/// schedule by this function. Systems are added in later milestones only.
pub fn register_schedules(world: &mut World) {
    world.add_schedule(Schedule::new(StartupGeneration));
    world.add_schedule(Schedule::new(FixedSimulationTick));
    world.add_schedule(Schedule::new(PostTickValidation));
    world.add_schedule(Schedule::new(PersistenceBoundary));
    world.add_schedule(Schedule::new(ObservationBoundary));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::schedule::Schedules;

    fn world_with_schedules() -> World {
        let mut world = World::new();
        register_schedules(&mut world);
        world
    }

    #[test]
    fn startup_generation_is_registered() {
        let world = world_with_schedules();
        let schedules = world.resource::<Schedules>();
        assert!(schedules.contains(StartupGeneration));
    }

    #[test]
    fn fixed_simulation_tick_is_registered() {
        let world = world_with_schedules();
        let schedules = world.resource::<Schedules>();
        assert!(schedules.contains(FixedSimulationTick));
    }

    #[test]
    fn post_tick_validation_is_registered() {
        let world = world_with_schedules();
        let schedules = world.resource::<Schedules>();
        assert!(schedules.contains(PostTickValidation));
    }

    #[test]
    fn persistence_boundary_is_registered() {
        let world = world_with_schedules();
        let schedules = world.resource::<Schedules>();
        assert!(schedules.contains(PersistenceBoundary));
    }

    #[test]
    fn observation_boundary_is_registered() {
        let world = world_with_schedules();
        let schedules = world.resource::<Schedules>();
        assert!(schedules.contains(ObservationBoundary));
    }

    #[test]
    fn all_five_phase1_schedules_are_registered() {
        let world = world_with_schedules();
        let schedules = world.resource::<Schedules>();

        assert!(schedules.contains(StartupGeneration));
        assert!(schedules.contains(FixedSimulationTick));
        assert!(schedules.contains(PostTickValidation));
        assert!(schedules.contains(PersistenceBoundary));
        assert!(schedules.contains(ObservationBoundary));
    }
}

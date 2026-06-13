//! Phase 1 event type definitions.
//!
//! Events represent notable transitions or boundary signals.
//! They are type definitions only in Milestone 2.
//!
//! No systems that emit or consume these events are added until
//! those systems are introduced in their respective milestones.

use bevy_ecs::prelude::Event;

/// Signals that startup world generation completed and the world
/// passed initial validation.
///
/// Required because the fixed simulation tick must not begin against
/// a partially generated world. This event provides a clean boundary
/// signal for gating tick execution.
///
/// Emitted by: the final system in `StartupGeneration` (Milestone 3).
/// Consumed by: the system that enables `FixedSimulationTick` (Milestone 3).
#[derive(Event, Debug, Clone)]
pub struct WorldGenerationCompleted;

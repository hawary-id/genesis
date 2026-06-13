# Milestone 2 Architecture — Core ECS Setup

## Status

Architectural definition only.

No Rust code, no component implementations, no world generation logic.

This document defines what Milestone 2 must establish and why,
so that implementation can proceed without ambiguity.

---

## Goal

Define the complete ECS shape for Phase 1.

Milestone 1 established the crate structure, module boundaries,
and three foundational resources: `WorldConfig`, `WorldSeed`, and `SimulationClock`.

Milestone 2 extends that foundation by defining:

- WorldBounds
- Phase 1 event definitions
- Phase 1 schedule labels

required for future simulation systems.

When Milestone 2 is complete, the ECS world will provide
the correct ECS structural foundation for Phase 1 —
resources registered, schedules defined,
events declared — but no simulation behavior yet.

World generation, terrain, climate, resources, energy availability,
and persistence are not part of this milestone.

---

## Scope

Milestone 2 introduces:

- WorldBounds resource.
- All Phase 1 event type definitions.
- All Phase 1 schedule definitions as named labels.
- Schedule registration does not imply schedule execution.
- No schedule execution pipeline is introduced in Milestone 2.
- Tests verifying that the ECS world can be initialized with Phase 1 resources.
- Tests verifying that schedules are registered in the correct order.

---

## Non-Goals

The following are explicitly out of scope for Milestone 2.

### Simulation behavior

No systems execute simulation logic in this milestone.

Schedules are defined and registered.
Systems are not added to them yet.

### Component definitions

`ChunkCoord`, `TerrainChunk`, `ClimateChunk`, `ResourceChunk`,
`EnergyAvailabilityChunk`, `DirtyChunk`, and `Generated`
belong to the world module.

They are introduced in Milestone 3 alongside world generation.

Defining component shells or empty structs here would constitute
premature scaffolding and is forbidden by the architecture rules.

### World generation

Terrain, climate, resources, energy availability, and chunk spawning
are Milestone 3 deliverables.

They depend on components that do not exist yet.

### Persistence

Snapshot construction, save/load behavior, snapshot policies,
and persistence configuration are Milestone 9 deliverables.

No persistence resources are introduced in Milestone 2.

No snapshot logic is implemented.

### Observation and metrics

No metrics collection, debug summaries, or read-model exports are implemented.

`ObservationBoundary` is defined as a schedule label.
No systems are registered under it.

### Agent systems

No agent, life, ecology, culture, economy, politics, or civilization
concepts are introduced in any form.

---

## ECS Boundaries

### What belongs in resources

Resources hold globally shared simulation context.

A resource is appropriate when the state it represents is not
naturally attached to one entity and must be readable across
multiple systems.

Resources must not become god objects.
Each resource must represent one focused domain.

If a resource begins absorbing unrelated fields, it must be split.

### What does not belong in resources

Entity-local data belongs in components, not resources.

Chunk terrain fields, climate fields, resource fields, and energy
availability fields are entity-local.
They belong in components on chunk entities, not in resources.

Behavior and logic do not belong in resources.
Resources are data.

### What belongs in events

Events represent notable transitions or boundary signals.

An event is appropriate when one system must notify another
that a boundary was crossed, without coupling the two systems
through shared mutable state.

Events must not carry core simulation state.
Events must not be used as long-term storage.
Simulation truth lives in resources and components.

### What belongs in schedules

Schedules define the execution pipeline for simulation systems.

A schedule defines a named phase of execution.
Systems are assigned to schedules.
Schedules run in a defined sequence.

Order-sensitive systems within a schedule must declare
explicit ordering constraints rather than relying on
undefined scheduler behavior.

---

## Resources

The following resources are introduced in Milestone 2.
`WorldConfig`, `WorldSeed`, and `SimulationClock` already exist from Milestone 1.

---

### `WorldBounds`

**Purpose:**
Stores the validated coordinate boundaries derived from `WorldConfig`.

**Ownership:**
ECS world resource.

**Mutability:**
Immutable after startup.

**Contents:**
- World width in cells.
- World height in cells.
- Number of chunks along the x axis.
- Number of chunks along the y axis.
- Chunk size in cells.

**Rationale:**
Systems that validate coordinates, spawn chunks, or perform
boundary checks need a stable, pre-validated bounds reference.
Deriving bounds from `WorldConfig` every time adds redundancy.
A dedicated resource makes the validated state explicit.

**Relationship to other resources:**
Derived from `WorldConfig` during startup.
Must not diverge from `WorldConfig` after initialization.

---

## Events

The following events are introduced in Milestone 2.

Events are type definitions only.
No systems that emit or consume these events are added until
those systems are introduced in their respective milestones.

---

### `WorldGenerationCompleted`

**Purpose:**
Signals that startup world generation completed and the world
passed initial validation.

**Required because:**
The fixed simulation tick must not begin against a partially
generated world.

This event provides a clean boundary signal for gating tick execution.

**Emitted by:**
The final system in `StartupGeneration` (Milestone 3).

**Consumed by:**
The system that enables `FixedSimulationTick` scheduling (Milestone 3).

---

### Events Deferred

The following events are intentionally deferred until their
owning systems exist:

- SnapshotRequested (Milestone 9)
- SnapshotCompleted (Milestone 9)
- SeasonChanged
- ChunkUpdated
- Resource discovery events
- History events
- Agent events

### Rationale:

Genesis follows Pressure-Driven Development.

Events should only be introduced when a real system requires them.
Defining events for systems that do not yet exist creates
premature scaffolding and increases architectural complexity
without providing immediate value.



## Schedules

The following schedules are introduced in Milestone 2 as named labels.

Systems are not assigned in this milestone.
Schedule definitions establish the execution pipeline contract
that all later milestones depend on.

---

### `StartupGeneration`

**Purpose:**
Runs once at startup to build the initial deterministic world
from configuration and seed.

**When it runs:**
Once, before the simulation tick begins.

**Systems it will contain (future milestones):**
Configuration loading and validation, seed initialization,
world bounds derivation, chunk entity spawning, terrain generation,
climate generation, resource generation, energy availability generation,
time-related initialization, generated chunk marking,
world validation, and world generation completed event emission.

**Invariant:**
`FixedSimulationTick` must not run until `StartupGeneration` completes.

---

### `FixedSimulationTick`

**Purpose:**
Advances the world by exactly one deterministic simulation tick.

**When it runs:**
Once per simulation tick, after `StartupGeneration` completes.

**Systems it will contain (future milestones):**
Clock advancement, time-related updates, climate field updates,
resource field updates, energy availability updates,
dirty chunk marking.

**Invariants:**
- No database writes occur in this schedule.
- No snapshot or observation export occurs in this schedule.
- No history or event archive is written in this schedule.

---

### `PostTickValidation`

**Purpose:**
Detects invalid world state immediately after deterministic mutation.

**When it runs:**
After `FixedSimulationTick` completes, before any persistence or observation.

**Systems it will contain (future milestones):**
Clock monotonicity validation, chunk coordinate validation,
chunk dimension validation, terrain range validation,
climate range validation, resource range validation,
energy availability range validation, time state validation.

**Invariant:**
Debug and test builds must run full validation.
Release builds may support reduced validation after correctness is established.

---

### `PersistenceBoundary`

**Purpose:**
Saves stable world state without affecting simulation outcomes.

**When it runs:**
After `PostTickValidation`, whenever a persistence policy
requests snapshot creation.
Persistence scheduling details are defined in Milestone 9.

**Systems it will contain (future milestones):**
Snapshot due detection, snapshot request handling, world snapshot
construction, snapshot write, snapshot completed event emission,
dirty marker cleanup.

**Invariant:**
Persistence failure must not mutate simulation state.
PostgreSQL integration, if used, belongs behind this boundary.

---

### `ObservationBoundary`

**Purpose:**
Produces read-only summaries for debugging, metrics, and future
dashboard and API layers.

**When it runs:**
After `PersistenceBoundary` completes.

**Systems it will contain (future milestones):**
World metrics collection, generation metrics collection,
validation metrics collection, observation snapshot export.

**Invariant:**
Observation must not mutate simulation state.
The dashboard must observe the world, not shape it.

---

## Execution Order

The schedule execution order is fixed and must not vary:

```
StartupGeneration
    (runs once)

FixedSimulationTick
PostTickValidation
PersistenceBoundary
ObservationBoundary
    (repeats per tick)
```

This ordering is a determinism requirement.
Persistence and observation must not interleave with simulation mutation.
Validation must occur before persistence so that only valid state is saved.

---

## Success Criteria

Milestone 2 is complete when:

- The ECS world can be initialized with all currently defined resources:
  - WorldConfig
  - WorldSeed
  - SimulationClock
  - WorldBounds
- `WorldBounds` are registered.
- WorldGenerationCompleted is defined.
- All five schedule labels are defined and registered in the ECS world.
- No systems rely on hidden global state.
- Resources are narrow, purposeful, and documented.
- No component definitions exist yet.
- No simulation execution logic exists yet.
- No future civilization, agent, society, economy, culture, or AI modules have been introduced.
- Tests verify the ECS world initializes cleanly with all Phase 1 resources present.

---

## Risks

### Over-engineering resources prematurely

Each new resource must carry only the state it needs for Phase 1.

Resources are storage.
Systems are behavior.

### Placeholder systems

Milestone 2 must not introduce placeholder systems to fill empty schedules.

Empty schedules are correct at this stage.
Systems are added only when the functionality they serve is implemented.

### Naming conflicts

The term `resources` refers to both ECS resources (global shared state)
and environmental resources (water, nutrients, minerals).

Code that uses the word `resources` must be unambiguous about which meaning is intended.
Module naming must not conflate the two concepts.

The `world` module will contain environmental resource systems.
The `app` and `config` modules contain ECS resource definitions.

---

## Dependencies

- Milestone 1 must be accepted as complete before Milestone 2 begins.
- `WorldConfig`, `WorldSeed`, and `SimulationClock` must already be registered.
- The module structure established in Milestone 1 must not change.

## Milestone Boundary

Milestone 2 defines ECS structure only.

Milestone 2 does not introduce:

- simulation execution
- App::tick()
- world generation
- terrain generation
- climate systems
- resource systems
- energy systems
- persistence systems

Schedule labels may exist, but they remain empty until the functionality
they support is implemented in later milestones.

The existence of a schedule does not imply the existence of simulation behavior.

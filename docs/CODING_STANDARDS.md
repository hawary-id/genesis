# CODING_STANDARDS

## Purpose

This document defines implementation standards for Genesis.

Its purpose is to ensure that all code remains aligned with:

- VISION.md
- PRINCIPLES.md
- ARCHITECTURE_BASELINE.md
- DETERMINISM.md
- ECS_GUIDELINES.md
- PHASE1_WORLD_TECH_SPEC.md

When implementation decisions conflict with convenience, implementation must follow architecture.

Documentation remains the source of truth.

---

# Core Philosophy

Genesis is a simulation project.

Not a game.

Not a CRUD application.

Not a collection of features.

The objective is to create the conditions under which emergence becomes possible.

Every implementation decision must support:

- determinism
- scalability
- maintainability
- observability
- emergence

---

# Architectural Rules

## Rule 1: No Manager Classes

Do not create:

- WorldManager
- SimulationManager
- ResourceManager
- AgentManager
- CivilizationManager

or equivalent structures.

Reason:

Managers tend to accumulate responsibilities and become god objects.

Use ECS systems, resources, and events instead.

---

## Rule 2: No God Objects

No single object should contain the majority of simulation state.

Avoid structures such as:

```text
World {
    terrain,
    climate,
    resources,
    agents,
    economy,
    politics,
    ...
}
```

Prefer decomposition into:

- components
- resources
- events
- systems

---

## Rule 3: Components Contain Data Only

Components must not contain simulation logic.

Allowed:

```rust
pub struct TerrainChunk {
    pub elevation: Vec<f32>,
    pub fertility: Vec<f32>,
}
```

Not allowed:

```rust
impl TerrainChunk {
    pub fn update_climate(&mut self) {
        ...
    }
}
```

Simulation behavior belongs in systems.

---

## Rule 4: Components Must Remain Passive

Components are storage only.

Components must not:

- perform calculations
- validate world rules
- mutate unrelated state
- contain simulation decisions

Components exist only to store data.

---

## Rule 5: Systems Contain Logic

Simulation behavior must be implemented in systems.

Systems are responsible for:

- reading state
- transforming state
- emitting events
- enforcing rules

Systems should remain focused on a single responsibility.

---

## Rule 6: Resources Represent Global State

Resources should only represent world-wide state.

Examples:

- WorldConfig
- WorldSeed
- SimulationClock
- SeasonState

Resources must not become hidden global storage.

---

## Rule 7: Explicit Data Ownership

Ownership of simulation data must be obvious.

Every piece of state should have a clearly defined owner.

Avoid duplicated state unless justified by performance requirements.

---

# Development Order Rule

Future phases must not introduce systems whose prerequisites do not exist.

Examples:

Do not build:

- Society before agents.
- Culture before society.
- Language before communication.
- Economy before production and scarcity.
- Politics before group conflict.
- Civilization before institutions.

All higher-level systems must emerge from lower-level pressures.

---

# Simulation Integrity

Never fake emergence.

If a phenomenon appears:

- economy
- migration
- cooperation
- conflict
- culture
- technology
- institutions
- civilization

it must arise from simulation rules.

Do not create hidden shortcuts that force desired outcomes.

---

# Rust Style Guidelines

## Prefer Idiomatic Rust

Follow standard Rust conventions.

Use:

- cargo fmt
- cargo clippy

regularly.

Warnings should be treated as problems to investigate.

---

## Small Modules

Modules should remain focused.

Avoid files that grow beyond a few hundred lines without strong justification.

Prefer:

```text
world/
├── terrain.rs
├── climate.rs
├── generation.rs
├── validation.rs
```

over large monolithic modules.

---

## Explicit Types

Prefer explicit types for public APIs.

Avoid unnecessary type inference in critical simulation code.

Clarity is more important than brevity.

---

## Avoid Clever Code

Prefer readable code over compact code.

Readable code is easier to validate, test, and maintain.

---

# ECS Coding Rules

## Systems Must Be Focused

One system should do one thing.

Good:

- update climate
- regenerate resources
- advance clock

Bad:

- update entire world state

---

## Systems Must Be Deterministic

A system should produce the same output when given the same input.

No hidden randomness.

No wall-clock dependence.

No external side effects.

---

## Explicit Scheduling

Order-sensitive systems must declare execution order explicitly.

Never rely on accidental scheduler behavior.

---

## Chunk-Oriented Design

For Phase 1:

Cells are data.

Chunks are entities.

Avoid creating one ECS entity per terrain cell.

---

## Events Are Temporary

Events communicate transitions.

Events are not long-term storage.

Do not reconstruct simulation truth from event history.

---

# Module Organization

Recommended structure:

```text
engine/src/

app/
world/
time/
rng/
persistence/
config/
validation/
testing/
```

Rules:

- One domain per module.
- Minimize cross-module coupling.
- Avoid circular dependencies.
- Keep interfaces small.

---

# Naming Conventions

## Types

Use PascalCase.

Examples:

```rust
TerrainChunk
ClimateChunk
SimulationClock
WorldConfig
```

---

## Functions

Use snake_case.

Examples:

```rust
generate_terrain()
update_climate()
advance_clock()
```

---

## Constants

Use UPPER_SNAKE_CASE.

Examples:

```rust
DEFAULT_WORLD_SIZE
DEFAULT_CHUNK_SIZE
```

---

## Modules

Use snake_case.

Examples:

```text
terrain.rs
climate.rs
simulation_clock.rs
```

---

# Error Handling Rules

## Recoverable Errors

Use Result.

Example:

```rust
Result<T, ValidationError>
```

---

## Unrecoverable Errors

Use panic only when invariants are violated during development.

Examples:

- invalid coordinate generation
- impossible chunk state
- corrupted test fixtures

---

## Validation Policy

Debug/Test builds:

- panic on invariant violation

Release builds:

- return structured validation errors

This rule must remain consistent with DETERMINISM.md.

---

# Testing Requirements

## Unit Tests

Required for:

- terrain generation
- climate calculations
- resource calculations
- energy calculations
- coordinate conversion

---

## Determinism Tests

Required for:

- world generation
- simulation ticking
- save/load continuation

---

## Invariant Tests

Required for:

- coordinate validity
- climate ranges
- resource ranges
- energy ranges
- chunk consistency

---

## Long-Run Tests

Required before Phase 1 completion.

Purpose:

- detect drift
- detect invariant violations
- detect hidden state corruption

---

# Serialization Rules

## Human Readable First

Phase 1 uses:

- JSON

Optional:

- RON

Do not introduce binary formats unless justified by profiling.

---

## Version Every Snapshot

Every snapshot must include:

- schema version
- world seed
- configuration version

Future migrations depend on this information.

---

## Deterministic Persistence

Saving and loading must preserve simulation behavior.

Required property:

```text
N ticks

must equal

A ticks
+ save
+ load
+ B ticks

where

A + B = N
```

---

# Determinism Protection Rules

## No Wall Clock Logic

Never use:

- current time
- frame duration
- system clock

for simulation outcomes.

Simulation time comes exclusively from:

```rust
SimulationClock
```

---

## No Hidden Randomness

All randomness must come from seeded RNG resources.

Forbidden inside simulation logic:

```rust
thread_rng()
rand::random()
SystemTime::now()
```

---

## Stable Iteration

Do not rely on unordered iteration.

When order affects outcomes:

- sort explicitly
- use stable keys
- document assumptions

---

## Reproducibility First

If a surprising event occurs:

Developers must be able to reproduce it using:

- world seed
- configuration
- tick count

---

# Performance Guidelines

## Correctness Before Optimization

Do not optimize speculative bottlenecks.

Measure first.

Optimize second.

---

## Profile Before Refactoring

Use evidence.

Not intuition.

---

## Avoid Premature Parallelism

Parallel execution is allowed only when determinism is preserved.

Correctness is more important than throughput.

---

## Minimize Allocations In Hot Paths

Repeated allocations inside simulation loops should be avoided only when profiling demonstrates a measurable cost.

Do not optimize blindly.

---

# Documentation Requirements

## Public Types

All public types require documentation comments.

---

## Public Functions

All public functions require documentation comments explaining:

- purpose
- inputs
- outputs
- invariants

---

## Systems

Every ECS system must document:

- what it reads
- what it writes
- assumptions
- execution order requirements

---

## Architectural Changes

Any architectural change must update:

- ARCHITECTURE_BASELINE.md
- TECH_SPEC documents
- related design documents

before or alongside implementation.

Documentation is part of the implementation.

---

# Architecture Change Policy

Before changing architecture:

1. Update documentation.
2. Explain rationale.
3. Explain tradeoffs.
4. Explain determinism impact.
5. Explain persistence impact.

Code must not become the source of truth.

Documentation remains the source of truth.

---

# AI Assistance Rule

AI may:

- generate code
- refactor code
- generate tests
- generate documentation

AI must not make architectural decisions without review.

Architectural authority remains with the project maintainer.

---

# Final Rule

Prefer clarity over cleverness.

Prefer determinism over convenience.

Prefer simplicity over abstraction.

Prefer systems over objects.

Prefer emergence over scripting.

If a design decision violates these principles, reconsider the design.

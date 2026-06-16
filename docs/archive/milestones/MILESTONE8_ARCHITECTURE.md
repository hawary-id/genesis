# Milestone 8 Architecture — Validation Framework

**Date:** 2026-06-13  
**Status:** Approved & Locked  
**Dependencies:** Milestone 1 ✅, Milestone 2 ✅, Milestone 3 ✅, Milestone 4 ✅, Milestone 5 ✅, Milestone 6 ✅, Milestone 7 ✅

---

## Architecture Review

### Relevant Roadmap & Spec References
- **`PHASE1_WORLD_TECH_SPEC.md`:** Lists concrete invariants for chunk consistency, terrain ranges, resource ranges, climate ranges, energy ranges, and time validity. States that validation failures must panic in development/test builds, and release builds must report structured errors and halt/reject progression.
- **`PHASE1_IMPLEMENTATION_PLAN.md`:** Formally places Milestone 8 as the validation centralization phase, specifying panic behavior in debug/test and structured validation error output in release.
- **[ADR-001: ECS Boundaries](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-001-ecs-architectural-boundaries.md):** Validation logic must be implemented inside read-only systems scheduled at explicit boundaries, keeping components/resources as passive data containers.
- **[ADR-002: Deterministic Execution Contract](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-002-deterministic-execution-contract.md):** Validation checks must be deterministic, order-independent, and must not introduce side-effects or external dependencies.

---

## Milestone 8 Scope

### Objectives
Centralize the scheduling and orchestration of simulation invariant checks across the codebase. Milestone 8 introduces a central orchestration layer (`engine/src/validation/`) that schedules and invokes domain validation logic under a unified policy: panic on failure in debug/test builds, and report errors and halt simulation progression in release builds.

### Responsibilities
- Provide a unified structured error model for invariant violations.
- Coordinate execution of existing validation functions from a central module and schedules.
- Enforce the compilation-dependent validation policy (Panic in Debug/Test, Error-reporting and Halting in Release).

### Explicit Non-Goals
- **No rewriting of domain validators:** Subsystem-specific range and value logic remains inside their respective modules.
- **No external telemetry/events:** No events or monitoring layers are introduced.
- **No runtime validation level tuning:** No validation level configuration is introduced.
- **No ECS graph auditing:** Validation checks focus strictly on physical simulation invariants, not ECS graph connectivity.

---

## ECS Architecture

A new module `engine/src/validation` acts as the coordinator. It does not introduce new components, resources, or events.

### Resources
*None.*

### Components
*None.*

### Events
*None.*

### Systems

#### `validate_world_on_startup` [NEW]
*Orchestrates validation checks immediately following the startup world generation sequence.*
- **Reads:** `Res<WorldConfig>`, `Res<WorldBounds>`, `Res<SimulationClock>`, `Query<(&ChunkCoord, &TerrainChunk, &ClimateChunk, &ResourceChunk, &EnergyAvailabilityChunk)>`.
- **Writes:** None.
- **Logic:** Invokes domain validators for terrain, climate, resource, and energy chunk data. In debug/test mode, panics on failure. In release mode, reports the errors and halts simulation progression.

#### `validate_world_on_tick` [NEW]
*Orchestrates validation checks following each simulation tick.*
- **Reads:** `Res<WorldConfig>`, `Res<SimulationClock>`, `Res<SeasonState>`, `Query<(&ChunkCoord, &ClimateChunk, &ResourceChunk, &EnergyAvailabilityChunk)>`.
- **Writes:** None.
- **Logic:** Invokes domain validators for climate, resource, energy, and simulation clock/season states. In debug/test mode, panics on failure. In release mode, reports the errors and halts simulation progression.

---

## Schedule Integration

The systems are integrated into existing schedules to ensure verification runs automatically.

### 1. `StartupGeneration` Schedule
The startup validation system runs at the end of the generation sequence:
```text
...
6. generate_energy_availability_chunks
7. validate_world_on_startup [NEW] (orchestrates initial verification)
8. mark_chunks_generated
9. emit_world_generation_completed
```

### 2. `PostTickValidation` Schedule
The centralized post-tick validation system executes after all update systems run in the `FixedSimulationTick` schedule, replacing subsystem-specific schedule registrations:
```text
Schedule: FixedSimulationTick
├── advance_simulation_clock
├── update_season_state
├── update_climate_fields
├── update_resource_fields
└── update_energy_availability_fields

Schedule: PostTickValidation
└── validate_world_on_tick [NEW] (orchestrates post-tick verification)
```

---

## Validation Framework Design

### 1. Structured Error Representation
A simplified, simulation-focused error representation represents all failure vectors using strongly-typed fields:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationError {
    ClockRegression {
        previous: u32,
        current: u32,
    },

    SeasonStateMismatch {
        total_ticks: u32,
    },

    ChunkInconsistency {
        coord: ChunkCoord,
        detail: &'static str,
    },

    TerrainOutOfBounds {
        coord: ChunkCoord,
        field: &'static str,
        value: f32,
    },

    ClimateOutOfBounds {
        coord: ChunkCoord,
        field: &'static str,
        value: f32,
    },

    ResourceOutOfBounds {
        coord: ChunkCoord,
        field: &'static str,
        value: f32,
    },

    EnergyOutOfBounds {
        coord: ChunkCoord,
        field: &'static str,
        value: f32,
    },
}
```

### 2. Debug vs. Release Validation Policy
To satisfy the development vs. production performance and diagnostics requirements:

* **Debug & Test Targets (`#[cfg(debug_assertions)]` or `#[cfg(test)]`):**
  - Validation systems will immediately trigger a standard `panic!` containing a detailed message (e.g. location, coordinates, cell index, config value, and invalid value) upon detecting a `ValidationError`.

* **Release & Production Targets (`#[cfg(not(debug_assertions))]`):**
  - Validation systems will return/report structured validation errors (e.g., logging them or storing them in a dedicated return buffer) and halt simulation progression (e.g., by transitioning the runner state or using a run condition to disable future ticks).
  - **Ownership Clarification:** Milestone 8 defines validation execution and error generation only. The mechanism used by future runner, persistence, orchestration, server, or application layers to surface release validation failures is intentionally outside the scope of Phase 1. This prevents accidental introduction of future infrastructure requirements.

---

## Validation Module Structure

The centralized framework is structured as follows:

```text
engine/src/validation/
├── mod.rs        (re-exports and integration hooks)
├── errors.rs     (defines ValidationError)
└── systems.rs    (validate_world_on_startup and validate_world_on_tick)
```

Existing domain validators remain owned by and located within their respective files:
* `world/terrain.rs`
* `world/climate.rs`
* `world/resource.rs`
* `world/energy.rs`
* `time/season_state.rs`

---

## Migration Strategy

This milestone does **NOT** rewrite existing validation logic.

1. **Keep Ownership with Domains:** Existing validation functions remain defined in, and owned by, their respective files.
2. **Centralize Scheduling:** The new `validation` module acts strictly as an orchestration layer. Instead of subsystem modules directly registering their own validation systems to the `PostTickValidation` or `StartupGeneration` schedules, the unified systems (`validate_world_on_startup` and `validate_world_on_tick`) are registered.
3. **Delegate Invariant Checks:** These centralized systems retrieve the necessary ECS components and resources, delegate actual cell-by-cell and range verification to the existing domain validators, and map any returned failures to the unified `ValidationError` type.
4. **Enforce Uniform Policy:** The centralized systems wrap the delegation calls in a unified compilation-dependent check block (panic immediately in debug/test, report and halt in release), guaranteeing a consistent error handling contract across all systems.

---

## Determinism Analysis

- **Read-Only Access:** All validation systems query chunk components and time resources via read-only guards (`Res` and `&`). This enforces that validation execution does not cause side-effects.
- **Deterministic Error Collection:** In release builds, multiple validation checks might fail. To ensure deterministic reporting, validation errors are collected sequentially according to a fixed chunk coordinate scan order (e.g., sorted by coordinate row-major layout) and cell-index order.
- **Deterministic Bounds Comparison:** Value limits are evaluated using strict inequality checks against constant configuration parameters, guaranteeing identical verification behavior across execution environments.

---

## Risk Analysis

### 1. Performance Overhead
- **Risk:** Exhaustively sweeping large cell arrays (e.g., `512 x 512 = 262,144` cells) every tick in `PostTickValidation` will slow down execution during long runs.
- **Mitigation:**
  - Since terrain values are static, terrain validation is only executed once during `validate_world_on_startup` and is skipped entirely during post-tick validation.
  - Climate, resource, and energy updates only occur on daily boundaries. Post-tick cell-level checks can be optimized to execute only on daily boundaries rather than every simulation hour.

### 2. Graceful Halting in Release
- **Risk:** If a validation error is encountered in release, the simulation runner might proceed anyway, propagating corrupt state.
- **Mitigation:**
  - Introduce a lightweight halting mechanism (e.g. setting an execution run state or utilizing a simple flag) that run conditions check before executing schedules, ensuring tick execution terminates cleanly.

---

## Open Questions

1. **Should daily validation checks be governed by run conditions?**
   - *Recommendation:* Yes, checking the simulation clock daily boundary is an efficient way to restrict sweeping checks to only the tick steps that perform updates.

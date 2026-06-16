# Milestone 2 Review — Core ECS Setup

**Date:** 2026-06-13
**Status:** Complete
**Authoritative specification:** `docs/MILESTONE2_ARCHITECTURE.md`

---

## Files Created

### `engine/src/config/world_bounds.rs`

`WorldBounds` ECS resource. Derived from `WorldConfig` during startup.
Stores validated coordinate boundaries: `world_width`, `world_height`, `chunks_x`, `chunks_y`, `chunk_size`.

Contains 3 unit tests:
- `world_bounds_from_default_config` — verifies 512×512 world produces 16×16 chunk grid
- `world_bounds_from_test_config` — verifies 256×256 world produces 8×8 chunk grid
- `chunk_count_covers_entire_world` — verifies chunk coverage equals total cell count

### `engine/src/app/events.rs`

`WorldGenerationCompleted` event type definition only.
No emitter or consumer systems exist. Per `MILESTONE2_ARCHITECTURE.md`, this event is used in Milestone 3.

---

## Files Modified

### `engine/src/config/mod.rs`

Added `pub mod world_bounds` and `pub use world_bounds::WorldBounds`.

### `engine/src/app/schedules.rs`

Replaced the Milestone 1 placeholder `Schedules` stub with:

- Five `ScheduleLabel` types in canonical execution order:
  1. `StartupGeneration`
  2. `FixedSimulationTick`
  3. `PostTickValidation`
  4. `PersistenceBoundary`
  5. `ObservationBoundary`
- `register_schedules(world: &mut World)` — registers all five schedules via `world.add_schedule`
- 6 unit tests verifying each schedule is registered and all five together

Import decision: `ScheduleLabel` derive macro must be imported as
`use bevy_ecs::schedule::ScheduleLabel` — not from `bevy_ecs::prelude`. The derive macro
`ScheduleLabel` is re-exported from `bevy_ecs_macros` via `bevy_ecs::schedule`, not via
`bevy_ecs::prelude`. Using `bevy_ecs::prelude::*` does not bring the derive into scope.

### `engine/src/app/plugins.rs`

Updated `register_initial_resources` to:
1. Derive `WorldBounds` from `&config` before `config` is moved into `world.insert_resource`.
2. Insert `WorldBounds` as a fourth ECS resource.
3. Call `register_schedules(world)` to register all five Phase 1 schedule labels.

The doc comment is updated to reflect Milestone 2 scope.

### `engine/src/app/mod.rs`

- Added `pub mod events` declaration.
- Added `pub use events::WorldGenerationCompleted` re-export.
- Added `pub use schedules::{...}` re-exports for all five schedule labels.
- Updated module-level doc comment to describe Milestone 2 scope.
- Added 6 integration tests in `#[cfg(test)]`:
  - `app_initializes_world_config`
  - `app_initializes_world_seed`
  - `app_initializes_simulation_clock`
  - `app_initializes_world_bounds`
  - `app_registers_all_phase1_schedules`
  - `world_bounds_consistent_with_world_config`

---

## Architectural Decisions

### WorldBounds placement: `config/` module

`WorldBounds` is placed in `engine/src/config/world_bounds.rs` because it is derived from
`WorldConfig` and represents stable coordinate configuration. The `config` module holds
world-wide configuration data. `WorldBounds` is config-derived, immutable after startup,
and naturally belongs alongside `WorldConfig`.

Alternative considered: `app/` module. Rejected because `app/` holds bootstrap and
scheduling concerns. `WorldBounds` is data, not behavior.

### Schedule registration: inside `register_initial_resources`

`register_schedules` is called from `register_initial_resources` (in `plugins.rs`), making
`App::new()` the single initialization call that produces a fully structured ECS world.

Alternative considered: separate `App::new()` calls to `register_initial_resources` and
`register_schedules`. Rejected to keep the bootstrap surface minimal. `App::new()` already
calls `plugins::register_initial_resources`, and calling schedule registration from there
maintains one clear initialization path.

### Event type only — no `Events<T>` resource registration

`WorldGenerationCompleted` is a type definition only. The `Events<WorldGenerationCompleted>`
resource is NOT registered in Milestone 2. Per `MILESTONE2_ARCHITECTURE.md`:

> Events are type definitions only. No systems that emit or consume these events are added
> until those systems are introduced in their respective milestones.

The `Events<T>` resource will be registered in Milestone 3 when the emitter system is introduced.

### `SnapshotRequested` and `SnapshotCompleted` deferred

Per `MILESTONE2_ARCHITECTURE.md` section "Events Deferred", these events are intentionally
excluded from Milestone 2 and belong to Milestone 9. They are not defined here.

### `SeasonState`, `GenerationState`, `SnapshotConfig` not introduced

Per milestone scope and the explicit prohibition in the task specification, these resources
are deferred to later milestones.

---

## Deviations from Architecture

**None.**

All deliverables match `MILESTONE2_ARCHITECTURE.md` exactly:

| Deliverable | Specified | Delivered |
|---|---|---|
| `WorldBounds` resource | ✅ | ✅ |
| `WorldGenerationCompleted` event type | ✅ | ✅ |
| `StartupGeneration` schedule label | ✅ | ✅ |
| `FixedSimulationTick` schedule label | ✅ | ✅ |
| `PostTickValidation` schedule label | ✅ | ✅ |
| `PersistenceBoundary` schedule label | ✅ | ✅ |
| `ObservationBoundary` schedule label | ✅ | ✅ |
| Tests verifying ECS world initialization | ✅ | ✅ |
| Tests verifying schedule registration | ✅ | ✅ |
| No component definitions | ✅ | ✅ |
| No simulation execution | ✅ | ✅ |
| No placeholder systems | ✅ | ✅ |
| No `SeasonState`, `GenerationState`, `SnapshotConfig` | ✅ | ✅ |
| No `SnapshotRequested`, `SnapshotCompleted` | ✅ | ✅ |

---

## Risks

### ScheduleLabel derive requires explicit import path

The `ScheduleLabel` derive macro in `bevy_ecs 0.13.2` must be imported via
`use bevy_ecs::schedule::ScheduleLabel`, not via `use bevy_ecs::prelude::*`.
The prelude glob does not re-export the derive macro. This is documented in code
and should be kept in mind when adding schedule labels in future milestones.

### WorldBounds must be derived before WorldConfig is consumed

In `plugins.rs`, `WorldBounds::from_config(&config)` must be called before
`world.insert_resource(config)` consumes `config` by move. This order is correct
in the current implementation and must remain correct in future refactors.

### Empty schedules are structurally correct but must stay empty until their milestone

The five Phase 1 schedule labels are registered and empty. Future milestones must add
systems only to the schedule appropriate to their functionality. Adding systems to the
wrong schedule, or adding systems before their prerequisite state exists, would violate
determinism requirements and the pressure-driven development rule.

### f32 / floating-point determinism is not yet a concern

No continuous environmental fields exist yet. This risk becomes relevant in Milestone 3
(terrain generation) and must be managed at that point.

---

## Test Results

```
running 19 tests
test app::schedules::tests::fixed_simulation_tick_is_registered ... ok
test app::schedules::tests::startup_generation_is_registered ... ok
test app::schedules::tests::all_five_phase1_schedules_are_registered ... ok
test app::schedules::tests::persistence_boundary_is_registered ... ok
test app::schedules::tests::observation_boundary_is_registered ... ok
test app::schedules::tests::post_tick_validation_is_registered ... ok
test app::tests::app_initializes_world_bounds ... ok
test app::tests::app_initializes_world_config ... ok
test app::tests::app_initializes_simulation_clock ... ok
test app::tests::app_initializes_world_seed ... ok
test app::tests::world_bounds_consistent_with_world_config ... ok
test app::tests::app_registers_all_phase1_schedules ... ok
test config::world_bounds::tests::chunk_count_covers_entire_world ... ok
test config::world_bounds::tests::world_bounds_from_default_config ... ok
test config::world_bounds::tests::world_bounds_from_test_config ... ok
test config::world_config::tests::default_config_is_valid ... ok
test rng::seed::tests::default_seed_is_zero ... ok
test rng::seed::tests::world_seed_creation ... ok
test time::simulation_clock::tests::simulation_clock_defaults_are_valid ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**cargo fmt:** clean
**cargo check:** clean (exit 0, no warnings)
**cargo test:** 19/19 passed

Tests added in this milestone: **16** (3 in `world_bounds`, 6 in `schedules`, 6 in `app::mod`).
Tests inherited from Milestone 1: **3** (`world_config`, `seed`, `simulation_clock`).

---

## Readiness for Milestone 3

Milestone 3 requires:

- `ChunkCoord`, `TerrainChunk`, `ClimateChunk`, `ResourceChunk`, `EnergyAvailabilityChunk`, `DirtyChunk`, `Generated` component definitions
- Chunk entity spawning via `StartupGeneration` schedule
- Coordinate conversion utilities (`WorldCoord`, `ChunkCoord`, `LocalCoord`)
- Terrain generation (elevation, slope, water, soil depth, soil fertility)
- Seed stream derivation from `WorldSeed`

**Prerequisites from Milestone 2 that Milestone 3 depends on:**

| Prerequisite | Status |
|---|---|
| `WorldConfig` resource (world dimensions, chunk size) | ✅ Available |
| `WorldSeed` resource (root seed for generation) | ✅ Available |
| `WorldBounds` resource (derived chunk counts for spawning) | ✅ Available |
| `StartupGeneration` schedule label (for generation systems) | ✅ Registered |
| `WorldGenerationCompleted` event type (emitted at end of generation) | ✅ Defined |

**Milestone 3 must not begin until Milestone 2 is accepted as complete.**

No known blockers for Milestone 3.

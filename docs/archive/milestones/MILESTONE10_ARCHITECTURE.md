# Milestone 10 Architecture — Determinism Testing

**Date:** 2026-06-14  
**Status:** Approved & Locked  
**Dependencies:** Milestone 1 ✅, Milestone 2 ✅, Milestone 3 ✅, Milestone 4 ✅, Milestone 5 ✅, Milestone 6 ✅, Milestone 7 ✅, Milestone 8 ✅, Milestone 9 ✅

---

## 1. Objective

The primary objective of Milestone 10 is to verify and guarantee the deterministic execution of the Genesis simulation engine. This verification establishes that the Environmental Substrate (Phase 1, Milestones 1–9) is stable, correct, and reproducible. Specifically:
- **Generation Determinism:** Generating a world from the same seed and configuration must yield binary-identical initial states, while different seeds must generate distinct worlds.
- **Ticking Determinism:** Advancing a generated world for a given number of ticks must result in identical state changes.
- **Save/Load Equivalence:** Snapping, serializing, and reloading a world state must result in continuation states that are binary-identical to uninterrupted simulation runs.
- **Long-Run Stability:** The engine must run default configuration worlds for long durations (up to 1 simulation year (8,640 ticks)) without invariant failure or numerical drift.

This milestone is strictly verification-only. No new simulation mechanics, features, resources, or scheduling boundaries are added.

---

## 2. Roadmap Requirements

According to [PHASE1_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE1_IMPLEMENTATION_PLAN.md#L543-L575), the deliverables for Milestone 10 are:
- Deterministic generation tests.
- Deterministic ticking tests.
- Save/load equivalence tests.
- Long-run stability tests.
- Test fixtures for `256 x 256` world configuration.
- Default-world long-run test for `512 x 512` world configuration.

The criteria for success are:
- Same seed + config => identical world.
- Different seeds => different world.
- Same tick count => identical final state.
- $N$ ticks continuous run matches $A$ ticks + save/load + $B$ ticks run (where $A + B = N$).
- Long-run deterministic stability.

---

## 3. Tech Spec Requirements

According to [PHASE1_WORLD_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE1_WORLD_TECH_SPEC.md#L943-L995), the verification plan must validate:
- **Deterministic Generation:** Same seed + config produce identical terrain, climate, resource, and energy availability chunk values. Different seeds produce different chunk values. Stable generation order.
- **Deterministic Ticking:** Running the same generated world for the same tick count produces identical final states. Clock advances exactly one tick per simulation step. Season state changes only according to config.
- **Save/Load Equivalence:** Continuous ticking yields identical binary outcomes to a split save/load run. Snapshot preserves all state required for continuation without mutating simulation state.
- **Invariant Validation:** Validation checks coordinates, chunk count, climate/resource/energy field ranges, and simulation clock monotonicity.
- **Long-Run Stability:** A default `512 x 512` world runs for a target of **1 simulation year** (8,640 ticks) without invariant failure, field drift, or loss of save/load validity.

---

## 4. Current Codebase Assessment

The current codebase is highly prepared for determinism verification:
- **Sequential Schedule Ordering:** The schedule registration in [app/schedules.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/app/schedules.rs) defines a strict sequential pipeline: `StartupGeneration` -> `FixedSimulationTick` -> `PostTickValidation` -> `PersistenceBoundary` -> `ObservationBoundary`. In [app/mod.rs:L48-L57](https://github.com/hawary-id/genesis/blob/main/engine/src/app/mod.rs#L48-L57), ticking systems are explicitly ordered using Bevy `.after()` constraints, completely eliminating schedule-based race conditions.
- **Coordinate-Salted RNG:** Pseudo-randomness in [world/terrain.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/terrain.rs) and [world/resource.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/resource.rs) is derived locally from coordinate-salted hashes via `rand_chacha`, meaning chunk values are independent of processing order.
- **Stable Serialization Layout:** The persistence module in [persistence/io.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/io.rs) sorts chunk snapshots by `(coord.y, coord.x)` before JSON serialization, preventing Bevy ECS query iteration order from producing non-deterministic snapshot structures.
- **Existing Verification Logic:** The codebase has a unit test for save/load equivalence (`save_load_equivalence` in [persistence/io.rs:L852-L919](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/io.rs#L852-L919)) which tests $A+B=N$ equivalence on a `256 x 256` configuration. However, this is currently a unit test on a test configuration, rather than a broad integration test covering all requirements and scales.

---

## 5. Existing Determinism Coverage

The following tests in the current test suite satisfy Milestone 10 determinism validation:

| Test Name | File Location | Verified Determinism Element |
| :--- | :--- | :--- |
| `generation_is_deterministic` | [world/terrain.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/terrain.rs) | Verifies that a terrain chunk is generated identically on identical seeds. |
| `different_seeds_produce_different_terrain` | [world/terrain.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/terrain.rs) | Verifies that different seeds generate distinct elevation profiles. |
| `value_noise_is_deterministic` | [world/resource.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/resource.rs) | Verifies that mineral value noise generation is deterministic. |
| `chunk_seed_derivation_is_deterministic` | [rng/seed.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/rng/seed.rs) | Verifies that coordinate-salted seed derivation is reproducible. |
| `deterministic_season_state_generation` | [time/simulation_clock.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/time/simulation_clock.rs) | Verifies that season updates are deterministic across tick runs. |
| `season_state_reconstruction` | [time/simulation_clock.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/time/simulation_clock.rs) | Verifies that season state can be reconstructed deterministically on load. |
| `build_snapshot_is_deterministic` | [persistence/io.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/io.rs) | Verifies that compiling the snapshot structure is deterministic. |
| `save_load_equivalence` | [persistence/io.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/io.rs) | Verifies $A+B=N$ ticks save/load continuation equivalence for `256 x 256`. |
| `persistence_does_not_mutate_state` | [persistence/systems.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/systems.rs) | Verifies that taking a snapshot has no side effects on simulation state. |

---

## 6. Gap Analysis

The following roadmap and spec requirements are currently missing from the test coverage:

1.  **Full-World Generation Determinism Test:** We have a unit test for single terrain chunk determinism, but we do not verify that the *entire generated world* (terrain, climate, resources, and energy) is binary-identical when generated across separate ECS apps.
2.  **Full-World Different Seeds Test:** We lack an integration test asserting that different seeds generate distinct climate, resource, and energy layers across a full world (terrain elevation is currently the only field verified).
3.  **Full-World Ticking Determinism Test:** We lack a test asserting that running two identical worlds for $N$ ticks in parallel results in binary-identical final states.
4.  **Long-Run Stability and Drift Test:** No tests execute the simulation over a long sequence of ticks (e.g. 1 simulation year, i.e., 8,640 ticks) to assert invariant preservation, clock monotonicity, and confirm the absence of continuous field drift.

---

## 7. Determinism Risk Analysis

The table below outlines the primary architectural risks to determinism in Genesis:

| Risk Source | Severity | Impact | Mitigation Status / Recommendation |
| :--- | :--- | :--- | :--- |
| **Floating-Point Math Variations** | Medium | Compiler target optimizations or instruction set selection (e.g., FMA) can cause slight variations in `f32` vectors across platforms. | **Mitigated:** The update logic uses basic math expressions and clamps all outputs. Epsilon-based approximations are avoided in core assertions to ensure bit-perfect determinism on the primary compilation target. |
| **ECS Iteration Ordering** | High | Bevy ECS query iteration order is unstable. If chunk update logic depends on neighbor states, iteration order will cause state drift. | **Mitigated:** All simulation updates in Phase 1 are cell-local and chunk-local. There are no spatial interactions or neighbor references. Persistence and validation sort chunk entities by coordinate. |
| **RNG Call Order Drift** | High | Using a shared mutable RNG state introduces dependency on system update order. | **Mitigated:** Seeding is local and coordinate-salted (via `rand_chacha`), removing order sensitivity. |
| **Save/Load Continuation** | High | Reconstructing derived values or omitting active fields would break continuation. | **Mitigated:** All authoritative world state is persisted. Derived state (`SeasonState`, `WorldBounds`) is reconstructed deterministically from clock ticks and configuration on load. |
| **Schedule Ordering** | High | Unconstrained system schedules can run updates in different orders across ticks. | **Mitigated:** Schedules are explicitly registered and systems are sequentially chained using Bevy `.after()` dependencies. |

---

## 8. Design Decisions

The following decisions lock the execution of Milestone 10:
- **No new simulation systems:** The simulation logic in `world/` remains untouched. No new mechanics or simulation systems are introduced.
- **No new resources:** No new ECS resources are registered.
- **No new schedules:** No new scheduling boundaries are introduced.
- **Determinism proven through tests only:** All requirements are verified through automated integration and unit tests.
- **Existing save/load equivalence remains authoritative:** The `assert_worlds_equivalent` assertion helper in [persistence/io.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/io.rs) is the authoritative check for world state equality.
- **Absolute Float Equality:** Assertions comparing continuous environmental fields (`f32` arrays) must check for absolute binary equality (`assert_eq!`) rather than epsilon-based checks.
- **In-Memory Fixtures:** Rather than checking in massive JSON snapshot files to the repository (which bloats git history), test configurations and reference worlds are generated in memory from fixed seeds.

---

## 9. Required Test Additions

To fulfill all requirements, the following integration tests will be implemented in `engine/src/testing/`:

1.  **Full-World Generation Determinism Test:** Generates two worlds using the same seed, runs startup generation, and asserts that all components and resources are binary-equivalent.
2.  **Full-World Seed Sensitivity Test:** Generates two worlds with different seeds and asserts that terrain, climate, resources, and energy fields are not equal.
3.  **Full-World Ticking Determinism Test:** Generates two identical worlds, ticks both for 100 ticks, and asserts that they remain binary-identical.
4.  **Long-Run Stability Test (512x512):** Runs the default world configuration for `8,640` ticks (1 simulation year), verifying at each day boundary that `PostTickValidation` passes, clock ticks are monotonic, and no continuous fields drift out of bounds. Additionally, at the end of the run, a snapshot must be saved and successfully loaded/validated to satisfy the "save/load remains valid after long runs" requirement. This test is marked `#[ignore]` by default to keep standard cargo runs fast, but is executed in CI/CD.

---

## 10. Explicit Non-Goals

The following items are explicitly out of scope for Milestone 10:
- New simulation features (weather events, erosion, resource consumption).
- New persistence features (binary/compressed formats, PostgreSQL integration).
- New ECS schedules or boundaries.
- New ECS resources.
- New world layers or entities.

---

## 11. Verification Plan

The verification process consists of the following steps:
1.  Verify compilation:
    ```powershell
    cargo clippy -- -D warnings
    cargo fmt --check
    ```
2.  Run standard test suite:
    ```powershell
    cargo test
    ```
3.  Run long-run stability integration tests:
    ```powershell
    cargo test -- --ignored
    ```
All tests must compile and pass with zero warnings, errors, or invariant failures.

---

## 12. Design Lock Verdict

**APPROVED & LOCKED.**

The architecture has been implemented and verified.

All Milestone 10 determinism requirements have been validated through automated testing, including:

- Full-world generation determinism
- Full-world seed sensitivity
- Full-world ticking determinism
- Save/load equivalence
- Long-run stability verification (8,640 ticks / 1 simulation year)

The implementation compiles cleanly, passes formatting and lint verification, and introduces no architectural deviations from the approved design.

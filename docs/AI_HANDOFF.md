# AI Handoff Briefing

This document serves as the immediate handoff instructions for any AI model resuming development on Project Genesis.

---

## Current Repository State

* **Branch:** `main`
* **Status:** Clean. All Phase 2 modules compile under standard profiles, and formatting (`cargo fmt`) and Clippy checks pass successfully.
* **Test Outcome:** `124 passed, 0 failed, 1 ignored` (standard); `1 passed, 0 failed` (`--ignored` stability test).

---

## Current Phase & Milestone

* **Current Phase:** Phase 3 — Evolution (Transitioning from Phase 2)
* **Current Milestone:** Phase 2 Complete (v0.2.0-phase2 release candidate)
* **Phase Progress:** Milestone 15 completed, verified, and locked.

---

## Recent Major Changes

* **Milestone 15 Completed:** Implemented agent persistence and integration testing.
  - Snapshot schema version upgraded to `2` to include the `StableIdGenerator` and `agents` collections.
  - Mapped agent metadata, position, and metabolic stock to snapshot formats.
  - Restored `StableIdGenerator` resource and spawned reconstructed agent entities with default `ActionRequest(None)` to prevent query freezes.
  - Updated `assert_worlds_equivalent` to perform structural comparisons of both the generator resource and live agent collections sorted by stable identifier.
  - Verified A+B=N save/load split ticking equivalence through standard integration tests and the year-long `test_long_run_stability_512` stability test.
* **Milestone 14 Completed:** Implemented spatial movement execution. Agents step cardinally, validating against world boundaries, slopes, and water zones.

---

## Completed Milestone 15 Summary

### Completed Work
* Snapshot schema version 2 added
* `StableIdGenerator` resource serialization and load reconstruction completed
* Agent metadata, coordinates, and metabolic stock serialization and reconstruction completed
* Deterministic sorting by stable ID completed
* World equivalence utility (`assert_worlds_equivalent`) updated to compare ID generators and sorted agent populations
* `save_load_equivalence` test updated to verify agent saving and loading
* `test_long_run_stability_512` stability test updated to verify year-long split run equivalence with agents enabled

---

## Known Risks & Design Constraints

* **Startup Validation Mismatch on Restored Worlds:**
  - `validate_world_on_startup` is designed to validate freshly generated worlds at tick `0` (asserting `initial_agent_count`, agent ages `== 0`, and agent energy `== initial_agent_energy`).
  - This system must **not** be executed on reconstructed worlds loaded from snapshots at tick $>0$, as age bounds and count expectations will fail.
  - This is a known architectural risk to be redesigned in subsequent validation updates.
* **Float Target Optimization Divergence:** Vector mathematics in the environment are calculated using standard `f32` floats. Compile-time optimizations (like contract/FMA) might introduce minor divergence on target CPUs other than the test host.

---

## Open Problems & Technical Debt

* **Known Technical Debt (Metabolism/Sensing/Movement O(N) Lookup):** ClimateChunk and TerrainChunk lookups in agent systems currently perform linear chunk scans (O(agent_count × chunk_count)). This is intentionally accepted for Phase 2 correctness-first implementation and should be replaced with indexed chunk lookup (HashMap or equivalent deterministic spatial index) during a future performance optimization pass.
* **Manual Array Indexing:** Systems calculate 1D cell offsets manually from 2D coordinates: `local_y * chunk_size + local_x`. This requires introducing an abstract `ChunkField<T>` struct to avoid index out-of-bounds risks in future phases.
* **Synchronous Serialization Blocking:** Snapshot loading and saving operations are run synchronously on the Bevy main loop, which will block frames during large world persistence in a client interface context.

---

## Current Development Target

Begin Phase 3 (Evolution) planning to design genetics, mutation, inheritance, and natural selection.

## Recommended Next Actions

*   Confirm that standard tests and clippy compile cleanly: `cargo test` and `cargo clippy -- -D warnings`.
*   Formulate the `docs/PHASE3_EVOLUTION_TECH_SPEC.md` and `docs/PHASE3_IMPLEMENTATION_PLAN.md` roadmap.
*   Tag the repository with release marker `v0.2.0-phase2`.

---

## Important Files

*   **AI Briefing Entrypoint:** [GENESIS_CONTEXT.md](https://github.com/hawary-id/genesis/blob/main/docs/GENESIS_CONTEXT.md)
*   **Current State:** [CURRENT_STATE.md](https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md)
*   **Architecture Decisions Registry:** [ARCHITECTURE_DECISIONS.md](https://github.com/hawary-id/genesis/blob/main/docs/ARCHITECTURE_DECISIONS.md)
*   **Milestone Progress Registry:** [MILESTONE_STATUS.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE_STATUS.md)
*   **Integration Tests:** [testing/determinism.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/determinism.rs)

---

## Standard Prompt For Future AI Sessions

> "Read [GENESIS_CONTEXT.md](https://github.com/hawary-id/genesis/blob/main/docs/GENESIS_CONTEXT.md) and [CURRENT_STATE.md](https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md) first and use them as the authoritative source of truth. Follow [ARCHITECTURE_DECISIONS.md](https://github.com/hawary-id/genesis/blob/main/docs/ARCHITECTURE_DECISIONS.md) before making architectural changes. Use [MILESTONE_STATUS.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE_STATUS.md) to determine project progress. Do not modify source code or introduce new architectures unless explicitly requested."

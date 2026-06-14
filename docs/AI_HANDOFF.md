# AI Handoff Briefing

This document serves as the immediate handoff instructions for any AI model resuming development on Project Genesis.

---

## Current Repository State

* **Branch:** `main` (hash `cf6e76f`)
* **Status:** Clean. All Phase 1 modules compile under standard profiles, and formatting (`cargo fmt`) and Clippy checks pass successfully.
* **Test Outcome:** `98 passed, 0 failed, 1 ignored` (standard); `1 passed, 0 failed` (`--ignored` stability test).

---

## Current Phase & Milestone

* **Current Phase:** Phase 1 — World Substrate
* **Current Milestone:** Milestone 10 — Determinism Testing
* **Phase Progress:** 100% (Substrate generation, simulation ticking, validation checks, serialization persistence, and integration testing complete).

---

## Recent Major Changes

* **Integration Test Hardening:** Refactored chunk querying inside `test_full_world_seed_sensitivity` to map extracted environmental components to a coordinate-keyed `HashMap<ChunkCoord, ChunkData>`, eliminating iteration ordering dependencies on Bevy ECS.
* **Architecture Verification:** Aligned tick-to-year ratios in long-run tests (`test_long_run_stability_512`), validating 1 simulation year (8,640 ticks) instead of incorrect 10-year descriptions.
* **Locked Architecture Docs:** Locked and approved Milestone 10 documentation to serve as the baseline context.

---

## Known Risks

* **Float Target Optimization Divergence:** Vector mathematics in the environment are calculated using standard `f32` floats. Host-specific compile optimizations (like FMA instruction contraction) might cause minor floating-point divergence on target CPUs other than the primary test platform.

---

## Open Problems & Technical Debt

* **Manual Array Indexing:** Systems calculate 1D cell offsets manually from 2D coordinates: `local_y * chunk_size + local_x`. This requires introducing an abstract `ChunkField<T>` struct to avoid index out-of-bounds risks in future phases.
* **Synchronous Serialization Blocking:** Snapshot loading and saving operations are run synchronously on the Bevy main loop, which will block frames during large world persistence in a client interface context.
* **Imperative Bootstrapping:** App initialization uses procedural helper functions (`register_initial_resources`) rather than standard Bevy plugin traits (`Plugin`), violating standard Bevy ecosystem idioms.

---

## Current Development Target

Transition to Phase 2 (Life), establishing core agent entities, environmental query APIs, and metabolic ticking loops.

## Current Open Tasks

1. **Agent Entity Definition:** Declare the `Agent` component and design its data layout (metabolic rates, age, coordinates).
2. **Environment Query API:** Implement helper functions that allow agents to query environmental resources (e.g., fresh water, nutrients) from chunk components based on world coordinate lookups.
3. **Agent Metabolism schedule:** Bind systems under the `FixedSimulationTick` schedule to decrement agent energy, update agent age, and trigger agent death on exhaustion.

## Blockers

None. The Environmental Substrate (Phase 1) is fully verified, compilable, and locked.

## Human Decisions Pending

Formal review and design approval for Phase 2 agent specifications, specifically the exact genomic encoding for evolution (planned for Phase 3 but requiring initialization foresight).

## Safe Next Actions

* Spawn a dummy `Agent` entity on the world grid during startup and verify its coordinates.
* Write a read-only query helper that maps agent grid coordinates to target `ChunkCoord` and retrieves `ResourceChunk` nutrient values.

---

## Important Files

* **AI Briefing Entrypoint:** [GENESIS_CONTEXT.md](https://github.com/hawary-id/genesis/blob/main/docs/GENESIS_CONTEXT.md)
* **Current State:** [CURRENT_STATE.md](https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md)
* **Architecture Decisions Registry:** [ARCHITECTURE_DECISIONS.md](https://github.com/hawary-id/genesis/blob/main/docs/ARCHITECTURE_DECISIONS.md)
* **Milestone Progress Registry:** [MILESTONE_STATUS.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE_STATUS.md)
* **Repository Structure Map:** [PROJECT_STRUCTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/PROJECT_STRUCTURE.md)
* **Integration Tests:** [testing/determinism.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/determinism.rs)

---

## Standard Prompt For Future AI Sessions

> "Read [GENESIS_CONTEXT.md](https://github.com/hawary-id/genesis/blob/main/docs/GENESIS_CONTEXT.md) and [CURRENT_STATE.md](https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md) first and use them as the authoritative source of truth. Follow [ARCHITECTURE_DECISIONS.md](https://github.com/hawary-id/genesis/blob/main/docs/ARCHITECTURE_DECISIONS.md) before making architectural changes. Use [MILESTONE_STATUS.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE_STATUS.md) to determine project progress. Do not modify source code or introduce new architectures unless explicitly requested."

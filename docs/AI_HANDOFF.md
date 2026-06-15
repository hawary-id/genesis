# AI Handoff Briefing

This document serves as the immediate handoff instructions for any AI model resuming development on Project Genesis.

---

## Current Repository State

* **Branch:** `main`
* **Status:** Clean. All Phase 3 Evolution Milestone 17 modules compile under standard profiles, and formatting (`cargo fmt`) and Clippy checks pass successfully.
* **Test Outcome:** `131 passed, 0 failed, 1 ignored` (standard); `1 passed, 0 failed` (`--ignored` stability test).

---

## Current Phase & Milestone

* **Current Phase:** Phase 3 — Evolution (Active)
* **Current Milestone:** Milestone 18 — Reproduction, Inheritance & Lineage (Active)
* **Last Completed Milestone:** Milestone 17 — Resource Consumption (Eating & Drinking)
* **Phase Progress:** Milestone 17 completed, verified, and locked. Milestone 18 is active.

---

## Recent Major Changes

* **Milestone 17 Completed:** Implemented resource consumption.
  - Implemented the `process_agent_consumption` system to harvest cell nutrients and fresh water resources.
  - Subtracted harvested quantities from chunk resource cells and replenished agent `MetabolicStock` energy (clamped at `agent_energy_max`).
  - Added dietary preference and consumption efficiency scaling (herbivore/hydrator preference).
  - Ensured determinism by sorting agent list by `AgentMetadata.id` ascending before processing.
  - Added config fields `max_harvest_rate` and `consumption_efficiency` to `WorldConfig` and testing configurations.
  - Added unit tests checking nutrient/water consumption, diet preference scaling, energy clamping, resource depletion, resource non-negativity, and coordinate order determinism.
* **Milestone 16 Completed:** Implemented genetics and phenotype mapping.
  - Implemented `Genome`, `Phenotype`, and `LineageMetadata` components and `GenomeConfig` resource.
  - Implemented dynamic phenotype caching derived on spawn (`derive_phenotype_on_spawn` system).
  - Upgraded snapshot version to schema version `3` to serialize `Genome` and `LineageMetadata`, dynamically re-deriving `Phenotype` on load.
  - Implemented genetics validation (bounds checking and lineage validity).
  - Updated tests and validation systems to allow running on restored worlds at tick $>0$ without initial spawner constraints failing.
  - Added genome mapping mapping, serialization round-trip, and phenotype reconstruction unit tests.
* **Milestone 15 Completed:** Implemented agent persistence and integration testing.

---

## Completed Milestone 17 Summary

### Features Delivered
*   `process_agent_consumption` system (performs local grid resource harvesting and agent energy replenishment).
*   Diet preference and consumption efficiency scaling formulas.
*   Stable order execution using `AgentMetadata.id` ascending (ADR-002 compliance).
*   Mass conservation subtraction from `ResourceChunk` arrays.
*   Config fields `max_harvest_rate` and `consumption_efficiency` added to `WorldConfig`.

### Files Modified
*   `engine/src/agent/mod.rs`
*   `engine/src/agent/systems.rs`
*   `engine/src/app/mod.rs`
*   `engine/src/config/world_config.rs`
*   `engine/src/testing/fixtures.rs`

### Test Results & Determinism Status
*   All 131 standard tests and clippy checks pass.
*   Ignored stability test `test_long_run_stability_512` passes, proving save/load determinism holds bit-perfectly over 1 simulation year with consumption enabled.

### Technical Debt Notes
*   Consumption lookup iterates O(agents * chunks) linearly to resolve chunk coordinates instead of caching or reusing sensing API helpers (accepted debt matching movement systems design).

---

## Completed Milestone 16 Summary

### Completed Work
* Snapshot schema version 3 added
* `Genome` and `LineageMetadata` components and `GenomeConfig` resource added
* `derive_phenotype_on_spawn` system added to cache dynamic phenotypes on spawn
* Reconstruct loader in `reconstruct_world_from_snapshot` updated to deserialize `Genome` and `LineageMetadata`, dynamically re-deriving agent `Phenotype` components
* Genetics validation rules added to `validate_world_on_startup` and `validate_world_on_tick`
* Startup validation corrected to support running at tick $>0$ for loaded snapshots

---

## Known Risks & Design Constraints

* **Phenotype Influences & Selection Penalties Pending:**
  - The `Phenotype` cache component is correctly calculated and reconstructed on load, but it does **not** yet influence agent movement limits, metabolic decay, or natural selection survival.
  - Agents currently still use default/global configurations for these metrics. This is **intentional**; trait-driven adaptation and metabolic costs are scheduled for Milestone 20.
* **Float Target Optimization Divergence:** Vector mathematics in the environment are calculated using standard `f32` floats. Compile-time optimizations (like contract/FMA) might introduce minor divergence on target CPUs other than the test host.

---

## Open Problems & Technical Debt

* **Known Technical Debt (Metabolism/Sensing/Movement/Consumption O(N) Lookup):** ClimateChunk and TerrainChunk lookups in agent systems currently perform linear chunk scans (O(agent_count × chunk_count)). This is intentionally accepted for Phase 2/3 correctness-first implementation and should be replaced with indexed chunk lookup (HashMap or equivalent deterministic spatial index) during a future performance optimization pass.
* **Manual Array Indexing:** Systems calculate 1D cell offsets manually from 2D coordinates: `local_y * chunk_size + local_x`. This requires introducing an abstract `ChunkField<T>` struct to avoid index out-of-bounds risks in future phases.
* **Synchronous Serialization Blocking:** Snapshot loading and saving operations are run synchronously on the Bevy main loop, which will block frames during large world persistence in a client interface context.

---

## Current Development Target

Begin Milestone 18 — Reproduction, Inheritance & Lineage.

## Recommended Next Actions

*   Confirm that standard tests and clippy compile cleanly: `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings`.
*   Formulate the implementation strategy for Milestone 18.
*   Tag the repository with release marker `v0.3.0-phase3-m17` or similar.

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

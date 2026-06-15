# AI Handoff Briefing

This document serves as the immediate handoff instructions for any AI model resuming development on Project Genesis.

---

## Current Repository State

* **Branch:** `main`
* **Status:** Clean. All Phase 3 Evolution Milestone 16 modules compile under standard profiles, and formatting (`cargo fmt`) and Clippy checks pass successfully.
* **Test Outcome:** `127 passed, 0 failed, 1 ignored` (standard); `1 passed, 0 failed` (`--ignored` stability test).

---

## Current Phase & Milestone

* **Current Phase:** Phase 3 — Evolution (Active)
* **Current Milestone:** Milestone 17 — Resource Consumption (Eating & Drinking) (Active)
* **Last Completed Milestone:** Milestone 16 — Genetics & Phenotype Mapping
* **Phase Progress:** Milestone 16 completed, verified, and locked. Milestone 17 (Resource Consumption) is active.

---

## Recent Major Changes

* **Milestone 16 Completed:** Implemented genetics and phenotype mapping.
  - Implemented `Genome`, `Phenotype`, and `LineageMetadata` components and `GenomeConfig` resource.
  - Implemented dynamic phenotype caching derived on spawn (`derive_phenotype_on_spawn` system).
  - Upgraded snapshot version to schema version `3` to serialize `Genome` and `LineageMetadata`, dynamically re-deriving `Phenotype` on load.
  - Implemented genetics validation (bounds checking and lineage validity).
  - Updated tests and validation systems to allow running on restored worlds at tick $>0$ without initial spawner constraints failing.
  - Added genome mapping mapping, serialization round-trip, and phenotype reconstruction unit tests.
* **Milestone 15 Completed:** Implemented agent persistence and integration testing.
  - Snapshot schema version upgraded to `2` to include the `StableIdGenerator` and `agents` collections.
  - Mapped agent metadata, position, and metabolic stock to snapshot formats.
  - Restored `StableIdGenerator` resource and spawned reconstructed agent entities with default `ActionRequest(None)` to prevent query freezes.
  - Updated `assert_worlds_equivalent` to perform structural comparisons of both the generator resource and live agent collections sorted by stable identifier.
  - Verified A+B=N save/load split ticking equivalence through standard integration tests and the year-long `test_long_run_stability_512` stability test.
* **Milestone 14 Completed:** Implemented spatial movement execution. Agents step cardinally, validating against world boundaries, slopes, and water zones.

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

* **Known Technical Debt (Metabolism/Sensing/Movement O(N) Lookup):** ClimateChunk and TerrainChunk lookups in agent systems currently perform linear chunk scans (O(agent_count × chunk_count)). This is intentionally accepted for Phase 2 correctness-first implementation and should be replaced with indexed chunk lookup (HashMap or equivalent deterministic spatial index) during a future performance optimization pass.
* **Manual Array Indexing:** Systems calculate 1D cell offsets manually from 2D coordinates: `local_y * chunk_size + local_x`. This requires introducing an abstract `ChunkField<T>` struct to avoid index out-of-bounds risks in future phases.
* **Synchronous Serialization Blocking:** Snapshot loading and saving operations are run synchronously on the Bevy main loop, which will block frames during large world persistence in a client interface context.

---

## Current Development Target

Begin Milestone 17 — Resource Consumption (Eating & Drinking).

## Recommended Next Actions

*   Confirm that standard tests and clippy compile cleanly: `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings`.
*   Formulate the implementation strategy for Milestone 17.
*   Tag the repository with release marker `v0.3.0-phase3-m16` or similar.

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

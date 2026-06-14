# AI Handoff Briefing

This document serves as the immediate handoff instructions for any AI model resuming development on Project Genesis.

---

## Current Repository State

* **Branch:** `main`
* **Status:** Clean. Phase 2 Milestone 11 modules compile under standard profiles, and formatting (`cargo fmt`) and Clippy checks pass successfully.
* **Test Outcome:** `108 passed, 0 failed, 1 ignored` (standard); `1 passed, 0 failed` (`--ignored` stability test).

---

## Current Phase & Milestone

* **Current Phase:** Phase 2 — Life
* **Current Milestone:** Milestone 12 — Environmental Sensing Query API (Not Started)
* **Phase Progress:** Milestone 11 completed and verified.

---

## Recent Major Changes

* **Milestone 11 Completed:** Implemented agent data components (`AgentMetadata`, `AgentPosition`, `MetabolicStock`, `ActionRequest`), deterministic agent seed derivation, stable sequence ID generation (`StableIdGenerator` resource), and spawner system in Bevy startup generation schedule. Added startup validation rules.
* **Link Portability Standardized:** Replaced all local `file:///` system references in the documentation with portable GitHub URLs.
* **Governance Refinement:** Updated documentation authority governance to establish repository documentation as the primary source of truth, requiring clarification on conflicts.
* **Lightweight State Registry:** Added [CURRENT_STATE.md](https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md) for high-velocity project tracking.

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

Begin Milestone 12 of Phase 2 (Life) to implement environmental sensing query API mapping cells to climate/resources.

> [!IMPORTANT]
> **No Architectural Redesign:** Under no circumstances should you redesign or replace the approved Phase 2 architecture, components, data responsibilities, or scheduling sequences locked in [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md) and [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md) without explicit human request.

## Current Open Tasks

1.  **Coordinate Mapping & Sensing Queries (Milestone 12):** Build read-only query utilities mapping coordinates to nutrient and climate arrays inside chunk entities.

## Blockers

None.

## Human Decisions Pending

None for the active milestone.

## Safe Next Actions

*   Propose a command to run tests to ensure the base compile is clean: `cargo test`.
*   Define spatial queries linking agent locations to chunk-mapped coordinate offsets.

---

## Important Files

*   **AI Briefing Entrypoint:** [GENESIS_CONTEXT.md](https://github.com/hawary-id/genesis/blob/main/docs/GENESIS_CONTEXT.md)
*   **Current State:** [CURRENT_STATE.md](https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md)
*   **Architecture Decisions Registry:** [ARCHITECTURE_DECISIONS.md](https://github.com/hawary-id/genesis/blob/main/docs/ARCHITECTURE_DECISIONS.md)
*   **Milestone Progress Registry:** [MILESTONE_STATUS.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE_STATUS.md)
*   **Repository Structure Map:** [PROJECT_STRUCTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/PROJECT_STRUCTURE.md)
*   **Integration Tests:** [testing/determinism.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/determinism.rs)

---

## Standard Prompt For Future AI Sessions

> "Read [GENESIS_CONTEXT.md](https://github.com/hawary-id/genesis/blob/main/docs/GENESIS_CONTEXT.md) and [CURRENT_STATE.md](https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md) first and use them as the authoritative source of truth. Follow [ARCHITECTURE_DECISIONS.md](https://github.com/hawary-id/genesis/blob/main/docs/ARCHITECTURE_DECISIONS.md) before making architectural changes. Use [MILESTONE_STATUS.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE_STATUS.md) to determine project progress. Do not modify source code or introduce new architectures unless explicitly requested."

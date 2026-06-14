# AI Handoff Briefing

This document serves as the immediate handoff instructions for any AI model resuming development on Project Genesis.

---

## Current Repository State

* **Branch:** `main`
* **Status:** Clean. Phase 2 Milestone 12 modules compile under standard profiles, and formatting (`cargo fmt`) and Clippy checks pass successfully.
* **Test Outcome:** `111 passed, 0 failed, 1 ignored` (standard); `1 passed, 0 failed` (`--ignored` stability test).

---

## Current Phase & Milestone

* **Current Phase:** Phase 2 — Life
* **Current Milestone:** Milestone 13 — Metabolic Decay & Lifetimes (Not Started)
* **Phase Progress:** Milestone 12 completed and verified.

---

## Recent Major Changes

* **Milestone 12 Completed:** Implemented environmental sensing query API (`agent/sensing.rs`) providing `query_cell` and `query_neighborhood` lookup functions, supported by a `sensing_radius` config field and reusable `WorldBounds::contains_world_coord` validation helpers.
* **Milestone 11 Completed:** Implemented agent data components (`AgentMetadata`, `AgentPosition`, `MetabolicStock`, `ActionRequest`), deterministic agent seed derivation, stable sequence ID generation (`StableIdGenerator` resource), and spawner system in Bevy startup generation schedule. Added startup validation rules.
* **Link Portability Standardized:** Replaced all local `file:///` system references in the documentation with portable GitHub URLs.
* **Governance Refinement:** Updated documentation authority governance to establish repository documentation as the primary source of truth, requiring clarification on conflicts.
* **Lightweight State Registry:** Added [CURRENT_STATE.md](https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md) for high-velocity project tracking.

---

## Known Risks

* **Float Target Optimization Divergence:** Vector mathematics in the environment are calculated using standard `f32` floats. Host-specific compile optimizations (like FMA instruction contraction) might cause minor floating-point divergence on target CPUs other than the primary test platform.

---

## Open Problems & Technical Debt

* **Linear Chunk Lookups**: The environmental query API `query_cell` performs $O(N)$ linear scans over chunks to locate coordinates in Bevy ECS. This is acceptable for current sizes but should be optimized to $O(1)$ using a spatial entity lookup map resource in Milestone 13/14 when systems need on-demand cell access.
* **Determinism Testing for ECS Entity Order**: While we verified chunk spawn order independence and row-major sorting stability, more comprehensive integration-level determinism tests should be extended in Milestone 15 to cover larger world sizes and complex entity interactions.
* **Manual Array Indexing:** Systems calculate 1D cell offsets manually from 2D coordinates: `local_y * chunk_size + local_x`. This requires introducing an abstract `ChunkField<T>` struct to avoid index out-of-bounds risks in future phases.
* **Synchronous Serialization Blocking:** Snapshot loading and saving operations are run synchronously on the Bevy main loop, which will block frames during large world persistence in a client interface context.
* **Imperative Bootstrapping:** App initialization uses procedural helper functions (`register_initial_resources`) rather than standard Bevy plugin traits (`Plugin`), violating standard Bevy ecosystem idioms.

---

## Current Development Target

Begin Milestone 13 of Phase 2 (Life) to implement metabolic decay systems, agent aging, energy decay per tick, and starvation/age limits validation.

> [!IMPORTANT]
> **No Architectural Redesign:** Under no circumstances should you redesign or replace the approved Phase 2 architecture, components, data responsibilities, or scheduling sequences locked in [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md) and [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md) without explicit human request.

## Current Open Tasks

1.  **Metabolic Decay & Agent Aging (Milestone 13):** Implement Bevy update systems that increment agent age and apply base and environmental metabolic energy penalties per simulation tick, triggering starvation or age-limit deaths.

## Blockers

None.

## Human Decisions Pending

None for the active milestone.

## Safe Next Actions

*   Propose a command to run tests to ensure the base compile is clean: `cargo test`.
*   Implement Bevy systems that query agent `MetabolicStock` and update age and energy.

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

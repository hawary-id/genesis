# AI Handoff Briefing

This document serves as the immediate handoff instructions for any AI model resuming development on Project Genesis.

---

## Current Repository State

* **Branch:** `main`
* **Status:** Clean. Phase 2 Milestone 14 modules compile under standard profiles, and formatting (`cargo fmt`) and Clippy checks pass successfully.
* **Test Outcome:** `124 passed, 0 failed, 1 ignored` (standard); `1 passed, 0 failed` (`--ignored` stability test).

---

## Current Phase & Milestone

* **Current Phase:** Phase 2 — Life
* **Current Milestone:** Milestone 15 — Agent Persistence & Integration Testing (Not Started)
* **Phase Progress:** Milestone 14 completed, verified, and locked under tag `phase2-milestone-14`.

---

## Recent Major Changes

* **Milestone 14 Completed:** Implemented spatial movement execution. Agents translate movement action requests into target coordinates, validating against boundaries and terrain zones (slopes and water depths). Movement costs are deducted from metabolic energy, and action request intents are cleared on all paths.
* **Milestone 13 Completed:** Implemented agent metabolism decay (`update_agent_metabolism`) using the absolute-difference formula penalty, chronological aging (`saturating_add(1)`), and survival/death processing (`process_agent_deaths`) at the end of simulation ticks. Sequenced systems correctly inside the `FixedSimulationTick` schedule.
* **Milestone 12 Completed:** Implemented environmental sensing query API (`agent/sensing.rs`) providing `query_cell` and `query_neighborhood` lookup functions, supported by a `sensing_radius` config field and reusable `WorldBounds::contains_world_coord` validation helpers.
* **Milestone 11 Completed:** Implemented agent data components (`AgentMetadata`, `AgentPosition`, `MetabolicStock`, `ActionRequest`), deterministic agent seed derivation, stable sequence ID generation (`StableIdGenerator` resource), and spawner system in Bevy startup generation schedule. Added startup validation rules.

---

## Completed Milestone 14 Summary

### Completed Work
* `process_agent_movement` implemented
* `ActionIntent` execution implemented
* Terrain movement validation implemented (slope limits and water depth barriers)
* Movement energy cost implemented (energy clamped to `0.0` on deduction)
* `FixedSimulationTick` integration completed
* Movement test coverage added (5 new unit tests)

### Important Notes
* Movement currently uses **linear chunk lookup** matching the same lookup strategy as sensing.
* This is accepted technical debt. Do not optimize during Phase 2 unless specifically required.

### Default Parameters
```text
agent_movement_max_slope = 0.40
agent_movement_max_water_depth = 0.30
agent_movement_cost = 1.0
```

### Schedule Ordering
```text
Energy
→ Movement
→ Metabolism
→ Death
```

---

## Known Risks

* **Float Target Optimization Divergence:** Vector mathematics in the environment are calculated using standard `f32` floats. Host-specific compile optimizations (like FMA instruction contraction) might cause minor floating-point divergence on target CPUs other than the primary test platform.

---

## Open Problems & Technical Debt

* **Known Technical Debt (Metabolism/Sensing/Movement O(N) Lookup):** ClimateChunk and TerrainChunk lookups in agent systems currently perform linear chunk scans (O(agent_count × chunk_count)). This is intentionally accepted for Phase 2 correctness-first implementation and should be replaced with indexed chunk lookup (HashMap or equivalent deterministic spatial index) during a future performance optimization pass.
* **Determinism Testing for ECS Entity Order:** While we verified chunk spawn order independence and row-major sorting stability, more comprehensive integration-level determinism tests should be extended in Milestone 15 to cover larger world sizes and complex entity interactions.
* **Manual Array Indexing:** Systems calculate 1D cell offsets manually from 2D coordinates: `local_y * chunk_size + local_x`. This requires introducing an abstract `ChunkField<T>` struct to avoid index out-of-bounds risks in future phases.
* **Synchronous Serialization Blocking:** Snapshot loading and saving operations are run synchronously on the Bevy main loop, which will block frames during large world persistence in a client interface context.
* **Imperative Bootstrapping:** App initialization uses procedural helper functions (`register_initial_resources`) rather than Bevy plugin traits (`Plugin`), violating Bevy ecosystem idioms.

---

## Current Development Target

Begin Milestone 15 of Phase 2 (Life) to implement agent persistence and integration testing (extending snapshot serialization to save sorted agent states, and verifying execution determinism under save/load equivalence integration tests).

> [!IMPORTANT]
> **No Architectural Redesign:** Under no circumstances should you redesign or replace the approved Phase 2 architecture, components, data responsibilities, or scheduling sequences locked in [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md) and [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md) without explicit human request.

## Current Open Tasks

1.  **Agent Persistence & Integration Testing (Milestone 15):** Extend snapshot serialization to save sorted agent states, and verify execution determinism under save/load equivalence integration tests.

## Blockers

None.

## Human Decisions Pending

None for the active milestone.

## Safe Next Actions

*   Propose a command to run tests to ensure the base compile is clean: `cargo test`.
*   Implement state serialization changes for agent components within persistence modules.

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

# Current State

* **Current Phase:** Phase 3 — Evolution (Transitioning from Phase 2)
* **Current Milestone:** Phase 2 Complete (v0.2.0-phase2 release candidate)
* **Current Branch:** main
* **Current Status:** Phase 2 (Life) implementation is fully completed, clippy-compliant, and verified under integration/stability tests.
* **Current Focus:** Transitioning to Phase 3 — Evolution (genetics, mutation, inheritance, and natural selection).
* **Next Task:** Design and plan Phase 3 (Evolution) implementation roadmap.
* **Last Verified Test Counts:**
  - `cargo test`: 124 passed, 0 failed, 1 ignored
  - `cargo test -- --ignored`: 1 passed (test_long_run_stability_512 checks A+B=N save/load equivalence over 8,640 ticks / 1 simulation year)
  - `cargo clippy -- -D warnings`: PASS
* **Last Updated:** 2026-06-15T00:45:00+07:00

## Completed in Milestone 15: Persistence & Integration Testing

* **Snapshot Schema Upgrade**: Upgraded snapshot version to schema version `2` in [`snapshot.rs`](file:///c:/Genesis/engine/src/persistence/snapshot.rs) to include the `StableIdGenerator` resource and a collection of `AgentSnapshot` structures.
* **Agent & ID Generator Persistence**: Implemented saving/loading of agent metadata, positions, and metabolic stocks, along with the sequential `StableIdGenerator` counter state in [`io.rs`](file:///c:/Genesis/engine/src/persistence/io.rs).
* **Deterministic Sorting**: Agent snapshots are automatically sorted by their stable sequence ID ascending before serialization, satisfying ADR-002 and ensuring snapshot formatting determinism.
* **Agent Reconstruction**: The load path in `reconstruct_world_from_snapshot` spawns agent entities with a default `ActionRequest(None)` component so they integrate correctly into Bevy systems on subsequent ticks.
* **A+B=N Equivalence Testing**: Verified using `assert_worlds_equivalent` that splitting simulation runs across a save/load boundary yields identical float/state configurations compared to a continuous run.

## Completed in Milestone 14: Agent Movement & Kinematics

* **Agent Movement Execution**: Cardinal grid-cell steps are executed based on agent `ActionRequest` (`ActionIntent::MoveNorth/South/East/West`).
* **Boundary & Terrain Validation**: Movement requests are validated against world boundaries, elevation slope limits (0.40), and water depth limits (0.30).
* **Movement Energy Costs**: Energy step costs are applied upon successful movement, and action request intents are cleared on all paths.

## Known Technical Debt

> [!NOTE]
> **ClimateChunk and TerrainChunk Lookup Scan Complexity**
> ClimateChunk and TerrainChunk lookups in agent sensing, metabolism, and movement currently perform linear chunk scans (O(agent_count × chunk_count)).
> This is intentionally accepted for Phase 2 correctness-first implementation and should be replaced with indexed chunk lookup (HashMap or equivalent deterministic spatial index) during a future performance optimization pass.

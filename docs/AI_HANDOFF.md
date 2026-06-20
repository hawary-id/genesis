# AI Handoff Briefing

This document serves as the immediate tactical handoff instructions for any AI model resuming development on Project Genesis.

## Current Status

**Milestone 23 — Event Memory: COMPLETE**

Phase 4 (Memory) is active. M23 is complete. Genesis now supports chronological event memory recording alongside location memory.

## What Exists

* **Location Memory**: Agents track locations (`Nutrient`, `FreshWater`, `Hazard`) with deterministic LRU eviction.
* **Event Memory**: Agents track distinct life events (`ResourceConsumed`, `FailedMovement`, `Reproduced`, `HazardEncountered`) with chronological ordering.
* **Persistence Integration**: Seamless save/load functionality using snapshot schema v4, maintaining backwards compatibility with v3 via `#[serde(default)]`.
* **Validation Integration**: Startup and post-tick bounds checking enforces monotonic sequences and capacity constraints.
* **Determinism Guarantees**: Strict sequence-in-tick counters prevent race conditions during sub-tick event generation, ensuring A+B=N save/load equivalence over long simulation runs.

## Important Constraints

* **ADR-001**: Strict adherence to ECS boundaries. No god objects or manager classes.
* **ADR-002**: Strict adherence to determinism contracts. All systems must produce identical outcomes across platforms and runs given the same seed.
* **ECS Boundaries**: Prefer systems over objects. Keep systems deterministic. Avoid global mutable state.
* **Save/Load Equivalence**: Any execution run interrupted by a save and load must produce exactly the same results as an uninterrupted run.

## Next Milestone

* **Milestone 24 — Social Memory**: According to the roadmap for Phase 4, the final remaining feature for the Memory phase is social memory. 

## Next Actions for AI Model

1. **M23 is officially closed**: No M23 implementation work remains. The next architectural focus is M24 planning.
2. Read `MILESTONE_STATUS.md` to determine the next planned milestone.
3. Read `docs/ROADMAP.md` for Phase 4 context.
4. Perform documentation audit, codebase audit, and gap analysis for Milestone 24.
5. Produce an implementation plan and await user approval before writing any code.

## Known Blockers & Technical Debt

* **Metabolism/Sensing O(N) Lookup:** Systems currently perform linear chunk scans (O(agent_count × chunk_count)). Needs optimization via O(1) Spatial Map lookup.
* **Manual Array Indexing:** Systems calculate 1D cell offsets manually from 2D coordinates.
* **Synchronous Serialization Blocking:** Snapshot saving blocks frames synchronously.

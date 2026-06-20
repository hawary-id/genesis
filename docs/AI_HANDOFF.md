# AI Handoff Briefing

This document serves as the immediate tactical handoff instructions for any AI model resuming development on Project Genesis.

## Current Status

**Milestone 24 — Social Memory: COMPLETE**
**Phase 4 (Memory): COMPLETE**

Phase 4 (Memory) is now fully complete. Genesis supports location memory, chronological event memory, and subjective social (kinship) memory.

## What Exists

* **Location Memory**: Agents track locations (`Nutrient`, `FreshWater`, `Hazard`) with deterministic LRU eviction.
* **Event Memory**: Agents track distinct life events (`ResourceConsumed`, `FailedMovement`, `Reproduced`, `HazardEncountered`) with chronological ordering.
* **Social Memory**: Agents track basic kinship relationships (`Parent`, `Child`) deterministically generated during reproduction.
* **Persistence Integration**: Seamless save/load functionality using snapshot schema v5, maintaining backwards compatibility with older snapshots via `#[serde(default)]`.
* **Validation Integration**: Startup and post-tick bounds checking enforces monotonic sequences, capacity constraints, duplicate target prevention, and self-reference prevention.
* **Determinism Guarantees**: Strict sequence-in-tick counters prevent race conditions during sub-tick event generation, ensuring A+B=N save/load equivalence over long simulation runs.

## Important Constraints

* **ADR-001**: Strict adherence to ECS boundaries. No god objects or manager classes.
* **ADR-002**: Strict adherence to determinism contracts. All systems must produce identical outcomes across platforms and runs given the same seed.
* **ECS Boundaries**: Prefer systems over objects. Keep systems deterministic. Avoid global mutable state.
* **Save/Load Equivalence**: Any execution run interrupted by a save and load must produce exactly the same results as an uninterrupted run.

## Next Milestone

* **Phase 5 Planning**: Phase 4 is complete. The next focus is starting Phase 5 (Agency).
* **Milestone 25 — Goal Formation Foundation**: The expected first milestone of Phase 5. With memory in place, agents now have the architectural prerequisites (knowledge of self, environment, history, and relations) to begin forming subjective goals.

## Next Actions for AI Model

1. **Phase 4 is officially closed**: All Phase 4 memory implementation work is finished.
2. Read `MILESTONE_STATUS.md` to confirm current project state.
3. Read `docs/ROADMAP.md` for Phase 5 (Agency) context.
4. Do NOT invent M25 implementation details. Focus strictly on architectural planning based on the newly available memory prerequisites.
5. Produce an implementation plan and await user approval before writing any code.

## Known Blockers & Technical Debt

* **Metabolism/Sensing O(N) Lookup:** Systems currently perform linear chunk scans (O(agent_count × chunk_count)). Needs optimization via O(1) Spatial Map lookup.
* **Manual Array Indexing:** Systems calculate 1D cell offsets manually from 2D coordinates.
* **Synchronous Serialization Blocking:** Snapshot saving blocks frames synchronously.

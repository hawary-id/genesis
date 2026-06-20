# AI Handoff Briefing

This document serves as the immediate tactical handoff instructions for any AI model resuming development on Project Genesis.

## Current Status

**Milestone 22 — Location Memory Foundation: COMPLETE**

All Phase 3 (Evolution) milestones are complete. Phase 4 (Memory) has officially started and M22 is complete. Genesis now supports subjective location memory.

## What Was Completed in M22

### Location Memory Architecture
* Agents now track locations via `LocationMemory` ECS components holding fixed-capacity `LocationMemoryNode` entries.
* Senses and interactions trigger `ObservationEvent` emissions (`Nutrient`, `FreshWater`, `Hazard`) inside `agent/systems.rs`.
* Observations are collected by `process_memory_consolidation` running in `FixedSimulationTick` after standard agent action execution.

### Deterministic LRU Eviction
* Memory uses deterministic chronological eviction sorting. Observations tie-break LRU chronologically via `coord.y` and `coord.x` to preserve save/load and branching identical equivalence.
* Validation checks bounds and capacity inside `validation/systems.rs`.

### Persistence
* Snapshot schema updated to persist `LocationMemory` under the `AgentSnapshot` model seamlessly. Includes `#[serde(default)]` compatibility for loading previous v3 architecture saves seamlessly without panicking.

## Key Architectural Decisions Made in M22

1. **Deterministic Eviction**: Ensuring spatial coordinates dictate deterministic tie-breaking. 
2. **Backward Compatibility**: Utilizing `serde(default)` protects v3 save branches from panics upon loading newer memory traits.
3. **Decoupled Perception**: `ObservationEvent` allows sensors to trigger abstract alerts asynchronously without knowing how they are stored or processed.

## Verified Test Status (M22 Completion)

* `cargo fmt`: PASS
* `cargo clippy -- -D warnings`: PASS
* `cargo test`: PASS
* `cargo test -- --ignored`: PASS (test_long_run_stability_512, 8,640 ticks / 1 year)
* Long-run determinism test: PASS
* Snapshot validation tests: PASS

## Next Actions for AI Model

1. **M22 is officially closed**: No M22 implementation work remains. Any remaining observations (like isolated unit tests) are optional improvements only. The next architectural focus is M23 planning.
2. Read `MILESTONE_STATUS.md` to determine the next planned milestone.
3. Read `docs/ROADMAP.md` for Phase 4 context.
4. Perform documentation audit, codebase audit, and gap analysis for Milestone 23.
5. Produce an implementation plan and await user approval before writing any code.

## Known Blockers & Technical Debt

* **Metabolism/Sensing O(N) Lookup:** Systems currently perform linear chunk scans (O(agent_count × chunk_count)). Needs optimization via O(1) Spatial Map lookup.
* **Manual Array Indexing:** Systems calculate 1D cell offsets manually from 2D coordinates.
* **Synchronous Serialization Blocking:** Snapshot saving blocks frames synchronously.

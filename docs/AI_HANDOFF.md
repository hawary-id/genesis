# AI Handoff Briefing

This document serves as the immediate tactical handoff instructions for any AI model resuming development on Project Genesis.

## Current Status

**Milestone 21 — Evolution Diagnostics and Validation: COMPLETE**

All Phase 3 (Evolution) milestones through M21 are complete. Phase 3 is complete.

## What Was Completed in M21

### PopulationStatistics Resource
* Added `PopulationStatistics` as a **runtime-only telemetry resource** in `engine/src/agent/diagnostics.rs`.
* Populated each tick by `compute_population_statistics`, registered in the `ObservationBoundary` schedule.
* **Not persisted in snapshots.** Fully reconstructable from live ECS world state. This is an intentional architectural constraint — telemetry must not influence determinism.
* Fields: `total_population`, `mean_thermal_optimum`, `mean_diet_preference`, `mean_max_slope`, `mean_max_water_depth`, `mean_sensing_radius`, `mean_physical_size`, `standard_deviation_thermal_optimum`.

### Lineage Validation Invariants
Bidirectional lineage invariants are enforced in both `validate_world_on_startup` and `validate_world_on_tick`:
* `generation == 0 ⟹ parent_id == None` (founder agents have no parent)
* `generation > 0 ⟹ parent_id == Some(...)` (descended agents always have a parent)
* Both invalid states are covered by dedicated `#[should_panic]` tests.

### Genome Reconstruction Padding
* On snapshot restore, genomes shorter than `GENOME_SIZE` are padded with `0.5` (neutral mid-range) to ensure all runtime genome operations remain safe.
* Loading never truncates valid genomes; it only pads.
* Two tests verify: `test_reconstruct_pads_undersized_genomes` and `test_reconstruct_preserves_genome_length`.

### GENOME_SIZE Constant
* Extracted `pub const GENOME_SIZE: usize = 8` into `agent/components.rs` and re-exported via `agent/mod.rs`.
* All test and production code now uses `crate::agent::GENOME_SIZE` instead of hardcoded literals.

### Snapshot Validation Workflow
* Snapshot load → genome padding → phenotype re-derivation → startup validation (invariants checked).
* Validation enforces lineage and genome invariants on every startup and post-tick boundary.

## Key Architectural Decisions Made in M21

1. **Runtime-only telemetry:** `PopulationStatistics` is never serialized. It violates no determinism contract because it is purely derived from ECS state.
2. **Genome padding is silent and forward-compatible:** Padding uses `0.5` (neutral), preserving migration from older snapshots without data loss.
3. **Lineage invariants are bidirectional:** Both directions (generation→parent and parent→generation) are enforced to prevent partial or corrupt lineage states.

## Verified Test Status (M21 Completion)

* `cargo fmt`: PASS
* `cargo clippy -- -D warnings`: PASS
* `cargo test`: 131 passed, 0 failed, 1 ignored
* `cargo test -- --ignored`: PASS (test_long_run_stability_512, 8,640 ticks / 1 year)
* Long-run determinism test: PASS
* Snapshot validation tests: PASS
* Lineage validation tests: PASS (both bidirectional invalid states)
* Population diagnostics tests: PASS

## Next Actions for AI Model

1. Read `MILESTONE_STATUS.md` to determine the next planned milestone after M21.
2. Read `docs/ROADMAP.md` for Phase 4 context.
3. Perform documentation audit, codebase audit, and gap analysis for the next milestone.
4. Produce an implementation plan and await user approval before writing any code.

## Known Blockers & Technical Debt

* **Metabolism/Sensing O(N) Lookup:** Systems currently perform linear chunk scans (O(agent_count × chunk_count)). Needs optimization via O(1) Spatial Map lookup.
* **Manual Array Indexing:** Systems calculate 1D cell offsets manually from 2D coordinates.
* **Synchronous Serialization Blocking:** Snapshot saving blocks frames synchronously.

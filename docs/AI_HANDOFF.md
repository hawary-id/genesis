# AI Handoff Briefing

This document serves as the immediate tactical handoff instructions for any AI model resuming development on Project Genesis.

## Current Objective
* Prepare for Milestone 21.

## Active Milestone

M21 — Evolution Diagnostics and Validation

(Source of Truth: CURRENT_STATE.md)

## Next Actions
1. Verify system stability: run `cargo test` and `cargo clippy --all-targets -- -D warnings`.
2. Await user instructions for starting the next milestone (M21) or any pending architecture refactors.

## Known Blockers & Technical Debt
* **Metabolism/Sensing O(N) Lookup:** Systems currently perform linear chunk scans (O(agent_count × chunk_count)). Needs optimization via O(1) Spatial Map lookup.
* **Manual Array Indexing:** Systems calculate 1D cell offsets manually from 2D coordinates.
* **Synchronous Serialization Blocking:** Snapshot saving blocks frames synchronously.

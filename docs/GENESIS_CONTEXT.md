# Genesis AI Context Layer

This document is the primary briefing source of truth for AI agents interacting with the Genesis repository. It summarizes project vision, milestones, core architecture, navigation guides, and permanent design choices to facilitate immediate context loading.

---

## Vision

Project Genesis builds an Artificial Civilization Engine designed to study emergent societal, cultural, and technological patterns.
The core philosophy is:
> Do not build civilization. Build the conditions under which civilization becomes inevitable.

The engine is engineered in Rust using data-oriented ECS (Entity Component System) principles to enable high-throughput simulation of environment, life, and emergence.

---

## Current State

* **Current Phase:** Phase 3 — Evolution (Active)
* **Current Milestone:** Milestone 17 — Resource Consumption (Eating & Drinking)
* **Last Completed Milestone:** Milestone 16 — Genetics & Phenotype Mapping
* **Repository Status:** Clean
* **Build Status:** PASS (Compares cleanly with zero warnings under Clippy)
* **Test Status:** PASS
  - Standard Test Suite: `127 passed; 0 failed; 1 ignored`
  - Ignored Test Suite (`cargo test -- --ignored`): `1 passed` (stability check of 8,640 ticks / 1 simulation year completes successfully in 216.53s)

---

## Project Summary

Genesis simulates a 2D grid divided into coordinate-mapped chunk entities. Phase 1 provides the environmental substrate representing:
* **Terrain:** Soil depth, fertility, slope gradients, and elevations.
* **Climate:** Temperature, sunlight exposure, moisture, and rainfall cycles.
* **Resources:** Mineral nodes, nutrients, and fresh water.
* **Energy:** Solar exposure and biomass potentials.

All physical phenomena advance on daily tick boundaries, synchronized by a simulation clock and seasonal modifier progressions. 

---

## Core Architecture

Genesis uses Bevy ECS to decouple data structures from simulation logic:
* **Dense Components:** Environmental layers are stored as flat 1D vectors (`Vec<f32>` arrays of length `chunk_size * chunk_size`) within chunk components (`TerrainChunk`, etc.), avoiding memory fragmentation.
* **Pure Systems:** Climate, resource, and energy systems read resources and write modifications to components inside the Bevy `World`.
* **Sequential schedules:** Tick progressions are partitioned into five sequentially chained schedules: `StartupGeneration` $\rightarrow$ `FixedSimulationTick` $\rightarrow$ `PostTickValidation` $\rightarrow$ `PersistenceBoundary` $\rightarrow$ `ObservationBoundary`.

---

## Permanent Architectural Decisions

The following architectural rules are locked under ADR guidelines:
1. **Single-threaded System Sequencing:** Systems must be explicitly sequenced using Bevy `.after()` dependencies to avoid race conditions.
2. **Coordinate-Salted Randomness:** All procedural operations must derive seeds locally from chunk coordinates and the root seed via `rand_chacha::ChaCha8Rng`. Entropy generation must remain invariant to spawn order.
3. **Stable Snapshot Alignment:** Prior to JSON serialization, chunk snapshots must be sorted by `(coord.y, coord.x)` ascending to guarantee binary-identical file outputs.
4. **Binary Float Parity:** Verification tests comparing float fields must check exact bitwise parity (`assert_eq!`) rather than epsilon thresholds.

---

## Active Development Focus

With Phase 2 (Life) completed and verified, Phase 3 (Evolution) has begun:
* Spawning biological agent entities with stable sequence identifiers (Milestone 11 - Completed).
* Implementing neighborhood sensory queries mapping to Phase 1 environmental chunks (Milestone 12 - Completed).
* Driving agent lifecycle progression via metabolic decay tick updates (Milestone 13 - Completed).
* Constraining spatial transitions under grid-cell movement rules (Milestone 14 - Completed).
* Extending snapshot serialization and verifying execution determinism under save/load equivalence integration tests (Milestone 15 - Completed).
* Implementing genetic representations, phenotype mapping cache, generation tracking, and version 3 snapshots (Milestone 16 - Completed).

---

## Next Recommended Objectives

1. **Milestone 17: Resource Consumption (Eating & Drinking):** Implement environmental resource harvesting, neighborhood sensing of nutrient/fresh water chunks, and agent energy stock replenishment.

---

## Important Documents

* **Roadmap:** https://github.com/hawary-id/genesis/blob/main/docs/ROADMAP.md
* **Architecture Baseline:** https://github.com/hawary-id/genesis/blob/main/docs/ARCHITECTURE_BASELINE.md
* **World Technical Specification:** https://github.com/hawary-id/genesis/blob/main/docs/PHASE1_WORLD_TECH_SPEC.md
* **Phase 1 Completion Report:** https://github.com/hawary-id/genesis/blob/main/docs/PHASE1_COMPLETION_REPORT.md
* **Phase 1 Repository Structure:** https://github.com/hawary-id/genesis/blob/main/docs/PHASE1_REPOSITORY_STRUCTURE.md
* **ADR Directory:** https://github.com/hawary-id/genesis/blob/main/docs/adr/
* **Current State:** https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md
* **Milestone Status:** https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE_STATUS.md
* **AI Handoff Briefing:** https://github.com/hawary-id/genesis/blob/main/docs/AI_HANDOFF.md

---

## AI Reading Workflow

To load repository context quickly and efficiently while minimizing repository scans and token consumption, future AI models should follow this sequence:
1. **Read [GENESIS_CONTEXT.md](https://github.com/hawary-id/genesis/blob/main/docs/GENESIS_CONTEXT.md)**: Load the current phase state, glossary, rules, and repository settings.
2. **Read [CURRENT_STATE.md](https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md)**: Fetch the lightweight, frequently updated operational status of the project.
3. **Read [AI_HANDOFF.md](https://github.com/hawary-id/genesis/blob/main/docs/AI_HANDOFF.md)**: Fetch immediate development targets, blockers, safe next steps, and pending decisions.
4. **Read [ARCHITECTURE_DECISIONS.md](https://github.com/hawary-id/genesis/blob/main/docs/ARCHITECTURE_DECISIONS.md)**: Understand the design rules and stability classifications of all ADRs.
5. **Read [MILESTONE_STATUS.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE_STATUS.md)**: Retrieve the project progress registry.
6. **Inspect target code only if needed**: Use [PROJECT_STRUCTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/PROJECT_STRUCTURE.md) to locate the relevant source code. Avoid full repository recursive scanning.

---

## Repository Navigation Guide

* **Configuration:** Core parameters and bounds reside in [engine/src/config/world_config.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/config/world_config.rs) and [world_bounds.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/config/world_bounds.rs).
* **Clock & Seasons:** Clock advancement and seasonal transitions are in [engine/src/time/simulation_clock.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/time/simulation_clock.rs) and [season_state.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/time/season_state.rs).
* **Environmental Simulation:** Soil updates, climate dynamics, resource cycles, and energy potential updates reside in [engine/src/world/](https://github.com/hawary-id/genesis/blob/main/engine/src/world/).
* **Schedules & Orchestration:** The Bevy App wrapper and system execution configurations are located in [engine/src/app/mod.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/app/mod.rs) and [schedules.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/app/schedules.rs).
* **Snapshot & Persistence:** File snapshot mapping, reading, and writing are handled in [engine/src/persistence/io.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/io.rs) and [snapshot.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/snapshot.rs).
* **Verification Invariants:** Invariant validation assertions reside in [engine/src/validation/systems.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/validation/systems.rs).
* **Integration Testing:** Core determinism tests are compiled in [engine/src/testing/determinism.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/determinism.rs).

---

## Last Updated State

* **Timestamp:** 2026-06-15T09:54:00+07:00
* **Repository State:** Phase 3 Milestone 16 complete, verified, and active development on Milestone 17.

## Repository State

* **Primary Branch:** main
* **Active Development Branch:** main
* **Last Verified Tag:** v0.2.0-phase2

---

## AI Development Rules

1. Preserve deterministic execution.
2. Preserve ECS-first architecture.
3. Do not replace accepted ADR decisions without explicit approval.
4. Documentation takes precedence over conversational memory.
5. Never introduce new architectural layers unless requested.
6. Follow existing repository patterns before creating new ones.
7. Prefer consistency over novelty.

---

## Documentation Governance

To prevent context drift and ensure long-term maintainability:
* **[GENESIS_CONTEXT.md](https://github.com/hawary-id/genesis/blob/main/docs/GENESIS_CONTEXT.md)** is the master entry point and primary source of truth for glossary, rules, and repository state.
* **[CURRENT_STATE.md](https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md)** provides a lightweight, frequently-updated overview of the current phase, milestone, focus, and next task.
* **[AI_HANDOFF.md](https://github.com/hawary-id/genesis/blob/main/docs/AI_HANDOFF.md)** holds temporary operational context, active goals, and immediate safe next steps.
* **[ARCHITECTURE_DECISIONS.md](https://github.com/hawary-id/genesis/blob/main/docs/ARCHITECTURE_DECISIONS.md)** governs all architectural decisions and defines ADR stability levels.
* **[MILESTONE_STATUS.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE_STATUS.md)** records project progress and milestone completion milestones.
* **Repository documentation is the primary source of truth.** If repository documentation conflicts with new user instructions, request clarification before proceeding.

---

## Project Glossary

* **Chunk**: An ECS entity representing a fixed-size square partition of the simulation grid. Chunks contain dense, cache-local environmental layer data stored in flat 1D vectors rather than spawning individual cell entities (ADR-003).
* **ChunkCoord**: A spatial struct representing the 2D coordinate `(x, y)` of a Chunk in the grid of chunks. Used to locate chunk entities and map world coordinates to local cells.
* **StartupGeneration**: A dedicated Bevy schedule sequence run during initialization. It executes systems that procedurally generate coordinates, terrain layers, climate zones, resources, and solar/energy values (ADR-005).
* **FixedSimulationTick**: The main Bevy schedule executing simulation update systems on a fixed, deterministic hourly timestep (ADR-004).
* **PostTickValidation**: Runs immediately after `FixedSimulationTick`, validating world state invariants, coordinate boundaries, array sizes, and clock monotonicity before persistence or observation occurs.
* **PersistenceBoundary**: Evaluates snapshot requests and triggers stable serialization (JSON snapshots) and database writes without mutating simulation state (ADR-002).
* **ObservationBoundary**: Produces read-only summaries, logs, metrics, or telemetry diagnostics for dashboards and APIs at the very end of each tick, ensuring zero side-effects on simulation state.
* **Deterministic Execution**: The contract (ADR-002) ensuring that a simulation progresses identically across independent runs with the same seed and config. Enforced by sequential system ordering via `.after()` and coordinate-salted RNG.
* **World Substrate**: The complete physical and environmental foundation of the simulation world, comprised of Terrain, Climate, Resource, and Energy availability layers.
* **StableIdGenerator**: State-tracking counter resource that assigns unique, stable sequential identifiers to spawned agent entities to prevent ID collisions upon snapshot restoration.
* **AgentSnapshot**: Struct representing the complete serialized state (metadata, coordinates, metabolic stock) of an agent entity inside a version 2 `WorldSnapshot` payload.

---

## Standard AI Startup Prompt

```md
You are resuming development on the Genesis project. To align context and prevent architectural drift, execute the following startup workflow:

1. **Load Context in Order:** Read the documentation in this exact sequence before scanning or modifying the codebase:
   - `docs/GENESIS_CONTEXT.md` (This master context file)
   - `docs/CURRENT_STATE.md` (Lightweight current operational status)
   - `docs/AI_HANDOFF.md` (Immediate status, pending tasks, and blockers)
   - `docs/PROJECT_STRUCTURE.md` (Repository navigation guide)
   - `docs/ARCHITECTURE_DECISIONS.md` (ADR summaries and stability classifications)
   - `docs/MILESTONE_STATUS.md` (Current milestone tracking)
2. **Follow Architectural Truth:** Treat the repository documentation (specifically ADRs under `docs/adr/` and `docs/ARCHITECTURE_BASELINE.md`) as the absolute source of truth over conversational memory or assumptions.
3. **Respect ADR Decisions & Stability:** Check the Stability Level of all referenced ADRs in `docs/ARCHITECTURE_DECISIONS.md`. Under no circumstances should you redesign or replace `LOCKED` or `STABLE` architectural decisions without explicit human instruction.
4. **Preserve Determinism & ECS Conventions:** Ensure all code adheres strictly to ECS principles (separate data from logic) and maintains absolute bitwise determinism (sequential execution order via Bevy `.after()`, coordinate-salted RNG).
5. **Avoid Novel Paradigms:** Do not introduce new libraries, frameworks, patterns, or abstraction layers unless specifically requested. Follow existing implementation patterns found in the repository.
```

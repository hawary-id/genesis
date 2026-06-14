# Genesis Project Structure

This document catalogs the directory layout and modular architecture of Project Genesis during Phase 2 (as of Milestone 13 completion). It details the purpose, responsibilities, key files, and dependencies of each submodule.

---

## Workspace Directory Overview

The root workspace contains the following top-level directories:

### docs/
* **Purpose:** Stores project specifications, design decisions, roadmap, and milestone reviews.
* **Responsibilities:** Documentation of system behaviors, timelines, and constraints.
* **Key Files:**
  - [GENESIS_CONTEXT.md](https://github.com/hawary-id/genesis/blob/main/docs/GENESIS_CONTEXT.md) — Master AI context file.
  - [CURRENT_STATE.md](https://github.com/hawary-id/genesis/blob/main/docs/CURRENT_STATE.md) — Lightweight operational status file.
  - [AI_HANDOFF.md](https://github.com/hawary-id/genesis/blob/main/docs/AI_HANDOFF.md) — Immediate operational briefing and next steps.
  - [ARCHITECTURE_DECISIONS.md](https://github.com/hawary-id/genesis/blob/main/docs/ARCHITECTURE_DECISIONS.md) — ADR index with stability classifications.
  - [MILESTONE_STATUS.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE_STATUS.md) — Milestone tracking registry.
  - [ROADMAP.md](https://github.com/hawary-id/genesis/blob/main/docs/ROADMAP.md) — Long-term project roadmap.
  - [ARCHITECTURE_BASELINE.md](https://github.com/hawary-id/genesis/blob/main/docs/ARCHITECTURE_BASELINE.md) — Architectural layout baseline.
  - [PHASE1_WORLD_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE1_WORLD_TECH_SPEC.md) — Environmental substrate specification.
  - [PHASE1_COMPLETION_REPORT.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE1_COMPLETION_REPORT.md) — Verification and audit metrics report.
  - [PHASE1_REPOSITORY_STRUCTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE1_REPOSITORY_STRUCTURE.md) — Repository structure map.
* **Dependencies:** None.

### engine/
* **Purpose:** The main Rust Cargo workspace folder.
* **Responsibilities:** Holds the crate manifest, dependencies lists, and rust sources.
* **Key Files:**
  - [Cargo.toml](https://github.com/hawary-id/genesis/blob/main/engine/Cargo.toml) — Engine dependencies and build targets.
* **Dependencies:** `bevy_ecs`, `rand`, `rand_chacha`, `serde`, `serde_json`.

---

## Crate Source Modules (engine/src/)

All simulation logic resides under [engine/src/](https://github.com/hawary-id/genesis/blob/main/engine/src/). The crate roots are:
* [lib.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/lib.rs) — Crate declaration and submodule exposure.
* [main.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/main.rs) — Binary entry point initializing the app loop.

The crate is structured into the following submodules:

### agent/
* **Purpose:** Biological agent entities structure, metadata, ID generation, spawning, and environmental sensing queries.
* **Responsibilities:** Defines agent metadata with stable sequence identifiers, spatial positions, metabolic stocks, action requests, spawns the initial agent population deterministically, provides query utilities to sense nutrients and fresh water resources in the local neighborhood chunk cells, updates agent age and metabolic stock energy, and despawns dead or over-aged agents.
* **Key Files:**
  - [components.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/components.rs) — Agent data structures (`AgentMetadata`, `AgentPosition`, `MetabolicStock`, `ActionRequest`).
  - [resources.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/resources.rs) — `StableIdGenerator` identifier generation logic.
  - [sensing.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/sensing.rs) — Environmental sensing query API (`query_cell`, `query_neighborhood`, and `SensedResource`).
  - [systems.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/systems.rs) — Spawning, metabolic decay update, age progression, and death processing systems.
  - [mod.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/mod.rs) — Submodule interface re-exports.
* **Dependencies:** `bevy_ecs`, `config`, `rng`, `world::coord`, `world::resource`.

### app/
* **Purpose:** Application container bootstrap and schedule pipeline.
* **Responsibilities:** Initializes the Bevy `World`, registers the Bevy simulation schedules (StartupGeneration, FixedSimulationTick, PostTickValidation, PersistenceBoundary, ObservationBoundary), binds execution systems, and manages event signals.
* **Key Files:**
  - [mod.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/app/mod.rs) — Struct `App` implementation and bootstrap logic.
  - [schedules.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/app/schedules.rs) — Schedule labels (`StartupGeneration`, `FixedSimulationTick`, etc.).
  - [plugins.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/app/plugins.rs) — Global resource registration helpers.
* **Dependencies:** `bevy_ecs`, `config`, `rng`, `time`, `persistence`.

### config/
* **Purpose:** Global parameter storage and boundary mapping.
* **Responsibilities:** Defines the authoritative simulation configuration structures.
* **Key Files:**
  - [world_config.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/config/world_config.rs) — Config parameters (amplitudes, lapse rates, size settings).
  - [world_bounds.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/config/world_bounds.rs) — Derived grid coordinates bounds logic.
* **Dependencies:** `bevy_ecs`.

### rng/
* **Purpose:** Coordinate-salted deterministic random number seed derivation.
* **Responsibilities:** Exposes pure functions to derive seeds based on chunk coordinate inputs.
* **Key Files:**
  - [seed.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/rng/seed.rs) — CSPRNG seed derivation algorithms using `ChaCha8Rng`.
* **Dependencies:** `rand`, `rand_chacha`.

### time/
* **Purpose:** Tracks simulation tick progression and year/season modifications.
* **Responsibilities:** Increments simulation clock ticks and translates them to days, seasons, and years.
* **Key Files:**
  - [simulation_clock.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/time/simulation_clock.rs) — Chronological ticks tracking.
  - [season_state.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/time/season_state.rs) — Dynamic temperature and climate seasonal modifiers calculation.
* **Dependencies:** `bevy_ecs`, `config`.

### world/
* **Purpose:** Spatial substrate chunks generation and dynamic simulation updates.
* **Responsibilities:** Simulates physical layers (terrain elevation, slope, soil, temperature, moisture, rainfall, resources, solar exposure, energy availability) inside flat cell vectors.
* **Key Files:**
  - [generation.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/generation.rs) — Sequenced startup procedural map generation.
  - [terrain.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/terrain.rs) — Terrain slope and soil calculations.
  - [climate.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/climate.rs) — Sunlight factor and weather recalculations.
  - [resource.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/resource.rs) — Fresh water, mineral node, and nutrient depletion.
  - [energy.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/energy.rs) — Solar exposure and potential energy tracking.
* **Dependencies:** `bevy_ecs`, `config`, `rng`, `time`.

### validation/
* **Purpose:** Post-generation and post-tick runtime invariants checks.
* **Responsibilities:** Performs boundary limit checks, clock monotonicity checks, and array size validation.
* **Key Files:**
  - [systems.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/validation/systems.rs) — validation assertions executed under Bevy schedules.
  - [errors.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/validation/errors.rs) — validation error types.
* **Dependencies:** `bevy_ecs`, `config`, `world`.

### persistence/
* **Purpose:** State snapshot serialization and file save/load.
* **Responsibilities:** Compiles state snapshots, maps ECS components to file models, and writes/loads JSON strings.
* **Key Files:**
  - [io.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/io.rs) — Stable snapshot sort serialization routines.
  - [snapshot.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/snapshot.rs) — Serialization structures.
  - [systems.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/systems.rs) — Persistence tick evaluation systems.
* **Dependencies:** `bevy_ecs`, `config`, `time`, `world`, `serde`, `serde_json`.

### testing/
* **Purpose:** Integration test execution.
* **Responsibilities:** Hosts verification tests for seed sensitivity, ticking determinism, and year-long simulation stability.
* **Key Files:**
  - [determinism.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/determinism.rs) — The Milestone 10 integration test suite.
  - [fixtures.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/fixtures.rs) — Test helpers and world assertions.
* **Dependencies:** `bevy_ecs`, `app`, `config`, `rng`, `time`, `persistence`, `world`.

---

## Dependency Overview Map

The dependency graph below represents the hierarchical boundaries between Genesis modules:

```text
       ┌─────────────────────────────────────────────────────────┐
       │                       app / root                        │
       └────────────────────────────┬────────────────────────────┘
                                    │
                                    ▼
       ┌─────────────────────────────────────────────────────────┐
       │             testing / validation / persistence          │
       └────────────────────────────┬────────────────────────────┘
                                    │
                                    ▼
       ┌─────────────────────────────────────────────────────────┐
       │                     world / agent                       │
       └────────────────────────────┬────────────────────────────┘
                                    │
                                    ▼
       ┌─────────────────────────────────────────────────────────┐
       │                          time                           │
       └────────────────────────────┬────────────────────────────┘
                                    │
                                    ▼
       ┌────────────────────────────┴────────────────────────────┐
       │                    config  /  rng                       │
       └─────────────────────────────────────────────────────────┘
```
All system coordinates, data models, and logic flow downstream. Circular dependencies are avoided.

# Genesis Project Structure

This document catalogs the directory layout and modular architecture of Project Genesis during Phase 3 (as of Milestone 19 completion). It details the purpose, responsibilities, key files, and dependencies of each submodule.

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
  - [PHASE1_WORLD_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/archive/phases/PHASE1_WORLD_TECH_SPEC.md) — Environmental substrate specification.
  - [PHASE1_COMPLETION_REPORT.md](https://github.com/hawary-id/genesis/blob/main/docs/archive/phases/PHASE1_COMPLETION_REPORT.md) — Verification and audit metrics report.
  - [PHASE1_REPOSITORY_STRUCTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/archive/phases/PHASE1_REPOSITORY_STRUCTURE.md) — Repository structure map.
* **Dependencies:** None.

### engine/
* **Purpose:** The main Rust Cargo workspace folder.
* **Responsibilities:** Holds the crate manifest, dependencies lists, and rust sources.
* **Key Files:**
  - [Cargo.toml](https://github.com/hawary-id/genesis/blob/main/engine/Cargo.toml) — Engine dependencies and build targets.
* **Dependencies:** `bevy_ecs`, `serde`, `serde_json`, `rand`, `rand_chacha`.

---

## Crate Source Modules (engine/src/)

All simulation logic resides under [engine/src/](https://github.com/hawary-id/genesis/blob/main/engine/src/). The crate roots are:
* [lib.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/lib.rs) — Crate declaration and submodule exposure.
* [main.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/main.rs) — Binary entry point initializing the app loop.

The crate is structured into the following submodules:

### agent/
* **Purpose:** Biological agent entities structure, metadata, ID generation, spawning, environmental sensing, genetics/phenotype mapping, resource consumption, and asexual reproduction.
* **Responsibilities:** Defines agent metadata with stable sequence identifiers, spatial positions, metabolic stocks, action requests, `Genome` vectors, cached `Phenotype` traits, and `LineageMetadata`. Spawns the initial agent population deterministically, derives agent phenotypes on spawn, provides query utilities to sense resources in neighborhood cells, processes resource consumption and energy stock replenishment, executes deterministic asexual reproduction with parent sorting and cardinal direction coordinate search, enforces population density caps, updates agent metabolism and age, and handles death. Manages the `StableIdGenerator` and `GenomeConfig` resources.
* **Key Files:**
  - [components.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/components.rs) — Agent data structures (`AgentMetadata`, `AgentPosition`, `MetabolicStock`, `ActionRequest`, `Genome`, `Phenotype`, `LineageMetadata`).
  - [resources.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/resources.rs) — Resources (`StableIdGenerator` counter state and `GenomeConfig` traits bounds).
  - [sensing.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/sensing.rs) — Environmental sensing query API (`query_cell`, `query_neighborhood`, and `SensedResource`).
  - [systems.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/systems.rs) — Spawning, phenotype derivation (`derive_phenotype_on_spawn`), resource consumption (`process_agent_consumption`), asexual reproduction (`process_agent_reproduction`), metabolic decay update, age progression, and death processing systems.
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
* **Responsibilities:** Compiles state snapshots, maps ECS components (both environmental chunk data and agent data) to schema version 3 file models, persists the `StableIdGenerator` resource, serializes agent `Genome` and `LineageMetadata`, dynamically reconstructs agent `Phenotype` cache on load, and writes/loads JSON strings.
* **Key Files:**
  - [io.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/io.rs) — Stable snapshot sort serialization and agent load reconstruction routines with phenotype re-derivation.
  - [snapshot.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/snapshot.rs) — Serialization structures supporting version 3 (genetics and lineage metadata).
  - [systems.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/systems.rs) — Persistence tick evaluation systems collecting agent components.
* **Dependencies:** `bevy_ecs`, `config`, `time`, `world`, `agent`, `serde`, `serde_json`.

### testing/
* **Purpose:** Integration test execution.
* **Responsibilities:** Hosts verification tests for seed sensitivity, ticking determinism, save/load equivalence, and year-long simulation stability with active agent populations.
* **Key Files:**
  - [determinism.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/determinism.rs) — The integration test suite verifying determinism and long-run save/load stability.
  - [fixtures.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/fixtures.rs) — Test helpers and world assertions verifying chunks, agent counts, agent states, and ID generator parity.
* **Dependencies:** `bevy_ecs`, `app`, `config`, `rng`, `time`, `persistence`, `world`, `agent`.

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

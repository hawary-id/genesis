# Phase 1 Repository Structure

## Status

* Phase 1 Complete
* Release Tag: v0.1.0-phase1
* Date Generated: 2026-06-14

## Purpose

This document captures the repository layout and module architecture of Project Genesis at the completion of Phase 1 (Milestone 10). It provides an authoritative baseline for future code expansions, planning for Phase 2 (Life), and reviewing design decisions made during the setup of the Environmental Substrate.

## Repository Tree

The plain ASCII structure below lists all directories and source files up to 4 directory levels deep, excluding `target/`, `.git/`, and temporary build outputs:

```text
Genesis/
├── api/ (empty)
├── dashboard/ (empty)
├── database/ (empty)
├── docs/
│   ├── adr/
│   │   ├── ADR-001-ecs-architectural-boundaries.md
│   │   ├── ADR-002-deterministic-execution-contract.md
│   │   ├── ADR-003-spatial-coordinate-model.md
│   │   ├── ADR-004-physical-time-model.md
│   │   └── ADR-005-world-generation-strategy.md
│   ├── references/
│   │   ├── GLOSSARY.md
│   │   └── RESEARCH.md
│   ├── ARCHITECTURE.md
│   ├── ARCHITECTURE_BASELINE.md
│   ├── CODING_STANDARDS.md
│   ├── DETERMINISM.md
│   ├── ECS_GUIDELINES.md
│   ├── MILESTONE10_ARCHITECTURE.md
│   ├── MILESTONE2_ARCHITECTURE.md
│   ├── MILESTONE2_REVIEW.md
│   ├── MILESTONE3_ARCHITECTURE.md
│   ├── MILESTONE4_ARCHITECTURE.md
│   ├── MILESTONE5_ARCHITECTURE.md
│   ├── MILESTONE6_ARCHITECTURE.md
│   ├── MILESTONE7_ARCHITECTURE.md
│   ├── MILESTONE8_ARCHITECTURE.md
│   ├── MILESTONE9_ARCHITECTURE.md
│   ├── PHASE1_IMPLEMENTATION_PLAN.md
│   ├── PHASE1_WORLD_TECH_SPEC.md
│   ├── PRINCIPLES.md
│   ├── ROADMAP.md
│   ├── SPATIAL_MODEL.md
│   ├── TIME_MODEL.md
│   └── VISION.md
├── engine/
│   ├── src/
│   │   ├── app/
│   │   │   ├── events.rs
│   │   │   ├── mod.rs
│   │   │   ├── plugins.rs
│   │   │   └── schedules.rs
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   ├── world_bounds.rs
│   │   │   └── world_config.rs
│   │   ├── persistence/
│   │   │   ├── errors.rs
│   │   │   ├── io.rs
│   │   │   ├── mod.rs
│   │   │   ├── snapshot.rs
│   │   │   └── systems.rs
│   │   ├── rng/
│   │   │   ├── mod.rs
│   │   │   └── seed.rs
│   │   ├── testing/
│   │   │   ├── determinism.rs
│   │   │   ├── fixtures.rs
│   │   │   └── mod.rs
│   │   ├── time/
│   │   │   ├── mod.rs
│   │   │   ├── season_state.rs
│   │   │   └── simulation_clock.rs
│   │   ├── validation/
│   │   │   ├── errors.rs
│   │   │   ├── mod.rs
│   │   │   └── systems.rs
│   │   ├── world/
│   │   │   ├── climate.rs
│   │   │   ├── coord.rs
│   │   │   ├── energy.rs
│   │   │   ├── generation.rs
│   │   │   ├── mod.rs
│   │   │   ├── resource.rs
│   │   │   └── terrain.rs
│   │   ├── lib.rs
│   │   └── main.rs
│   └── Cargo.toml
├── AGENTS.md
├── Cargo.lock
├── Cargo.toml
├── INITIAL_ARCHITECTURE_REVIEW.md
└── README.md
```

## Module Inventory

### app
* **Responsibility:** Serves as the primary application bootstrapper. It owns the Bevy `World`, registers the five Phase 1 schedules, binds simulation systems to their respective schedules, and manages global simulation event signals.
* **Key resources:** `Schedules` (registers `StartupGeneration`, `FixedSimulationTick`, `PostTickValidation`, `PersistenceBoundary`, `ObservationBoundary`), `SnapshotConfig` (tracks interval configuration and schema version).
* **Key systems:** `register_initial_resources` (inserts foundational resources and configures startup settings).
* **Dependencies:** `bevy_ecs`, `config`, `rng`, `time`, `persistence`.

### config
* **Responsibility:** Stores simulation config values and derives grid coordinate boundaries.
* **Key resources:** `WorldConfig` (authoritative ranges, heights, amplitudes, and parameters), `WorldBounds` (tracks chunk dimensions and counts).
* **Key systems:** None.
* **Dependencies:** `bevy_ecs` (for Resource traits).

### rng
* **Responsibility:** Standardizes coordinate-salted seed derivation to support independent, space-invariant randomness.
* **Key resources:** `WorldSeed` (wraps the root `u64` seed).
* **Key systems:** None (exposes helper functions like `derive_chunk_seed`, `derive_terrain_seed`, `derive_resource_seed`).
* **Dependencies:** `rand`, `rand_chacha`.

### time
* **Responsibility:** Drives physical simulation clock progression and tracks seasonal modifiers.
* **Key resources:** `SimulationClock` (tracks total ticks and day lengths), `SeasonState` (tracks current season index, tick offset, progression fraction, and temperature modifier).
* **Key systems:** `advance_simulation_clock` (advances the total tick count), `update_season_state` (updates modifiers on seasonal transitions).
* **Dependencies:** `bevy_ecs`, `config`.

### world
* **Responsibility:** Manages the spatial substrate chunks, initial procedural values generation (terrain, climate, resources, energy), and daily update logic.
* **Key resources:** None (operates via component queries).
* **Key components/structs:** `ChunkCoord`, `TerrainChunk`, `ClimateChunk`, `ResourceChunk`, `EnergyAvailabilityChunk`, `Generated` (marker).
* **Key systems:**
  - *Generation:* `spawn_chunk_entities`, `generate_terrain_chunks`, `generate_climate_chunks`, `generate_resource_chunks`, `generate_energy_availability_chunks`, `mark_generated_chunks`.
  - *Simulation:* `update_climate_fields`, `update_resource_fields`, `update_energy_availability_fields`.
* **Dependencies:** `bevy_ecs`, `config`, `rng`, `time`.

### validation
* **Responsibility:** Performs post-generation and post-tick invariant checks to detect simulation drift, coordinate invalidation, or range errors.
* **Key resources:** None.
* **Key systems:** `validate_world_on_startup` (asserts correctness after startup/load), `validate_world_on_tick` (asserts range limits after each update).
* **Dependencies:** `bevy_ecs`, `config`, `world::coord`.

### persistence
* **Responsibility:** Serializes ECS components into structured JSON snapshots and writes/reads snapshot files on disk.
* **Key resources:** `SnapshotConfig` (interval ticks).
* **Key systems:** `detect_snapshot_due` (scans ticks), `handle_snapshot_requests` (creates and writes snapshot JSONs), `clear_persisted_dirty_markers` (clears chunk flags).
* **Dependencies:** `bevy_ecs`, `config`, `time`, `world::coord`, `serde`, `serde_json`.

### testing
* **Responsibility:** Integrates verification tests to check seed sensitivity, ticking determinism, long-run stability, and save/load equivalence.
* **Key resources:** None.
* **Key systems/tests:** `test_full_world_generation_determinism`, `test_full_world_seed_sensitivity`, `test_full_world_ticking_determinism`, `test_long_run_stability_512`.
* **Dependencies:** `bevy_ecs`, `app`, `config`, `rng`, `time`, `persistence`, `world`, `validation`.

---

## Architecture Boundaries

The dependency flow in Phase 1 is unidirectional and strictly hierarchical to prevent cyclic compilation risks:
* Crate roots (`lib.rs` and `main.rs`) and the orchestration layer (`app`) depend on all modules.
* The `testing`, `validation`, and `persistence` modules act as output boundaries, depending on the internal data structures of `world`, `time`, `config`, and `rng`.
* The `world` module depends on `time` (clock ticks drive updates), `config` (limits/scales), and `rng` (randomness seeds).
* The `time` module depends on `config` for season cycle lengths.
* The `config` and `rng` modules occupy the base, depending only on external crates.

---

## ECS Execution Flow

System execution is sequenced through five schedules registered within Bevy:

```text
StartupGeneration
└── FixedSimulationTick
    └── PostTickValidation
        └── PersistenceBoundary
            └── ObservationBoundary
```

* **StartupGeneration:** Executes once before ticks begin. Validates configuration limits, spawns chunk entities, procedurally generates environmental fields, validates the output, and fires `WorldGenerationCompleted`.
* **FixedSimulationTick:** Executes once per tick. Advances `SimulationClock`, updates `SeasonState` when crossing boundaries, and executes climate, resource, and energy updates on daily boundaries.
* **PostTickValidation:** Executes after ticking systems complete. Asserts that clock monotonicity holds, that chunk dimensions are unaltered, and that float vectors remain within limits.
* **PersistenceBoundary:** Evaluates snapshot triggers. Constructs snapshot buffers, serializes to JSON, writes files, and fires `SnapshotCompleted`.
* **ObservationBoundary:** Reserved for read-only metrics collection and exporter systems. Empty in Phase 1.

---

## Persistence Flow

The save/load pipeline transfers world state across serialization boundaries:

### Save (Serialize)
1. System receives a `SnapshotRequested { path }` event.
2. `handle_snapshot_requests` queries all chunk entities in the world: `(&ChunkCoord, &TerrainChunk, &ClimateChunk, &ResourceChunk, &EnergyAvailabilityChunk)`.
3. The retrieved chunk set is collected into a vector and sorted by coordinate `(coord.y, coord.x)` using a stable sort, neutralizing ECS iteration order variations.
4. Authoritative resources (`WorldConfig`, `WorldSeed`, `SimulationClock`) are fetched.
5. The `WorldSnapshot` structure is constructed, carrying the configuration, clock, root seed, schema version (`1`), and sorted chunk snapshots.
6. The snapshot is serialized to JSON via `serde_json` and written to disk.
7. A `SnapshotCompleted` event is emitted.

### Load (Deserialize)
1. The JSON string is loaded from disk and deserialized into a `WorldSnapshot`.
2. The loader verifies that the schema version matches `SNAPSHOT_SCHEMA_VERSION`.
3. Authoritative resources (`WorldConfig`, `WorldSeed`, `SimulationClock`) are inserted into the ECS world.
4. Existing chunk entities are cleared.
5. Chunk snapshots are processed, spawning entities with matching `TerrainChunk`, `ClimateChunk`, `ResourceChunk`, and `EnergyAvailabilityChunk` components, plus their `ChunkCoord` and a `Generated` marker.
6. Derived resources (`SeasonState`, `WorldBounds`) are reconstructed deterministically from configuration and clock tick counts.
7. Post-generation validation is executed to guarantee state consistency.

---

## Determinism Guarantees

Phase 1 guarantees bit-perfect replication through several core designs:
* **Coord-Salted Seeds:** Rather than calling global generators, every cell coordinate is hashed with the chunk seed and root seed via `rand_chacha::ChaCha8Rng`. Spawning sequence cannot impact random numbers.
* **Sequential Scheduling Chaining:** Systems in `app/mod.rs` use strict `.after()` ordering constraints, preventing multi-threaded execution ordering anomalies.
* **Stable Sort Serialization:** Sorting chunk queries by spatial coordinate before writing ensures JSON outputs are binary-identical.
* **Strict Float Asserts:** Determinism checks verify float values using exact bitwise comparison (`assert_eq!`) rather than epsilon thresholds.

---

## Known Technical Debt

* **Array Indexing Boilerplate:** Calculations of flat 1D indices `local_y * chunk_size + local_x` from 2D coordinates are scattered throughout update systems, increasing the risk of boundary indexing errors.
* **Float Target Divergence:** Normal `f32` operations are compiled to instructions (e.g. SSE/AVX/FMA) depending on host hardware. Across separate architectures (e.g., x86_64 vs ARM64) or compile optimizations, slight compiler float optimizations could theoretically compromise determinism.
* **Synchronous File IO:** JSON snapshot saving/loading blocks Bevy's main thread loop. Large worlds cause frame rate stalls.

---

## Deferred Refactors

* **Separate Crate Modularization:** Extracting config, time, rng, and serialization into separate crates to clean up boundaries and speed up compilation.
* **Grid Field Wrapper:** Introducing a unified, abstract `ChunkField<T>` struct to wrap raw vector storage and clean up index calculation routines.
* **Bevy Plugin Migration:** Replacing custom functions with formal implementations of the `bevy_ecs::prelude::Plugin` trait.
* **Relational Database Storage:** Upgrading snapshot serialization to map elements directly to a Postgres relational database.

---

## Architecture Assessment

The environmental substrate is highly suitable for Phase 2. The physical layers (terrain, climate, resources, energy) are decoupled and behave deterministically. Ticking updates are synchronized on daily boundaries. 
For Phase 2 (Life), introducing autonomous agents will require spawning distinct ECS entities that move dynamically across coordinates. Since chunk components contain cell-local vectors rather than discrete cell entities, agents must query their local coordinate, locate the corresponding chunk entity, and offset their index into the chunk's resource arrays to feed, move, or spawn. This decoupled layout prevents core environmental component changes while allowing agents to be handled as standard ECS entities.

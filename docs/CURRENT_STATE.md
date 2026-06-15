# Current State

### Phase Status

* Phase 1: COMPLETE
* Phase 2: COMPLETE
* Phase 3: ACTIVE

### Completed Milestones

* Milestone 16 — Genetics & Phenotype Mapping
* Milestone 17 — Resource Consumption
* Milestone 18 — Reproduction, Inheritance & Lineage
* Milestone 19 — Mutation Engine & Genetic Drift

### Current Active Milestone

* Milestone 20 — Natural Selection & Adaptation

### Newly Added Systems

* Deterministic Gaussian mutation integrated into reproduction using Box-Muller transform
* Deterministic mutation seed derivation via SplitMix64 finalizer (`deterministic_mix_64` function)
* Transient stateless `ChaCha8Rng` for reproductive mutation trials
* Explicit validation checks for non-empty and finite genomes (`is_finite` check)
* Asexual reproduction (`process_agent_reproduction` system)
* Lineage propagation and stable ID generation during reproduction
* Emergency density cap enforcement during reproduction
* Resource consumption (`process_agent_consumption` system)
* Nutrient & Fresh Water cell harvesting and environmental depletion (Conservation of Mass)
* Diet-preference scaled assimilation to energy stock
* Stable ID sorting ascending for consumption determinism
* Consumption parameters config (`max_harvest_rate` & `consumption_efficiency`)
* Genetics foundation (`Genome`, `Phenotype`, `LineageMetadata`, `GenomeConfig`)
* Phenotype derivation on spawn/load (`derive_phenotype_on_spawn`)
* Snapshot v3 persistence (`Genome` & `LineageMetadata` serialized, `Phenotype` re-derived)

### Verification & Testing Status

* **Branch:** main
* **Status:** Milestone 19 is fully completed, verified, clippy-compliant, and passes determinism and snapshot validation.
* **Test Counts:**
  - `cargo test`: 144 passed, 0 failed, 1 ignored
  - `cargo test -- --ignored`: 1 passed (test_long_run_stability_512 checks A+B=N save/load equivalence over 8,640 ticks / 1 simulation year with genetics, consumption, and reproduction enabled)
  - `cargo clippy --all-targets --all-features -- -D warnings`: PASS
  - `cargo fmt`: PASS
* **Last Updated:** 2026-06-15T16:51:30+07:00

---

## Completed in Milestone 19: Mutation Engine & Genetic Drift

* **Deterministic Mutation**: Implemented Gaussian gene mutation during asexual reproduction, utilizing a local, transient `ChaCha8Rng` instance seeded deterministically per reproduction event.
* **Seeding & Seed Derivation**: Derived platform-invariant mutation seeds using a SplitMix64 finalizer hash (`deterministic_mix_64`) mixing world seed, parent ID, tick, and parent coordinate (which acts as spatial salt).
* **Configuration Parameters**: Added `mutation_rate` and `mutation_step_size` config parameters to the centralized `WorldConfig` resource.
* **Dynamic Mutation Loop**: Mutation loops process the active genes vector length (`parent_genome.genes.len()`) to support dynamic genome size expansion in future phases.
* **Genetics Validation Invariants**: Added explicit validation checks in `validate_world_on_startup` and `validate_world_on_tick` verifying that genomes are non-empty, and all gene values are finite (`is_finite()`) and clamped to `[0.0, 1.0]`.
* **Save/Load Equivalence Integration**: Verified that stateless mutation RNGs do not break save/load determinism, successfully maintaining bit-perfect A+B=N run equivalence.
* **Emergent Genetic Drift**: Validated that genetic drift emerges purely as a consequence of spatial carrying capacity and environmental selection pressures over generations.

## Completed in Milestone 18: Reproduction, Inheritance & Lineage

* **Deterministic Reproduction**: Implemented the asexual reproduction system `process_agent_reproduction` processing birth requests deterministically.
* **Deterministic Parent Ordering**: Reproduction requests are sorted by `AgentMetadata.id` ascending before processing offspring entities to guarantee order-independence.
* **Stable ID Allocation**: Allocates unique sequence identifiers to offspring via the `StableIdGenerator` resource.
* **Energy Splitting**: Divides parent energy stock 50/50, allocating half to the parent and half to the offspring.
* **Genome Inheritance**: Transmits parent genome to offspring.
* **Lineage Propagation**: Populates `LineageMetadata` with `parent_id` (parent stable ID) and `generation` (parent generation + 1).
* **Density Cap Enforcement**: Enforces `agent_density_cap` as an emergency population safety guard.
* **Persistence Compatibility**: Verified that world snapshots deserialize and reconstruct variable agent counts and lineages.
* **Phenotype Status Explicit Note**: Phenotype currently exists, is inherited, and is reconstructed after load. Phenotype intentionally does NOT yet influence:
  * movement
  * metabolism
  * survival pressure
  This behavior is deferred to Milestone 20 and must not be treated as a bug.

## Completed in Milestone 17: Resource Consumption (Eating & Drinking)

* **Deterministic Consumption System**: Implemented the `process_agent_consumption` system to harvest cell nutrients and fresh water resource components.
* **Mass Conservation**: Subtracts harvested resource quantities directly from chunk cells and adds corresponding energy to agent `MetabolicStock`, clamped at `agent_energy_max`.
* **Preference-Scaled Assimilation**: Integrates agent `diet_preference` and global `consumption_efficiency` to scale energy gains (omnivore vs specialist).
* **ID-Based Order Determinism**: Collects and sorts agent list by unique `AgentMetadata.id` ascending to ensure order-independent consumption execution.
* **Configuration Metrics**: Added `max_harvest_rate` and `consumption_efficiency` to `WorldConfig` and testing configurations.

## Completed in Milestone 16: Genetics & Phenotype Mapping

* **Extensible Genome Component**: Added the `Genome` component storing genetic traits in a `Vec<f32>` format of length `8` for startup agents, allowing future genes to be appended without breaking struct layout compiler interfaces.
* **Lineage & Generation Tracking**: Added the `LineageMetadata` component storing `parent_id` (`Option<u64>`) and `generation` (`u32`). Startup agents default to `parent_id = None` and `generation = 0`.
* **Phenotype Derivation & Cache**: Added the `Phenotype` component caching mapped physical traits. Mapped raw gene float arrays to concrete trait bounds defined in `GenomeConfig`. Base metabolic decay and movement step cost penalties are mathematically derived based on size, sensing radius, slope tolerance, and water limits. *Note: In accordance with the roadmap, these derived traits do not yet influence movement, metabolism, or survival; their integration into agent behavior systems is deferred to Milestone 20 (Natural Selection & Adaptation).*
* **Snapshot Version 3 Upgrade**: Upgraded snapshot version to schema version `3` to serialize `Genome` and `LineageMetadata`. Excluded `Phenotype` from serialization, re-deriving it dynamically on load.
* **Genetics Validation**: Implemented validation systems in `validate_world_on_startup` and `validate_world_on_tick` checking genome boundaries and lineage metadata.
* **Restored World Spawning Rules**: Bypassed initial world count and age/energy constraints in startup validation when `clock.total_ticks > 0` to support loaded snapshots.

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
> ClimateChunk and TerrainChunk lookups in agent sensing, metabolism, movement, and consumption currently perform linear chunk scans (O(agent_count × chunk_count)).
> This is intentionally accepted for Phase 2/3 correctness-first implementation and should be replaced with indexed chunk lookup (HashMap or equivalent deterministic spatial index) during a future performance optimization pass.

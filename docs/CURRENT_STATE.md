# Current State

### Phase Status

* Phase 1: COMPLETE
* Phase 2: COMPLETE
* Phase 3: ACTIVE

### Last Completed Milestone

Milestone 21 — Evolution Diagnostics and Validation

Key Outcomes:
* `PopulationStatistics` runtime telemetry resource added (fully reconstructable from ECS state; not persisted)
* `compute_population_statistics` system registered in `ObservationBoundary` schedule
* Bidirectional lineage invariant enforced in both startup and post-tick validation:
  - `generation == 0 ⟹ parent_id == None`
  - `generation > 0 ⟹ parent_id == Some(...)`
* Genome padding on snapshot restore: undersized genomes are padded to `GENOME_SIZE` on load
* `GENOME_SIZE` extracted as a `pub const` in `agent/components.rs` (no more magic numbers)
* All validation invariants verified with bidirectional test coverage

### Current Focus

* Phase 3 evolution diagnostics and validation complete. Ready for next milestone.

### Current Active Milestone

* Milestone 21 — Evolution Diagnostics and Validation (COMPLETE)

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
* Metabolic adaptation linking Phenotype base decay to survival
* Movement adaptation linking Phenotype movement cost and slope/water_depth limits to navigation
* Climate adaptation and terrain specialization deterministic integration tests
* `PopulationStatistics` resource and `compute_population_statistics` telemetry system (M21)
* Bidirectional lineage invariants in validation (M21)
* Genome reconstruction padding on snapshot restore (M21)

### Verification & Testing Status

* **Branch:** main
* **Status:** Milestone 21 is fully completed, verified, clippy-compliant, and passes all determinism and snapshot validation tests.
* **Test Counts:**
  - `cargo test`: 131 passed, 0 failed, 1 ignored
  - `cargo test -- --ignored`: 1 passed (test_long_run_stability_512 checks A+B=N save/load equivalence over 8,640 ticks / 1 simulation year with full Phase 3 evolution stack active)
  - `cargo clippy -- -D warnings`: PASS
  - `cargo fmt`: PASS
* **Last Updated:** 2026-06-16T08:37:00+00:00

---

## Known Technical Debt

> [!NOTE]
> **ClimateChunk and TerrainChunk Lookup Scan Complexity**
> ClimateChunk and TerrainChunk lookups in agent sensing, metabolism, movement, and consumption currently perform linear chunk scans (O(agent_count × chunk_count)).
> This is intentionally accepted for Phase 2/3 correctness-first implementation and should be replaced with indexed chunk lookup (HashMap or equivalent deterministic spatial index) during a future performance optimization pass.

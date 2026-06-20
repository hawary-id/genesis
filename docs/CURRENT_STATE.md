# Current State

### Phase Status

* Phase 1: COMPLETE
* Phase 2: COMPLETE
* Phase 3: COMPLETE
* Phase 4: COMPLETE

### Last Completed Milestone

Milestone 24 — Social Memory (Kinship Memory Foundation)

Key Outcomes:
* `SocialRelationCategory` enum (`Parent`, `Child`)
* `SocialMemoryNode` and `SocialMemory` ECS components
* Reciprocal `SocialMemoryEvent` generation during reproduction
* Deterministic consolidation with capacity limit 10 and oldest-first eviction
* SocialMemory persistence added (snapshot schema v5 upgrade)
* Validation checks for capacity bounds, duplicate targets, chronologies, and self-reference prevention

### Current Focus

* Phase 4 (Memory) is COMPLETE. Next focus is Phase 5 (Agency) planning.

### Next Planned Milestone

* Milestone 25 — Goal Formation Foundation

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

* **Branch:** phase4-m24
* **Snapshot Schema Version:** v5
* **Status:** Milestone 24 COMPLETE. Phase 4 COMPLETE.
  - Kinship memory tracking (`Parent`/`Child`) during reproduction.
  - Strict deterministic consolidation and oldest-first eviction.
  - Snapshot persistence v5 upgrade with backward compatibility.
  - Comprehensive invariant validation support.
* **Test Counts:**
  - `cargo test`: PASS
  - `cargo test -- --ignored`: PASS (test_long_run_stability_512 checks A+B=N save/load equivalence over 8,640 ticks / 1 simulation year with full stack active)
  - `cargo clippy -- -D warnings`: PASS
  - `cargo fmt`: PASS
* **Last Updated:** 2026-06-20T14:20:00+00:00

---

## Known Technical Debt

> [!NOTE]
> **ClimateChunk and TerrainChunk Lookup Scan Complexity**
> ClimateChunk and TerrainChunk lookups in agent sensing, metabolism, movement, and consumption currently perform linear chunk scans (O(agent_count × chunk_count)).
> This is intentionally accepted for Phase 2/3 correctness-first implementation and should be replaced with indexed chunk lookup (HashMap or equivalent deterministic spatial index) during a future performance optimization pass.

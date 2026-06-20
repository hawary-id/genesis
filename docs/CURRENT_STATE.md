# Current State

### Phase Status

* Phase 1: COMPLETE
* Phase 2: COMPLETE
* Phase 3: COMPLETE
* Phase 4: ACTIVE

### Last Completed Milestone

Milestone 23 — Event Memory

Key Outcomes:
* `EventCategory` enum (`ResourceConsumed`, `FailedMovement`, `Reproduced`, `HazardEncountered`)
* `EventMemoryNode` and `EventMemory` ECS components
* `EventMemoryEvent` and `EventSequenceCounter` resource for strict deterministic chronological ordering
* `process_event_memory_consolidation` system with fixed-capacity chronological eviction (oldest entries removed first, MAX_EVENT_MEMORY_CAPACITY = 10)
* EventMemory persistence added (snapshot schema v4 upgrade) with backwards compatibility
* Validation checks for capacity bounds and strict chronological properties

### Current Focus

* Phase 4 (Memory) continues. Milestone 23 (Event Memory) implemented. Next focus is M24 planning.

### Next Planned Milestone

* Milestone 24 — Social Memory

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

* **Branch:** phase4-m23
* **Snapshot Schema Version:** v4
* **Status:** Milestone 23 COMPLETE.
  - Event memory tracking system with distinct event categories.
  - Strict deterministic chronologial ordering within ticks via sequence counters.
  - Snapshot persistence v4 upgrade with backward compatibility.
  - Validation support and determinism verified.
* **Test Counts:**
  - `cargo test`: PASS
  - `cargo test -- --ignored`: PASS (test_long_run_stability_512 checks A+B=N save/load equivalence over 8,640 ticks / 1 simulation year with full stack active)
  - `cargo clippy -- -D warnings`: PASS
  - `cargo fmt`: PASS
* **Last Updated:** 2026-06-20T12:00:00+00:00

---

## Known Technical Debt

> [!NOTE]
> **ClimateChunk and TerrainChunk Lookup Scan Complexity**
> ClimateChunk and TerrainChunk lookups in agent sensing, metabolism, movement, and consumption currently perform linear chunk scans (O(agent_count × chunk_count)).
> This is intentionally accepted for Phase 2/3 correctness-first implementation and should be replaced with indexed chunk lookup (HashMap or equivalent deterministic spatial index) during a future performance optimization pass.

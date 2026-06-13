# Milestone 5 Architecture — Resource System

**Date:** 2026-06-13  
**Status:** Approved and Locked  
**Dependencies:** Milestone 1 ✅, Milestone 2 ✅, Milestone 3 ✅, Milestone 4 ✅  

---

## Architecture Review

### Relevant Roadmap References
- **`PHASE1_WORLD_TECH_SPEC.md`:** Standardizes the `ResourceChunk` variables: fresh water availability, nutrients, minerals, and biomass carrying potential.
- **`PHASE1_IMPLEMENTATION_PLAN.md`:** Places the Resource System as Milestone 5, acting as the primary abiotic material constraint for the future Life system.

### Relevant ADR Constraints
- **[ADR-001: ECS Boundaries](file:///c:/Genesis/docs/adr/ADR-001-ecs-architectural-boundaries.md):** `ResourceChunk` must remain a passive, data-only component. System logic handles all generation and periodic calculations. No manager classes.
- **[ADR-002: Deterministic Execution Contract](file:///c:/Genesis/docs/adr/ADR-002-deterministic-execution-contract.md):** Seeding uses coordinate-salted deterministic generators. Clock ticks are fixed. Updates run on daily boundaries.
- **[ADR-003: Spatial Model](file:///c:/Genesis/docs/adr/ADR-003-spatial-coordinate-model.md):** Arrays are stored contiguously in row-major order within chunk entities.
- **[ADR-004: Physical Time Model](file:///c:/Genesis/docs/adr/ADR-004-physical-time-model.md):** Tick count drives the cycle cadence of updates.
- **[ADR-005: World Generation Strategy](file:///c:/Genesis/docs/adr/ADR-005-world-generation-strategy.md):** Unique salted domain derivation for resources RNG initialization.

### Dependency Analysis from Milestone 1–4
- **Milestone 3 (Terrain):** Reads `elevation`, `slope`, `water_depth`, `soil_depth`, and `soil_fertility` variables to calculate baseline nutrients and water accumulation.
- **Milestone 4 (Climate):** Reads cell `temperature`, `moisture`, and `rainfall` values to drive regeneration, runoff decay, and biomass potential.
- **Milestone 2 (SimulationClock):** Ticking clock controls the daily update trigger.

---

## Milestone 5 Scope

### Objectives
Generate and update material environmental resource fields as low-level abiotic availability parameters.

### Responsibilities
- Initial baseline resource generation on startup.
- Daily resource field updates in response to terrain hydrology limits, climate oscillations, and seasonal inputs.
- Post-generation and post-tick validation checking that resources never slip below zero.

### Explicit Non-Goals
- **No economic goods:** No prices, ownership, labor, trade, exchange, storage vaults, or commodity consumption.
- **No agent biological consumption:** No active plant/animal metabolism, growth cycle, or grazing (deferred to the Life phase).
- **No active biomass:** Biomass carrying potential represents the environment's carrying capacity, not active biological counts.

### Boundaries
- Updates are strictly cell-local. No cross-chunk transfers or boundary hydration sync cycles.

---

## ECS Architecture

### Components
- `ResourceChunk` (Component, Serialize, Deserialize):
  - `fresh_water: Vec<f32>` (normalized range `[0.0, fresh_water_max]`)
  - `nutrients: Vec<f32>` (normalized range `[0.0, nutrients_max]`)
  - `minerals: Vec<f32>` (normalized range `[0.0, minerals_max]`)
  - `biomass_potential: Vec<f32>` (normalized range `[0.0, biomass_potential_max]`)

### Resources
- None added. (Existing `WorldConfig`, `WorldBounds`, `WorldSeed`, `SimulationClock` are reused).

### Events
- None added.

### Systems

#### `generate_resource_chunks` (Startup)
- **Reads:** `ChunkCoord`, `TerrainChunk`, `ClimateChunk`, `WorldConfig`, `WorldSeed`
- **Writes:** Inserts `ResourceChunk` on chunk entities.
- **Location:** `world::generation`

#### `update_resource_fields` (Simulation Tick)
- **Reads:** `ChunkCoord`, `TerrainChunk`, `ClimateChunk`, `SimulationClock`, `WorldConfig`
- **Writes:** `ResourceChunk` (mutable borrow)
- **Location:** `world::resource`

### Schedule Integration

#### `StartupGeneration` Update
```text
1. validate_world_config
2. spawn_chunk_entities
3. generate_terrain_chunks
4. generate_climate_chunks
5. generate_resource_chunks [NEW]
6. validate_generated_world [NEW - replaces Milestone 4 validate_generated_world]
7. mark_chunks_generated [NEW - requires Terrain, Climate, and Resource components]
8. emit_world_generation_completed
```

#### `FixedSimulationTick` Update
```text
1. update_climate_fields
2. update_resource_fields [NEW]
```

---

## Resource Generation Architecture

### Input Dependencies
1. **Fresh Water:** Derived from terrain `water_depth` (high weight) and initial climate `moisture` / `rainfall`.
2. **Nutrients:** Derived from `soil_fertility` and `soil_depth`.
3. **Minerals:** Derived from coordinate-salted value noise representing geological deposits.
4. **Biomass Potential:** Derived from temperature, moisture, and soil nutrient baseline availability.

### Generation Ordering
- Setup resource RNG using coordinate-salt on the derived resource seed stream.
- Evaluate mineral noise.
- Extract terrain elevation and water profiles.
- Formulate moisture/nutrient coefficients to calculate baseline biomass capacity.

### Resource Field Relationships
- Nutrients are bounded by local `soil_depth`.
- Water presence replenishment increases local moisture, supporting nutrient solubility and biomass carry threshold.
- Minerals are static, geological deposits that are distributed deterministically on startup.

### Runtime Update Model
- **Update Cadence:** The resource update system must run on a periodic daily interval driven by the simulation clock.
- **Water Dynamics:** Fresh water levels must adapt dynamically, increasing with local rainfall/precipitation and decaying in response to local temperature-driven evaporation and terrain-driven slope runoff.
- **Nutrient Dynamics:** Nutrients must reflect a balance between depletion (such as leaching runoff on sloped terrain) and slow replenishment based on underlying soil characteristics and temperature.
- **Mineral Dynamics:** Minerals are static geological deposits and remain constant over time.
- **Biomass Carrying Potential:** Biomass carrying potential must be derived dynamically on each update cycle based on temperature suitability, moisture availability, and nutrient levels.

---

## Determinism Analysis

### Seed Derivation Requirements
- **Domain Seeding:** The resource system must derive its own domain seed from the root world seed using a deterministic salt. The specific salt constant value is deferred to the implementation.
- **Chunk RNG Seeding:** Each chunk must initialize its deterministic RNG using a seed derived from the domain seed and the chunk's spatial coordinates. This RNG is used for procedural resource generation (such as mineral deposit distribution).

### Stable Iteration Requirements
- Chunk logic reads only cell-local values from components.
- No shared mutable RNG state across chunks.

### Save/Load Equivalence Requirements
- The `ResourceChunk` vectors (`fresh_water`, `nutrients`, `minerals`, `biomass_potential`) must be persisted in the snapshot.
- Continuation tick calculations must yield binary-identical outputs to continuous updating.

---

## Validation Architecture

### Invariants
- All resource quantities must remain non-negative ($val \ge 0.0$).
- All values must remain below configured limits.

### Range Validation
Add validation limits to `WorldConfig`:
```rust
pub fresh_water_max: f32,
pub nutrients_max: f32,
pub minerals_max: f32,
pub biomass_potential_max: f32,
```
Verification runs post-generation and post-tick (`validate_generated_world` and post-tick validations), asserting value correctness.

### Failure Handling
- Panic in debug/test builds on invariant failure.
- Returns structured validation error in release mode.

---

## Persistence Impact

### Snapshot Requirements
- `fresh_water`, `nutrients`, `minerals`, and `biomass_potential` are serialized in JSON snapshots.
- Sub-fields are fully serialized as flat vectors.

### Deterministic Continuation Requirements
- Restoring state from a file instantiates `ResourceChunk` components with identical vector layouts.

---

## Observation Impact

### Metrics
- Mean resource quantities across chunks (e.g. average soil nutrition index, fresh water volume density).
- Mineral geological mapping telemetry.

### Read-Only Observation Outputs
- Grid array data mapped row-major for visualization in telemetry boundaries.

---

## Risks

### Architectural Risks
- **Ecosystem creep:** Risk of implementing active foliage growth, populations, or metabolic budgets before agents exist.
  - *Mitigation:* Biomass carrying potential must remain a passive availability scalar limit, not a dynamic agent model.

### Determinism Risks
- **Negative value drift:** Subtracting decay values (evaporation, runoff) could cause float variables to slip below zero due to precision rounding.
  - *Mitigation:* Apply `.max(0.0)` clamp on all calculations.

### Scalability Risks
- Multi-vector allocation scaling overhead.
  - *Mitigation:* Vectors are initialized once on startup/loading; values are updated in place to prevent memory fragmentation.

---

## Architecture Decisions Deferred To Implementation

The following details and parameters are deferred to the implementation phase:
- **Numerical Constants and Coefficients:** Specific values for decay rates, leaching coefficients, runoff rates, evaporation rates, and temperature/moisture/nutrient scaling curves.
- **Seeding and Salt Value Constants:** The specific integer or hex values used as domain-specific salts for deterministic seed derivation.
- **Exact Formulation of Derived Values:** The precise equations for combining temperature, moisture, and nutrients to calculate biomass carrying capacity/potential and daily resource regeneration.

## Future Design Considerations

- *Are mineral deposits depletable in Phase 1?*  
  **Recommendation:** No. They are static geological values. Depletion mechanics belong to later industrial/labor civilization milestones.
- *Does rainfall decay soil nutrients?*  
  **Recommendation:** Yes, a minor leaching runoff coefficient based on `slope` should be applied, providing environmental pressure that forces future life to adapt to steep terrain resource scarcity.

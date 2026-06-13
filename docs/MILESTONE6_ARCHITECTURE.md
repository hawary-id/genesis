# Milestone 6 Architecture — Energy Availability System

**Date:** 2026-06-13  
**Status:** Approved and Locked  
**Dependencies:** Milestone 1 ✅, Milestone 2 ✅, Milestone 3 ✅, Milestone 4 ✅, Milestone 5 ✅  

---

## Architecture Review

### Relevant Roadmap References
- **`PHASE1_WORLD_TECH_SPEC.md`:** Standardizes the `EnergyAvailabilityChunk` component and fields, highlighting energy availability as a first-class environmental substrate factor.
- **`PHASE1_IMPLEMENTATION_PLAN.md`:** Places the Energy Availability System as Milestone 6, acting as the primary abiotic energy constraint for the future Life system.

### Relevant ADR Constraints
- **[ADR-001: ECS Boundaries](file:///c:/Genesis/docs/adr/ADR-001-ecs-architectural-boundaries.md):** `EnergyAvailabilityChunk` must remain a passive, data-only component. System logic handles all generation and periodic calculations. No manager classes.
- **[ADR-002: Deterministic Execution Contract](file:///c:/Genesis/docs/adr/ADR-002-deterministic-execution-contract.md):** Seeding uses coordinate-salted deterministic generators. Clock ticks are fixed. Updates run on daily boundaries.
- **[ADR-003: Spatial Model](file:///c:/Genesis/docs/adr/ADR-003-spatial-coordinate-model.md):** Arrays are stored contiguously in row-major order within chunk entities.
- **[ADR-004: Physical Time Model](file:///c:/Genesis/docs/adr/ADR-004-physical-time-model.md):** Tick count drives the cycle cadence of updates.
- **[ADR-005: World Generation Strategy](file:///c:/Genesis/docs/adr/ADR-005-world-generation-strategy.md):** Unique seed derivation strategy for energy availability RNG initialization.

### Dependency Analysis from Milestone 1–5
- **Milestone 3 (Terrain):** Provides elevation, slope, and water depth.
- **Milestone 4 (Climate):** Provides temperature, moisture, and sunlight/latitude factor.
- **Milestone 5 (Resource):** Provides nutrients, minerals, and biomass carrying potential.
- **Milestones 1 & 2 (ECS & Time Clock):** Monotonic ticking clock controls simulation progress.

---

## Milestone 6 Scope

### Objectives
Generate and update first-class environmental energy availability as an aggregate resource field in the environmental substrate.

### Responsibilities
- Initial baseline energy availability and solar exposure generation on startup.
- Dynamic energy availability field updates in response to terrain, climate, and resource inputs.
- Post-generation and post-tick validation checking that energy fields remain non-negative.

### Explicit Non-Goals
- **No agent biological consumption:** No active plant/animal metabolism, energy storage in organisms, energy expenditure, or hunger meters.
- **No thermodynamic transfer simulation:** No local heat conduction, convection, or cell-to-cell thermodynamic transfer calculations.
- **No industrial resource conversion:** No converting resources or minerals into fuel stockpiles, electricity, or economic goods.

### Boundaries
- Updates are strictly cell-local. No cross-chunk transfers or boundary hydration sync cycles.

---

## ECS Architecture

### Components
- `EnergyAvailabilityChunk` (Component, Serialize, Deserialize):
  - `solar_exposure: Vec<f32>` (normalized range `[0.0, solar_exposure_max]`)
  - `energy_availability: Vec<f32>` (normalized range `[0.0, energy_availability_max]`)

### Resources
- None added. Existing `WorldConfig`, `WorldBounds`, `WorldSeed`, `SimulationClock`, and `SeasonState` are reused.

### Events
- None added.

### Systems

#### `generate_energy_availability_chunks` (Startup)
- **Reads:** `ChunkCoord`, `TerrainChunk`, `ClimateChunk`, `ResourceChunk`, `WorldConfig`, `WorldSeed`
- **Writes:** Inserts `EnergyAvailabilityChunk` on chunk entities.
- **Location:** `world::energy`

#### `update_energy_availability_fields` (Simulation Tick)
- **Reads:** `ChunkCoord`, `TerrainChunk`, `ClimateChunk`, `ResourceChunk`, `SimulationClock`, `WorldConfig`, `SeasonState`
- **Writes:** `EnergyAvailabilityChunk` (mutable borrow)
- **Location:** `world::energy`

### Schedule Integration

#### `StartupGeneration` Update
```text
1. validate_world_config
2. spawn_chunk_entities
3. generate_terrain_chunks
4. generate_climate_chunks
5. generate_resource_chunks
6. generate_energy_availability_chunks [NEW]
7. validate_generated_world [UPDATED - includes energy checks]
8. mark_chunks_generated [UPDATED - requires Terrain, Climate, Resource, and Energy availability components]
9. emit_world_generation_completed
```

#### `FixedSimulationTick` Update
```text
1. update_climate_fields
2. update_resource_fields
3. update_energy_availability_fields [NEW]
```

---

## Energy Availability Architecture

### Input Dependencies
- Energy availability depends on terrain-derived state.
- Energy availability depends on climate-derived state.
- Energy availability depends on resource-derived state.
- Exact environmental relationships, equations, and combination functions are deferred to implementation.

### Generation Ordering
- Runs during startup within `StartupGeneration` after terrain, climate, and resource vectors are generated. Seeds are coordinate-salted chunk by chunk, generating `solar_exposure` and initial `energy_availability` arrays.

### Relationship to Terrain
- Terrain elevation and slope fields serve as input dependencies for the calculation of cell solar exposure and aggregate energy availability.

### Relationship to Climate
- Climate temperature, sunlight/latitude factor, and seasonal modifiers serve as input dependencies for the calculation of cell solar exposure and aggregate energy availability.

### Relationship to Resources
- Resource biomass carrying potential and nutrients serve as input dependencies for the calculation of aggregate energy availability.

### Runtime Update Model
- Update cadence is driven by the simulation clock.
- Energy availability is recalculated from current terrain, climate, resource, and seasonal state.
- Exact update equations, coefficients, and balancing parameters are deferred to implementation.

---

## Determinism Analysis

### Seed Derivation Requirements
- **Domain Seeding:** Energy generation must derive entropy from the deterministic root seed using the established domain-isolation strategy.
- **Chunk RNG Seeding:** Each chunk derives its unique seed using coordinates to initialize the `rand_chacha` generator per chunk, preserving parallel order independence.

### Stable Iteration Requirements
- Chunk logic reads only cell-local values from components.
- No shared mutable RNG state across chunks.

### Save/Load Equivalence Requirements
- The `EnergyAvailabilityChunk` vectors (`solar_exposure`, `energy_availability`) must be persisted in the snapshot.
- Continuation tick calculations must yield binary-identical outputs to continuous updating.

---

## Validation Architecture

### Invariants
- All energy availability and solar exposure quantities must remain non-negative ($val \ge 0.0$).
- All values must remain below configured limits.

### Range Validation
Add validation limits to `WorldConfig`:
```rust
pub solar_exposure_max: f32,
pub energy_availability_max: f32,
```
Verification runs post-generation and post-tick (`validate_generated_world` and post-tick validations), asserting value correctness.

### Failure Handling
- Panic in debug/test builds on invariant failure.
- Returns structured validation error in release mode.

---

## Persistence Impact

### Snapshot Requirements
- `solar_exposure` and `energy_availability` are serialized in JSON snapshots as flat vectors.

### Deterministic Continuation Requirements
- Restoring state from a file instantiates `EnergyAvailabilityChunk` components with identical vector layouts.

---

## Observation Impact

### Metrics
- Mean and maximum energy availability across chunks.
- Total solar exposure mapping.

### Read-Only Observation Outputs
- Grid array data mapped row-major for visualization in telemetry boundaries.

---

## Risks

### Architectural Risks
- **Ecosystem creep:** Risk of implementing active foliage growth, populations, or metabolic budgets before agents exist.
  - *Mitigation:* Energy availability must remain a passive availability scalar limit, not a dynamic agent model.

### Determinism Risks
- **Negative value drift:** Subtracting decay values could cause float variables to slip below zero due to precision rounding.
  - *Mitigation:* Apply `.max(0.0)` clamp on all calculations.
- **Platform-specific compilation drift:** Small compiler optimizations or differences in float handling on different architectures.
  - *Mitigation:* Avoid transcendental float math (`sin`, `cos`, `pow`) in the tick update calculations, limiting formulas to basic arithmetic.

### Scalability Risks
- Multi-vector allocation scaling overhead.
  - *Mitigation:* Vectors are initialized once on startup/loading; values are updated in place to prevent memory fragmentation.

---

## Open Questions

- Should solar exposure fluctuate on a diurnal (hourly) tick cadence, or should it remain daily-averaged and seasonally-scaled?
- What mathematical equations or scaling coefficients should be used to combine solar, thermal, and biomass potential into aggregate energy availability?
- How will energy availability interact with future life extraction, and does it require dynamic drawdown and replenishment rate mechanics in Phase 1?
- Should energy availability be one combined field initially, or separate solar, thermal, biomass, and chemical fields?

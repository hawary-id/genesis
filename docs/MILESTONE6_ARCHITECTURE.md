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
- **[ADR-001: ECS Boundaries](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-001-ecs-architectural-boundaries.md):** `EnergyAvailabilityChunk` must remain a passive, data-only component. System logic handles all generation and periodic calculations. No manager classes.
- **[ADR-002: Deterministic Execution Contract]:**
Energy availability must remain a pure deterministic function of previously generated world state and configuration parameters. Clock ticks are fixed and updates run on daily boundaries.
- **[ADR-003: Spatial Model](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-003-spatial-coordinate-model.md):** Arrays are stored contiguously in row-major order within chunk entities.
- **[ADR-004: Physical Time Model](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-004-physical-time-model.md):** Tick count drives the cycle cadence of updates.
- **[ADR-005: World Generation Strategy](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-005-world-generation-strategy.md):** Energy availability is derived directly and deterministically from other generated substrate components.

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
No new resources are introduced.

The system reuses:
- WorldConfig
- SimulationClock

and consumes existing world-state components:
- ChunkCoord
- TerrainChunk
- ClimateChunk
- ResourceChunk

### Events
- None added.

### Systems

#### `generate_energy_availability_chunks` (Startup)
- **Reads:** `ChunkCoord`, `TerrainChunk`, `ClimateChunk`, `ResourceChunk`, `WorldConfig`
- **Writes:** Inserts `EnergyAvailabilityChunk` on chunk entities.
- **Location:** `world::energy`

#### `update_energy_availability_fields` (Simulation Tick)
- **Reads:** `ChunkCoord`, `TerrainChunk`, `ClimateChunk`, `ResourceChunk`, `SimulationClock`, `WorldConfig`
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
- Energy availability depends on terrain-derived state (elevation and slope).
- Energy availability depends on climate-derived state (temperature and sunlight factor).
- Energy availability depends on resource-derived state (nutrients and biomass potential).
- Relationships, equations, and combination functions are parameterized via `WorldConfig` coefficients.

### Generation Ordering
- Runs during startup within `StartupGeneration` after terrain, climate, and resource vectors are generated, producing `solar_exposure` and initial `energy_availability` arrays deterministically from inputs.

### Relationship to Terrain
- Terrain elevation and slope fields serve as input dependencies for the calculation of cell solar exposure and aggregate energy availability.

### Relationship to Climate
- Climate temperature, sunlight/latitude factor, and seasonal modifiers serve as input dependencies for the calculation of cell solar exposure and aggregate energy availability.

### Relationship to Resources
- Resource biomass carrying potential and nutrients serve as input dependencies for the calculation of aggregate energy availability.

### Runtime Update Model
- Update cadence is driven by the simulation clock.
- Energy availability is recalculated from current terrain, climate, and resource state.
- Update equations use the parameterized coefficients and weights configured in `WorldConfig`.

### Mathematical Model & Helper Functions
To avoid calculation duplication and ensure model consistency, the system uses two pure helper functions in [energy.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/energy.rs):

1. **Solar Exposure (`calculate_solar_exposure`):**
   $$solar\_exposure = base\_solar \times solar\_exposure\_max$$
   where:
   $$base\_solar = \Big(sunlight\_factor \times (1.0 + elevation \times solar\_elevation\_coeff) \times (1.0 - slope \times solar\_slope\_coeff)\Big)_{[0.0, 1.0]}$$

2. **Energy Availability (`calculate_energy_availability`):**
   $$energy\_availability = aggregate \times energy\_availability\_max$$
   where:
   $$norm\_solar = \frac{solar\_exposure}{\max(solar\_exposure\_max, \epsilon)}$$
   $$norm\_biomass = \frac{biomass\_potential}{\max(biomass\_potential\_max, \epsilon)}$$
   $$norm\_nut = \frac{nutrients}{\max(nutrients\_max, \epsilon)}$$
   $$aggregate = \Big(norm\_solar \times energy\_solar\_weight + temperature \times energy\_temp\_weight + norm\_biomass \times energy\_biomass\_weight + norm\_nut \times energy\_nutrient\_weight\Big)_{[0.0, 1.0]}$$

---

## Determinism Analysis

### Input-Derived Determinism
- **No Randomness:** Energy availability generation is a pure deterministic mapping from already-generated substrate chunk data (terrain, climate, resource) and configuration parameters. It does not use or maintain any internal RNG state.
- **Independence:** Generation logic is coordinate-independent and does not mutate shared state, ensuring order-independent execution.

### Stable Iteration Requirements
- Chunk logic reads only cell-local values from components.

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
- How will energy availability interact with future life extraction, and does it require dynamic drawdown and replenishment rate mechanics in Phase 1?
- Should energy availability be one combined field initially, or separate solar, thermal, biomass, and chemical fields?

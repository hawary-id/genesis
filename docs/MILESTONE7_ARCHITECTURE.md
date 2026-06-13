# Milestone 7 Architecture — Simulation Clock & Season State

**Date:** 2026-06-13  
**Status:** Approved and Locked  
**Dependencies:** Milestone 1 ✅, Milestone 2 ✅, Milestone 3 ✅, Milestone 4 ✅, Milestone 5 ✅, Milestone 6 ✅

---

## Architecture Review

### Relevant Roadmap References
- **`TIME_MODEL.md`:** Establishes the tick as the atomic unit of time (1 tick = 1 simulation hour). Day length defaults to 24 ticks, season length to 90 days, and year length to 4 seasons (360 days). Time is strictly physical, not cultural.
- **`PHASE1_WORLD_TECH_SPEC.md`:** Defines `SimulationClock` as the canonical time resource and `SeasonState` as a derived seasonal resource consumed by environmental systems.
- **`PHASE1_IMPLEMENTATION_PLAN.md`:** Defines Milestone 7 as the Simulation Clock implementation phase, where derived calculations (day, season, year) are formalized, and environmental systems are refactored to consume unified time and season state.

### Relevant ADR Constraints
- **[ADR-001: ECS Boundaries](file:///c:/Genesis/docs/adr/ADR-001-ecs-architectural-boundaries.md):** Both `SimulationClock` and `SeasonState` must remain passive, data-only Bevy resources. System logic handles all state progression.
- **[ADR-002: Deterministic Execution Contract](file:///c:/Genesis/docs/adr/ADR-002-deterministic-execution-contract.md):** Updates are strictly driven by ticks. No wall-clock timings or platform-dependent compilation schedulers are permitted.
- **[ADR-004: Physical Time Model](file:///c:/Genesis/docs/adr/ADR-004-physical-time-model.md):** Ticking is monotonic. All cycles run on integer boundaries (e.g. daily, seasonally).

---

## Milestone 7 Scope

### Objectives
Formalize tick progression and unify seasonal tracking. Under ADR-004, `SimulationClock.total_ticks` is established as the sole canonical source of time truth. `SeasonState` acts as a derived, cached resource computed directly from `SimulationClock` and `WorldConfig` (it is not an independent source of truth) for environmental systems to consume.

### Responsibilities
- Increment the simulation clock tick monotonically.
- Calculate and cache the current season index, tick position within the season, normalized progress, and seasonal modifier in `SeasonState` based on elapsed ticks.
- Refactor the Climate, Resource, and Energy systems to read seasonal progress and modifiers from `SeasonState` rather than calculating it independently.
- Provide unit tests validating tick progression, season boundaries, and determinism.

### Explicit Non-Goals
- **No wall-clock or asynchronous ticking:** Ticks are advanced solely by schedule execution (fixed step).
- **No cultural calendars:** No weeks, months, holidays, or civil labels.
- **No agent or life integration:** No biological cycles, aging timers, or circadian behaviors.

---

## ECS Architecture

### Resources

#### `SimulationClock` (Resource, Serialize, Deserialize) [Refined]
*Tracks elapsed ticks and hours.*
- `total_ticks: u32` (elapsed ticks since simulation start)
- `tick_duration_hours: u32` (default `1`)

#### `SeasonState` (Resource, Serialize, Deserialize) [NEW]
*Tracks derived seasonal metrics computed once per tick. Built on a configuration-driven model that makes no Earth-specific or four-season assumptions.*
- `season_index: u32` (index of the current season, from `0` to `seasons_per_year - 1`)
- `tick_in_season: u32` (tick position within the current season)
- `progress: f32` (normalized progress within the current season `[0.0, 1.0]`)
- `seasonal_modifier: f32` (triangular modifier in `[-1.0, 1.0]` used in temperature calculations)

### Components
*None.*

### Events
*None.*

### Systems

#### `advance_simulation_clock` [NEW]
- **Reads:** None.
- **Writes:** `ResMut<SimulationClock>`.
- **Logic:** Increments `total_ticks` by 1.
- **Location:** `time::systems`

#### `update_season_state` [NEW]
- **Reads:** `Res<SimulationClock>`, `Res<WorldConfig>`.
- **Writes:** `ResMut<SeasonState>`.
- **Logic:** Recomputes `SeasonState` based on the new clock tick count.
- **Location:** `time::systems`

#### `update_climate_fields` [Modified]
- **Reads:** `Res<SeasonState>` (replaces calculating the modifier locally), `Res<WorldConfig>`, `Res<SimulationClock>`, `Query<(&ChunkCoord, &TerrainChunk, &mut ClimateChunk)>`.
- **Writes:** `ClimateChunk` (via Query).
- **Location:** `world::climate`

---

## Schedule Integration

### `FixedSimulationTick` Schedule
The systems are sequenced within `FixedSimulationTick` to ensure clock and season updates occur first:

```text
1. advance_simulation_clock [NEW]
2. update_season_state [NEW]   (runs .after(advance_simulation_clock))
3. update_climate_fields      (runs .after(update_season_state))
4. update_resource_fields     (runs .after(update_climate_fields))
5. update_energy_fields       (runs .after(update_resource_fields))
```

---

## Evaluation of `calculate_seasonal_modifier` Ownership

### Alternatives Considered

1. **Keep `calculate_seasonal_modifier` inside `climate.rs`**
   - *Pros:* Keeps changes minimal in the climate module.
   - *Cons:* Seasonal progression is a macro-environmental state. Future systems (e.g. foliage growth, water freezing, migration patterns) will need season info. Coupling this mathematical logic to `climate.rs` forces other modules to import `climate` or implement duplicate math, violating ECS boundary rules.

2. **Migrate into `SeasonState` ownership (Recommended)**
   - *Pros:* Establishes `SeasonState` as the derived cache for seasonal calculations. Downstream systems query the pre-calculated resource `SeasonState` directly rather than invoking helper functions.
   - *Cons:* Slightly larger code reorganization inside the `time` crate.

### Decision & Future Justification
Migrate `calculate_seasonal_modifier` logic into the `SeasonState` resource derivation. This completely decouples seasonal state tracking from climate logic, conforming to the ECS architecture baseline.

Seasonal modifier is intentionally cached despite being derivable from seasonal progress.

It represents a shared environmental signal expected to be consumed by multiple environmental systems. Caching the value avoids duplicate derivation logic across climate, resource, and future environmental subsystems while preserving deterministic behavior because the value remains a pure function of SimulationClock and WorldConfig.

---

## Determinism Analysis

- **Pure State Derivation:** `SeasonState` is computed via a pure function of `SimulationClock.total_ticks` and `WorldConfig`. No random numbers are involved, keeping season progress deterministic on supported targets under the accepted execution contract.
- **Order Independence:** Clock and season updates are registered as single systems scheduled in an explicit sequence before chunk-iterating systems run. Query iteration order inside chunk updates remains isolated.
- **Save/Load Equivalence & Snapshot Strategy:**
  - **Canonical Persistence:** To minimize redundant state, only the canonical `SimulationClock` is persisted in save snapshots.
  - **Reconstruction on Load:** Upon loading, the `SeasonState` is reconstructed dynamically using the loaded `SimulationClock` and the active `WorldConfig`.
  - **Validation Guard:** The reconstruction logic guarantees that `SeasonState == derive(SimulationClock, WorldConfig)`, ensuring identical continuation tick behavior on supported targets under the accepted execution contract.

---

## Validation Requirements
- **Clock Invariants:** `SimulationClock.total_ticks` must be strictly monotonic (asserted in tests and tick systems).
- **Derivation Invariants:** `SeasonState` calculated fields must match configuration definitions. Let $T_s = day\_length \times season\_length\_days$ represent ticks per season:
  - $tick\_in\_season = total\_ticks \pmod{T_s}$
  - $season\_index = (total\_ticks / T_s) \pmod{seasons\_per\_year}$
  - $progress = tick\_in\_season / T_s$

---

## Risks
- **Multiple Time Sources:** Risk of climate or energy systems keeping local counters or reading wall-clock time.
  - *Mitigation:* System dependencies are restricted to reading `SimulationClock` and `SeasonState` resources.
- **Rounding Precision Drift:** Potential differences in `f32` division for progress calculation across platforms.
  - *Mitigation:* Calculations are restricted to simple division of integer values (`u32`), ensuring deterministic behavior on supported targets under the accepted execution contract.

---

## Open Questions

- Should seasonal progression be continuous (changing every tick) or stepped (changing once per day)?
  - *Current recommendation:* Continuous progression provides smooth interpolation of temperature modifiers, avoiding harsh diurnal spikes.
- Do we need to support worlds with asymmetric season lengths (e.g., long winters, short summers) in Phase 1?
  - *Current recommendation:* Keep season length uniform in configuration to satisfy minimal implementation guidelines.

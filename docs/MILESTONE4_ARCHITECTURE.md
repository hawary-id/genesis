# Milestone 4 Architecture — Climate System

**Date:** 2026-06-13  
**Status:** Implemented and Locked  
**Dependencies:** Milestone 1 ✅, Milestone 2 ✅, Milestone 3 ✅  

---

## Objective

Introduce the deterministic baseline climate generation and periodic (daily) climate updates. Climate creates the spatial-temporal abiotic pressure substrate (temperature, moisture, rainfall, sunlight) necessary for future resource and life systems, without introducing a full weather simulator.

---

## Scope

### Included

1. **`ClimateChunk` Component:** Dense cell arrays storing `temperature`, `moisture`, `rainfall`, and `sunlight_factor`. Global `seasonal_modifier` is excluded from chunk memory storage and derived on the fly from the clock.
2. **Climate Config Fields:** Validation ranges and coefficient parameters added to `WorldConfig`.
3. **Module Extension:** Create `world/climate.rs` containing the `ClimateChunk` struct and pure generation/update functions.
4. **Initial Climate Generation:** System added to `StartupGeneration` that reads coordinate metrics and `TerrainChunk` components to generate initial temperature and moisture.
5. **Periodic Climate updates:** System added to `FixedSimulationTick` that reads `TerrainChunk` and recalculates climate arrays once per simulation day (e.g., `clock.total_ticks % config.day_length_ticks == 0`).
6. **Post-Generation Range Validation:** Replaces `validate_generated_terrain` with `validate_generated_world` to check both terrain and climate fields.
7. **Simulation Ticking Tests:** Verifies that climate updates run deterministically across ticks and match mathematical expectations.

### Excluded

1. **Atmospheric weather simulation:** No wind velocity vectors, barometric pressure, cloud formations, humidity layers, or storms.
2. **Cross-chunk heat/moisture transfer:** All climate updates are derived local to the cell or chunk coordinates, avoiding iteration-order dependencies and boundary synchronization issues.
3. **`SeasonState` Mutator:** Seasonal modifiers are calculated as pure derived functions of the `SimulationClock.total_ticks` instead of introducing the mutable `SeasonState` resource.
4. **`App::tick()` loop:** Execution of the tick schedule is triggered explicitly in unit/integration tests; headless looping/tick-rate pacing is deferred.

---

## ECS Impact

### New Components

| Component | Fields | Module |
|---|---|---|
| `ClimateChunk` | `temperature: Vec<f32>`, `moisture: Vec<f32>`, `rainfall: Vec<f32>`, `sunlight_factor: Vec<f32>` | `world::climate` |

*Note: All fields are contiguous `Vec<f32>` of length `chunk_size * chunk_size` indexed in row-major order. `seasonal_modifier` is derived dynamically and not stored on the chunk.*

### Resources Modified

`WorldConfig` gains validation ranges and cycle parameters:

```rust
// Climate validation bounds
pub temperature_min: f32,
pub temperature_max: f32,
pub moisture_min: f32,
pub moisture_max: f32,
pub rainfall_min: f32,
pub rainfall_max: f32,
pub sunlight_factor_min: f32,
pub sunlight_factor_max: f32,

// Climate generation settings
pub sea_level_temperature_base: f32, // baseline temperature at sea level
pub temperature_lapse_rate: f32,     // temperature drop rate per unit elevation
pub seasonal_temperature_amplitude: f32, // seasonal temperature variance
```

### Schedules & Systems

#### `StartupGeneration` Update
```text
1. validate_world_config
2. spawn_chunk_entities
3. generate_terrain_chunks
4. generate_climate_chunks [NEW - reads: ChunkCoord, TerrainChunk, WorldConfig, WorldSeed; writes: ClimateChunk]
5. validate_generated_world [NEW - replaces validate_generated_terrain - reads: ChunkCoord, TerrainChunk, ClimateChunk, WorldConfig; writes: none]
6. mark_chunks_generated
7. emit_world_generation_completed
```

*Ordering:* `generate_climate_chunks` runs `.after(generate_terrain_chunks)`. `validate_generated_world` runs `.after(generate_climate_chunks)`.

#### `FixedSimulationTick` Update
```text
1. update_climate_fields [NEW - reads: ChunkCoord, TerrainChunk, SimulationClock, WorldConfig; writes: ClimateChunk]
```

---

## Algorithms

### Deterministic Seasonal Modifier
To fulfill the determinism contract (ADR-002) and avoid transcendental floating-point drift (`cos`/`sin`), the seasonal cycle is calculated using a deterministic triangle wave oscillating between `-1.0` and `1.0` over the course of a year:

```rust
let year_length_ticks = config.day_length_ticks * config.season_length_days * config.seasons_per_year;
let progress = (total_ticks % year_length_ticks) as f32 / year_length_ticks as f32;

// Triangle wave calculation
let seasonal_modifier = if progress < 0.25 {
    4.0 * progress
} else if progress < 0.75 {
    2.0 - 4.0 * progress
} else {
    4.0 * progress - 4.0
};
```

### Temperature
Determined by latitude (y coordinate), elevation, and seasonal progress:
1. **Latitude/Sunlight factor:** Linear gradient from pole (north/top, value `0.0`) to equator (south/bottom, value `1.0`).
   `sunlight_factor = y as f32 / world_height as f32`
2. **Base Temperature:** 
   `base_temp = sea_level_temp - (elevation * lapse_rate) + (sunlight_factor * range)`
3. **Seasonal Temperature:**
   `temp = base_temp + seasonal_modifier * seasonal_temp_amplitude`
4. Clamped to `[temperature_min, temperature_max]`.

### Moisture
Varies with elevation and water depth:
- `moisture = (water_depth / sea_level) * 0.8 + (1.0 - elevation) * 0.2`
- Clamped to `[moisture_min, moisture_max]`.

### Rainfall
Derived from local humidity/moisture and temperature:
- `rainfall = moisture * temperature` (clamped to `[rainfall_min, rainfall_max]`).

---

## Dependencies

- **Milestone 3 spatial types:** Relies on coordinate conversions (`WorldCoord` to global `y` index) and `TerrainChunk` values (`elevation`, `water_depth`).
- **Milestone 2 SimulationClock:** Relies on `SimulationClock.total_ticks` to schedule periodic calculations.

---

## Risks

1. **Unintentional Meteorological Complexity:** Risk of adding wind patterns, moisture transport across cell borders, or humidity cycles.
   - *Mitigation:* Explicitly ban atmospheric updates and cross-cell physics. Maintain cell-local updates.
2. **Frequency Churn:** Re-evaluating climate arrays every hour increases overhead without adding value.
   - *Mitigation:* Restrict system logic to run only on daily boundaries: `total_ticks % config.day_length_ticks == 0`.
3. **Transcendental math drift:** Cross-platform compiler differences on trigonometric operations can break replayability.
   - *Mitigation:* Triangle-wave cycle replaces trigonometric equations for seasonal modeling.

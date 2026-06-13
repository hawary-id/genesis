# Milestone 3 Architecture — World Generation

**Date:** 2026-06-13
**Status:** Approved
**Dependencies:** Milestone 1 ✅, Milestone 2 ✅

---

## Objective

Generate the initial deterministic terrain substrate and chunk layout.

The world can be generated from a seed. The same seed and configuration always produce the same world. All generated fields are within configured valid ranges.

---

## Scope

### Included

- World configuration validation before generation begins.
- Domain seed derivation (`terrain_seed` from `WorldSeed.root_seed`).
- `world/` module introduction.
- `WorldCoord`, `ChunkCoord` (as ECS component), `LocalCoord` types with conversion functions.
- Chunk entity spawning — one entity per chunk position.
- `TerrainChunk` component — elevation, slope, water_depth, soil_depth, soil_fertility.
- Terrain generation systems added to `StartupGeneration`.
- `Generated` marker component.
- Post-generation terrain range validation.
- `WorldGenerationCompleted` event registration and emission.
- `App::run_startup()` method.
- Terrain validation range fields added to `WorldConfig`.
- Required unit and integration tests.

### Excluded

All of the following are deferred to their designated milestones:

- `ClimateChunk`, `ResourceChunk`, `EnergyAvailabilityChunk` (Milestones 4–6)
- `DirtyChunk` (Milestone 4 — no mutation after generation yet)
- `SeasonState`, `GenerationState`, `SnapshotConfig` (Milestones 7, deferred, 9)
- Climate, resource, and energy generation systems (Milestones 4–6)
- `PostTickValidation` systems (Milestone 8)
- `PersistenceBoundary` systems (Milestone 9)
- `SnapshotRequested`, `SnapshotCompleted` events (Milestone 9)
- `App::tick()` or simulation loop
- `FixedSimulationTick` systems
- Any noise library dependency — hash-based RNG only

---

## Decisions

### Terrain Generation Algorithm

**Decision:** Hash-based value noise using existing `rand_chacha`. No new dependency.

Each chunk is seeded independently from `(terrain_seed, chunk_x, chunk_y)` using wrapping integer arithmetic. `ChaCha8Rng::seed_from_u64(chunk_seed)` is used per chunk. Spatial coherence is achieved through interpolation over a coarse lattice. Generation is a pure function with full determinism control. ECS query iteration order is irrelevant because no RNG state is shared across chunks.

### GenerationState

**Decision:** Deferred. `WorldGenerationCompleted` event is sufficient for Milestone 3. `GenerationState` will be introduced when `FixedSimulationTick` needs to be explicitly gated.

### Slope Storage

**Decision:** Stored in `TerrainChunk`. Computed during terrain generation using in-chunk neighbors only. Chunk-edge cells use the boundary approximation (edge cells treat the world or chunk edge as equal elevation). This is acceptable — Genesis is not a geology simulator.

### Water Representation

**Decision:** `water_depth: f32` per cell. A value of `0.0` means dry. Presence is derivable from `water_depth > 0.0`. This avoids retrofitting a second field later and aligns with the "measurable fields before labels" principle.

### TerrainChunk Field Layout

**Decision:** One `Vec<f32>` per field. Cell index: `local_y * chunk_size + local_x`. This is cache-friendly for single-field processing passes (which is how climate, resource, and energy systems will access terrain data in later milestones).

### Terrain Validation Ranges

**Decision:** Added as flat fields directly to `WorldConfig`. These are world parameters. No sub-struct.

---

## Module Structure

```
engine/src/
  app/
    mod.rs          — ADD run_startup(), register Events<WorldGenerationCompleted>
    events.rs       — (unchanged)
    plugins.rs      — (unchanged)
    schedules.rs    — (unchanged)
  config/
    world_config.rs — ADD terrain validation range fields
    world_bounds.rs — (unchanged)
    mod.rs          — (unchanged)
  rng/
    seed.rs         — ADD TERRAIN_DOMAIN_SALT constant + derive_terrain_seed()
    mod.rs          — (unchanged)
  world/            — NEW
    mod.rs
    coord.rs        — WorldCoord, ChunkCoord (Component), LocalCoord, conversions
    terrain.rs      — TerrainChunk (Component), terrain generation pure functions
    generation.rs   — systems: validate_world_config, spawn_chunk_entities,
                      generate_terrain_chunks, validate_generated_terrain,
                      mark_chunks_generated, emit_world_generation_completed
  testing/          — (unchanged)
  time/             — (unchanged)
  lib.rs            — ADD pub mod world
  main.rs           — CALL App::run_startup() after App::new()
```

---

## ECS Types

### New Components

| Component | Fields | Module |
|---|---|---|
| `ChunkCoord` | `x: u32, y: u32` | `world::coord` |
| `TerrainChunk` | `elevation: Vec<f32>`, `slope: Vec<f32>`, `water_depth: Vec<f32>`, `soil_depth: Vec<f32>`, `soil_fertility: Vec<f32>` | `world::terrain` |
| `Generated` | marker (no fields) | `world::generation` |

### Coordinate Types (not ECS components)

| Type | Fields | Module |
|---|---|---|
| `WorldCoord` | `x: u32, y: u32` | `world::coord` |
| `LocalCoord` | `x: u32, y: u32` | `world::coord` |

### Resources Modified

`WorldConfig` — new terrain validation range fields:

```
elevation_min: f32      // default: 0.0
elevation_max: f32      // default: 1.0
slope_max: f32          // default: 1.0
water_depth_max: f32    // default: 1.0
soil_depth_max: f32     // default: 1.0
soil_fertility_max: f32 // default: 1.0
```

### Events

`WorldGenerationCompleted` — defined in Milestone 2.

Milestone 3 adds:
- Registration: `world.init_resource::<Events<WorldGenerationCompleted>>()` in `App::new()`
- Emission: `emit_world_generation_completed` system as the final system in `StartupGeneration`

### New App Method

```rust
pub fn run_startup(&mut self) {
    self.world.run_schedule(StartupGeneration);
}
```

---

## Systems in StartupGeneration

Explicit ordering. No implicit ordering dependencies.

| # | System | Reads | Writes |
|---|---|---|---|
| 1 | `validate_world_config` | `WorldConfig` | — |
| 2 | `spawn_chunk_entities` | `WorldBounds` | Chunk entities + `ChunkCoord` |
| 3 | `generate_terrain_chunks` | `WorldConfig`, `WorldSeed`, chunk entities with `ChunkCoord` | `TerrainChunk` on each chunk |
| 4 | `validate_generated_terrain` | `WorldConfig`, chunk entities with `TerrainChunk` | — |
| 5 | `mark_chunks_generated` | Chunk entities with `TerrainChunk` | `Generated` on each chunk |
| 6 | `emit_world_generation_completed` | — | `Events<WorldGenerationCompleted>` |

Systems 1–6 use `.after()` ordering constraints. Every system depends on the one before it.

---

## Terrain Generation Algorithm

### Seed Derivation

```
TERRAIN_DOMAIN_SALT: u64 = 0x74657272_61696e00   // "terrain\0" in hex
terrain_seed = root_seed.wrapping_add(TERRAIN_DOMAIN_SALT)
```

### Per-Chunk Seed

```
chunk_seed = terrain_seed
    .wrapping_add(chunk_x as u64 * 0x9e3779b97f4a7c15)
    .wrapping_add(chunk_y as u64 * 0x6c62272e07bb0142)
```

Uses well-known multiplicative hash constants (Fibonacci hashing and FNV prime). Pure integer arithmetic. Deterministic. No iteration-order dependency.

### Elevation Generation

Value noise approach:

1. The world is divided into a coarse grid of `noise_scale` spacing (e.g., every 8 cells).
2. For each coarse grid point, a height value is generated from its seeded RNG.
3. Each cell's elevation is computed by bilinear interpolation of its four surrounding coarse grid values.
4. Optional: sum multiple octaves at different scales for more complex terrain (fractal-like).

All operations are additions, multiplications, and linear interpolation — no transcendental functions in the critical path.

### Slope Computation

Computed from elevation after all cells in the chunk are generated.

For cell `(lx, ly)`:
- `dx = elevation[lx+1, ly] - elevation[lx-1, ly]` (clamped to available neighbors)
- `dy = elevation[lx, ly+1] - elevation[lx, ly-1]` (clamped to available neighbors)
- `slope = sqrt(dx*dx + dy*dy) / 2.0` (normalized by neighbor distance)
- Clamped to `[0.0, config.slope_max]`

Boundary cells: missing neighbors are treated as same elevation (zero gradient). This is documented as a Phase 1 approximation.

### Water Depth

Simple threshold: cells below a configured `sea_level` elevation get water depth proportional to how far below sea level they are. Above sea level: `0.0`.

### Soil Fields

- `soil_depth`: inversely proportional to slope (steep terrain has thin soil). Clamped to configured range.
- `soil_fertility`: proportional to soil depth and water proximity. Clamped to configured range.

---

## Determinism Constraints

1. Each chunk is seeded from `(terrain_seed, chunk_x, chunk_y)` only. No shared mutable RNG across chunks.
2. Slope computed in-chunk. Two-pass: all elevations first, then all slopes. No cross-chunk dependency.
3. Systems are ordered explicitly with `.after()`. No implicit schedule ordering.
4. Terrain generation is a pure function of seed and coordinates.
5. No `thread_rng()`, `rand::random()`, or `SystemTime::now()` anywhere in simulation code.
6. `f32` formulas are kept to simple arithmetic. No `sin`, `cos`, or `pow` in the generation hot path.

---

## Required Tests

### Determinism
- Same seed + config → identical `TerrainChunk` arrays across two separate `App` instances
- Different seeds → measurably different elevation distributions

### Chunk Counting
- After `run_startup()`, chunk count == `bounds.chunks_x * bounds.chunks_y`
- Every chunk entity has exactly one `ChunkCoord`, one `TerrainChunk`, one `Generated`

### Coordinate Round-Trips
- `chunk_local_to_world(world_to_chunk(wc, s), world_to_local(wc, s), s) == wc` for all valid `wc`
- Coordinate conversion is purely integer arithmetic — no precision loss

### Terrain Range Validity
- All fields on all chunks fall within configured ranges after `run_startup()`

### Event Emission
- `Events<WorldGenerationCompleted>` contains exactly one event after `run_startup()`

### Config Validation
- `validate_world_config` rejects: chunk_size 0, world dimensions not divisible by chunk_size, zero day_length_ticks

---

## Success Criteria

- Same seed and configuration produce identical terrain chunks.
- Generated chunk count matches world dimensions and chunk size.
- Coordinate conversions round-trip correctly.
- All generated chunks have `ChunkCoord`, `TerrainChunk`, and `Generated`.
- All terrain field values are within configured ranges.
- `WorldGenerationCompleted` is emitted exactly once.
- All tests pass. `cargo check` and `cargo fmt` are clean.

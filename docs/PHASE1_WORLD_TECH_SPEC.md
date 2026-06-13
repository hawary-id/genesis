# Phase 1 World Technical Specification

## Status

This document freezes the Phase 1 World architecture before Rust implementation begins.

It consolidates decisions from:

- `VISION.md`
- `PRINCIPLES.md`
- `ROADMAP.md`
- `ARCHITECTURE.md`
- `RESEARCH.md`
- `PHASE_1_WORLD.md`
- `DETERMINISM.md`
- `ECS_GUIDELINES.md`
- `SPATIAL_MODEL.md`
- `TIME_MODEL.md`

Phase 1 builds the deterministic environmental substrate only. It does not build life, agents, culture, economy, politics, or civilization.

## World Dimensions

Recommended default: `512 x 512` cells.

### Evaluated Options

#### `256 x 256`

Total cells: `65,536`.

Rationale:

- Very fast to generate, validate, snapshot, and test.
- Useful for unit tests and small deterministic fixtures.
- Low memory and persistence cost.

Tradeoffs:

- Too small for meaningful large-scale climate gradients.
- Limited room for resource scarcity patterns.
- Later movement, migration, and population pressure may hit world edges too quickly.

Scalability path:

- Keep as a test fixture size.
- Use for deterministic CI tests and debugging.

#### `512 x 512`

Total cells: `262,144`.

Rationale:

- Large enough for visible terrain, climate, resource, and energy gradients.
- Small enough for early full-world validation and snapshot tests.
- Works cleanly with `32 x 32` chunks, producing `16 x 16 = 256` chunks.

Tradeoffs:

- More expensive than `256 x 256` for rapid test cycles.
- Still too small for truly continental simulation.
- Some large-scale migration or ecological effects may require larger worlds later.

Scalability path:

- Treat as the default development and first playable research size.
- Add larger configurations only after chunk update, validation, and persistence costs are measured.

#### `1024 x 1024`

Total cells: `1,048,576`.

Rationale:

- Better for large-scale gradients and future population spread.
- More realistic for long-running worlds.
- Provides room for regional differentiation.

Tradeoffs:

- Too large for the first implementation default.
- Slower deterministic tests and save/load equivalence checks.
- More likely to push early work toward performance before model correctness is proven.

Scalability path:

- Target as the first large-world milestone after Phase 1 correctness is stable.
- Requires profiling, chunk-level dirty tracking, and careful persistence batching.

### Decision

Use `512 x 512` cells as the Phase 1 default.

Use `256 x 256` for fast tests.

Reserve `1024 x 1024` for scalability validation after the initial implementation is correct.

## Chunk Architecture

Recommended default chunk size: `32 x 32` cells.

With a `512 x 512` world, this produces `16 x 16 = 256` chunks.

### Evaluated Options

#### `16 x 16`

Cells per chunk: `256`.

Rationale:

- Fine-grained dirty tracking.
- Smaller snapshot units.
- More granular future streaming.

Tradeoffs:

- Produces `32 x 32 = 1,024` chunks in a `512 x 512` world.
- More ECS entities and scheduling overhead.
- More chunk boundary handling.

#### `32 x 32`

Cells per chunk: `1,024`.

Rationale:

- Balanced chunk count and chunk-local data size.
- Produces a manageable number of chunk entities.
- Large enough for efficient contiguous field storage.
- Small enough for targeted validation and dirty tracking.

Tradeoffs:

- Coarser dirty tracking than `16 x 16`.
- More data per chunk must be loaded or saved together.
- Some future streaming systems may want smaller chunks.

#### `64 x 64`

Cells per chunk: `4,096`.

Rationale:

- Produces only `8 x 8 = 64` chunks in a `512 x 512` world.
- Lower ECS entity overhead.
- Efficient for full-world batch updates.

Tradeoffs:

- Dirty tracking is coarse.
- Snapshot and validation units are larger.
- Less flexible for future local updates or streaming.

### Decision

Use `32 x 32` cells per chunk.

Chunk dimensions must divide the default world dimensions exactly. Non-divisible world dimensions are out of scope for the first implementation.

## Coordinate Types

Phase 1 uses a chunked square grid with non-negative coordinates.

### `WorldCoord`

Purpose:

- Identifies a cell in global world space.

Expected fields:

- `x`
- `y`

Validity:

- `0 <= x < world_width`
- `0 <= y < world_height`

Ownership:

- Used by world query APIs, validation, generation, and future perception boundaries.

### `ChunkCoord`

Purpose:

- Identifies a chunk in global chunk space.

Expected fields:

- `x`
- `y`

Validity:

- `0 <= x < chunks_wide`
- `0 <= y < chunks_high`

Ownership:

- Stored as a component on each chunk entity.

### `LocalCoord`

Purpose:

- Identifies a cell inside one chunk.

Expected fields:

- `x`
- `y`

Validity:

- `0 <= x < chunk_width`
- `0 <= y < chunk_height`

Ownership:

- Used internally by chunk field accessors and validation.

### Conversion Rules

Given:

- `chunk_size = 32`
- `WorldCoord(wx, wy)`

Conversion to chunk and local coordinates:

- `ChunkCoord.x = wx / chunk_size`
- `ChunkCoord.y = wy / chunk_size`
- `LocalCoord.x = wx % chunk_size`
- `LocalCoord.y = wy % chunk_size`

Conversion back to world coordinates:

- `WorldCoord.x = ChunkCoord.x * chunk_size + LocalCoord.x`
- `WorldCoord.y = ChunkCoord.y * chunk_size + LocalCoord.y`

Rules:

- Conversions must be deterministic integer operations.
- Negative coordinates are not supported in Phase 1.
- Coordinates outside bounds are validation errors.
- Future infinite or streaming worlds must introduce a new coordinate policy instead of weakening these rules silently.

## Core Components

Chunk entities are the only required world entities in Phase 1.

Cells are data inside chunk components, not ECS entities.

### `ChunkCoord`

Purpose:

- Gives each chunk entity a stable spatial identity.

Ownership:

- Owned by the chunk entity.

Expected data:

- Chunk-space `x`.
- Chunk-space `y`.

### `TerrainChunk`

Purpose:

- Stores terrain fields for all cells in a chunk.

Ownership:

- Owned by the chunk entity.

Expected data:

- Elevation field.
- Slope field, either stored or derived during generation.
- Water presence or water depth field.
- Soil depth field.
- Soil fertility field.

### `ClimateChunk`

Purpose:

- Stores climate fields for all cells in a chunk.

Ownership:

- Owned by the chunk entity.

Expected data:

- Temperature field.
- Moisture field.
- Rainfall field.
- Seasonal modifier field or derived seasonal influence cache.
- Sunlight or latitude factor field.

### `ResourceChunk`

Purpose:

- Stores material environmental availability for all cells in a chunk.

Ownership:

- Owned by the chunk entity.

Expected data:

- Fresh water availability.
- Nutrient availability.
- Mineral concentration.
- Biomass carrying potential.

Notes:

- These are not economic goods.
- Ownership, price, labor, production, and exchange do not exist in Phase 1.

### `EnergyAvailabilityChunk`

Purpose:

- Stores usable environmental energy potential for all cells in a chunk.

Ownership:

- Owned by the chunk entity.

Expected data:

- Solar exposure potential.
- Thermal gradient potential if modeled.
- Biomass energy potential if derived from climate/resources.
- Chemical or mineral energy potential if modeled.

Notes:

- This is not an agent energy meter.
- Biological metabolism, hunger, calories, labor, fuel stockpiles, and industry are not Phase 1 concepts.
- Energy availability is first-class because life, ecology, technology, and economy ultimately depend on usable energy gradients.

### `DirtyChunk`

Purpose:

- Marks a chunk whose state changed since the last persistence or observation boundary.

Ownership:

- Owned by the chunk entity as a marker component.

Expected data:

- Marker only, unless implementation later needs a compact dirty reason.

### `Generated`

Purpose:

- Marks a chunk that completed initial generation.

Ownership:

- Owned by the chunk entity as a marker component.

Expected data:

- Marker only.

## Core Resources

### `WorldConfig`

Purpose:

- Stores immutable world configuration for a simulation run.

Ownership:

- Owned by the ECS world as a resource.

Mutability expectations:

- Immutable after startup generation begins.

Expected data:

- World width and height.
- Chunk width and height.
- Time configuration.
- Generation version.
- Climate configuration.
- Resource configuration.
- Energy availability configuration.
- Validation ranges.

### `WorldSeed`

Purpose:

- Stores the root deterministic seed.

Ownership:

- Owned by the ECS world as a resource.

Mutability expectations:

- Immutable after creation.

Expected data:

- Root seed.
- Named deterministic seed derivation policy for terrain, climate, resources, and energy availability.

### `SimulationClock`

Purpose:

- Stores canonical simulation time.

Ownership:

- Owned by the ECS world as a resource.

Mutability expectations:

- Mutated only by the clock advancement system in `FixedSimulationTick`.

Expected data:

- Total elapsed ticks.
- Tick duration in simulation hours.

### `WorldBounds`

Purpose:

- Stores validated bounds for world and chunk coordinates.

Ownership:

- Owned by the ECS world as a resource.

Mutability expectations:

- Immutable after startup.

Expected data:

- World width.
- World height.
- Chunk count in x.
- Chunk count in y.
- Chunk size.

### `SeasonState`

Purpose:

- Stores current derived seasonal phase for systems that need it.

Ownership:

- Owned by the ECS world as a resource.

Mutability expectations:

- Mutated only by the season update system.
- Must be derivable from `SimulationClock` and `WorldConfig` for validation.

Expected data:

- Current season index.
- Tick position within season.
- Normalized seasonal progress.
- Current seasonal modifiers.

### `GenerationState`

Purpose:

- Tracks startup generation progress and completion.

Ownership:

- Owned by the ECS world as a resource.

Mutability expectations:

- Mutable only during `StartupGeneration`.
- Should become complete before fixed ticking starts.

Expected data:

- Current generation stage.
- Completion flag.
- Generation version.

### `SnapshotConfig`

Purpose:

- Defines snapshot behavior at persistence boundaries.

Ownership:

- Owned by the ECS world as a resource.

Mutability expectations:

- Usually immutable during a run.
- May be changed by external control only between ticks.

Expected data:

- Snapshot interval in ticks.
- Snapshot target policy.
- Snapshot schema version.

## Core Events

Only the following events are required for Phase 1.

### `WorldGenerationCompleted`

Purpose:

- Signals that startup generation completed and the world passed initial validation.

Required because:

- Fixed ticking must not begin against a partially generated world.

### `SnapshotRequested`

Purpose:

- Requests a snapshot at the next persistence boundary.

Required because:

- Persistence must remain outside core simulation updates.

### `SnapshotCompleted`

Purpose:

- Reports that a snapshot finished successfully.

Required because:

- Observation and test harnesses need a boundary signal for save/load workflows.

Not required in Phase 1:

- `SeasonChanged`
- `ChunkUpdated`
- Weather events
- Resource discovery events
- History events
- Agent events

Season changes can be derived from clock and configuration. Chunk updates can be tracked with `DirtyChunk`.

## Schedules

Phase 1 uses explicit schedules. Systems listed here are conceptual system names, not implementation code.

## `StartupGeneration`

Purpose:

- Build the initial deterministic world from configuration and seed.

Execution order:

1. `load_world_config`
2. `validate_world_config`
3. `initialize_world_seed`
4. `derive_world_bounds`
5. `initialize_simulation_clock`
6. `initialize_generation_state`
7. `spawn_chunk_entities`
8. `generate_terrain_chunks`
9. `generate_climate_chunks`
10. `generate_resource_chunks`
11. `generate_energy_availability_chunks`
12. `initialize_season_state`
13. `mark_chunks_generated`
14. `validate_generated_world`
15. `emit_world_generation_completed`

Systems contained:

- Configuration loading and validation systems.
- Seed initialization systems.
- Chunk spawning systems.
- Terrain generation systems.
- Climate generation systems.
- Resource generation systems.
- Energy availability generation systems.
- Initial validation systems.

## `FixedSimulationTick`

Purpose:

- Advance the world by exactly one deterministic simulation tick.

Execution order:

1. `advance_simulation_clock`
2. `update_season_state`
3. `update_climate_fields`
4. `update_resource_fields`
5. `update_energy_availability_fields`
6. `mark_dirty_chunks`

Systems contained:

- Clock advancement system.
- Season update system.
- Climate update systems.
- Resource update systems.
- Energy availability update systems.
- Dirty tracking systems.

Notes:

- No database writes occur in this schedule.
- No observation export occurs in this schedule.
- No history or event archive is written in this schedule.

## `PostTickValidation`

Purpose:

- Detect invalid world state immediately after deterministic mutation.

Execution order:

1. `validate_clock_monotonicity`
2. `validate_chunk_coordinates`
3. `validate_chunk_dimensions`
4. `validate_terrain_ranges`
5. `validate_climate_ranges`
6. `validate_resource_ranges`
7. `validate_energy_availability_ranges`
8. `validate_season_state`

Systems contained:

- Invariant validation systems.

Notes:

- Debug and test builds should run full validation.
- Release builds may support reduced validation only after correctness is established.

## `PersistenceBoundary`

Purpose:

- Save stable world state without affecting simulation outcomes.

Execution order:

1. `detect_snapshot_due`
2. `handle_snapshot_requests`
3. `build_world_snapshot`
4. `write_world_snapshot`
5. `emit_snapshot_completed`
6. `clear_persisted_dirty_markers`

Systems contained:

- Snapshot scheduling systems.
- Snapshot construction systems.
- Snapshot write systems.
- Dirty marker cleanup systems.

Notes:

- Persistence is a boundary concern.
- Persistence failure must not mutate simulation state.
- PostgreSQL integration, if used, belongs behind this boundary.

## `ObservationBoundary`

Purpose:

- Produce read-only summaries for debugging, metrics, and future dashboard/API layers.

Execution order:

1. `collect_world_metrics`
2. `collect_generation_metrics`
3. `collect_validation_metrics`
4. `export_observation_snapshot`

Systems contained:

- Metrics systems.
- Debug summary systems.
- Read-model export systems.

Notes:

- Observation must not mutate simulation state.
- The dashboard must observe the world, not shape it.

## World Generation Pipeline

Exact generation order:

1. Seed initialization.
2. Terrain generation.
3. Climate generation.
4. Resource generation.
5. Energy availability generation.
6. Validation.

### Rationale For Ordering

Seed initialization comes first because every generated field must be reproducible.

Terrain comes before climate because elevation, slope, water presence, and soil influence temperature, moisture, rainfall, and sunlight exposure.

Climate comes before resources because resource availability depends on temperature, moisture, rainfall, and seasonal baseline conditions.

Resource generation comes before energy availability because biomass potential and material availability may affect usable environmental energy potential.

Energy availability comes after terrain, climate, and resources because it can be derived from solar exposure, temperature gradients, biomass potential, and chemical or mineral context.

Validation comes last because the full generated world must be checked as an integrated substrate.

## Climate Scope

Genesis is not a weather simulator.

Phase 1 climate exists to create deterministic environmental pressure. It should not attempt detailed meteorology.

### Included Climate Features

Phase 1 includes:

- Temperature field.
- Moisture field.
- Rainfall field.
- Seasonal modifiers.
- Sunlight or latitude factor.
- Deterministic climate updates at fixed tick or lower-frequency intervals.

### Excluded Climate Features

Phase 1 excludes:

- Storms.
- Atmospheric simulation.
- Fluid dynamics.
- Wind simulation beyond optional static exposure fields.
- Pressure systems.
- Humidity layers.
- Cloud simulation.
- Detailed erosion.
- River formation simulation.
- Plate tectonics.
- Ocean currents.
- Fire weather.
- Per-cell real-time weather events.

### Design Rule

Add climate realism only when it creates meaningful pressure for emergence.

Do not add climate systems solely because they are realistic.

## Persistence Scope

Persistence must remain minimal and boundary-oriented.

The purpose of Phase 1 persistence is:

- Reproducibility.
- Resume.
- Save/load equivalence testing.
- Debug inspection.

It is not a history system.

### Required

Persist exactly enough state to resume deterministic simulation:

- Configuration.
- Seed.
- Generation version.
- Simulation clock.
- Chunk state.
- Climate state.
- Resource state.
- Energy availability state.
- Terrain state.
- Snapshot schema version.

### Not Required

Phase 1 does not require:

- History.
- Event sourcing.
- Civilization archives.
- Per-tick logs.
- Social records.
- Economic records.
- Political records.
- Cultural records.
- Agent memories.
- Append-only world event streams.
- Database-first simulation state.

### Boundary Rule

Simulation systems update ECS state.

Persistence systems observe stable ECS state at `PersistenceBoundary`.

Persistence I/O must not affect the result of a simulation tick.

## Time Configuration

Default recommendation:

- `1 tick = 1 simulation hour`.

The tick duration remains the base recommendation because it balances long-running simulation with daily and seasonal environmental change.

### Configurable World Parameters

The following must be world configuration parameters, not hardcoded engine constants:

- Day length in ticks.
- Season length in days.
- Seasons per year.
- Year length derived from day length, season length, and season count.

Recommended Phase 1 defaults:

- `1 tick = 1 simulation hour`
- `1 day = 24 ticks`
- `1 season = 90 days`
- `1 year = 4 seasons = 360 days = 8,640 ticks`

### Rationale

Configurability is important because Genesis is a research engine, not a fixed Earth clone.

Future worlds may need:

- Longer or shorter days.
- Different seasonal cadence.
- No seasons.
- More than four seasons.
- Non-Earth-like environmental cycles.

The engine should support world diversity without hardcoding culture or calendar assumptions.

### Rule

The canonical stored time is total elapsed ticks.

Day, season, and year values are derived from `SimulationClock` and `WorldConfig`.

Cultural calendars, weeks, holidays, eras, rituals, and work schedules are non-goals.

## Validation Rules

Phase 1 invariants are mandatory.

### Coordinate Validity

- Every `WorldCoord` must be inside world bounds.
- Every `ChunkCoord` must be inside chunk bounds.
- Every `LocalCoord` must be inside chunk-local bounds.
- World-to-chunk conversion must round-trip correctly.

### Chunk Consistency

- Every chunk entity must have exactly one `ChunkCoord`.
- Every generated chunk must have terrain, climate, resource, and energy availability components.
- Chunk field arrays must match configured chunk dimensions.
- Number of generated chunks must match world dimensions and chunk size.

### Terrain Ranges

- Elevation values must remain within configured range.
- Slope values must remain within configured range.
- Water values must remain within configured range.
- Soil depth and soil fertility must remain within configured ranges.

### Resource Ranges

- Resource quantities must be non-negative.
- Fresh water availability must be non-negative.
- Nutrient availability must be non-negative.
- Mineral concentration must be non-negative.
- Biomass carrying potential must be non-negative.

### Climate Ranges

- Temperature must remain within configured range.
- Moisture must remain within configured range.
- Rainfall must remain within configured range.
- Seasonal modifiers must remain within configured range.
- Sunlight or latitude factors must remain within configured range.

### Energy Ranges

- Energy availability must be non-negative.
- Solar exposure potential must remain within configured range.
- Thermal, biomass, chemical, or mineral energy potential must remain within configured ranges if present.

### Time Validity

- Simulation tick must be monotonic.
- Tick count must never move backward.
- Derived day, season, and year values must match `WorldConfig`.
- `SeasonState` must be derivable from `SimulationClock` and `WorldConfig`.

## Testing Requirements

Implementation is not complete until these tests exist and pass.

### Deterministic Generation

Required tests:

- Same seed and configuration produce identical terrain, climate, resources, and energy availability.
- Different seeds produce different worlds.
- Generation order is stable.
- Generated chunks all pass invariants.

### Deterministic Ticking

Required tests:

- Running the same generated world for the same tick count produces identical final state.
- Clock advances by exactly one tick per `FixedSimulationTick`.
- Season state changes only according to configured time parameters.
- Climate, resources, and energy availability update deterministically.

### Save/Load Equivalence

Required tests:

- Running `N` ticks continuously equals running `A` ticks, saving, loading, then running `B` ticks where `A + B = N`.
- Snapshot includes all state needed for deterministic continuation.
- Persistence does not mutate simulation state.

### Invariant Validation

Required tests:

- Invalid coordinates fail validation.
- Invalid chunk dimensions fail validation.
- Negative resources fail validation.
- Invalid climate ranges fail validation.
- Invalid energy ranges fail validation.
- Non-monotonic clock state fails validation.

### Long-Run Stability

Required tests:

- A default `512 x 512` world can run for a long deterministic interval without invariant failure.
- No resource or energy field drifts into invalid ranges.
- Save/load remains valid after long runs.

Recommended first long-run target:

- `10` simulation years using default time configuration.

## Non Goals

Phase 1 must not implement:

- Agents.
- AI.
- Pathfinding.
- Movement.
- Perception.
- Hunger.
- Biological energy budgets.
- Aging.
- Death.
- Genetics.
- Evolution.
- Memory.
- Decision-making.
- Learning.
- Knowledge.
- Society.
- Cooperation.
- Conflict.
- Culture.
- Language.
- Economy.
- Trade.
- Production.
- Technology.
- Politics.
- Governance.
- Diplomacy.
- Civilization.
- Cities.
- Institutions.
- Religion.
- Cultural calendars.
- History archives.
- Weather simulation.
- Storm systems.
- Detailed erosion.
- Fluid dynamics.
- Continuous world physics.

## Final Architecture Review

### Major Risks

- The world system may expand beyond substrate concerns and begin modeling life or civilization too early.
- Climate work may become weather simulation instead of emergence-focused pressure modeling.
- Persistence may grow into event sourcing or historical archiving before agents exist.
- ECS resources may become god objects if world configuration, state, and services are merged carelessly.
- Grid artifacts may influence future movement or ecology.
- Floating-point behavior may complicate exact deterministic comparison across platforms.

### Scalability Concerns

- `1024 x 1024` and larger worlds will require profiling before becoming defaults.
- Chunk-level dirty tracking must stay reliable as systems mutate more fields.
- Full validation may become expensive and may need debug/test versus release modes.
- Persistence must batch chunk snapshots and avoid per-tick database coupling.
- Future agents must query world state through stable APIs instead of depending on internal chunk storage.

### Assumptions

- Phase 1 uses Bevy ECS from Rust.
- The world is finite and rectangular.
- Coordinates are non-negative in Phase 1.
- The default world is `512 x 512`.
- The default chunk size is `32 x 32`.
- Chunk dimensions divide world dimensions exactly.
- `1 tick = 1 simulation hour` by default.
- Day, season, and year lengths are configurable world parameters.
- Cells are not ECS entities.
- Persistence is snapshot-based before agents exist.

### Unresolved Questions

- What numeric representation should field values use: floating point, fixed point, or bounded integer scales?
- What exact serialization format should snapshots use?
- Should slope be stored or derived from elevation on demand?
- Should energy availability be one combined field initially, or separate solar, thermal, biomass, and chemical fields?
- What is the minimum useful climate update frequency: hourly, daily, or seasonal?
- Should validation failures panic in development builds or return structured errors?
- When PostgreSQL is introduced, should it store raw snapshots, derived read models, or both?

## Implementation Gate

Rust implementation should not begin until this document is accepted as the Phase 1 technical baseline.

Any implementation that adds a non-goal must be rejected or moved to a later phase.

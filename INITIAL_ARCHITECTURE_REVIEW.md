# Initial Architecture Review

## Scope

This review is based on the current repository documentation:

- `docs/VISION.md`
- `docs/PRINCIPLES.md`
- `docs/ROADMAP.md`
- `docs/ARCHITECTURE_BASELINE.md`
- `docs/RESEARCH.md`
- `docs/GLOSSARY.md`
- `README.md`

No simulation code has been reviewed or implemented. The repository is currently in pre-development state, with empty top-level folders for `engine`, `api`, `dashboard`, and `database`.

## 1. Repository Assessment

Genesis has a strong conceptual foundation. The project vision is clear: do not directly program civilization, but create low-level conditions from which complex civilization-like behavior can emerge. This aligns well with ECS, deterministic systems, data-oriented design, and pressure-driven development.

The roadmap is correctly layered from environmental substrate to life, evolution, memory, decision-making, knowledge, society, culture, economy, technology, and civilization. This ordering is one of the repository's strongest architectural assets because it prevents higher-order concepts from being introduced before the simulation contains the pressures that justify them.

The current architecture documentation identifies the intended stack:

- Rust for the simulation engine.
- Bevy ECS for data-oriented simulation.
- PostgreSQL for persistence and historical storage.
- NestJS for API access.
- Next.js for dashboard and visualization.

This stack is reasonable, but Phase 1 should remain focused almost entirely on the Rust simulation engine. The API, database, and dashboard should not drive engine design yet. They should observe or persist outputs once the simulation has stable deterministic state transitions.

The repository is currently documentation-heavy and implementation-light. That is appropriate for this stage, but the next step should be to define enough engine boundaries to prevent the first implementation from turning into a collection of loosely related systems.

### Alignment With Vision

The proposed Phase 1 work should only model the world substrate:

- Terrain.
- Climate.
- Seasons.
- Resources.
- Simulation clock.
- Persistence.

It should not model agents, society, economies, politics, culture, or civilization. Those must remain emergent targets for later phases.

### Alignment With Principles

The repository principles favor:

- Simulation depth over graphics.
- Emergence over scripted features.
- Systems over objects.
- Data-oriented ECS.
- Pressure-driven development.

The Phase 1 implementation should therefore prioritize deterministic environmental pressures rather than visual polish or narrative outcomes. A successful Phase 1 world is not an interesting civilization; it is a stable, inspectable, persistent substrate capable of creating constraints for future life.

### Current Strengths

- Clear philosophical north star.
- Good phase ordering.
- Explicit rejection of hardcoded culture, economy, religion, and politics.
- Strong fit between goals and ECS architecture.
- Early recognition that the world must exist before life, agents, or history.

### Current Weaknesses

- Architecture is still high-level and does not define crate boundaries, data ownership, schedules, persistence strategy, or deterministic rules.
- The relationship between Bevy ECS and long-running headless simulation is not yet documented.
- PostgreSQL is listed, but its role in deterministic simulation versus derived observation/history is not clarified.
- There is no testing strategy yet, despite unit tests being required.
- There is no documented definition of determinism, tick semantics, save/load semantics, or simulation reproducibility.
- Documentation contains visible encoding artifacts in `ROADMAP.md` and `ARCHITECTURE_BASELINE.md`, likely from non-UTF-8 rendering or saved box-drawing characters.

## 2. Missing Documentation

The following documentation should be added before or alongside Phase 1 implementation.

## Required Before Phase 1 Code

### `docs/PHASE_1_WORLD.md`

Define the exact Phase 1 scope. This should include:

- World grid or spatial representation.
- Terrain model.
- Climate model.
- Season model.
- Resource model.
- Simulation clock.
- Persistence expectations.
- Explicit non-goals.

Non-goals should state that Phase 1 does not include agents, life, evolution, culture, economy, politics, language, or civilization.

### `docs/DETERMINISM.md`

Define how deterministic behavior is preserved:

- Fixed timestep.
- Seeded random number generation.
- Stable system ordering.
- No wall-clock-dependent simulation logic.
- No unordered iteration where ordering affects results.
- Deterministic save/load round trips.
- Reproducibility tests.

This is one of the most important missing documents.

### `docs/ECS_GUIDELINES.md`

Define how ECS should be used in Genesis:

- What belongs in components.
- What belongs in resources.
- What belongs in events.
- How systems should be scheduled.
- How to avoid manager classes and god resources.
- How to organize feature modules.
- How to keep systems small and testable.

### `docs/PERSISTENCE.md`

Clarify what persistence means:

- Is PostgreSQL the source of truth, or is it only for snapshots/history?
- Are saves full snapshots, deltas, event logs, or checkpoints?
- How are schema migrations handled?
- How is deterministic replay handled?
- What simulation state must be persisted in Phase 1?

Recommendation: for Phase 1, the engine state should be the source of truth while running. PostgreSQL should initially be treated as persistence/observation infrastructure, not as a live dependency inside every simulation system.

### `docs/TESTING.md`

Define required test categories:

- Unit tests for pure world-generation functions.
- System tests for ECS transitions.
- Determinism tests using fixed seeds.
- Save/load equivalence tests.
- Long-run stability tests.
- Property-style tests for invariants where practical.

### `docs/DATA_MODEL.md`

Define Phase 1 data types at a conceptual level:

- Coordinates.
- Terrain cells or chunks.
- Climate fields.
- Resource deposits.
- Time state.
- World generation configuration.
- World seed.

### `docs/OBSERVABILITY.md`

Define how the simulation will be inspected without turning inspection into gameplay scripting:

- Metrics.
- Logs.
- Snapshot exports.
- Debug queries.
- Dashboard read models.

### `docs/TERMINOLOGY.md` or Expanded Glossary

The current glossary is useful but short. It should expand to include:

- Pressure.
- Substrate.
- System.
- Component.
- Resource.
- Tick.
- Determinism.
- Emergence.
- Snapshot.
- Replay.
- Cell.
- Chunk.
- Biome, if used.

## Useful Soon, But Not Blocking

- `docs/API_BOUNDARIES.md` for engine/API/dashboard separation.
- `docs/HISTORY_MODEL.md` before memory or historical recording begins.
- `docs/PERFORMANCE_BUDGET.md` before large worlds are generated.
- `docs/MIGRATIONS.md` before PostgreSQL schema stabilizes.

## 3. Potential Architectural Risks

### Risk: Building Civilizational Concepts Too Early

The architecture document lists many high-level systems, including culture, language, economy, technology, politics, and history. That list is useful as a destination map, but it could encourage premature module creation.

Recommendation: do not create empty future systems in code. Only create Phase 1 modules that represent world substrate and environmental pressures.

### Risk: World System Becoming a God System

The term "World System" can easily grow into a catch-all for terrain, climate, resources, time, generation, persistence, events, and queries.

Recommendation: treat "World" as a domain boundary, not a single system. Inside it, use smaller systems such as terrain generation, climate update, seasonal update, resource regeneration, erosion, and snapshot export.

### Risk: Global Mutable State

Simulation projects often drift toward global world state, especially for terrain maps, random number generators, and clocks.

Recommendation: keep state in explicit ECS resources and components. Keep random generation seeded and passed through controlled resources. Avoid hidden singletons.

### Risk: Database-Coupled Simulation

PostgreSQL is part of the stack, but if simulation systems read/write the database during normal ticks, determinism and performance will suffer.

Recommendation: separate simulation state from persistence state. Systems should update ECS data. Persistence should happen at scheduled boundaries through snapshots, exports, or append-only records.

### Risk: Non-Deterministic Bevy Scheduling

Bevy ECS is powerful, but deterministic simulation requires careful schedule ordering and fixed timesteps.

Recommendation: define explicit schedules for world generation, tick simulation, post-tick validation, and persistence/export. Avoid relying on ambiguous system ordering when results depend on order.

### Risk: Overusing Entities for Dense Terrain

Representing every terrain tile as an ECS entity may become expensive for large worlds.

Recommendation: for Phase 1, evaluate a chunked data representation stored in ECS resources or chunk entities. Individual cells can be plain data within chunks. Use entities for coarse simulation objects, not necessarily every grid cell.

### Risk: Hardcoded Biomes Becoming Hardcoded Ecology

Terrain and climate may require classification, but hardcoded biome labels can become disguised design scripting.

Recommendation: store low-level measurable fields first: elevation, moisture, temperature, sunlight, soil fertility, water presence, mineral composition. Derive labels only for debugging or visualization.

### Risk: Visualization Driving Simulation Shape

The dashboard and visualization layer are valuable, but Phase 1 should not optimize the engine around rendering.

Recommendation: expose read-only snapshots or query outputs. Do not let UI needs define simulation data ownership.

### Risk: Missing Invariants

Without invariants, long-running simulation bugs will look like emergent behavior.

Recommendation: define world invariants early, such as valid coordinate bounds, non-negative resource quantities, stable tick progression, valid climate ranges, and reproducible generation from seed.

## 4. Recommended Folder Structure for Phase 1

The Phase 1 repository should keep the engine isolated and testable. Since the coding rules require Rust only for simulation code, the engine should be the first real implementation area.

Recommended structure:

```text
Genesis/
  docs/
    VISION.md
    PRINCIPLES.md
    ROADMAP.md
    ARCHITECTURE_BASELINE.md
    RESEARCH.md
    GLOSSARY.md
    PHASE_1_WORLD.md
    DETERMINISM.md
    ECS_GUIDELINES.md
    PERSISTENCE.md
    TESTING.md
    DATA_MODEL.md
    OBSERVABILITY.md

  engine/
    Cargo.toml
    src/
      lib.rs
      main.rs

      app/
        mod.rs
        schedules.rs
        plugins.rs

      world/
        mod.rs
        components.rs
        resources.rs
        systems.rs
        generation.rs
        terrain.rs
        climate.rs
        seasons.rs
        resources.rs
        coordinates.rs
        chunks.rs
        validation.rs

      time/
        mod.rs
        simulation_clock.rs
        systems.rs

      persistence/
        mod.rs
        snapshot.rs
        save.rs
        load.rs

      rng/
        mod.rs
        seed.rs

      config/
        mod.rs
        world_config.rs

      testing/
        mod.rs
        fixtures.rs

    tests/
      determinism.rs
      world_generation.rs
      persistence_roundtrip.rs
      long_run_world.rs

  database/
    migrations/
    README.md

  api/
    README.md

  dashboard/
    README.md
```

### Notes on Structure

- `engine/src/lib.rs` should expose the simulation library.
- `engine/src/main.rs` should be a thin executable wrapper for running the simulation.
- `world/` should contain Phase 1 substrate logic only.
- `time/` should be separate because later systems will depend on the simulation clock.
- `rng/` should be explicit because deterministic generation is foundational.
- `persistence/` should initially support snapshots and round-trip tests before database integration becomes elaborate.
- `api/` and `dashboard/` should remain placeholders until there is stable state to inspect.

One naming concern: Rust cannot have both `world/resources.rs` for ECS resources and a domain concept also named `resources.rs` without ambiguity in conversation. Consider naming the domain file `materials.rs`, `resource_fields.rs`, or `deposits.rs`, while keeping ECS resources in `ecs_resources.rs`.

## 5. Recommended ECS Architecture for World System

The World System should be a collection of small ECS systems operating on explicit data. It should create environmental pressure, not scripted outcomes.

## Core Design Goal

Phase 1 should produce a persistent world that can continue changing independently. Its job is to generate and maintain terrain, climate, seasonal cycles, and resource distributions that later life systems must adapt to.

## ECS Boundary

Use ECS for:

- Simulation scheduling.
- Global simulation resources.
- Chunk entities or world-region entities.
- Systems that transform world state.
- Validation and snapshot export.

Avoid using ECS as a storage mechanism for every scalar value if a dense array is more appropriate. A chunk can be an entity, while cells inside the chunk can be contiguous Rust data.

## Suggested Resources

### `WorldConfig`

Immutable configuration for world dimensions, chunk size, generation parameters, climate constants, and resource parameters.

### `WorldSeed`

The root deterministic seed for all generation and stochastic updates.

### `SimulationClock`

Current tick, day, season, year, and fixed timestep metadata.

### `WorldBounds`

Defines valid coordinate space.

### `SeasonState`

Current seasonal phase and derived seasonal modifiers.

### `GenerationState`

Tracks whether initial world generation has completed and which generation stage is active.

### `SnapshotConfig`

Controls when snapshots are emitted for persistence or inspection.

## Suggested Components

### `ChunkCoord`

Identifies the chunk position in world space.

### `TerrainChunk`

Dense storage for elevation, slope, water, soil, and other terrain fields.

### `ClimateChunk`

Dense storage for temperature, moisture, wind exposure, rainfall, and related climate fields.

### `ResourceChunk`

Dense storage for resource quantities or deposits. These should be low-level resources such as water, nutrients, minerals, biomass potential, or energy availability, not economic goods.

### `DirtyChunk`

Marks chunks changed since the last snapshot or export.

### `Generated`

Marks chunks that have completed initial generation.

## Suggested Events

Events should be used sparingly. For deterministic simulation, persistent state transitions should not rely on unbounded event history.

Useful Phase 1 events:

- `WorldGenerated`
- `SeasonChanged`
- `ChunkUpdated`
- `SnapshotRequested`
- `SnapshotCompleted`

## Suggested Schedules

### `StartupGeneration`

Runs once to create deterministic world substrate.

Order:

1. Load world configuration.
2. Initialize seeded RNG streams.
3. Spawn chunk entities.
4. Generate terrain fields.
5. Generate climate baselines.
6. Generate initial resource fields.
7. Validate initial world invariants.

### `FixedSimulationTick`

Runs once per deterministic simulation tick.

Order:

1. Advance simulation clock.
2. Update seasonal state.
3. Update climate fields.
4. Update water and moisture movement if included in Phase 1.
5. Update resource regeneration or depletion pressures.
6. Mark changed chunks.
7. Validate world invariants.

### `PersistenceBoundary`

Runs after simulation ticks at controlled intervals.

Order:

1. Build snapshot from ECS state.
2. Write snapshot or export data.
3. Clear snapshot-related dirty markers.

### `ObservationBoundary`

Runs after simulation state is stable.

Order:

1. Produce metrics.
2. Export debug summaries.
3. Provide dashboard/API read models when those layers exist.

## Recommended World Data Model

Phase 1 should begin with measurable fields rather than named abstractions.

Terrain fields:

- Elevation.
- Slope.
- Water depth or water presence.
- Soil depth.
- Soil fertility.

Climate fields:

- Temperature.
- Moisture.
- Rainfall.
- Seasonal modifier.
- Sunlight or latitude factor, if relevant.

Resource fields:

- Fresh water availability.
- Nutrient availability.
- Mineral concentration.
- Biomass carrying potential.

Time fields:

- Tick.
- Day.
- Season.
- Year.

Avoid starting with labels such as "kingdom", "market", "tribe", "religion", or even highly prescriptive biome behavior. Labels can be derived for inspection, but low-level fields should drive simulation pressure.

## Determinism Requirements

The World System should be considered valid only if:

- The same seed and config produce the same initial world.
- The same initial world and tick count produce the same final world.
- Save/load followed by continued simulation matches uninterrupted simulation.
- System ordering is explicit where order affects state.
- Tests cover generation, ticking, and persistence round trips.

## Testing Recommendations

Minimum Phase 1 tests:

- World generation is reproducible from a fixed seed.
- Different seeds produce different worlds.
- Generated terrain values stay within valid ranges.
- Climate values stay within valid ranges after many ticks.
- Resources never become negative.
- Simulation clock advances predictably.
- Save/load preserves world state.
- Running `N` ticks continuously equals running `A` ticks, saving, loading, then running `B` ticks where `A + B = N`.

## Tradeoffs

### Chunk Entities vs Cell Entities

Chunk entities are better for large worlds and data-oriented storage. Cell entities are simpler conceptually but may become expensive quickly.

Recommendation: use chunk entities with dense cell arrays inside components.

### Rich Climate Early vs Simple Climate Early

A rich climate model may create better future pressures, but it can delay implementation and make bugs harder to diagnose.

Recommendation: start with a small deterministic climate model using temperature, moisture, rainfall, and seasonal modifiers. Add complexity only when later phases need it.

### Database Early vs Snapshot Files First

PostgreSQL is useful for persistence and history, but early database coupling may slow iteration and obscure determinism bugs.

Recommendation: first support engine-native snapshots and round-trip tests. Add PostgreSQL after the snapshot format and persistence boundaries are stable.

### Bevy Full App vs Minimal ECS World

A full Bevy app can standardize scheduling, plugins, and resources. A minimal ECS world can be easier to test.

Recommendation: use Bevy ECS patterns, but keep simulation logic in pure functions and small systems that can be unit tested without launching a full application.

## Future Scalability Concerns

- Large worlds will require chunking, streaming, or sparse storage.
- Long histories will require careful separation between current state, snapshots, and historical events.
- Database writes must be batched and isolated from deterministic tick execution.
- Climate and resource models may need multi-resolution simulation to avoid expensive per-cell updates every tick.
- Future agents should interact with world pressure through clear query APIs instead of directly mutating terrain or climate internals.
- The dashboard will eventually need derived read models so it does not depend on internal ECS layout.
- As phases accumulate, schedule ordering must remain explicit and documented.

## Recommended Immediate Next Steps

1. Fix documentation encoding artifacts in `ROADMAP.md` and `ARCHITECTURE_BASELINE.md`.
2. Add `docs/PHASE_1_WORLD.md`.
3. Add `docs/DETERMINISM.md`.
4. Add `docs/ECS_GUIDELINES.md`.
5. Decide whether Phase 1 persistence starts with engine snapshots before PostgreSQL integration.
6. Scaffold the Rust engine only after the Phase 1 world scope and determinism rules are documented.

## Final Recommendation

Genesis should treat Phase 1 as the creation of a deterministic environmental substrate. The most maintainable path is to keep the initial engine narrow: world generation, time, seasons, climate, resources, validation, and persistence boundaries.

Do not create empty high-level civilization modules yet. Let future systems appear only when lower-level simulation pressure makes them necessary.

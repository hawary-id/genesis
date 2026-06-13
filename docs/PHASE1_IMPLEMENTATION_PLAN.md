# Phase 1 Implementation Plan

## Purpose

This document turns `PHASE1_WORLD_TECH_SPEC.md` into a step-by-step implementation roadmap.

It does not define Rust code or project scaffolding. It defines the engineering order, decisions, deliverables, and completion criteria needed to move from architecture to a working Phase 1 world simulation.

Phase 1 remains limited to deterministic world substrate:

- Terrain.
- Climate.
- Resources.
- Energy availability.
- Simulation time.
- Validation.
- Minimal snapshot persistence.
- Determinism tests.

## Pre-Implementation Decisions

The following decisions resolve implementation blockers identified in `PHASE1_WORLD_TECH_SPEC.md`.

## Numeric Representation Decision

### Decision

Use `f32` for continuous environmental fields:

- Temperature.
- Moisture.
- Rainfall.
- Elevation if represented as normalized or continuous height.
- Slope.
- Soil fertility.
- Biomass carrying potential if continuous.
- Energy availability.
- Solar exposure.
- Seasonal modifiers.
- Other continuous environmental fields.

Use `u32` for discrete simulation and indexing values:

- Simulation ticks.
- Coordinates.
- Indexes.
- Chunk counts.
- Cell counts.
- Array indexes where appropriate.
- Non-negative counters.

Use `u16` or `u32` for bounded environmental quantities:

- Resource quantities.
- Water availability if discrete.
- Nutrient quantities if discrete.
- Mineral quantities if discrete.
- Other bounded stock-like values.

Do not introduce fixed-point arithmetic in Phase 1.

### Rationale

`f32` is sufficient for Phase 1 environmental gradients and keeps memory cost lower than `f64`.

The default world has `512 x 512 = 262,144` cells. Multiple dense fields will be stored per cell, so field width matters. `f32` gives enough precision for climate and resource pressure without doubling continuous field memory.

`u32` is simple and safe for Phase 1 ticks, coordinates, indexes, and counters. It comfortably covers the default world dimensions, chunk counts, and long simulations using hourly ticks.

`u16` is acceptable for compact bounded resource quantities when the range is intentionally small. `u32` should be used when a quantity may need wider range or when avoiding conversion complexity is more important than memory savings.

Fixed-point arithmetic is deferred because Phase 1 needs clarity, testability, and iteration speed more than cross-platform bit-perfect numeric behavior.

### Tradeoffs

- `f32` may produce small platform or compiler differences in some calculations.
- Exact snapshot comparisons may require canonical serialization and careful comparison policy.
- `u32` is larger than necessary for small coordinates but avoids premature micro-optimization.
- `u16` saves memory but can overflow if ranges are chosen poorly.
- Fixed-point may eventually be needed for stronger cross-platform determinism.

### Future Implications

- Validation ranges must be explicit for every continuous field.
- Avoid chaotic formulas where tiny `f32` differences can amplify rapidly.
- Determinism tests should run on the supported development target before cross-platform guarantees are claimed.
- If future phases require bitwise-identical replay across platforms, selected fields can migrate to fixed-point behind versioned snapshot schemas.
- Snapshot schema versions must record numeric representation assumptions.

## Snapshot Format Decision

### Decision

Use a human-readable snapshot format for Phase 1.

Recommended default: JSON.

Acceptable alternative: RON.

Do not use in Phase 1:

- Custom binary formats.
- Compressed binary formats.
- Performance-oriented serialization formats.
- Database-native persistence as the primary snapshot representation.

### Rationale

Phase 1 persistence exists to support reproducibility, debugging, resume, and save/load equivalence tests. Human-readable snapshots make it easier to inspect world configuration, clock state, chunk metadata, and field ranges while the model is still evolving.

JSON is the safest default because it is widely supported by tools, easy to diff at small scales, easy to inspect, and straightforward for future API/dashboard layers to consume.

RON is acceptable if the Rust implementation strongly benefits from Rust-native readability, but JSON should remain the default unless implementation experience proves RON materially simpler.

### Debugging Benefits

- Snapshots can be opened and inspected without custom tools.
- Schema mistakes are easier to spot.
- Test fixtures can be reviewed in code review.
- Save/load failures can be diagnosed by comparing readable fields.
- Early persistence bugs are less likely to hide behind binary encoders.

### Determinism Implications

- Snapshot writing must use stable field ordering where the format and serializer allow it.
- Floating-point values must serialize consistently enough for save/load equivalence on supported targets.
- Deterministic continuation matters more than byte-for-byte identical formatting.
- Tests should compare loaded simulation state, not only raw snapshot text.

### Future Migration Path

- Phase 1 snapshots must include a schema version.
- Later phases may add compressed or binary formats after correctness is proven.
- Future PostgreSQL integration should import or store snapshots behind the persistence boundary.
- Event logs and history archives can be layered later, but they must not replace the Phase 1 snapshot baseline until agents and history exist.

## Validation Policy Decision

### Decision

Debug and test builds:

- Panic on invariant violation.

Release builds:

- Return structured validation errors.

### Rationale

During development, invariant violations indicate engine bugs or invalid test setup. Panicking early makes failures loud and prevents corrupt state from being mistaken for emergence.

In release-style execution, the engine should report validation failures in a structured way so callers, future API layers, or long-running runners can stop, snapshot diagnostics, or surface errors cleanly.

### Debugging Implications

- Bugs fail close to their source.
- Stack traces point to the violating system or validation rule.
- Tests can assert that invalid states fail immediately.
- Developers are discouraged from ignoring invalid world state.

### Production Implications

- Long-running simulations can fail gracefully.
- Future API or dashboard layers can display validation errors instead of crashing the process.
- Error types become part of the engine contract.
- Release behavior must still halt or reject invalid simulation progression; structured errors are not permission to continue with corrupt state.

### Future Implications

- Validation can later support severity levels, but Phase 1 should begin with strict invariants.
- Debug/test panics protect model correctness while systems are still small.
- Structured release errors prepare the engine for headless runs, API control, and automated experiments.

## Additional Implementation Defaults

These defaults remove remaining ambiguity without expanding Phase 1 scope.

### Slope

Slope should be derived during terrain generation and stored in `TerrainChunk`.

Rationale:

- Climate, water, validation, and future movement pressure can read slope without repeatedly deriving it.
- The stored value can be validated and snapshotted.

Tradeoff:

- If elevation changes later, slope must be recomputed.

### Energy Availability Shape

Begin with one aggregate `energy_availability` field plus optional source fields only if they are needed by generation.

Rationale:

- Phase 1 needs energy pressure, not a full energy taxonomy.
- A single aggregate field minimizes surface area before life consumes it.

Tradeoff:

- Later ecology or technology may need source-specific energy fields.

### Climate Update Frequency

Begin with daily climate updates derived from hourly ticks.

Rationale:

- `1 tick = 1 hour` remains the base time unit.
- Climate does not need to mutate every hour before agents exist.
- Daily updates reduce churn while preserving seasonal progression.

Tradeoff:

- Diurnal temperature variation may be coarse or derived rather than fully simulated.

## Implementation Roadmap

Milestones are ordered to minimize refactoring and preserve deterministic behavior. A milestone should not begin until its dependencies are stable enough to test.

## Milestone 1: Project Foundation

### Objective

Establish the Rust engine foundation, configuration vocabulary, and module boundaries needed for Phase 1.

### Deliverables

- Rust engine crate structure.
- Phase 1 module boundaries for app, world, time, rng, config, persistence, and validation.
- Documented public data type stubs for architectural concepts.
- Dependency decisions for ECS, serialization, and testing.
- Initial test harness setup.

### Dependencies

- `PHASE1_WORLD_TECH_SPEC.md`.
- Numeric representation decision.
- Snapshot format decision.
- Validation policy decision.

### Success Criteria

- The project can compile with no simulation behavior.
- Public modules reflect Phase 1 boundaries only.
- No future civilization, agent, society, economy, culture, or AI modules exist.
- Testing infrastructure can run an empty or placeholder test suite.

### Risks

- Scaffolding future phases too early.
- Creating manager-like modules or global mutable state.
- Letting dependency setup dictate architecture.

## Milestone 2: Core ECS Setup

### Objective

Define the ECS shape for Phase 1 without implementing full simulation behavior.

### Deliverables

- Core component definitions:
  - `ChunkCoord`.
  - `TerrainChunk`.
  - `ClimateChunk`.
  - `ResourceChunk`.
  - `EnergyAvailabilityChunk`.
  - `DirtyChunk`.
  - `Generated`.
- Core resource definitions:
  - `WorldConfig`.
  - `WorldSeed`.
  - `SimulationClock`.
  - `WorldBounds`.
  - `SeasonState`.
  - `GenerationState`.
  - `SnapshotConfig`.
- Required Phase 1 event definitions:
  - `WorldGenerationCompleted`.
  - `SnapshotRequested`.
  - `SnapshotCompleted`.
- Explicit schedule definitions:
  - `StartupGeneration`.
  - `FixedSimulationTick`.
  - `PostTickValidation`.
  - `PersistenceBoundary`.
  - `ObservationBoundary`.

### Dependencies

- Milestone 1.

### Success Criteria

- ECS world can be initialized with Phase 1 resources.
- Schedules exist in the specified order.
- No systems rely on hidden global state.
- Components are data-only.
- Resources remain narrow and purposeful.

### Risks

- Overloading `WorldConfig` with runtime state.
- Making events carry core simulation truth.
- Introducing manager objects under different names.

## Milestone 3: World Generation

### Objective

Generate the initial deterministic terrain substrate and chunk layout.

### Deliverables

- World configuration validation.
- Seed initialization and deterministic seed derivation.
- `512 x 512` default world support.
- `32 x 32` default chunk support.
- `256 x 256` test world support.
- Chunk entity spawning.
- Coordinate conversion utilities for `WorldCoord`, `ChunkCoord`, and `LocalCoord`.
- Terrain generation for elevation, slope, water, soil depth, and soil fertility.
- Initial generated chunk marking.

### Dependencies

- Milestone 2.

### Success Criteria

- Same seed and configuration produce identical terrain chunks.
- Generated chunk count matches world dimensions and chunk size.
- Coordinate conversions round-trip correctly.
- All generated chunks have required terrain data.
- Terrain fields remain within configured ranges.

### Risks

- Accidentally depending on iteration order.
- Generating terrain in ways that make later climate impossible.
- Making terrain generation too realistic or too expensive.

## Milestone 4: Climate System

### Objective

Add deterministic climate baseline generation and daily climate updates without becoming a weather simulator.

### Deliverables

- Climate baseline generation for temperature, moisture, rainfall, seasonal modifier, and sunlight or latitude factor.
- Daily climate update system driven by `SimulationClock` and `WorldConfig`.
- Climate range validation hooks.
- Climate generation tests.

### Dependencies

- Milestone 3.
- Simulation clock data type from Milestone 2.

### Success Criteria

- Same seed, terrain, and configuration produce identical climate fields.
- Climate updates are deterministic for a given tick sequence.
- Climate values remain within configured ranges.
- No storms, atmospheric simulation, fluid dynamics, or detailed weather systems are introduced.

### Risks

- Drifting into weather simulation.
- Updating climate every tick without need.
- Creating formulas that amplify `f32` differences chaotically.

## Milestone 5: Resource System

### Objective

Generate and update material environmental resources as low-level availability fields.

### Deliverables

- Resource generation for fresh water availability, nutrients, minerals, and biomass carrying potential.
- Resource update system using terrain, climate, and seasonal context.
- Non-negative resource validation.
- Resource generation and update tests.

### Dependencies

- Milestone 3.
- Milestone 4.

### Success Criteria

- Same seed, terrain, climate, and configuration produce identical resource fields.
- Resource quantities never become negative.
- Resources are not modeled as goods, prices, ownership, or production.
- Resource updates are deterministic.

### Risks

- Sneaking in economic concepts.
- Making biomass imply organisms before life exists.
- Choosing resource ranges too narrow for future phases.

## Milestone 6: Energy Availability System

### Objective

Generate and update first-class environmental energy availability.

### Deliverables

- Aggregate energy availability field.
- Energy generation derived from terrain, climate, sunlight, resources, and configuration.
- Daily or season-aware energy update system.
- Energy range validation.
- Energy availability tests.

### Dependencies

- Milestone 3.
- Milestone 4.
- Milestone 5.

### Success Criteria

- Energy availability is explicit and deterministic.
- Energy values remain within configured ranges.
- Energy is not modeled as agent stamina, calories, labor, fuel inventory, industry, or economy.
- Same seed and configuration produce identical energy fields.

### Risks

- Over-modeling thermodynamics.
- Under-modeling energy so it becomes meaningless later.
- Duplicating climate or resource fields without a clear aggregate purpose.

## Milestone 7: Simulation Clock

### Objective

Implement canonical tick advancement and configurable day, season, and year derivation.

### Deliverables

- `SimulationClock` tick advancement behavior.
- Configurable day length in ticks.
- Configurable season length in days.
- Configurable seasons per year.
- Derived day, season, year, and seasonal progress helpers.
- `SeasonState` update system.
- Clock and season tests.

### Dependencies

- Milestone 2.
- Climate/resource/energy systems may initially use placeholder time until this milestone, but must be integrated here.

### Success Criteria

- `1 tick = 1 hour` is the default.
- Day, season, and year lengths come from `WorldConfig`.
- Tick count is monotonic.
- `SeasonState` is derivable from clock and config.
- No cultural calendars are introduced.

### Risks

- Hardcoding day or season constants in systems.
- Allowing multiple sources of time truth.
- Tying simulation time to wall-clock time.

## Milestone 8: Validation Framework

### Objective

Centralize invariant checks and enforce debug/release validation policy.

### Deliverables

- Validation rule definitions for coordinates, chunks, terrain, climate, resources, energy, and time.
- Structured validation error model for release behavior.
- Debug/test panic behavior on invariant violation.
- Validation systems for `StartupGeneration` and `PostTickValidation`.
- Validation test suite.

### Dependencies

- Milestone 3.
- Milestone 4.
- Milestone 5.
- Milestone 6.
- Milestone 7.

### Success Criteria

- Invalid states are rejected.
- Debug/test builds panic on invariant violation.
- Release builds can return structured validation errors.
- Validation detects non-negative resource failures, invalid climate ranges, invalid energy ranges, invalid coordinates, chunk inconsistency, and non-monotonic time.

### Risks

- Validation becoming too expensive for long runs.
- Validation errors being too vague to debug.
- Release behavior accidentally continuing after invalid state.

## Milestone 9: Persistence

### Objective

Add minimal snapshot-based persistence behind the `PersistenceBoundary`.

### Deliverables

- JSON snapshot schema.
- Snapshot schema version.
- Snapshot construction from stable ECS state.
- Snapshot load path sufficient for deterministic continuation.
- Snapshot request and completion event handling.
- Save/load tests.

### Dependencies

- Milestone 8.

### Success Criteria

- Snapshots include configuration, seed, generation version, simulation clock, terrain state, climate state, resource state, energy availability state, and schema version.
- Persistence does not mutate simulation state.
- No event sourcing, per-tick logs, history archives, or database-first state exists.
- Snapshots can be inspected by humans.

### Risks

- Persistence taking over the architecture.
- Snapshot format expanding before agents exist.
- Comparing raw snapshot formatting instead of loaded simulation state.

## Milestone 10: Determinism Testing

### Objective

Prove that Phase 1 generation, ticking, validation, and persistence are deterministic enough for implementation completion.

### Deliverables

- Deterministic generation tests.
- Deterministic ticking tests.
- Save/load equivalence tests.
- Long-run stability tests.
- Test fixtures for `256 x 256`.
- Default-world long-run test for `512 x 512`.

### Dependencies

- Milestone 9.

### Success Criteria

- Same seed and config produce identical worlds.
- Different seeds produce different worlds.
- Running the same world for the same tick count produces identical final state.
- Running `N` ticks continuously equals running `A` ticks, saving, loading, then running `B` ticks where `A + B = N`.
- A default `512 x 512` world can run for the agreed long-run target without invariant failure.

### Risks

- Tests becoming brittle because they compare incidental formatting.
- Long-run tests becoming too slow for normal development.
- Hidden nondeterminism from unordered iteration or RNG call ordering.

## Implementation Readiness Review

### Resolved Decisions

The following previously unresolved decisions are now resolved for Phase 1:

- Numeric representation:
  - `f32` for continuous environmental fields.
  - `u32` for ticks, coordinates, indexes, and counters.
  - `u16` or `u32` for bounded environmental quantities.
  - No fixed-point arithmetic in Phase 1.
- Snapshot format:
  - JSON by default.
  - RON acceptable only if JSON proves materially awkward during Rust implementation.
  - No custom binary, compressed binary, or performance-oriented snapshot format in Phase 1.
- Validation policy:
  - Debug/test builds panic on invariant violation.
  - Release builds return structured validation errors.
- Slope:
  - Derived during terrain generation and stored in `TerrainChunk`.
- Energy availability:
  - Begin with one aggregate energy availability field.
- Climate update frequency:
  - Begin with daily updates derived from hourly ticks.

### Remaining Non-Blocking Questions

The following remain intentionally deferred and are not blockers for Phase 1 implementation:

- Whether future phases need fixed-point arithmetic for cross-platform bitwise determinism.
- Whether future snapshots should migrate from JSON to a compact binary format.
- Whether PostgreSQL later stores raw snapshots, derived read models, or both.
- Whether energy availability later splits into separate solar, thermal, biomass, and chemical fields.
- Whether climate updates later need hourly, event-driven, or multi-rate behavior.

### Readiness Statement

No critical architectural blockers remain before Rust implementation begins.

Phase 1 is ready for implementation, provided implementation stays within this plan and does not introduce Phase 1 non-goals.

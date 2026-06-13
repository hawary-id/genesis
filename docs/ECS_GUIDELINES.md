# ECS Guidelines

## Purpose

Genesis uses ECS to model interactions between data and systems.

ECS is not only a technical choice. It protects the project philosophy:

- Systems over objects.
- Data-oriented design.
- Emergence over scripting.
- No manager classes.
- No god objects.
- No global mutable state.

These guidelines define how Phase 1 should use ECS before implementation begins.

## Design Choice: Systems Own Behavior, Components Own Data

Components should store data. Systems should transform data.

Rationale:

- This keeps behavior composable and inspectable.
- Data remains easy to serialize, validate, and test.
- It avoids object-style entities with hidden methods and private simulation logic.

Tradeoffs:

- Related behavior may be spread across multiple systems.
- Data types need clear names because they carry less behavior.
- Some developers may find object-oriented modeling more familiar.

Future implications:

- Later phases can add systems that interpret existing data in new ways.
- Components remain stable contracts between systems.
- Emergent behavior can arise from system interaction rather than scripted object methods.

## Design Choice: Components Are Small And Domain-Specific

Components should represent focused pieces of state.

For Phase 1, likely components include:

- `ChunkCoord`.
- `TerrainChunk`.
- `ClimateChunk`.
- `ResourceChunk`.
- `EnergyAvailabilityChunk`.
- `DirtyChunk`.
- `Generated`.

Rationale:

- Small components make system access patterns explicit.
- Domain-specific data avoids vague containers such as `WorldData`.
- Focused components are easier to test and serialize.

Tradeoffs:

- More component types require more module organization.
- Some systems may query several components together.
- Too much fragmentation can make the model harder to read.

Future implications:

- New world data can be added without rewriting one large world object.
- Later systems can depend only on the components they need.
- Performance can be tuned by grouping data according to access patterns.

## Design Choice: Energy Availability Has Explicit Storage

Energy availability should be represented explicitly rather than hidden inside climate or resource fields.

Rationale:

- Energy flow is a substrate-level pressure for life, ecology, technology, and economy.
- Keeping it explicit prevents later systems from inventing incompatible energy concepts.
- Energy can vary spatially and seasonally, making it a natural chunk field.

Tradeoffs:

- Phase 1 stores a field that no agent consumes yet.
- The model must remain abstract enough to avoid premature metabolism or industry.
- Some energy availability may be derivable from sunlight and climate, creating possible duplication.

Future implications:

- Life can later read environmental energy opportunity without hardcoded food placement.
- Technology and economy can later emerge from energy access and conversion constraints.
- Energy storage, consumption, and labor should remain later-phase concepts, not Phase 1 behavior.

## Design Choice: Resources Represent Shared Simulation Context

ECS resources should hold shared context that is not naturally attached to one entity.

For Phase 1, likely resources include:

- `WorldConfig`.
- `WorldSeed`.
- `SimulationClock`.
- `WorldBounds`.
- `SeasonState`.
- `GenerationState`.
- `SnapshotConfig`.

Rationale:

- Some state is global to a simulation run.
- Explicit resources are better than hidden globals.
- Systems can declare their shared dependencies clearly.

Tradeoffs:

- Resources can become god objects if they absorb too much state.
- Global access can make dependencies feel easy even when they should be local.
- Mutable resources can create scheduling conflicts.

Future implications:

- Shared state must remain narrow and purposeful.
- Future phases should add their own resources only when the state is truly shared.
- Any resource that grows too large should be split by responsibility.

## Design Choice: No Manager Classes

Genesis should not introduce manager classes or manager-like resources.

Avoid names and structures such as:

- `WorldManager`.
- `SimulationManager`.
- `ResourceManager`.
- `ClimateManager`.
- `EntityManager`.

Rationale:

- Managers centralize behavior and tend to become god objects.
- ECS already provides coordination through schedules, queries, resources, and systems.
- Manager objects hide dependencies that systems should declare explicitly.

Tradeoffs:

- Some coordination logic must be expressed through schedules and resources.
- There may be no single object that explains "how everything works".
- Developers must learn to navigate system flow instead of manager methods.

Future implications:

- The engine will scale by adding systems, not by expanding central controllers.
- Schedule documentation becomes important.
- Debug tooling should inspect resources and systems rather than manager internals.

## Design Choice: World Is A Domain Boundary, Not One System

The "World System" should be implemented as a group of small systems and data types.

Rationale:

- Terrain, climate, seasons, resources, validation, and persistence have different responsibilities.
- A single world system would become too broad.
- Smaller systems are easier to order, test, and replace.

Tradeoffs:

- Module organization matters more.
- Some cross-system data flow must be made explicit.
- There is more up-front design pressure around schedule boundaries.

Future implications:

- Future ecology and life systems can integrate with specific world systems.
- Terrain or climate models can evolve independently.
- Performance tuning can target individual systems.

## Design Choice: Chunk Entities For Dense World Data

Use ECS entities to represent chunks or regions, not every terrain cell.

Rationale:

- ECS entity-per-cell designs can become expensive for large worlds.
- Dense cell arrays fit terrain and climate data better.
- Chunk entities still allow ECS scheduling, markers, and change tracking.

Tradeoffs:

- Cell access requires helper APIs.
- Systems cannot query individual cells as entities.
- Chunk boundary logic must be designed carefully.

Future implications:

- Streaming, dirty tracking, snapshots, and parallel chunk updates become practical.
- Future agents can query world chunks without forcing terrain cells into ECS.
- If some future feature needs entity-like cells, it can be added selectively.

## Design Choice: Events Are For Boundaries, Not Core State

Events should represent notable transitions or requests, not the main storage of simulation truth.

Useful Phase 1 events may include:

- `WorldGenerated`.
- `SeasonChanged`.
- `ChunkUpdated`.
- `SnapshotRequested`.
- `SnapshotCompleted`.

Rationale:

- Events are useful for decoupling systems at boundaries.
- Core state should remain in components and resources.
- Deterministic replay is easier when persistent state is explicit.

Tradeoffs:

- Some workflows may require direct state queries instead of event streams.
- Events must be drained and ordered carefully.
- Overuse of events can hide causality.

Future implications:

- History systems can later observe important events after state transitions.
- Dashboard and persistence layers can react to boundary events.
- Core simulation will remain understandable from state plus schedule.

## Design Choice: Explicit Schedules

Phase 1 should define explicit schedules for world generation, fixed ticks, validation, persistence, and observation.

Recommended schedules:

- `StartupGeneration`.
- `FixedSimulationTick`.
- `PostTickValidation`.
- `PersistenceBoundary`.
- `ObservationBoundary`.

Rationale:

- Determinism requires stable order where order affects results.
- Schedules communicate architecture better than scattered system registration.
- Persistence and observation should not run in the middle of state mutation.

Tradeoffs:

- Explicit schedules require maintenance.
- Overly strict ordering can reduce parallel execution.
- Schedule names become part of project vocabulary.

Future implications:

- Later phases can be inserted into known schedule points.
- System ordering can be reviewed before behavior changes.
- Tests can run specific schedules without launching the full application.

## Design Choice: Systems Should Be Small And Testable

Systems should orchestrate data access and call smaller functions for core logic where practical.

Rationale:

- Small systems are easier to reason about.
- Pure or near-pure functions can be unit tested directly.
- ECS integration tests can focus on data flow and scheduling.

Tradeoffs:

- More functions and modules may be created.
- Some logic may feel split between systems and domain functions.
- Excessive fragmentation can hurt readability.

Future implications:

- Future model changes can be tested without rewriting schedule code.
- Performance-critical functions can be optimized in isolation.
- Bugs in emergence can be traced to specific transformations.

## Design Choice: Public APIs Must Be Documented

Public engine APIs and public data types must be documented.

Rationale:

- Genesis is a long-term project and will accumulate many systems.
- Documentation prevents accidental misuse across phases.
- Public APIs define the contract between world, life, persistence, API, and dashboard layers.

Tradeoffs:

- Documentation takes time to maintain.
- Early APIs may change as the model matures.
- Over-documenting unstable internals can slow iteration.

Future implications:

- Stable world query APIs can protect internal storage from future agents and dashboards.
- Persistence and external layers can depend on documented contracts.
- Documentation can become a design review tool before implementation changes.

## Design Choice: Module Boundaries Follow Simulation Domains

Modules should follow simulation responsibilities, not technical convenience.

Recommended Phase 1 engine modules:

- `app` for schedules and plugin assembly.
- `world` for terrain, climate, resources, chunks, coordinates, and validation.
- `time` for simulation clock behavior.
- `rng` for deterministic seed handling.
- `config` for world configuration.
- `persistence` for snapshots and load/save boundaries.

Rationale:

- Domain-oriented modules make the simulation easier to navigate.
- Technical utilities stay separate from world rules.
- Phase boundaries remain visible in code structure.

Tradeoffs:

- Some files may start small.
- Cross-module dependencies must be kept intentional.
- Naming must avoid ambiguity, especially around resources as ECS resources versus world resources.

Future implications:

- Later phases can add modules only when lower-level pressures justify them.
- Empty future modules should be avoided.
- Domain boundaries can become review boundaries for architecture decisions.

## Design Choice: Persistence Is A Boundary Module

Persistence should be a boundary module that snapshots stable ECS state. It should not own world logic or run database writes inside core simulation systems.

Rationale:

- Persistence is necessary for reproducibility, but it is not the purpose of Phase 1.
- Keeping persistence at a boundary protects deterministic tick execution.
- Before agents exist, snapshots are enough to resume and compare world state.

Tradeoffs:

- Some persistence code may duplicate serialization concerns from world components.
- Real-time database views may lag behind current ECS state.
- Detailed historical replay is deferred.

Future implications:

- PostgreSQL can be introduced as a snapshot and read-model store without changing world systems.
- Event history can be added when agents, memory, and history exist.
- Persistence failures can be handled outside the simulation update path.

## Design Choice: Read-Only Query Boundaries For Future Layers

API, dashboard, and future agent systems should read world state through explicit query boundaries rather than directly owning internal world storage.

Rationale:

- Internal storage can change without breaking external layers.
- The dashboard should observe, not shape, simulation state.
- Agents should experience environmental pressure through controlled access.

Tradeoffs:

- Query APIs require design and maintenance.
- Direct access may be faster for some internal systems.
- Too much abstraction too early can slow Phase 1.

Future implications:

- Dashboard read models can be derived from snapshots.
- Agent perception can later become a constrained query layer.
- Internal chunk storage can evolve without rewriting external code.

## Design Choice: Validation Systems Are First-Class Systems

Validation should be part of scheduled simulation flow, especially after generation and ticks.

Rationale:

- Invalid state should be detected before later systems consume it.
- Long-running simulations need guardrails.
- Tests can reuse validation logic.

Tradeoffs:

- Runtime validation adds cost.
- Validation must distinguish fatal errors from acceptable variation.
- Debug builds may validate more aggressively than release runs.

Future implications:

- Later phases can add their own invariants.
- Emergent behavior can be trusted more when base invariants hold.
- Validation failures can become high-quality debugging signals.

## ECS Review Checklist

Before Phase 1 code is accepted:

- No manager classes or manager-like resources exist.
- No future civilization modules are scaffolded prematurely.
- Components are data-only.
- Resources are narrow and purposeful.
- Systems are small and scheduled explicitly.
- Dense terrain data uses chunks rather than entity-per-cell by default.
- Energy availability is modeled explicitly as an environmental field.
- Events are used only for boundaries or notable transitions.
- Persistence remains a boundary concern and does not dominate Phase 1.
- Public APIs are documented.
- Determinism-sensitive systems have tests.

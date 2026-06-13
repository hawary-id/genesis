# Determinism

## Purpose

Determinism means that the same initial state, configuration, seed, and tick sequence always produce the same result.

For Genesis, determinism is not an optimization. It is a foundation for debugging, testing, persistence, replay, and long-term trust in emergence.

If a surprising outcome appears, the engine must be able to reproduce it.

## Design Choice: Fixed Timestep

Simulation must advance through fixed ticks.

Wall-clock time may control how quickly ticks are executed, but it must not change simulation results.

Rationale:

- Fixed ticks make systems reproducible.
- Tests can run exact tick counts.
- Save/load and replay can compare state after known intervals.

Tradeoffs:

- Real-time rendering or dashboard updates must be decoupled from simulation time.
- Fast-forward modes must process many fixed ticks instead of using variable time jumps.
- Some slow systems may require lower-frequency schedules.

Future implications:

- Later agents, memory, aging, seasons, and history can share one canonical timeline.
- Multi-rate schedules can be added while preserving the fixed tick as the base unit.
- Historical records can reference exact ticks instead of approximate timestamps.

## Design Choice: Seeded Randomness Only

All randomness must come from explicit seeded random number generators.

No system may use hidden entropy, current time, process IDs, unordered memory addresses, or platform-specific randomness during simulation.

Rationale:

- Seeded randomness allows generated worlds to be reproduced.
- Bugs involving rare outcomes can be replayed.
- Tests can assert exact behavior for known seeds.

Tradeoffs:

- RNG ownership must be designed carefully.
- Parallel systems cannot casually share mutable RNG state.
- Changing RNG algorithms or call order can change generated worlds.

Future implications:

- Seed streams should be named by domain, such as terrain, climate, resources, and future life.
- Save files must persist enough RNG state to continue exactly.
- Versioned generation algorithms may be needed when old worlds must remain reproducible.

## Design Choice: Stable System Ordering

Any systems whose results depend on order must be ordered explicitly.

Rationale:

- ECS scheduling can otherwise make order ambiguous.
- Deterministic output requires deterministic read/write timing.
- Explicit schedules document the simulation model.

Tradeoffs:

- Explicit ordering can reduce parallelism.
- Schedule definitions require maintenance as systems grow.
- Over-ordering can make the engine more rigid than necessary.

Future implications:

- Independent systems can still run in parallel when they do not affect each other.
- Future phases must document where they enter the schedule.
- Regression tests should detect accidental schedule changes.

## Design Choice: Deterministic Iteration

Simulation logic must not depend on unordered iteration.

When order affects results, use stable ordering by coordinates, entity identifiers designed for stability, or explicit sorted keys.

Rationale:

- Hash map iteration and ECS query order may not be stable enough for simulation semantics.
- Neighbor interactions can produce different results if processed in different orders.
- Stable iteration makes cross-run comparison possible.

Tradeoffs:

- Sorting can add cost.
- Stable keys must be designed and preserved.
- Some ECS conveniences may be inappropriate for order-sensitive systems.

Future implications:

- Chunk coordinates should become natural stable ordering keys.
- Agent systems in later phases will need similar rules for conflict resolution.
- Deterministic batching can support parallel execution later if merge rules are explicit.

## Design Choice: Pure Functions For Core Transformations

Core transformations should be written as pure or near-pure functions where practical.

Examples:

- Terrain generation for a coordinate and seed.
- Seasonal modifier for a tick.
- Climate update for a cell given previous values.
- Resource and energy availability updates for a cell given environmental inputs.

Rationale:

- Pure functions are easier to unit test.
- ECS systems become orchestration rather than hidden logic containers.
- Deterministic behavior is easier to reason about.

Tradeoffs:

- Some ECS boilerplate may be needed to pass data into pure functions.
- Complex systems may still require stateful resources.
- Over-purifying code can fragment logic if taken too far.

Future implications:

- Generation algorithms can be tested without launching the full app.
- Later phases can reuse world functions through controlled query APIs.
- Refactoring ECS schedules will be safer when behavior lives in small functions.

## Design Choice: Save/Load Equivalence

Saving and loading must preserve simulation state exactly enough for deterministic continuation, without expanding Phase 1 into a full history or database project.

Required equivalence:

Running `N` ticks continuously must match running `A` ticks, saving, loading, and then running `B` ticks where `A + B = N`.

Rationale:

- Persistence is part of Phase 1 success.
- Long-running worlds require reliable checkpoints.
- Replay and debugging depend on exact restoration.
- Before agents exist, snapshots are sufficient because there are no memories, decisions, social events, or institutions to preserve.

Tradeoffs:

- Snapshots must include all state that can affect future ticks.
- Floating-point values may require careful serialization choices.
- Save formats become part of the engine contract.
- Phase 1 will not provide complete event replay or historical auditability.

Future implications:

- Save files should carry schema and engine version metadata.
- Event logs or deltas can be added later, but snapshots should come first.
- Historical storage can build on deterministic checkpoints.
- PostgreSQL integration should remain behind the persistence boundary and should not shape tick execution.

## Design Choice: Controlled Floating-Point Use

Floating-point values may be used, but their role must be controlled and tested.

Rationale:

- Terrain, climate, and resource fields naturally use continuous values.
- Avoiding floating point entirely would make models awkward.
- Controlled use is practical for Phase 1 if tests run on supported targets.

Tradeoffs:

- Floating-point behavior can differ across platforms, compiler settings, or math functions.
- Exact binary comparison may be too strict for some future calculations.
- Deterministic replay across every possible platform may require fixed-point arithmetic later.

Future implications:

- Phase 1 should avoid chaotic floating-point formulas where tiny differences explode quickly.
- Persistence should use stable serialization formats.
- If cross-platform bitwise determinism becomes required, selected fields may move to fixed-point types.

## Design Choice: No Wall-Clock Simulation Logic

Simulation outcomes must not depend on the current date, current time, frame duration, CPU speed, or network timing.

Rationale:

- External timing cannot be replayed reliably.
- Wall-clock dependence makes tests flaky.
- The simulation clock should be the only time source for world behavior.

Tradeoffs:

- Real-time integrations must translate wall-clock time into requested tick counts.
- Dashboard and API layers cannot directly drive simulation state by timing side effects.
- Pausing, resuming, and fast-forwarding require explicit tick control.

Future implications:

- Headless simulation, replay, and server execution will remain consistent.
- UI framerate can change without changing the world.
- Distributed or remote control can be added without compromising core simulation semantics.

## Design Choice: Deterministic Persistence Boundary

Persistence must occur after simulation state reaches a stable boundary.

Rationale:

- Saving mid-update can capture invalid partial state.
- Explicit boundaries make snapshots easier to validate.
- Simulation systems should not depend on database timing or I/O success.

Tradeoffs:

- The newest tick may not be persisted immediately.
- Snapshot frequency becomes a design parameter.
- Persistence failures must be reported without mutating simulation results.

Future implications:

- PostgreSQL can store snapshots and derived history without becoming part of tick logic.
- Future history systems can record events after state transitions are complete.
- Crash recovery policies can be defined independently from simulation rules.

## Design Choice: Minimal Phase 1 Persistence Scope

Phase 1 persistence should support only the minimum needed to resume and compare deterministic world state.

Required persisted state:

- Configuration.
- Seed and generation version.
- Simulation clock.
- Chunk terrain fields.
- Chunk climate fields.
- Chunk resource and energy availability fields.
- Snapshot schema version.

Not required before agents:

- Append-only event history.
- Per-tick database writes.
- Social, economic, political, or cultural records.
- Memory records.
- Full replay from every intermediate event.

Rationale:

- Persistence should protect determinism, not dominate the first implementation.
- Agentless worlds have state, but they do not yet have historical meaning.
- A narrow snapshot contract is easier to test and evolve.

Tradeoffs:

- Fine-grained historical debugging will be limited.
- Snapshot cadence determines how much progress can be lost after failure.
- Later migration work will be needed when history becomes a first-class system.

Future implications:

- Once agents exist, meaningful events can be logged at stable schedule boundaries.
- Snapshot persistence can become the base layer beneath future event sourcing.
- Database schema design can wait until the engine has stable world state to persist.

## Design Choice: Determinism Tests As Required Tests

Determinism must be tested directly.

Minimum tests:

- Same seed and configuration produce identical worlds.
- Different seeds produce meaningfully different worlds.
- Running the same world for the same ticks produces identical state.
- Save/load continuation matches uninterrupted simulation.
- Clock progression is exact.
- Invariants hold after long runs.

Rationale:

- Determinism cannot be assumed from code style.
- Tests protect against accidental schedule, RNG, and persistence changes.
- Long-term maintainability depends on reproducible failures.

Tradeoffs:

- Snapshot comparison tests can be brittle if they compare too much incidental data.
- Test fixtures must be versioned with care.
- Deterministic tests may need updating when intentional algorithm changes occur.

Future implications:

- Engine changes can be validated before they affect higher phases.
- Emergent behavior can be investigated from saved seeds and tick counts.
- Future phases must add their own determinism tests when they introduce new state.

## Determinism Checklist

Before Phase 1 code is considered ready:

- All simulation time comes from `SimulationClock`.
- All randomness comes from explicit seeded RNG state.
- Order-sensitive systems have explicit order.
- Order-sensitive iteration uses stable keys.
- Core generation and update logic has unit tests.
- Save/load equivalence has an integration test.
- Persistence occurs only at stable boundaries.
- Persistence stores only the minimum state needed before agents exist.
- Invariants are checked after generation and ticks.

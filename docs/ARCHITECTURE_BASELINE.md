# Architecture Baseline

## 1. Project Status

Genesis is currently in Phase 3 (Evolution).

- **Milestones 1-15 (Phase 1 & Phase 2):** Complete and Locked ✅
- **Milestones 16-19 (Phase 3 Genetics, Consumption, Reproduction, Mutation):** Implemented and Verified ✅

---

## 2. Current ECS Architecture

Genesis uses Bevy ECS as its simulation framework. In accordance with data-oriented principles, all state storage is decoupled from execution logic:
- **Components and Resources** store data and derive serialization traits. They do not run logic.
- **Systems** perform state queries, logic execution, and event emission. They are scheduled in explicit sequences.
- **Managers** (controllers, centralized classes, god objects) are strictly banned.

### Active Resources
- [`WorldConfig` (in config)](https://github.com/hawary-id/genesis/blob/main/engine/src/config/world_config.rs): Immutable simulation parameters (world size, chunk size, cycle ticks, validation ranges).
- [`WorldBounds` (in config)](https://github.com/hawary-id/genesis/blob/main/engine/src/config/world_bounds.rs): Derived world coordinate limits, verified at startup.
- [`WorldSeed` (in rng)](https://github.com/hawary-id/genesis/blob/main/engine/src/rng/seed.rs): Root seed resource for reproducible execution.
- [`SimulationClock` (in time)](https://github.com/hawary-id/genesis/blob/main/engine/src/time/simulation_clock.rs): Canonically tracks monotonic tick progression.
- [`StableIdGenerator` (in agent)](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/resources.rs): Sequential agent identification counter.

### Active Components (Milestone 3 Approved)
- `ChunkCoord`: Attaches global chunk-space indices `(x, y)` to chunk entities.
- `TerrainChunk`: Stores cell-level variables (elevation, slope, water depth, soil depth, soil fertility) in flat, contiguous row-major vectors.
- `Generated`: Marker component identifying chunks that completed generation.
- `AgentMetadata` (in agent): Stores unique stable identifier for an agent.
- `AgentPosition` (in agent): Stores coordinate representation `WorldCoord`.
- `MetabolicStock` (in agent): Tracks agent age and current metabolic energy stocks.
- `ActionRequest` (in agent): Holds requests for movement actions.

### Active Events
- `WorldGenerationCompleted`: Signals that the startup generation schedule successfully finished and passed initial world validation.

### Active Schedules
Schedules are registered in Bevy's execution registry in canonical order:
1. `StartupGeneration`: Orchestrates validation, entity spawning, terrain generation, and completion signaling.
2. `FixedSimulationTick`: Advances simulation updates by exactly one tick.
3. `PostTickValidation`: Asserts world invariants after execution updates.
4. `PersistenceBoundary`: Handles snapshots of stable state.
5. `ObservationBoundary`: Extracts telemetry metrics for read-only layers.

---

## 3. Accepted ADR Index

All changes to Genesis must conform to the decisions recorded in the following Architecture Decision Records:
- [ADR-001: ECS Architectural Boundaries](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-001-ecs-architectural-boundaries.md)
- [ADR-002: Deterministic Execution Contract](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-002-deterministic-execution-contract.md)
- [ADR-003: Spatial Coordinate Model](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-003-spatial-coordinate-model.md)
- [ADR-004: Physical Time Model](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-004-physical-time-model.md)
- [ADR-005: World Generation Strategy](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-005-world-generation-strategy.md)

---

## 4. Architectural Constraints
- **Pressure-Driven Development:** Higher-level systems must never be implemented before lower-level environmental pressures exist.
- **Culture Neutrality:** Cultural, political, and economic rules must not be hardcoded; they must emerge solely from base environmental constraints.
- **Environmental Scope Limit:** Genesis is not a geology, weather, or physics simulator. Complex dynamics (such as erosion or storms) are out of scope unless required to create biological pressure.

---

## 5. Determinism Constraints
- **Seeded Randomness:** All entropy must branch from the root seed via coordinate-salted derivation. No thread-local or clock-based entropy is allowed.
- **Fixed Timestep:** Simulation ticks are discrete. Wall-clock timing cannot change outcomes.
- **Order-Independent Iteration:** Query iterations must be sorted or mathematically coordinate-independent to avoid hardware/compilation schedule differences.
- **Save/Load Equivalence:** Continual ticking must be binary-identical to loading and resuming.
- **Floating-Point Scope:** Genesis currently targets Local and x86_64 Platform Determinism. Universal Bit-Perfect Determinism is suspended for prototypes. Transcendental math (`sin`, `cos`, `pow`) is permitted if necessary (e.g., Gaussian Mutation), but must be documented with `@CROSS-PLATFORM-RISK`.

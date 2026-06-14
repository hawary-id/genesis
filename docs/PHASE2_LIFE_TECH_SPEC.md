# Phase 2 Life Technical Specification

## Status

This document freezes the Phase 2 Life architecture before Rust implementation begins.

It builds upon the completed Phase 1 Environmental Substrate and aligns with:
- `VISION.md`
- `PRINCIPLES.md`
- `ROADMAP.md`
- `ARCHITECTURE_BASELINE.md`
- `ARCHITECTURE_DECISIONS.md`
- [ADR-001: ECS Architectural Boundaries](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-001-ecs-architectural-boundaries.md)
- [ADR-002: Deterministic Execution Contract](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-002-deterministic-execution-contract.md)
- [ADR-003: Spatial Coordinate Model](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-003-spatial-coordinate-model.md)
- [ADR-004: Physical Time Model](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-004-physical-time-model.md)
- [ADR-005: World Generation Strategy](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-005-world-generation-strategy.md)

---

## Purpose

Phase 2 introduces the biotic layer of Project Genesis. It models autonomous biological agents experiencing environmental pressures from the Phase 1 substrate (Terrain, Climate, Resources, Energy) and undergoing metabolic decay, movement, and survival limits.

---

## Scope

### In Scope
- **Agent Data Responsibilities:** Formulating the categories of data associated with agent entities.
- **Environmental Sensing:** Reading cell values (nutrients, fresh water, temperature) from Phase 1 environmental chunks based on agent coordinates.
- **Metabolic Ticking:** Simulating energy decay and aging progression on simulation ticks, modified by environmental conditions and behavioral steps.
- **Grid-based Movement:** Transitioning agent positions to neighboring cells, validated against world boundaries, slope thresholds, and water limits.
- **Agent Death:** Processing removal from the active living population when energy is exhausted or age limits are exceeded.
- **State Validation:** Verifying agent state consistency, bounds, and counts inside the validation schedule.
- **State Persistence:** Extending snapshots to serialize and deserialize agent entities.

### Out of Scope
- **Genetic Encoding:** Genomic arrays, reproduction, mutation, and inheritance (Phase 3).
- **Neural Network Decisions:** Sensory buffer networks, learning models, and cognitive decision trees (Phase 4).
- **Physical Collision:** Hard coordinate blocks preventing multiple agents from occupying the same grid cell.
- **Client Rendering:** Agent sprites, animations, or UI rendering configurations.

---

## Technical & Logical Boundaries

### Data Ownership & Responsibilities
*   **Environmental Autonomy:** Agent systems are strictly read-only relative to Phase 1 environmental chunks. Agents cannot directly mutate elevation, slope, climate, resources, or clock states.
*   **Agent Autonomy:** Agent state (energy, age, coordinate) is owned strictly by individual agent entities. Environmental systems cannot directly modify agent components.
*   **Event Communication:** Transition boundaries (e.g., agent death, agent spawning) must use Bevy ECS event signals to notify telemetry or logging systems, preserving decoupled boundaries.

### Coordinate Mapping
*   To interface with the chunk-based spatial model (ADR-003), systems must translate world-space coordinate values to chunk-space indices and local-cell offsets using a standardized coordinate translation interface, preserving spatial boundaries without duplicating indexing calculations.

---

## Evaluated Options & Decisions

### 1. Agent Representation

#### Option A: Agents as Data Elements inside Chunk Components
Agents are stored as nested structures inside the existing chunk components.

*   **Rationale:** Keeps all spatial state contiguous in memory.
*   **Tradeoffs:** Chunk serialization becomes highly complex. Spawning agents and removing them from the active living population requires shifting vector positions. Bypasses Bevy's native query and entity-archetype filtering.
*   **Future Implications:** Adding unique behavior fields to specific agent classes would require modifying chunk definitions.

#### Option B: Agents as Discrete ECS Entities
Each agent is spawned as a unique ECS entity carrying specialized data components.

*   **Rationale:** Aligns with ECS boundaries (ADR-001) and Bevy idioms. Leverages Bevy's optimized query filters, entity lifetime management, and archetype storage.
*   **Tradeoffs:** System queries must fetch positions and match them to chunk entities via coordinate translations.
*   **Future Implications:** Future genomic mutations and neural networks can be attached as optional components without modifying chunk structures.

#### Decision
**Option B (Agents as Discrete ECS Entities).** This preserves architectural flexibility and allows the engine to scale logically as agent behaviors differentiate.

---

### 2. Movement Model

#### Option A: Continuous Movement
Agents carry floating-point positions and move continuously across chunk domains.

*   **Rationale:** Avoids grid-alignment artifacts.
*   **Tradeoffs:** Requires continuous spatial indexing structures to check neighbor distances and environmental values. Increases calculation cost and float divergence risks.
*   **Future Implications:** Introduces physics engine complexity prematurely.

#### Option B: Grid-Cell Movement
Agents carry integer coordinates and step discretely between adjacent cells.

*   **Rationale:** Aligns with the Phase 1 chunked square grid (ADR-003). Neighbors are indexed deterministically using integer offsets, avoiding continuous spatial lookup costs.
*   **Tradeoffs:** Movement displays directional grid bias.
*   **Future Implications:** Simplifies pathfinding, perception bounds, and deterministic save/load checks.

#### Decision
**Option B (Grid-Cell Movement).** The model maintains spatial parity with the environmental substrate and ensures reproducible step resolution.

---

### 3. Sensory Model

#### Option A: Local Cell Sensing
Agents can only perceive environmental resources in the cell they currently occupy.

*   **Rationale:** Low query cost. No neighbor lookups required.
*   **Tradeoffs:** Agents cannot locate distant resources, forcing them to move randomly to find food, which reduces survival rates.
*   **Future Implications:** Prevents pathfinding, spatial tracking, or memory map implementations.

#### Option B: Neighborhood Sensing
Agents manage perception-related state within a configured spatial cell radius.

*   **Rationale:** Provides directional resource gradients, generating biological pressure. Keeps query limits bounded.
*   **Tradeoffs:** Incurs coordinate lookup overhead for surrounding cells.
*   **Future Implications:** Allows future neural network components to receive structured environmental input arrays.

#### Decision
**Option B (Neighborhood Sensing).** Agent sensing systems must query environmental chunk components within a bounded spatial neighborhood centered on the agent's location, rather than querying the entire world substrate, to preserve performance.

---

### 4. Metabolism Energy Decay

#### Option A: Flat Hourly Decay
Agents lose a fixed quantity of energy every simulation tick.

*   **Rationale:** Simple, predictable, and trivial to validate.
*   **Tradeoffs:** Decoupled from Phase 1 climate, terrain, or movement choices. The abiotic environment exerts no physical survival pressure.
*   **Future Implications:** Fails the primary philosophy of building emergent behavior from base environmental pressures.

#### Option B: Environmental & Behavioral Modifiers
Energy loss is calculated as a base rate plus penalties for temperature extremes and movement steps taken.

*   **Rationale:** Directly links agent survival to physical substrate conditions. Climate changes and slope friction become existential pressures.
*   **Tradeoffs:** Requires coordinate lookups to fetch local temperature and slope values on every tick.
*   **Future Implications:** Drives evolution toward metabolic efficiency and environmental specialization.

#### Decision
**Option B (Environmental & Behavioral Modifiers).** Metabolic decay rates must incorporate environmental modifiers derived from local climate or terrain layers and behavioral actions such as movement steps, directly translating abiotic conditions into biological pressures.

---

## Data Ownership & Functional Responsibilities

Agent data must be modularly structured to satisfy the following functional responsibilities:

*   **Spatial Location Responsibility:** The agent entity must carry state identifying its location in grid-cell coordinates, satisfying the spatial model boundaries.
*   **Metabolic State Responsibility:** The agent entity must track internal energy level stocks and elapsed chronological lifespan metrics.
*   **Sensory State Responsibility:** The agent entity must manage perception-related state within its neighborhood.
*   **Stable Identity Responsibility:** The agent entity must maintain stable identifiers necessary for persistence, determinism, and future compatibility.

---

## System Ownership & Scheduling

Agent behavior logic is executed by pure systems scheduled in explicit sequences.

### Simulation Schedule Integration
*   **Schedule Registration:** Agent update logic must execute within the `FixedSimulationTick` schedule, ordered after environmental substrate updates are complete.
*   **Sensing Sequencing:** Environmental sensing logic must execute before metabolic or behavioral actions are processed, ensuring choices rely on valid local state.
*   **Action Execution Sequencing:** Behavioral actions (like movement) and metabolic consumption must execute sequentially to prevent race conditions or duplicate evaluations.
*   **Removal Sequencing:** Agent lifetime termination processing must occur at the end of the tick cycle, ensuring agents removed from the active living population do not participate in validation or persistence.

---

## Validation Requirements (`PostTickValidation`)

The validation system must execute safety checks at each tick boundary:
*   **Entity Count Sanity:** The active agent count must equal the total number of entities carrying the agent metadata component.
*   **Boundary Enforcement:** Every agent's coordinate position must reside within configured world limits.
*   **Thermal Safety:** Agent energy values must remain within configured bounds. Any agent whose energy is exhausted must be removed from the active living population by the end of the tick.
*   **Coordinate Consistency:** Agent coordinates must map to valid chunk locations.

---

## Determinism Requirements

*   **Entropy Seeding:** All agent-specific stochastic behavior must branch from seed state linked to the agent's stable identifier and the current clock tick, preventing synchronization anomalies across separate entities or threads.
*   **Stable Snapshot Sort:** Serialized agent entities must be ordered by a stable, unique identifier ascending before writing to the snapshot file, ensuring output is independent of ECS query iteration or entity memory allocation order.
*   **Bit-Perfect Float Asserts:** Validation of agent float values in determinism checks must use strict bitwise assertions to protect replication parity.

---

## Future Compatibility Requirements

*   **Parameter Abstraction:** All update thresholds and rates must be loaded as external configuration parameters, allowing future phases to dynamically modify metabolic rates or sensor bounds without altering execution code.
*   **Decision-Execution Separation:** Movement execution responsibilities must remain decoupled from decision-generation responsibilities, allowing future phases to replace decision-generation mechanisms without modifying movement execution systems.

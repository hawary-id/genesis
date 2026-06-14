# Phase 2 Implementation Plan

## Purpose

This document turns `PHASE2_LIFE_TECH_SPEC.md` into a step-by-step engineering roadmap for Phase 2 (Life) of Project Genesis. It defines the implementation sequence, data responsibilities, configuration parameters, milestone deliverables, and verification tests needed to build a deterministic agent simulation layer.

---

## Pre-Implementation Decisions

The following decisions resolve design details left open by the technical specification.

### 1. Numeric Representation
*   **Continuous Stocks:** Continuous variables (such as energy or modifiers) must use continuous representations, while age metrics, grid indices, and ticks must use discrete representations, maintaining alignment with Phase 1 types.

### 2. Stable Identifier Representation
*   **Decision:** Agent entities carry a stable identifier assigned upon spawning from a deterministic, unique identifier generation resource.
*   **Rationale:** Sequentially assigned identifiers require minimal storage and permit efficient sorting, protecting execution speeds.
*   **Persistence:** The identifier generator state must be persisted in snapshots to prevent identifier collisions after loading a saved state.

### 3. Action Request Interface
*   **Decision:** Agent entities carry action request values representing the agent's behavioral intent, separating action selection from movement step translation.
*   **Rationale:** Decouples movement calculations from action generation. Phase 2 will implement a simple procedural system to write direction choices; future phases can replace this system with neural network outputs without modifying coordinate translation or constraint validation.

### 4. Snapshot Format Expansion
*   **Decision:** The snapshot serialization schema must be extended to include agent state components.
*   **Ordering:** Prior to writing, agent lists must be sorted ascending by their stable identifier to guarantee binary-identical snapshot outputs.

---

## Configuration Parameters

The configuration resource must be extended to support the following configurable parameters:

*   **Thermal Optimum:** Baseline temperature optimum at which metabolic decay is minimized.
*   **Base Decay Rate:** Base metabolic energy consumption per simulation tick.
*   **Behavioral Modifiers:** Energy penalties applied for actions such as movement.
*   **Age Limits:** Chronological threshold after which agents are removed from the active living population.
*   **Agent Density Cap:** Maximum active agent population size.
*   **Sensing Radius:** Spatial cells distance defining the neighborhood sensing bounds.

---

## Implementation Roadmap

### Milestone 11: Agent Data Foundation & Spawning [COMPLETED]
*   **Objective:** Register the agent data components and implement stable spawning and initialization routines.
*   **Deliverables:**
    - Register components satisfying the agent data responsibilities (metadata, coordinates, metabolic stocks, and action requests).
    - Implement a unique stable identifier generation resource.
    - Build a spawning system capable of initializing agents at configured coordinates with specified initial energy levels.
*   **Dependencies:** Milestone 10 (Determinism Testing baseline).
*   **Success Criteria:**
    - App compiles successfully.
    - Unit tests confirm that spawning an agent assigns a unique stable identifier.
    - Queries retrieve coordinates and metadata values correctly.
*   **Risks:** ECS query iteration sorting could compromise initial test assertions if identifiers are not tracked deterministically.

### Milestone 12: Environmental Sensing Query API [COMPLETED]
*   **Objective:** Implement coordinate-mapping queries translating agent locations to environmental chunk cells.
*   **Deliverables:**
    - Standardize a coordinate mapping utility converting world coordinate values to chunk indices.
    - Build a sensory system that reads local resource values (nutrients, fresh water) in the agent's spatial neighborhood.
*   **Dependencies:** Milestone 11.
*   **Success Criteria:**
    - Unit tests confirm that sensory queries return cell values matching the corresponding chunk's environmental vectors.
    - Attempting queries outside of configured world bounds is validated as out-of-bounds.
*   **Risks:** Complex neighborhood searches can introduce performance overhead if executed per-agent every tick.

### Milestone 13: Metabolic Tick Systems [COMPLETED]
*   **Objective:** Implement metabolic energy decay and lifecycle age tracking.
*   **Deliverables:**
    - Build a metabolism system scheduled in the simulation tick schedule that decrements energy based on base rates and environmental modifiers, and increments age.
    - Build a termination system scheduled at the end of the tick cycle that removes entities from the active living population if energy is exhausted or age limits are exceeded.
*   **Dependencies:** Milestone 12.
*   **Success Criteria:**
    - Unit tests verify that agents age monotonically by tick increments.
    - Energy depletion scales correctly under environmental extremes.
    - Agents whose energy is exhausted or whose age exceeds configured limits are successfully removed from the active population before tick validation occurs.
*   **Risks:** Retaining dead agents past the tick boundary can cause validation invariant panics.

### Milestone 14: Spatial Movement Execution [COMPLETED]
*   **Objective:** Implement grid step transitions and constraint validation.
*   **Deliverables:**
    - Build a movement system that reads direction choices, translates positions, and validates destinations against world boundaries, slope limits, and water barriers.
    - Apply energy step costs to metabolic stocks upon successful movement.
*   **Dependencies:** Milestone 13.
*   **Success Criteria:**
    - Movement systems successfully block agents from crossing configured boundaries, climbing excessive slopes, or stepping into deep water.
    - Position values update correctly on valid steps, and energy is decremented by behavioral modifiers.
*   **Risks:** Multi-threaded system sequencing could allow agents to move during validation; explicit sequencing constraints must block this.

### Milestone 15: Persistence & Integration Testing [COMPLETED]
*   **Objective:** Extend snapshot saving/loading to handle agent entities and verify save/load equivalence.
*   **Deliverables:**
    - Extend serialization systems to write sorted agent state data to snapshots.
    - Extend world loading systems to reconstruct agent entities, positions, and metabolic values.
    - Build integration tests verifying A+B=N execution equivalence with active agent populations.
*   **Dependencies:** Milestone 14.
*   **Success Criteria:**
    - Snapshots with agents are written and read successfully, passing binary float checks.
    - Integration tests confirm that running a world with agents continuously matches split save/load executions.
*   **Risks:** ECS entity reallocations on load could break stable identification if query arrays are not sorted correctly by identifier values.

---

## Phase Completion Criteria

1.  **Bit-Perfect Replay:** Running the integration test suite with active agents verifies complete execution determinism across separate runs.
2.  **Lint & Style Compliance:** Code compiles with zero warnings under Clippy:
    ```bash
    cargo clippy -- -D warnings
    ```
3.  **Clean formatting:** Code conforms to standard formatting:
    ```bash
    cargo fmt --check
    ```
4.  **No Document Drift:** Active architecture baselines and directory maps are updated to reflect all added Phase 2 components, systems, and structures.

---

## Phase Exit Conditions

*   All Phase 2 implementation code is committed to the primary development branch.
*   The test suite executes and passes cleanly:
    ```bash
    cargo test
    ```
*   The repository is tagged with a release marker:
    ```text
    v0.2.0-phase2
    ```

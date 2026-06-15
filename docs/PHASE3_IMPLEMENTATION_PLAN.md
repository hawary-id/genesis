# Phase 3 Evolution Implementation Plan

## Purpose

This document provides a step-by-step engineering roadmap for Phase 3 (Evolution) of Project Genesis. It defines the implementation sequence, components, systems, tests, and exit conditions to implement a deterministic genetics, mutation, and selection model, building on the completed Phase 2.

---

## Part 1 — Milestone Roadmap

Based on the architecture constraints of Genesis, the implementation is partitioned into six sequential, independently testable milestones. 

---

### Milestone 16: Genetics and Phenotype Mapping [COMPLETE]
*   **Goal:** Introduce the genetic representation and map continuous gene vectors to concrete phenotypic traits.
*   **Scope:** 
    *   Create the `Genome` component storing genes in a `Vec<f32>` vector for long-term future extensibility.
    *   Create the `LineageMetadata` component (stores `parent_id: Option<u64>` and `generation: u32` initialized to `None` and `0` for startup spawned agents).
    *   Configure trait mapping boundaries in `GenomeConfig`.
    *   Implement the cached `Phenotype` component, derived upon spawn.
    *   Upgrade the serialization schema to Version 3 to support genome and lineage persistence.
*   **New Components:**
    *   `Genome` (stores `Vec<f32>` genes).
    *   `Phenotype` (cached derived traits).
    *   `LineageMetadata` (stores `parent_id: Option<u64>` and `generation: u32`).
*   **New Resources:**
    *   `GenomeConfig` (defines trait range limits).
*   **New Systems:**
    *   `derive_phenotype_on_spawn` (derives `Phenotype` when `Genome` is added).
*   **Snapshot Changes:**
    *   Upgrade `WorldSnapshot` schema to version `3`.
    *   Extend `AgentSnapshot` with the `Genome` struct (storing `Vec<f32>`) and `LineageMetadata` fields.
    *   Load path reconstructs phenotypes and attaches lineage. If loaded vector size < compile-time trait count, pads trailing genes with default values.
*   **Tests Required:**
    *   Unit tests checking mapping bounds (e.g., verifying that a gene of $0.5$ maps exactly to the midpoint of the trait's range).
    *   Serialization round-trip test asserting that genomes and lineage are persisted and restored bit-perfectly.
*   **Exit Criteria:** Code compiles, genomes and lineage metadata save and load successfully, and all mapping unit tests pass.

---

### Milestone 17: Resource Consumption (Eating & Drinking) [COMPLETE]
*   **Goal:** Establish the metabolic energy acquisition loop.
*   **Scope:**
    *   Allow agents to consume nutrients and fresh water at their coordinates.
    *   Deduct consumed resources from chunk arrays and add energy to the agent's stock.
    *   Apply dietary phenotype modifiers (herbivore vs hydrator preference).
*   **New Components:** None.
*   **New Resources:** None.
*   **New Systems:**
    *   `process_agent_consumption` (runs after sensing, before movement).
*   **Snapshot Changes:** None.
*   **Tests Required:**
    *   Unit test confirming local chunk resource depletion and corresponding agent energy increase.
    *   Unit test verifying diet preference scaling (e.g., nutrient-optimum agents gain more energy from nutrients than water).
*   **Exit Criteria:** Consumption updates resources and energy deterministically, respecting cell limits and mass conservation.

---

### Milestone 18: Reproduction, Inheritance, and Lineage [COMPLETE]
*   **Goal:** Enable population continuity via deterministic asexual reproduction and lineage tracking.
*   **Scope:**
    *   Check birth requirements (energy threshold and maturity age bounds).
    *   Implement asexual splitting, dividing energy stock 50/50.
    *   Trace lineage by populating the `LineageMetadata` component with the parent's stable ID and generation depth + 1.
    *   Select adjacent coordinates in a fixed cardinal order (N -> S -> E -> W). If all directions are blocked by slope/water/boundary limits, reproduction is canceled.
    *   Sort reproduction requests by parent stable ID ascending before spawning to guarantee order-independence.
*   **New Components:** None.
*   **New Resources:** None.
*   **New Systems:**
    *   `process_agent_reproduction` (runs after movement/consumption).
*   **Snapshot Changes:**
    *   Reconstructed loaded worlds accommodate variable agent counts and deserialize lineage data.
*   **Tests Required:**
    *   Unit test verifying parent energy is halved and transferred to offspring.
    *   Unit test verifying deterministic order-independent spawning by sorting parent list.
    *   Unit test verifying reproduction cancellation when cardinally trapped (mountain/deep water).
    *   Unit test asserting lineage assignment (`parent_id == parent.id`, `generation == parent.generation + 1`).
*   **Exit Criteria:** Deterministic reproduction processes births, assigns lineage, handles adjacent limits without overlapping fallbacks, and populations survive under abundant resource conditions.

---

### Milestone 19: Mutation and Deterministic Drift [COMPLETE]
*   **Goal:** Introduce genetic diversity through seeded mutations.
*   **Scope:**
    *   Implement Gaussian gene mutation during reproduction.
    *   Derive mutation seeds deterministically using parent metadata (stable ID, coordinates) and current ticks, satisfying ADR-002.
    *   **Drift Emergence:** Genetic drift is a completely emergent phenomenon resulting from mutation and spatial carrying capacity pressures. No dedicated drift subsystem or drift-related systems are implemented.
*   **New Components:** None.
*   **New Resources:** None (`mutation_rate` and `mutation_step_size` belong to `WorldConfig` to consolidate global simulation parameters and avoid introducing redundant resources in compliance with ADR-001).
*   **New Systems:** None (integrated as pure helper functions inside the reproduction system in `systems.rs`).
*   **Snapshot Changes:** None.
*   **Tests Required:**
    *   Unit tests confirming mutation determinism given a fixed seed.
    *   Unit tests verifying mutation limits (genes are finite and clamped exactly within `[0.0, 1.0]`).
    *   Save/load equivalence integration tests confirming that mutation-induced drift does not break A+B=N determinism.
*   **Exit Criteria:** Offspring exhibit mutation drift, and split run equivalence holds bit-perfectly.

---

### Milestone 20: Natural Selection and Adaptation
*   **Goal:** Validate adaptation to environmental gradients.
*   **Scope:**
    *   Integrate phenotypes into metabolic decay, movement costs, and navigation barriers.
    *   Verify that agents with unfavorable traits die, while adaptive traits survive.
*   **New Components:** None.
*   **New Resources:** None.
*   **New Systems:**
    *   Updated `update_agent_metabolism` and `process_agent_movement` to query `Phenotype` parameters instead of global configuration.
*   **Snapshot Changes:** None.
*   **Tests Required:**
    *   **Climate Adaptation Test:** Place agents in a cold environment. Verify that agents with lower thermal optimum genes survive and reproduce, while high thermal optimum agents die out.
    *   **Terrain Specialization Test:** Place agents near steep terrain. Verify that agents with higher slope limits climb and access resources, while other agents remain trapped or starve.
*   **Exit Criteria:** Simulation runs demonstrate trait distributions shifting over generations towards higher environmental fitness.

---

### Milestone 21: Evolution Diagnostics and Validation
*   **Goal:** Implement telemetry monitoring and fix snapshot validation bugs.
*   **Scope:**
    *   Collect evolutionary metrics (mean traits, gene frequencies, standard deviations).
    *   Update startup and tick validation systems to support loaded worlds with variable ticks, agent counts, and agent age/energy distributions.
*   **New Components:** None.
*   **New Resources:**
    *   `PopulationStats` (statistical registry of the active population).
*   **New Systems:**
    *   `gather_population_statistics` (runs in `ObservationBoundary`).
    *   Updated `validate_world_on_startup` and `validate_world_on_tick` (disables hardcoded initial spawning assertions for tick > 0 loaded snapshots).
*   **Snapshot Changes:** None.
*   **Tests Required:**
    *   Unit tests verifying statistics calculation accuracy.
    *   Integration test verifying that loading a snapshot at tick > 0 does not trigger startup validation failures.
*   **Exit Criteria:** All validation and statistics systems compile cleanly, and standard test suites pass.

---

## Part 2 — Future Compatibility

The Phase 3 genomic and selection architecture is designed to support long-term milestones (Phase 4 to Phase 12) without requiring core rewrites:

*   **Social Behavior (Phase 7):** Genomes can be extended with social genes (e.g., altruism threshold, cooperation preference). These traits will modify action intents when agents interact.
*   **Tribes & Settlements (Phase 7/12):** Genetic proximity (calculated via genomic distance) provides a natural mechanism for kin selection. Agents can recognize related agents to form tribes, share resources, and establish settlements.
*   **Culture (Phase 8):** Cultural norms can be modeled as memes that interact with genetic preferences. For example, a genetic trait for learning speed or tradition retention will control how quickly cultural practices propagate.
*   **Economy (Phase 10):** Specialized dietary genes (e.g., carnivores vs herbivores) or resource gathering efficiencies create natural trade pressures. If one group gathers water more efficiently while another harvests nutrients, barter and markets will emerge.
*   **Civilization (Phase 12):** Institutional structures (diplomacy, laws, governance) will emerge to regulate conflicts arising from genetic or cultural differences, satisfying the pressure-driven design of Genesis.

---

## Part 3 — Critical Review

### 1. Architectural Risks

*   **Risk:** Direct genetic mutation logic inside systems.
*   **Impact:** If mutation logic is directly embedded in Bevy systems, testing individual mutation algorithms requires spinning up a full Bevy world, creating code rigidity.
*   **Mitigation:** Write all mutation, crossover, and trait mapping logic as pure, non-ECS helper functions. The ECS reproduction system will act solely as an orchestrator calling these pure functions.

---

### 2. ECS Bottlenecks

*   **Risk:** Linear scan complexity on chunk lookups.
*   **Impact:** Consumption and metabolism systems query chunk resources on every tick. Currently, this performs a linear search over chunk entities ($O(\text{agents} \times \text{chunks})$), which will bottleneck performance as populations grow.
*   **Mitigation:** Replace linear chunk scans with an indexed coordinate lookup resource (e.g., `ChunkLookupMap` storing `HashMap<ChunkCoord, Entity>`), reducing lookup time to $O(1)$ per agent.

---

### 3. Determinism Risks

*   **Risk:** Order-dependent reproduction and collision resolution.
*   **Impact:** If multiple agents reproduce on the same tick, the order in which offspring are spawned could dictate which agent gets adjacent cells. If query iteration order is unstable, execution determinism breaks.
*   **Mitigation:** Sort all reproduction requests by the parent's stable ID ascending before spawning offspring entities, ensuring spatial cell allocation is invariant to ECS scheduling.

---

### 4. Persistence Risks

*   **Risk:** Schema version incompatibilities on custom genomes.
*   **Impact:** Future expansions that add new genes will break compatibility with older version 3 snapshots.
*   **Mitigation:** Store genomes as a flexible vector of floats (`Vec<f32>`) in snapshots, with a `version` identifier. If a loaded snapshot has a shorter genome vector, apply default values for the missing genes.

---

### 5. Scalability & Population Explosion Risks

*   **Risk:** Population explosions saturating CPU execution pipelines.
*   **Semantic Choice:** Carrying capacity is an **emergent phenomenon** driven by resource scarcity and metabolic pressure, not hard-coded limits.
*   **Carrying Capacity Mechanisms:**
    *   **Resource Depletion:** High agent densities exhaust local cell nutrients and fresh water faster than they replenish, driving natural starvation deaths.
    *   **Reproduction Division Penalty:** Every birth halves the parent's metabolic stock, requiring the parent to forage and recover energy before reproducing again.
*   **Mitigation (Emergency Safety Guard):**
    *   Set the `agent_density_cap` high (e.g., 5,000) so that it acts strictly as a hardware crash safeguard. The simulation should naturally stabilize far below this threshold via environmental constraints and metabolic pressures. If the emergency cap is hit, reproduction is temporarily halted.

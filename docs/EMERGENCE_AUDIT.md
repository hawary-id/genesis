# Genesis Emergence Audit Guidelines

## Purpose
This document establishes the audit framework for evaluating and validating emergent behaviors within the Genesis simulation engine. It translates the core principles of the Genesis Constitution into operational guidelines to protect the scientific integrity of the simulation. This framework is designed to remain valid across all development phases, independent of specific programming languages, software architectures, or data structures.

---

## Core Axiom: Non-Guaranteed Emergence
Genesis does not guarantee the emergence of any specific macro-level phenomenon.
* Trade may never emerge.
* Society may never emerge.
* Culture may never emerge.
* Language may never emerge.
* Technology may never emerge.
* Civilization may never emerge.

The absence of a desired phenomenon is a valid scientific simulation outcome. Developers must never modify systems solely to force or guarantee a pre-determined emergent result. If the environmental pressures and adaptive unit interaction rules do not naturally give rise to a behavior, the correct response is to analyze and document the limiting factors, not to script shortcuts.

---

## Definition of Emergence
In Genesis, **Emergence** is the spontaneous appearance of macro-level organizational structures, behavioral patterns, or spatially organized networks that arise strictly from micro-level constraints (e.g., operational depletion, local sensing ranges, physical terrain boundaries) without macro-level scripting or explicit coordination templates.

We distinguish emergence from other development concepts:
* **Emergence:** Bottom-up macro patterns arising from simple micro-level rules (e.g., adaptive divergence emerging purely from spatial limits).
* **Automation:** Systematic execution of static tasks that do not dynamically adapt (e.g., resource replenishment cycles).
* **Scripting:** Pre-programmed procedural templates prescribing specific adaptive unit behaviors under conditions (e.g., pathfind to nearest resource when energy is low).
* **Hardcoding:** Presetting structures, groups, or rules directly in code (e.g., assigning adaptive units to predefined group structures at spawn).
* **Optimization:** Pure local mathematical minimization of individual goals without social or spatial interaction (e.g., pathfinding utilizing static traversal costs).

---

## The Anti-Teleology Rule
Developers are strictly prohibited from implementing teleological reward mechanisms that steer adaptive unit behaviors toward desired emergent outcomes.
* Execution logic must not reward adaptive units simply because they perform a targeted behavior.
* Execution logic must not inject artificial utility bonuses, resource adjustments, or survival modifiers to encourage social or trade interactions.
* All state changes (e.g., resource stock changes, lifetime updates, or behavioral selections) must arise strictly from conservation rules and local physics constraints, never from developer intent.

---

## Emergence Classification System
To track development progress, the engine classifies emergence along five abstract levels:

1. **Level 0: No Emergence:** Static adaptive units, random walks, or hardcoded behavioral scripts.
2. **Level 1: Substrate & Adaptive Emergence:** Adaptive divergence and specialization along environmental gradients.
3. **Level 2: Cognitive & Behavioral Emergence:** Dynamic trajectory planning and sensing optimization based on individual historical records and internal states.
4. **Level 3: Social & Economic Emergence:** Coordinated actions, localized resource exchanges, and symbolic signal correlation.
5. **Level 4: Civilizational Emergence:** Territorial zones, permanent spatial modifications, and localized rule enforcement.

---

## Emergence Validation Framework
Every development phase must pass specific validation criteria to confirm true emergence:

### 1. Memory (Phase 4)
* **What Counts:** Adaptive units modifying search trajectories or spatial selections based on stored internal representations of past environmental or social interactions.
* **What Does NOT Count:** Global pathfinding memory structures or shared state oracles that feed data to adaptive units.
* **Observable Indicators:** Path divergence from random walks, clustering around historically favorable spatial partitions.
* **Failure Indicators:** Adaptive units accessing spatial data outside their historical trajectories.

### 2. Decision (Phase 5)
* **What Counts:** Adaptive unit action selection that varies dynamically based on changes in internal state in response to environmental inputs.
* **What Does NOT Count:** Procedural behavioral trees or hardcoded script paths.
* **Observable Indicators:** Multi-dimensional action distributions across age, energy, and environmental gradients.
* **Failure Indicators:** Uniform behavioral patterns across all adaptive units regardless of state or phenotype.

### 3. Knowledge (Phase 6)
* **What Counts:** Updatable parameters representing generalized spatial or environmental values derived from experience.
* **What Does NOT Count:** Shared semantic databases or hardcoded ontologies.
* **Observable Indicators:** Transmission lineages showing higher survival efficiency over selective cycles without adaptive divergence.
* **Failure Indicators:** Static knowledge structures that never mutate or update based on simulation events.

### 4. Society (Phase 7)
* **What Counts:** Differentiated behaviors toward other adaptive units based on kinship markers, historical interactions, or local group classifications.
* **What Does NOT Count:** Preset spawn-time assignments of adaptive units to predefined group templates.
* **Observable Indicators:** Spatial grouping and cooperative defense zones.
* **Failure Indicators:** Alliances or coordination formed between adaptive units who have never occupied neighboring spatial partitions.

### 5. Culture (Phase 8)
* **What Counts:** Non-genetic behavioral preferences diffusing horizontally across spatial boundaries.
* **What Does NOT Count:** Hardcoded cultural labels or preset regional tags.
* **Observable Indicators:** Sharp geographical clustering of behavioral traits forming distinct boundaries.
* **Failure Indicators:** Uniform trait distributions across the entire simulation space.

### 6. Language (Phase 9)
* **What Counts:** Mutual information increases between adaptive unit communicative outputs (signals) and receiver unit behaviors.
* **What Does NOT Count:** Pre-programmed token translation dictionaries.
* **Observable Indicators:** Consistent behavioral alignment following signal emissions.
* **Failure Indicators:** Invariant responses to signal emissions without training or evolutionary loops.

### 7. Economy (Phase 10)
* **What Counts:** Localized resource exchange rates stabilizing around resource scarcity ratios.
* **What Does NOT Count:** Central auction clearers or predefined currency values.
* **Observable Indicators:** Resource swaps occur locally between adjacent adaptive units; exchange rates vary with local scarcity.
* **Failure Indicators:** Exchange transactions occurring across non-adjacent spatial boundaries.

### 8. Technology (Phase 11)
* **What Counts:** Adaptive unit adaptations or tool usage that modifies resource extraction, traversal, or resource conversion rates beyond substrate baseline constraints.
* **What Does NOT Count:** Unlocking tool states via pre-programmed recipe trees or update timelines.
* **Observable Indicators:** Step-changes in carrying capacity without alterations to global configuration constants.
* **Failure Indicators:** Pre-programmed technology unlocking events.

### 9. Civilization (Phase 12)
* **What Counts:** Spatial modification and enforcement zones mapping to local group boundaries.
* **What Does NOT Count:** Hardcoded cities, laws, or governments.
* **Observable Indicators:** Paths align along modified terrain; spatial boundaries restrict unauthorized adaptive units.
* **Failure Indicators:** Enforcing rules globally without spatial unit activity or operational costs.

---

## Fake Emergence Detection

Reviewers must identify and reject these common over-engineering failure modes:

### 1. Information Oracles
* **Danger:** Systems bypass spatial constraints, allowing adaptive units to act on global data.
* **Detection:** Check if state lookups query variables of distant entities without spatial neighborhood distance checks.
* **Prevention:** Clamp sensory queries strictly to local spatial boundaries.

### 2. Physical Oracles
* **Danger:** Simulation steps bypass physical coordinates, teleporting actions or resources.
* **Detection:** Resources or adaptive units move between non-adjacent spatial partitions in a single update step without traversing intermediate coordinates.
* **Prevention:** Enforce physical adjacency constraints on all movement and transaction updates.

### 3. Outcome-Driven Design
* **Danger:** The engine is programmed to force a specific outcome, bypassing biological adaptation.
* **Detection:** The simulation fails or crashes if adaptive units do not cooperate or establish trade.
* **Prevention:** Maintain value-neutral rules. If adaptive units fail to adapt and die out, this is a valid simulation outcome.

---

## Architectural Audit Checklist

Before accepting any milestone:

- [ ] **Did this change create conditions or outcomes?**
  * *Requirement:* Verify the code only defines substrate-level rules. If it defines the final behavior or social structure, reject.
- [ ] **Is the behavior reproducible?**
  * *Requirement:* Verify that execution replication comparison tests pass, yielding identical outcomes across independent runs.
- [ ] **Is the behavior pressure-driven?**
  * *Requirement:* Prove the behavior only occurs when operational depletion, scarcity, or density limits demand it.
- [ ] **Could this result have been hardcoded?**
  * *Requirement:* If the outcome was pre-determined by structural templates or scripts, reject.

---

## Evidence Standards

To prove that a phenomenon has emerged, developers must provide:
1. **Repeated Observation:** The phenomenon must occur across multiple independent simulation updates in a run.
2. **Multi-Seed Verification:** The behavior must appear across at least five distinct, randomly selected world seeds.
3. **Lineage Persistence:** The behavior must persist across at least ten selective cycles of replication.
4. **Pressure-Response Correlation:** Demonstrate that relaxing the pressure (e.g., setting resource replenishment to infinite) dissolves the emergent behavior.

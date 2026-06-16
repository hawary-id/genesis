# Genesis Forbidden Implementations

## Purpose
This document defines the implementation shortcuts, patterns, and designs that are permanently prohibited in Project Genesis. These "red lines" exist to protect the engine from fake emergence, outcome biasing, and structural design drift. Any implementation that violates a rule in this document must be rejected during review.

---

## Severity Classification
Violations of these guidelines are classified into three severity levels:

| Severity | Definition | Review Action |
|---|---|---|
| **Critical** | Violation of a Constitutional law or foundational emergence constraint. | Immediate rejection; system redesign required. |
| **High** | Creation of information or physical oracles that bypass spatial boundaries. | Rejection; review alternative designs. |
| **Medium** | Implementation of design shortcuts that limit the complexity of future phases. | Requires detailed human architectural review. |

---

## Scope of Examples
The examples and scenarios provided throughout this document are illustrative and meant for clarification. The prohibitions apply strictly to the underlying architectural, informational, and physical patterns. Any design, algorithm, or data structure that violates the same patterns is prohibited under this Constitution, regardless of whether it is explicitly cataloged as an example.

---

## Forbidden Categories

### 1. Outcome Injection
* **Description:** Programming adaptive units, resources, or environments to automatically transition into pre-determined target states (such as cooperative groups, cultural zones, or specialized roles).
* **Why It Is Dangerous:** It replaces emergent behavior with pre-scripted state transitions, making the simulation results scientifically invalid.
* **Constitutional Principles Violated:** Build Conditions, Not Outcomes.
* **Emergence Audit Violated:** Section 2: Emergence vs. Scripting/Hardcoding.
* **Examples (Forbidden):** Automatically setting adaptive units to "allied" status when their spatial boundaries overlap.
* **Acceptable Principle:** Adaptive unit coordination and social organizations must arise dynamically from the history of interactions between individuals.

### 2. Information Oracles
* **Description:** Allowing adaptive units to access, query, or act on environmental or social state variables outside their spatial sensing ranges or individual state bounds.
* **Why It Is Dangerous:** It removes the constraint of local observation, allowing adaptive units to react to events they cannot physically detect.
* **Constitutional Principles Violated:** Prefer the Simplest Plausible Model; Determinism First.
* **Emergence Audit Violated:** Section 7: Information Oracles.
* **Examples (Forbidden):** Querying a distant spatial partition's resource density variable without the adaptive unit traversing to or sensing that partition.
* **Acceptable Principle:** Adaptive units must act strictly on information acquired within their local observation and interaction boundaries.

### 3. Physical Oracles (Adjacency Bypassing)
* **Description:** Allowing resources, actions, or information signals to bypass spatial locations, executing instant state changes over non-adjacent spatial partitions in a single simulation update.
* **Why It Is Dangerous:** Bypassing spatial adjacency constraints eliminates physical carrying capacity and spatial traversal limits, creating artificial, teleological efficiencies.
* **Constitutional Principles Violated:** Build Conditions, Not Outcomes.
* **Emergence Audit Violated:** Section 7: Physical Oracles.
* **Examples (Forbidden):** Moving resource state quantities directly from a seller's state representation to a buyer's state representation when they occupy non-adjacent spatial locations.
* **Acceptable Principle:** All changes in simulation state and resource values must resolve through spatially contiguous transitions over discrete simulation updates.

### 4. Teleological Rewards
* **Description:** Injecting utility bonuses, energy adjustments, or survival modifiers to encourage desired emergent behaviors.
* **Why It Is Dangerous:** It steers evolutionary selection pressures artificially, forcing behaviors rather than letting them emerge from substrate constraints.
* **Constitutional Principles Violated:** Build Conditions, Not Outcomes; Pressure-Driven Development.
* **Emergence Audit Violated:** Section 4: The Anti-Teleology Rule.
* **Examples (Forbidden):** Giving adaptive units an operational depletion reduction simply because they are communicating or trading.
* **Acceptable Principle:** Adaptive unit updates and operational cost calculations must be computed strictly from physical attributes and environmental variables, completely independent of behavioral outcomes.

### 5. Global Coordination Systems
* **Description:** Implementing centralized managers, brokers, or global controllers that manage interactions between adaptive units or environments.
* **Why It Is Dangerous:** It introduces centralized control, violating bottom-up emergence and creating god-object scaling issues.
* **Constitutional Principles Violated:** Build Conditions, Not Outcomes.
* **Emergence Audit Violated:** Section 7: Information Oracles.
* **Examples (Forbidden):** Implementing a global broker system that matches buy/sell requests map-wide.
* **Acceptable Principle:** Interactions and resource exchanges must resolve locally and peer-to-peer, using information from surrounding spatial locations.

---

## Review Checklist
Before approving any code implementation, reviewers must ask:

- [ ] **Does this inject outcomes?**
  * *Verification:* Does the code force adaptive units into predefined states, groups, or paths?
- [ ] **Does this bypass pressure?**
  * *Verification:* Does the behavior function when operational depletion, scarcity, or carrying capacity limits are removed?
- [ ] **Does this create an oracle?**
  * *Verification:* Does the adaptive unit query spatial data outside its local sensor or state boundaries?
- [ ] **Does this reward desired behavior?**
  * *Verification:* Are there any state variables updated based on developer-intended behavioral milestones?
- [ ] **Does this predetermine success?**
  * *Verification:* Will the simulation crash or fail if adaptive units fail to develop the target behavior?

---

## Red Team Findings: 20 Prohibited Patterns

Reviewers must monitor and block the following 20 shortcuts during future phases:

### Phase 4 & 5: Memory & Decision
1. **Global navigation systems operating on unknown spatial locations:** Planning paths using information outside historical or local boundaries.
2. **Direct state copying bypassing spatial signaling:** Bypassing local communication limits to copy another entity's internal variables.
3. **Out-of-range threat perception:** Adaptive units reacting to threat states outside local detection ranges.
4. **Scripted behavior selection based on fixed conditionals:** Decision logic driven by fixed condition sequences rather than utility calculations.

### Phase 6 & 7: Knowledge & Society
5. **Centralized information ledgers sharing spatial data:** A single global resource sharing spatial data maps globally.
6. **Static group allocations driven by structural labels:** Spawning adaptive units with fixed structural groupings or alliances.
7. **Programmatic conflict constraints based on state propagation variables:** Enforcing cooperation or preventing conflicts programmatically based on state propagation variables.
8. **Static relationship configurations:** Social relations initialized with immutable cooperation values.

### Phase 8 & 9: Culture & Language
9. **Preset behavioral tags overriding dynamic preferences:** Tagging adaptive units with variables that override dynamic behavioral adaptation.
10. **Ideological behavioral rules scripted into data structures:** Pre-scripted behavioral rules or norms directly driving adaptive unit logic.
11. **Hardcoded signal translation maps:** Translating signal inputs directly to actions via fixed maps.
12. **Non-spatial broadcast channels bypassing local spatial boundaries:** Sending signal broadcasts globally bypassing spatial boundaries.

### Phase 10 & 11: Economy & Technology
13. **Centralized transaction matching managers:** Centralized matching algorithms resolving resource trade requests.
14. **Non-adjacent state swaps bypassing spatial locations:** Swapping assets or resources between non-adjacent spatial partitions in a single simulation update.
15. **Centralized currency ledgers and assets:** A centralized banking or currency resource.
16. **Timeline-based capability upgrades:** Scaling tool capacities or properties based strictly on updates elapsed.
17. **Fixed discovery trees for technology optimization:** Pre-scripted technology dependencies or trees.

### Phase 12: Civilization
18. **Un-enforced spatial blocks restricting traversal without guard entities:** Restricting movement using global boundary scripts.
19. **Predefined adaptive unit specializations:** Hardcoding adaptive unit roles.
20. **Coordinating multi-unit updates globally without local communication:** Simultaneous construction updates globally without local coordination.

# Genesis Constitution

## Purpose
This Constitution establishes the immutable, non-negotiable laws governing the development and evolution of the Genesis Artificial Civilization Engine. These laws define the core identity of the project. They govern all architectural decisions, design patterns, and implementations across all phases of development, ensuring the simulation remains a scientifically rigorous, emergence-driven artificial civilization engine.

---

## Constitutional Principles

### 1. Determinism First
* **Statement:** The simulation must execute identically and repeatably across independent runs given the same initial seed, configuration parameters, and simulation update counts. 
* **Rationale:** Genesis is a scientific instrument for studying emergent behavior. Without bit-perfect reproducibility, anomalies cannot be reliably investigated, state replication cannot be guaranteed, and observations cannot be verified.
* **Implications:**
  * Wall-clock time, rendering framerates, and system thread execution speeds must never affect simulation updates.
  * Platform-native thread-local or clock-based entropy is prohibited. All random states must derive from explicit, seeded random number generators.
  * Any state query or collection iteration order that influences simulation updates must be sorted deterministically before execution.

### 2. Emergence Over Hardcoding
* **Statement:** Higher-level social, cultural, political, and institutional outcomes must arise solely as emergent properties of selective pressures and adaptive unit interactions. They must never be hardcoded or scripted.
* **Rationale:** The ultimate success of Genesis is defined by the appearance of phenomena that were never explicitly programmed. Coding shortcuts to force desired patterns invalidate the simulation.
* **Implications:**
  * No simulation systems may directly allocate adaptive units into preset social, religious, economic, or governmental structures.
  * Environmental and adaptive unit interaction rules must remain value-neutral, providing substrate-level constraints from which higher-level structures assemble naturally.

### 3. Explicit State Representation
* **Statement:** All physical and informational state relevant to simulation outcomes must be represented explicitly and remain observable independently of the mechanisms that transform it.
* **Rationale:** Scientific inspectability requires that no hidden state, transient execution context, or implicit control pathway can influence simulation outcomes without being represented within the observable simulation state. This ensures reproducibility, auditability, and deterministic analysis across substrate implementations.
* **Implications:**
  * The complete simulation state must be inspectable and recoverable at any simulation update boundary.
  * Outcome-relevant state must not exist solely within transient execution contexts.
  * State transitions must be derivable from observable state and explicit transformation rules.
  * Any mechanism capable of influencing future simulation outcomes must expose its relevant state through the observable simulation state.

### 4. Pressure-Driven Progression
* **Statement:** Higher-level systems must never be implemented before the lower-level pressures that demand them exist.
* **Rationale:** Specialization arises from trade, trade from scarcity, and scarcity from operational limits. Implementing complex social behaviors before base operational depletion pressures are established results in artificial, scripted dynamics.
* **Implications:**
  * Development must progress sequentially by establishing fundamental substrate-level constraints before introducing mechanisms that respond to those constraints.
  * No simulation system or feature may be added unless the constraints of the preceding layers create the specific selective pressures that make the system necessary.

### 5. Simulation Over Visualization
* **Statement:** The simulation engine must operate independently from visualization, client interface, and real-time presentation layers.
* **Rationale:** The simulation must be able to run headless at maximum compute rates without being bottlenecked or modified by rendering pipelines.
* **Implications:**
  * The presentation layer is strictly read-only relative to the simulation state.
  * Pausing, fast-forwarding, or resuming the simulation must translate to discrete simulation updates, completely independent of rendering loop cadences.

### 6. Prefer the Simplest Plausible Model
* **Statement:** When multiple models of a phenomenon are possible, the design must adopt the simplest model that remains deterministic, maintains physical invariants, and generates sufficient pressure to enable the next layer of emergence.
* **Rationale:** The goal of Genesis is enabling emergence, not modeling realism or geology. Over-engineering substrate physics wastes computational budgets and introduces persistence or verification bottlenecks.
* **Implications:**
  * Substrate features and environmental variables must be modeled using the simplest mathematical abstractions sufficient to generate selective pressures, rather than high-fidelity physical representations.

---

## Amendment Rule
This Constitution represents the non-negotiable core of Genesis. It should be changed only under extreme circumstances. Any amendment to a constitutional lock must provide overwhelming theoretical and empirical evidence that the principle creates a fundamental and irreducible barrier to the project's stated objectives. The amendment must preserve deterministic parity, establish physical conservation laws, and ensure existing experimental reproducibility guarantees and historical validation baselines remain valid, or provide a deterministic migration path.

# ADR-001: ECS Architectural Boundaries

# Status

Accepted

# Context

Genesis requires a design system that enforces data-oriented structure, scales to support thousands of active elements, and avoids the traditional object-oriented pitfalls of god objects and monolithic manager classes. The architecture must strictly uphold the project philosophy: "Systems over objects. Data-oriented design. Emergence over scripting."

# Decision

1. **Bevy ECS** is selected as the primary architectural framework for the simulation engine.
2. **Strict Separation of Data and Logic:**
   - **Components and Resources contain data only.** They are passive containers. They must not contain simulation logic, perform calculations, validate world rules, or mutate unrelated state.
   - **Systems contain logic.** All simulation behavior, rules enforcement, state updates, and event emissions are executed inside systems.
3. **Prohibition of Manager Classes:**
   - No manager classes (such as `WorldManager`, `SimulationManager`, `ResourceManager`) or manager-like resources are permitted.
   - Coordination and control flow must be handled entirely through ECS schedules, systems, queries, resources, and event triggers.
4. **Shared Simulation Context:**
   - ECS Resources must only represent global, shared context (e.g., configurations, clock state, boundaries) and must not become general-purpose global storage.

# Consequences

- **Benefits:**
  - Highly composable and inspectable behavior.
  - Data structures are trivial to serialize, deserialize, validate, and unit test in isolation.
  - New simulation rules and systems can be added modularly without refactoring core state structs.
- **Drawbacks:**
  - Coordination is decentralized, requiring developers to navigate system scheduler logic rather than calling clean manager methods.
  - Increased boilerplate for routing data from resources/components into system parameters.

# Constraints

- All components and resources must derive `serde::Serialize` and `serde::Deserialize` to support snapshotting.
- Public APIs and data structures must be fully documented.

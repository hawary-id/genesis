# Genesis Architecture Decisions

This document summarizes the Architectural Decision Records (ADRs) that govern the Genesis codebase. Any modifications to execution patterns, data layouts, or scheduling constraints must align with these records.

---

## ADR-001: ECS Architectural Boundaries

* **Decision:** Enforce absolute separation between data representation and behavioral logic. Data is stored strictly inside ECS components and resources; execution logic resides strictly in pure system functions. No object-oriented "manager" classes or god objects are permitted.
* **Status:** Accepted
* **Stability Level:** LOCKED
* **Reason:** Ensures cache-friendly, data-oriented memory layout and leverages the parallelism and query optimization of `bevy_ecs`.
* **Consequences:** Systems execute as side-effect-free functions querying dense arrays. This isolates simulation states and ensures time steps are deterministic.
* **Related Files:**
  - [ADR-001-ecs-architectural-boundaries.md](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-001-ecs-architectural-boundaries.md) — The core architectural decision record.
  - [engine/src/app/mod.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/app/mod.rs) — App wrapper managing Bevy boundaries.

---

## ADR-002: Deterministic Execution Contract

* **Decision:** The simulation must execute identically and repeatably across independent runs given the same seed and configuration. The engine sequences all ticking systems sequentially using Bevy `.after()` ordering, and derives coordinate-salted random seeds locally per chunk.
* **Status:** Accepted
* **Stability Level:** LOCKED
* **Reason:** Reproducibility is a foundational requirement for verifying emergent behavior, testing algorithms, and validating snapshot round-trips.
* **Consequences:** Race conditions from concurrent schedule threads are avoided. Pseudo-random generation streams remain space-invariant and step-invariant.
* **Related Files:**
  - [ADR-002-deterministic-execution-contract.md](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-002-deterministic-execution-contract.md) — The determinism contract.
  - [engine/src/testing/determinism.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/determinism.rs) — Determinism integration test suite.

---

## ADR-003: Spatial Coordinate Model

* **Decision:** The simulation grid is divided into coordinate-mapped chunk entities. Environmental values are stored inside chunk components as flat 1D vectors rather than spawning cell entities. Local cell coordinate indexing calculations are handled dynamically.
* **Status:** Accepted
* **Stability Level:** STABLE
* **Reason:** Spawning millions of individual cell entities in Bevy would split archetypes and saturate memory bandwidth, resulting in severe simulation slow-downs.
* **Consequences:** Spatial data is kept cache-local. Systems must manually map 2D cell offsets `local_y * chunk_size + local_x` into flat vectors, presenting minor boilerplate debt but optimizing performance.
* **Related Files:**
  - [ADR-003-spatial-coordinate-model.md](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-003-spatial-coordinate-model.md) — The spatial model.
  - [engine/src/world/coord.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/coord.rs) — Coordinate types and translation methods.

---

## ADR-004: Physical Time Model

* **Decision:** Time advances using a deterministic fixed-timestep model. One simulation tick represents one hour. Day cycles occupy exactly 24 ticks, and seasons span a configured number of days. Environmental variables are updated strictly on daily boundaries.
* **Status:** Accepted
* **Stability Level:** LOCKED
* **Reason:** Decoupling simulation logic from real-world CPU processing speeds ensures consistent physical progression across all systems.
* **Consequences:** Climate, resources, and energy variables are recalculate at daily transitions rather than every tick, saving performance. Clock ticks map to days, seasons, and years.
* **Related Files:**
  - [ADR-004-physical-time-model.md](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-004-physical-time-model.md) — The time model.
  - [engine/src/time/simulation_clock.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/time/simulation_clock.rs) — Tick tracking resources.

---

## ADR-005: World Generation Strategy

* **Decision:** Procedural world generation runs in sequence during a dedicated `StartupGeneration` schedule. Individual systems spawn coordinates, generate terrain, derive climate, sample minerals, and compute solar exposure before marking entities as `Generated`.
* **Status:** Accepted
* **Stability Level:** LOCKED
* **Reason:** Ensures that runtime schedules never evaluate initialization checks, maintaining high ticking efficiency.
* **Consequences:** Generation steps are modular and validation invariants run immediately after generation is completed, guaranteeing initial consistency.
* **Related Files:**
  - [ADR-005-world-generation-strategy.md](https://github.com/hawary-id/genesis/blob/main/docs/adr/ADR-005-world-generation-strategy.md) — The generation strategy.
  - [engine/src/world/generation.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/generation.rs) — Sequenced generators registration.

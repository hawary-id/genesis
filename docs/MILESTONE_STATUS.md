# Genesis Milestone Status Registry

This registry tracks the status and deliverables of all milestones in the Genesis roadmap.

* **Overall Progress Estimate:** Phase 1 Complete; Phase 2 Planning Complete
* **Current Phase:** Phase 2 — Life
* **Current Milestone:** Milestone 11 — Agent Data Foundation & Spawning

---

## Current Active Work

* **Active Phase:** Phase 2 — Life (Planning complete; Implementation not started)
* **Current Milestone:** Milestone 11 — Agent Data Foundation & Spawning

## Current Focus

* Beginning Milestone 11 (Agent Data Foundation & Spawning).
* Establishing agent data components, stable identifier generators, and spawning systems in Rust.

## Next Milestone

* **Milestone 11:** Agent Data Foundation & Spawning (Phase 2, Milestone 1)

---

## Phase 1 — World (Completed)

### Milestone 1: Foundation
* **Status:** Completed
* **Summary:** Established the Rust workspace, target configuration, and core dependencies on `bevy_ecs`, `rand`, and `serde`.
* **Dependencies:** None.
* **Related Documents:**
  - [ROADMAP.md](https://github.com/hawary-id/genesis/blob/main/docs/ROADMAP.md) — Overall project roadmap.
* **Related Source Code:**
  - [Cargo.toml](https://github.com/hawary-id/genesis/blob/main/Cargo.toml) — Workspace manifest.
  - [engine/Cargo.toml](https://github.com/hawary-id/genesis/blob/main/engine/Cargo.toml) — Crate dependencies list.

### Milestone 2: ECS Setup
* **Status:** Completed
* **Summary:** Defined the `App` container wrapper, declared the five primary schedules, and registered them within Bevy.
* **Dependencies:** Milestone 1.
* **Related Documents:**
  - [MILESTONE2_ARCHITECTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE2_ARCHITECTURE.md) — Milestone 2 architecture guidelines.
* **Related Source Code:**
  - [app/schedules.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/app/schedules.rs) — Schedule labels declaration.
  - [app/mod.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/app/mod.rs) — App wrapper and schedulers registrations.

### Milestone 3: Terrain
* **Status:** Completed
* **Summary:** Implemented spatial coordinates, dense elevation, slope, and soil components, and coordinate-salted bilinear value noise generator.
* **Dependencies:** Milestone 2.
* **Related Documents:**
  - [MILESTONE3_ARCHITECTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE3_ARCHITECTURE.md) — Terrain layer specs.
* **Related Source Code:**
  - [world/terrain.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/terrain.rs) — Terrain vectors and generator.
  - [world/coord.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/coord.rs) — Coordinates translating helper logic.

### Milestone 4: Climate
* **Status:** Completed
* **Summary:** Added latitude sunlight factor, lapse-rate temperature, moisture, and rainfall cycles, advancing daily.
* **Dependencies:** Milestone 3.
* **Related Documents:**
  - [MILESTONE4_ARCHITECTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE4_ARCHITECTURE.md) — Climate layer specs.
* **Related Source Code:**
  - [world/climate.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/climate.rs) — Climate chunk fields and update system.

### Milestone 5: Resources
* **Status:** Completed
* **Summary:** Modeled environmental resources depletion and replenishment cycles (minerals, fresh water, and nutrients).
* **Dependencies:** Milestone 4.
* **Related Documents:**
  - [MILESTONE5_ARCHITECTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE5_ARCHITECTURE.md) — Resource layer specs.
* **Related Source Code:**
  - [world/resource.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/resource.rs) — Resources components and daily update logic.

### Milestone 6: Energy Availability
* **Status:** Completed
* **Summary:** Computed environmental solar potential and energy availability across slopes and climate zones.
* **Dependencies:** Milestone 5.
* **Related Documents:**
  - [MILESTONE6_ARCHITECTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE6_ARCHITECTURE.md) — Energy specs.
* **Related Source Code:**
  - [world/energy.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/world/energy.rs) — Energy chunk components and calculations.

### Milestone 7: Simulation Clock & Seasons
* **Status:** Completed
* **Summary:** Established simulation tick cycles, physical clocks, and transitional season modifier calculations.
* **Dependencies:** Milestone 6.
* **Related Documents:**
  - [MILESTONE7_ARCHITECTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE7_ARCHITECTURE.md) — Clock specs.
* **Related Source Code:**
  - [time/simulation_clock.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/time/simulation_clock.rs) — Chronological ticking.
  - [time/season_state.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/time/season_state.rs) — Seasons modifiers calculation.

### Milestone 8: Validation
* **Status:** Completed
* **Summary:** Added post-generation and post-tick runtime invariants bounds and clock monotonicity checks.
* **Dependencies:** Milestone 7.
* **Related Documents:**
  - [MILESTONE8_ARCHITECTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE8_ARCHITECTURE.md) — validation specs.
* **Related Source Code:**
  - [validation/systems.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/validation/systems.rs) — validation checks systems.

### Milestone 9: Persistence
* **Status:** Completed
* **Summary:** Developed JSON snapshot serialization and save/load world reconstruction routines.
* **Dependencies:** Milestone 8.
* **Related Documents:**
  - [MILESTONE9_ARCHITECTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE9_ARCHITECTURE.md) — Snapshot specs.
* **Related Source Code:**
  - [persistence/io.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/io.rs) — Stable chunk sort serialization.
  - [persistence/snapshot.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/persistence/snapshot.rs) — File data models.

### Milestone 10: Determinism
* **Status:** Completed
* **Summary:** Hardened determinism integration tests, verifying seed sensitivity, ticking determinism, save/load equivalence, and 1 simulation year stability.
* **Dependencies:** Milestone 9.
* **Related Documents:**
  - [MILESTONE10_ARCHITECTURE.md](https://github.com/hawary-id/genesis/blob/main/docs/MILESTONE10_ARCHITECTURE.md) — Determinism verification specs.
  - [PHASE1_COMPLETION_REPORT.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE1_COMPLETION_REPORT.md) — Audit completion numbers.
* **Related Source Code:**
  - [testing/determinism.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/determinism.rs) — Integration verification suite.
  - [testing/fixtures.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/fixtures.rs) — Equivalence check asserts.

---

## Phase 2 — Life (Planning Approved; Implementation Pending)

* **Related Documents:**
  - [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md) — Technical Specification.
  - [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md) — Implementation Plan.

### Milestone 11: Agent Data Foundation & Spawning
* **Status:** Ready
* **Summary:** Spawning agent entities carrying data components for positions, metabolic stocks, stable sequence identifiers, and action request targets.
* **Dependencies:** Milestone 10.
* **Related Documents:**
  - [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md#data-ownership--functional-responsibilities) — Data layout guidelines.
  - [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md#milestone-11-agent-data-foundation--spawning) — Milestone roadmap.

### Milestone 12: Environmental Sensing Query API
* **Status:** Planned
* **Summary:** Implementing standard coordinate translation utilities and neighborhood sensing systems reading resource values from chunk components.
* **Dependencies:** Milestone 11.
* **Related Documents:**
  - [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md#coordinate-mapping) — Sensing specifications.
  - [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md#milestone-12-environmental-sensing-query-api) — Milestone roadmap.

### Milestone 13: Metabolic Tick Systems
* **Status:** Planned
* **Summary:** Advancing metabolic decay and lifespan age counters on ticks, and processing removals when energy is exhausted or lifespan thresholds are crossed.
* **Dependencies:** Milestone 12.
* **Related Documents:**
  - [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md#4-metabolism-energy-decay) — Metabolic guidelines.
  - [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md#milestone-13-metabolic-tick-systems) — Milestone roadmap.

### Milestone 14: Spatial Movement Execution
* **Status:** Planned
* **Summary:** Executing grid steps on tick schedules, validating targets against boundaries, slopes, and water zones.
* **Dependencies:** Milestone 13.
* **Related Documents:**
  - [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md#2-movement-model) — Movement rules.
  - [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md#milestone-14-spatial-movement-execution) — Milestone roadmap.

### Milestone 15: Persistence & Integration Testing
* **Status:** Planned
* **Summary:** Extending snapshot serialization to save sorted agent states, and verifying execution determinism under A+B=N save/load equivalence integration tests.
* **Dependencies:** Milestone 14.
* **Related Documents:**
  - [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md#determinism-requirements) — Determinism rules.
  - [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md#milestone-15-persistence--integration-testing) — Milestone roadmap.

### Phases 3 through 12 — Evolution, Memory, Agency, Society, Economy, Civilization
* **Status:** Planned
* **Summary:** Implementation of genomes, location memories, priorities prioritization, rep trading, specialized technology trees, and institutional governance.
* **Dependencies:** Phase 2.
* **Related Documents:**
  - [ROADMAP.md](https://github.com/hawary-id/genesis/blob/main/docs/ROADMAP.md#L43-L184) — Long-term phases descriptions.

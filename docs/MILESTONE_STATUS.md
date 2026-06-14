# Genesis Milestone Status Registry

This registry tracks the status and deliverables of all milestones in the Genesis roadmap.

* **Overall Progress Estimate:** Phase 1 Complete; Phase 2 Milestone 14 Completed
* **Current Phase:** Phase 2 — Life
* **Current Milestone:** Milestone 15 — Agent Persistence & Integration Testing

---

## Current Active Work

* **Active Phase:** Phase 2 — Life (Milestone 14 complete and verified)
* **Current Milestone:** Milestone 15 — Agent Persistence & Integration Testing

## Current Focus

* Extending snapshot serialization to save sorted agent states, and verifying execution determinism under A+B=N save/load equivalence integration tests (Milestone 15).

## Next Milestone

* **Milestone 15:** Persistence & Integration Testing (Phase 2, Milestone 15)


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

## Phase 2 — Life (Implementation in Progress)

* **Related Documents:**
  - [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md) — Technical Specification.
  - [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md) — Implementation Plan.

### Milestone 11: Agent Data Foundation & Spawning
* **Status:** Completed
* **Summary:** Implemented foundational agent ECS components (`AgentMetadata`, `AgentPosition`, `MetabolicStock`, `ActionRequest`), deterministic seed derivation, stable identifier sequence generators, and spawner logic running on StartupGeneration. Extended validation framework to enforce ID uniqueness, count caps, coordinate boundary validity, and metabolic ranges.
* **Dependencies:** Milestone 10.
* **Related Documents:**
  - [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md#data-ownership--functional-responsibilities) — Data layout guidelines.
  - [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md#milestone-11-agent-data-foundation--spawning) — Milestone roadmap.
* **Related Source Code:**
  - [agent/components.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/components.rs) — ECS structures.
  - [agent/resources.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/resources.rs) — Stable sequence counter.
  - [agent/systems.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/systems.rs) — Spawn logic and spawn unit tests.
  - [agent/mod.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/mod.rs) — Submodule re-exports.

### Milestone 12: Environmental Sensing Query API
* **Status:** Completed
* **Summary:** Implemented the environmental sensing query API providing coordinates translation and read-only cell and neighborhood resource queries (nutrients and fresh water) from chunk entities. All tests are passing, ready for tag phase2-milestone-12.
* **Dependencies:** Milestone 11.
* **Related Documents:**
  - [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md#coordinate-mapping) — Sensing specifications.
  - [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md#milestone-12-environmental-sensing-query-api) — Milestone roadmap.
* **Related Source Code:**
  - [agent/sensing.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/sensing.rs) — Sensing APIs (`query_cell`, `query_neighborhood`, and `SensedResource`) and unit tests.
  - [agent/mod.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/mod.rs) — Expose sensing components.

### Milestone 13: Metabolic Tick Systems
* **Status:** Completed
* **Summary:** Implemented metabolic decay updates using the approved absolute difference penalty formula, chronological agent aging (using `saturating_add(1)`), and agent removal on tick boundaries when energy <= 0.0 or age > agent_age_limit. Registered systems in FixedSimulationTick schedule (metabolism sequentially after climate/resource/energy updates; death processing sequentially after metabolism).
* **Dependencies:** Milestone 12.
* **Related Documents:**
  - [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md#4-metabolism-energy-decay) — Metabolic guidelines.
  - [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md#milestone-13-metabolic-tick-systems) — Milestone roadmap.
* **Related Source Code:**
  - [config/world_config.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/config/world_config.rs) — Metabolic decay configurations.
  - [testing/fixtures.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/fixtures.rs) — Test configurations setup.
  - [agent/systems.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/systems.rs) — `update_agent_metabolism` and `process_agent_deaths` systems and test suite.
  - [agent/mod.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/mod.rs) — Agent module re-exports.
  - [app/mod.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/app/mod.rs) — Sequenced schedule registration.

### Milestone 14: Spatial Movement Execution
* **Status:** Completed
* **Summary:** Implemented spatial movement execution along cardinal directions under Bevy schedules. Validates movement targets against boundaries, elevation slopes, and water zones, applying energy costs and clearing action requests on all execution paths. Deterministic behavior and movement limits were fully verified via unit tests.
* **Dependencies:** Milestone 13.
* **Related Documents:**
  - [PHASE2_LIFE_TECH_SPEC.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_LIFE_TECH_SPEC.md#2-movement-model) — Movement rules.
  - [PHASE2_IMPLEMENTATION_PLAN.md](https://github.com/hawary-id/genesis/blob/main/docs/PHASE2_IMPLEMENTATION_PLAN.md#milestone-14-spatial-movement-execution) — Milestone roadmap.
* **Related Source Code:**
  - [config/world_config.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/config/world_config.rs) — Movement thresholds parameters.
  - [testing/fixtures.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/testing/fixtures.rs) — Fixtures updates.
  - [agent/systems.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/systems.rs) — `process_agent_movement` logic and tests.
  - [agent/mod.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/agent/mod.rs) — Exporting the movement system.
  - [app/mod.rs](https://github.com/hawary-id/genesis/blob/main/engine/src/app/mod.rs) — Schedule ordering integration.

### Milestone 15: Persistence & Integration Testing
* **Status:** Active
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

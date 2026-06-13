# ADR-004: Physical Time Model

# Status

Accepted

# Context

The simulation needs to model cycles of abundance and scarcity, temperature oscillations, and seasonal changes. We need a time model that supports deterministic progression, is simple to save and load, and does not hardcode human cultural concepts (e.g., months, holidays, work weeks) into the core physical engine.

# Decision

1. **Atomic Tick Unit:**
   - A simulation tick is the smallest, indivisible unit of time progression.
   - All physical simulation systems advance strictly in integer steps of ticks.
2. **Single Source of Truth:**
   - The canonical representation of time is a single monotonic integer field, `total_ticks`, stored in the `SimulationClock` resource.
3. **Derived Cycles:**
   - Day, season, and year cycle locations are derived mathematically from `total_ticks` using configured cycle intervals.
   - No separate calendar structs or clock variables are maintained as core state to prevent drift or synchronization errors.
4. **Physical Time Only:**
   - Time in the engine represents physical cycles only. Cultural interpretations, calendar divisions, and social schedules do not belong in the physical simulation and must not be hardcoded.
5. **Standard Cadence Configurations (Default):**
   - $1\text{ tick} = 1\text{ simulation hour}$.
   - $1\text{ day} = 24\text{ ticks}$.
   - $1\text{ season} = 90\text{ days} = 2,160\text{ ticks}$.
   - $1\text{ year} = 4\text{ seasons} = 360\text{ days} = 8,640\text{ ticks}$.

# Consequences

- **Benefits:**
  - Zero state duplication; time cannot drift because all cycles are derived from one field.
  - Snapshot persistence only needs to store `total_ticks` and configuration bounds.
  - Future calendars or agent-level schedules can be layered on top as read-only systems without touching clock logic.
- **Drawbacks:**
  - Cycle calculations (like determining the current day of the year) require helper functions for conversion.
  - Configured cycle variables must divide cleanly; irregular cycle configurations are not supported.

# Constraints

- Simulation ticks must advance monotonically.
- All derived calendar cycles must use deterministic integer division and modulo math.

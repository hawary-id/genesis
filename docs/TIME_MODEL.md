# Time Model

## Purpose

The time model defines how Genesis advances simulation state.

Phase 1 time must support deterministic ticks, day cycles, seasons, years, persistence, and future long-running worlds.

Time should be simple, explicit, and stable before agents exist.

## Design Choice: Tick Is The Atomic Time Unit

A simulation tick is the smallest canonical unit of state transition.

All deterministic simulation systems must advance by whole ticks.

Rationale:

- Whole ticks are easy to count, save, compare, and replay.
- Deterministic tests can run exact tick counts.
- Future systems can schedule work against a shared base unit.

Tradeoffs:

- Sub-tick behavior is not represented in Phase 1.
- Fast-forwarding means executing or safely batching tick-equivalent updates.
- Systems that change slowly need explicit lower-frequency schedules.

Future implications:

- Agents, memory, aging, history, and evolution can share the same time base.
- Multi-rate systems can be layered on top of ticks.
- Historical events can reference exact tick numbers.

## Design Choice: One Tick Represents One Simulation Hour

For Phase 1, one tick should represent one simulation hour.

Rationale:

- Hourly ticks are coarse enough for long-running worlds.
- They are fine enough for day/night cycles, temperature change, and seasonal pressure.
- They avoid excessive tick counts before agents exist.

Tradeoffs:

- Fast processes such as storms, fires, or individual movement are not modeled precisely in Phase 1.
- Later life systems may need sub-hour action modeling or action queues.
- Some climate processes may still need daily or seasonal aggregation.

Future implications:

- Future agents can act on hourly ticks at first, then gain sub-tick action scheduling only if necessary.
- Long simulations remain computationally feasible.
- Day, season, and year lengths can be expressed as exact integer tick counts.

## Design Choice: Day Length Is 24 Ticks

One day is 24 simulation ticks.

Rationale:

- A 24-hour day is intuitive and stable.
- It supports daily climate cycles without extra conversion.
- It gives future organisms and agents a familiar rhythm without hardcoding culture.

Tradeoffs:

- It assumes an Earth-like day length.
- Alien world configurations would require later parameterization.
- Some environmental processes may not need hourly resolution.

Future implications:

- Circadian or daily behavior can emerge later without changing the clock.
- Future world configurations may override day length if Genesis explores non-Earth-like worlds.
- Persistence must store the current tick, while day can be derived.

## Design Choice: Season Length Is 90 Days

One season is 90 days, or 2,160 ticks.

Rationale:

- Ninety-day seasons are simple, predictable, and close to an Earth-like seasonal cadence.
- Integer lengths avoid drift.
- Seasons are long enough to create scarcity cycles without changing too rapidly.

Tradeoffs:

- This simplifies real orbital variation.
- Four equal seasons may feel artificial.
- Some worlds may need different seasonal structures later.

Future implications:

- Life can later adapt to predictable abundance and scarcity windows.
- Memory and planning can later discover seasonal regularity.
- Alternate calendars can be introduced as configuration only after the base model is stable.

## Design Choice: Year Length Is 360 Days

One year is four seasons, 360 days, or 8,640 ticks.

Rationale:

- A 360-day year divides cleanly into four equal 90-day seasons.
- Integer cycles support deterministic tests and simple save/load behavior.
- The model prioritizes long-term consistency over calendar realism.

Tradeoffs:

- The year is not Earth-accurate.
- Leap days, irregular months, and orbital eccentricity are excluded.
- Calendar simplicity may hide some forms of environmental variability.

Future implications:

- Aging, evolution, and history can use stable annual units later.
- Cultural calendars must not be hardcoded from this model; they may emerge later.
- If astronomical realism becomes useful, it should be added as configurable world physics, not as a default complexity.

## Design Choice: Derived Time Values

The canonical stored time should be total elapsed ticks. Day, season, and year should be derived from tick count unless a future system proves otherwise.

Rationale:

- One canonical value avoids inconsistent time state.
- Derived values are deterministic.
- Save files stay smaller and easier to validate.

Tradeoffs:

- Queries need conversion helpers.
- If future worlds support irregular calendars, derivation will become more complex.
- Storing only ticks makes human-readable debugging less immediate.

Future implications:

- Persistence only needs exact tick count for time continuity.
- Future calendar systems can derive their own interpretations from ticks.
- Cultural timekeeping can emerge separately from physical simulation time.

## Design Choice: Time Is Physical, Not Cultural

Phase 1 time represents physical simulation progression only.

It does not define months, weeks, holidays, eras, rituals, work schedules, or cultural calendars.

Rationale:

- Culture must not be hardcoded.
- A physical clock is needed before social meaning exists.
- Later agents should create or infer temporal meaning from experience.

Tradeoffs:

- Debug output may lack familiar calendar names.
- Dashboard displays may need simple derived labels for inspection.
- Future culture systems will need their own representation of social time.

Future implications:

- Societies can later invent calendars or traditions from environmental cycles.
- History can record ticks first and cultural dates later.
- The engine remains neutral about social interpretations of time.

## Phase 1 Time Constants

Recommended defaults:

- 1 tick = 1 simulation hour.
- 1 day = 24 ticks.
- 1 season = 90 days = 2,160 ticks.
- 1 year = 4 seasons = 360 days = 8,640 ticks.

These values should be treated as Phase 1 configuration defaults, not immutable laws of all Genesis worlds.

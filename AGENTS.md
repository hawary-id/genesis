# Genesis

An Artificial Civilization Engine.

## Philosophy

Do not build civilization.

Build the conditions under which civilization becomes inevitable.

## Architecture Rules

- Follow ECS principles.
- Prefer emergence over scripting.
- Prefer systems over objects.
- Use data-oriented design.
- Keep systems deterministic.
- Avoid manager classes.
- Avoid god objects.
- Avoid global mutable state.
- Avoid premature abstractions.

## Development Rules

- Never implement higher-level systems before lower-level pressures exist.
- Never hardcode culture.
- Never hardcode economy.
- Never hardcode politics.
- Never hardcode religion.

## Coding Rules

- Rust only.
- Follow idiomatic Rust.
- Small modules.
- Unit tests required.
- Document public APIs.

## Review Rules

Before implementing:

1. Verify the design aligns with VISION.md.
2. Verify the design aligns with PRINCIPLES.md.
3. Explain tradeoffs.
4. Identify future scalability concerns.

When uncertain, choose simplicity.

## Environmental Scope Rule

Genesis is not a weather simulator.

Genesis is not a geology simulator.

Genesis is not a physics simulator.

Environmental systems should only be as detailed as necessary to generate meaningful pressures for future life and civilization.

When choosing between realism and emergence, prefer emergence.

## Architecture Governance

* Accepted ADRs under `docs/adr/` are authoritative architectural decisions.

* `docs/ARCHITECTURE_BASELINE.md` is the authoritative summary of the current architecture.

* Agents must review relevant ADRs before proposing architectural changes.

* Agents must not revisit, redesign, or replace accepted ADRs unless explicitly requested by the user.

* If a proposed change conflicts with an accepted ADR, the agent must:

  1. Identify the conflict.
  2. Stop implementation.
  3. Request architectural review.

* If architecture is not covered by an existing ADR, agents may propose a new ADR candidate.

* Agents must not silently change architectural behavior. Any ADR-impacting change must be explicitly identified.

* Implementation code is not the source of architectural truth. Architecture is defined by ADRs and architecture documentation.

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

# ADR-002: Deterministic Execution Contract

# Status

Accepted

# Context

To trust emergent behaviors in Genesis, every simulation run must be 100% reproducible. If a rare or surprising phenomenon occurs, developers must be able to reproduce it using only the world seed, configuration parameters, and tick count. Therefore, determinism is not an optimization; it is a hard architectural requirement.

# Decision

1. **Fixed Timestep Execution:**
   - The simulation progresses only in discrete, uniform steps (ticks). Wall-clock time and render frame rates must never affect the simulation updates.
2. **Explicit Seeded Entropy:**
   - All random calculations must derive from explicit, seeded random number generators (specifically using `rand_chacha`).
   - Hidden entropy (e.g., `rand::thread_rng()`, `rand::random()`, `SystemTime::now()`, platform CPU cycle counts) is prohibited in simulation logic.
3. **Explicit Schedule Ordering:**
   - Any systems whose execution order affects the simulation state must be ordered explicitly using Bevy ECS scheduler constraints (`.after()`, `.before()`). Never rely on accidental or implicit scheduler behavior.
4. **Stable Iteration:**
   - Iteration over ECS queries or hash maps must not affect outcomes unless the collection is explicitly sorted using stable, reproducible keys (such as coordinate types or unique identifiers).
5. **Save/Load Continuation Equivalence:**
   - Running $N$ ticks continuously must yield identical binary outcomes to running $A$ ticks, saving a snapshot, loading it, and running $B$ ticks where $A + B = N$.

# Consequences

- **Benefits:**
  - Guaranteed reproducibility for debugging and long-term testing.
  - Snapshot-based persistence behaves identically to live runs.
  - Eliminates class of bugs related to scheduler race conditions or clock-drift variation.
- **Drawbacks:**
  - Parallel execution requires careful chunk-level RNG partitioning (domain-specific seeding) to avoid coordination overhead.
  - Sorting and stable iteration logic can introduce minor performance costs.

# Constraints

- Continuous fields use standard `f32` types. Transcendental functions (`sin`, `cos`, `pow`) are permitted for prototype phases (accepting local/x86_64 determinism limits) but must be flagged for cross-platform divergence risks.
- Snapshot persistence must capture all RNG state required for continuation.

# ADR-005: World Generation Strategy

# Status

Accepted

# Context

Genesis requires a deterministic terrain generation pipeline. The initial world substrate (terrain elevation, slope, water presence, soil depth, and soil fertility) must be built from a single root seed, where the same seed always produces the exact same world. Furthermore, chunk generation must be independent of chunk iteration order to preserve determinism during parallel spawning.

# Decision

1. **Hash-Based Value Noise:**
   - Instead of adding external noise libraries, terrain is generated using bilinear interpolation of a coarse height lattice.
   - Random lattice point values are drawn using the existing `rand_chacha` dependency.
   - The final elevation is computed by blending two noise octaves at different lattice scales.
2. **Domain-Specific & Coordinate-Salting Seeding Model:**
   - The terrain domain seed is derived using a wrapping addition of a domain salt (`TERRAIN_DOMAIN_SALT`) to the root seed.
   - Each chunk coordinate `(x, y)` derives its own unique chunk seed using deterministic integer multiplicative hashes on the domain seed.
   - This isolates chunk RNG streams, meaning chunk generation order has no impact on the generated output.
3. **Two-Pass Chunk Generation Pipeline:**
   - **Pass 1:** Generate elevation for all cells in the chunk. Determine cell water depth using a threshold comparison against a configured `sea_level`.
   - **Pass 2:** Compute slope for all cells using a central difference gradient approximation from local elevation cells. Finalize soil depth (inversely proportional to slope) and soil fertility (proportional to soil depth and water proximity).
4. **Local Boundary Approximation:**
   - When computing slopes at chunk or world borders, missing neighbors are treated as having the same elevation as the cell itself (zero slope contribution). This removes the need for cross-chunk queries or sync states during generation.

# Consequences

- **Benefits:**
  - Chunks can be generated in parallel or in arbitrary order with guaranteed determinism.
  - The generation pipeline is a pure function of configuration, root seed, and chunk coordinates.
  - Avoids external dependencies, keeping compilation fast and codebase footprint minimal.
- **Drawbacks:**
  - Boundary approximation creates minor slope errors at chunk borders, which is acceptable since Genesis is not a geology or physics simulator.
  - Coarse value noise with bilinear interpolation is visually simpler than advanced simplex or Gabor noise.

# Constraints

- All generated terrain fields must be validated post-generation against the ranges defined in `WorldConfig`.
- Slope math must clamp outputs to configured maximums.
- No transcendental math (like trigonometric or power functions) is permitted in the hot loop of noise sampling.

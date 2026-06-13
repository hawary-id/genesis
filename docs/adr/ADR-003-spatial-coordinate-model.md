# ADR-003: Spatial Coordinate Model

# Status

Accepted

# Context

Genesis requires a spatial representation of environmental fields (such as terrain elevation, moisture, and soil) that is performant, easily serializable, and simple to query. Modeling every grid cell as a distinct ECS entity would create millions of entities, leading to severe scheduling and memory overhead.

# Decision

1. **Chunked Square Grid:**
   - The world space is divided into a 2D square grid.
   - Cells are grouped into square chunks (default: $32 \times 32$ cells).
2. **Chunks as ECS Entities:**
   - ECS entities represent chunks, not individual cells.
   - Dense cell arrays are stored as contiguous `Vec<f32>` arrays inside components attached to chunk entities (such as `TerrainChunk`).
   - Cell indices follow row-major ordering: `index = local_y * chunk_size + local_x`.
3. **Explicit Coordinate Types:**
   - The engine defines three non-negative integer coordinate spaces:
     - `WorldCoord`: Global cell address $(x, y)$ in world space.
     - `ChunkCoord`: Global chunk address $(x, y)$ in chunk space (attached as a component to chunk entities).
     - `LocalCoord`: Cell address $(x, y)$ local to a single chunk.
   - Coordinate conversions must consist strictly of deterministic integer arithmetic (divisions and modulo).
4. **Dimension Constraints:**
   - World width and height must be perfectly divisible by the chunk size. Non-divisible dimensions are out of scope.
5. **Adjacency Rules:**
   - **4-neighbor adjacency** is used for conservative transfer processes (e.g., water or soil nutrient diffusion).
   - **8-neighbor adjacency** is reserved for derived analysis and agent perception where diagonal awareness is explicitly required.

# Consequences

- **Benefits:**
  - Dramatically reduced ECS entity overhead, enabling fast tick execution and small snapshot file sizes.
  - Linear array structures are cache-friendly and optimize bulk vector calculations (e.g., climate updates).
  - Strongly typed coordinates prevent spatial index mismatch bugs.
- **Drawbacks:**
  - Accessing individual cell variables requires indexing calculations rather than querying ECS components directly.
  - Boundary handling at chunk edges is slightly more complex, requiring coordinate lookup translations.

# Constraints

- Negative coordinates are not supported.
- Out-of-bounds coordinates must be treated as validation errors.
- Chunk borders must use defined boundary approximations (e.g., treating missing neighbors as equal elevation) to avoid cross-chunk generation dependency.

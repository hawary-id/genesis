# Spatial Model

## Purpose

The spatial model defines how Genesis represents location, neighborhood, distance, and environmental fields.

Phase 1 needs a spatial model that supports deterministic world generation, terrain, climate, resources, energy availability, chunking, persistence, and future agent queries.

The spatial model should be simple enough to implement early and stable enough to survive later phases.

## Evaluation: Square Grid

A square grid divides the world into rectangular cells with four-direction or eight-direction adjacency.

Rationale:

- Square grids are simple to index, store, serialize, chunk, and visualize.
- Coordinates map naturally to dense arrays.
- Neighbor lookup is straightforward and deterministic.
- Most debugging tools and dashboard views can represent square grids easily.

Tradeoffs:

- Four-direction movement has directional bias.
- Eight-direction movement introduces diagonal distance questions.
- Natural processes such as diffusion and movement may look grid-aligned unless rules compensate.

Future implications:

- Chunked storage remains straightforward.
- Future pathfinding, perception, and region queries are easy to implement.
- If directional bias becomes a serious research problem, hex-like neighbor rules or continuous overlays can be added later.

## Evaluation: Hex Grid

A hex grid divides the world into cells with six evenly spaced neighbors.

Rationale:

- Hex grids reduce directional bias for movement and diffusion.
- Neighbor relationships are more uniform than square grids.
- Hex grids often feel more natural for region-based simulations.

Tradeoffs:

- Coordinate systems are more complex.
- Dense rectangular storage is less direct.
- Chunking, serialization, and dashboard rendering require more care.
- Many future developers and tools are less familiar with hex indexing.

Future implications:

- Movement and local interaction systems may benefit later.
- Visualization and persistence complexity arrives immediately in Phase 1.
- If Genesis eventually emphasizes local movement ecology, hex grids may be worth reconsidering.

## Evaluation: Continuous World

A continuous world represents positions as numeric coordinates rather than fixed cells.

Rationale:

- Continuous space is the most flexible representation.
- It avoids grid artifacts.
- It can support fine-grained movement, perception, and physics later.

Tradeoffs:

- Terrain, climate, resource, and energy fields need sampling structures anyway.
- Deterministic neighbor queries become more complex.
- Persistence and spatial indexing become harder.
- Continuous simulation can invite premature physics work.

Future implications:

- Agents could later use continuous positions on top of sampled environmental fields.
- Spatial indexes would become necessary for performance.
- Phase 1 complexity would rise before life or movement exists.

## Recommendation For Phase 1: Chunked Square Grid

Phase 1 should use a chunked square grid.

Each chunk should contain dense cell arrays. Chunks can be ECS entities. Cells should not be ECS entities by default.

Rationale:

- Phase 1 is about environmental substrate, not precise movement.
- Square grids are the simplest deterministic representation for terrain, climate, resources, and energy availability.
- Chunked square grids keep persistence and validation manageable.
- The model aligns with the existing recommendation to use chunk entities with dense cell data.

Tradeoffs:

- Directional artifacts may appear in water flow, diffusion, or future movement.
- Later agent movement may need careful distance rules.
- Some ecological patterns may be affected by grid topology.

Future implications:

- Agent positions can initially be cell-based, then later become sub-cell or continuous if research goals demand it.
- Hex or continuous models remain possible later, but switching will be a major migration.
- Early APIs should avoid exposing storage details too broadly so future spatial changes remain possible.

## Design Choice: Explicit Coordinate Types

The engine should distinguish world coordinates, chunk coordinates, and cell-local coordinates.

Rationale:

- Explicit coordinate types prevent accidental mixing of spaces.
- Deterministic indexing depends on clear coordinate conversion.
- Persistence and debugging need stable location identifiers.

Tradeoffs:

- More small types must be documented.
- Coordinate conversion helpers become necessary.
- Some code may feel more verbose.

Future implications:

- Future agents can refer to locations without depending on internal chunk layout.
- Snapshot formats can preserve stable spatial identities.
- Spatial APIs can evolve while keeping coordinate contracts clear.

## Design Choice: Neighbor Rules Are Part Of The Model

Phase 1 must define whether systems use four-neighbor, eight-neighbor, or domain-specific neighbor rules.

Recommendation:

- Use four-neighbor adjacency for conservative transfer processes such as water or resource diffusion.
- Use eight-neighbor adjacency only for derived analysis or future perception where diagonal awareness is explicitly intended.

Rationale:

- Neighbor rules affect determinism and emergent patterns.
- Four-neighbor rules are simpler and avoid diagonal transfer ambiguity.
- Domain-specific rules make assumptions visible.

Tradeoffs:

- Four-neighbor rules can produce grid-shaped artifacts.
- Eight-neighbor rules may be more natural for some processes.
- Different systems using different neighbor rules require documentation.

Future implications:

- Movement can later define its own distance model.
- Climate or hydrology can adopt richer kernels if needed.
- Tests should cover neighbor ordering and boundary behavior.

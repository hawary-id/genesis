# Milestone 9 Architecture — Persistence

**Date:** 2026-06-13
**Status:** Approved & Locked
**Dependencies:** Milestone 1 ✅, Milestone 2 ✅, Milestone 3 ✅, Milestone 4 ✅, Milestone 5 ✅, Milestone 6 ✅, Milestone 7 ✅, Milestone 8 ✅

---

## Architecture Review

### Relevant Roadmap & Spec References

- **`PHASE1_IMPLEMENTATION_PLAN.md`:** Milestone 9 objective is "Add minimal snapshot-based persistence behind the `PersistenceBoundary`." Required deliverables: JSON snapshot schema, schema version, snapshot construction from stable ECS state, load path for deterministic continuation, event handling for `SnapshotRequested`/`SnapshotCompleted`, and save/load tests.
- **`PHASE1_WORLD_TECH_SPEC.md`:** Persistence scope is reproducibility, resume, save/load equivalence testing, and debug inspection only. States exact required fields. Prohibits history, event sourcing, per-tick logs, and database-first state. Defines six `PersistenceBoundary` systems in order.
- **[ADR-001: ECS Boundaries](file:///c:/Genesis/docs/adr/ADR-001-ecs-architectural-boundaries.md):** Persistence systems may only read stable ECS state. They must not mutate simulation components or resources. State participating in persistence must be serializable.
- **[ADR-002: Deterministic Execution Contract](file:///c:/Genesis/docs/adr/ADR-002-deterministic-execution-contract.md):** Running N ticks continuously must yield identical outcomes to running A ticks, saving, loading, and running B ticks where A + B = N. All RNG state needed for continuation must be captured.
- **[ADR-004: Physical Time Model](file:///c:/Genesis/docs/adr/ADR-004-physical-time-model.md):** Only `SimulationClock.total_ticks` and `WorldConfig` time parameters need to be persisted for time. `SeasonState` is reconstructed, not persisted as authoritative state.

---

## Codebase Assessment

### Milestone 9 Status: Not Started

No `persistence` module exists anywhere under `engine/src/`. The following persistence-related items are defined in Milestone 2 but have no implementations:

- `SnapshotRequested` event — not implemented
- `SnapshotCompleted` event — not implemented
- `SnapshotConfig` resource — not implemented

The `PersistenceBoundary` schedule is registered and empty.

### Already Serializable

The following types already derive `Serialize, Deserialize` and are ready for snapshot construction without modification:

| Type | Kind | Module |
|---|---|---|
| `WorldConfig` | Resource | `config/world_config.rs` |
| `WorldSeed` | Resource | `rng/seed.rs` |
| `SimulationClock` | Resource | `time/simulation_clock.rs` |
| `SeasonState` | Resource | `time/season_state.rs` |
| `ChunkCoord` | Component | `world/coord.rs` |
| `TerrainChunk` | Component | `world/terrain.rs` |
| `ClimateChunk` | Component | `world/climate.rs` |
| `ResourceChunk` | Component | `world/resource.rs` |
| `EnergyAvailabilityChunk` | Component | `world/energy.rs` |
| `ValidationError` | Enum | `validation/errors.rs` |

The `serde_json` crate is already a direct dependency.

### Not Yet Serializable

No types are missing serialization. Nothing new needs to derive `Serialize, Deserialize` beyond what already exists.

---

## Gap Analysis

| Required (from spec) | Status |
|---|---|
| JSON snapshot schema | Missing |
| Snapshot schema version | Missing |
| `SnapshotConfig` resource | Missing |
| `SnapshotRequested` event | Missing |
| `SnapshotCompleted` event | Missing |
| `SnapshotConfig` persistence boundary systems | Missing |
| `WorldSnapshot` struct | Missing |
| `ChunkSnapshot` struct | Missing |
| Snapshot write (file I/O) | Missing |
| Snapshot load (file I/O) | Missing |
| Save/load equivalence tests | Missing |

---

## Milestone 9 Scope

### Objective

Implement minimal snapshot-based persistence behind the `PersistenceBoundary` schedule. Persistence exists solely to support reproducibility, resume, and save/load equivalence testing in Phase 1.

### Explicit Non-Goals

- No history system.
- No event sourcing.
- No per-tick logs.
- No append-only event streams.
- No civilization, agent, or social records.
- No database-first simulation state.
- No PostgreSQL integration (Phase 2+).
- No compressed or binary snapshot format.
- No snapshot diffing or incremental writes.
- No snapshot schema migration tooling.

---

## ECS Architecture

### New Module: `engine/src/persistence/`

A new module is introduced at `engine/src/persistence/`. It does not own any simulation state. It observes ECS state at the `PersistenceBoundary` and writes it to disk.

### Resources

#### `SnapshotConfig` [NEW]

Defined in `PHASE1_WORLD_TECH_SPEC.md` as a required Phase 1 resource. Defines snapshot behavior at persistence boundaries.

```rust
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Number of ticks between automatic snapshots. 0 disables automatic snapshots.
    pub snapshot_interval_ticks: u32,

    /// Schema version embedded in written snapshot files.
    pub schema_version: u32,
}
```

Ownership: registered as an ECS resource at startup.
Mutability: immutable during a simulation run.

**Note on `snapshot_target_policy`:** The tech spec mentions a target policy field. In Phase 1 there is only one target (the file system). The policy field is deferred; the current struct represents the minimal required surface.

### Events

#### `SnapshotRequested` [NEW]

Requests a snapshot at the next persistence boundary.

```rust
#[derive(Event, Debug, Clone)]
pub struct SnapshotRequested;
```

Emitted by: `detect_snapshot_due` (automatic interval) or external callers (test harness, manual trigger).
Consumed by: `handle_snapshot_requests`.

#### `SnapshotCompleted` [NEW]

Reports that a snapshot finished successfully. Defined in `PHASE1_WORLD_TECH_SPEC.md` as a required boundary signal for test harnesses and future observation layers.

```rust
#[derive(Event, Debug, Clone)]
pub struct SnapshotCompleted {
    /// Total ticks at the time of snapshot.
    pub total_ticks: u32,
}
```

Emitted by: `handle_snapshot_requests` on successful write.
Consumed by: test harnesses and future observation layers.

**Payload justification:** The spec requires `SnapshotCompleted` as a boundary signal. `total_ticks` is sufficient for test harnesses to verify that a snapshot occurred at the correct tick. `output_path` is not required by the roadmap or tech spec and is omitted to keep the event minimal.

### Components

None. Persistence does not introduce new components.

---

## Snapshot Schema Design

### Top-Level Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Snapshot schema version. Must be checked on load.
    pub schema_version: u32,

    /// Total ticks at the time this snapshot was written.
    pub total_ticks: u32,

    /// World configuration at snapshot time.
    pub config: WorldConfig,

    /// Root world seed.
    pub seed: WorldSeed,

    /// Chunk state records, ordered by (chunk_y, chunk_x) for stable output.
    pub chunks: Vec<ChunkSnapshot>,
}
```

### Chunk Record

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSnapshot {
    /// Chunk coordinate.
    pub coord: ChunkCoord,

    /// Terrain field data.
    pub terrain: TerrainChunk,

    /// Climate field data.
    pub climate: ClimateChunk,

    /// Resource field data.
    pub resources: ResourceChunk,

    /// Energy availability field data.
    pub energy: EnergyAvailabilityChunk,
}
```

### Field Ordering Rule

Chunk records must be sorted by `(coord.y, coord.x)` before serialization. This ensures stable JSON output regardless of ECS entity iteration order, satisfying ADR-002.

### Schema Version

`schema_version` is `1` for the initial Phase 1 implementation. It must be checked on load and rejected with a clear error if the version does not match the current engine schema.

---

## Persisted vs Reconstructed State

| State | Policy | Reason |
|---|---|---|
| `WorldConfig` | **Persisted** | Required to reconstruct derived values and validate ranges on resume |
| `WorldSeed` | **Persisted** | Required for deterministic continuation from any snapshot tick |
| `SimulationClock.total_ticks` | **Persisted** | The only canonical time source (ADR-004) |
| `TerrainChunk` | **Persisted** | Terrain is static post-generation; snapshots must include it for complete world state |
| `ClimateChunk` | **Persisted** | Climate is updated each tick; current field values are required for continuation |
| `ResourceChunk` | **Persisted** | Resources are updated each tick; current field values are required for continuation |
| `EnergyAvailabilityChunk` | **Persisted** | Energy is updated each tick; current field values are required for continuation |
| `ChunkCoord` | **Persisted** | Required to map chunk data back to ECS entities on load |
| `SeasonState` | **Reconstructed** | Derived entirely from `total_ticks` and `WorldConfig`; no independent truth |
| `WorldBounds` | **Reconstructed** | Derived entirely from `WorldConfig` at startup |
| `SimulationClock.tick_duration_hours` | **Reconstructed** | Constant; always 1 in Phase 1 |

### Why Terrain Is Persisted

Terrain is deterministic and could theoretically be regenerated from seed and config. However, the snapshot spec explicitly requires terrain state to be included. Including terrain eliminates the regeneration cost on load and ensures the snapshot is a self-contained resumable unit.

### Why SeasonState Is Not Persisted

`SeasonState` is a derived resource. The architecture (Milestone 7) explicitly defines it as reconstructible from `total_ticks` and `WorldConfig`. Persisting it would create a redundant truth source that could diverge from the canonical clock. On load, the load path calls `SeasonState::derive(total_ticks, &config)` to reconstruct it.

---

## Systems

All systems run under the `PersistenceBoundary` schedule in the following order:

### 1. `detect_snapshot_due`

Listed explicitly in `PHASE1_WORLD_TECH_SPEC.md` as system #1 in `PersistenceBoundary` execution order. Reads `SnapshotConfig` and `SimulationClock`. Emits `SnapshotRequested` when `snapshot_interval_ticks > 0` and `total_ticks % snapshot_interval_ticks == 0`.

Setting `snapshot_interval_ticks = 0` disables automatic snapshots. Manual triggers via `SnapshotRequested` still work.

```
Reads:  Res<SnapshotConfig>, Res<SimulationClock>
Writes: EventWriter<SnapshotRequested>
```

### 2. `handle_snapshot_requests`

Reads `SnapshotRequested` events. For each pending request, runs the snapshot pipeline: `build_world_snapshot` → `write_world_snapshot`. Emits `SnapshotCompleted` on success. No intermediate state is stored as a resource; the snapshot struct is constructed, written, and dropped within this system.

Filename is derived from `total_ticks` alone: `snapshot_{total_ticks:010}.json`. No output path is stored in the event or in any resource.

```
Reads:  EventReader<SnapshotRequested>, Res<WorldConfig>, Res<WorldSeed>,
        Res<SimulationClock>, Res<SnapshotConfig>,
        Query<(&ChunkCoord, &TerrainChunk, &ClimateChunk, &ResourceChunk, &EnergyAvailabilityChunk)>
Writes: EventWriter<SnapshotCompleted>
Side effects: File I/O (write JSON to disk)
```

### 3. Snapshot Construction

`build_world_snapshot` is a pure function called from `handle_snapshot_requests`:

```rust
pub fn build_world_snapshot(
    config: &WorldConfig,
    seed: &WorldSeed,
    clock: &SimulationClock,
    schema_version: u32,
    chunks: &[(ChunkCoord, &TerrainChunk, &ClimateChunk, &ResourceChunk, &EnergyAvailabilityChunk)],
) -> WorldSnapshot
```

Chunks are sorted by `(coord.y, coord.x)` before inclusion. Returns a fully populated `WorldSnapshot`.

### 4. Snapshot Write

`write_world_snapshot` is called from `handle_snapshot_requests`:

```rust
pub fn write_world_snapshot(
    snapshot: &WorldSnapshot,
    output_directory: &str,
) -> Result<(), SnapshotError>
```

Filename format: `snapshot_{total_ticks:010}.json`

On success, returns `Ok(())`. On I/O failure, returns `Err(SnapshotError)`. Does not panic. Does not mutate simulation state. The caller (`handle_snapshot_requests`) decides whether to emit `SnapshotCompleted`.

### 5. Emit `SnapshotCompleted`

Emitted inside `handle_snapshot_requests` on successful write. Carries `total_ticks` only. There is no separate `emit_snapshot_completed` system; this is an inline action within system #2.

**Note on spec system order:** The tech spec lists `emit_snapshot_completed` as step #5 and `build_world_snapshot` / `write_world_snapshot` as steps #3 and #4. In the implementation, these are helper functions called from `handle_snapshot_requests` (step #2) rather than separate registered systems. This consolidation is intentional: separate ECS systems for construction and write would require passing a `WorldSnapshot` value between systems through a resource, which would violate ADR-001 (resources are for shared context, not inter-system data passing). A pure function call within one system is simpler and correct.

### 6. `clear_persisted_dirty_markers`

Reserved for future dirty-chunk tracking. Not required in Phase 1 but the system slot is registered for architectural completeness per the tech spec system order.

---

## Load Path

The load path is not a Bevy ECS system. It is a standalone function in `persistence/` called by test harnesses and future runner infrastructure.

```rust
pub fn load_world_snapshot(path: &str) -> Result<WorldSnapshot, SnapshotError>
```

After loading a snapshot, the caller reconstructs the ECS world:

1. Insert `WorldConfig` from `snapshot.config`.
2. Insert `WorldSeed` from `snapshot.seed`.
3. Insert `SimulationClock` from `snapshot.total_ticks`.
4. Reconstruct `WorldBounds` from `WorldConfig`.
5. Reconstruct `SeasonState` via `SeasonState::derive(total_ticks, &config)`.
6. Spawn chunk entities from `snapshot.chunks`, attaching `ChunkCoord`, `TerrainChunk`, `ClimateChunk`, `ResourceChunk`, and `EnergyAvailabilityChunk` to each.
7. Mark all loaded chunks as `Generated`.

No regeneration is needed. The snapshot contains all required field data.

---

## Error Design

```rust
#[derive(Debug)]
pub enum SnapshotError {
    /// Snapshot schema version does not match the current engine version.
    SchemaMismatch { found: u32, expected: u32 },

    /// File system I/O failure during write or read.
    IoError(std::io::Error),

    /// JSON serialization or deserialization failure.
    SerializationError(serde_json::Error),

    /// Loaded snapshot is structurally incomplete.
    IncompleteSnapshot { detail: &'static str },
}
```

**Note on `OutputDirectoryError`:** Removed. Directory creation errors are I/O errors and are already covered by `IoError`. A separate variant adds no diagnostic value that `IoError` does not already provide.

`SnapshotError` is not a `ValidationError`. Persistence errors are I/O and schema concerns, not simulation invariant violations.

---

## Determinism Review

| Concern | Resolution |
|---|---|
| Chunk iteration order affects JSON output | Chunks sorted by `(coord.y, coord.x)` before serialization |
| `f32` serialization consistency | `serde_json` uses shortest-round-trip representation; continuation tests compare loaded simulation state, not raw JSON text |
| `SeasonState` on resume | Reconstructed from `total_ticks` and `WorldConfig`; identical to a continuous run by construction |
| RNG state continuity | No per-chunk RNG state exists at runtime after generation; terrain and climate are computed from seeds + coordinates as pure functions; no running RNG stream needs to be saved |
| Save/load equivalence | A+B tick run must equal N tick continuous run; verified by the save/load equivalence test |

**RNG Note:** Genesis derives chunk seeds from `(domain_seed, chunk_x, chunk_y)` using deterministic hash functions. There is no running per-chunk RNG stream that needs to be captured. The seed in the snapshot is sufficient to reproduce any RNG-dependent generation if needed. This satisfies ADR-002.

---

## Validation on Load

After loading, the caller runs `validate_world_on_startup` (from Milestone 8) against the reconstructed ECS world before enabling `FixedSimulationTick`. This ensures:

- Schema version is verified before reconstruction begins.
- All loaded chunk field data passes the standard invariant checks.
- `SeasonState` consistency is confirmed by the startup validator.
- No special load-path validation is needed; Milestone 8 validation is sufficient.

---

## Module Layout

```
engine/src/persistence/
    mod.rs          -- module root; exports public API
    snapshot.rs     -- WorldSnapshot, ChunkSnapshot structs
    errors.rs       -- SnapshotError enum
    systems.rs      -- ECS systems: detect_snapshot_due, handle_snapshot_requests
    io.rs           -- build_world_snapshot, write_world_snapshot, load_world_snapshot
```

`lib.rs` gains:

```rust
pub mod persistence;
```

`app/events.rs` gains `SnapshotRequested` and `SnapshotCompleted`.

`app/plugins.rs` registers `SnapshotConfig` and events.

`app/mod.rs` registers persistence systems in `PersistenceBoundary`.

---

## Testing Strategy

### Save/Load Equivalence Test

Run a generated world for A ticks. Save snapshot. Load snapshot into a new ECS world. Run B more ticks. Compare final state field-by-field against a continuous N = A + B tick run.

- Validates: `WorldConfig`, `SimulationClock.total_ticks`, all chunk fields for all chunk coords.
- `SeasonState` is compared via `SeasonState::derive(total_ticks, &config)` on both sides.
- Test world: `256 x 256` (8x8 = 64 chunks) for speed.

### Round-Trip Serialization Test

Serialize a `WorldSnapshot` to JSON. Deserialize it. Assert field-by-field equality.

Verifies that `serde_json` round-trips all `f32` fields without information loss sufficient to cause divergence.

### Schema Version Rejection Test

Attempt to load a snapshot with a mismatched `schema_version`. Assert `SnapshotError::SchemaMismatch` is returned.

### Snapshot Does Not Mutate Simulation State Test

Run a tick. Take a snapshot. Verify that all resource and component values are identical before and after the snapshot write.

### Automatic Interval Test

Configure `snapshot_interval_ticks = 10`. Run 30 ticks. Assert that exactly 3 `SnapshotCompleted` events were emitted (at ticks 10, 20, 30).

---

## Verification Plan

### Automated Tests

```
cargo test -p genesis-engine
```

All 66 existing tests must continue to pass. New persistence tests added in `persistence/systems.rs` and `persistence/io.rs`.

### Manual Verification

- Inspect a written snapshot JSON file for human readability.
- Confirm field order is stable across two runs with the same seed.
- Confirm `SeasonState` on resume matches a continuous run at the same tick.

---

## Minimum Architecture That Fully Satisfies Milestone 9

The following are strictly required by the roadmap, tech spec, and ADRs:

| Item | Required By |
|---|---|
| `WorldSnapshot` struct | Roadmap deliverable: JSON snapshot schema |
| `ChunkSnapshot` struct | Roadmap deliverable: snapshot construction from ECS state |
| `schema_version: u32` in `WorldSnapshot` | Roadmap deliverable: snapshot schema version |
| `SnapshotConfig` resource (`snapshot_interval_ticks`, `schema_version`) | Tech spec: defined as required Phase 1 resource |
| `SnapshotRequested` event | Tech spec: required Phase 1 event |
| `SnapshotCompleted { total_ticks }` event | Tech spec: required Phase 1 event |
| `detect_snapshot_due` system | Tech spec: system #1 in PersistenceBoundary order |
| `handle_snapshot_requests` system | Tech spec: system #2 in PersistenceBoundary order |
| `clear_persisted_dirty_markers` system slot | Tech spec: system #6 in PersistenceBoundary order |
| `build_world_snapshot` pure function | Roadmap deliverable: snapshot construction |
| `write_world_snapshot` function | Roadmap deliverable: snapshot construction |
| `load_world_snapshot` function | Roadmap deliverable: load path for deterministic continuation |
| Chunk sort by `(coord.y, coord.x)` | ADR-002: stable iteration required |
| `SnapshotError` enum | Required for load path and write failure handling |
| Save/load equivalence test | Roadmap success criterion |

The following are **optional** for Milestone 9 (not required by roadmap, spec, or ADRs):

| Item | Status | Reason |
|---|---|---|
| `output_directory` in `SnapshotConfig` | Optional | Not in tech spec; can be a constant or hardcoded default for Phase 1 |
| `output_path` in `SnapshotCompleted` | Removed | Not required by spec; `total_ticks` is sufficient |
| `OutputDirectoryError` variant | Removed | Covered by `IoError` |
| Separate `emit_snapshot_completed` system | Removed | Inlined into `handle_snapshot_requests` to avoid resource-passing pattern |
| `snapshot_target_policy` field | Deferred | Tech spec mentions it; only one target exists in Phase 1 |

---

## Design Lock Verdict

**APPROVED FOR IMPLEMENTATION.**

Revision summary applied:

1. **`SnapshotConfig`:** `output_directory` removed (not in tech spec). Field set reduced to `snapshot_interval_ticks` and `schema_version`. Target policy deferred.
2. **Automatic scheduling:** Kept. `detect_snapshot_due` is listed explicitly in the tech spec system order and cannot be removed. Setting `snapshot_interval_ticks = 0` disables it cleanly.
3. **`SnapshotCompleted`:** Simplified to `{ total_ticks: u32 }`. `output_path` removed — not required by roadmap or spec.
4. **Module layout:** `snapshot.rs` is the correct home for `WorldSnapshot` and `ChunkSnapshot`. No separate `schema.rs` needed at this scale. Layout unchanged.
5. **`OutputDirectoryError` variant:** Removed. Covered by `IoError`.
6. **System consolidation:** Construction and write steps are helper functions within `handle_snapshot_requests`, not separate ECS systems. Justified by ADR-001: inter-system data passing via resource would violate the resource ownership contract.

All remaining items are justified by roadmap deliverables, tech spec system order, or ADR constraints. No new build dependencies are introduced.

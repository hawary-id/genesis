//! Snapshot-based persistence for the Genesis simulation engine.
//!
//! This module implements the `PersistenceBoundary` concerns:
//! snapshot construction, file I/O, and load path for deterministic continuation.
//!
//! # Scope
//!
//! Persistence exists solely to support reproducibility, resume,
//! and save/load equivalence testing in Phase 1.
//!
//! It does not implement history, event sourcing, per-tick logs,
//! or database-first state.
//!
//! # Architecture
//!
//! Persistence systems read stable ECS state at `PersistenceBoundary`.
//! They do not mutate any simulation component or resource.
//!
//! # Schema Version
//!
//! All snapshot files carry a `schema_version` field.
//! Loading a snapshot with a mismatched version returns [`errors::SnapshotError::SchemaMismatch`].

pub mod errors;
pub mod io;
pub mod snapshot;
pub mod systems;

pub use errors::SnapshotError;
pub use io::{
    build_world_snapshot, load_world_snapshot, reconstruct_world_from_snapshot,
    write_world_snapshot,
};
pub use snapshot::{AgentSnapshot, ChunkSnapshot, WorldSnapshot};

/// Current snapshot schema version.
///
/// Increment this constant when the snapshot schema changes in a breaking way.
/// All written snapshots embed this version. Load path rejects mismatches.
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 3;

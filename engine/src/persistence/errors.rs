//! Snapshot error types for the Genesis persistence layer.

/// Errors that can occur during snapshot write, load, or schema validation.
///
/// `SnapshotError` is distinct from [`crate::validation::ValidationError`].
/// Persistence errors represent I/O and schema concerns, not simulation
/// invariant violations.
#[derive(Debug)]
pub enum SnapshotError {
    /// Snapshot schema version does not match the current engine version.
    ///
    /// The snapshot file was written by a different version of the engine
    /// and cannot be safely loaded.
    SchemaMismatch {
        /// Schema version found in the snapshot file.
        found: u32,
        /// Schema version expected by the current engine.
        expected: u32,
    },

    /// File system I/O failure during write or read.
    ///
    /// Covers directory creation failures, file write failures,
    /// and file read failures.
    IoError(std::io::Error),

    /// JSON serialization or deserialization failure.
    SerializationError(serde_json::Error),

    /// Loaded snapshot is structurally incomplete.
    ///
    /// Used when the snapshot deserializes successfully but is missing
    /// required content (e.g. zero chunks when chunks are expected).
    IncompleteSnapshot {
        /// Description of the structural violation.
        detail: &'static str,
    },
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::SchemaMismatch { found, expected } => {
                write!(
                    f,
                    "snapshot schema mismatch: found version {found}, expected {expected}"
                )
            }
            SnapshotError::IoError(e) => write!(f, "snapshot I/O error: {e}"),
            SnapshotError::SerializationError(e) => {
                write!(f, "snapshot serialization error: {e}")
            }
            SnapshotError::IncompleteSnapshot { detail } => {
                write!(f, "snapshot incomplete: {detail}")
            }
        }
    }
}

impl From<std::io::Error> for SnapshotError {
    fn from(e: std::io::Error) -> Self {
        SnapshotError::IoError(e)
    }
}

impl From<serde_json::Error> for SnapshotError {
    fn from(e: serde_json::Error) -> Self {
        SnapshotError::SerializationError(e)
    }
}

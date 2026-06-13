use crate::world::coord::ChunkCoord;
use serde::{Deserialize, Serialize};

/// Structured validation errors for the Genesis world simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationError {
    /// Clock ticks decreased or regression occurred.
    ClockRegression {
        /// Previous recorded tick count.
        previous: u32,
        /// Current tick count.
        current: u32,
    },

    /// The derived SeasonState does not match the active SeasonState resource.
    SeasonStateMismatch {
        /// Current simulation clock tick count.
        total_ticks: u32,
    },

    /// Basic structural inconsistency of a chunk.
    ChunkInconsistency {
        /// The chunk coordinate.
        coord: ChunkCoord,
        /// Description of the violation.
        detail: &'static str,
    },

    /// Cell-level terrain value out of range bounds.
    TerrainOutOfBounds {
        /// The chunk coordinate.
        coord: ChunkCoord,
        /// The terrain field name.
        field: &'static str,
        /// The violating cell value.
        value: f32,
    },

    /// Cell-level climate value out of range bounds.
    ClimateOutOfBounds {
        /// The chunk coordinate.
        coord: ChunkCoord,
        /// The climate field name.
        field: &'static str,
        /// The violating cell value.
        value: f32,
    },

    /// Cell-level resource value out of range bounds.
    ResourceOutOfBounds {
        /// The chunk coordinate.
        coord: ChunkCoord,
        /// The resource field name.
        field: &'static str,
        /// The violating cell value.
        value: f32,
    },

    /// Cell-level energy value out of range bounds.
    EnergyOutOfBounds {
        /// The chunk coordinate.
        coord: ChunkCoord,
        /// The energy field name.
        field: &'static str,
        /// The violating cell value.
        value: f32,
    },
}

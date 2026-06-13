//! World bounds resource.
//!
//! Stores validated coordinate boundaries derived from [`WorldConfig`].
//! Immutable after startup initialization.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::config::WorldConfig;

/// Validated coordinate boundaries for the Genesis simulation world.
///
/// Derived from [`WorldConfig`] during startup and stored as an ECS resource.
/// Systems that validate coordinates, spawn chunks, or perform boundary checks
/// read this resource instead of re-deriving bounds from [`WorldConfig`] each time.
///
/// Immutable after startup. Must not diverge from [`WorldConfig`] after initialization.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct WorldBounds {
    /// World width in cells.
    pub world_width: u32,
    /// World height in cells.
    pub world_height: u32,
    /// Number of chunks along the x axis.
    pub chunks_x: u32,
    /// Number of chunks along the y axis.
    pub chunks_y: u32,
    /// Chunk size in cells (chunks are square).
    pub chunk_size: u32,
}

impl WorldBounds {
    /// Derives world bounds from a [`WorldConfig`].
    ///
    /// Assumes chunk size divides world dimensions exactly.
    /// Non-divisible world dimensions are out of scope for Phase 1.
    pub fn from_config(config: &WorldConfig) -> Self {
        Self {
            world_width: config.world_width,
            world_height: config.world_height,
            chunks_x: config.world_width / config.chunk_size,
            chunks_y: config.world_height / config.chunk_size,
            chunk_size: config.chunk_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::create_test_config;

    #[test]
    fn world_bounds_from_default_config() {
        let config = WorldConfig::default();
        let bounds = WorldBounds::from_config(&config);

        assert_eq!(bounds.world_width, 512);
        assert_eq!(bounds.world_height, 512);
        assert_eq!(bounds.chunk_size, 32);
        assert_eq!(bounds.chunks_x, 16);
        assert_eq!(bounds.chunks_y, 16);
    }

    #[test]
    fn world_bounds_from_test_config() {
        let config = create_test_config();
        let bounds = WorldBounds::from_config(&config);

        assert_eq!(bounds.world_width, 256);
        assert_eq!(bounds.world_height, 256);
        assert_eq!(bounds.chunk_size, 32);
        assert_eq!(bounds.chunks_x, 8);
        assert_eq!(bounds.chunks_y, 8);
    }

    #[test]
    fn chunk_count_covers_entire_world() {
        let config = WorldConfig::default();
        let bounds = WorldBounds::from_config(&config);

        let total_cells_via_chunks =
            bounds.chunks_x * bounds.chunks_y * bounds.chunk_size * bounds.chunk_size;
        let total_cells_direct = bounds.world_width * bounds.world_height;

        assert_eq!(total_cells_via_chunks, total_cells_direct);
    }
}

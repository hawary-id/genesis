//! Coordinate types and conversion logic for the Genesis spatial model.
//!
//! Enforces non-negative boundaries and supports world-to-chunk, world-to-local,
//! and local-to-world transformations.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

/// Global address of a single grid cell in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldCoord {
    /// Global x coordinate.
    pub x: u32,
    /// Global y coordinate.
    pub y: u32,
}

impl WorldCoord {
    /// Creates a new `WorldCoord`.
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

/// Global address of a single chunk in the world chunk layout.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkCoord {
    /// Chunk grid x coordinate.
    pub x: u32,
    /// Chunk grid y coordinate.
    pub y: u32,
}

impl ChunkCoord {
    /// Creates a new `ChunkCoord`.
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

/// Address of a cell local to its parent chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalCoord {
    /// Local x offset inside the chunk [0, chunk_size - 1].
    pub x: u32,
    /// Local y offset inside the chunk [0, chunk_size - 1].
    pub y: u32,
}

impl LocalCoord {
    /// Creates a new `LocalCoord`.
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

/// Converts a global world cell coordinate to the chunk coordinate containing it.
pub fn world_to_chunk(coord: WorldCoord, chunk_size: u32) -> ChunkCoord {
    debug_assert!(chunk_size > 0, "chunk_size must be greater than zero");
    ChunkCoord::new(coord.x / chunk_size, coord.y / chunk_size)
}

/// Converts a global world cell coordinate to its offset inside its chunk.
pub fn world_to_local(coord: WorldCoord, chunk_size: u32) -> LocalCoord {
    debug_assert!(chunk_size > 0, "chunk_size must be greater than zero");
    LocalCoord::new(coord.x % chunk_size, coord.y % chunk_size)
}

/// Restores a global world cell coordinate from chunk and local coordinates.
pub fn chunk_local_to_world(chunk: ChunkCoord, local: LocalCoord, chunk_size: u32) -> WorldCoord {
    debug_assert!(chunk_size > 0, "chunk_size must be greater than zero");
    WorldCoord::new(
        chunk.x * chunk_size + local.x,
        chunk.y * chunk_size + local.y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_round_trip() {
        let chunk_size = 32;
        let test_coords = vec![
            WorldCoord::new(0, 0),
            WorldCoord::new(15, 29),
            WorldCoord::new(32, 32),
            WorldCoord::new(33, 64),
            WorldCoord::new(511, 511),
        ];

        for coord in test_coords {
            let chunk = world_to_chunk(coord, chunk_size);
            let local = world_to_local(coord, chunk_size);

            assert!(local.x < chunk_size);
            assert!(local.y < chunk_size);

            let reconstructed = chunk_local_to_world(chunk, local, chunk_size);
            assert_eq!(coord, reconstructed);
        }
    }

    #[test]
    fn world_to_chunk_boundaries() {
        let chunk_size = 32;

        let c0 = world_to_chunk(WorldCoord::new(31, 31), chunk_size);
        assert_eq!(c0, ChunkCoord::new(0, 0));

        let c1 = world_to_chunk(WorldCoord::new(32, 32), chunk_size);
        assert_eq!(c1, ChunkCoord::new(1, 1));
    }
}

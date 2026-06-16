use crate::world::coord::ChunkCoord;
use bevy_ecs::prelude::{Entity, Resource};

/// SpatialMap is a fast O(1) lookup structure that maps static ChunkCoords to their ECS Entities.
/// It assumes chunks are spawned in a fixed grid and never despawned during Phase 1-3.
#[derive(Resource, Default, Debug, Clone)]
pub struct SpatialMap {
    pub chunks: Vec<Option<Entity>>,
    pub width: u32,
    pub height: u32,
}

impl SpatialMap {
    /// Creates a new empty SpatialMap covering the specified chunk grid dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            chunks: vec![None; (width * height) as usize],
            width,
            height,
        }
    }

    /// Converts a 2D chunk coordinate into a 1D array index.
    fn index(&self, coord: ChunkCoord) -> usize {
        (coord.y * self.width + coord.x) as usize
    }

    /// Retrieves the ECS Entity ID for the chunk at the given coordinate, if it exists.
    pub fn get(&self, coord: ChunkCoord) -> Option<Entity> {
        if coord.x >= self.width || coord.y >= self.height {
            return None;
        }
        self.chunks.get(self.index(coord)).copied().flatten()
    }

    /// Assigns the given Entity ID to the specified chunk coordinate.
    pub fn set(&mut self, coord: ChunkCoord, entity: Entity) {
        if coord.x >= self.width || coord.y >= self.height {
            return;
        }
        let idx = self.index(coord);
        self.chunks[idx] = Some(entity);
    }

    /// Returns true if a valid chunk entity exists at the specified coordinate.
    pub fn contains(&self, coord: ChunkCoord) -> bool {
        self.get(coord).is_some()
    }
}

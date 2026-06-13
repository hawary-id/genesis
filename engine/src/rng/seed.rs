//! Deterministic world seed definition.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Root deterministic seed for a Genesis world.
///
/// All future randomness will originate from this seed.
///
/// Milestone 1 only defines the seed structure.
/// RNG stream derivation will be introduced in later milestones.
#[derive(Resource, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct WorldSeed {
    /// Root world seed.
    pub root_seed: u64,
}

impl WorldSeed {
    /// Creates a new world seed.
    pub fn new(root_seed: u64) -> Self {
        Self { root_seed }
    }
}

/// Domain salt for terrain seed derivation.
///
/// Encodes "terrain\0" as a little-endian `u64`.
pub const TERRAIN_DOMAIN_SALT: u64 = 0x74657272_61696e00;

/// Domain salt for resource seed derivation.
///
/// Encodes "resource\0" as a little-endian `u64`.
pub const RESOURCE_DOMAIN_SALT: u64 = 0x7265736f_75726365;

/// Derives the terrain domain seed from the root seed.
pub fn derive_terrain_seed(root_seed: u64) -> u64 {
    root_seed.wrapping_add(TERRAIN_DOMAIN_SALT)
}

/// Derives the resource domain seed from the root seed.
pub fn derive_resource_seed(root_seed: u64) -> u64 {
    root_seed.wrapping_add(RESOURCE_DOMAIN_SALT)
}

/// Derives a deterministic seed for a single chunk.
///
/// Derived from chunk coordinates and the terrain domain seed.
pub fn derive_chunk_seed(terrain_seed: u64, chunk_x: u32, chunk_y: u32) -> u64 {
    terrain_seed
        .wrapping_add((chunk_x as u64).wrapping_mul(0x9e3779b97f4a7c15))
        .wrapping_add((chunk_y as u64).wrapping_mul(0x6c62272e07bb0142))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_seed_creation() {
        let seed = WorldSeed::new(12345);

        assert_eq!(seed.root_seed, 12345);
    }

    #[test]
    fn default_seed_is_zero() {
        let seed = WorldSeed::default();

        assert_eq!(seed.root_seed, 0);
    }

    #[test]
    fn terrain_seed_derivation_is_deterministic() {
        let root = 42;
        let derived_a = derive_terrain_seed(root);
        let derived_b = derive_terrain_seed(root);
        assert_eq!(derived_a, derived_b);
        assert_eq!(derived_a, root + TERRAIN_DOMAIN_SALT);
    }

    #[test]
    fn resource_seed_derivation_is_deterministic() {
        let root = 42;
        let derived_a = derive_resource_seed(root);
        let derived_b = derive_resource_seed(root);
        assert_eq!(derived_a, derived_b);
        assert_eq!(derived_a, root.wrapping_add(RESOURCE_DOMAIN_SALT));
    }

    #[test]
    fn chunk_seed_derivation_is_deterministic() {
        let terrain_seed = derive_terrain_seed(42);
        let seed_a = derive_chunk_seed(terrain_seed, 3, 5);
        let seed_b = derive_chunk_seed(terrain_seed, 3, 5);
        let seed_c = derive_chunk_seed(terrain_seed, 4, 5);

        assert_eq!(seed_a, seed_b);
        assert_ne!(seed_a, seed_c);
    }
}

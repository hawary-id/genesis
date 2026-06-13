//! Deterministic world seed definition.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Root deterministic seed for a Genesis world.
///
/// All future randomness will originate from this seed.
///
/// Milestone 1 only defines the seed structure.
/// RNG stream derivation will be introduced in later milestones.
#[derive(Resource, Debug, Clone, Copy, Serialize, Deserialize)]
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

impl Default for WorldSeed {
    fn default() -> Self {
        Self { root_seed: 0 }
    }
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
}

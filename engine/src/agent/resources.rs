//! ECS resources for agents in Genesis.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

/// State-tracking stable identifier generator resource.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StableIdGenerator {
    next_id: u64,
}

impl StableIdGenerator {
    /// Creates a new stable identifier generator starting from `1`.
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    /// Allocates and returns the next sequential unique stable ID.
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }
}

impl Default for StableIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_allocates_sequentially() {
        let mut gen = StableIdGenerator::new();
        assert_eq!(gen.next_id(), 1);
        assert_eq!(gen.next_id(), 2);
        assert_eq!(gen.next_id(), 3);
    }
}

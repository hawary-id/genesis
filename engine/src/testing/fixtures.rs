//! Test fixtures and helpers for Genesis engine tests.

use crate::config::WorldConfig;
use crate::rng::WorldSeed;

/// Returns a deterministic test configuration.
///
/// Uses a smaller world size to keep future tests fast.
pub fn create_test_config() -> WorldConfig {
    WorldConfig {
        world_width: 256,
        world_height: 256,
        chunk_size: 32,

        day_length_ticks: 24,
        season_length_days: 90,
        seasons_per_year: 4,

        generation_version: 1,
    }
}

/// Returns a deterministic test seed.
pub fn create_test_seed() -> WorldSeed {
    WorldSeed::new(987_654_321)
}

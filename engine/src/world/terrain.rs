//! Terrain component and deterministic terrain generation functions.
//!
//! # TerrainChunk
//!
//! Each chunk entity carries one [`TerrainChunk`] component storing five dense
//! cell arrays: elevation, slope, water depth, soil depth, and soil fertility.
//! Cells are not ECS entities. They are indexed by `local_y * chunk_size + local_x`.
//!
//! # Generation Model
//!
//! Terrain is generated from a deterministic seed per chunk using hash-based
//! value noise. The algorithm:
//!
//! 1. A coarse grid of noise lattice points (spaced `NOISE_SCALE` cells apart)
//!    is seeded from `(chunk_seed, lattice_x, lattice_y)`.
//! 2. Each cell's elevation is computed by bilinear interpolation of its four
//!    surrounding lattice values.
//! 3. A second octave at half the scale is blended in for more terrain variety.
//! 4. Slope is derived from in-chunk elevation neighbors only. Boundary cells
//!    treat missing neighbors as equal elevation (zero gradient).
//! 5. Water depth is assigned to cells at or below `WorldConfig.sea_level`.
//! 6. Soil depth is inversely proportional to slope.
//! 7. Soil fertility is proportional to soil depth and water proximity.
//!
//! All operations are pure functions of seed and coordinates. No global state.
//! No `thread_rng()`. No wall-clock entropy.

use bevy_ecs::prelude::Component;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::config::WorldConfig;
use crate::rng::derive_chunk_seed;

use super::coord::ChunkCoord;

/// Number of cells between coarse noise lattice points.
/// Smaller values produce more jagged terrain; larger values are smoother.
const NOISE_SCALE: u32 = 8;

/// Terrain fields for all cells in one chunk.
///
/// Each field is a `Vec<f32>` of length `chunk_size * chunk_size`.
/// Cell index: `local_y * chunk_size + local_x`.
///
/// All values must remain within the ranges defined in [`WorldConfig`].
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct TerrainChunk {
    /// Elevation of each cell. Range: `[elevation_min, elevation_max]`.
    pub elevation: Vec<f32>,

    /// Slope magnitude of each cell. Range: `[0.0, slope_max]`.
    /// Derived from elevation during generation. Stored for read efficiency.
    /// Boundary approximation: edge cells treat the chunk border as flat.
    pub slope: Vec<f32>,

    /// Water depth of each cell. Range: `[0.0, water_depth_max]`.
    /// A value of `0.0` means the cell is dry.
    pub water_depth: Vec<f32>,

    /// Soil depth of each cell. Range: `[0.0, soil_depth_max]`.
    /// Inversely proportional to slope: steep cells have thin soil.
    pub soil_depth: Vec<f32>,

    /// Soil fertility of each cell. Range: `[0.0, soil_fertility_max]`.
    /// Proportional to soil depth and water proximity.
    pub soil_fertility: Vec<f32>,
}

/// Computes the flat index into a chunk's cell array.
///
/// # Panics (debug only)
///
/// Panics in debug builds if `local_x >= chunk_size` or `local_y >= chunk_size`.
pub fn cell_index(local_x: u32, local_y: u32, chunk_size: u32) -> usize {
    debug_assert!(
        local_x < chunk_size,
        "local_x {} >= chunk_size {}",
        local_x,
        chunk_size
    );
    debug_assert!(
        local_y < chunk_size,
        "local_y {} >= chunk_size {}",
        local_y,
        chunk_size
    );
    (local_y * chunk_size + local_x) as usize
}

/// Generates a complete [`TerrainChunk`] for the given chunk coordinate.
///
/// This is a pure function — it depends only on `chunk`, `terrain_seed`,
/// and `config`. Given the same inputs it always returns the same output.
///
/// # Algorithm
///
/// Two-pass:
/// 1. Generate elevation and water/soil fields from the noise model.
/// 2. Compute slope from the generated elevation field.
pub fn generate_terrain_chunk(
    chunk: ChunkCoord,
    terrain_seed: u64,
    config: &WorldConfig,
) -> TerrainChunk {
    let chunk_size = config.chunk_size;
    let n = (chunk_size * chunk_size) as usize;

    let chunk_seed = derive_chunk_seed(terrain_seed, chunk.x, chunk.y);

    // Pass 1 — elevation, water, soil depth, soil fertility.
    let mut elevation = vec![0.0f32; n];
    let mut water_depth = vec![0.0f32; n];
    let mut soil_depth = vec![0.0f32; n];
    let mut soil_fertility = vec![0.0f32; n];

    for ly in 0..chunk_size {
        for lx in 0..chunk_size {
            let idx = cell_index(lx, ly, chunk_size);

            // Global cell coordinates (needed for cross-chunk noise coherence).
            let gx = chunk.x * chunk_size + lx;
            let gy = chunk.y * chunk_size + ly;

            let elev = sample_elevation(gx, gy, terrain_seed, config);
            let elev = elev.clamp(config.elevation_min, config.elevation_max);
            elevation[idx] = elev;

            let wd = compute_water_depth(elev, config);
            water_depth[idx] = wd;

            // Soil depth and fertility require slope, computed in pass 2.
            // Store intermediate: soil fields are finalized after slope pass.
            // Use elevation as a placeholder signal for now.
            // They are overwritten in pass 2.
            let _ = chunk_seed; // explicitly suppress unused warning until pass 2
            soil_depth[idx] = 0.0;
            soil_fertility[idx] = 0.0;
        }
    }

    // Pass 2 — slope from elevation, then soil fields from slope.
    let mut slope = vec![0.0f32; n];

    for ly in 0..chunk_size {
        for lx in 0..chunk_size {
            let idx = cell_index(lx, ly, chunk_size);

            let s = compute_slope(&elevation, lx, ly, chunk_size);
            let s = s.clamp(0.0, config.slope_max);
            slope[idx] = s;

            let sd = compute_soil_depth(s, config);
            soil_depth[idx] = sd;

            let wd = water_depth[idx];
            let sf = compute_soil_fertility(sd, wd, config);
            soil_fertility[idx] = sf;
        }
    }

    TerrainChunk {
        elevation,
        slope,
        water_depth,
        soil_depth,
        soil_fertility,
    }
}

// ---------------------------------------------------------------------------
// Pure terrain computation functions
// ---------------------------------------------------------------------------

/// Samples elevation at global cell coordinates using two-octave value noise.
///
/// Pure function of `(gx, gy, terrain_seed, config)`.
fn sample_elevation(gx: u32, gy: u32, terrain_seed: u64, config: &WorldConfig) -> f32 {
    // Octave 1 — coarse features at NOISE_SCALE.
    let coarse = value_noise_sample(
        gx,
        gy,
        terrain_seed,
        NOISE_SCALE,
        config.elevation_min,
        config.elevation_max,
    );

    // Octave 2 — fine detail at half the scale, blended at 25%.
    let fine_scale = (NOISE_SCALE / 2).max(1);
    let fine = value_noise_sample(
        gx,
        gy,
        terrain_seed.wrapping_add(1),
        fine_scale,
        config.elevation_min,
        config.elevation_max,
    );

    // Blend: 75% coarse + 25% fine, then remap to full range.
    let blended = coarse * 0.75 + fine * 0.25;
    blended.clamp(config.elevation_min, config.elevation_max)
}

/// Samples a value-noise field at global coordinate `(gx, gy)`.
///
/// Divides the world into a lattice with spacing `scale`. Each lattice point
/// is seeded deterministically and produces a height. Cell values are
/// bilinearly interpolated from the four surrounding lattice points.
///
/// Pure function. All inputs are deterministic.
fn value_noise_sample(gx: u32, gy: u32, seed: u64, scale: u32, min_val: f32, max_val: f32) -> f32 {
    // Lattice coordinates.
    let lx0 = gx / scale;
    let ly0 = gy / scale;
    let lx1 = lx0 + 1;
    let ly1 = ly0 + 1;

    // Fractional position within the lattice cell.
    let fx = (gx % scale) as f32 / scale as f32;
    let fy = (gy % scale) as f32 / scale as f32;

    // Heights at the four surrounding lattice points.
    let h00 = lattice_height(lx0, ly0, seed, min_val, max_val);
    let h10 = lattice_height(lx1, ly0, seed, min_val, max_val);
    let h01 = lattice_height(lx0, ly1, seed, min_val, max_val);
    let h11 = lattice_height(lx1, ly1, seed, min_val, max_val);

    // Bilinear interpolation.
    let h0 = h00 + fx * (h10 - h00);
    let h1 = h01 + fx * (h11 - h01);
    h0 + fy * (h1 - h0)
}

/// Returns a deterministic height for a noise lattice point.
///
/// Derives a per-point seed from the lattice coordinate and domain seed,
/// then draws one `f32` in `[min_val, max_val]`.
fn lattice_height(lx: u32, ly: u32, seed: u64, min_val: f32, max_val: f32) -> f32 {
    let point_seed = derive_chunk_seed(seed, lx, ly);
    let mut rng = ChaCha8Rng::seed_from_u64(point_seed);
    let t: f32 = rng.gen(); // uniform [0.0, 1.0)
    min_val + t * (max_val - min_val)
}

/// Computes slope magnitude for a cell using in-chunk neighbors only.
///
/// Uses a central difference gradient approximation:
/// `slope = sqrt(dx² + dy²) / 2`
///
/// Boundary cells treat missing neighbors as equal elevation (zero gradient).
/// This is a documented Phase 1 approximation.
pub fn compute_slope(elevation: &[f32], local_x: u32, local_y: u32, chunk_size: u32) -> f32 {
    let get = |x: i32, y: i32| -> f32 {
        if x < 0 || y < 0 || x >= chunk_size as i32 || y >= chunk_size as i32 {
            // Boundary approximation: treat as flat.
            let cx = local_x.min(chunk_size - 1);
            let cy = local_y.min(chunk_size - 1);
            elevation[cell_index(cx, cy, chunk_size)]
        } else {
            elevation[cell_index(x as u32, y as u32, chunk_size)]
        }
    };

    let lx = local_x as i32;
    let ly = local_y as i32;

    let dx = get(lx + 1, ly) - get(lx - 1, ly);
    let dy = get(lx, ly + 1) - get(lx, ly - 1);

    ((dx * dx + dy * dy) as f32).sqrt() / 2.0
}

/// Computes water depth for a cell based on its elevation.
///
/// Cells at or below `sea_level` receive depth proportional to how far below
/// they are. Cells above sea level are dry (`0.0`).
fn compute_water_depth(elevation: f32, config: &WorldConfig) -> f32 {
    if elevation <= config.sea_level {
        let depth = (config.sea_level - elevation) / config.sea_level.max(f32::EPSILON);
        depth.clamp(0.0, config.water_depth_max)
    } else {
        0.0
    }
}

/// Computes soil depth inversely proportional to slope.
///
/// Steep cells have thin soil; flat cells have deep soil.
fn compute_soil_depth(slope: f32, config: &WorldConfig) -> f32 {
    let normalized_slope = (slope / config.slope_max.max(f32::EPSILON)).clamp(0.0, 1.0);
    let depth = config.soil_depth_max * (1.0 - normalized_slope);
    depth.clamp(0.0, config.soil_depth_max)
}

/// Computes soil fertility proportional to soil depth and water availability.
fn compute_soil_fertility(soil_depth: f32, water_depth: f32, config: &WorldConfig) -> f32 {
    let depth_factor = (soil_depth / config.soil_depth_max.max(f32::EPSILON)).clamp(0.0, 1.0);
    let water_factor = if water_depth > 0.0 { 0.8 } else { 0.4 };
    let fertility = config.soil_fertility_max * depth_factor * water_factor;
    fertility.clamp(0.0, config.soil_fertility_max)
}

use crate::validation::ValidationError;

/// Validates all cell values inside the terrain chunk against limits specified in [`WorldConfig`].
pub fn validate_terrain_chunk(
    coord: &ChunkCoord,
    terrain: &TerrainChunk,
    config: &WorldConfig,
) -> Result<(), ValidationError> {
    let chunk_size = config.chunk_size as usize;
    let expected_len = chunk_size * chunk_size;

    if terrain.elevation.len() != expected_len {
        return Err(ValidationError::ChunkInconsistency {
            coord: *coord,
            detail: "elevation array length mismatch",
        });
    }
    if terrain.slope.len() != expected_len {
        return Err(ValidationError::ChunkInconsistency {
            coord: *coord,
            detail: "slope array length mismatch",
        });
    }
    if terrain.water_depth.len() != expected_len {
        return Err(ValidationError::ChunkInconsistency {
            coord: *coord,
            detail: "water_depth array length mismatch",
        });
    }
    if terrain.soil_depth.len() != expected_len {
        return Err(ValidationError::ChunkInconsistency {
            coord: *coord,
            detail: "soil_depth array length mismatch",
        });
    }
    if terrain.soil_fertility.len() != expected_len {
        return Err(ValidationError::ChunkInconsistency {
            coord: *coord,
            detail: "soil_fertility array length mismatch",
        });
    }

    for &val in &terrain.elevation {
        if val < config.elevation_min || val > config.elevation_max {
            return Err(ValidationError::TerrainOutOfBounds {
                coord: *coord,
                field: "elevation",
                value: val,
            });
        }
    }

    for &val in &terrain.slope {
        if val < 0.0 || val > config.slope_max {
            return Err(ValidationError::TerrainOutOfBounds {
                coord: *coord,
                field: "slope",
                value: val,
            });
        }
    }

    for &val in &terrain.water_depth {
        if val < 0.0 || val > config.water_depth_max {
            return Err(ValidationError::TerrainOutOfBounds {
                coord: *coord,
                field: "water_depth",
                value: val,
            });
        }
    }

    for &val in &terrain.soil_depth {
        if val < 0.0 || val > config.soil_depth_max {
            return Err(ValidationError::TerrainOutOfBounds {
                coord: *coord,
                field: "soil_depth",
                value: val,
            });
        }
    }

    for &val in &terrain.soil_fertility {
        if val < 0.0 || val > config.soil_fertility_max {
            return Err(ValidationError::TerrainOutOfBounds {
                coord: *coord,
                field: "soil_fertility",
                value: val,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::derive_terrain_seed;

    fn test_config() -> WorldConfig {
        WorldConfig {
            world_width: 256,
            world_height: 256,
            chunk_size: 32,
            ..WorldConfig::default()
        }
    }

    #[test]
    fn cell_index_first_cell() {
        assert_eq!(cell_index(0, 0, 32), 0);
    }

    #[test]
    fn cell_index_last_cell() {
        assert_eq!(cell_index(31, 31, 32), 1023);
    }

    #[test]
    fn cell_index_row_major() {
        // Row 1, col 0 should be at index chunk_size.
        assert_eq!(cell_index(0, 1, 32), 32);
    }

    #[test]
    fn generate_terrain_chunk_produces_correct_size() {
        let config = test_config();
        let terrain_seed = derive_terrain_seed(42);
        let chunk = ChunkCoord::new(0, 0);
        let tc = generate_terrain_chunk(chunk, terrain_seed, &config);

        let expected_len = (config.chunk_size * config.chunk_size) as usize;
        assert_eq!(tc.elevation.len(), expected_len);
        assert_eq!(tc.slope.len(), expected_len);
        assert_eq!(tc.water_depth.len(), expected_len);
        assert_eq!(tc.soil_depth.len(), expected_len);
        assert_eq!(tc.soil_fertility.len(), expected_len);
    }

    #[test]
    fn elevation_within_configured_range() {
        let config = test_config();
        let terrain_seed = derive_terrain_seed(42);

        for cx in 0..8u32 {
            for cy in 0..8u32 {
                let tc = generate_terrain_chunk(ChunkCoord::new(cx, cy), terrain_seed, &config);
                for &e in &tc.elevation {
                    assert!(
                        e >= config.elevation_min && e <= config.elevation_max,
                        "elevation {} out of range [{}, {}]",
                        e,
                        config.elevation_min,
                        config.elevation_max
                    );
                }
            }
        }
    }

    #[test]
    fn slope_within_configured_range() {
        let config = test_config();
        let terrain_seed = derive_terrain_seed(42);

        for cx in 0..8u32 {
            for cy in 0..8u32 {
                let tc = generate_terrain_chunk(ChunkCoord::new(cx, cy), terrain_seed, &config);
                for &s in &tc.slope {
                    assert!(
                        s >= 0.0 && s <= config.slope_max,
                        "slope {} out of range [0.0, {}]",
                        s,
                        config.slope_max
                    );
                }
            }
        }
    }

    #[test]
    fn water_depth_non_negative() {
        let config = test_config();
        let terrain_seed = derive_terrain_seed(42);
        let tc = generate_terrain_chunk(ChunkCoord::new(0, 0), terrain_seed, &config);

        for &w in &tc.water_depth {
            assert!(w >= 0.0, "water_depth {} is negative", w);
        }
    }

    #[test]
    fn soil_fields_non_negative() {
        let config = test_config();
        let terrain_seed = derive_terrain_seed(42);
        let tc = generate_terrain_chunk(ChunkCoord::new(0, 0), terrain_seed, &config);

        for &sd in &tc.soil_depth {
            assert!(sd >= 0.0);
        }
        for &sf in &tc.soil_fertility {
            assert!(sf >= 0.0);
        }
    }

    #[test]
    fn generation_is_deterministic() {
        let config = test_config();
        let terrain_seed = derive_terrain_seed(12345);
        let chunk = ChunkCoord::new(3, 5);

        let a = generate_terrain_chunk(chunk, terrain_seed, &config);
        let b = generate_terrain_chunk(chunk, terrain_seed, &config);

        assert_eq!(a.elevation, b.elevation);
        assert_eq!(a.slope, b.slope);
        assert_eq!(a.water_depth, b.water_depth);
        assert_eq!(a.soil_depth, b.soil_depth);
        assert_eq!(a.soil_fertility, b.soil_fertility);
    }

    #[test]
    fn different_chunks_produce_different_terrain() {
        let config = test_config();
        let terrain_seed = derive_terrain_seed(42);

        let a = generate_terrain_chunk(ChunkCoord::new(0, 0), terrain_seed, &config);
        let b = generate_terrain_chunk(ChunkCoord::new(1, 0), terrain_seed, &config);

        // Terrain should differ between chunks (not guaranteed by construction, but
        // statistically overwhelmingly likely for any non-trivial seed).
        assert_ne!(
            a.elevation, b.elevation,
            "adjacent chunks produced identical terrain — check seed derivation"
        );
    }

    #[test]
    fn different_seeds_produce_different_terrain() {
        let config = test_config();
        let seed_a = derive_terrain_seed(1);
        let seed_b = derive_terrain_seed(2);
        let chunk = ChunkCoord::new(0, 0);

        let a = generate_terrain_chunk(chunk, seed_a, &config);
        let b = generate_terrain_chunk(chunk, seed_b, &config);

        assert_ne!(a.elevation, b.elevation);
    }
}

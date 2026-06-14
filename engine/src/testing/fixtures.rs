//! Test fixtures and helpers for Genesis engine tests.

use crate::config::WorldConfig;
use crate::rng::WorldSeed;
use bevy_ecs::prelude::World;

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

        elevation_min: 0.0,
        elevation_max: 1.0,
        slope_max: 1.0,
        water_depth_max: 1.0,
        soil_depth_max: 1.0,
        soil_fertility_max: 1.0,
        sea_level: 0.35,

        temperature_min: 0.0,
        temperature_max: 1.0,
        moisture_min: 0.0,
        moisture_max: 1.0,
        rainfall_min: 0.0,
        rainfall_max: 1.0,
        sunlight_factor_min: 0.0,
        sunlight_factor_max: 1.0,

        fresh_water_max: 1.0,
        nutrients_max: 1.0,
        minerals_max: 1.0,
        biomass_potential_max: 1.0,
        solar_exposure_max: 1.0,
        energy_availability_max: 1.0,
        solar_elevation_coeff: 0.2,
        solar_slope_coeff: 0.3,
        energy_solar_weight: 0.4,
        energy_temp_weight: 0.3,
        energy_biomass_weight: 0.2,
        energy_nutrient_weight: 0.1,

        sea_level_temperature_base: 0.6,
        temperature_lapse_rate: 0.4,
        seasonal_temperature_amplitude: 0.15,

        initial_agent_count: 10,
        initial_agent_energy: 100.0,
        agent_energy_max: 1000.0,
        agent_age_limit: 1000,
        agent_density_cap: 1000,
        sensing_radius: 1,
        agent_base_decay_rate: 1.0,
        agent_thermal_optimum: 0.5,
        agent_movement_max_slope: 0.40,
        agent_movement_max_water_depth: 0.30,
        agent_movement_cost: 1.0,
    }
}

/// Returns a deterministic test seed.
pub fn create_test_seed() -> WorldSeed {
    WorldSeed::new(987_654_321)
}

/// Asserts that two ECS worlds are structurally and field-by-field equivalent.
///
/// Under Milestone 10 architecture guidelines, all environmental fields (f32 vectors)
/// must be validated using absolute binary equality (assert_eq!).
pub fn assert_worlds_equivalent(world_a: &mut World, world_b: &mut World) {
    use crate::config::WorldConfig;
    use crate::rng::WorldSeed;
    use crate::time::{SeasonState, SimulationClock};
    use crate::world::climate::ClimateChunk;
    use crate::world::coord::ChunkCoord;
    use crate::world::energy::EnergyAvailabilityChunk;
    use crate::world::resource::ResourceChunk;
    use crate::world::terrain::TerrainChunk;

    // 1. Compare simulation clock
    let clock_a = world_a.resource::<SimulationClock>();
    let clock_b = world_b.resource::<SimulationClock>();
    assert_eq!(
        clock_a.total_ticks, clock_b.total_ticks,
        "simulation clock ticks mismatch"
    );
    assert_eq!(
        clock_a.tick_duration_hours, clock_b.tick_duration_hours,
        "tick duration hours mismatch"
    );

    // 2. Compare SeasonState
    let season_a = world_a.resource::<SeasonState>();
    let season_b = world_b.resource::<SeasonState>();
    assert_eq!(
        season_a.season_index, season_b.season_index,
        "season index mismatch"
    );
    assert_eq!(
        season_a.tick_in_season, season_b.tick_in_season,
        "tick in season mismatch"
    );
    assert_eq!(
        season_a.progress, season_b.progress,
        "season progress mismatch"
    );
    assert_eq!(
        season_a.seasonal_modifier, season_b.seasonal_modifier,
        "seasonal modifier mismatch"
    );

    // 3. Compare WorldSeed
    let seed_a = world_a.resource::<WorldSeed>();
    let seed_b = world_b.resource::<WorldSeed>();
    assert_eq!(seed_a.root_seed, seed_b.root_seed, "world seed mismatch");

    // 4. Compare WorldConfig
    let config_a = world_a.resource::<WorldConfig>();
    let config_b = world_b.resource::<WorldConfig>();
    assert_eq!(
        config_a.world_width, config_b.world_width,
        "config world_width mismatch"
    );
    assert_eq!(
        config_a.world_height, config_b.world_height,
        "config world_height mismatch"
    );
    assert_eq!(
        config_a.chunk_size, config_b.chunk_size,
        "config chunk_size mismatch"
    );

    // 5. Query and collect all chunk entities from both worlds
    let mut query_a = world_a.query::<(
        &ChunkCoord,
        &TerrainChunk,
        &ClimateChunk,
        &ResourceChunk,
        &EnergyAvailabilityChunk,
    )>();
    let mut chunks_a: Vec<_> = query_a
        .iter(world_a)
        .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
        .collect();
    chunks_a.sort_by_key(|c| (c.0.y, c.0.x));

    let mut query_b = world_b.query::<(
        &ChunkCoord,
        &TerrainChunk,
        &ClimateChunk,
        &ResourceChunk,
        &EnergyAvailabilityChunk,
    )>();
    let mut chunks_b: Vec<_> = query_b
        .iter(world_b)
        .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
        .collect();
    chunks_b.sort_by_key(|c| (c.0.y, c.0.x));

    assert_eq!(chunks_a.len(), chunks_b.len(), "chunk count mismatch");

    for (chunk_a, chunk_b) in chunks_a.iter().zip(chunks_b.iter()) {
        assert_eq!(chunk_a.0, chunk_b.0, "chunk coord mismatch");

        // Compare TerrainChunk fields
        let terrain_a = &chunk_a.1;
        let terrain_b = &chunk_b.1;
        assert_eq!(terrain_a.elevation.len(), terrain_b.elevation.len());
        for (va, vb) in terrain_a.elevation.iter().zip(terrain_b.elevation.iter()) {
            assert_eq!(
                va, vb,
                "elevation mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
        assert_eq!(terrain_a.slope.len(), terrain_b.slope.len());
        for (va, vb) in terrain_a.slope.iter().zip(terrain_b.slope.iter()) {
            assert_eq!(
                va, vb,
                "slope mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
        assert_eq!(terrain_a.water_depth.len(), terrain_b.water_depth.len());
        for (va, vb) in terrain_a
            .water_depth
            .iter()
            .zip(terrain_b.water_depth.iter())
        {
            assert_eq!(
                va, vb,
                "water_depth mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
        assert_eq!(terrain_a.soil_depth.len(), terrain_b.soil_depth.len());
        for (va, vb) in terrain_a.soil_depth.iter().zip(terrain_b.soil_depth.iter()) {
            assert_eq!(
                va, vb,
                "soil_depth mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
        assert_eq!(
            terrain_a.soil_fertility.len(),
            terrain_b.soil_fertility.len()
        );
        for (va, vb) in terrain_a
            .soil_fertility
            .iter()
            .zip(terrain_b.soil_fertility.iter())
        {
            assert_eq!(
                va, vb,
                "soil_fertility mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }

        // Compare ClimateChunk fields
        let climate_a = &chunk_a.2;
        let climate_b = &chunk_b.2;
        assert_eq!(climate_a.temperature.len(), climate_b.temperature.len());
        for (va, vb) in climate_a
            .temperature
            .iter()
            .zip(climate_b.temperature.iter())
        {
            assert_eq!(
                va, vb,
                "temperature mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
        assert_eq!(climate_a.moisture.len(), climate_b.moisture.len());
        for (va, vb) in climate_a.moisture.iter().zip(climate_b.moisture.iter()) {
            assert_eq!(
                va, vb,
                "moisture mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
        assert_eq!(climate_a.rainfall.len(), climate_b.rainfall.len());
        for (va, vb) in climate_a.rainfall.iter().zip(climate_b.rainfall.iter()) {
            assert_eq!(
                va, vb,
                "rainfall mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
        assert_eq!(
            climate_a.sunlight_factor.len(),
            climate_b.sunlight_factor.len()
        );
        for (va, vb) in climate_a
            .sunlight_factor
            .iter()
            .zip(climate_b.sunlight_factor.iter())
        {
            assert_eq!(
                va, vb,
                "sunlight_factor mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }

        // Compare ResourceChunk fields
        let res_a = &chunk_a.3;
        let res_b = &chunk_b.3;
        assert_eq!(res_a.fresh_water.len(), res_b.fresh_water.len());
        for (va, vb) in res_a.fresh_water.iter().zip(res_b.fresh_water.iter()) {
            assert_eq!(
                va, vb,
                "fresh_water mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
        assert_eq!(res_a.nutrients.len(), res_b.nutrients.len());
        for (va, vb) in res_a.nutrients.iter().zip(res_b.nutrients.iter()) {
            assert_eq!(
                va, vb,
                "nutrients mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
        assert_eq!(res_a.minerals.len(), res_b.minerals.len());
        for (va, vb) in res_a.minerals.iter().zip(res_b.minerals.iter()) {
            assert_eq!(
                va, vb,
                "minerals mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
        assert_eq!(res_a.biomass_potential.len(), res_b.biomass_potential.len());
        for (va, vb) in res_a
            .biomass_potential
            .iter()
            .zip(res_b.biomass_potential.iter())
        {
            assert_eq!(
                va, vb,
                "biomass_potential mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }

        // Compare EnergyAvailabilityChunk fields
        let energy_a = &chunk_a.4;
        let energy_b = &chunk_b.4;
        assert_eq!(energy_a.solar_exposure.len(), energy_b.solar_exposure.len());
        for (va, vb) in energy_a
            .solar_exposure
            .iter()
            .zip(energy_b.solar_exposure.iter())
        {
            assert_eq!(
                va, vb,
                "solar_exposure mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
        assert_eq!(
            energy_a.energy_availability.len(),
            energy_b.energy_availability.len()
        );
        for (va, vb) in energy_a
            .energy_availability
            .iter()
            .zip(energy_b.energy_availability.iter())
        {
            assert_eq!(
                va, vb,
                "energy_availability mismatch at {:?}: {} vs {}",
                chunk_a.0, va, vb
            );
        }
    }
}

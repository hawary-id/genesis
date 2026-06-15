//! Integration tests for Genesis simulation engine determinism and stability.

use crate::app::schedules::{FixedSimulationTick, PersistenceBoundary, PostTickValidation};
use crate::app::App;
use crate::config::WorldConfig;
use crate::persistence::{
    build_world_snapshot, reconstruct_world_from_snapshot, SNAPSHOT_SCHEMA_VERSION,
};
use crate::rng::WorldSeed;
use crate::testing::{assert_worlds_equivalent, create_test_config, create_test_seed};
use crate::time::SimulationClock;
use crate::world::climate::ClimateChunk;
use crate::world::coord::ChunkCoord;
use crate::world::energy::EnergyAvailabilityChunk;
use crate::world::resource::ResourceChunk;
use crate::world::terrain::TerrainChunk;
use std::collections::HashMap;

#[test]
fn test_full_world_generation_determinism() {
    let config = create_test_config();
    let seed = create_test_seed();

    let mut app_a = App::new(config.clone(), seed);
    app_a.run_startup();

    let mut app_b = App::new(config.clone(), seed);
    app_b.run_startup();

    assert_worlds_equivalent(app_a.world_mut(), app_b.world_mut());
}

struct ChunkData {
    terrain: TerrainChunk,
    climate: ClimateChunk,
    resources: ResourceChunk,
    energy: EnergyAvailabilityChunk,
}

#[test]
fn test_full_world_seed_sensitivity() {
    let config = create_test_config();
    let seed_a = create_test_seed(); // 987_654_321
    let seed_b = WorldSeed::new(11223344);

    let mut app_a = App::new(config.clone(), seed_a);
    app_a.run_startup();

    let mut app_b = App::new(config.clone(), seed_b);
    app_b.run_startup();

    // Verify that their root seeds are indeed different
    assert_ne!(
        app_a.world().resource::<WorldSeed>().root_seed,
        app_b.world().resource::<WorldSeed>().root_seed
    );

    // Query chunks from both worlds
    let world_a = app_a.world_mut();
    let mut query_a = world_a.query::<(
        &ChunkCoord,
        &TerrainChunk,
        &ClimateChunk,
        &ResourceChunk,
        &EnergyAvailabilityChunk,
    )>();
    let map_a: HashMap<ChunkCoord, ChunkData> = query_a
        .iter(world_a)
        .map(|(coord, t, c, r, e)| {
            (
                *coord,
                ChunkData {
                    terrain: t.clone(),
                    climate: c.clone(),
                    resources: r.clone(),
                    energy: e.clone(),
                },
            )
        })
        .collect();

    let world_b = app_b.world_mut();
    let mut query_b = world_b.query::<(
        &ChunkCoord,
        &TerrainChunk,
        &ClimateChunk,
        &ResourceChunk,
        &EnergyAvailabilityChunk,
    )>();
    let map_b: HashMap<ChunkCoord, ChunkData> = query_b
        .iter(world_b)
        .map(|(coord, t, c, r, e)| {
            (
                *coord,
                ChunkData {
                    terrain: t.clone(),
                    climate: c.clone(),
                    resources: r.clone(),
                    energy: e.clone(),
                },
            )
        })
        .collect();

    assert_eq!(map_a.len(), map_b.len());

    // We expect differences in terrain, climate, resources, and energy fields due to different seeds
    let mut terrain_diff = false;
    let mut climate_diff = false;
    let mut resource_diff = false;
    let mut energy_diff = false;

    for (coord, a) in &map_a {
        if let Some(b) = map_b.get(coord) {
            if a.terrain.elevation != b.terrain.elevation {
                terrain_diff = true;
            }
            if a.climate.temperature != b.climate.temperature {
                climate_diff = true;
            }
            if a.resources.minerals != b.resources.minerals {
                resource_diff = true;
            }
            if a.energy.energy_availability != b.energy.energy_availability {
                energy_diff = true;
            }
        } else {
            panic!("ChunkCoord {:?} not found in World B map", coord);
        }
    }

    assert!(
        terrain_diff,
        "terrain elevations did not differ on different seeds"
    );
    assert!(
        climate_diff,
        "climate temperature did not differ on different seeds"
    );
    assert!(
        resource_diff,
        "resource minerals did not differ on different seeds"
    );
    assert!(
        energy_diff,
        "energy availability did not differ on different seeds"
    );
}

#[test]
fn test_full_world_ticking_determinism() {
    let config = create_test_config();
    let seed = create_test_seed();

    let mut app_a = App::new(config.clone(), seed);
    app_a.run_startup();

    let mut app_b = App::new(config.clone(), seed);
    app_b.run_startup();

    for _ in 0..100 {
        app_a.world_mut().run_schedule(FixedSimulationTick);
        app_a.world_mut().run_schedule(PostTickValidation);
        app_a.world_mut().run_schedule(PersistenceBoundary);

        app_b.world_mut().run_schedule(FixedSimulationTick);
        app_b.world_mut().run_schedule(PostTickValidation);
        app_b.world_mut().run_schedule(PersistenceBoundary);
    }

    assert_worlds_equivalent(app_a.world_mut(), app_b.world_mut());
}

#[test]
#[ignore]
fn test_long_run_stability_512() {
    let config = WorldConfig {
        initial_agent_count: 10,
        ..WorldConfig::default()
    };
    let seed = create_test_seed();

    // 1. Continuous Run (World A): startup + 8,640 ticks + 1 additional tick
    let mut app_continuous = App::new(config.clone(), seed);
    app_continuous.run_startup();

    for _ in 0..8640 {
        app_continuous.world_mut().run_schedule(FixedSimulationTick);
        app_continuous.world_mut().run_schedule(PostTickValidation);
        app_continuous.world_mut().run_schedule(PersistenceBoundary);
    }

    // 2. Split Run (World B): startup + 8,640 ticks -> snapshot -> load -> 1 additional tick
    let mut app_split = App::new(config.clone(), seed);
    app_split.run_startup();

    for _ in 0..8640 {
        app_split.world_mut().run_schedule(FixedSimulationTick);
        app_split.world_mut().run_schedule(PostTickValidation);
        app_split.world_mut().run_schedule(PersistenceBoundary);
    }

    // Build snapshot from the split run at tick 8,640
    let world_split = app_split.world_mut();
    let mut query = world_split.query::<(
        &ChunkCoord,
        &TerrainChunk,
        &ClimateChunk,
        &ResourceChunk,
        &EnergyAvailabilityChunk,
    )>();
    let chunks: Vec<_> = query
        .iter(world_split)
        .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
        .collect();

    let clock = world_split.resource::<SimulationClock>().clone();
    assert_eq!(clock.total_ticks, 8640);

    let id_generator = *world_split.resource::<crate::agent::StableIdGenerator>();
    let mut agent_query = world_split.query::<(
        &crate::agent::AgentMetadata,
        &crate::agent::AgentPosition,
        &crate::agent::MetabolicStock,
        &crate::agent::Genome,
        &crate::agent::LineageMetadata,
    )>();
    let agents: Vec<_> = agent_query
        .iter(world_split)
        .map(|(m, p, s, g, l)| crate::persistence::AgentSnapshot {
            metadata: *m,
            position: *p,
            stock: *s,
            genome: g.clone(),
            lineage: *l,
        })
        .collect();

    let snapshot = build_world_snapshot(
        &config,
        &seed,
        &clock,
        SNAPSHOT_SCHEMA_VERSION,
        &chunks,
        &id_generator,
        &agents,
    );

    // Serialize and deserialize snapshot to simulate save/load boundary
    let json = serde_json::to_string(&snapshot).expect("serialize failed");
    let deserialized = serde_json::from_str(&json).expect("deserialize failed");

    // Reconstruct world from deserialized snapshot in a new App
    let mut app_loaded = App::new(config.clone(), seed);
    reconstruct_world_from_snapshot(app_loaded.world_mut(), deserialized);

    // Note: validate_world_on_startup is intentionally not called here.
    // Running startup validation on a reconstructed world would incorrectly
    // report AgentCountMismatch because initial_agent_count > 0 but the
    // simulation state has evolved beyond tick 0 (agents may have died/aged).

    // Run the 8,641st tick on both apps
    // World A (Continuous) runs its 8641st tick:
    app_continuous.world_mut().run_schedule(FixedSimulationTick);
    app_continuous.world_mut().run_schedule(PostTickValidation);
    app_continuous.world_mut().run_schedule(PersistenceBoundary);

    // World B (Loaded) runs its 8641st tick:
    app_loaded.world_mut().run_schedule(FixedSimulationTick);
    app_loaded.world_mut().run_schedule(PostTickValidation);
    app_loaded.world_mut().run_schedule(PersistenceBoundary);

    // Assert absolute equivalence between continuous and loaded worlds at tick 8,641
    assert_worlds_equivalent(app_continuous.world_mut(), app_loaded.world_mut());
}

#[test]
fn test_reproduction_with_mutation_integration() {
    let mut config = create_test_config();
    config.initial_agent_count = 0;
    config.agent_density_cap = 5;
    config.mutation_rate = 1.0;
    config.mutation_step_size = 0.1;

    let seed = create_test_seed();
    let mut app = App::new(config, seed);
    app.run_startup();

    let parent_coord = crate::world::coord::WorldCoord::new(1, 1);
    let chunk_size = 32;
    app.world_mut().spawn((
        ChunkCoord::new(0, 0),
        TerrainChunk {
            elevation: vec![0.0; (chunk_size * chunk_size) as usize],
            slope: vec![0.0; (chunk_size * chunk_size) as usize],
            water_depth: vec![0.0; (chunk_size * chunk_size) as usize],
            soil_depth: vec![0.0; (chunk_size * chunk_size) as usize],
            soil_fertility: vec![0.0; (chunk_size * chunk_size) as usize],
        },
    ));

    app.world_mut().spawn((
        crate::agent::AgentMetadata::new(10),
        crate::agent::AgentPosition::new(parent_coord),
        crate::agent::MetabolicStock::new(300.0, 20),
        crate::agent::Genome::new(vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.0, 0.0, 0.5]),
        crate::agent::LineageMetadata::new(None, 0),
    ));

    app.world_mut().run_schedule(FixedSimulationTick);
    app.world_mut().run_schedule(PostTickValidation);
    app.world_mut().run_schedule(PersistenceBoundary);

    let mut query = app.world_mut().query::<(
        bevy_ecs::entity::Entity,
        &crate::agent::AgentMetadata,
        &crate::agent::Genome,
        &crate::agent::LineageMetadata,
    )>();
    let agents: Vec<_> = query.iter(app.world()).collect();

    assert_eq!(
        agents.len(),
        2,
        "Expected exactly 2 agents (parent and offspring)"
    );

    let parent_idx = if agents[0].3.parent_id.is_none() {
        0
    } else {
        1
    };
    let offspring_idx = 1 - parent_idx;

    let parent_data = &agents[parent_idx];
    let offspring_data = &agents[offspring_idx];

    assert_eq!(offspring_data.3.parent_id, Some(parent_data.1.id));
    assert_eq!(offspring_data.3.generation, 1);

    assert_ne!(
        offspring_data.2.genes, parent_data.2.genes,
        "Offspring genome did not drift"
    );

    for &gene in &offspring_data.2.genes {
        assert!(gene >= 0.0 && gene <= 1.0, "Gene out of bounds: {}", gene);
        assert!(gene.is_finite(), "Gene is not finite");
    }
}

#[test]
fn test_save_load_equivalence_with_mutation_integration() {
    let mut config = create_test_config();
    config.initial_agent_count = 5;
    config.agent_density_cap = 20;
    config.initial_agent_energy = 300.0;
    config.mutation_rate = 0.5;
    config.mutation_step_size = 0.05;

    let seed = create_test_seed();

    let mut app_continuous = App::new(config.clone(), seed);
    app_continuous.run_startup();

    for _ in 0..30 {
        app_continuous.world_mut().run_schedule(FixedSimulationTick);
        app_continuous.world_mut().run_schedule(PostTickValidation);
        app_continuous.world_mut().run_schedule(PersistenceBoundary);
    }

    let mut app_split = App::new(config.clone(), seed);
    app_split.run_startup();

    for _ in 0..30 {
        app_split.world_mut().run_schedule(FixedSimulationTick);
        app_split.world_mut().run_schedule(PostTickValidation);
        app_split.world_mut().run_schedule(PersistenceBoundary);
    }

    let world_split = app_split.world_mut();
    let mut query = world_split.query::<(
        &ChunkCoord,
        &TerrainChunk,
        &ClimateChunk,
        &ResourceChunk,
        &EnergyAvailabilityChunk,
    )>();
    let chunks: Vec<_> = query
        .iter(world_split)
        .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
        .collect();

    let clock = world_split.resource::<SimulationClock>().clone();
    assert_eq!(clock.total_ticks, 30);

    let id_generator = *world_split.resource::<crate::agent::StableIdGenerator>();
    let mut agent_query = world_split.query::<(
        &crate::agent::AgentMetadata,
        &crate::agent::AgentPosition,
        &crate::agent::MetabolicStock,
        &crate::agent::Genome,
        &crate::agent::LineageMetadata,
    )>();
    let agents: Vec<_> = agent_query
        .iter(world_split)
        .map(|(m, p, s, g, l)| crate::persistence::AgentSnapshot {
            metadata: *m,
            position: *p,
            stock: *s,
            genome: g.clone(),
            lineage: *l,
        })
        .collect();

    let snapshot = build_world_snapshot(
        &config,
        &seed,
        &clock,
        SNAPSHOT_SCHEMA_VERSION,
        &chunks,
        &id_generator,
        &agents,
    );

    let json = serde_json::to_string(&snapshot).expect("serialize failed");
    let deserialized = serde_json::from_str(&json).expect("deserialize failed");

    let mut app_loaded = App::new(config.clone(), seed);
    reconstruct_world_from_snapshot(app_loaded.world_mut(), deserialized);

    for _ in 0..20 {
        app_continuous.world_mut().run_schedule(FixedSimulationTick);
        app_continuous.world_mut().run_schedule(PostTickValidation);
        app_continuous.world_mut().run_schedule(PersistenceBoundary);

        app_loaded.world_mut().run_schedule(FixedSimulationTick);
        app_loaded.world_mut().run_schedule(PostTickValidation);
        app_loaded.world_mut().run_schedule(PersistenceBoundary);
    }

    assert_worlds_equivalent(app_continuous.world_mut(), app_loaded.world_mut());
}

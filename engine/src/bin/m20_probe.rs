use bevy_ecs::prelude::*;
use bevy_ecs::schedule::IntoSystemConfigs;
use genesis_engine::agent::components::{AgentPosition, MetabolicStock, Phenotype};
use genesis_engine::agent::systems::{process_agent_deaths, update_agent_metabolism};
use genesis_engine::config::world_config::WorldConfig;
use genesis_engine::config::WorldBounds;
use genesis_engine::world::climate::ClimateChunk;
use genesis_engine::world::coord::{ChunkCoord, WorldCoord};
use genesis_engine::world::spatial::SpatialMap;
use std::env;

#[derive(serde::Serialize)]
struct AgentRecord {
    id: u32,
    lifespan: u32,
    thermal_optimum: f32,
}

#[derive(Resource, Default)]
struct AgentRecords {
    records: Vec<AgentRecord>,
}

fn record_agent_deaths(
    mut records: ResMut<AgentRecords>,
    config: Res<WorldConfig>,
    query: Query<(Entity, &MetabolicStock, &Phenotype)>,
) {
    for (entity, stock, phenotype) in &query {
        if stock.energy <= 0.0 || stock.age > config.agent_age_limit {
            records.records.push(AgentRecord {
                id: entity.index(),
                lifespan: stock.age,
                thermal_optimum: phenotype.thermal_optimum,
            });
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let stage = if args.len() > 1 && args[1] == "A" {
        "A"
    } else {
        "B"
    };

    let (num_agents, ticks) = match stage {
        "A" => (100, 1000),
        "B" => (5000, 10000),
        _ => (5000, 10000),
    };

    let mut world = World::new();

    let config = WorldConfig {
        thermal_penalty_multiplier: 2.0,
        agent_base_decay_rate: 1.0,
        ..WorldConfig::default()
    };
    let bounds = WorldBounds::from_config(&config);
    let mut spatial_map = SpatialMap::new(bounds.chunks_x, bounds.chunks_y);

    world.insert_resource(config);
    world.insert_resource(bounds.clone());
    world.insert_resource(AgentRecords::default());

    let chunk_size = bounds.chunk_size;
    let mut temp = vec![0.5f32; (chunk_size * chunk_size) as usize];
    for y in 0..chunk_size {
        for x in 0..chunk_size {
            temp[(y * chunk_size + x) as usize] = (x as f32) / (chunk_size as f32);
        }
    }

    let chunk_entity = world
        .spawn((
            ChunkCoord::new(0, 0),
            ClimateChunk {
                temperature: temp,
                moisture: vec![0.0; (chunk_size * chunk_size) as usize],
                rainfall: vec![0.0; (chunk_size * chunk_size) as usize],
                sunlight_factor: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ))
        .id();

    spatial_map.set(ChunkCoord::new(0, 0), chunk_entity);
    world.insert_resource(spatial_map);

    for i in 0..num_agents {
        let x = i % chunk_size;
        let y = (i / chunk_size) % chunk_size;

        world.spawn((
            AgentPosition::new(WorldCoord::new(x, y)),
            MetabolicStock::new(100.0, 0),
            Phenotype {
                thermal_optimum: 0.5,
                diet_preference: 0.0,
                max_slope: 0.0,
                max_water_depth: 0.0,
                sensing_radius: 1,
                reproduction_threshold: 0.0,
                maturity_age: 0,
                physical_size: 1.0,
                derived_base_decay: 1.0,
                derived_movement_cost: 1.0,
            },
        ));
    }

    let mut schedule = Schedule::new(genesis_engine::app::schedules::FixedSimulationTick);
    schedule.add_systems(
        (
            update_agent_metabolism,
            record_agent_deaths,
            process_agent_deaths,
        )
            .chain(),
    );

    for _ in 0..ticks {
        schedule.run(&mut world);
    }

    let records = world.resource::<AgentRecords>();
    let json = serde_json::to_string_pretty(&records.records).unwrap();
    println!("{}", json);
}

//! ECS systems for agent lifecycle and spawning in Genesis.

use bevy_ecs::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::agent::components::{
    ActionIntent, ActionRequest, AgentMetadata, AgentPosition, MetabolicStock,
};
use crate::agent::resources::StableIdGenerator;
use crate::config::{WorldBounds, WorldConfig};
use crate::rng::{derive_agent_seed, WorldSeed};
use crate::world::climate::ClimateChunk;
use crate::world::coord::{ChunkCoord, WorldCoord};

/// Spawns the initial population of agents deterministically at startup.
pub fn spawn_initial_agents(
    mut commands: Commands,
    config: Res<WorldConfig>,
    bounds: Res<WorldBounds>,
    seed: Res<WorldSeed>,
    mut id_gen: ResMut<StableIdGenerator>,
) {
    let agent_seed = derive_agent_seed(seed.root_seed);
    let mut rng = ChaCha8Rng::seed_from_u64(agent_seed);

    let spawn_count = config.initial_agent_count.min(config.agent_density_cap);

    for _ in 0..spawn_count {
        let rx = rng.gen_range(0..bounds.world_width);
        let ry = rng.gen_range(0..bounds.world_height);
        let coord = WorldCoord::new(rx, ry);
        let id = id_gen.next_id();

        commands.spawn((
            AgentMetadata::new(id),
            AgentPosition::new(coord),
            MetabolicStock::new(config.initial_agent_energy, 0),
            ActionRequest::new(ActionIntent::None),
        ));
    }
}

/// Updates agent age and computes metabolic energy decay.
///
/// Environmental modifiers are required by the spec. However, the exact mathematical
/// formulation is not defined by the specification. The absolute-difference formulation:
///
/// `decay = agent_base_decay_rate + (temperature - agent_thermal_optimum).abs()`
///
/// was selected because it is deterministic, minimal, monotonic, and requires no
/// additional parameters.
pub fn update_agent_metabolism(
    config: Res<WorldConfig>,
    bounds: Res<WorldBounds>,
    climate_chunks: Query<(&ChunkCoord, &ClimateChunk)>,
    mut agents: Query<(&AgentPosition, &mut MetabolicStock)>,
) {
    let chunk_size = bounds.chunk_size;

    for (pos, mut stock) in &mut agents {
        let coord = pos.coord;

        // Verify bounds
        if !bounds.contains_world_coord(coord) {
            // Apply fallback base decay if coordinate is somehow out of bounds
            let decay = config.agent_base_decay_rate;
            stock.energy = (stock.energy - decay).max(0.0);
            stock.age = stock.age.saturating_add(1);
            continue;
        }

        // Map world coordinate to chunk and local coordinate
        let target_chunk = crate::world::coord::world_to_chunk(coord, chunk_size);
        let local_coord = crate::world::coord::world_to_local(coord, chunk_size);
        let index = (local_coord.y * chunk_size + local_coord.x) as usize;

        // Lookup temperature from ClimateChunk
        let mut local_temp = None;
        for (chunk_coord, climate_chunk) in &climate_chunks {
            if *chunk_coord == target_chunk {
                if index < climate_chunk.temperature.len() {
                    local_temp = Some(climate_chunk.temperature[index]);
                }
                break;
            }
        }

        let temp = local_temp.unwrap_or(config.agent_thermal_optimum);

        // Apply approved decay formula
        let decay = config.agent_base_decay_rate + (temp - config.agent_thermal_optimum).abs();

        stock.energy = (stock.energy - decay).max(0.0);
        stock.age = stock.age.saturating_add(1);
    }
}

/// Despawns agent entities when their energy is exhausted or age exceeds configured limits.
pub fn process_agent_deaths(
    mut commands: Commands,
    config: Res<WorldConfig>,
    query: Query<(Entity, &MetabolicStock)>,
) {
    for (entity, stock) in &query {
        if stock.energy <= 0.0 || stock.age > config.agent_age_limit {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WorldBounds;
    use crate::config::WorldConfig;
    use crate::rng::WorldSeed;
    use crate::time::{SeasonState, SimulationClock};
    use crate::world::climate::ClimateChunk;
    use crate::world::coord::ChunkCoord;

    #[test]
    fn spawn_determinism() {
        let mut world1 = World::new();
        let config1 = WorldConfig {
            initial_agent_count: 5,
            ..WorldConfig::default()
        };
        let bounds1 = WorldBounds::from_config(&config1);
        let seed1 = WorldSeed::new(42);
        let id_gen1 = StableIdGenerator::new();
        world1.insert_resource(config1);
        world1.insert_resource(bounds1);
        world1.insert_resource(seed1);
        world1.insert_resource(id_gen1);

        let mut schedule1 = Schedule::new(crate::app::StartupGeneration);
        schedule1.add_systems(spawn_initial_agents);
        world1.add_schedule(schedule1);
        world1.run_schedule(crate::app::StartupGeneration);

        // Run second world
        let mut world2 = World::new();
        let config2 = WorldConfig {
            initial_agent_count: 5,
            ..WorldConfig::default()
        };
        let bounds2 = WorldBounds::from_config(&config2);
        let seed2 = WorldSeed::new(42);
        let id_gen2 = StableIdGenerator::new();
        world2.insert_resource(config2);
        world2.insert_resource(bounds2);
        world2.insert_resource(seed2);
        world2.insert_resource(id_gen2);

        let mut schedule2 = Schedule::new(crate::app::StartupGeneration);
        schedule2.add_systems(spawn_initial_agents);
        world2.add_schedule(schedule2);
        world2.run_schedule(crate::app::StartupGeneration);

        // Fetch agents from both worlds
        let mut query1 = world1.query::<(
            &AgentMetadata,
            &AgentPosition,
            &MetabolicStock,
            &ActionRequest,
        )>();
        let mut query2 = world2.query::<(
            &AgentMetadata,
            &AgentPosition,
            &MetabolicStock,
            &ActionRequest,
        )>();

        let mut agents1: Vec<_> = query1
            .iter(&world1)
            .map(|(m, p, s, a)| (m.id, p.coord.x, p.coord.y, s.energy, s.age, a.intent))
            .collect();
        let mut agents2: Vec<_> = query2
            .iter(&world2)
            .map(|(m, p, s, a)| (m.id, p.coord.x, p.coord.y, s.energy, s.age, a.intent))
            .collect();

        // Sort by ID to ensure deterministic comparison
        agents1.sort_by_key(|a| a.0);
        agents2.sort_by_key(|a| a.0);

        assert_eq!(agents1.len(), 5);
        assert_eq!(agents1, agents2);
    }

    #[test]
    fn spawn_seed_sensitivity() {
        let mut world1 = World::new();
        let config1 = WorldConfig {
            initial_agent_count: 5,
            ..WorldConfig::default()
        };
        let bounds1 = WorldBounds::from_config(&config1);
        let seed1 = WorldSeed::new(42);
        world1.insert_resource(config1);
        world1.insert_resource(bounds1);
        world1.insert_resource(seed1);
        world1.insert_resource(StableIdGenerator::new());

        let mut schedule1 = Schedule::new(crate::app::StartupGeneration);
        schedule1.add_systems(spawn_initial_agents);
        world1.add_schedule(schedule1);
        world1.run_schedule(crate::app::StartupGeneration);

        let mut world2 = World::new();
        let config2 = WorldConfig {
            initial_agent_count: 5,
            ..WorldConfig::default()
        };
        let bounds2 = WorldBounds::from_config(&config2);
        let seed2 = WorldSeed::new(99); // different seed
        world2.insert_resource(config2);
        world2.insert_resource(bounds2);
        world2.insert_resource(seed2);
        world2.insert_resource(StableIdGenerator::new());

        let mut schedule2 = Schedule::new(crate::app::StartupGeneration);
        schedule2.add_systems(spawn_initial_agents);
        world2.add_schedule(schedule2);
        world2.run_schedule(crate::app::StartupGeneration);

        let mut query1 = world1.query::<(&AgentMetadata, &AgentPosition)>();
        let mut query2 = world2.query::<(&AgentMetadata, &AgentPosition)>();

        let mut agents1: Vec<_> = query1
            .iter(&world1)
            .map(|(m, p)| (m.id, p.coord.x, p.coord.y))
            .collect();
        let mut agents2: Vec<_> = query2
            .iter(&world2)
            .map(|(m, p)| (m.id, p.coord.x, p.coord.y))
            .collect();

        agents1.sort_by_key(|a| a.0);
        agents2.sort_by_key(|a| a.0);

        assert_eq!(agents1.len(), 5);
        assert_eq!(agents2.len(), 5);
        // They should not have the same coordinates
        assert_ne!(agents1, agents2);
    }

    #[test]
    fn spawn_cap_enforcement() {
        let mut world = World::new();
        let config = WorldConfig {
            initial_agent_count: 200,
            agent_density_cap: 15, // Cap is smaller than initial_agent_count
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        let seed = WorldSeed::new(42);
        world.insert_resource(config);
        world.insert_resource(bounds);
        world.insert_resource(seed);
        world.insert_resource(StableIdGenerator::new());

        let mut schedule = Schedule::new(crate::app::StartupGeneration);
        schedule.add_systems(spawn_initial_agents);
        world.add_schedule(schedule);
        world.run_schedule(crate::app::StartupGeneration);

        let mut query = world.query::<&AgentMetadata>();
        let count = query.iter(&world).count();
        assert_eq!(count, 15);
    }

    #[test]
    fn test_age_progression() {
        let mut world = World::new();
        let config = WorldConfig::default();
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let entity = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(update_agent_metabolism);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);
        let stock = world.entity(entity).get::<MetabolicStock>().unwrap();
        assert_eq!(stock.age, 1);

        world.run_schedule(crate::app::FixedSimulationTick);
        let stock = world.entity(entity).get::<MetabolicStock>().unwrap();
        assert_eq!(stock.age, 2);
    }

    #[test]
    fn test_energy_decay() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_base_decay_rate: 1.5,
            agent_thermal_optimum: 0.5,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let chunk_size = 32;
        let temp = vec![0.5f32; (chunk_size * chunk_size) as usize];
        world.spawn((
            ChunkCoord::new(0, 0),
            ClimateChunk {
                temperature: temp,
                moisture: vec![0.0; (chunk_size * chunk_size) as usize],
                rainfall: vec![0.0; (chunk_size * chunk_size) as usize],
                sunlight_factor: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let entity = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(update_agent_metabolism);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);
        let stock = world.entity(entity).get::<MetabolicStock>().unwrap();
        assert_eq!(stock.energy, 98.5);
    }

    #[test]
    fn test_thermal_modifier_behavior() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_base_decay_rate: 1.0,
            agent_thermal_optimum: 0.5,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let chunk_size = 32;
        let mut temp = vec![0.5f32; (chunk_size * chunk_size) as usize];
        temp[1] = 0.8;
        temp[2] = 0.2;

        world.spawn((
            ChunkCoord::new(0, 0),
            ClimateChunk {
                temperature: temp,
                moisture: vec![0.0; (chunk_size * chunk_size) as usize],
                rainfall: vec![0.0; (chunk_size * chunk_size) as usize],
                sunlight_factor: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let a1 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
            ))
            .id();

        let a2 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(1, 0)),
                MetabolicStock::new(100.0, 0),
            ))
            .id();

        let a3 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(2, 0)),
                MetabolicStock::new(100.0, 0),
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(update_agent_metabolism);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        assert_eq!(
            world.entity(a1).get::<MetabolicStock>().unwrap().energy,
            99.0
        );
        assert_eq!(
            world.entity(a2).get::<MetabolicStock>().unwrap().energy,
            98.7
        );
        assert_eq!(
            world.entity(a3).get::<MetabolicStock>().unwrap().energy,
            98.7
        );
    }

    #[test]
    fn test_starvation_death() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_base_decay_rate: 1.0,
            agent_thermal_optimum: 0.5,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let a1 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(0.5, 0),
            ))
            .id();

        let a2 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(5.0, 0),
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems((
            update_agent_metabolism,
            process_agent_deaths.after(update_agent_metabolism),
        ));
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        assert!(
            world.get_entity(a1).is_none(),
            "starved agent should be despawned"
        );
        assert!(
            world.get_entity(a2).is_some(),
            "surviving agent should not be despawned"
        );
        assert_eq!(
            world.entity(a2).get::<MetabolicStock>().unwrap().energy,
            4.0
        );
    }

    #[test]
    fn test_old_age_death() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_age_limit: 10,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let a1 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 9),
            ))
            .id();

        let a2 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 10),
            ))
            .id();

        let a3 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 11),
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_deaths);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        assert!(world.get_entity(a1).is_some(), "age 9 should survive");
        assert!(world.get_entity(a2).is_some(), "age 10 should survive");
        assert!(world.get_entity(a3).is_none(), "age 11 should be despawned");
    }

    #[test]
    fn test_validation_compatibility() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_age_limit: 10,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        let clock = SimulationClock::new();
        let season_state = SeasonState::derive(clock.total_ticks, &config);

        world.insert_resource(config);
        world.insert_resource(bounds);
        world.insert_resource(clock);
        world.insert_resource(season_state);

        let a1 = world
            .spawn((
                AgentMetadata::new(1),
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 10),
            ))
            .id();

        let mut schedules = Schedules::new();

        let mut sim_schedule = Schedule::new(crate::app::FixedSimulationTick);
        sim_schedule.add_systems((
            update_agent_metabolism,
            process_agent_deaths.after(update_agent_metabolism),
        ));
        schedules.insert(sim_schedule);

        let mut val_schedule = Schedule::new(crate::app::PostTickValidation);
        val_schedule.add_systems(crate::validation::systems::validate_world_on_tick);
        schedules.insert(val_schedule);

        world.insert_resource(schedules);

        world.run_schedule(crate::app::FixedSimulationTick);

        assert!(world.get_entity(a1).is_none());

        world.run_schedule(crate::app::PostTickValidation);
    }

    #[test]
    fn test_metabolism_determinism() {
        let mut world1 = World::new();
        let mut world2 = World::new();
        let config = WorldConfig::default();
        let bounds = WorldBounds::from_config(&config);

        world1.insert_resource(config.clone());
        world1.insert_resource(bounds.clone());
        world2.insert_resource(config);
        world2.insert_resource(bounds);

        let a1 = world1
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
            ))
            .id();

        let a2 = world2
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
            ))
            .id();

        let mut s1 = Schedule::new(crate::app::FixedSimulationTick);
        s1.add_systems(update_agent_metabolism);
        world1.add_schedule(s1);

        let mut s2 = Schedule::new(crate::app::FixedSimulationTick);
        s2.add_systems(update_agent_metabolism);
        world2.add_schedule(s2);

        world1.run_schedule(crate::app::FixedSimulationTick);
        world2.run_schedule(crate::app::FixedSimulationTick);

        let stock1 = world1.entity(a1).get::<MetabolicStock>().unwrap();
        let stock2 = world2.entity(a2).get::<MetabolicStock>().unwrap();

        assert_eq!(stock1.energy, stock2.energy);
        assert_eq!(stock1.age, stock2.age);
    }

    #[test]
    fn test_old_age_death_after_metabolism_tick() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_age_limit: 10,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let a1 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 10),
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems((
            update_agent_metabolism,
            process_agent_deaths.after(update_agent_metabolism),
        ));
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        assert!(
            world.get_entity(a1).is_none(),
            "agent at age limit should be despawned after metabolism increment"
        );
    }
}

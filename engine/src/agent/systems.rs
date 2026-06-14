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
use crate::world::coord::WorldCoord;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WorldBounds;
    use crate::config::WorldConfig;
    use crate::rng::WorldSeed;

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
}

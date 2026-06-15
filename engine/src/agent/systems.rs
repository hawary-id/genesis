//! ECS systems for agent lifecycle and spawning in Genesis.

use bevy_ecs::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::agent::components::{
    ActionIntent, ActionRequest, AgentMetadata, AgentPosition, Genome, LineageMetadata,
    MetabolicStock, Phenotype,
};
use crate::agent::resources::{GenomeConfig, StableIdGenerator};
use crate::config::{WorldBounds, WorldConfig};
use crate::rng::{derive_agent_seed, WorldSeed};
use crate::world::climate::ClimateChunk;
use crate::world::coord::{ChunkCoord, WorldCoord};
use crate::world::terrain::TerrainChunk;

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
            Genome::new(vec![0.5; 8]),
            LineageMetadata::new(None, 0),
        ));
    }
}

/// Maps a gene float in the range [0.0, 1.0] to a continuous float range.
pub fn map_gene_to_range(gene: f32, min: f32, max: f32) -> f32 {
    min + gene.clamp(0.0, 1.0) * (max - min)
}

/// Maps a gene float in the range [0.0, 1.0] to a discrete u32 range.
pub fn map_gene_to_range_discrete(gene: f32, min: u32, max: u32) -> u32 {
    let range = (max - min) as f32;
    let val = (min as f32 + gene.clamp(0.0, 1.0) * range).round() as u32;
    val.min(max).max(min)
}

/// Pure helper function to derive a Phenotype from a Genome and configurations.
pub fn derive_phenotype(
    genome: &Genome,
    gen_config: &GenomeConfig,
    world_config: &WorldConfig,
) -> Phenotype {
    let g0 = genome.genes.first().copied().unwrap_or(0.5); // thermal optimum
    let g1 = genome.genes.get(1).copied().unwrap_or(0.5); // diet preference
    let g2 = genome.genes.get(2).copied().unwrap_or(0.5); // max slope limit
    let g3 = genome.genes.get(3).copied().unwrap_or(0.5); // max water limit
    let g4 = genome.genes.get(4).copied().unwrap_or(0.5); // sensing radius
    let g5 = genome.genes.get(5).copied().unwrap_or(0.5); // reproduction threshold
    let g6 = genome.genes.get(6).copied().unwrap_or(0.5); // maturity age
    let g7 = genome.genes.get(7).copied().unwrap_or(0.5); // physical size

    let thermal_optimum = map_gene_to_range(
        g0,
        gen_config.thermal_optimum_range.0,
        gen_config.thermal_optimum_range.1,
    );
    let diet_preference = map_gene_to_range(
        g1,
        gen_config.diet_preference_range.0,
        gen_config.diet_preference_range.1,
    );
    let max_slope = map_gene_to_range(
        g2,
        gen_config.max_slope_range.0,
        gen_config.max_slope_range.1,
    );
    let max_water_depth = map_gene_to_range(
        g3,
        gen_config.max_water_depth_range.0,
        gen_config.max_water_depth_range.1,
    );
    let sensing_radius = map_gene_to_range_discrete(
        g4,
        gen_config.sensing_radius_range.0,
        gen_config.sensing_radius_range.1,
    );
    let reproduction_threshold = map_gene_to_range(
        g5,
        gen_config.reproduction_threshold_range.0,
        gen_config.reproduction_threshold_range.1,
    );
    let maturity_age = map_gene_to_range_discrete(
        g6,
        gen_config.maturity_age_range.0,
        gen_config.maturity_age_range.1,
    );
    let physical_size = map_gene_to_range(
        g7,
        gen_config.physical_size_range.0,
        gen_config.physical_size_range.1,
    );

    // Size-Sensing Metabolic Penalty (from tech spec section 3.2):
    // decay_base = agent_base_decay_rate * size * (1.0 + 0.15 * (sensing_radius - 1))
    let derived_base_decay = world_config.agent_base_decay_rate
        * physical_size
        * (1.0 + 0.15 * (sensing_radius.saturating_sub(1) as f32));

    // Kinematic Frictional Costs (from tech spec section 3.2):
    // movement_cost = agent_movement_cost * (1.0 + 0.50 * max_slope + 0.50 * max_water)
    let derived_movement_cost =
        world_config.agent_movement_cost * (1.0 + 0.50 * max_slope + 0.50 * max_water_depth);

    Phenotype::new(
        thermal_optimum,
        diet_preference,
        max_slope,
        max_water_depth,
        sensing_radius,
        reproduction_threshold,
        maturity_age,
        physical_size,
        derived_base_decay,
        derived_movement_cost,
    )
}

/// Derives a Phenotype component and attaches it when a Genome is added.
pub fn derive_phenotype_on_spawn(
    mut commands: Commands,
    gen_config: Res<GenomeConfig>,
    world_config: Res<WorldConfig>,
    query: Query<(Entity, &Genome), Added<Genome>>,
) {
    for (entity, genome) in &query {
        let phenotype = derive_phenotype(genome, &gen_config, &world_config);
        commands.entity(entity).insert(phenotype);
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

/// Processes resource consumption for biological agents, depleting nutrients and water
/// from environment chunks and replenishing agent energy stocks.
pub fn process_agent_consumption(
    config: Res<WorldConfig>,
    bounds: Res<WorldBounds>,
    mut chunks: Query<(&ChunkCoord, &mut crate::world::resource::ResourceChunk)>,
    mut agents: Query<(
        Entity,
        &AgentPosition,
        &Phenotype,
        &mut MetabolicStock,
        &AgentMetadata,
    )>,
) {
    // 1. Collect and sort agents by ID ascending to guarantee deterministic order-independence
    let mut sorted_agents: Vec<_> = agents
        .iter()
        .map(|(entity, pos, phenotype, _stock, metadata)| {
            (
                metadata.id,
                entity,
                pos.coord,
                phenotype.physical_size,
                phenotype.diet_preference,
            )
        })
        .collect();
    sorted_agents.sort_by_key(|a| a.0);

    let chunk_size = bounds.chunk_size;

    // 2. Iterate through sorted agents to perform consumption
    for (_id, entity, coord, size, diet_preference) in sorted_agents {
        // Validate coordinate bounds
        if !bounds.contains_world_coord(coord) {
            continue;
        }

        let target_chunk = crate::world::coord::world_to_chunk(coord, chunk_size);
        let local_coord = crate::world::coord::world_to_local(coord, chunk_size);
        let cell_index = (local_coord.y * chunk_size + local_coord.x) as usize;

        // Lookup containing chunk mutably
        for (chunk_coord, mut resource_chunk) in &mut chunks {
            if *chunk_coord == target_chunk {
                if cell_index < resource_chunk.nutrients.len()
                    && cell_index < resource_chunk.fresh_water.len()
                {
                    // Step 1: Sensing local cell densities
                    let cell_nutrient = resource_chunk.nutrients[cell_index];
                    let cell_water = resource_chunk.fresh_water[cell_index];

                    // Step 2: Harvesting resources capped by physical size and max_harvest_rate
                    let max_harvest = config.max_harvest_rate * size;
                    let intake_nutrient = cell_nutrient.min(max_harvest);
                    let intake_water = cell_water.min(max_harvest);

                    // Step 3: Deplete resource chunk (Conservation of mass)
                    resource_chunk.nutrients[cell_index] =
                        (cell_nutrient - intake_nutrient).max(0.0);
                    resource_chunk.fresh_water[cell_index] = (cell_water - intake_water).max(0.0);

                    // Step 4: Convert to metabolic energy
                    let energy_gain =
                        intake_nutrient * (1.0 - diet_preference) * config.consumption_efficiency
                            + intake_water * diet_preference * config.consumption_efficiency;

                    // Step 5: Replenish and clamp agent stock energy
                    if let Ok((_, _, _, mut stock, _)) = agents.get_mut(entity) {
                        stock.energy = (stock.energy + energy_gain).min(config.agent_energy_max);
                    }
                }
                break;
            }
        }
    }
}

/// Processes agent movement requests, validating destinations against boundaries and terrain barriers.
pub fn process_agent_movement(
    config: Res<WorldConfig>,
    bounds: Res<WorldBounds>,
    terrain_chunks: Query<(&ChunkCoord, &TerrainChunk)>,
    mut agents: Query<(&mut AgentPosition, &mut MetabolicStock, &mut ActionRequest)>,
) {
    let chunk_size = bounds.chunk_size;

    for (mut pos, mut stock, mut req) in &mut agents {
        let intent = req.intent;
        if intent == ActionIntent::None {
            continue;
        }

        let current_coord = pos.coord;
        let mut target_coord = current_coord;

        match intent {
            ActionIntent::MoveNorth => {
                if current_coord.y > 0 {
                    target_coord.y -= 1;
                } else {
                    req.intent = ActionIntent::None;
                    continue;
                }
            }
            ActionIntent::MoveSouth => {
                target_coord.y += 1;
            }
            ActionIntent::MoveEast => {
                target_coord.x += 1;
            }
            ActionIntent::MoveWest => {
                if current_coord.x > 0 {
                    target_coord.x -= 1;
                } else {
                    req.intent = ActionIntent::None;
                    continue;
                }
            }
            ActionIntent::None => unreachable!(),
        }

        // Validate boundary constraints
        if !bounds.contains_world_coord(target_coord) {
            req.intent = ActionIntent::None;
            continue;
        }

        // Translate cell coordinate to chunk index
        let target_chunk = crate::world::coord::world_to_chunk(target_coord, chunk_size);
        let local_coord = crate::world::coord::world_to_local(target_coord, chunk_size);
        let index = (local_coord.y * chunk_size + local_coord.x) as usize;

        // Retrieve local cell terrain
        let mut local_terrain = None;
        for (chunk_coord, chunk) in &terrain_chunks {
            if *chunk_coord == target_chunk {
                if index < chunk.slope.len() && index < chunk.water_depth.len() {
                    local_terrain = Some((chunk.slope[index], chunk.water_depth[index]));
                }
                break;
            }
        }

        let Some((slope, water_depth)) = local_terrain else {
            req.intent = ActionIntent::None;
            continue;
        };

        // Validate slope and water constraints
        if slope > config.agent_movement_max_slope
            || water_depth > config.agent_movement_max_water_depth
        {
            req.intent = ActionIntent::None;
            continue;
        }

        // Apply successful movement and deduct energy cost
        pos.coord = target_coord;
        stock.energy = (stock.energy - config.agent_movement_cost).max(0.0);
        req.intent = ActionIntent::None;
    }
}

/// Processes asexual reproduction for biological agents, validating energy/age constraints,
/// performing cardinal adjacent cell search, dividing energy, and setting up generational lineage.
pub fn process_agent_reproduction(
    mut commands: Commands,
    config: Res<WorldConfig>,
    bounds: Res<WorldBounds>,
    mut id_gen: ResMut<StableIdGenerator>,
    terrain_chunks: Query<(&ChunkCoord, &TerrainChunk)>,
    mut agents: Query<(
        Entity,
        &AgentMetadata,
        &AgentPosition,
        &mut MetabolicStock,
        &Genome,
        &LineageMetadata,
        &Phenotype,
    )>,
) {
    let initial_population = agents.iter().count();
    let mut current_population = initial_population;
    let chunk_size = bounds.chunk_size;

    // 1. Collect all eligible parents (energy >= threshold AND age >= maturity_age)
    let mut parents = Vec::new();
    for (entity, metadata, position, stock, genome, lineage, phenotype) in &agents {
        if stock.energy >= phenotype.reproduction_threshold && stock.age >= phenotype.maturity_age {
            parents.push((
                metadata.id,
                entity,
                position.coord,
                stock.energy,
                genome.clone(),
                *lineage,
                *phenotype,
            ));
        }
    }

    // 2. Sort by stable parent ID ascending to ensure determinism
    parents.sort_by_key(|p| p.0);

    // 3. Process reproduction sequentially
    for parent in parents {
        // Enforce population density cap check first
        if current_population >= config.agent_density_cap as usize {
            break;
        }

        let parent_id = parent.0;
        let parent_entity = parent.1;
        let parent_coord = parent.2;
        let parent_genome = parent.4;
        let parent_lineage = parent.5;
        let parent_pheno = parent.6;

        let max_slope = parent_pheno.max_slope;
        let max_water = parent_pheno.max_water_depth;

        // Sequence: North (y - 1), South (y + 1), East (x + 1), West (x - 1)
        let directions = [
            (Some(parent_coord.x), parent_coord.y.checked_sub(1)), // North
            (Some(parent_coord.x), Some(parent_coord.y.saturating_add(1))), // South
            (Some(parent_coord.x.saturating_add(1)), Some(parent_coord.y)), // East
            (parent_coord.x.checked_sub(1), Some(parent_coord.y)), // West
        ];

        let mut chosen_coord = None;
        for &(cx_opt, cy_opt) in &directions {
            let (Some(cx), Some(cy)) = (cx_opt, cy_opt) else {
                continue;
            };
            let candidate = WorldCoord::new(cx, cy);

            if !bounds.contains_world_coord(candidate) {
                continue;
            }

            // Translate candidate cell coordinate to chunk index
            let target_chunk = crate::world::coord::world_to_chunk(candidate, chunk_size);
            let local_coord = crate::world::coord::world_to_local(candidate, chunk_size);
            let index = (local_coord.y * chunk_size + local_coord.x) as usize;

            // Retrieve local cell terrain
            let mut local_terrain = None;
            for (chunk_coord, chunk) in &terrain_chunks {
                if *chunk_coord == target_chunk {
                    if index < chunk.slope.len() && index < chunk.water_depth.len() {
                        local_terrain = Some((chunk.slope[index], chunk.water_depth[index]));
                    }
                    break;
                }
            }

            if let Some((slope, water_depth)) = local_terrain {
                if slope <= max_slope && water_depth <= max_water {
                    chosen_coord = Some(candidate);
                    break;
                }
            }
        }

        // If a valid location was found, perform energy split, stable ID generation, and spawn offspring
        if let Some(target_coord) = chosen_coord {
            if let Ok((_, _, _, mut parent_stock, _, _, _)) = agents.get_mut(parent_entity) {
                let offspring_energy = parent_stock.energy * 0.5;
                parent_stock.energy *= 0.5;

                let offspring_id = id_gen.next_id();
                commands.spawn((
                    AgentMetadata::new(offspring_id),
                    AgentPosition::new(target_coord),
                    MetabolicStock::new(offspring_energy, 0),
                    ActionRequest::new(ActionIntent::None),
                    parent_genome, // inherited genome (no mutation)
                    LineageMetadata::new(Some(parent_id), parent_lineage.generation + 1),
                ));

                current_population += 1;
            }
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
                Genome::new(vec![0.5; 8]),
                LineageMetadata::new(None, 0),
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

    #[test]
    fn test_movement_success() {
        let mut world = World::new();
        let config = WorldConfig::default();
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let chunk_size = 32;
        let expected_len = (chunk_size * chunk_size) as usize;
        world.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; expected_len],
                slope: vec![0.0; expected_len],
                water_depth: vec![0.0; expected_len],
                soil_depth: vec![0.0; expected_len],
                soil_fertility: vec![0.0; expected_len],
            },
        ));

        let entity = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
                ActionRequest::new(ActionIntent::MoveSouth),
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_movement);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let pos = world.entity(entity).get::<AgentPosition>().unwrap();
        let stock = world.entity(entity).get::<MetabolicStock>().unwrap();
        let req = world.entity(entity).get::<ActionRequest>().unwrap();

        assert_eq!(pos.coord, WorldCoord::new(0, 1));
        assert_eq!(stock.energy, 99.0);
        assert_eq!(req.intent, ActionIntent::None);
    }

    #[test]
    fn test_movement_boundary_clamp() {
        let mut world = World::new();
        let config = WorldConfig::default();
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let entity = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
                ActionRequest::new(ActionIntent::MoveNorth),
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_movement);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let pos = world.entity(entity).get::<AgentPosition>().unwrap();
        let stock = world.entity(entity).get::<MetabolicStock>().unwrap();
        let req = world.entity(entity).get::<ActionRequest>().unwrap();

        assert_eq!(pos.coord, WorldCoord::new(0, 0));
        assert_eq!(stock.energy, 100.0);
        assert_eq!(req.intent, ActionIntent::None);
    }

    #[test]
    fn test_movement_slope_barrier() {
        let mut world = World::new();
        let config = WorldConfig::default();
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let chunk_size = 32;
        let expected_len = (chunk_size * chunk_size) as usize;
        let mut slope = vec![0.0; expected_len];
        // Target is index 1 i.e. (1, 0)
        slope[1] = 0.5;

        world.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; expected_len],
                slope,
                water_depth: vec![0.0; expected_len],
                soil_depth: vec![0.0; expected_len],
                soil_fertility: vec![0.0; expected_len],
            },
        ));

        let entity = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
                ActionRequest::new(ActionIntent::MoveEast),
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_movement);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let pos = world.entity(entity).get::<AgentPosition>().unwrap();
        let stock = world.entity(entity).get::<MetabolicStock>().unwrap();
        let req = world.entity(entity).get::<ActionRequest>().unwrap();

        assert_eq!(pos.coord, WorldCoord::new(0, 0));
        assert_eq!(stock.energy, 100.0);
        assert_eq!(req.intent, ActionIntent::None);
    }

    #[test]
    fn test_movement_water_barrier() {
        let mut world = World::new();
        let config = WorldConfig::default();
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let chunk_size = 32;
        let expected_len = (chunk_size * chunk_size) as usize;
        let mut water_depth = vec![0.0; expected_len];
        // Target is index 1 i.e. (1, 0)
        water_depth[1] = 0.35;

        world.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; expected_len],
                slope: vec![0.0; expected_len],
                water_depth,
                soil_depth: vec![0.0; expected_len],
                soil_fertility: vec![0.0; expected_len],
            },
        ));

        let entity = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
                ActionRequest::new(ActionIntent::MoveEast),
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_movement);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let pos = world.entity(entity).get::<AgentPosition>().unwrap();
        let stock = world.entity(entity).get::<MetabolicStock>().unwrap();
        let req = world.entity(entity).get::<ActionRequest>().unwrap();

        assert_eq!(pos.coord, WorldCoord::new(0, 0));
        assert_eq!(stock.energy, 100.0);
        assert_eq!(req.intent, ActionIntent::None);
    }

    #[test]
    fn test_movement_action_clearing() {
        let mut world = World::new();
        let config = WorldConfig::default();
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let chunk_size = 32;
        let expected_len = (chunk_size * chunk_size) as usize;
        world.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; expected_len],
                slope: vec![0.0; expected_len],
                water_depth: vec![0.0; expected_len],
                soil_depth: vec![0.0; expected_len],
                soil_fertility: vec![0.0; expected_len],
            },
        ));

        // Successful path
        let entity1 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
                ActionRequest::new(ActionIntent::MoveSouth),
            ))
            .id();

        // Blocked path (boundary clamp)
        let entity2 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
                ActionRequest::new(ActionIntent::MoveNorth),
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_movement);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        assert_eq!(
            world.entity(entity1).get::<ActionRequest>().unwrap().intent,
            ActionIntent::None
        );
        assert_eq!(
            world.entity(entity2).get::<ActionRequest>().unwrap().intent,
            ActionIntent::None
        );
    }

    #[test]
    fn test_genome_mapping() {
        let gen_config = GenomeConfig::default();
        let world_config = WorldConfig::default();

        // gene = 0.0
        let g_min = Genome::new(vec![0.0; 8]);
        let pheno_min = derive_phenotype(&g_min, &gen_config, &world_config);
        assert_eq!(pheno_min.thermal_optimum, 0.0);
        assert_eq!(pheno_min.diet_preference, 0.0);
        assert_eq!(pheno_min.max_slope, 0.10);
        assert_eq!(pheno_min.max_water_depth, 0.10);
        assert_eq!(pheno_min.sensing_radius, 1);
        assert_eq!(pheno_min.reproduction_threshold, 150.0);
        assert_eq!(pheno_min.maturity_age, 20);
        assert_eq!(pheno_min.physical_size, 0.5);

        // gene = 0.5
        let g_mid = Genome::new(vec![0.5; 8]);
        let pheno_mid = derive_phenotype(&g_mid, &gen_config, &world_config);
        assert_eq!(pheno_mid.thermal_optimum, 0.5);
        assert_eq!(pheno_mid.diet_preference, 0.5);
        assert_eq!(pheno_mid.max_slope, 0.35);
        assert_eq!(pheno_mid.max_water_depth, 0.30);
        assert_eq!(pheno_mid.sensing_radius, 3); // (1 + 0.5 * (4 - 1)) = 2.5, round() is 3
        assert_eq!(pheno_mid.reproduction_threshold, 325.0);
        assert_eq!(pheno_mid.maturity_age, 110); // (20 + 0.5 * 180) = 110
        assert_eq!(pheno_mid.physical_size, 1.25);

        // gene = 1.0
        let g_max = Genome::new(vec![1.0; 8]);
        let pheno_max = derive_phenotype(&g_max, &gen_config, &world_config);
        assert_eq!(pheno_max.thermal_optimum, 1.0);
        assert_eq!(pheno_max.diet_preference, 1.0);
        assert_eq!(pheno_max.max_slope, 0.60);
        assert_eq!(pheno_max.max_water_depth, 0.50);
        assert_eq!(pheno_max.sensing_radius, 4);
        assert_eq!(pheno_max.reproduction_threshold, 500.0);
        assert_eq!(pheno_max.maturity_age, 200);
        assert_eq!(pheno_max.physical_size, 2.0);
    }

    #[test]
    fn test_serialization_and_lineage_preservation() {
        use crate::persistence::{
            build_world_snapshot, reconstruct_world_from_snapshot, AgentSnapshot, WorldSnapshot,
            SNAPSHOT_SCHEMA_VERSION,
        };
        use crate::rng::WorldSeed;
        use crate::time::SimulationClock;

        let config = WorldConfig::default();
        let seed = WorldSeed::new(12345);
        let clock = SimulationClock::default();
        let id_generator = crate::agent::StableIdGenerator::new();

        let agents = vec![
            AgentSnapshot {
                metadata: AgentMetadata::new(1),
                position: AgentPosition::new(WorldCoord::new(2, 3)),
                stock: MetabolicStock::new(100.0, 5),
                genome: Genome::new(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]),
                lineage: LineageMetadata::new(None, 0),
            },
            AgentSnapshot {
                metadata: AgentMetadata::new(2),
                position: AgentPosition::new(WorldCoord::new(5, 7)),
                stock: MetabolicStock::new(200.0, 10),
                genome: Genome::new(vec![0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1]),
                lineage: LineageMetadata::new(Some(1), 1),
            },
        ];

        let snapshot1 = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &[],
            &id_generator,
            &agents,
        );

        let json1 = serde_json::to_string(&snapshot1).expect("serialization failed");
        let deserialized1: WorldSnapshot =
            serde_json::from_str(&json1).expect("deserialization failed");

        let mut world2 = World::new();
        reconstruct_world_from_snapshot(&mut world2, deserialized1);

        let mut agent_query = world2.query::<(
            &AgentMetadata,
            &AgentPosition,
            &MetabolicStock,
            &Genome,
            &LineageMetadata,
        )>();

        let mut agents2: Vec<_> = agent_query
            .iter(&world2)
            .map(|(m, p, s, g, l)| AgentSnapshot {
                metadata: *m,
                position: *p,
                stock: *s,
                genome: g.clone(),
                lineage: *l,
            })
            .collect();
        agents2.sort_by_key(|a| a.metadata.id);

        assert_eq!(agents2.len(), 2);
        assert_eq!(agents2[0].metadata.id, 1);
        assert_eq!(agents2[0].position.coord, WorldCoord::new(2, 3));
        assert_eq!(agents2[0].stock.energy, 100.0);
        assert_eq!(agents2[0].stock.age, 5);
        assert_eq!(
            agents2[0].genome.genes,
            vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]
        );
        assert_eq!(agents2[0].lineage.parent_id, None);
        assert_eq!(agents2[0].lineage.generation, 0);

        assert_eq!(agents2[1].metadata.id, 2);
        assert_eq!(agents2[1].position.coord, WorldCoord::new(5, 7));
        assert_eq!(agents2[1].stock.energy, 200.0);
        assert_eq!(agents2[1].stock.age, 10);
        assert_eq!(
            agents2[1].genome.genes,
            vec![0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1]
        );
        assert_eq!(agents2[1].lineage.parent_id, Some(1));
        assert_eq!(agents2[1].lineage.generation, 1);

        let snapshot2 = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &[],
            &id_generator,
            &agents2,
        );
        let json2 = serde_json::to_string(&snapshot2).expect("serialization failed");
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_phenotype_reconstruction() {
        use crate::persistence::{
            build_world_snapshot, reconstruct_world_from_snapshot, AgentSnapshot, WorldSnapshot,
            SNAPSHOT_SCHEMA_VERSION,
        };
        use crate::rng::WorldSeed;
        use crate::time::SimulationClock;

        let config = WorldConfig::default();
        let seed = WorldSeed::new(12345);
        let clock = SimulationClock::default();
        let id_generator = crate::agent::StableIdGenerator::new();
        let gen_config = GenomeConfig::default();

        let parent_genome = Genome::new(vec![0.2, 0.4, 0.6, 0.8, 0.1, 0.3, 0.5, 0.7]);
        let parent_phenotype = derive_phenotype(&parent_genome, &gen_config, &config);

        let agents = vec![AgentSnapshot {
            metadata: AgentMetadata::new(1),
            position: AgentPosition::new(WorldCoord::new(1, 1)),
            stock: MetabolicStock::new(100.0, 0),
            genome: parent_genome.clone(),
            lineage: LineageMetadata::new(None, 0),
        }];

        let snapshot = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &[],
            &id_generator,
            &agents,
        );

        let json = serde_json::to_string(&snapshot).expect("serialization failed");
        let deserialized: WorldSnapshot =
            serde_json::from_str(&json).expect("deserialization failed");

        let mut world2 = World::new();
        // Since reconstruct_world_from_snapshot internally runs derive_phenotype and inserts it,
        // we can query the phenotype component after reconstruction.
        reconstruct_world_from_snapshot(&mut world2, deserialized);

        let mut pheno_query = world2.query::<&Phenotype>();
        let reconstructed_phenotype = pheno_query.iter(&world2).next().unwrap();

        assert_eq!(
            reconstructed_phenotype.thermal_optimum,
            parent_phenotype.thermal_optimum
        );
        assert_eq!(
            reconstructed_phenotype.diet_preference,
            parent_phenotype.diet_preference
        );
        assert_eq!(
            reconstructed_phenotype.max_slope,
            parent_phenotype.max_slope
        );
        assert_eq!(
            reconstructed_phenotype.max_water_depth,
            parent_phenotype.max_water_depth
        );
        assert_eq!(
            reconstructed_phenotype.sensing_radius,
            parent_phenotype.sensing_radius
        );
        assert_eq!(
            reconstructed_phenotype.reproduction_threshold,
            parent_phenotype.reproduction_threshold
        );
        assert_eq!(
            reconstructed_phenotype.maturity_age,
            parent_phenotype.maturity_age
        );
        assert_eq!(
            reconstructed_phenotype.physical_size,
            parent_phenotype.physical_size
        );
        assert_eq!(
            reconstructed_phenotype.derived_base_decay,
            parent_phenotype.derived_base_decay
        );
        assert_eq!(
            reconstructed_phenotype.derived_movement_cost,
            parent_phenotype.derived_movement_cost
        );
    }

    #[test]
    fn test_resource_consumption_semantics() {
        use crate::world::resource::ResourceChunk;

        let mut world = World::new();
        let config = WorldConfig {
            max_harvest_rate: 10.0,
            consumption_efficiency: 0.8,
            agent_energy_max: 100.0,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let chunk_size = 32;
        let mut nutrients = vec![0.0; (chunk_size * chunk_size) as usize];
        let mut fresh_water = vec![0.0; (chunk_size * chunk_size) as usize];
        nutrients[0] = 20.0;
        fresh_water[0] = 20.0;

        world.spawn((
            ChunkCoord::new(0, 0),
            ResourceChunk {
                fresh_water,
                nutrients,
                minerals: vec![0.0; (chunk_size * chunk_size) as usize],
                biomass_potential: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let pheno_nutrient = Phenotype {
            diet_preference: 0.0,
            physical_size: 1.0,
            ..Phenotype::new(0.5, 0.0, 0.4, 0.3, 1, 200.0, 50, 1.0, 1.0, 1.0)
        };
        let a1 = world
            .spawn((
                AgentMetadata::new(1),
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(50.0, 0),
                pheno_nutrient,
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_consumption);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let mut chunk_query = world.query::<&ResourceChunk>();
        let rc = chunk_query.iter(&world).next().unwrap();
        assert_eq!(rc.nutrients[0], 10.0);
        assert_eq!(rc.fresh_water[0], 10.0);

        let stock_a1 = world.entity(a1).get::<MetabolicStock>().unwrap();
        assert_eq!(stock_a1.energy, 58.0);
    }

    #[test]
    fn test_water_consumption_and_clamping() {
        use crate::world::resource::ResourceChunk;

        let mut world = World::new();
        let config = WorldConfig {
            max_harvest_rate: 10.0,
            consumption_efficiency: 0.8,
            agent_energy_max: 100.0,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let chunk_size = 32;
        let mut nutrients = vec![0.0; (chunk_size * chunk_size) as usize];
        let mut fresh_water = vec![0.0; (chunk_size * chunk_size) as usize];
        nutrients[0] = 5.0;
        fresh_water[0] = 50.0;

        world.spawn((
            ChunkCoord::new(0, 0),
            ResourceChunk {
                fresh_water,
                nutrients,
                minerals: vec![0.0; (chunk_size * chunk_size) as usize],
                biomass_potential: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let pheno_water = Phenotype {
            diet_preference: 1.0,
            physical_size: 1.0,
            ..Phenotype::new(0.5, 1.0, 0.4, 0.3, 1, 200.0, 50, 1.0, 1.0, 1.0)
        };
        let a1 = world
            .spawn((
                AgentMetadata::new(1),
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(95.0, 0),
                pheno_water,
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_consumption);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let mut chunk_query = world.query::<&ResourceChunk>();
        let rc = chunk_query.iter(&world).next().unwrap();
        assert_eq!(rc.nutrients[0], 0.0);
        assert_eq!(rc.fresh_water[0], 40.0);

        let stock_a1 = world.entity(a1).get::<MetabolicStock>().unwrap();
        assert_eq!(stock_a1.energy, 100.0);
    }

    #[test]
    fn test_diet_preference_scaling_omnivore() {
        use crate::world::resource::ResourceChunk;

        let mut world = World::new();
        let config = WorldConfig {
            max_harvest_rate: 10.0,
            consumption_efficiency: 0.8,
            agent_energy_max: 200.0,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let chunk_size = 32;
        let mut nutrients = vec![0.0; (chunk_size * chunk_size) as usize];
        let mut fresh_water = vec![0.0; (chunk_size * chunk_size) as usize];
        nutrients[0] = 20.0;
        fresh_water[0] = 20.0;

        world.spawn((
            ChunkCoord::new(0, 0),
            ResourceChunk {
                fresh_water,
                nutrients,
                minerals: vec![0.0; (chunk_size * chunk_size) as usize],
                biomass_potential: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let pheno_omni = Phenotype {
            diet_preference: 0.5,
            physical_size: 1.0,
            ..Phenotype::new(0.5, 0.5, 0.4, 0.3, 1, 200.0, 50, 1.0, 1.0, 1.0)
        };
        let a1 = world
            .spawn((
                AgentMetadata::new(1),
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(10.0, 0),
                pheno_omni,
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_consumption);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let stock_a1 = world.entity(a1).get::<MetabolicStock>().unwrap();
        assert_eq!(stock_a1.energy, 18.0);
    }

    #[test]
    fn test_consumption_determinism_same_cell() {
        use crate::world::resource::ResourceChunk;

        let mut world = World::new();
        let config = WorldConfig {
            max_harvest_rate: 10.0,
            consumption_efficiency: 0.8,
            agent_energy_max: 200.0,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds);

        let chunk_size = 32;
        let mut nutrients = vec![0.0; (chunk_size * chunk_size) as usize];
        nutrients[0] = 10.0;

        world.spawn((
            ChunkCoord::new(0, 0),
            ResourceChunk {
                fresh_water: vec![0.0; (chunk_size * chunk_size) as usize],
                nutrients,
                minerals: vec![0.0; (chunk_size * chunk_size) as usize],
                biomass_potential: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let pheno = Phenotype {
            diet_preference: 0.0,
            physical_size: 1.0,
            ..Phenotype::new(0.5, 0.0, 0.4, 0.3, 1, 200.0, 50, 1.0, 1.0, 1.0)
        };

        let a2 = world
            .spawn((
                AgentMetadata::new(2),
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(10.0, 0),
                pheno,
            ))
            .id();

        let a1 = world
            .spawn((
                AgentMetadata::new(1),
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(10.0, 0),
                pheno,
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_consumption);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let stock_a1 = world.entity(a1).get::<MetabolicStock>().unwrap();
        let stock_a2 = world.entity(a2).get::<MetabolicStock>().unwrap();

        assert_eq!(stock_a1.energy, 18.0);
        assert_eq!(stock_a2.energy, 10.0);
    }

    #[test]
    fn test_reproduction_energy_split() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_density_cap: 10,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config.clone());
        world.insert_resource(bounds);

        let mut id_gen = StableIdGenerator::new();
        for _ in 0..10 {
            id_gen.next_id();
        }
        world.insert_resource(id_gen);

        let chunk_size = 32;
        world.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; (chunk_size * chunk_size) as usize],
                slope: vec![0.0; (chunk_size * chunk_size) as usize],
                water_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_fertility: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let pheno = Phenotype::new(0.5, 0.5, 0.4, 0.3, 1, 200.0, 50, 1.0, 1.0, 1.0);
        let parent = world
            .spawn((
                AgentMetadata::new(10),
                AgentPosition::new(WorldCoord::new(1, 1)),
                MetabolicStock::new(300.0, 50),
                Genome::new(vec![0.5; 8]),
                LineageMetadata::new(None, 0),
                pheno,
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_reproduction);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let parent_stock = world.entity(parent).get::<MetabolicStock>().unwrap();
        assert_eq!(parent_stock.energy, 150.0);

        let mut query = world.query::<(&AgentMetadata, &MetabolicStock)>();
        let mut agents: Vec<_> = query.iter(&world).collect();
        agents.sort_by_key(|a| a.0.id);

        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].0.id, 10); // parent
        assert_eq!(agents[1].0.id, 11); // offspring ID allocated sequentially starting at 11
        assert_eq!(agents[1].1.energy, 150.0);
    }

    #[test]
    fn test_reproduction_lineage_assignment() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_density_cap: 10,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config.clone());
        world.insert_resource(bounds);

        let mut id_gen = StableIdGenerator::new();
        for _ in 0..42 {
            id_gen.next_id();
        }
        world.insert_resource(id_gen);

        let chunk_size = 32;
        world.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; (chunk_size * chunk_size) as usize],
                slope: vec![0.0; (chunk_size * chunk_size) as usize],
                water_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_fertility: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let pheno = Phenotype::new(0.5, 0.5, 0.4, 0.3, 1, 200.0, 50, 1.0, 1.0, 1.0);
        world.spawn((
            AgentMetadata::new(42),
            AgentPosition::new(WorldCoord::new(1, 1)),
            MetabolicStock::new(300.0, 50),
            Genome::new(vec![0.5; 8]),
            LineageMetadata::new(Some(10), 2),
            pheno,
        ));

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_reproduction);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let mut query = world.query::<(&AgentMetadata, &LineageMetadata)>();
        let mut agents: Vec<_> = query.iter(&world).collect();
        agents.sort_by_key(|a| a.0.id);

        assert_eq!(agents.len(), 2);
        // Parent ID is 42
        assert_eq!(agents[0].0.id, 42);
        assert_eq!(agents[0].1.parent_id, Some(10));
        assert_eq!(agents[0].1.generation, 2);
        // Offspring ID is 43
        assert_eq!(agents[1].0.id, 43);
        assert_eq!(agents[1].1.parent_id, Some(42));
        assert_eq!(agents[1].1.generation, 3);
    }

    #[test]
    fn test_reproduction_cardinal_precedence() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_density_cap: 10,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config.clone());
        world.insert_resource(bounds);

        let mut id_gen = StableIdGenerator::new();
        for _ in 0..10 {
            id_gen.next_id();
        }
        world.insert_resource(id_gen);

        let chunk_size = 32;
        let mut slope = vec![0.0; (chunk_size * chunk_size) as usize];
        // Block North cell (1, 0)
        slope[1] = 0.5;

        world.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; (chunk_size * chunk_size) as usize],
                slope,
                water_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_fertility: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let pheno = Phenotype::new(0.5, 0.5, 0.4, 0.3, 1, 200.0, 50, 1.0, 1.0, 1.0);
        world.spawn((
            AgentMetadata::new(10),
            AgentPosition::new(WorldCoord::new(1, 1)),
            MetabolicStock::new(300.0, 50),
            Genome::new(vec![0.5; 8]),
            LineageMetadata::new(None, 0),
            pheno,
        ));

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_reproduction);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let mut query = world.query::<(&AgentMetadata, &AgentPosition)>();
        let mut agents: Vec<_> = query.iter(&world).collect();
        agents.sort_by_key(|a| a.0.id);

        assert_eq!(agents.len(), 2);
        // Parent ID is 10
        assert_eq!(agents[0].0.id, 10);
        // Offspring ID is 11. North is blocked. Precedence: North (fail) -> South (success). South is cell (1, 2)
        assert_eq!(agents[1].0.id, 11);
        assert_eq!(agents[1].1.coord, WorldCoord::new(1, 2));
    }

    #[test]
    fn test_reproduction_cancellation() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_density_cap: 10,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config.clone());
        world.insert_resource(bounds);

        let mut id_gen = StableIdGenerator::new();
        for _ in 0..10 {
            id_gen.next_id();
        }
        world.insert_resource(id_gen);

        let chunk_size = 32;
        let mut slope = vec![0.0; (chunk_size * chunk_size) as usize];
        let mut water_depth = vec![0.0; (chunk_size * chunk_size) as usize];
        // Block South (0, 1) with slope
        slope[chunk_size as usize] = 0.5;
        // Block East (1, 0) with water depth
        water_depth[1] = 0.4;

        world.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; (chunk_size * chunk_size) as usize],
                slope,
                water_depth,
                soil_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_fertility: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let pheno = Phenotype::new(0.5, 0.5, 0.4, 0.3, 1, 200.0, 50, 1.0, 1.0, 1.0);
        let parent = world
            .spawn((
                AgentMetadata::new(10),
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(300.0, 50),
                Genome::new(vec![0.5; 8]),
                LineageMetadata::new(None, 0),
                pheno,
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_reproduction);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        // Reproduction cancelled: agent count remains 1, parent energy remains 300.0
        let count = world.query::<&AgentMetadata>().iter(&world).count();
        assert_eq!(count, 1);

        let parent_stock = world.entity(parent).get::<MetabolicStock>().unwrap();
        assert_eq!(parent_stock.energy, 300.0);
    }

    #[test]
    fn test_reproduction_density_cap() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_density_cap: 3, // very low cap
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config.clone());
        world.insert_resource(bounds);

        let mut id_gen = StableIdGenerator::new();
        for _ in 0..20 {
            id_gen.next_id();
        }
        world.insert_resource(id_gen);

        let chunk_size = 32;
        world.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; (chunk_size * chunk_size) as usize],
                slope: vec![0.0; (chunk_size * chunk_size) as usize],
                water_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_fertility: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let pheno = Phenotype::new(0.5, 0.5, 0.4, 0.3, 1, 200.0, 50, 1.0, 1.0, 1.0);
        // Spawn 2 agents (both eligible to reproduce)
        world.spawn((
            AgentMetadata::new(10),
            AgentPosition::new(WorldCoord::new(1, 1)),
            MetabolicStock::new(300.0, 50),
            Genome::new(vec![0.5; 8]),
            LineageMetadata::new(None, 0),
            pheno,
        ));
        world.spawn((
            AgentMetadata::new(20),
            AgentPosition::new(WorldCoord::new(5, 5)),
            MetabolicStock::new(300.0, 50),
            Genome::new(vec![0.5; 8]),
            LineageMetadata::new(None, 0),
            pheno,
        ));

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_reproduction);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        // Initial pop is 2. Cap is 3. Only 1 offspring should be spawned.
        let count = world.query::<&AgentMetadata>().iter(&world).count();
        assert_eq!(count, 3);

        let mut query = world.query::<&AgentMetadata>();
        let mut ids: Vec<_> = query.iter(&world).map(|m| m.id).collect();
        ids.sort();
        assert_eq!(ids, vec![10, 20, 21]); // ID 21 allocated sequentially
    }

    #[test]
    fn test_reproduction_determinism_sorting() {
        let mut world1 = World::new();
        let config = WorldConfig {
            agent_density_cap: 10,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world1.insert_resource(config.clone());
        world1.insert_resource(bounds.clone());

        let mut id_gen1 = StableIdGenerator::new();
        for _ in 0..20 {
            id_gen1.next_id();
        }
        world1.insert_resource(id_gen1);

        let chunk_size = 32;
        world1.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; (chunk_size * chunk_size) as usize],
                slope: vec![0.0; (chunk_size * chunk_size) as usize],
                water_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_fertility: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        let pheno = Phenotype::new(0.5, 0.5, 0.4, 0.3, 1, 200.0, 50, 1.0, 1.0, 1.0);
        // Spawn parents with specific ID orders
        world1.spawn((
            AgentMetadata::new(10),
            AgentPosition::new(WorldCoord::new(1, 1)),
            MetabolicStock::new(300.0, 50),
            Genome::new(vec![0.5; 8]),
            LineageMetadata::new(None, 0),
            pheno,
        ));
        world1.spawn((
            AgentMetadata::new(20),
            AgentPosition::new(WorldCoord::new(1, 1)), // same cell
            MetabolicStock::new(300.0, 50),
            Genome::new(vec![0.5; 8]),
            LineageMetadata::new(None, 0),
            pheno,
        ));

        let mut schedule1 = Schedule::new(crate::app::FixedSimulationTick);
        schedule1.add_systems(process_agent_reproduction);
        world1.add_schedule(schedule1);
        world1.run_schedule(crate::app::FixedSimulationTick);

        // World 2: Spawn in reverse order
        let mut world2 = World::new();
        world2.insert_resource(config);
        world2.insert_resource(bounds);

        let mut id_gen2 = StableIdGenerator::new();
        for _ in 0..20 {
            id_gen2.next_id();
        }
        world2.insert_resource(id_gen2);

        world2.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; (chunk_size * chunk_size) as usize],
                slope: vec![0.0; (chunk_size * chunk_size) as usize],
                water_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_depth: vec![0.0; (chunk_size * chunk_size) as usize],
                soil_fertility: vec![0.0; (chunk_size * chunk_size) as usize],
            },
        ));

        world2.spawn((
            AgentMetadata::new(20),
            AgentPosition::new(WorldCoord::new(1, 1)),
            MetabolicStock::new(300.0, 50),
            Genome::new(vec![0.5; 8]),
            LineageMetadata::new(None, 0),
            pheno,
        ));
        world2.spawn((
            AgentMetadata::new(10),
            AgentPosition::new(WorldCoord::new(1, 1)),
            MetabolicStock::new(300.0, 50),
            Genome::new(vec![0.5; 8]),
            LineageMetadata::new(None, 0),
            pheno,
        ));

        let mut schedule2 = Schedule::new(crate::app::FixedSimulationTick);
        schedule2.add_systems(process_agent_reproduction);
        world2.add_schedule(schedule2);
        world2.run_schedule(crate::app::FixedSimulationTick);

        let mut query1 = world1.query::<(&AgentMetadata, &AgentPosition)>();
        let mut agents1: Vec<_> = query1
            .iter(&world1)
            .map(|(m, p)| (m.id, p.coord.x, p.coord.y))
            .collect();
        agents1.sort_by_key(|a| a.0);

        let mut query2 = world2.query::<(&AgentMetadata, &AgentPosition)>();
        let mut agents2: Vec<_> = query2
            .iter(&world2)
            .map(|(m, p)| (m.id, p.coord.x, p.coord.y))
            .collect();
        agents2.sort_by_key(|a| a.0);

        assert_eq!(agents1, agents2);
    }
}

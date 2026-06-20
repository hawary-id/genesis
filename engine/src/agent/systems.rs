//! ECS systems for agent lifecycle and spawning in Genesis.

use bevy_ecs::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::agent::components::LocationCategory;
use crate::agent::components::{
    ActionIntent, ActionRequest, AgentMetadata, AgentPosition, Genome, LineageMetadata,
    LocationMemory, MetabolicStock, Phenotype,
};
use crate::agent::events::ObservationEvent;
use crate::agent::resources::{GenomeConfig, StableIdGenerator};
use crate::config::{WorldBounds, WorldConfig};
use crate::rng::{derive_agent_seed, WorldSeed};
use crate::time::SimulationClock;
use crate::world::climate::ClimateChunk;
use crate::world::coord::{ChunkCoord, WorldCoord};
use crate::world::terrain::TerrainChunk;
use bevy_ecs::event::EventWriter;

/// Deterministic 64-bit integer mixer (SplitMix64 finalizer)
pub fn deterministic_mix_64(val: u64) -> u64 {
    let mut x = val;
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^= x >> 31;
    x
}

/// Derives a stable, platform-independent mutation seed for ChaCha8Rng.
pub fn derive_mutation_seed(parent_id: u64, tick: u32, coord: WorldCoord, root_seed: u64) -> u64 {
    let mut mix = root_seed;
    mix = deterministic_mix_64(mix.wrapping_add(parent_id));
    mix = deterministic_mix_64(mix.wrapping_add(tick as u64));
    mix = deterministic_mix_64(mix.wrapping_add(coord.x as u64));
    mix = deterministic_mix_64(mix.wrapping_add(coord.y as u64));
    mix
}

/// Pure helper function to apply deterministic Gaussian mutation to a genome.
pub fn mutate_genome(
    parent_genome: &Genome,
    mutation_rate: f32,
    mutation_step_size: f32,
    seed: u64,
) -> Genome {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut mutated_genes = parent_genome.genes.clone();

    for gene in &mut mutated_genes {
        let r: f32 = rng.gen(); // Uniformly distributed in [0.0, 1.0)
        if r < mutation_rate {
            // Apply Gaussian displacement using Box-Muller transform
            let mut u1: f32 = rng.gen();
            while u1 == 0.0 {
                u1 = rng.gen();
            }
            let u2: f32 = rng.gen();

            let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
            let d = z0 * mutation_step_size;
            *gene = (*gene + d).clamp(0.0, 1.0);
        }
    }

    Genome {
        genes: mutated_genes,
    }
}

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
            Genome::new(vec![0.5; crate::agent::GENOME_SIZE]),
            LineageMetadata::new(None, 0),
            LocationMemory::new(),
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
    spatial_map: Res<crate::world::spatial::SpatialMap>,
    climate_chunks: Query<&ClimateChunk>,
    mut agents: Query<(&AgentPosition, &Phenotype, &mut MetabolicStock)>,
) {
    let chunk_size = bounds.chunk_size;

    for (pos, phenotype, mut stock) in &mut agents {
        let coord = pos.coord;

        // Verify bounds
        if !bounds.contains_world_coord(coord) {
            // Apply fallback base decay if coordinate is somehow out of bounds
            let decay = phenotype.derived_base_decay;
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
        if let Some(chunk_entity) = spatial_map.get(target_chunk) {
            if let Ok(climate_chunk) = climate_chunks.get(chunk_entity) {
                if index < climate_chunk.temperature.len() {
                    local_temp = Some(climate_chunk.temperature[index]);
                }
            }
        }

        let temp = local_temp.unwrap_or(phenotype.thermal_optimum);

        // Apply approved decay formula (M20 Linear Penalty)
        let delta_t = (temp - phenotype.thermal_optimum).abs();
        let penalty = config.thermal_penalty_multiplier * delta_t;
        let decay = phenotype.derived_base_decay + penalty;

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
#[allow(clippy::too_many_arguments)]
pub fn process_agent_consumption(
    config: Res<WorldConfig>,
    bounds: Res<WorldBounds>,
    clock: Res<crate::time::SimulationClock>,
    mut chunks: Query<(&ChunkCoord, &mut crate::world::resource::ResourceChunk)>,
    mut events: EventWriter<ObservationEvent>,
    mut mem_events: EventWriter<crate::agent::events::EventMemoryEvent>,
    mut seq_counter: ResMut<crate::agent::resources::EventSequenceCounter>,
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

                    let mut consumed = false;
                    if intake_nutrient > 0.0 {
                        events.send(ObservationEvent::new(
                            _id,
                            coord,
                            LocationCategory::Nutrient,
                        ));
                        consumed = true;
                    }
                    if intake_water > 0.0 {
                        events.send(ObservationEvent::new(
                            _id,
                            coord,
                            LocationCategory::FreshWater,
                        ));
                        consumed = true;
                    }

                    if consumed {
                        mem_events.send(crate::agent::events::EventMemoryEvent::new(
                            _id,
                            crate::agent::components::EventCategory::ResourceConsumed,
                            clock.total_ticks,
                            seq_counter.next_sequence(),
                        ));
                    }

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
    bounds: Res<WorldBounds>,
    clock: Res<crate::time::SimulationClock>,
    terrain_chunks: Query<(&ChunkCoord, &TerrainChunk)>,
    mut events: EventWriter<ObservationEvent>,
    mut mem_events: EventWriter<crate::agent::events::EventMemoryEvent>,
    mut seq_counter: ResMut<crate::agent::resources::EventSequenceCounter>,
    mut agents: Query<(
        Entity,
        &AgentMetadata,
        &mut AgentPosition,
        &mut MetabolicStock,
        &mut ActionRequest,
        &Phenotype,
    )>,
) {
    let chunk_size = bounds.chunk_size;

    let to_move: Vec<_> = agents
        .iter()
        .filter(|(_, _, _, _, req, _)| req.intent != ActionIntent::None)
        .map(|(entity, _, _, _, _, _)| entity)
        .collect();

    for entity in to_move {
        let (
            metadata_id,
            pos,
            stock,
            req,
            phenotype_max_slope,
            phenotype_max_water_depth,
            phenotype_derived_movement_cost,
        ) = {
            let (_, metadata, pos, stock, req, phenotype) = agents.get(entity).unwrap();
            (
                metadata.id,
                *pos,
                *stock,
                *req,
                phenotype.max_slope,
                phenotype.max_water_depth,
                phenotype.derived_movement_cost,
            )
        };

        let intent = req.intent;
        let current_coord = pos.coord;
        let mut target_coord = current_coord;
        let mut valid_move = true;

        match intent {
            ActionIntent::MoveNorth => {
                if current_coord.y > 0 {
                    target_coord.y -= 1;
                } else {
                    valid_move = false;
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
                    valid_move = false;
                }
            }
            ActionIntent::None => unreachable!(),
        }

        if !valid_move || !bounds.contains_world_coord(target_coord) {
            mem_events.send(crate::agent::events::EventMemoryEvent::new(
                metadata_id,
                crate::agent::components::EventCategory::FailedMovement,
                clock.total_ticks,
                seq_counter.next_sequence(),
            ));
            if let Ok((_, _, _, _, mut original_req, _)) = agents.get_mut(entity) {
                original_req.intent = ActionIntent::None;
            }
            continue;
        }

        let target_chunk = crate::world::coord::world_to_chunk(target_coord, chunk_size);
        let local_coord = crate::world::coord::world_to_local(target_coord, chunk_size);
        let index = (local_coord.y * chunk_size + local_coord.x) as usize;

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
            if let Ok((_, _, _, _, mut original_req, _)) = agents.get_mut(entity) {
                original_req.intent = ActionIntent::None;
            }
            continue;
        };

        if slope > phenotype_max_slope || water_depth > phenotype_max_water_depth {
            events.send(ObservationEvent::new(
                metadata_id,
                target_coord,
                LocationCategory::Hazard,
            ));
            mem_events.send(crate::agent::events::EventMemoryEvent::new(
                metadata_id,
                crate::agent::components::EventCategory::HazardEncountered,
                clock.total_ticks,
                seq_counter.next_sequence(),
            ));
            mem_events.send(crate::agent::events::EventMemoryEvent::new(
                metadata_id,
                crate::agent::components::EventCategory::FailedMovement,
                clock.total_ticks,
                seq_counter.next_sequence(),
            ));
            if let Ok((_, _, _, _, mut original_req, _)) = agents.get_mut(entity) {
                original_req.intent = ActionIntent::None;
            }
            continue;
        }

        if let Ok((_, _, mut original_pos, mut original_stock, mut original_req, _)) =
            agents.get_mut(entity)
        {
            original_pos.coord = target_coord;
            original_stock.energy = (stock.energy - phenotype_derived_movement_cost).max(0.0);
            original_req.intent = ActionIntent::None;
        }
    }
}

/// Allows agents to passively sense their environment and record high-density resources.
pub fn process_agent_sensing(
    config: Res<WorldConfig>,
    bounds: Res<WorldBounds>,
    spatial_map: Res<crate::world::spatial::SpatialMap>,
    chunks: Query<&crate::world::resource::ResourceChunk>,
    mut events: EventWriter<ObservationEvent>,
    agents: Query<(&AgentMetadata, &AgentPosition, &Phenotype)>,
) {
    let threshold_nutrient = config.nutrients_max * 0.8;
    let threshold_water = config.fresh_water_max * 0.8;

    let mut sorted_agents: Vec<_> = agents
        .iter()
        .map(|(metadata, pos, pheno)| (metadata.id, pos.coord, pheno.sensing_radius))
        .collect();
    sorted_agents.sort_by_key(|a| a.0);

    for (agent_id, coord, sensing_radius) in sorted_agents {
        let neighborhood = crate::agent::sensing::query_neighborhood(
            coord,
            sensing_radius,
            &bounds,
            &spatial_map,
            &chunks,
        );
        for (target_coord, resource) in neighborhood {
            if resource.nutrients >= threshold_nutrient {
                events.send(ObservationEvent::new(
                    agent_id,
                    target_coord,
                    LocationCategory::Nutrient,
                ));
            }
            if resource.fresh_water >= threshold_water {
                events.send(ObservationEvent::new(
                    agent_id,
                    target_coord,
                    LocationCategory::FreshWater,
                ));
            }
        }
    }
}

/// Processes asexual reproduction for biological agents, validating energy/age constraints,
/// performing cardinal adjacent cell search, dividing energy, and setting up generational lineage.
#[allow(clippy::too_many_arguments)]
pub fn process_agent_reproduction(
    mut commands: Commands,
    config: Res<WorldConfig>,
    bounds: Res<WorldBounds>,
    seed: Res<WorldSeed>,
    clock: Res<SimulationClock>,
    mut id_gen: ResMut<StableIdGenerator>,
    terrain_chunks: Query<(&ChunkCoord, &TerrainChunk)>,
    mut mem_events: EventWriter<crate::agent::events::EventMemoryEvent>,
    mut seq_counter: ResMut<crate::agent::resources::EventSequenceCounter>,
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

    parents.sort_by_key(|p| p.0);

    for parent in parents {
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

        let directions = [
            (Some(parent_coord.x), parent_coord.y.checked_sub(1)),
            (Some(parent_coord.x), Some(parent_coord.y.saturating_add(1))),
            (Some(parent_coord.x.saturating_add(1)), Some(parent_coord.y)),
            (parent_coord.x.checked_sub(1), Some(parent_coord.y)),
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

            let target_chunk = crate::world::coord::world_to_chunk(candidate, chunk_size);
            let local_coord = crate::world::coord::world_to_local(candidate, chunk_size);
            let index = (local_coord.y * chunk_size + local_coord.x) as usize;

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

        if let Some(target_coord) = chosen_coord {
            if let Ok((_, _, _, mut parent_stock, _, _, _)) = agents.get_mut(parent_entity) {
                let offspring_energy = parent_stock.energy * 0.5;
                parent_stock.energy *= 0.5;

                let offspring_id = id_gen.next_id();
                let mutation_seed = derive_mutation_seed(
                    parent_id,
                    clock.total_ticks,
                    parent_coord,
                    seed.root_seed,
                );
                let mutated_genome = mutate_genome(
                    &parent_genome,
                    config.mutation_rate,
                    config.mutation_step_size,
                    mutation_seed,
                );

                commands.spawn((
                    AgentMetadata::new(offspring_id),
                    AgentPosition::new(target_coord),
                    MetabolicStock::new(offspring_energy, 0),
                    ActionRequest::new(ActionIntent::None),
                    mutated_genome,
                    LineageMetadata::new(Some(parent_id), parent_lineage.generation + 1),
                    LocationMemory::new(),
                    crate::agent::components::EventMemory::new(),
                ));

                mem_events.send(crate::agent::events::EventMemoryEvent::new(
                    parent_id,
                    crate::agent::components::EventCategory::Reproduced,
                    clock.total_ticks,
                    seq_counter.next_sequence(),
                ));

                current_population += 1;
            }
        }
    }
}

/// Consolidates ObservationEvents into agents' subjective location memory.
///
/// Ensures memory updates are deterministic and cap at `max_location_memory_capacity` using
/// chronological LRU eviction with coordinate-based tie-breaking.
pub fn process_memory_consolidation(
    config: Res<crate::config::WorldConfig>,
    clock: Res<crate::time::SimulationClock>,
    mut events: ResMut<Events<ObservationEvent>>,
    mut agents: Query<(&AgentMetadata, &mut LocationMemory)>,
) {
    let mut pending_events: Vec<_> = events.drain().collect();
    if pending_events.is_empty() {
        return;
    }

    // Sort to ensure deterministic processing order
    pending_events.sort_by(|a, b| {
        a.agent_id
            .cmp(&b.agent_id)
            .then(a.coord.y.cmp(&b.coord.y))
            .then(a.coord.x.cmp(&b.coord.x))
    });

    // Group events by agent_id to minimize query lookups
    let mut grouped_events: std::collections::HashMap<u64, Vec<ObservationEvent>> =
        std::collections::HashMap::new();
    for ev in pending_events {
        grouped_events.entry(ev.agent_id).or_default().push(ev);
    }

    for (metadata, mut memory) in &mut agents {
        if let Some(agent_events) = grouped_events.get(&metadata.id) {
            for event in agent_events {
                // Check if we already have this coordinate in memory
                let mut found = false;
                for node in &mut memory.nodes {
                    if node.coord == event.coord && node.category == event.category {
                        node.last_observed_tick = clock.total_ticks;
                        found = true;
                        break;
                    }
                }

                if !found {
                    memory
                        .nodes
                        .push(crate::agent::components::LocationMemoryNode::new(
                            event.coord,
                            event.category,
                            clock.total_ticks,
                        ));
                }
            }

            // Enforce capacity via deterministic LRU eviction
            if memory.nodes.len() > config.max_location_memory_capacity {
                // Sort by last_observed_tick descending (newest first).
                // Tie breaker: coord.y descending, coord.x descending (so largest coord kept first, smallest coord evicted first)
                memory.nodes.sort_by(|a, b| {
                    b.last_observed_tick
                        .cmp(&a.last_observed_tick)
                        .then(b.coord.y.cmp(&a.coord.y))
                        .then(b.coord.x.cmp(&a.coord.x))
                });
                memory.nodes.truncate(config.max_location_memory_capacity);
            }
        }
    }
}

/// Resets the event sequence counter at the beginning of each tick.
pub fn reset_event_sequence(mut counter: ResMut<crate::agent::resources::EventSequenceCounter>) {
    counter.reset();
}

/// Consolidates EventMemoryEvents into agents' subjective episodic memory.
///
/// Ensures memory updates are deterministic and cap at `MAX_EVENT_MEMORY_CAPACITY`.
pub fn process_event_memory_consolidation(
    mut events: ResMut<Events<crate::agent::events::EventMemoryEvent>>,
    mut agents: Query<(&AgentMetadata, &mut crate::agent::components::EventMemory)>,
) {
    let mut pending_events: Vec<_> = events.drain().collect();
    if pending_events.is_empty() {
        return;
    }

    // Sort to ensure deterministic processing order.
    // The spec requires sorting by (tick, sequence_in_tick)
    pending_events.sort_by(|a, b| {
        a.agent_id
            .cmp(&b.agent_id)
            .then(a.tick.cmp(&b.tick))
            .then(a.sequence_in_tick.cmp(&b.sequence_in_tick))
    });

    let mut grouped_events: std::collections::HashMap<
        u64,
        Vec<crate::agent::events::EventMemoryEvent>,
    > = std::collections::HashMap::new();
    for ev in pending_events {
        grouped_events.entry(ev.agent_id).or_default().push(ev);
    }

    for (metadata, mut memory) in &mut agents {
        if let Some(agent_events) = grouped_events.get(&metadata.id) {
            for event in agent_events {
                memory
                    .nodes
                    .push(crate::agent::components::EventMemoryNode::new(
                        event.category,
                        event.tick,
                        event.sequence_in_tick,
                    ));
            }

            // Enforce capacity via deterministic LRU eviction (oldest first)
            if memory.nodes.len() > crate::agent::components::MAX_EVENT_MEMORY_CAPACITY {
                let overflow =
                    memory.nodes.len() - crate::agent::components::MAX_EVENT_MEMORY_CAPACITY;
                memory.nodes.drain(0..overflow);
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
        world.insert_resource(bounds.clone());
        world.insert_resource(crate::world::spatial::SpatialMap::new(
            bounds.chunks_x,
            bounds.chunks_y,
        ));

        let entity = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
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
    fn test_matching_phenotype_has_zero_penalty() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_base_decay_rate: 1.0,
            thermal_penalty_multiplier: 2.0,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds.clone());
        world.insert_resource(crate::world::spatial::SpatialMap::new(
            bounds.chunks_x,
            bounds.chunks_y,
        ));

        let a1 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
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
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(update_agent_metabolism);
        world.add_schedule(schedule);
        world.run_schedule(crate::app::FixedSimulationTick);

        // No climate chunk means temp falls back to phenotype.thermal_optimum (0.5).
        // Delta T = 0. Penalty = 0.
        // Total decay = base_decay = 1.0.
        assert_eq!(
            world.entity(a1).get::<MetabolicStock>().unwrap().energy,
            99.0
        );
    }

    #[test]
    fn test_thermal_penalty_is_linear() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_base_decay_rate: 1.0,
            thermal_penalty_multiplier: 2.0,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds.clone());
        let mut spatial_map =
            crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);

        let mut temp = vec![0.5f32; 1024];
        temp[0] = 0.0; // delta 0.5 -> penalty 1.0
        temp[1] = -0.5; // delta 1.0 -> penalty 2.0

        let e = world
            .spawn((
                ChunkCoord::new(0, 0),
                ClimateChunk {
                    temperature: temp,
                    moisture: vec![0.0; 1024],
                    rainfall: vec![0.0; 1024],
                    sunlight_factor: vec![0.0; 1024],
                },
            ))
            .id();
        spatial_map.set(ChunkCoord::new(0, 0), e);
        world.insert_resource(spatial_map);

        let a1 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
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
            ))
            .id();

        let a2 = world
            .spawn((
                AgentPosition::new(WorldCoord::new(1, 0)),
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
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(update_agent_metabolism);
        world.add_schedule(schedule);
        world.run_schedule(crate::app::FixedSimulationTick);

        // a1: base 1.0 + penalty 1.0 = 2.0 decay
        assert_eq!(
            world.entity(a1).get::<MetabolicStock>().unwrap().energy,
            98.0
        );

        // a2: base 1.0 + penalty 2.0 = 3.0 decay
        assert_eq!(
            world.entity(a2).get::<MetabolicStock>().unwrap().energy,
            97.0
        );
    }

    #[test]
    fn test_unfit_agent_starves_faster() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_base_decay_rate: 1.0,
            thermal_penalty_multiplier: 10.0,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds.clone());
        let mut spatial_map =
            crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);

        let temp = vec![0.5f32; 1024];
        let e = world
            .spawn((
                ChunkCoord::new(0, 0),
                ClimateChunk {
                    temperature: temp,
                    moisture: vec![0.0; 1024],
                    rainfall: vec![0.0; 1024],
                    sunlight_factor: vec![0.0; 1024],
                },
            ))
            .id();
        spatial_map.set(ChunkCoord::new(0, 0), e);
        world.insert_resource(spatial_map);

        // Fit agent
        let fit = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
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
            ))
            .id();

        // Unfit agent
        let unfit = world
            .spawn((
                AgentPosition::new(WorldCoord::new(1, 0)),
                MetabolicStock::new(100.0, 0),
                Phenotype {
                    thermal_optimum: 1.0,
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
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(update_agent_metabolism);
        world.add_schedule(schedule);

        // Run 15 ticks. Fit loses 15 energy. Unfit loses 15 + (0.5 * 10 * 15) = 15 + 75 = 90 energy.
        for _ in 0..15 {
            world.run_schedule(crate::app::FixedSimulationTick);
        }

        assert_eq!(
            world.entity(fit).get::<MetabolicStock>().unwrap().energy,
            85.0
        );
        assert_eq!(
            world.entity(unfit).get::<MetabolicStock>().unwrap().energy,
            10.0
        );
    }

    #[test]
    fn test_deterministic_mutation() {
        let parent = Genome::new(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]);
        let rate = 0.5;
        let step = 0.1;
        let seed = 123456789;

        let child1 = mutate_genome(&parent, rate, step, seed);
        let child2 = mutate_genome(&parent, rate, step, seed);

        assert_eq!(child1, child2);
    }

    #[test]
    fn test_gene_bound_clamping() {
        let parent = Genome::new(vec![0.0, 1.0, 0.05, 0.95]);
        let rate = 1.0;
        let step = 10.0;
        let seed = 42;

        let child = mutate_genome(&parent, rate, step, seed);
        for &gene in &child.genes {
            assert!(gene >= 0.0 && gene <= 1.0, "Gene out of bounds: {}", gene);
            assert!(gene.is_finite(), "Gene is not finite");
        }
    }

    #[test]
    fn test_parent_immutability() {
        let parent = Genome::new(vec![0.5; crate::agent::GENOME_SIZE]);
        let rate = 1.0;
        let step = 0.2;
        let seed = 999;

        let child = mutate_genome(&parent, rate, step, seed);
        assert_ne!(child.genes, parent.genes);
        assert_eq!(parent.genes, vec![0.5; 8]);
    }

    #[test]
    fn test_mutation_step_distribution() {
        let parent = Genome::new(vec![0.5]);
        let rate = 1.0;
        let step = 0.1;

        let mut sum_diff = 0.0;
        let mut sum_diff_sq = 0.0;
        let count = 2000;

        for seed in 0..count {
            let child = mutate_genome(&parent, rate, step, seed as u64);
            let diff = child.genes[0] - parent.genes[0];
            sum_diff += diff;
            sum_diff_sq += diff * diff;
        }

        let mean = sum_diff / count as f32;
        let variance = sum_diff_sq / count as f32 - mean * mean;
        let std_dev = variance.sqrt();

        assert!(
            mean.abs() < 0.02,
            "Mean of mutations too far from 0: {}",
            mean
        );
        assert!(
            (std_dev - 0.1).abs() < 0.02,
            "Std dev of mutations too far from 0.1: {}",
            std_dev
        );
    }

    #[test]
    fn test_climate_adaptation() {
        let mut world = World::new();
        let config = WorldConfig {
            agent_base_decay_rate: 1.0,
            thermal_penalty_multiplier: 10.0,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds.clone());
        let mut spatial_map =
            crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);

        // Cold environment
        let temp = vec![-10.0f32; 1024];
        let e = world
            .spawn((
                ChunkCoord::new(0, 0),
                ClimateChunk {
                    temperature: temp,
                    moisture: vec![0.0; 1024],
                    rainfall: vec![0.0; 1024],
                    sunlight_factor: vec![0.0; 1024],
                },
            ))
            .id();
        spatial_map.set(ChunkCoord::new(0, 0), e);
        world.insert_resource(spatial_map);

        // Cold-adapted agent (low thermal optimum)
        let adapted = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
                Phenotype {
                    thermal_optimum: -10.0,
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
            ))
            .id();

        // Unadapted agent (high thermal optimum)
        let unadapted = world
            .spawn((
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
                Phenotype {
                    thermal_optimum: 20.0,
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
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(update_agent_metabolism);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let stock_adapted = world.get::<MetabolicStock>(adapted).unwrap().energy;
        let stock_unadapted = world.get::<MetabolicStock>(unadapted).unwrap().energy;

        // Adapted agent only suffers base decay
        assert_eq!(stock_adapted, 99.0);
        // Unadapted agent suffers massive thermal penalty
        assert!(stock_unadapted < stock_adapted);
    }

    #[test]
    fn test_terrain_specialization() {
        let mut world = World::new();
        let config = WorldConfig {
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        world.insert_resource(config);
        world.insert_resource(bounds.clone());
        world.init_resource::<bevy_ecs::event::Events<ObservationEvent>>();
        world.insert_resource(crate::time::SimulationClock::default());
        world.init_resource::<bevy_ecs::event::Events<crate::agent::events::EventMemoryEvent>>();
        world.insert_resource(crate::agent::resources::EventSequenceCounter::default());

        // Steep terrain in cell (0, 1) (South of origin)
        let mut slope = vec![0.0f32; 1024];
        let chunk_size = bounds.chunk_size;
        let target_index = chunk_size as usize;
        slope[target_index] = 0.5; // Very steep

        world.spawn((
            ChunkCoord::new(0, 0),
            TerrainChunk {
                elevation: vec![0.0; 1024],
                slope,
                water_depth: vec![0.0; 1024],
                soil_fertility: vec![0.0; 1024],
                soil_depth: vec![0.0; 1024],
            },
        ));

        // Specialized agent (high slope tolerance)
        let specialized = world
            .spawn((
                AgentMetadata::new(1),
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
                ActionRequest {
                    intent: ActionIntent::MoveSouth,
                },
                Phenotype {
                    thermal_optimum: 0.0,
                    diet_preference: 0.0,
                    max_slope: 0.6,
                    max_water_depth: 0.0,
                    sensing_radius: 1,
                    reproduction_threshold: 0.0,
                    maturity_age: 0,
                    physical_size: 1.0,
                    derived_base_decay: 1.0,
                    derived_movement_cost: 2.0,
                },
            ))
            .id();

        // Unadapted agent (low slope tolerance)
        let unadapted = world
            .spawn((
                AgentMetadata::new(2),
                AgentPosition::new(WorldCoord::new(0, 0)),
                MetabolicStock::new(100.0, 0),
                ActionRequest {
                    intent: ActionIntent::MoveSouth,
                },
                Phenotype {
                    thermal_optimum: 0.0,
                    diet_preference: 0.0,
                    max_slope: 0.2,
                    max_water_depth: 0.0,
                    sensing_radius: 1,
                    reproduction_threshold: 0.0,
                    maturity_age: 0,
                    physical_size: 1.0,
                    derived_base_decay: 1.0,
                    derived_movement_cost: 1.0,
                },
            ))
            .id();

        let mut schedule = Schedule::new(crate::app::FixedSimulationTick);
        schedule.add_systems(process_agent_movement);
        world.add_schedule(schedule);

        world.run_schedule(crate::app::FixedSimulationTick);

        let pos_spec = world.get::<AgentPosition>(specialized).unwrap().coord;
        let stock_spec = world.get::<MetabolicStock>(specialized).unwrap().energy;
        let req_spec = world.get::<ActionRequest>(specialized).unwrap().intent;

        let pos_unad = world.get::<AgentPosition>(unadapted).unwrap().coord;
        let stock_unad = world.get::<MetabolicStock>(unadapted).unwrap().energy;
        let req_unad = world.get::<ActionRequest>(unadapted).unwrap().intent;

        // Specialized agent successfully moved south
        assert_eq!(pos_spec, WorldCoord::new(0, 1));
        assert_eq!(stock_spec, 98.0); // 100 - 2.0 cost
        assert_eq!(req_spec, ActionIntent::None);

        // Unadapted agent failed to move
        assert_eq!(pos_unad, WorldCoord::new(0, 0));
        assert_eq!(stock_unad, 100.0); // no cost deducted
        assert_eq!(req_unad, ActionIntent::None);
    }
}

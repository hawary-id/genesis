use crate::config::{WorldBounds, WorldConfig};
use crate::time::season_state::validate_season_state;
use crate::time::{SeasonState, SimulationClock};
use crate::validation::ValidationError;
use crate::world::climate::{validate_climate_chunk, ClimateChunk};
use crate::world::coord::ChunkCoord;
use crate::world::energy::{validate_energy_chunk, EnergyAvailabilityChunk};
use crate::world::resource::{validate_resource_chunk, ResourceChunk};
use crate::world::terrain::{validate_terrain_chunk, TerrainChunk};
use crate::agent::{AgentMetadata, AgentPosition, MetabolicStock};
use bevy_ecs::prelude::*;

/// Startup validation system. Runs at the end of the `StartupGeneration` schedule.
pub fn validate_world_on_startup(
    config: Res<WorldConfig>,
    bounds: Res<WorldBounds>,
    query: Query<(
        &ChunkCoord,
        &TerrainChunk,
        &ClimateChunk,
        &ResourceChunk,
        &EnergyAvailabilityChunk,
    )>,
    agents_query: Query<(&AgentMetadata, &AgentPosition, &MetabolicStock)>,
) {
    let expected_chunks = (bounds.chunks_x * bounds.chunks_y) as usize;
    let actual_chunks = query.iter().count();

    if actual_chunks != expected_chunks {
        let err = ValidationError::ChunkInconsistency {
            coord: ChunkCoord::new(0, 0),
            detail: "total chunk count mismatch against world dimensions",
        };
        handle_validation_error(err);
        return;
    }

    // Sort chunks deterministically to guarantee order-independent checks
    let mut chunks: Vec<_> = query.iter().collect();
    chunks.sort_by_key(|(coord, _, _, _, _)| (coord.y, coord.x));

    for (coord, terrain, climate, resource, energy) in chunks {
        if let Err(err) = validate_terrain_chunk(coord, terrain, &config) {
            handle_validation_error(err);
            return;
        }
        if let Err(err) = validate_climate_chunk(coord, climate, &config) {
            handle_validation_error(err);
            return;
        }
        if let Err(err) = validate_resource_chunk(coord, resource, &config) {
            handle_validation_error(err);
            return;
        }
        if let Err(err) = validate_energy_chunk(coord, energy, &config) {
            handle_validation_error(err);
            return;
        }
    }

    // Validate agents at startup
    let expected_agent_count = (config.initial_agent_count.min(config.agent_density_cap)) as usize;
    let actual_agent_count = agents_query.iter().count();
    if actual_agent_count != expected_agent_count {
        handle_validation_error(ValidationError::AgentCountMismatch {
            expected: expected_agent_count,
            actual: actual_agent_count,
        });
        return;
    }

    // Sort agents by stable ID to guarantee order-independent verification
    let mut agents: Vec<_> = agents_query.iter().collect();
    agents.sort_by_key(|(meta, _, _)| meta.id);

    let mut seen_ids = std::collections::HashSet::new();

    for (meta, pos, stock) in agents {
        // Unique non-zero IDs
        if meta.id == 0 || !seen_ids.insert(meta.id) {
            handle_validation_error(ValidationError::AgentDuplicateId {
                agent_id: meta.id,
            });
            return;
        }

        // Position bounds
        if pos.coord.x >= bounds.world_width || pos.coord.y >= bounds.world_height {
            handle_validation_error(ValidationError::AgentPositionOutOfBounds {
                agent_id: meta.id,
                coord: pos.coord,
            });
            return;
        }

        // Initial stocks: energy must be exactly initial_agent_energy
        if stock.energy != config.initial_agent_energy || stock.energy < 0.0 || stock.energy > config.agent_energy_max {
            handle_validation_error(ValidationError::AgentEnergyOutOfBounds {
                agent_id: meta.id,
                energy: stock.energy,
            });
            return;
        }

        if stock.age != 0 {
            handle_validation_error(ValidationError::AgentAgeOutOfBounds {
                agent_id: meta.id,
                age: stock.age,
            });
            return;
        }
    }
}

/// Tick validation system. Runs within the `PostTickValidation` schedule.
pub fn validate_world_on_tick(
    config: Res<WorldConfig>,
    bounds: Res<WorldBounds>,
    clock: Res<SimulationClock>,
    season_state: Res<SeasonState>,
    query: Query<(
        &ChunkCoord,
        &ClimateChunk,
        &ResourceChunk,
        &EnergyAvailabilityChunk,
    )>,
    agents_query: Query<(&AgentMetadata, &AgentPosition, &MetabolicStock)>,
    mut previous_tick: Local<Option<u32>>,
) {
    // 1. Clock monotonicity check
    if let Some(prev) = *previous_tick {
        if clock.total_ticks < prev {
            let err = ValidationError::ClockRegression {
                previous: prev,
                current: clock.total_ticks,
            };
            handle_validation_error(err);
            return;
        }
    }
    *previous_tick = Some(clock.total_ticks);

    // 2. SeasonState derivation check
    if let Err(err) = validate_season_state(&clock, &season_state, &config) {
        handle_validation_error(err);
        return;
    }

    // 3. Sweep all active chunks sorted deterministically
    let mut chunks: Vec<_> = query.iter().collect();
    chunks.sort_by_key(|(coord, _, _, _)| (coord.y, coord.x));

    for (coord, climate, resource, energy) in chunks {
        if let Err(err) = validate_climate_chunk(coord, climate, &config) {
            handle_validation_error(err);
            return;
        }
        if let Err(err) = validate_resource_chunk(coord, resource, &config) {
            handle_validation_error(err);
            return;
        }
        if let Err(err) = validate_energy_chunk(coord, energy, &config) {
            handle_validation_error(err);
            return;
        }
    }

    // 4. Validate active agents on tick
    let actual_agent_count = agents_query.iter().count();
    if actual_agent_count > config.agent_density_cap as usize {
        handle_validation_error(ValidationError::AgentCountMismatch {
            expected: config.agent_density_cap as usize,
            actual: actual_agent_count,
        });
        return;
    }

    // Sort agents by stable ID to guarantee order-independent checks
    let mut agents: Vec<_> = agents_query.iter().collect();
    agents.sort_by_key(|(meta, _, _)| meta.id);

    let mut seen_ids = std::collections::HashSet::new();

    for (meta, pos, stock) in agents {
        // Unique non-zero IDs
        if meta.id == 0 || !seen_ids.insert(meta.id) {
            handle_validation_error(ValidationError::AgentDuplicateId {
                agent_id: meta.id,
            });
            return;
        }

        // Position bounds
        if pos.coord.x >= bounds.world_width || pos.coord.y >= bounds.world_height {
            handle_validation_error(ValidationError::AgentPositionOutOfBounds {
                agent_id: meta.id,
                coord: pos.coord,
            });
            return;
        }

        // Metabolic stock limits
        if stock.energy < 0.0 || stock.energy > config.agent_energy_max {
            handle_validation_error(ValidationError::AgentEnergyOutOfBounds {
                agent_id: meta.id,
                energy: stock.energy,
            });
            return;
        }

        if stock.age > config.agent_age_limit {
            handle_validation_error(ValidationError::AgentAgeOutOfBounds {
                agent_id: meta.id,
                age: stock.age,
            });
            return;
        }
    }
}

/// Dispatches validation errors according to compilation targets.
fn handle_validation_error(err: ValidationError) {
    #[cfg(any(debug_assertions, test))]
    {
        panic!("Genesis Invariant Violation in Debug/Test Mode: {:?}", err);
    }
    #[cfg(not(any(debug_assertions, test)))]
    {
        eprintln!("Genesis Invariant Violation in Release Mode: {:?}", err);
        // TODO: Surface structured validation error to future runner/application layers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::WorldGenerationCompleted;
    use crate::config::WorldBounds;
    use crate::rng::WorldSeed;
    use bevy_ecs::event::Events;

    fn test_world() -> World {
        let mut world = World::new();
        let config = WorldConfig {
            world_width: 64,
            world_height: 64,
            chunk_size: 16,
            initial_agent_count: 0,
            ..WorldConfig::default()
        };
        let bounds = WorldBounds::from_config(&config);
        let seed = WorldSeed::new(12345);
        let clock = SimulationClock::default();
        let season_state = SeasonState::derive(0, &config);

        world.insert_resource(config);
        world.insert_resource(bounds);
        world.insert_resource(seed);
        world.insert_resource(clock);
        world.insert_resource(season_state);
        world.insert_resource(crate::agent::StableIdGenerator::new());
        world.init_resource::<Events<WorldGenerationCompleted>>();

        let mut schedules = bevy_ecs::schedule::Schedules::default();

        let mut startup_schedule = Schedule::new(crate::app::StartupGeneration);
        startup_schedule.add_systems((
            crate::world::generation::validate_world_config,
            crate::world::generation::spawn_chunk_entities
                .after(crate::world::generation::validate_world_config),
            crate::world::generation::generate_terrain_chunks
                .after(crate::world::generation::spawn_chunk_entities),
            crate::world::generation::generate_climate_chunks
                .after(crate::world::generation::generate_terrain_chunks),
            crate::world::generation::generate_resource_chunks
                .after(crate::world::generation::generate_climate_chunks),
            crate::world::energy::generate_energy_availability_chunks
                .after(crate::world::generation::generate_resource_chunks),
            validate_world_on_startup
                .after(crate::world::energy::generate_energy_availability_chunks),
            crate::world::generation::mark_chunks_generated.after(validate_world_on_startup),
            crate::world::generation::emit_world_generation_completed
                .after(crate::world::generation::mark_chunks_generated),
        ));
        schedules.insert(startup_schedule);

        let mut post_tick_schedule = Schedule::new(crate::app::PostTickValidation);
        post_tick_schedule.add_systems(validate_world_on_tick);
        schedules.insert(post_tick_schedule);

        world.insert_resource(schedules);

        world
    }

    #[test]
    fn test_validation_error_construction() {
        let err1 = ValidationError::ClockRegression {
            previous: 10,
            current: 5,
        };
        let err2 = ValidationError::SeasonStateMismatch { total_ticks: 10 };
        let err3 = ValidationError::ChunkInconsistency {
            coord: ChunkCoord::new(0, 0),
            detail: "test",
        };
        let err4 = ValidationError::TerrainOutOfBounds {
            coord: ChunkCoord::new(0, 0),
            field: "elevation",
            value: 1.5,
        };
        let err5 = ValidationError::ClimateOutOfBounds {
            coord: ChunkCoord::new(0, 0),
            field: "temperature",
            value: 2.0,
        };
        let err6 = ValidationError::ResourceOutOfBounds {
            coord: ChunkCoord::new(0, 0),
            field: "fresh_water",
            value: -1.0,
        };
        let err7 = ValidationError::EnergyOutOfBounds {
            coord: ChunkCoord::new(0, 0),
            field: "solar_exposure",
            value: 100.0,
        };

        assert_eq!(
            err1,
            ValidationError::ClockRegression {
                previous: 10,
                current: 5
            }
        );
        assert_eq!(
            err2,
            ValidationError::SeasonStateMismatch { total_ticks: 10 }
        );
        assert_eq!(
            err3,
            ValidationError::ChunkInconsistency {
                coord: ChunkCoord::new(0, 0),
                detail: "test"
            }
        );
        assert_eq!(
            err4,
            ValidationError::TerrainOutOfBounds {
                coord: ChunkCoord::new(0, 0),
                field: "elevation",
                value: 1.5
            }
        );
        assert_eq!(
            err5,
            ValidationError::ClimateOutOfBounds {
                coord: ChunkCoord::new(0, 0),
                field: "temperature",
                value: 2.0
            }
        );
        assert_eq!(
            err6,
            ValidationError::ResourceOutOfBounds {
                coord: ChunkCoord::new(0, 0),
                field: "fresh_water",
                value: -1.0
            }
        );
        assert_eq!(
            err7,
            ValidationError::EnergyOutOfBounds {
                coord: ChunkCoord::new(0, 0),
                field: "solar_exposure",
                value: 100.0
            }
        );
    }

    #[test]
    fn test_startup_validation_execution_success() {
        let mut world = test_world();
        world.run_schedule(crate::app::StartupGeneration);
    }

    #[test]
    fn test_post_tick_validation_execution_success() {
        let mut world = test_world();
        world.run_schedule(crate::app::StartupGeneration);
        world.run_schedule(crate::app::PostTickValidation);
    }

    #[test]
    #[should_panic(expected = "ClockRegression")]
    fn test_validation_catches_clock_regression() {
        let mut world = test_world();
        world.run_schedule(crate::app::StartupGeneration);

        // Run first validation tick to set the previous_tick local to Some(0)
        world.run_schedule(crate::app::PostTickValidation);

        // Let's set clock to 10, update season_state to tick 10, and tick once
        {
            let config = world.resource::<WorldConfig>().clone();
            let mut clock = world.resource_mut::<SimulationClock>();
            clock.total_ticks = 10;
            let mut season_state = world.resource_mut::<SeasonState>();
            *season_state = SeasonState::derive(10, &config);
        }
        world.run_schedule(crate::app::PostTickValidation);

        // Now regress to 5
        {
            let mut clock = world.resource_mut::<SimulationClock>();
            clock.total_ticks = 5;
        }
        world.run_schedule(crate::app::PostTickValidation);
    }

    #[test]
    #[should_panic(expected = "SeasonStateMismatch")]
    fn test_validation_catches_season_mismatch() {
        let mut world = test_world();
        world.run_schedule(crate::app::StartupGeneration);

        // Modify SeasonState to conflict with derived state
        {
            let mut season_state = world.resource_mut::<SeasonState>();
            season_state.season_index = 99; // Invalid
        }

        world.run_schedule(crate::app::PostTickValidation);
    }

    #[test]
    #[should_panic(expected = "ResourceOutOfBounds")]
    fn test_validation_catches_resource_out_of_bounds() {
        let mut world = test_world();
        world.run_schedule(crate::app::StartupGeneration);

        // Corrupt resources in one of the spawned chunks
        {
            let mut query = world.query::<&mut ResourceChunk>();
            let mut resource = query.iter_mut(&mut world).next().unwrap();
            resource.nutrients[0] = -0.5; // Nutrient must be non-negative
        }

        world.run_schedule(crate::app::PostTickValidation);
    }

    #[test]
    #[should_panic(expected = "EnergyOutOfBounds")]
    fn test_validation_catches_energy_out_of_bounds() {
        let mut world = test_world();
        world.run_schedule(crate::app::StartupGeneration);

        // Corrupt energy in one of the chunks
        {
            let mut query = world.query::<&mut EnergyAvailabilityChunk>();
            let mut energy = query.iter_mut(&mut world).next().unwrap();
            energy.solar_exposure[0] = 5000.0; // Limit is config.solar_exposure_max
        }

        world.run_schedule(crate::app::PostTickValidation);
    }

    #[test]
    #[should_panic(expected = "AgentDuplicateId")]
    fn test_validation_catches_duplicate_agent_ids() {
        let mut world = test_world();
        world.run_schedule(crate::app::StartupGeneration);

        // Spawn two agents with duplicate ID
        world.spawn((
            AgentMetadata::new(10),
            AgentPosition::new(crate::world::coord::WorldCoord::new(0, 0)),
            MetabolicStock::new(100.0, 0),
        ));
        world.spawn((
            AgentMetadata::new(10),
            AgentPosition::new(crate::world::coord::WorldCoord::new(1, 1)),
            MetabolicStock::new(100.0, 0),
        ));

        world.run_schedule(crate::app::PostTickValidation);
    }

    #[test]
    #[should_panic(expected = "AgentPositionOutOfBounds")]
    fn test_validation_catches_agent_out_of_bounds() {
        let mut world = test_world();
        world.run_schedule(crate::app::StartupGeneration);

        // Spawn agent out of bounds (world width is 64)
        world.spawn((
            AgentMetadata::new(1),
            AgentPosition::new(crate::world::coord::WorldCoord::new(100, 0)),
            MetabolicStock::new(100.0, 0),
        ));

        world.run_schedule(crate::app::PostTickValidation);
    }

    #[test]
    #[should_panic(expected = "AgentCountMismatch")]
    fn test_validation_catches_agent_count_mismatch() {
        let mut world = test_world();
        // Modify config to expect 5 agents
        {
            let mut config = world.resource_mut::<WorldConfig>();
            config.initial_agent_count = 5;
        }

        // Spawn only 2 agents
        world.spawn((
            AgentMetadata::new(1),
            AgentPosition::new(crate::world::coord::WorldCoord::new(0, 0)),
            MetabolicStock::new(100.0, 0),
        ));
        world.spawn((
            AgentMetadata::new(2),
            AgentPosition::new(crate::world::coord::WorldCoord::new(1, 1)),
            MetabolicStock::new(100.0, 0),
        ));

        // This schedule run calls validate_world_on_startup
        world.run_schedule(crate::app::StartupGeneration);
    }

    #[test]
    #[should_panic(expected = "AgentEnergyOutOfBounds")]
    fn test_validation_catches_invalid_agent_stocks() {
        let mut world = test_world();
        world.run_schedule(crate::app::StartupGeneration);

        // Spawn agent with negative energy
        world.spawn((
            AgentMetadata::new(1),
            AgentPosition::new(crate::world::coord::WorldCoord::new(0, 0)),
            MetabolicStock::new(-5.0, 0),
        ));

        world.run_schedule(crate::app::PostTickValidation);
    }

    #[test]
    #[should_panic(expected = "AgentAgeOutOfBounds")]
    fn test_validation_catches_invalid_agent_age() {
        let mut world = test_world();
        world.run_schedule(crate::app::StartupGeneration);

        // Spawn agent with age exceeding config limit
        let limit = world.resource::<WorldConfig>().agent_age_limit;
        world.spawn((
            AgentMetadata::new(1),
            AgentPosition::new(crate::world::coord::WorldCoord::new(0, 0)),
            MetabolicStock::new(100.0, limit + 1),
        ));

        world.run_schedule(crate::app::PostTickValidation);
    }
}

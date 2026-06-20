//! Snapshot schema types for the Genesis persistence layer.
//!
//! [`WorldSnapshot`] is the top-level document written to disk.
//! [`ChunkSnapshot`] records the complete state of one chunk entity.

use serde::{Deserialize, Serialize};

use crate::agent::{
    AgentMetadata, AgentPosition, Genome, LineageMetadata, MetabolicStock, StableIdGenerator,
};
use crate::config::WorldConfig;
use crate::rng::WorldSeed;
use crate::world::climate::ClimateChunk;
use crate::world::coord::ChunkCoord;
use crate::world::energy::EnergyAvailabilityChunk;
use crate::world::resource::ResourceChunk;
use crate::world::terrain::TerrainChunk;

/// Top-level snapshot document.
///
/// Contains all state required to resume deterministic simulation from the
/// tick at which the snapshot was taken.
///
/// `schema_version` must be checked against [`crate::persistence::SNAPSHOT_SCHEMA_VERSION`]
/// before the snapshot is used. A mismatch returns
/// [`crate::persistence::SnapshotError::SchemaMismatch`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Schema version. Must equal [`crate::persistence::SNAPSHOT_SCHEMA_VERSION`] on load.
    pub schema_version: u32,

    /// Total simulation ticks elapsed at the time of snapshot.
    ///
    /// This is the canonical time value. All derived state (season, day, year)
    /// is reconstructed from this field and `config`.
    pub total_ticks: u32,

    /// World configuration at snapshot time.
    ///
    /// Required to reconstruct `WorldBounds`, `SeasonState`, and all
    /// validation ranges on resume.
    pub config: WorldConfig,

    /// Root world seed.
    ///
    /// Required for deterministic continuation. Domain seeds are re-derived
    /// from this root on demand.
    pub seed: WorldSeed,

    /// Stable identifier generator state.
    pub id_generator: StableIdGenerator,

    /// Chunk state records.
    ///
    /// Ordered by `(coord.y, coord.x)` ascending to satisfy ADR-002
    /// stable iteration requirements. The load path must not assume any
    /// other ordering.
    pub chunks: Vec<ChunkSnapshot>,

    /// Agent state records.
    ///
    /// Ordered by stable ID ascending to guarantee deterministic persistence.
    pub agents: Vec<AgentSnapshot>,
}

/// Complete state of one agent entity at snapshot time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSnapshot {
    /// Unique stable identifier metadata.
    pub metadata: AgentMetadata,

    /// Spatial grid coordinates.
    pub position: AgentPosition,

    /// Metabolic stocks (energy, age).
    pub stock: MetabolicStock,

    /// Raw genetic values vector.
    pub genome: Genome,

    /// Generational lineage metadata.
    pub lineage: LineageMetadata,

    /// Subjective spatial memory of notable locations.
    #[serde(default)]
    pub location_memory: Option<crate::agent::LocationMemory>,

    /// Subjective episodic memory of key experiential events.
    #[serde(default)]
    pub event_memory: Option<crate::agent::components::EventMemory>,
}

/// Complete state of one chunk entity at snapshot time.
///
/// All five domain components are included. The load path spawns one ECS entity
/// per `ChunkSnapshot` and attaches these components directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSnapshot {
    /// Chunk grid coordinate. Used to spawn the entity with the correct `ChunkCoord`.
    pub coord: ChunkCoord,

    /// Terrain field data (elevation, slope, water depth, soil depth, soil fertility).
    pub terrain: TerrainChunk,

    /// Climate field data (temperature, moisture, rainfall, sunlight factor).
    pub climate: ClimateChunk,

    /// Resource field data (fresh water, nutrients, minerals, biomass potential).
    pub resources: ResourceChunk,

    /// Energy availability field data (solar exposure, aggregate energy).
    pub energy: EnergyAvailabilityChunk,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::SnapshotError;
    use crate::persistence::SNAPSHOT_SCHEMA_VERSION;
    use crate::testing::{create_test_config, create_test_seed};

    #[test]
    fn world_snapshot_schema_version_field_is_accessible() {
        let snapshot = WorldSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            total_ticks: 0,
            config: create_test_config(),
            seed: create_test_seed(),
            id_generator: StableIdGenerator::new(),
            chunks: vec![],
            agents: vec![],
        };
        assert_eq!(snapshot.schema_version, 4);
    }

    #[test]
    fn world_snapshot_total_ticks_field_is_accessible() {
        let snapshot = WorldSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            total_ticks: 42,
            config: create_test_config(),
            seed: create_test_seed(),
            id_generator: StableIdGenerator::new(),
            chunks: vec![],
            agents: vec![],
        };
        assert_eq!(snapshot.total_ticks, 42);
    }

    #[test]
    fn world_snapshot_empty_chunks_is_valid() {
        let snapshot = WorldSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            total_ticks: 0,
            config: create_test_config(),
            seed: create_test_seed(),
            id_generator: StableIdGenerator::new(),
            chunks: vec![],
            agents: vec![],
        };
        assert!(snapshot.chunks.is_empty());
    }

    #[test]
    fn snapshot_error_schema_mismatch_carries_versions() {
        let err = SnapshotError::SchemaMismatch {
            found: 99,
            expected: 1,
        };
        match err {
            SnapshotError::SchemaMismatch { found, expected } => {
                assert_eq!(found, 99);
                assert_eq!(expected, 1);
            }
            _ => panic!("expected SchemaMismatch variant"),
        }
    }

    #[test]
    fn snapshot_error_incomplete_carries_detail() {
        let err = SnapshotError::IncompleteSnapshot {
            detail: "missing chunks",
        };
        match err {
            SnapshotError::IncompleteSnapshot { detail } => {
                assert_eq!(detail, "missing chunks");
            }
            _ => panic!("expected IncompleteSnapshot variant"),
        }
    }

    #[test]
    fn round_trip_serialization() {
        use crate::app::App;
        use crate::persistence::build_world_snapshot;
        use crate::time::SimulationClock;
        use crate::world::climate::ClimateChunk;
        use crate::world::coord::ChunkCoord;
        use crate::world::energy::EnergyAvailabilityChunk;
        use crate::world::resource::ResourceChunk;
        use crate::world::terrain::TerrainChunk;

        let config = create_test_config();
        let seed = create_test_seed();
        let mut app = App::new(config.clone(), seed);
        app.run_startup();
        let world = app.world_mut();

        let mut query = world.query::<(
            &ChunkCoord,
            &TerrainChunk,
            &ClimateChunk,
            &ResourceChunk,
            &EnergyAvailabilityChunk,
        )>();

        let chunks: Vec<_> = query
            .iter(world)
            .map(|(c, t, cl, r, e)| (*c, t.clone(), cl.clone(), r.clone(), e.clone()))
            .collect();

        let clock = SimulationClock {
            total_ticks: 10,
            tick_duration_hours: 1,
        };
        let id_generator = StableIdGenerator::new();
        let agents = vec![];
        let snapshot = build_world_snapshot(
            &config,
            &seed,
            &clock,
            SNAPSHOT_SCHEMA_VERSION,
            &chunks,
            &id_generator,
            &agents,
        );

        let json = serde_json::to_string(&snapshot).expect("serialize should succeed");
        let deserialized: WorldSnapshot =
            serde_json::from_str(&json).expect("deserialize should succeed");

        assert_eq!(deserialized.schema_version, snapshot.schema_version);
        assert_eq!(deserialized.total_ticks, snapshot.total_ticks);
        assert_eq!(deserialized.seed.root_seed, snapshot.seed.root_seed);
        assert_eq!(deserialized.chunks.len(), snapshot.chunks.len());

        for (a, b) in deserialized.chunks.iter().zip(snapshot.chunks.iter()) {
            assert_eq!(a.coord, b.coord);
            assert_eq!(a.terrain.elevation, b.terrain.elevation);
            assert_eq!(a.climate.temperature, b.climate.temperature);
            assert_eq!(a.resources.fresh_water, b.resources.fresh_water);
            assert_eq!(a.energy.solar_exposure, b.energy.solar_exposure);
        }
    }
}

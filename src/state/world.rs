use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::{Persistable, PersistenceBackend};

/// World state - represents the generated world zones
/// In multiplayer, this will be server-authoritative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    /// Seed for world generation (ensures consistency across clients)
    pub seed: u64,

    /// Generated zones (coordinate -> zone data)
    pub zones: std::collections::HashMap<ZoneCoordinate, ZoneData>,

    /// World metadata
    pub metadata: WorldMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZoneCoordinate {
    pub x: i32,
    pub y: i32,
}

impl ZoneCoordinate {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneData {
    /// Zone coordinate
    pub coord: ZoneCoordinate,

    /// Terrain data (simplified for now, expand as needed)
    pub terrain_seed: u64,

    /// Settlements in this zone
    pub settlements: Vec<SettlementData>,

    /// Points of interest
    pub points_of_interest: Vec<PointOfInterestData>,

    /// When this zone was first generated
    pub generated_at: chrono::DateTime<chrono::Utc>,

    /// Last time any player visited
    pub last_visited: Option<chrono::DateTime<chrono::Utc>>,

    /// Zone state flags
    pub flags: ZoneFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZoneFlags {
    /// Is this zone claimed by a faction
    pub claimed_by: Option<String>,

    /// Is there active conflict here
    pub in_conflict: bool,

    /// Has this been explored by any player
    pub explored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementData {
    pub name: String,
    pub settlement_type: SettlementType,
    pub position: LocalCoordinate,
    pub population: u32,
    pub prosperity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettlementType {
    Outpost,
    Village,
    Town,
    City,
    Capital,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointOfInterestData {
    pub name: String,
    pub poi_type: PoiType,
    pub position: LocalCoordinate,
    pub discovered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoiType {
    Dungeon,
    Ruins,
    Cave,
    Tower,
    Temple,
    Shrine,
    Camp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalCoordinate {
    pub x: i32,
    pub y: i32,
}

impl LocalCoordinate {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    /// When the world was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// World name/identifier
    pub world_name: String,

    /// World description
    pub description: String,

    /// Campaign flags and progress
    pub campaign_flags: std::collections::HashMap<String, bool>,
}

impl WorldState {
    pub fn new(seed: u64, world_name: String) -> Self {
        Self {
            seed,
            zones: std::collections::HashMap::new(),
            metadata: WorldMetadata {
                created_at: chrono::Utc::now(),
                world_name,
                description: String::new(),
                campaign_flags: std::collections::HashMap::new(),
            },
        }
    }

    /// Get or generate a zone
    pub fn get_zone(&self, coord: ZoneCoordinate) -> Option<&ZoneData> {
        self.zones.get(&coord)
    }

    /// Add or update a zone
    pub fn set_zone(&mut self, zone: ZoneData) {
        self.zones.insert(zone.coord, zone);
    }

    /// Check if a zone exists
    pub fn has_zone(&self, coord: ZoneCoordinate) -> bool {
        self.zones.contains_key(&coord)
    }
}

impl Persistable for WorldState {
    fn state_type() -> &'static str {
        "world"
    }

    fn save(&self, backend: &dyn PersistenceBackend) -> Result<()> {
        let key = format!("world_{}", self.metadata.world_name);
        let data = serde_json::to_vec_pretty(self)?;
        backend.save(&key, &data)
    }

    fn load(backend: &dyn PersistenceBackend, id: &str) -> Result<Self> {
        let key = format!("world_{}", id);
        let data = backend.load(&key)?;
        let state = serde_json::from_slice(&data)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_state() {
        let mut world = WorldState::new(12345, "TestWorld".to_string());

        let zone = ZoneData {
            coord: ZoneCoordinate::new(0, 0),
            terrain_seed: 67890,
            settlements: vec![],
            points_of_interest: vec![],
            generated_at: chrono::Utc::now(),
            last_visited: None,
            flags: ZoneFlags::default(),
        };

        world.set_zone(zone);

        assert!(world.has_zone(ZoneCoordinate::new(0, 0)));
        assert!(!world.has_zone(ZoneCoordinate::new(1, 1)));
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::{Persistable, PersistenceBackend};

/// Dungeon state - represents an instance/dungeon exploration
/// In multiplayer, this could be instanced per party or shared
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonState {
    /// Unique identifier for this dungeon instance
    pub instance_id: String,

    /// Reference to where this dungeon is in the world
    pub world_location: DungeonWorldLocation,

    /// Dungeon metadata
    pub metadata: DungeonMetadata,

    /// Current floor layouts
    pub floors: Vec<FloorState>,

    /// Active enemies in the dungeon
    pub enemies: Vec<EnemyInstance>,

    /// Loot that has been generated but not collected
    pub loot: Vec<LootInstance>,

    /// Which players/characters are in this instance
    pub participants: Vec<String>, // Character IDs

    /// Dungeon progress
    pub progress: DungeonProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonWorldLocation {
    pub zone_x: i32,
    pub zone_y: i32,
    pub local_x: i32,
    pub local_y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonMetadata {
    pub name: String,
    pub dungeon_type: DungeonType,
    pub difficulty: u32,
    pub seed: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DungeonType {
    Cave,
    Crypt,
    Ruins,
    Tower,
    Dungeon,
    Temple,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorState {
    pub floor_number: usize,

    /// Tiles that make up this floor
    pub tiles: Vec<Vec<TileState>>,

    /// Width and height
    pub width: i32,
    pub height: i32,

    /// Special features on this floor
    pub features: Vec<FloorFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileState {
    pub tile_type: TileType,
    pub explored: bool,
    pub visible: bool,

    /// Items on this tile
    pub items: Vec<String>, // Item IDs

    /// Special properties
    pub properties: TileProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileType {
    Floor,
    Wall,
    Door,
    LockedDoor,
    Stairs,
    Trap,
    Water,
    Pit,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TileProperties {
    pub is_door_open: bool,
    pub trap_triggered: bool,
    pub secret_revealed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorFeature {
    pub position: Position,
    pub feature_type: FeatureType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureType {
    Shrine,
    Fountain,
    Chest,
    Altar,
    Portal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyInstance {
    pub id: String,
    pub enemy_type: String,
    pub position: Position,
    pub floor: usize,
    pub health: i32,
    pub max_health: i32,
    pub status_effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootInstance {
    pub id: String,
    pub item_type: String,
    pub position: Position,
    pub floor: usize,
    pub claimed_by: Option<String>, // Character ID if claimed
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DungeonProgress {
    /// Enemies defeated
    pub enemies_defeated: u32,

    /// Rooms explored
    pub rooms_explored: u32,

    /// Treasure found
    pub treasure_found: u32,

    /// Boss defeated
    pub boss_defeated: bool,

    /// Time spent (in turns)
    pub turns_elapsed: u64,
}

impl DungeonState {
    pub fn new(
        instance_id: String,
        world_location: DungeonWorldLocation,
        metadata: DungeonMetadata,
    ) -> Self {
        Self {
            instance_id,
            world_location,
            metadata,
            floors: vec![],
            enemies: vec![],
            loot: vec![],
            participants: vec![],
            progress: DungeonProgress::default(),
        }
    }

    /// Get a specific floor
    pub fn get_floor(&self, floor_number: usize) -> Option<&FloorState> {
        self.floors.get(floor_number)
    }

    /// Get mutable floor
    pub fn get_floor_mut(&mut self, floor_number: usize) -> Option<&mut FloorState> {
        self.floors.get_mut(floor_number)
    }

    /// Add a participant to this dungeon instance
    pub fn add_participant(&mut self, character_id: String) {
        if !self.participants.contains(&character_id) {
            self.participants.push(character_id);
        }
    }

    /// Remove a participant
    pub fn remove_participant(&mut self, character_id: &str) {
        self.participants.retain(|id| id != character_id);
    }

    /// Check if instance is empty (no participants)
    pub fn is_empty(&self) -> bool {
        self.participants.is_empty()
    }
}

impl Persistable for DungeonState {
    fn state_type() -> &'static str {
        "dungeon"
    }

    fn save(&self, backend: &dyn PersistenceBackend) -> Result<()> {
        let key = format!("dungeon_{}", self.instance_id);
        let data = serde_json::to_vec_pretty(self)?;
        backend.save(&key, &data)
    }

    fn load(backend: &dyn PersistenceBackend, id: &str) -> Result<Self> {
        let key = format!("dungeon_{}", id);
        let data = backend.load(&key)?;
        let state = serde_json::from_slice(&data)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_state() {
        let location = DungeonWorldLocation {
            zone_x: 0,
            zone_y: 0,
            local_x: 10,
            local_y: 10,
        };

        let metadata = DungeonMetadata {
            name: "Test Cave".to_string(),
            dungeon_type: DungeonType::Cave,
            difficulty: 5,
            seed: 12345,
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
        };

        let mut dungeon = DungeonState::new("test_instance".to_string(), location, metadata);

        dungeon.add_participant("player1".to_string());
        assert_eq!(dungeon.participants.len(), 1);
        assert!(!dungeon.is_empty());

        dungeon.remove_participant("player1");
        assert!(dungeon.is_empty());
    }
}

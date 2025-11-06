use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::forge::ForgeCharacter;
use super::{Persistable, PersistenceBackend};

/// Character state that will be synchronized in multiplayer
/// Wraps ForgeCharacter with additional multiplayer-specific state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterState {
    /// The core character data from Forge
    /// (already includes position via current_zone and current_position)
    pub character: ForgeCharacter,

    /// Current dungeon position (if in dungeon)
    pub dungeon_position: Option<DungeonPosition>,

    /// Character-specific game progress
    pub progress: CharacterProgress,

    /// Session data (not persisted long-term)
    #[serde(skip)]
    pub session: SessionData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonPosition {
    /// Reference to the dungeon (POI coordinates)
    pub dungeon_zone_x: i32,
    pub dungeon_zone_y: i32,
    pub dungeon_local_x: i32,
    pub dungeon_local_y: i32,

    /// Current position in dungeon
    pub floor: usize,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterProgress {
    /// Completed quests
    pub completed_quests: Vec<String>,

    /// Active quests
    pub active_quests: Vec<String>,

    /// Discovered locations
    pub discovered_locations: Vec<(i32, i32)>, // Zone coordinates

    /// Explored world tiles - tracks which tiles have been seen
    /// Key: (zone_x, zone_y), Value: HashSet of (local_x, local_y)
    pub explored_tiles: std::collections::HashMap<(i32, i32), std::collections::HashSet<(i32, i32)>>,

    /// Faction relationships
    pub faction_relationships: std::collections::HashMap<String, i32>,

    /// Game time (turns elapsed)
    pub game_time: u64,

    /// Experience earned
    pub experience: u32,
}

#[derive(Debug, Clone, Default)]
pub struct SessionData {
    /// Last save time
    pub last_save: Option<std::time::SystemTime>,

    /// Client connection info (for multiplayer)
    pub client_id: Option<String>,
}

impl CharacterState {
    /// Create a new character state from a Forge character
    pub fn new(character: ForgeCharacter) -> Self {
        Self {
            character,
            dungeon_position: None,
            progress: CharacterProgress::default(),
            session: SessionData::default(),
        }
    }

    /// Get the character's unique ID
    pub fn id(&self) -> String {
        self.character.name.clone()
    }

    /// Check if character is in a dungeon
    pub fn is_in_dungeon(&self) -> bool {
        self.dungeon_position.is_some()
    }

    /// Get vision radius (accounts for racial abilities and equipment)
    pub fn get_vision_radius(&self) -> u8 {
        self.character.get_vision_radius()
    }

    /// Mark a tile as explored
    pub fn mark_tile_explored(&mut self, zone_x: i32, zone_y: i32, local_x: i32, local_y: i32) {
        self.progress.explored_tiles
            .entry((zone_x, zone_y))
            .or_insert_with(std::collections::HashSet::new)
            .insert((local_x, local_y));
    }

    /// Check if a tile has been explored
    pub fn is_tile_explored(&self, zone_x: i32, zone_y: i32, local_x: i32, local_y: i32) -> bool {
        self.progress.explored_tiles
            .get(&(zone_x, zone_y))
            .map(|tiles| tiles.contains(&(local_x, local_y)))
            .unwrap_or(false)
    }

    /// Reveal tiles within vision radius around a position
    pub fn reveal_around_position(&mut self, zone_x: i32, zone_y: i32, local_x: i32, local_y: i32, vision_radius: i32) {
        let zone_size = 64; // ZONE_SIZE constant

        for dy in -vision_radius..=vision_radius {
            for dx in -vision_radius..=vision_radius {
                // Calculate distance for circular vision
                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                if distance <= vision_radius as f32 {
                    let tile_x = local_x + dx;
                    let tile_y = local_y + dy;

                    // Handle tiles that cross zone boundaries
                    let (actual_zone_x, actual_zone_y, actual_local_x, actual_local_y) =
                        if tile_x < 0 || tile_x >= zone_size || tile_y < 0 || tile_y >= zone_size {
                            let zone_offset_x = if tile_x < 0 { -1 } else if tile_x >= zone_size { 1 } else { 0 };
                            let zone_offset_y = if tile_y < 0 { -1 } else if tile_y >= zone_size { 1 } else { 0 };

                            let new_local_x = if tile_x < 0 {
                                zone_size + tile_x
                            } else if tile_x >= zone_size {
                                tile_x - zone_size
                            } else {
                                tile_x
                            };

                            let new_local_y = if tile_y < 0 {
                                zone_size + tile_y
                            } else if tile_y >= zone_size {
                                tile_y - zone_size
                            } else {
                                tile_y
                            };

                            (zone_x + zone_offset_x, zone_y + zone_offset_y, new_local_x, new_local_y)
                        } else {
                            (zone_x, zone_y, tile_x, tile_y)
                        };

                    self.mark_tile_explored(actual_zone_x, actual_zone_y, actual_local_x, actual_local_y);
                }
            }
        }
    }
}

impl Persistable for CharacterState {
    fn state_type() -> &'static str {
        "character"
    }

    fn save(&self, backend: &dyn PersistenceBackend) -> Result<()> {
        let key = format!("character_{}", self.id());
        let data = serde_json::to_vec_pretty(self)?;
        backend.save(&key, &data)
    }

    fn load(backend: &dyn PersistenceBackend, id: &str) -> Result<Self> {
        let key = format!("character_{}", id);
        let data = backend.load(&key)?;
        let state = serde_json::from_slice(&data)?;
        Ok(state)
    }
}

/// Helper to convert from ForgeCharacter to CharacterState
impl From<ForgeCharacter> for CharacterState {
    fn from(character: ForgeCharacter) -> Self {
        Self::new(character)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_state_creation() {
        // We'll skip detailed testing here since ForgeCharacter
        // has complex initialization. Integration tests will cover this.

        // Test basic struct creation
        let progress = CharacterProgress::default();
        assert_eq!(progress.completed_quests.len(), 0);
        assert_eq!(progress.game_time, 0);
    }

    #[test]
    fn test_dungeon_position() {
        let pos = DungeonPosition {
            dungeon_zone_x: 0,
            dungeon_zone_y: 0,
            dungeon_local_x: 10,
            dungeon_local_y: 10,
            floor: 1,
            x: 5,
            y: 5,
        };

        assert_eq!(pos.floor, 1);
        assert_eq!(pos.x, 5);
    }
}

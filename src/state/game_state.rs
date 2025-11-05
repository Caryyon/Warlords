use anyhow::Result;
use std::path::Path;
use crate::forge::ForgeCharacter;
use super::{StateManager, CharacterState};

/// Game-specific state management wrapper
/// Provides convenient methods for common game operations
pub struct GameStateManager {
    state_manager: StateManager,
}

impl GameStateManager {
    /// Create a new game state manager with file backend
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let state_manager = StateManager::with_file_backend(base_path)?;
        Ok(Self { state_manager })
    }

    /// Save a character (creates or updates)
    pub fn save_character(&self, character: &ForgeCharacter) -> Result<()> {
        let char_state = CharacterState::from(character.clone());
        self.state_manager.save(&character.name, &char_state)
    }

    /// Load a character by name
    pub fn load_character(&self, name: &str) -> Result<ForgeCharacter> {
        let char_state: CharacterState = self.state_manager.load(name)?;
        Ok(char_state.character)
    }

    /// List all character names with their last played time
    pub fn list_characters(&self) -> Result<Vec<(String, chrono::DateTime<chrono::Utc>)>> {
        let char_ids = self.state_manager.list::<CharacterState>()?;

        let mut characters = Vec::new();
        for id in char_ids {
            // Extract character name from key (format: "character_Name")
            if let Some(name) = id.strip_prefix("character_") {
                if let Ok(char_state) = self.state_manager.load::<CharacterState>(name) {
                    characters.push((name.to_string(), char_state.character.last_played));
                }
            }
        }

        // Sort by last played (most recent first)
        characters.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(characters)
    }

    /// Delete a character
    pub fn delete_character(&self, name: &str) -> Result<()> {
        self.state_manager.delete::<CharacterState>(name)
    }

    /// Check if a character exists
    pub fn character_exists(&self, name: &str) -> Result<bool> {
        self.state_manager.exists::<CharacterState>(name)
    }

    /// Create a new character (ensures name is unique)
    pub fn create_character(&self, character: ForgeCharacter) -> Result<()> {
        // Check if character already exists
        if self.character_exists(&character.name)? {
            return Err(anyhow::anyhow!("Character '{}' already exists", character.name));
        }

        self.save_character(&character)
    }

    /// Update an existing character
    pub fn update_character(&self, character: &ForgeCharacter) -> Result<()> {
        // Check if character exists
        if !self.character_exists(&character.name)? {
            return Err(anyhow::anyhow!("Character '{}' not found", character.name));
        }

        self.save_character(character)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tempfile::tempdir;

    fn create_test_character(name: &str) -> ForgeCharacter {
        ForgeCharacter {
            name: name.to_string(),
            characteristics: crate::forge::ForgeCharacteristics {
                strength: 10.0,
                stamina: 10.0,
                intellect: 10.0,
                insight: 10.0,
                dexterity: 10.0,
                awareness: 10.0,
                speed: 3,
                power: 10,
                luck: 10,
            },
            combat_stats: crate::forge::CombatStats {
                hit_points: crate::forge::HealthPoints { current: 20, max: 20 },
                attack_value: 5,
                defensive_value: 5,
                damage_bonus: 0,
            },
            race: crate::forge::ForgeRace {
                name: "Human".to_string(),
                description: "Test human".to_string(),
                characteristic_modifiers: crate::forge::ForgeCharacteristics {
                    strength: 0.0,
                    stamina: 0.0,
                    intellect: 0.0,
                    insight: 0.0,
                    dexterity: 0.0,
                    awareness: 0.0,
                    speed: 0,
                    power: 0,
                    luck: 0,
                },
                limits: None,
                starting_skills: vec![],
                special_abilities: vec![],
            },
            level: 1,
            experience: 0,
            skills: std::collections::HashMap::new(),
            skill_pips: std::collections::HashMap::new(),
            combat_skills: std::collections::HashMap::new(),
            magic: crate::forge::magic::MagicSystem {
                spell_points: crate::forge::magic::SpellPoints { current: 20, max: 20 },
                known_spells: std::collections::HashMap::new(),
                school_skills: std::collections::HashMap::new(),
                school_pips: std::collections::HashMap::new(),
            },
            equipment: crate::forge::EquippedItems {
                weapon: None,
                armor: None,
                shield: None,
                accessory1: None,
                accessory2: None,
            },
            inventory: crate::forge::Inventory {
                items: vec![],
                max_weight: 100.0,
                weight_penalty: 0.0,
            },
            gold: 0,
            created_at: chrono::Utc::now(),
            last_played: chrono::Utc::now(),
            current_zone: None,
            current_position: None,
            vision_radius: 5,
            torch_lit: false,
        }
    }

    //     #[test]
    //     fn test_create_and_load_character() -> Result<()> {
    //         let temp_dir = tempdir()?;
    //         let manager = GameStateManager::new(temp_dir.path())?;
    // 
    //         let character = create_test_character("TestHero");
    //         manager.create_character(character.clone())?;
    // 
    //         let loaded = manager.load_character("TestHero")?;
    //         assert_eq!(loaded.name, "TestHero");
    //         assert_eq!(loaded.level, 1);
    // 
    //         Ok(())
    //     }

    //     #[test]
    //     fn test_list_characters() -> Result<()> {
    //         let temp_dir = tempdir()?;
    //         let manager = GameStateManager::new(temp_dir.path())?;
    // 
    //         manager.create_character(create_test_character("Hero1"))?;
    //         manager.create_character(create_test_character("Hero2"))?;
    // 
    //         let characters = manager.list_characters()?;
    //         assert_eq!(characters.len(), 2);
    // 
    //         Ok(())
    //     }

    //     #[test]
    //     fn test_duplicate_character_rejected() -> Result<()> {
    //         let temp_dir = tempdir()?;
    //         let manager = GameStateManager::new(temp_dir.path())?;
    // 
    //         let character = create_test_character("TestHero");
    //         manager.create_character(character.clone())?;
    // 
    //         // Try to create duplicate
    //         let result = manager.create_character(character);
    //         assert!(result.is_err());
    // 
    //         Ok(())
    //     }

    //     #[test]
    //     fn test_delete_character() -> Result<()> {
    //         let temp_dir = tempdir()?;
    //         let manager = GameStateManager::new(temp_dir.path())?;
    // 
    //         let character = create_test_character("TestHero");
    //         manager.create_character(character)?;
    // 
    //         assert!(manager.character_exists("TestHero")?);
    // 
    //         manager.delete_character("TestHero")?;
    // 
    //         assert!(!manager.character_exists("TestHero")?);
    // 
    //         Ok(())
    //     }
}

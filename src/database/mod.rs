use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::forge::ForgeCharacter;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRecord {
    pub character: ForgeCharacter,
    pub password_hash: String,
    pub salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDatabase {
    pub characters: HashMap<String, CharacterRecord>,
}

impl CharacterDatabase {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
        }
    }

    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            let data = fs::read_to_string(path)?;
            
            // Try to load with current format first
            match serde_json::from_str::<CharacterDatabase>(&data) {
                Ok(db) => Ok(db),
                Err(_) => {
                    // If that fails, try to migrate from old format
                    println!("🔄 Migrating character data to new format with magic system...");
                    
                    // Backup old file
                    let backup_path = path.with_extension("json.backup");
                    fs::copy(path, &backup_path)?;
                    println!("📁 Backed up old data to: {}", backup_path.display());
                    
                    // Try to load as old format and migrate
                    let migrated_db = Self::migrate_from_old_format(&data)?;
                    
                    // Save migrated data
                    migrated_db.save(path)?;
                    println!("✅ Migration complete!");
                    
                    Ok(migrated_db)
                }
            }
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn create_character(&mut self, name: String, password: String, character: ForgeCharacter) -> Result<()> {
        if self.characters.contains_key(&name) {
            return Err(anyhow!("Character with name '{}' already exists", name));
        }

        let salt = format!("{:x}", rand::random::<u64>());
        let password_hash = self.hash_password(&password, &salt);

        let record = CharacterRecord {
            character,
            password_hash,
            salt,
        };

        self.characters.insert(name, record);
        Ok(())
    }

    pub fn authenticate(&self, name: &str, password: &str) -> Result<ForgeCharacter> {
        let record = self.characters.get(name)
            .ok_or_else(|| anyhow!("Character '{}' not found", name))?;

        let expected_hash = self.hash_password(password, &record.salt);
        if expected_hash != record.password_hash {
            return Err(anyhow!("Invalid password"));
        }

        Ok(record.character.clone())
    }

    pub fn update_character(&mut self, name: &str, character: ForgeCharacter) -> Result<()> {
        let record = self.characters.get_mut(name)
            .ok_or_else(|| anyhow!("Character '{}' not found", name))?;

        record.character = character;
        Ok(())
    }

    pub fn list_characters(&self) -> Vec<(String, chrono::DateTime<chrono::Utc>)> {
        self.characters.iter()
            .map(|(name, record)| (name.clone(), record.character.last_played))
            .collect()
    }

    pub fn delete_character(&mut self, name: &str) -> Result<()> {
        self.characters.remove(name)
            .ok_or_else(|| anyhow!("Character '{}' not found", name))?;
        Ok(())
    }

    fn hash_password(&self, password: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    fn migrate_from_old_format(data: &str) -> Result<Self> {
        use serde_json::Value;
        
        // Parse as generic JSON first
        let mut json: Value = serde_json::from_str(data)?;
        
        // Add magic and vision fields to each character if missing
        if let Some(characters) = json.get_mut("characters").and_then(|c| c.as_object_mut()) {
            for (name, record) in characters.iter_mut() {
                if let Some(character) = record.get_mut("character").and_then(|c| c.as_object_mut()) {
                    let mut needs_migration = false;
                    
                    // Add magic system if missing
                    if !character.contains_key("magic") {
                        println!("🧙 Adding magic system to character: {}", name);
                        needs_migration = true;
                        
                        // Get power characteristic for spell points calculation
                        let power = character.get("characteristics")
                            .and_then(|c| c.get("power"))
                            .and_then(|p| p.as_u64())
                            .unwrap_or(10) as u8;
                        
                        // Create magic system
                        let magic_system = serde_json::json!({
                            "spell_points": {
                                "current": power as u32 * 2,
                                "max": power as u32 * 2
                            },
                            "known_spells": {},
                            "school_skills": {},
                            "school_pips": {}
                        });
                        
                        character.insert("magic".to_string(), magic_system);
                    }
                    
                    // Add vision system fields if missing
                    if !character.contains_key("vision_radius") {
                        println!("👁️ Adding vision system to character: {}", name);
                        needs_migration = true;
                        
                        // Calculate proper racial vision radius
                        let vision_radius = if let Some(race) = character.get("race") {
                            if let Some(special_abilities) = race.get("special_abilities") {
                                if let Some(abilities_array) = special_abilities.as_array() {
                                    let mut racial_vision = 2; // Default
                                    for ability in abilities_array {
                                        if let Some(ability_str) = ability.as_str() {
                                            if ability_str.contains("Heat Vision (30')") {
                                                racial_vision = 3; // 30 feet = ~3 tiles
                                                break;
                                            } else if ability_str.contains("Heat Vision (60')") {
                                                racial_vision = 6; // 60 feet = ~6 tiles  
                                                break;
                                            } else if ability_str.contains("Night Vision (90')") {
                                                racial_vision = 9; // 90 feet = ~9 tiles
                                                break;
                                            }
                                        }
                                    }
                                    racial_vision
                                } else {
                                    2 // Default vision
                                }
                            } else {
                                2 // Default vision
                            }
                        } else {
                            2 // Default vision
                        };
                        
                        character.insert("vision_radius".to_string(), serde_json::json!(vision_radius));
                    }
                    if !character.contains_key("torch_lit") {
                        if !needs_migration {
                            println!("🔥 Adding torch system to character: {}", name);
                        }
                        character.insert("torch_lit".to_string(), serde_json::json!(false));
                    }
                }
            }
        }
        
        // Now deserialize the updated JSON
        let migrated_db: CharacterDatabase = serde_json::from_value(json)?;
        Ok(migrated_db)
    }
}
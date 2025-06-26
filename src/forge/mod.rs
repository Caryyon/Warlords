use serde::{Deserialize, Serialize};
use rand::Rng;
use std::collections::HashMap;

pub mod combat;
pub mod magic;
pub use combat::*;
pub use magic::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeCharacteristics {
    pub strength: f32,      // STR - Physical might
    pub stamina: f32,       // STA - Endurance
    pub intellect: f32,     // INT - Intelligence
    pub insight: f32,       // INS - Wisdom
    pub dexterity: f32,     // DEX - Agility
    pub awareness: f32,     // AWR - Perception
    pub speed: u8,          // SPD - Movement rate (1-5)
    pub power: u8,          // POW - Magical ability (2-20)
    pub luck: u8,           // LUC - Fortune (6-16)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatStats {
    pub hit_points: HealthPoints,
    pub attack_value: u8,
    pub defensive_value: u8,
    pub damage_bonus: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthPoints {
    pub current: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeCharacter {
    pub name: String,
    pub characteristics: ForgeCharacteristics,
    pub combat_stats: CombatStats,
    pub race: ForgeRace,
    pub level: u8,
    pub experience: u32,
    pub skills: HashMap<String, u8>,
    pub skill_pips: HashMap<String, u8>, // Accumulated pips for skill advancement
    pub magic: MagicSystem,             // Magic system integration
    pub inventory: Vec<String>,
    pub gold: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_played: chrono::DateTime<chrono::Utc>,
    pub current_zone: Option<crate::world::ZoneCoord>,
    pub current_position: Option<crate::world::LocalCoord>,
    pub vision_radius: u8,              // Base vision radius in tiles
    pub torch_lit: bool,                // Whether a torch is currently lit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeRace {
    pub name: String,
    pub description: String,
    pub characteristic_modifiers: ForgeCharacteristics,
    pub limits: Option<ForgeLimits>,
    pub starting_skills: Vec<(String, u8)>,
    pub special_abilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeLimits {
    pub strength: Option<f32>,
    pub stamina: Option<f32>,
    pub intellect: Option<f32>,
    pub insight: Option<f32>,
    pub dexterity: Option<f32>,
    pub awareness: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRoll {
    pub d6_1: u8,
    pub d6_2: u8,
    pub d10: u8,
    pub total: f32,
    pub formula: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolledCharacteristics {
    pub strength: CharacterRoll,
    pub stamina: CharacterRoll,
    pub intellect: CharacterRoll,
    pub insight: CharacterRoll,
    pub dexterity: CharacterRoll,
    pub awareness: CharacterRoll,
    pub speed: CharacterRoll,
    pub power: CharacterRoll,
    pub luck: CharacterRoll,
}

pub struct ForgeCharacterCreation;

impl ForgeCharacterCreation {
    pub fn roll_characteristics() -> RolledCharacteristics {
        let mut rng = rand::thread_rng();
        
        // Helper function for 2d6 + 1d10 rolls
        let roll_2d6_1d10 = |rng: &mut rand::rngs::ThreadRng| -> CharacterRoll {
            let d6_1 = rng.gen_range(1..=6);
            let d6_2 = rng.gen_range(1..=6);
            let d10_raw = rng.gen_range(0..=9); // 0-9, where 0 = 1.0
            let d10 = if d10_raw == 0 { 10 } else { d10_raw }; // Convert 0 to 10 for display
            let decimal = if d10_raw == 0 { 1.0 } else { d10_raw as f32 / 10.0 };
            
            CharacterRoll {
                d6_1,
                d6_2,
                d10,
                total: (d6_1 + d6_2) as f32 + decimal,
                formula: format!("{}+{}+{}/10", d6_1, d6_2, d10),
            }
        };

        RolledCharacteristics {
            strength: roll_2d6_1d10(&mut rng),
            stamina: roll_2d6_1d10(&mut rng),
            intellect: roll_2d6_1d10(&mut rng),
            insight: roll_2d6_1d10(&mut rng),
            dexterity: roll_2d6_1d10(&mut rng),
            awareness: roll_2d6_1d10(&mut rng),
            speed: {
                let roll = rng.gen_range(1..=4) + 1;
                CharacterRoll {
                    d6_1: 0, d6_2: 0, d10: roll,
                    total: roll as f32,
                    formula: format!("1d4+1 = {}", roll),
                }
            },
            power: {
                let d10_1 = rng.gen_range(1..=10);
                let d10_2 = rng.gen_range(1..=10);
                let total = d10_1 + d10_2;
                CharacterRoll {
                    d6_1: 0, d6_2: 0, d10: total,
                    total: total as f32,
                    formula: format!("{}+{} = {}", d10_1, d10_2, total),
                }
            },
            luck: {
                let d6_1 = rng.gen_range(1..=6);
                let d6_2 = rng.gen_range(1..=6);
                let total = d6_1 + d6_2 + 4;
                CharacterRoll {
                    d6_1, d6_2, d10: total,
                    total: total as f32,
                    formula: format!("{}+{}+4 = {}", d6_1, d6_2, total),
                }
            },
        }
    }

    pub fn get_available_races() -> Vec<ForgeRace> {
        vec![
            // 1. Berserkers
            ForgeRace {
                name: "Berserker".to_string(),
                description: "Large aggressive warriors (300+ lbs) with piercing eyes and braided hair. Fear magic.".to_string(),
                characteristic_modifiers: ForgeCharacteristics {
                    strength: 1.0, stamina: 1.0, intellect: 0.0, insight: 0.0,
                    dexterity: -1.0, awareness: -1.0, speed: 0, power: 0, luck: 0,
                },
                limits: Some(ForgeLimits {
                    strength: Some(13.0), stamina: None, intellect: None,
                    insight: None, dexterity: None, awareness: None,
                }),
                starting_skills: vec![("Melee Combat".to_string(), 1)],
                special_abilities: vec!["+1 Attack Value".to_string(), "Fear of Magic (cannot use)".to_string()],
            },
            // 2. Dunnar
            ForgeRace {
                name: "Dunnar".to_string(),
                description: "Pale, thin beings with undead appearance. Mind protection but photosensitive.".to_string(),
                characteristic_modifiers: ForgeCharacteristics {
                    strength: 0.0, stamina: 0.0, intellect: 0.0, insight: 0.0,
                    dexterity: 0.0, awareness: 0.0, speed: 0, power: 0, luck: 0,
                },
                limits: Some(ForgeLimits {
                    strength: Some(9.0), stamina: None, intellect: None,
                    insight: None, dexterity: None, awareness: None,
                }),
                starting_skills: vec![("Mind Magic".to_string(), 1)],
                special_abilities: vec!["Mind Protection".to_string(), "Detect Magic by Touch".to_string(), "Photosensitive (1 dmg/hour in sun)".to_string()],
            },
            // 3. Dwarves
            ForgeRace {
                name: "Dwarf".to_string(),
                description: "Stout warriors (4' tall) with long beards. Heat vision and disease resistant.".to_string(),
                characteristic_modifiers: ForgeCharacteristics {
                    strength: 0.0, stamina: 0.0, intellect: 0.0, insight: 0.0,
                    dexterity: 0.0, awareness: -1.0, speed: 0, power: 0, luck: 0,
                },
                limits: Some(ForgeLimits {
                    strength: Some(11.5), stamina: None, intellect: None,
                    insight: None, dexterity: None, awareness: None,
                }),
                starting_skills: vec![("Axe".to_string(), 1), ("Smithing".to_string(), 1)],
                special_abilities: vec!["Heat Vision (30')".to_string(), "Sturdy (use medium weapons 1-handed)".to_string()],
            },
            // 4. Elves
            ForgeRace {
                name: "Elf".to_string(),
                description: "Slender, graceful beings with greenish-blue eyes. Magical affinity but slow healing.".to_string(),
                characteristic_modifiers: ForgeCharacteristics {
                    strength: 0.0, stamina: 0.0, intellect: 0.0, insight: 0.0,
                    dexterity: 0.0, awareness: 0.0, speed: 0, power: 0, luck: 0,
                },
                limits: Some(ForgeLimits {
                    strength: Some(9.0), stamina: None, intellect: None,
                    insight: None, dexterity: None, awareness: None,
                }),
                starting_skills: vec![("Magic".to_string(), 1), ("Archery".to_string(), 1)],
                special_abilities: vec!["+25% Magic skill base".to_string(), "+25% Reaction Rolls".to_string(), "Slow Healing (1 HP/24hr)".to_string()],
            },
            // 5. Ghantu
            ForgeRace {
                name: "Ghantu".to_string(),
                description: "Massive one-eyed humanoids (7'+, 400+ lbs) with gorilla-like appearance. Strongest race.".to_string(),
                characteristic_modifiers: ForgeCharacteristics {
                    strength: 2.0, stamina: 0.0, intellect: 0.0, insight: 0.0,
                    dexterity: 0.0, awareness: 0.0, speed: 0, power: 0, luck: 0,
                },
                limits: Some(ForgeLimits {
                    strength: Some(15.5), stamina: None, intellect: None,
                    insight: None, dexterity: None, awareness: None,
                }),
                starting_skills: vec![("Brawling".to_string(), 1)],
                special_abilities: vec!["Natural Armor 1".to_string(), "Claws (1d4 damage)".to_string(), "Single Eye (-3 missile AV)".to_string(), "Learning Disability (2x skill costs)".to_string()],
            },
            // 6. Higmoni
            ForgeRace {
                name: "Higmoni".to_string(),
                description: "Boar-like humanoids with tusks and leathery skin. Fast healing but strong odor.".to_string(),
                characteristic_modifiers: ForgeCharacteristics {
                    strength: 0.0, stamina: 0.0, intellect: 0.0, insight: 0.0,
                    dexterity: -1.0, awareness: 0.0, speed: 0, power: 0, luck: 0,
                },
                limits: Some(ForgeLimits {
                    strength: Some(13.5), stamina: None, intellect: None,
                    insight: None, dexterity: None, awareness: None,
                }),
                starting_skills: vec![("Tracking".to_string(), 1)],
                special_abilities: vec!["Heat Vision (60')".to_string(), "Accelerated Healing (2 HP/night)".to_string(), "Odor (-30% reactions, -10% hiding)".to_string()],
            },
            // 7. Humans
            ForgeRace {
                name: "Human".to_string(),
                description: "Most populous race, versatile with no special abilities or penalties.".to_string(),
                characteristic_modifiers: ForgeCharacteristics {
                    strength: 0.0, stamina: 0.0, intellect: 0.0, insight: 0.0,
                    dexterity: 0.0, awareness: 0.0, speed: 0, power: 0, luck: 0,
                },
                limits: Some(ForgeLimits {
                    strength: Some(11.0), stamina: None, intellect: None,
                    insight: None, dexterity: None, awareness: None,
                }),
                starting_skills: vec![("Choice".to_string(), 1)],
                special_abilities: vec!["None (balanced race)".to_string()],
            },
            // 8. Jher-em
            ForgeRace {
                name: "Jher-em".to_string(),
                description: "Small shrew-like beings (3' tall) with telepathy and spiked tails.".to_string(),
                characteristic_modifiers: ForgeCharacteristics {
                    strength: 0.0, stamina: 0.0, intellect: 0.0, insight: 0.0,
                    dexterity: 0.0, awareness: 0.0, speed: 0, power: 0, luck: 0,
                },
                limits: Some(ForgeLimits {
                    strength: Some(9.0), stamina: None, intellect: None,
                    insight: None, dexterity: None, awareness: None,
                }),
                starting_skills: vec![("Tracking".to_string(), 1)],
                special_abilities: vec!["Telepathy (30')".to_string(), "Heightened Smell (25% tracking)".to_string(), "Spiked Tail (1d3)".to_string(), "Misshapen (-1 AV)".to_string()],
            },
            // 9. Kithsara
            ForgeRace {
                name: "Kithsara".to_string(),
                description: "Lizard-like humanoids with green scales and fangs. Natural armor but thin blood.".to_string(),
                characteristic_modifiers: ForgeCharacteristics {
                    strength: 0.0, stamina: -1.0, intellect: 0.0, insight: 0.0,
                    dexterity: 0.0, awareness: 0.0, speed: 0, power: 3, luck: 0,
                },
                limits: Some(ForgeLimits {
                    strength: Some(11.0), stamina: None, intellect: None,
                    insight: None, dexterity: None, awareness: None,
                }),
                starting_skills: vec![("Nature Magic".to_string(), 1)],
                special_abilities: vec!["Natural Armor 2".to_string(), "Fangs (1d3 bite)".to_string(), "Thin Blood (lose 2 HP/min when negative)".to_string()],
            },
            // 10. Merikii
            ForgeRace {
                name: "Merikii".to_string(),
                description: "Bird-like beings with golden feathers and beaks. Can dual-wield but fragile.".to_string(),
                characteristic_modifiers: ForgeCharacteristics {
                    strength: -1.0, stamina: 0.0, intellect: 0.0, insight: 0.0,
                    dexterity: 0.0, awareness: 0.0, speed: 0, power: 0, luck: 0,
                },
                limits: Some(ForgeLimits {
                    strength: Some(9.0), stamina: None, intellect: None,
                    insight: None, dexterity: None, awareness: None,
                }),
                starting_skills: vec![("Animal Handling".to_string(), 1)],
                special_abilities: vec!["Night Vision (90')".to_string(), "Two-Handed Melee (small weapons)".to_string(), "Thin Blood (lose 2 HP/min when negative)".to_string()],
            },
            // 11. Sprites
            ForgeRace {
                name: "Sprite".to_string(),
                description: "Tiny humanoids (3' tall) with pointed ears. Empathic but physically weak.".to_string(),
                characteristic_modifiers: ForgeCharacteristics {
                    strength: -1.0, stamina: 0.0, intellect: 0.0, insight: 0.0,
                    dexterity: 0.0, awareness: 0.0, speed: 0, power: 0, luck: 0,
                },
                limits: Some(ForgeLimits {
                    strength: Some(7.0), stamina: None, intellect: None,
                    insight: None, dexterity: None, awareness: None,
                }),
                starting_skills: vec![("Herbalism".to_string(), 1)],
                special_abilities: vec!["Empathy (30' range)".to_string(), "Small Size".to_string()],
            },
        ]
    }

    pub fn apply_racial_modifiers(
        rolled: &RolledCharacteristics, 
        race: &ForgeRace
    ) -> ForgeCharacteristics {
        ForgeCharacteristics {
            strength: (rolled.strength.total + race.characteristic_modifiers.strength).max(1.0),
            stamina: (rolled.stamina.total + race.characteristic_modifiers.stamina).max(1.0),
            intellect: (rolled.intellect.total + race.characteristic_modifiers.intellect).max(1.0),
            insight: (rolled.insight.total + race.characteristic_modifiers.insight).max(1.0),
            dexterity: (rolled.dexterity.total + race.characteristic_modifiers.dexterity).max(1.0),
            awareness: (rolled.awareness.total + race.characteristic_modifiers.awareness).max(1.0),
            speed: (rolled.speed.total as u8 + race.characteristic_modifiers.speed).max(1),
            power: (rolled.power.total as u8 + race.characteristic_modifiers.power).max(1),
            luck: (rolled.luck.total as u8).saturating_add(race.characteristic_modifiers.luck).max(1),
        }
    }

    pub fn create_character(
        name: String,
        characteristics: ForgeCharacteristics,
        race: ForgeRace,
    ) -> ForgeCharacter {
        let hit_points = Self::calculate_hit_points(&characteristics);
        let combat_stats = Self::calculate_combat_stats(&characteristics, hit_points);
        
        let mut character = ForgeCharacter {
            name,
            characteristics: characteristics.clone(),
            combat_stats,
            race: race.clone(),
            level: 1,
            experience: 0,
            skills: race.starting_skills.clone().into_iter().collect(),
            skill_pips: HashMap::new(), // Start with no pips
            magic: {
                let mut magic_system = MagicSystem::new(characteristics.power);
                // Give new characters a starter spell based on their race's affinity
                let starter_spell = Self::get_racial_starter_spell(&race);
                if let Some((spell_name, school)) = starter_spell {
                    magic_system.add_known_spell(spell_name, school);
                }
                magic_system
            },
            inventory: vec!["Farm clothes".to_string(), "Simple tools".to_string(), "Torch (5)".to_string()],
            gold: 10,
            created_at: chrono::Utc::now(),
            last_played: chrono::Utc::now(),
            current_zone: Some(crate::world::ZoneCoord::new(4, 4)), // Center zone (256/64 = 4)
            current_position: Some(crate::world::LocalCoord::new(32, 32)), // Center of zone
            vision_radius: 2, // Will be set by racial abilities
            torch_lit: false,
        };
        
        // Set racial vision radius
        character.vision_radius = character.get_racial_vision_radius();
        character
    }

    fn calculate_hit_points(characteristics: &ForgeCharacteristics) -> u32 {
        // Hit Points = (STR + STA) / 2, minimum 1
        ((characteristics.strength + characteristics.stamina) / 2.0).max(1.0) as u32
    }

    fn calculate_combat_stats(characteristics: &ForgeCharacteristics, hit_points: u32) -> CombatStats {
        CombatStats {
            hit_points: HealthPoints {
                current: hit_points,
                max: hit_points,
            },
            attack_value: (characteristics.dexterity + characteristics.strength / 2.0) as u8,
            defensive_value: (characteristics.dexterity + characteristics.awareness / 2.0) as u8,
            damage_bonus: ((characteristics.strength - 10.0) / 3.0) as i8,
        }
    }
    
    fn get_racial_starter_spell(race: &ForgeRace) -> Option<(String, MagicSchool)> {
        match race.name.as_str() {
            "Human" => Some(("Heal Wounds".to_string(), MagicSchool::Divine)),
            "Elf" => Some(("Animal Communication".to_string(), MagicSchool::Beast)),
            "Dwarf" => Some(("Weapon Blessing".to_string(), MagicSchool::Enchantment)),
            "Halfling" => Some(("Heal Wounds".to_string(), MagicSchool::Divine)),
            "Orc" => Some(("Fire Bolt".to_string(), MagicSchool::Elemental)),
            "Goblin" => Some(("Weaken".to_string(), MagicSchool::Necromancer)),
            "Ogre" => Some(("Bear Strength".to_string(), MagicSchool::Beast)),
            "Troll" => Some(("Lightning Strike".to_string(), MagicSchool::Elemental)),
            "Giant" => Some(("Shield of Faith".to_string(), MagicSchool::Enchantment)),
            "Gnoll" => Some(("Animal Communication".to_string(), MagicSchool::Beast)),
            "Merikii" => Some(("Drain Life".to_string(), MagicSchool::Necromancer)),
            "Sprite" => Some(("Fire Bolt".to_string(), MagicSchool::Elemental)),
            _ => None, // Fallback for unknown races
        }
    }
}

impl ForgeCharacter {
    pub fn get_display_info(&self) -> Vec<String> {
        vec![
            format!("Name: {}", self.name),
            format!("Race: {} (Level {})", self.race.name, self.level),
            format!("Experience: {}", self.experience),
            "".to_string(),
            "=== CHARACTERISTICS ===".to_string(),
            format!("Strength:    {:.1}", self.characteristics.strength),
            format!("Stamina:     {:.1}", self.characteristics.stamina),
            format!("Intellect:   {:.1}", self.characteristics.intellect),
            format!("Insight:     {:.1}", self.characteristics.insight),
            format!("Dexterity:   {:.1}", self.characteristics.dexterity),
            format!("Awareness:   {:.1}", self.characteristics.awareness),
            format!("Speed:       {}", self.characteristics.speed),
            format!("Power:       {}", self.characteristics.power),
            format!("Luck:        {}", self.characteristics.luck),
            "".to_string(),
            "=== COMBAT STATS ===".to_string(),
            format!("Hit Points:  {}/{}", self.combat_stats.hit_points.current, self.combat_stats.hit_points.max),
            format!("Attack Val:  {}", self.combat_stats.attack_value),
            format!("Defense Val: {}", self.combat_stats.defensive_value),
            format!("Dmg Bonus:   {:+}", self.combat_stats.damage_bonus),
            "".to_string(),
            "=== MAGIC ===".to_string(),
            format!("Spell Points: {}/{}", self.magic.spell_points.current, self.magic.spell_points.max),
            format!("Known Spells: {}", self.magic.get_all_known_spells().len()),
            "".to_string(),
            format!("Gold: {}", self.gold),
        ]
    }

    pub fn update_last_played(&mut self) {
        self.last_played = chrono::Utc::now();
    }
    
    pub fn get_vision_radius(&self) -> u8 {
        let mut vision = self.vision_radius;
        
        // Apply torch bonus if lit and better than racial vision
        if self.torch_lit {
            let torch_vision = 4; // Torch provides 4 tile radius
            vision = vision.max(torch_vision);
        }
        
        vision
    }
    
    pub fn can_light_torch(&self) -> bool {
        !self.torch_lit && self.inventory.iter().any(|item| item.contains("Torch"))
    }
    
    pub fn light_torch(&mut self) -> bool {
        if self.can_light_torch() {
            self.torch_lit = true;
            // Consume a torch
            if let Some(pos) = self.inventory.iter().position(|item| item.contains("Torch")) {
                let item = self.inventory[pos].clone();
                if item == "Torch (5)" {
                    self.inventory[pos] = "Torch (4)".to_string();
                } else if item == "Torch (4)" {
                    self.inventory[pos] = "Torch (3)".to_string();
                } else if item == "Torch (3)" {
                    self.inventory[pos] = "Torch (2)".to_string();
                } else if item == "Torch (2)" {
                    self.inventory[pos] = "Torch (1)".to_string();
                } else if item == "Torch (1)" {
                    self.inventory.remove(pos);
                }
            }
            true
        } else {
            false
        }
    }
    
    pub fn extinguish_torch(&mut self) {
        self.torch_lit = false;
    }
    
    fn get_racial_vision_radius(&self) -> u8 {
        // Extract vision radius from race special abilities
        for ability in &self.race.special_abilities {
            if ability.contains("Heat Vision (30')") {
                return 3; // 30 feet = ~3 tiles
            } else if ability.contains("Heat Vision (60')") {
                return 6; // 60 feet = ~6 tiles  
            } else if ability.contains("Night Vision (90')") {
                return 9; // 90 feet = ~9 tiles
            }
        }
        2 // Default human vision radius in dungeons
    }
}
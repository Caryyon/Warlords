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
pub struct EquippedItems {
    pub weapon: Option<Weapon>,
    pub armor: Option<Armor>,
    pub shield: Option<Armor>,
    pub accessory1: Option<Accessory>,
    pub accessory2: Option<Accessory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accessory {
    pub name: String,
    pub accessory_type: AccessoryType,
    pub stat_bonuses: StatBonuses,
    pub magical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessoryType {
    Ring,
    Amulet,
    Belt,
    Cloak,
    Boots,
    Gloves,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_type: ItemType,
    pub name: String,
    pub weight: f32,        // Weight in pounds
    pub stack_size: u32,    // Maximum stack size (1 for unique items)
    pub quantity: u32,      // Current quantity in stack
    pub value: u32,         // Base value in gold pieces
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemType {
    Weapon(Weapon),
    Armor(Armor),
    Accessory(Accessory),
    Consumable(ConsumableItem),
    Material(MaterialItem),
    Misc(MiscItem),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableItem {
    pub consumable_type: ConsumableType,
    pub effect: String,
    pub duration: Option<u32>, // Duration in turns, None for instant
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumableType {
    HealthPotion,
    ManaPotion,
    Food,
    Torch,
    Scroll,
    Reagent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialItem {
    pub material_type: MaterialType,
    pub quality: Quality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaterialType {
    Metal,
    Leather,
    Cloth,
    Wood,
    Stone,
    Gem,
    Bone,
    Herb,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Quality {
    Poor,
    Common,
    Good,
    Excellent,
    Masterwork,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiscItem {
    pub misc_type: MiscType,
    pub special_properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MiscType {
    Tool,
    Book,
    Key,
    Container,
    Art,
    Trade,
    Quest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<InventoryItem>,
    pub max_weight: f32,        // Maximum carrying capacity
    pub weight_penalty: f32,    // Current weight penalty (0.0 - 1.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatBonuses {
    pub attack_bonus: i8,
    pub defense_bonus: i8,
    pub damage_bonus: i8,
    pub characteristic_bonuses: HashMap<String, i8>, // STR, DEX, etc.
    pub skill_bonuses: HashMap<String, i8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeCharacter {
    pub name: String,
    pub characteristics: ForgeCharacteristics,
    pub combat_stats: CombatStats,
    pub race: ForgeRace,
    pub level: u8,
    pub experience: u32,
    pub skills: HashMap<String, u8>,  // Skill percentages (0-100) or combat skill levels
    pub skill_pips: HashMap<String, u8>, // Accumulated pips for skill advancement
    pub combat_skills: HashMap<String, (u8, u8)>, // Combat skills: (level, percentage)
    pub magic: MagicSystem,             // Magic system integration
    pub equipment: EquippedItems,       // Currently equipped items
    pub inventory: Inventory,           // Player's inventory
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

#[derive(Debug, Clone, PartialEq)]
pub enum SkillType {
    Percentage,  // Roll >= current skill % to advance
    Combat,      // Roll <= current skill % to advance
    Magic,       // Magic school skills
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkillCategory {
    Basic,       // Basic skills with simple base calculations
    Percentile,  // Percentile skills that can advance beyond 100%
    Combat,      // Combat skills with level/percentage system
    Magic,       // Magic school skills
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkillBase {
    StrengthX2,
    StaminaX2,
    IntellectX2,
    InsightX2,
    DexterityX2,
    AwarenessX2,
    StrengthPlusDexterity,
    DexterityPlusAwareness,
    AwarenessPlusIntellect,
    StrengthPlusInsight,
    PowerBased,
}

#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub name: String,
    pub category: SkillCategory,
    pub base: SkillBase,
}

impl SkillDefinition {
    pub fn new(name: &str, category: SkillCategory, base: SkillBase) -> Self {
        Self {
            name: name.to_string(),
            category,
            base,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillAdvancementResult {
    pub skill_name: String,
    pub old_value: u8,
    pub new_value: u8,
    pub pips_used: u8,
    pub rolls: Vec<(u8, bool)>, // (roll, succeeded)
    pub leveled_up: bool,       // For combat skills that reached 100%
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
            skills: HashMap::new(), // Will be populated below
            skill_pips: HashMap::new(), // Start with no pips
            combat_skills: HashMap::new(), // Will be populated below
            magic: {
                let mut magic_system = MagicSystem::new(characteristics.power);
                // Give new characters a starter spell based on their race's affinity
                let starter_spell = Self::get_racial_starter_spell(&race);
                if let Some((spell_name, school)) = starter_spell {
                    magic_system.add_known_spell(spell_name, school);
                }
                magic_system
            },
            equipment: EquippedItems::new(),
            inventory: Inventory::new_starting_inventory(&characteristics),
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
        
        // Initialize starting skills based on type
        for (skill_name, skill_value) in &race.starting_skills {
            match ForgeCharacter::get_skill_type(skill_name) {
                SkillType::Combat => {
                    // Combat skills start at level 0 with the given percentage
                    character.combat_skills.insert(skill_name.clone(), (0, *skill_value));
                }
                SkillType::Percentage => {
                    // Regular skills use the percentage directly
                    character.skills.insert(skill_name.clone(), *skill_value);
                }
                SkillType::Magic => {
                    // Magic skills are handled by the magic system
                }
            }
        }
        
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
        !self.torch_lit && self.inventory.items.iter().any(|item| {
            item.name == "Torch" && matches!(item.item_type, ItemType::Consumable(_))
        })
    }
    
    pub fn light_torch(&mut self) -> bool {
        if self.can_light_torch() {
            self.torch_lit = true;
            // Consume a torch
            if let Some(pos) = self.inventory.items.iter().position(|item| {
                item.name == "Torch" && matches!(item.item_type, ItemType::Consumable(_))
            }) {
                let item = self.inventory.items[pos].clone();
                if item.quantity > 1 {
                    self.inventory.items[pos].quantity -= 1;
                } else {
                    self.inventory.items.remove(pos);
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
    
    // Skill advancement methods
    
    pub fn get_skill_type(skill_name: &str) -> SkillType {
        match skill_name {
            // Combat skills
            "Sword" | "Axe" | "Mace" | "Spear" | "Dagger" | "Bow" | "Crossbow" | 
            "Javelin" | "Sling" | "Staff" | "Unarmed Combat" | "Shield" => SkillType::Combat,
            
            // Magic skills are handled separately
            
            // Everything else is percentage-based
            _ => SkillType::Percentage,
        }
    }
    
    pub fn advance_skill_with_pips(&mut self, skill_name: &str, pips: u8) -> SkillAdvancementResult {
        let skill_type = Self::get_skill_type(skill_name);
        let mut rng = rand::thread_rng();
        
        match skill_type {
            SkillType::Combat => self.advance_combat_skill(skill_name, pips, &mut rng),
            SkillType::Percentage => self.advance_percentage_skill(skill_name, pips, &mut rng),
            SkillType::Magic => {
                // Magic skills are handled by the magic system
                SkillAdvancementResult {
                    skill_name: skill_name.to_string(),
                    old_value: 0,
                    new_value: 0,
                    pips_used: 0,
                    rolls: vec![],
                    leveled_up: false,
                }
            }
        }
    }
    
    fn advance_combat_skill(&mut self, skill_name: &str, pips: u8, rng: &mut impl Rng) -> SkillAdvancementResult {
        let (current_level, current_percentage) = self.combat_skills.get(skill_name).copied().unwrap_or((0, 0));
        let old_value = current_percentage;
        
        let mut new_percentage = current_percentage;
        let mut new_level = current_level;
        let mut rolls = Vec::new();
        let mut leveled_up = false;
        
        // Roll for each pip
        for _ in 0..pips {
            let roll = rng.gen_range(1..=100);
            let success = roll <= new_percentage;
            rolls.push((roll, success));
            
            if success {
                new_percentage += 1;
                
                // Check if we hit 100% and need to level up
                if new_percentage >= 100 {
                    new_level += 1;
                    new_percentage = (current_percentage.max(20) - 4).max(0); // Reset to base-4, minimum 0
                    leveled_up = true;
                }
            }
        }
        
        // Update the skill
        self.combat_skills.insert(skill_name.to_string(), (new_level, new_percentage));
        
        // Reset pips
        self.skill_pips.insert(skill_name.to_string(), 0);
        
        SkillAdvancementResult {
            skill_name: skill_name.to_string(),
            old_value,
            new_value: new_percentage,
            pips_used: pips,
            rolls,
            leveled_up,
        }
    }
    
    fn advance_percentage_skill(&mut self, skill_name: &str, pips: u8, rng: &mut impl Rng) -> SkillAdvancementResult {
        let current_skill = self.skills.get(skill_name).copied().unwrap_or(0);
        let old_value = current_skill;
        
        let mut new_skill = current_skill;
        let mut rolls = Vec::new();
        
        // Roll for each pip
        for _ in 0..pips {
            let roll = rng.gen_range(1..=100);
            let success = roll >= new_skill;
            rolls.push((roll, success));
            
            if success {
                new_skill = (new_skill + 1).min(100); // Cap at 100%
            }
        }
        
        // Update the skill
        self.skills.insert(skill_name.to_string(), new_skill);
        
        // Reset pips
        self.skill_pips.insert(skill_name.to_string(), 0);
        
        SkillAdvancementResult {
            skill_name: skill_name.to_string(),
            old_value,
            new_value: new_skill,
            pips_used: pips,
            rolls,
            leveled_up: false,
        }
    }
    
    pub fn get_combat_skill_info(&self, skill_name: &str) -> (u8, u8) {
        self.combat_skills.get(skill_name).copied().unwrap_or((0, 0))
    }
    
    pub fn get_skill_display(&self, skill_name: &str) -> String {
        match Self::get_skill_type(skill_name) {
            SkillType::Combat => {
                let (level, percentage) = self.get_combat_skill_info(skill_name);
                if level > 0 {
                    format!("{}% (Level {})", percentage, level)
                } else {
                    format!("{}%", percentage)
                }
            }
            SkillType::Percentage => {
                let percentage = self.skills.get(skill_name).copied().unwrap_or(0);
                format!("{}%", percentage)
            }
            SkillType::Magic => {
                // For magic skills, we can't easily parse the school from a string
                // so we'll just return a generic display
                format!("Magic Skill")
            }
        }
    }
    
    // Equipment management methods
    
    pub fn equip_weapon(&mut self, weapon: Weapon) -> Option<Weapon> {
        let old_weapon = self.equipment.weapon.take();
        self.equipment.weapon = Some(weapon);
        old_weapon
    }
    
    pub fn unequip_weapon(&mut self) -> Option<Weapon> {
        self.equipment.weapon.take()
    }
    
    pub fn equip_armor(&mut self, armor: Armor) -> Option<Armor> {
        let old_armor = self.equipment.armor.take();
        self.equipment.armor = Some(armor);
        old_armor
    }
    
    pub fn unequip_armor(&mut self) -> Option<Armor> {
        self.equipment.armor.take()
    }
    
    pub fn equip_shield(&mut self, shield: Armor) -> Option<Armor> {
        let old_shield = self.equipment.shield.take();
        self.equipment.shield = Some(shield);
        old_shield
    }
    
    pub fn unequip_shield(&mut self) -> Option<Armor> {
        self.equipment.shield.take()
    }
    
    pub fn equip_accessory(&mut self, accessory: Accessory) -> Result<Option<Accessory>, String> {
        if self.equipment.accessory1.is_none() {
            self.equipment.accessory1 = Some(accessory);
            Ok(None)
        } else if self.equipment.accessory2.is_none() {
            self.equipment.accessory2 = Some(accessory);
            Ok(None)
        } else {
            Err("Both accessory slots are full".to_string())
        }
    }
    
    pub fn unequip_accessory(&mut self, slot: u8) -> Option<Accessory> {
        match slot {
            1 => self.equipment.accessory1.take(),
            2 => self.equipment.accessory2.take(),
            _ => None,
        }
    }
    
    pub fn get_total_equipment_bonuses(&self) -> StatBonuses {
        let mut total_bonuses = StatBonuses::new();
        
        // Add weapon bonuses
        if let Some(weapon) = &self.equipment.weapon {
            total_bonuses.attack_bonus += weapon.attack_bonus;
            total_bonuses.damage_bonus += weapon.damage_bonus;
        }
        
        // Add accessory bonuses
        if let Some(accessory) = &self.equipment.accessory1 {
            total_bonuses.add_bonuses(&accessory.stat_bonuses);
        }
        if let Some(accessory) = &self.equipment.accessory2 {
            total_bonuses.add_bonuses(&accessory.stat_bonuses);
        }
        
        total_bonuses
    }
    
    pub fn get_equipped_weapon(&self) -> Option<&Weapon> {
        self.equipment.weapon.as_ref()
    }
    
    pub fn get_equipped_armor(&self) -> Option<&Armor> {
        self.equipment.armor.as_ref()
    }
    
    pub fn get_equipped_shield(&self) -> Option<&Armor> {
        self.equipment.shield.as_ref()
    }
    
    // Inventory management methods
    
    pub fn add_item_to_inventory(&mut self, item: InventoryItem) -> Result<(), String> {
        self.inventory.add_item(item)
    }
    
    pub fn remove_item_from_inventory(&mut self, index: usize) -> Option<InventoryItem> {
        self.inventory.remove_item(index)
    }
    
    pub fn get_inventory_weight(&self) -> f32 {
        self.inventory.get_total_weight()
    }
    
    pub fn get_weight_penalty(&self) -> f32 {
        self.inventory.calculate_weight_penalty()
    }
    
    pub fn can_carry_weight(&self, additional_weight: f32) -> bool {
        self.inventory.can_carry_additional_weight(additional_weight)
    }
    
    pub fn equip_item_from_inventory(&mut self, inventory_index: usize) -> Result<String, String> {
        let item = self.inventory.get_item(inventory_index)
            .ok_or("Invalid inventory index")?;
            
        match &item.item_type {
            ItemType::Weapon(_weapon) => {
                let removed_item = self.inventory.remove_item(inventory_index)
                    .ok_or("Failed to remove item from inventory")?;
                
                if let ItemType::Weapon(weapon) = removed_item.item_type {
                    if let Some(old_weapon) = self.equip_weapon(weapon) {
                        // Put the old weapon back in inventory
                        let old_item = InventoryItem::from_weapon(old_weapon);
                        self.inventory.add_item(old_item)?;
                    }
                    Ok(format!("Equipped {}", removed_item.name))
                } else {
                    Err("Item type mismatch".to_string())
                }
            }
            ItemType::Armor(_armor) => {
                let removed_item = self.inventory.remove_item(inventory_index)
                    .ok_or("Failed to remove item from inventory")?;
                
                if let ItemType::Armor(armor) = removed_item.item_type {
                    match armor.armor_type {
                        ArmorType::Shield => {
                            if let Some(old_shield) = self.equip_shield(armor) {
                                let old_item = InventoryItem::from_armor(old_shield);
                                self.inventory.add_item(old_item)?;
                            }
                        }
                        _ => {
                            if let Some(old_armor) = self.equip_armor(armor) {
                                let old_item = InventoryItem::from_armor(old_armor);
                                self.inventory.add_item(old_item)?;
                            }
                        }
                    }
                    Ok(format!("Equipped {}", removed_item.name))
                } else {
                    Err("Item type mismatch".to_string())
                }
            }
            ItemType::Accessory(_accessory) => {
                let removed_item = self.inventory.remove_item(inventory_index)
                    .ok_or("Failed to remove item from inventory")?;
                
                if let ItemType::Accessory(ref accessory) = removed_item.item_type {
                    match self.equip_accessory(accessory.clone()) {
                        Ok(old_accessory) => {
                            if let Some(old) = old_accessory {
                                let old_item = InventoryItem::from_accessory(old);
                                self.inventory.add_item(old_item)?;
                            }
                            Ok(format!("Equipped {}", removed_item.name))
                        }
                        Err(e) => {
                            // Put item back if equipping failed
                            self.inventory.add_item(removed_item)?;
                            Err(e)
                        }
                    }
                } else {
                    Err("Item type mismatch".to_string())
                }
            }
            _ => Err("Cannot equip this item type".to_string()),
        }
    }
    
    pub fn unequip_to_inventory(&mut self, slot: EquipmentSlot) -> Result<String, String> {
        match slot {
            EquipmentSlot::Weapon => {
                if let Some(weapon) = self.unequip_weapon() {
                    let item = InventoryItem::from_weapon(weapon);
                    let name = item.name.clone();
                    self.inventory.add_item(item)?;
                    Ok(format!("Unequipped {}", name))
                } else {
                    Err("No weapon equipped".to_string())
                }
            }
            EquipmentSlot::Armor => {
                if let Some(armor) = self.unequip_armor() {
                    let item = InventoryItem::from_armor(armor);
                    let name = item.name.clone();
                    self.inventory.add_item(item)?;
                    Ok(format!("Unequipped {}", name))
                } else {
                    Err("No armor equipped".to_string())
                }
            }
            EquipmentSlot::Shield => {
                if let Some(shield) = self.unequip_shield() {
                    let item = InventoryItem::from_armor(shield);
                    let name = item.name.clone();
                    self.inventory.add_item(item)?;
                    Ok(format!("Unequipped {}", name))
                } else {
                    Err("No shield equipped".to_string())
                }
            }
            EquipmentSlot::Accessory1 => {
                if let Some(accessory) = self.unequip_accessory(1) {
                    let item = InventoryItem::from_accessory(accessory);
                    let name = item.name.clone();
                    self.inventory.add_item(item)?;
                    Ok(format!("Unequipped {}", name))
                } else {
                    Err("No accessory in slot 1".to_string())
                }
            }
            EquipmentSlot::Accessory2 => {
                if let Some(accessory) = self.unequip_accessory(2) {
                    let item = InventoryItem::from_accessory(accessory);
                    let name = item.name.clone();
                    self.inventory.add_item(item)?;
                    Ok(format!("Unequipped {}", name))
                } else {
                    Err("No accessory in slot 2".to_string())
                }
            }
        }
    }
    
    // Comprehensive skill system for showing trained and untrained skills
    
    pub fn get_all_available_skills() -> Vec<SkillDefinition> {
        vec![
            // Basic Skills (DEX x2 base)
            SkillDefinition::new("Climbing", SkillCategory::Basic, SkillBase::DexterityX2),
            SkillDefinition::new("Move Silently", SkillCategory::Basic, SkillBase::DexterityX2),
            SkillDefinition::new("Hide", SkillCategory::Basic, SkillBase::DexterityX2),
            SkillDefinition::new("Sleight of Hand", SkillCategory::Basic, SkillBase::DexterityX2),
            SkillDefinition::new("Tumbling", SkillCategory::Basic, SkillBase::DexterityX2),
            SkillDefinition::new("Lock Picking", SkillCategory::Basic, SkillBase::DexterityX2),
            
            // Basic Skills (STR x2 base)
            SkillDefinition::new("Swimming", SkillCategory::Basic, SkillBase::StrengthX2),
            SkillDefinition::new("Jump", SkillCategory::Basic, SkillBase::StrengthX2),
            
            // Basic Skills (INT x2 base)
            SkillDefinition::new("Plant ID", SkillCategory::Basic, SkillBase::IntellectX2),
            SkillDefinition::new("Track ID", SkillCategory::Basic, SkillBase::IntellectX2),
            SkillDefinition::new("History", SkillCategory::Basic, SkillBase::IntellectX2),
            SkillDefinition::new("Lore", SkillCategory::Basic, SkillBase::IntellectX2),
            SkillDefinition::new("Language", SkillCategory::Basic, SkillBase::IntellectX2),
            
            // Basic Skills (AWR x2 base)
            SkillDefinition::new("Perception", SkillCategory::Basic, SkillBase::AwarenessX2),
            SkillDefinition::new("Listen", SkillCategory::Basic, SkillBase::AwarenessX2),
            SkillDefinition::new("Search", SkillCategory::Basic, SkillBase::AwarenessX2),
            
            // Basic Skills (STA x2 base)
            SkillDefinition::new("Survival", SkillCategory::Basic, SkillBase::StaminaX2),
            SkillDefinition::new("Endurance", SkillCategory::Basic, SkillBase::StaminaX2),
            
            // Combination Skills
            SkillDefinition::new("Tactics", SkillCategory::Basic, SkillBase::DexterityPlusAwareness),
            SkillDefinition::new("Blind Fighting", SkillCategory::Basic, SkillBase::DexterityPlusAwareness),
            SkillDefinition::new("Weapon Stomp", SkillCategory::Basic, SkillBase::DexterityPlusAwareness),
            SkillDefinition::new("Tracking", SkillCategory::Basic, SkillBase::AwarenessPlusIntellect),
            
            // Percentile Skills
            SkillDefinition::new("Crafting", SkillCategory::Percentile, SkillBase::IntellectX2),
            SkillDefinition::new("Healing", SkillCategory::Percentile, SkillBase::IntellectX2),
            SkillDefinition::new("Singing", SkillCategory::Percentile, SkillBase::InsightX2),
            SkillDefinition::new("Alchemy", SkillCategory::Percentile, SkillBase::IntellectX2),
            SkillDefinition::new("Engineering", SkillCategory::Percentile, SkillBase::IntellectX2),
            SkillDefinition::new("Leadership", SkillCategory::Percentile, SkillBase::InsightX2),
            SkillDefinition::new("Intimidation", SkillCategory::Percentile, SkillBase::StrengthPlusInsight),
            SkillDefinition::new("Persuasion", SkillCategory::Percentile, SkillBase::InsightX2),
            
            // Combat Skills (STR + DEX base)
            SkillDefinition::new("Sword", SkillCategory::Combat, SkillBase::StrengthPlusDexterity),
            SkillDefinition::new("Axe", SkillCategory::Combat, SkillBase::StrengthPlusDexterity),
            SkillDefinition::new("Mace", SkillCategory::Combat, SkillBase::StrengthPlusDexterity),
            SkillDefinition::new("Spear", SkillCategory::Combat, SkillBase::StrengthPlusDexterity),
            SkillDefinition::new("Dagger", SkillCategory::Combat, SkillBase::StrengthPlusDexterity),
            SkillDefinition::new("Staff", SkillCategory::Combat, SkillBase::StrengthPlusDexterity),
            SkillDefinition::new("Unarmed Combat", SkillCategory::Combat, SkillBase::StrengthPlusDexterity),
            SkillDefinition::new("Shield", SkillCategory::Combat, SkillBase::StrengthPlusDexterity),
            
            // Missile Combat Skills (DEX x2 base)
            SkillDefinition::new("Bow", SkillCategory::Combat, SkillBase::DexterityX2),
            SkillDefinition::new("Crossbow", SkillCategory::Combat, SkillBase::DexterityX2),
            SkillDefinition::new("Javelin", SkillCategory::Combat, SkillBase::DexterityX2),
            SkillDefinition::new("Sling", SkillCategory::Combat, SkillBase::DexterityX2),
            
            // Magic Skills (handled separately but listed for completeness)
            SkillDefinition::new("Beast Magic", SkillCategory::Magic, SkillBase::PowerBased),
            SkillDefinition::new("Elemental Magic", SkillCategory::Magic, SkillBase::PowerBased),
            SkillDefinition::new("Enchantment Magic", SkillCategory::Magic, SkillBase::PowerBased),
            SkillDefinition::new("Necromancer Magic", SkillCategory::Magic, SkillBase::PowerBased),
            SkillDefinition::new("Divine Magic", SkillCategory::Magic, SkillBase::PowerBased),
            SkillDefinition::new("Mind Magic", SkillCategory::Magic, SkillBase::PowerBased),
        ]
    }
    
    pub fn calculate_skill_base(&self, skill_base: &SkillBase) -> u8 {
        match skill_base {
            SkillBase::StrengthX2 => (self.characteristics.strength * 2.0) as u8,
            SkillBase::StaminaX2 => (self.characteristics.stamina * 2.0) as u8,
            SkillBase::IntellectX2 => (self.characteristics.intellect * 2.0) as u8,
            SkillBase::InsightX2 => (self.characteristics.insight * 2.0) as u8,
            SkillBase::DexterityX2 => (self.characteristics.dexterity * 2.0) as u8,
            SkillBase::AwarenessX2 => (self.characteristics.awareness * 2.0) as u8,
            SkillBase::StrengthPlusDexterity => (self.characteristics.strength + self.characteristics.dexterity) as u8,
            SkillBase::DexterityPlusAwareness => (self.characteristics.dexterity + self.characteristics.awareness) as u8,
            SkillBase::AwarenessPlusIntellect => (self.characteristics.awareness + self.characteristics.intellect) as u8,
            SkillBase::StrengthPlusInsight => (self.characteristics.strength + self.characteristics.insight) as u8,
            SkillBase::PowerBased => self.magic.spell_points.max as u8, // Magic skills use spell points as base
        }
    }
    
    pub fn get_effective_skill_value(&self, skill_name: &str) -> (u8, bool) {
        // Returns (skill_value, is_trained)
        
        // Check if it's a trained skill first
        if let Some(&level) = self.skills.get(skill_name) {
            return (level, true);
        }
        
        // Check if it's a trained combat skill
        if let Some(&(level, percentage)) = self.combat_skills.get(skill_name) {
            return (if level > 0 { percentage } else { percentage.max(self.get_untrained_combat_base(skill_name)) }, level > 0);
        }
        
        // Check magic skills
        let magic_schools = [
            ("Beast Magic", magic::MagicSchool::Beast),
            ("Elemental Magic", magic::MagicSchool::Elemental),
            ("Enchantment Magic", magic::MagicSchool::Enchantment),
            ("Necromancer Magic", magic::MagicSchool::Necromancer),
            ("Divine Magic", magic::MagicSchool::Divine),
            ("Mind Magic", magic::MagicSchool::Beast), // Note: Mind Magic might be its own school
        ];
        
        for (name, school) in magic_schools {
            if skill_name == name {
                let skill_level = self.magic.get_school_skill(&school);
                return (skill_level * 5, skill_level > 0); // Convert level to percentage
            }
        }
        
        // Return untrained base calculation
        let all_skills = Self::get_all_available_skills();
        for skill_def in all_skills {
            if skill_def.name == skill_name {
                return (self.calculate_skill_base(&skill_def.base), false);
            }
        }
        
        // Default if skill not found
        (0, false)
    }
    
    fn get_untrained_combat_base(&self, skill_name: &str) -> u8 {
        let all_skills = Self::get_all_available_skills();
        for skill_def in all_skills {
            if skill_def.name == skill_name && skill_def.category == SkillCategory::Combat {
                return self.calculate_skill_base(&skill_def.base);
            }
        }
        0
    }
    
    pub fn get_skills_by_category(&self) -> (Vec<(String, u8, bool)>, Vec<(String, u8, bool)>, Vec<(String, u8, bool)>, Vec<(String, u8, bool)>) {
        // Returns (basic_skills, percentile_skills, combat_skills, magic_skills)
        // Each tuple is (name, value, is_trained)
        
        let all_skills = Self::get_all_available_skills();
        let mut basic_skills = Vec::new();
        let mut percentile_skills = Vec::new();
        let mut combat_skills = Vec::new();
        let mut magic_skills = Vec::new();
        
        for skill_def in all_skills {
            let (value, is_trained) = self.get_effective_skill_value(&skill_def.name);
            let skill_info = (skill_def.name.clone(), value, is_trained);
            
            match skill_def.category {
                SkillCategory::Basic => basic_skills.push(skill_info),
                SkillCategory::Percentile => percentile_skills.push(skill_info),
                SkillCategory::Combat => combat_skills.push(skill_info),
                SkillCategory::Magic => magic_skills.push(skill_info),
            }
        }
        
        // Sort each category by name
        basic_skills.sort_by(|a, b| a.0.cmp(&b.0));
        percentile_skills.sort_by(|a, b| a.0.cmp(&b.0));
        combat_skills.sort_by(|a, b| a.0.cmp(&b.0));
        magic_skills.sort_by(|a, b| a.0.cmp(&b.0));
        
        (basic_skills, percentile_skills, combat_skills, magic_skills)
    }
}

#[derive(Debug, Clone)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Shield,
    Accessory1,
    Accessory2,
}

impl EquippedItems {
    pub fn new() -> Self {
        EquippedItems {
            weapon: None,
            armor: None,
            shield: None,
            accessory1: None,
            accessory2: None,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.weapon.is_none() && 
        self.armor.is_none() && 
        self.shield.is_none() && 
        self.accessory1.is_none() && 
        self.accessory2.is_none()
    }
}

impl StatBonuses {
    pub fn new() -> Self {
        StatBonuses {
            attack_bonus: 0,
            defense_bonus: 0,
            damage_bonus: 0,
            characteristic_bonuses: HashMap::new(),
            skill_bonuses: HashMap::new(),
        }
    }
    
    pub fn add_bonuses(&mut self, other: &StatBonuses) {
        self.attack_bonus += other.attack_bonus;
        self.defense_bonus += other.defense_bonus;
        self.damage_bonus += other.damage_bonus;
        
        for (characteristic, bonus) in &other.characteristic_bonuses {
            *self.characteristic_bonuses.entry(characteristic.clone()).or_insert(0) += bonus;
        }
        
        for (skill, bonus) in &other.skill_bonuses {
            *self.skill_bonuses.entry(skill.clone()).or_insert(0) += bonus;
        }
    }
}

impl Accessory {
    pub fn basic_ring() -> Self {
        Accessory {
            name: "Basic Ring".to_string(),
            accessory_type: AccessoryType::Ring,
            stat_bonuses: StatBonuses::new(),
            magical: false,
        }
    }
    
    pub fn ring_of_protection() -> Self {
        let mut bonuses = StatBonuses::new();
        bonuses.defense_bonus = 1;
        
        Accessory {
            name: "Ring of Protection".to_string(),
            accessory_type: AccessoryType::Ring,
            stat_bonuses: bonuses,
            magical: true,
        }
    }
    
    pub fn strength_belt() -> Self {
        let mut bonuses = StatBonuses::new();
        bonuses.characteristic_bonuses.insert("STR".to_string(), 2);
        bonuses.damage_bonus = 1;
        
        Accessory {
            name: "Belt of Strength".to_string(),
            accessory_type: AccessoryType::Belt,
            stat_bonuses: bonuses,
            magical: true,
        }
    }
}

impl Inventory {
    pub fn new(max_weight: f32) -> Self {
        Inventory {
            items: Vec::new(),
            max_weight,
            weight_penalty: 0.0,
        }
    }
    
    pub fn new_starting_inventory(characteristics: &ForgeCharacteristics) -> Self {
        let mut inventory = Inventory::new(characteristics.strength * 10.0); // STR * 10 = max weight
        
        // Add starting items
        inventory.add_item(InventoryItem::farm_clothes()).ok();
        inventory.add_item(InventoryItem::simple_tools()).ok();
        inventory.add_item(InventoryItem::torch()).ok();
        inventory.add_item(InventoryItem::torch()).ok();
        inventory.add_item(InventoryItem::torch()).ok();
        inventory.add_item(InventoryItem::rations()).ok();
        
        inventory
    }
    
    pub fn add_item(&mut self, mut item: InventoryItem) -> Result<(), String> {
        // Check weight capacity
        let additional_weight = item.weight * item.quantity as f32;
        if !self.can_carry_additional_weight(additional_weight) {
            return Err("Cannot carry that much weight".to_string());
        }
        
        // Try to stack with existing items
        if item.stack_size > 1 {
            for existing_item in &mut self.items {
                if existing_item.name == item.name && 
                   existing_item.quantity < existing_item.stack_size {
                    let can_add = existing_item.stack_size - existing_item.quantity;
                    let to_add = item.quantity.min(can_add);
                    existing_item.quantity += to_add;
                    item.quantity -= to_add;
                    
                    if item.quantity == 0 {
                        return Ok(());
                    }
                }
            }
        }
        
        // Add as new item if not fully stacked
        if item.quantity > 0 {
            self.items.push(item);
        }
        
        self.update_weight_penalty();
        Ok(())
    }
    
    pub fn remove_item(&mut self, index: usize) -> Option<InventoryItem> {
        if index < self.items.len() {
            let item = self.items.remove(index);
            self.update_weight_penalty();
            Some(item)
        } else {
            None
        }
    }
    
    pub fn get_item(&self, index: usize) -> Option<&InventoryItem> {
        self.items.get(index)
    }
    
    pub fn get_total_weight(&self) -> f32 {
        self.items.iter()
            .map(|item| item.weight * item.quantity as f32)
            .sum()
    }
    
    pub fn can_carry_additional_weight(&self, additional_weight: f32) -> bool {
        self.get_total_weight() + additional_weight <= self.max_weight
    }
    
    pub fn calculate_weight_penalty(&self) -> f32 {
        let current_weight = self.get_total_weight();
        let weight_ratio = current_weight / self.max_weight;
        
        if weight_ratio <= 0.75 {
            0.0 // No penalty up to 75% capacity
        } else if weight_ratio <= 0.9 {
            0.1 // 10% penalty from 75-90%
        } else if weight_ratio <= 1.0 {
            0.25 // 25% penalty from 90-100%
        } else {
            0.5 // 50% penalty when overloaded
        }
    }
    
    fn update_weight_penalty(&mut self) {
        self.weight_penalty = self.calculate_weight_penalty();
    }
    
    pub fn get_items_by_type(&self, item_type: &str) -> Vec<(usize, &InventoryItem)> {
        self.items.iter()
            .enumerate()
            .filter(|(_, item)| {
                match item_type {
                    "weapon" => matches!(item.item_type, ItemType::Weapon(_)),
                    "armor" => matches!(item.item_type, ItemType::Armor(_)),
                    "accessory" => matches!(item.item_type, ItemType::Accessory(_)),
                    "consumable" => matches!(item.item_type, ItemType::Consumable(_)),
                    "material" => matches!(item.item_type, ItemType::Material(_)),
                    "misc" => matches!(item.item_type, ItemType::Misc(_)),
                    _ => true,
                }
            })
            .collect()
    }
}

impl InventoryItem {
    pub fn from_weapon(weapon: Weapon) -> Self {
        let weight = match weapon.weapon_type {
            WeaponType::Dagger => 1.0,
            WeaponType::Sword => 3.0,
            WeaponType::Axe => 4.0,
            WeaponType::Mace => 3.5,
            WeaponType::Spear => 3.0,
            WeaponType::Bow => 2.0,
            WeaponType::Crossbow => 5.0,
            WeaponType::Staff => 2.5,
            WeaponType::Unarmed => 0.0,
        };
        
        let value = match weapon.weapon_type {
            WeaponType::Unarmed => 0,
            WeaponType::Dagger => 10,
            WeaponType::Sword => 50,
            WeaponType::Axe => 45,
            WeaponType::Mace => 40,
            WeaponType::Spear => 25,
            WeaponType::Bow => 35,
            WeaponType::Crossbow => 75,
            WeaponType::Staff => 15,
        };
        
        InventoryItem {
            name: weapon.name.clone(),
            item_type: ItemType::Weapon(weapon),
            weight,
            stack_size: 1,
            quantity: 1,
            value,
            description: "A weapon for combat".to_string(),
        }
    }
    
    pub fn from_armor(armor: Armor) -> Self {
        let weight = match armor.armor_type {
            ArmorType::Light => 10.0,
            ArmorType::Medium => 25.0,
            ArmorType::Heavy => 45.0,
            ArmorType::Shield => 8.0,
        };
        
        let value = armor.armor_rating as u32 * 25;
        
        InventoryItem {
            name: armor.name.clone(),
            item_type: ItemType::Armor(armor),
            weight,
            stack_size: 1,
            quantity: 1,
            value,
            description: "Protective armor".to_string(),
        }
    }
    
    pub fn from_accessory(accessory: Accessory) -> Self {
        let weight = match accessory.accessory_type {
            AccessoryType::Ring => 0.1,
            AccessoryType::Amulet => 0.5,
            AccessoryType::Belt => 2.0,
            AccessoryType::Cloak => 3.0,
            AccessoryType::Boots => 4.0,
            AccessoryType::Gloves => 1.0,
        };
        
        let value = if accessory.magical { 100 } else { 25 };
        
        InventoryItem {
            name: accessory.name.clone(),
            item_type: ItemType::Accessory(accessory),
            weight,
            stack_size: 1,
            quantity: 1,
            value,
            description: "An accessory item".to_string(),
        }
    }
    
    // Starting item constructors
    pub fn farm_clothes() -> Self {
        InventoryItem {
            name: "Farm Clothes".to_string(),
            item_type: ItemType::Misc(MiscItem {
                misc_type: MiscType::Trade,
                special_properties: vec!["Starting equipment".to_string()],
            }),
            weight: 2.0,
            stack_size: 1,
            quantity: 1,
            value: 5,
            description: "Simple farming clothes. Provides basic protection from the elements.".to_string(),
        }
    }
    
    pub fn simple_tools() -> Self {
        InventoryItem {
            name: "Simple Tools".to_string(),
            item_type: ItemType::Misc(MiscItem {
                misc_type: MiscType::Tool,
                special_properties: vec!["Basic crafting".to_string()],
            }),
            weight: 3.0,
            stack_size: 1,
            quantity: 1,
            value: 15,
            description: "A collection of basic tools for everyday tasks.".to_string(),
        }
    }
    
    pub fn torch() -> Self {
        InventoryItem {
            name: "Torch".to_string(),
            item_type: ItemType::Consumable(ConsumableItem {
                consumable_type: ConsumableType::Torch,
                effect: "Provides light in dark areas".to_string(),
                duration: Some(50), // 50 turns of light
            }),
            weight: 1.0,
            stack_size: 20,
            quantity: 1,
            value: 2,
            description: "A wooden torch that provides light for about an hour.".to_string(),
        }
    }
    
    pub fn rations() -> Self {
        InventoryItem {
            name: "Trail Rations".to_string(),
            item_type: ItemType::Consumable(ConsumableItem {
                consumable_type: ConsumableType::Food,
                effect: "Restores hunger and provides minor healing".to_string(),
                duration: None,
            }),
            weight: 1.5,
            stack_size: 10,
            quantity: 3,
            value: 1,
            description: "Dried food suitable for long journeys.".to_string(),
        }
    }
    
    pub fn health_potion() -> Self {
        InventoryItem {
            name: "Health Potion".to_string(),
            item_type: ItemType::Consumable(ConsumableItem {
                consumable_type: ConsumableType::HealthPotion,
                effect: "Restores 2d6+2 hit points".to_string(),
                duration: None,
            }),
            weight: 0.5,
            stack_size: 5,
            quantity: 1,
            value: 50,
            description: "A red potion that restores health when consumed.".to_string(),
        }
    }
    
    pub fn mana_potion() -> Self {
        InventoryItem {
            name: "Mana Potion".to_string(),
            item_type: ItemType::Consumable(ConsumableItem {
                consumable_type: ConsumableType::ManaPotion,
                effect: "Restores 1d6+1 spell points".to_string(),
                duration: None,
            }),
            weight: 0.5,
            stack_size: 5,
            quantity: 1,
            value: 75,
            description: "A blue potion that restores magical energy.".to_string(),
        }
    }
}
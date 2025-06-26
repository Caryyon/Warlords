use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MagicSchool {
    Beast,       // Animal magic, nature communication
    Elemental,   // Fire, water, earth, air magic
    Enchantment, // Weapon/armor enhancement, protection
    Necromancer, // Death magic, undead, life drain
    Divine,      // Healing, blessing, turning undead
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellPoints {
    pub current: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicSystem {
    pub spell_points: SpellPoints,
    pub known_spells: HashMap<MagicSchool, Vec<String>>, // School -> list of known spell names
    pub school_skills: HashMap<MagicSchool, u8>,         // School -> skill level (0-20)
    pub school_pips: HashMap<MagicSchool, u8>,          // School -> accumulated pips
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpellTarget {
    Self_,              // Caster only
    SingleEnemy,        // One enemy
    SingleAlly,         // One ally (including self)
    AllEnemies,         // All enemies in combat
    AllAllies,          // All allies in combat
    Area(u8),          // Area effect with radius
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpellEffect {
    Damage {
        dice: String,      // e.g., "2d6"
        bonus: i8,
        damage_type: super::DamageType,
    },
    Heal {
        dice: String,
        bonus: i8,
    },
    Buff {
        stat: String,      // "attack", "defense", "damage", etc.
        modifier: i8,
        duration: u8,      // rounds
    },
    Debuff {
        stat: String,
        modifier: i8,
        duration: u8,
    },
    Special {
        effect: String,    // Custom effect description
        duration: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    pub name: String,
    pub school: MagicSchool,
    pub level: u8,             // Spell level (1-5)
    pub cost: u8,              // Spell points required
    pub target: SpellTarget,
    pub effects: Vec<SpellEffect>,
    pub description: String,
    pub success_chance_base: u8,  // Base success chance (modified by skill)
    pub backfire_chance: u8,      // Chance of backfire on failure
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpellResult {
    Success {
        effects_applied: Vec<String>,
    },
    Failure {
        message: String,
    },
    Backfire {
        message: String,
        damage_to_caster: Option<u32>,
    },
}

impl MagicSystem {
    pub fn new(power: u8) -> Self {
        let max_spell_points = (power as u32) * 2; // Power × 2 starting spell points
        
        MagicSystem {
            spell_points: SpellPoints {
                current: max_spell_points,
                max: max_spell_points,
            },
            known_spells: HashMap::new(),
            school_skills: HashMap::new(),
            school_pips: HashMap::new(),
        }
    }
    
    pub fn can_cast_spell(&self, spell: &Spell) -> bool {
        self.spell_points.current >= spell.cost as u32
    }
    
    pub fn spend_spell_points(&mut self, cost: u8) -> bool {
        if self.spell_points.current >= cost as u32 {
            self.spell_points.current -= cost as u32;
            true
        } else {
            false
        }
    }
    
    pub fn restore_spell_points(&mut self, amount: u32) {
        self.spell_points.current = (self.spell_points.current + amount).min(self.spell_points.max);
    }
    
    pub fn get_school_skill(&self, school: &MagicSchool) -> u8 {
        self.school_skills.get(school).copied().unwrap_or(0)
    }
    
    pub fn add_known_spell(&mut self, spell_name: String, school: MagicSchool) {
        self.known_spells.entry(school).or_insert_with(Vec::new).push(spell_name);
    }
    
    pub fn knows_spell(&self, spell_name: &str, school: &MagicSchool) -> bool {
        self.known_spells
            .get(school)
            .map(|spells| spells.contains(&spell_name.to_string()))
            .unwrap_or(false)
    }
    
    pub fn get_all_known_spells(&self) -> Vec<(MagicSchool, String)> {
        let mut all_spells = Vec::new();
        for (school, spells) in &self.known_spells {
            for spell_name in spells {
                all_spells.push((school.clone(), spell_name.clone()));
            }
        }
        all_spells
    }
}

impl std::fmt::Display for MagicSchool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MagicSchool::Beast => write!(f, "Beast Magic"),
            MagicSchool::Elemental => write!(f, "Elemental Magic"),
            MagicSchool::Enchantment => write!(f, "Enchantment Magic"),
            MagicSchool::Necromancer => write!(f, "Necromancer Magic"),
            MagicSchool::Divine => write!(f, "Divine Magic"),
        }
    }
}

// Create starter spells for each school
pub fn create_starter_spells() -> HashMap<String, Spell> {
    let mut spells = HashMap::new();
    
    // Beast Magic Spells
    spells.insert("Animal Communication".to_string(), Spell {
        name: "Animal Communication".to_string(),
        school: MagicSchool::Beast,
        level: 1,
        cost: 2,
        target: SpellTarget::Self_,
        effects: vec![SpellEffect::Special {
            effect: "Communicate with animals for 10 minutes".to_string(),
            duration: 10,
        }],
        description: "Allows the caster to speak with and understand animals.".to_string(),
        success_chance_base: 70,
        backfire_chance: 10,
    });
    
    spells.insert("Bear Strength".to_string(), Spell {
        name: "Bear Strength".to_string(),
        school: MagicSchool::Beast,
        level: 1,
        cost: 3,
        target: SpellTarget::SingleAlly,
        effects: vec![SpellEffect::Buff {
            stat: "damage".to_string(),
            modifier: 3,
            duration: 5,
        }],
        description: "Grants the strength of a bear, increasing damage for 5 rounds.".to_string(),
        success_chance_base: 65,
        backfire_chance: 15,
    });
    
    // Elemental Magic Spells
    spells.insert("Fire Bolt".to_string(), Spell {
        name: "Fire Bolt".to_string(),
        school: MagicSchool::Elemental,
        level: 1,
        cost: 2,
        target: SpellTarget::SingleEnemy,
        effects: vec![SpellEffect::Damage {
            dice: "1d6".to_string(),
            bonus: 2,
            damage_type: super::DamageType::Magic,
        }],
        description: "Hurls a bolt of fire at a single enemy.".to_string(),
        success_chance_base: 75,
        backfire_chance: 10,
    });
    
    spells.insert("Lightning Strike".to_string(), Spell {
        name: "Lightning Strike".to_string(),
        school: MagicSchool::Elemental,
        level: 2,
        cost: 4,
        target: SpellTarget::SingleEnemy,
        effects: vec![SpellEffect::Damage {
            dice: "2d6".to_string(),
            bonus: 1,
            damage_type: super::DamageType::Magic,
        }],
        description: "Calls down a lightning bolt on a single enemy.".to_string(),
        success_chance_base: 65,
        backfire_chance: 20,
    });
    
    // Enchantment Magic Spells
    spells.insert("Weapon Blessing".to_string(), Spell {
        name: "Weapon Blessing".to_string(),
        school: MagicSchool::Enchantment,
        level: 1,
        cost: 3,
        target: SpellTarget::SingleAlly,
        effects: vec![SpellEffect::Buff {
            stat: "attack".to_string(),
            modifier: 2,
            duration: 8,
        }],
        description: "Blesses a weapon, increasing attack accuracy for 8 rounds.".to_string(),
        success_chance_base: 80,
        backfire_chance: 5,
    });
    
    spells.insert("Shield of Faith".to_string(), Spell {
        name: "Shield of Faith".to_string(),
        school: MagicSchool::Enchantment,
        level: 1,
        cost: 4,
        target: SpellTarget::SingleAlly,
        effects: vec![SpellEffect::Buff {
            stat: "defense".to_string(),
            modifier: 3,
            duration: 6,
        }],
        description: "Creates a magical shield that increases defense for 6 rounds.".to_string(),
        success_chance_base: 75,
        backfire_chance: 10,
    });
    
    // Necromancer Magic Spells
    spells.insert("Drain Life".to_string(), Spell {
        name: "Drain Life".to_string(),
        school: MagicSchool::Necromancer,
        level: 1,
        cost: 3,
        target: SpellTarget::SingleEnemy,
        effects: vec![
            SpellEffect::Damage {
                dice: "1d4".to_string(),
                bonus: 1,
                damage_type: super::DamageType::Magic,
            },
            SpellEffect::Heal {
                dice: "1d4".to_string(),
                bonus: 1,
            },
        ],
        description: "Drains life from an enemy and heals the caster.".to_string(),
        success_chance_base: 60,
        backfire_chance: 25,
    });
    
    spells.insert("Weaken".to_string(), Spell {
        name: "Weaken".to_string(),
        school: MagicSchool::Necromancer,
        level: 1,
        cost: 2,
        target: SpellTarget::SingleEnemy,
        effects: vec![SpellEffect::Debuff {
            stat: "attack".to_string(),
            modifier: -2,
            duration: 4,
        }],
        description: "Weakens an enemy, reducing their attack for 4 rounds.".to_string(),
        success_chance_base: 70,
        backfire_chance: 15,
    });
    
    // Divine Magic Spells
    spells.insert("Heal Wounds".to_string(), Spell {
        name: "Heal Wounds".to_string(),
        school: MagicSchool::Divine,
        level: 1,
        cost: 3,
        target: SpellTarget::SingleAlly,
        effects: vec![SpellEffect::Heal {
            dice: "1d6".to_string(),
            bonus: 2,
        }],
        description: "Channels divine energy to heal wounds.".to_string(),
        success_chance_base: 85,
        backfire_chance: 5,
    });
    
    spells.insert("Turn Undead".to_string(), Spell {
        name: "Turn Undead".to_string(),
        school: MagicSchool::Divine,
        level: 2,
        cost: 4,
        target: SpellTarget::AllEnemies,
        effects: vec![SpellEffect::Special {
            effect: "Causes undead enemies to flee or become paralyzed".to_string(),
            duration: 3,
        }],
        description: "Channels divine power to turn away undead creatures.".to_string(),
        success_chance_base: 70,
        backfire_chance: 10,
    });
    
    spells
}
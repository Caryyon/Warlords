use serde::{Deserialize, Serialize};
use rand::Rng;
use super::{ForgeCharacter, CombatStats};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DamageType {
    Slashing,
    Piercing,
    Bludgeoning,
    Magic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeaponType {
    Sword,
    Axe,
    Mace,
    Dagger,
    Spear,
    Bow,
    Crossbow,
    Staff,
    Unarmed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    pub weapon_type: WeaponType,
    pub damage_dice: String,  // e.g., "1d8", "2d6"
    pub damage_type: DamageType,
    pub damage_bonus: i8,
    pub attack_bonus: i8,
    pub two_handed: bool,
    pub ranged: bool,
    pub range: Option<u32>,  // in feet
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArmorType {
    Light,
    Medium,
    Heavy,
    Shield,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Armor {
    pub name: String,
    pub armor_type: ArmorType,
    pub armor_rating: u8,  // Added to defensive values
    pub armor_points: u32,  // Current armor points
    pub max_armor_points: u32,  // Maximum armor points
    pub penalty: i8,  // Speed/dexterity penalty
}

impl Armor {
    pub fn get_current_armor_rating(&self) -> u8 {
        // Armor rating decreases as armor takes damage
        let percentage = (self.armor_points as f32 / self.max_armor_points as f32) * 100.0;
        
        if percentage >= 90.0 {
            self.armor_rating
        } else if percentage >= 75.0 {
            self.armor_rating.saturating_sub(1)
        } else if percentage >= 50.0 {
            self.armor_rating.saturating_sub(2)
        } else if percentage >= 25.0 {
            self.armor_rating.saturating_sub(3)
        } else if percentage > 0.0 {
            1.min(self.armor_rating)
        } else {
            0
        }
    }
    
    pub fn take_damage(&mut self, damage: u32) {
        self.armor_points = self.armor_points.saturating_sub(damage);
    }
    
    pub fn is_destroyed(&self) -> bool {
        self.armor_points == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatParticipant {
    pub name: String,
    pub combat_stats: CombatStats,
    pub weapon: Option<Weapon>,
    pub armor: Option<Armor>,
    pub shield: Option<Armor>,
    pub initiative: u8,
    pub is_player: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatAction {
    Attack { target_index: usize },
    Defend,
    Flee,
    UseItem { item: String },
    CastSpell { spell_name: String, target_index: Option<usize> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatResult {
    pub success: bool,
    pub damage: Option<u32>,
    pub message: String,
    pub critical: bool,
}

#[derive(Debug, Clone)]
pub struct CombatEncounter {
    pub participants: Vec<CombatParticipant>,
    pub current_turn: usize,
    pub round: u32,
    pub combat_log: Vec<String>,
}

impl Weapon {
    pub fn unarmed() -> Self {
        Weapon {
            name: "Fists".to_string(),
            weapon_type: WeaponType::Unarmed,
            damage_dice: "1d3".to_string(),
            damage_type: DamageType::Bludgeoning,
            damage_bonus: 0,
            attack_bonus: 0,
            two_handed: false,
            ranged: false,
            range: None,
        }
    }

    pub fn rusty_sword() -> Self {
        Weapon {
            name: "Rusty Sword".to_string(),
            weapon_type: WeaponType::Sword,
            damage_dice: "1d6".to_string(),
            damage_type: DamageType::Slashing,
            damage_bonus: 0,
            attack_bonus: 0,
            two_handed: false,
            ranged: false,
            range: None,
        }
    }

    pub fn roll_damage(&self) -> (u32, u32) {
        let mut rng = rand::thread_rng();
        let mut total = 0u32;
        let mut dice_count = 0u32;
        
        // Parse damage dice string (e.g., "1d8", "2d6")
        if let Some((num_dice, die_size)) = self.damage_dice.split_once('d') {
            dice_count = num_dice.parse().unwrap_or(1);
            let die_size: u32 = die_size.parse().unwrap_or(4);
            
            for _ in 0..dice_count {
                total += rng.gen_range(1..=die_size);
            }
        }
        
        // Apply damage bonus (can be negative)
        let final_damage = if self.damage_bonus >= 0 {
            total + self.damage_bonus as u32
        } else {
            total.saturating_sub(self.damage_bonus.abs() as u32)
        };
        
        (final_damage, dice_count)
    }
}

impl CombatParticipant {
    pub fn from_character(character: &ForgeCharacter, weapon: Option<Weapon>) -> Self {
        CombatParticipant {
            name: character.name.clone(),
            combat_stats: character.combat_stats.clone(),
            weapon,
            armor: None,
            shield: None,
            initiative: 0,
            is_player: true,
        }
    }

    pub fn create_enemy(name: &str, hp: u32, attack: u8, defense: u8, weapon: Option<Weapon>) -> Self {
        CombatParticipant {
            name: name.to_string(),
            combat_stats: CombatStats {
                hit_points: super::HealthPoints { current: hp, max: hp },
                attack_value: attack,
                defensive_value: defense,
                damage_bonus: 0,
            },
            weapon,
            armor: None,
            shield: None,
            initiative: 0,
            is_player: false,
        }
    }

    pub fn roll_initiative(&mut self) {
        let mut rng = rand::thread_rng();
        self.initiative = rng.gen_range(1..=20) + (self.combat_stats.defensive_value / 2);
    }

    pub fn get_total_attack_value(&self) -> u8 {
        let weapon_bonus = self.weapon.as_ref().map(|w| w.attack_bonus).unwrap_or(0);
        (self.combat_stats.attack_value as i8 + weapon_bonus).max(0) as u8
    }

    pub fn get_total_defense_value(&self) -> u8 {
        let armor_rating = self.armor.as_ref().map(|a| a.get_current_armor_rating()).unwrap_or(0);
        let shield_rating = self.shield.as_ref().map(|s| s.get_current_armor_rating()).unwrap_or(0);
        self.combat_stats.defensive_value + armor_rating + shield_rating
    }

    pub fn get_total_damage_bonus(&self) -> i8 {
        let weapon_bonus = self.weapon.as_ref().map(|w| w.damage_bonus).unwrap_or(0);
        self.combat_stats.damage_bonus + weapon_bonus
    }

    pub fn is_alive(&self) -> bool {
        self.combat_stats.hit_points.current > 0
    }

    pub fn take_damage(&mut self, damage: u32, damage_dice_count: u32) -> (u32, u32) {
        // In Forge, each damage die inflicts 1 point of actual damage to HP
        // The rest is absorbed by armor (if any)
        let actual_damage = damage_dice_count.min(damage);
        let mut armor_damage = damage.saturating_sub(actual_damage);
        
        // Apply armor damage first
        if let Some(armor) = &mut self.armor {
            if !armor.is_destroyed() {
                let absorbed = armor_damage.min(armor.armor_points);
                armor.take_damage(absorbed);
                armor_damage = armor_damage.saturating_sub(absorbed);
            }
        }
        
        // Apply shield damage if attacking from front (DV1)
        if armor_damage > 0 {
            if let Some(shield) = &mut self.shield {
                if !shield.is_destroyed() {
                    let absorbed = armor_damage.min(shield.armor_points);
                    shield.take_damage(absorbed);
                    armor_damage = armor_damage.saturating_sub(absorbed);
                }
            }
        }
        
        // Any remaining damage becomes actual damage
        let total_actual_damage = actual_damage + armor_damage;
        
        // Apply actual damage to hit points
        self.combat_stats.hit_points.current = 
            self.combat_stats.hit_points.current.saturating_sub(total_actual_damage);
            
        (total_actual_damage, damage.saturating_sub(total_actual_damage))
    }

    pub fn heal(&mut self, amount: u32) {
        self.combat_stats.hit_points.current = 
            (self.combat_stats.hit_points.current + amount)
                .min(self.combat_stats.hit_points.max);
    }
}

impl CombatEncounter {
    pub fn new(mut participants: Vec<CombatParticipant>) -> Self {
        // Roll initiative for all participants
        for participant in &mut participants {
            participant.roll_initiative();
        }
        
        // Sort by initiative (highest first)
        participants.sort_by(|a, b| b.initiative.cmp(&a.initiative));
        
        CombatEncounter {
            participants,
            current_turn: 0,
            round: 1,
            combat_log: Vec::new(),
        }
    }

    pub fn add_log(&mut self, message: String) {
        self.combat_log.push(format!("[Round {}] {}", self.round, message));
    }

    pub fn get_current_participant(&self) -> Option<&CombatParticipant> {
        self.participants.get(self.current_turn)
    }

    pub fn get_current_participant_mut(&mut self) -> Option<&mut CombatParticipant> {
        self.participants.get_mut(self.current_turn)
    }

    pub fn perform_action(&mut self, action: CombatAction) -> CombatResult {
        let attacker_index = self.current_turn;
        
        match action {
            CombatAction::Attack { target_index } => {
                self.perform_attack(attacker_index, target_index)
            }
            CombatAction::Defend => {
                self.add_log(format!("{} takes a defensive stance!", 
                    self.participants[attacker_index].name));
                CombatResult {
                    success: true,
                    damage: None,
                    message: "Defending".to_string(),
                    critical: false,
                }
            }
            CombatAction::Flee => {
                let mut rng = rand::thread_rng();
                let flee_chance = rng.gen_range(1..=20);
                if flee_chance >= 10 {
                    self.add_log(format!("{} flees from combat!", 
                        self.participants[attacker_index].name));
                    CombatResult {
                        success: true,
                        damage: None,
                        message: "Fled successfully".to_string(),
                        critical: false,
                    }
                } else {
                    self.add_log(format!("{} fails to flee!", 
                        self.participants[attacker_index].name));
                    CombatResult {
                        success: false,
                        damage: None,
                        message: "Failed to flee".to_string(),
                        critical: false,
                    }
                }
            }
            CombatAction::UseItem { item } => {
                let participant_name = self.participants[attacker_index].name.clone();
                
                // Handle different item types
                match item.as_str() {
                    "Health Potion" => {
                        let heal_amount = 10; // Basic health potion heals 10 HP
                        self.participants[attacker_index].heal(heal_amount);
                        self.add_log(format!("{} drinks a health potion and recovers {} HP!", 
                            participant_name, heal_amount));
                        CombatResult {
                            success: true,
                            damage: None,
                            message: format!("Healed {} HP", heal_amount),
                            critical: false,
                        }
                    }
                    _ => {
                        self.add_log(format!("{} uses {} (no effect)", 
                            participant_name, item));
                        CombatResult {
                            success: false,
                            damage: None,
                            message: format!("Used {} (no effect)", item),
                            critical: false,
                        }
                    }
                }
            }
            CombatAction::CastSpell { spell_name, target_index: _ } => {
                // For now, return a placeholder - we'll implement spell casting in the game layer
                let participant_name = self.participants[attacker_index].name.clone();
                self.add_log(format!("{} attempts to cast {}!", participant_name, spell_name));
                CombatResult {
                    success: false,
                    damage: None,
                    message: "Spell casting not yet implemented at this layer".to_string(),
                    critical: false,
                }
            }
        }
    }

    fn perform_attack(&mut self, attacker_index: usize, target_index: usize) -> CombatResult {
        let mut rng = rand::thread_rng();
        
        // Get attack and defense values
        let attack_value = self.participants[attacker_index].get_total_attack_value();
        let defense_value = self.participants[target_index].get_total_defense_value();
        
        // Roll attack (1d20 + attack value vs defense value)
        let attack_roll = rng.gen_range(1..=20);
        let total_attack = attack_roll + attack_value;
        
        let attacker_name = self.participants[attacker_index].name.clone();
        let target_name = self.participants[target_index].name.clone();
        
        // Check for critical hit (natural 20)
        let critical = attack_roll == 20;
        
        // Check for hit
        if total_attack > defense_value || critical {
            // Roll damage
            let weapon = self.participants[attacker_index].weapon.clone()
                .unwrap_or_else(Weapon::unarmed);
            let (mut damage, dice_count) = weapon.roll_damage();
            
            // Add damage bonus from character
            let damage_bonus = self.participants[attacker_index].get_total_damage_bonus();
            if damage_bonus >= 0 {
                damage += damage_bonus as u32;
            } else {
                damage = damage.saturating_sub(damage_bonus.abs() as u32);
            }
            
            // Double damage on critical (including dice count for actual damage)
            let final_dice_count = if critical { dice_count * 2 } else { dice_count };
            if critical {
                damage *= 2;
            }
            
            // Apply damage using Forge rules
            let (actual_damage, armor_damage) = self.participants[target_index].take_damage(damage, final_dice_count);
            
            let message = if critical {
                format!("{} critically hits {} with {} for {} damage ({} actual, {} absorbed)!", 
                    attacker_name, target_name, weapon.name, damage, actual_damage, armor_damage)
            } else {
                format!("{} hits {} with {} for {} damage ({} actual, {} absorbed)!", 
                    attacker_name, target_name, weapon.name, damage, actual_damage, armor_damage)
            };
            
            self.add_log(message.clone());
            
            // Check if target is defeated
            if !self.participants[target_index].is_alive() {
                self.add_log(format!("{} has been defeated!", target_name));
            }
            
            CombatResult {
                success: true,
                damage: Some(damage),
                message,
                critical,
            }
        } else {
            let message = format!("{} misses {}!", attacker_name, target_name);
            self.add_log(message.clone());
            
            CombatResult {
                success: false,
                damage: None,
                message,
                critical: false,
            }
        }
    }

    pub fn next_turn(&mut self) {
        // Find next alive participant
        let start_turn = self.current_turn;
        loop {
            self.current_turn = (self.current_turn + 1) % self.participants.len();
            
            // If we've gone through all participants, increment round
            if self.current_turn == 0 {
                self.round += 1;
            }
            
            // If current participant is alive, break
            if self.participants[self.current_turn].is_alive() {
                break;
            }
            
            // If we've checked all participants and none are alive, combat is over
            if self.current_turn == start_turn {
                break;
            }
        }
    }

    pub fn is_combat_over(&self) -> bool {
        let alive_players = self.participants.iter()
            .filter(|p| p.is_player && p.is_alive())
            .count();
        let alive_enemies = self.participants.iter()
            .filter(|p| !p.is_player && p.is_alive())
            .count();
        
        alive_players == 0 || alive_enemies == 0
    }

    pub fn get_winner(&self) -> Option<String> {
        if !self.is_combat_over() {
            return None;
        }
        
        let alive_players = self.participants.iter()
            .filter(|p| p.is_player && p.is_alive())
            .count();
        
        if alive_players > 0 {
            Some("Player".to_string())
        } else {
            Some("Enemies".to_string())
        }
    }
}

impl Armor {
    pub fn leather() -> Self {
        Armor {
            name: "Leather Armor".to_string(),
            armor_type: ArmorType::Light,
            armor_rating: 2,
            armor_points: 20,
            max_armor_points: 20,
            penalty: 0,
        }
    }
    
    pub fn studded_leather() -> Self {
        Armor {
            name: "Studded Leather".to_string(),
            armor_type: ArmorType::Light,
            armor_rating: 3,
            armor_points: 30,
            max_armor_points: 30,
            penalty: -1,
        }
    }
    
    pub fn ring_mail() -> Self {
        Armor {
            name: "Ring Mail".to_string(),
            armor_type: ArmorType::Medium,
            armor_rating: 4,
            armor_points: 40,
            max_armor_points: 40,
            penalty: -1,
        }
    }
    
    pub fn chain_mail() -> Self {
        Armor {
            name: "Chain Mail".to_string(),
            armor_type: ArmorType::Medium,
            armor_rating: 5,
            armor_points: 50,
            max_armor_points: 50,
            penalty: -2,
        }
    }
    
    pub fn banded_mail() -> Self {
        Armor {
            name: "Banded Mail".to_string(),
            armor_type: ArmorType::Heavy,
            armor_rating: 6,
            armor_points: 60,
            max_armor_points: 60,
            penalty: -3,
        }
    }
    
    pub fn plate_mail() -> Self {
        Armor {
            name: "Plate Mail".to_string(),
            armor_type: ArmorType::Heavy,
            armor_rating: 7,
            armor_points: 70,
            max_armor_points: 70,
            penalty: -4,
        }
    }
    
    pub fn small_shield() -> Self {
        Armor {
            name: "Small Shield".to_string(),
            armor_type: ArmorType::Shield,
            armor_rating: 1,
            armor_points: 10,
            max_armor_points: 10,
            penalty: 0,
        }
    }
    
    pub fn medium_shield() -> Self {
        Armor {
            name: "Medium Shield".to_string(),
            armor_type: ArmorType::Shield,
            armor_rating: 2,
            armor_points: 20,
            max_armor_points: 20,
            penalty: -1,
        }
    }
    
    pub fn large_shield() -> Self {
        Armor {
            name: "Large Shield".to_string(),
            armor_type: ArmorType::Shield,
            armor_rating: 3,
            armor_points: 30,
            max_armor_points: 30,
            penalty: -2,
        }
    }
}

pub fn create_wild_boar() -> CombatParticipant {
    CombatParticipant::create_enemy(
        "Wild Boar",
        15,  // HP
        6,   // Attack
        4,   // Defense
        Some(Weapon {
            name: "Tusks".to_string(),
            weapon_type: WeaponType::Unarmed,
            damage_dice: "1d4".to_string(),
            damage_type: DamageType::Piercing,
            damage_bonus: 1,
            attack_bonus: 0,
            two_handed: false,
            ranged: false,
            range: None,
        })
    )
}

pub fn create_wolf() -> CombatParticipant {
    CombatParticipant::create_enemy(
        "Wolf",
        12,  // HP
        8,   // Attack
        6,   // Defense
        Some(Weapon {
            name: "Bite".to_string(),
            weapon_type: WeaponType::Unarmed,
            damage_dice: "1d6".to_string(),
            damage_type: DamageType::Piercing,
            damage_bonus: 0,
            attack_bonus: 2,
            two_handed: false,
            ranged: false,
            range: None,
        })
    )
}

pub fn create_goblin() -> CombatParticipant {
    let mut goblin = CombatParticipant::create_enemy(
        "Goblin",
        8,   // HP
        5,   // Attack
        5,   // Defense
        Some(Weapon {
            name: "Crude Sword".to_string(),
            weapon_type: WeaponType::Sword,
            damage_dice: "1d6".to_string(),
            damage_type: DamageType::Slashing,
            damage_bonus: -1,
            attack_bonus: 0,
            two_handed: false,
            ranged: false,
            range: None,
        })
    );
    goblin.armor = Some(Armor::leather());
    goblin
}

pub fn create_bandit() -> CombatParticipant {
    let mut bandit = CombatParticipant::create_enemy(
        "Bandit",
        18,  // HP
        7,   // Attack
        6,   // Defense
        Some(Weapon {
            name: "Short Sword".to_string(),
            weapon_type: WeaponType::Sword,
            damage_dice: "1d6".to_string(),
            damage_type: DamageType::Slashing,
            damage_bonus: 1,
            attack_bonus: 1,
            two_handed: false,
            ranged: false,
            range: None,
        })
    );
    bandit.armor = Some(Armor::studded_leather());
    bandit.shield = Some(Armor::small_shield());
    bandit
}

pub fn create_orc() -> CombatParticipant {
    let mut orc = CombatParticipant::create_enemy(
        "Orc",
        25,  // HP
        9,   // Attack
        7,   // Defense
        Some(Weapon {
            name: "Battle Axe".to_string(),
            weapon_type: WeaponType::Axe,
            damage_dice: "1d8".to_string(),
            damage_type: DamageType::Slashing,
            damage_bonus: 2,
            attack_bonus: 1,
            two_handed: false,
            ranged: false,
            range: None,
        })
    );
    orc.armor = Some(Armor::chain_mail());
    orc
}

pub fn create_giant_spider() -> CombatParticipant {
    CombatParticipant::create_enemy(
        "Giant Spider",
        10,  // HP
        7,   // Attack
        8,   // Defense
        Some(Weapon {
            name: "Venomous Bite".to_string(),
            weapon_type: WeaponType::Unarmed,
            damage_dice: "1d4".to_string(),
            damage_type: DamageType::Piercing,
            damage_bonus: 0,
            attack_bonus: 1,
            two_handed: false,
            ranged: false,
            range: None,
        })
    )
}

pub fn create_mountain_lion() -> CombatParticipant {
    CombatParticipant::create_enemy(
        "Mountain Lion",
        20,  // HP
        10,  // Attack
        8,   // Defense
        Some(Weapon {
            name: "Claws and Teeth".to_string(),
            weapon_type: WeaponType::Unarmed,
            damage_dice: "1d6".to_string(),
            damage_type: DamageType::Slashing,
            damage_bonus: 2,
            attack_bonus: 2,
            two_handed: false,
            ranged: false,
            range: None,
        })
    )
}

pub fn create_skeleton() -> CombatParticipant {
    let mut skeleton = CombatParticipant::create_enemy(
        "Skeleton",
        14,  // HP
        6,   // Attack
        5,   // Defense
        Some(Weapon {
            name: "Rusty Sword".to_string(),
            weapon_type: WeaponType::Sword,
            damage_dice: "1d6".to_string(),
            damage_type: DamageType::Slashing,
            damage_bonus: 0,
            attack_bonus: 0,
            two_handed: false,
            ranged: false,
            range: None,
        })
    );
    skeleton.armor = Some(Armor {
        name: "Bone Armor".to_string(),
        armor_type: ArmorType::Light,
        armor_rating: 1,
        armor_points: 10,
        max_armor_points: 10,
        penalty: 0,
    });
    skeleton
}

pub fn create_zombie() -> CombatParticipant {
    CombatParticipant::create_enemy(
        "Zombie",
        18,  // HP - zombies are tough
        4,   // Attack - slow but dangerous
        3,   // Defense - shambling, easy to hit
        Some(Weapon {
            name: "Rotting Claws".to_string(),
            weapon_type: WeaponType::Unarmed,
            damage_dice: "1d4".to_string(),
            damage_type: DamageType::Slashing,
            damage_bonus: 1,
            attack_bonus: 0,
            two_handed: false,
            ranged: false,
            range: None,
        })
    )
}
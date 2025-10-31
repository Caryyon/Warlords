use serde::{Deserialize, Serialize};
use rand::Rng;
use super::{ForgeCharacter, CombatStats};

// AI Behavior System
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIBehaviorType {
    Aggressive,   // Prefers attacking, takes risks
    Defensive,    // Prefers defending, plays safe
    Balanced,     // Mix of offensive and defensive
    Tactical,     // Uses positioning and environment
    Support,      // Focuses on healing and buffs
    Berserker,    // All-out attack, ignores defense
    Coward,       // Tries to flee when low on health
    Spellcaster,  // Prefers magical attacks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPersonality {
    pub behavior_type: AIBehaviorType,
    pub aggression: u8,        // 0-10, how likely to attack vs defend
    pub intelligence: u8,      // 0-10, tactical awareness
    pub self_preservation: u8, // 0-10, how much they care about survival
    pub spell_preference: u8,  // 0-10, how much they prefer magic
    pub ranged_preference: u8, // 0-10, how much they prefer ranged combat
    pub health_threshold: u8,  // 0-100, health % below which they become desperate
}

impl Default for AIPersonality {
    fn default() -> Self {
        Self {
            behavior_type: AIBehaviorType::Balanced,
            aggression: 5,
            intelligence: 5,
            self_preservation: 5,
            spell_preference: 3,
            ranged_preference: 3,
            health_threshold: 25,
        }
    }
}

impl AIPersonality {
    pub fn aggressive() -> Self {
        Self {
            behavior_type: AIBehaviorType::Aggressive,
            aggression: 8,
            intelligence: 4,
            self_preservation: 2,
            spell_preference: 2,
            ranged_preference: 2,
            health_threshold: 15,
        }
    }
    
    pub fn defensive() -> Self {
        Self {
            behavior_type: AIBehaviorType::Defensive,
            aggression: 2,
            intelligence: 6,
            self_preservation: 8,
            spell_preference: 4,
            ranged_preference: 6,
            health_threshold: 40,
        }
    }
    
    pub fn tactical() -> Self {
        Self {
            behavior_type: AIBehaviorType::Tactical,
            aggression: 5,
            intelligence: 8,
            self_preservation: 6,
            spell_preference: 5,
            ranged_preference: 7,
            health_threshold: 30,
        }
    }
    
    pub fn berserker() -> Self {
        Self {
            behavior_type: AIBehaviorType::Berserker,
            aggression: 10,
            intelligence: 2,
            self_preservation: 1,
            spell_preference: 1,
            ranged_preference: 1,
            health_threshold: 5,
        }
    }
    
    pub fn spellcaster() -> Self {
        Self {
            behavior_type: AIBehaviorType::Spellcaster,
            aggression: 4,
            intelligence: 7,
            self_preservation: 6,
            spell_preference: 9,
            ranged_preference: 8,
            health_threshold: 35,
        }
    }
    
    pub fn coward() -> Self {
        Self {
            behavior_type: AIBehaviorType::Coward,
            aggression: 1,
            intelligence: 5,
            self_preservation: 10,
            spell_preference: 3,
            ranged_preference: 8,
            health_threshold: 60,
        }
    }
    
    pub fn balanced() -> Self {
        Self {
            behavior_type: AIBehaviorType::Balanced,
            aggression: 5,
            intelligence: 5,
            self_preservation: 5,
            spell_preference: 3,
            ranged_preference: 4,
            health_threshold: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DamageType {
    Slashing,
    Piercing,
    Bludgeoning,
    Magic,
}

impl std::fmt::Display for DamageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DamageType::Slashing => write!(f, "slashing"),
            DamageType::Piercing => write!(f, "piercing"),
            DamageType::Bludgeoning => write!(f, "bludgeoning"),
            DamageType::Magic => write!(f, "magic"),
        }
    }
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
    pub ai_personality: Option<AIPersonality>,
    pub magic: super::magic::MagicSystem,
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
    
    pub fn iron_sword() -> Self {
        Weapon {
            name: "Iron Sword".to_string(),
            weapon_type: WeaponType::Sword,
            damage_dice: "1d8".to_string(),
            damage_type: DamageType::Slashing,
            damage_bonus: 1,
            attack_bonus: 1,
            two_handed: false,
            ranged: false,
            range: None,
        }
    }
    
    pub fn steel_sword() -> Self {
        Weapon {
            name: "Steel Sword".to_string(),
            weapon_type: WeaponType::Sword,
            damage_dice: "1d8".to_string(),
            damage_type: DamageType::Slashing,
            damage_bonus: 2,
            attack_bonus: 2,
            two_handed: false,
            ranged: false,
            range: None,
        }
    }
    
    pub fn war_axe() -> Self {
        Weapon {
            name: "War Axe".to_string(),
            weapon_type: WeaponType::Axe,
            damage_dice: "1d10".to_string(),
            damage_type: DamageType::Slashing,
            damage_bonus: 0,
            attack_bonus: 0,
            two_handed: true,
            ranged: false,
            range: None,
        }
    }
    
    pub fn iron_mace() -> Self {
        Weapon {
            name: "Iron Mace".to_string(),
            weapon_type: WeaponType::Mace,
            damage_dice: "1d6".to_string(),
            damage_type: DamageType::Bludgeoning,
            damage_bonus: 1,
            attack_bonus: 0,
            two_handed: false,
            ranged: false,
            range: None,
        }
    }
    
    pub fn dagger() -> Self {
        Weapon {
            name: "Dagger".to_string(),
            weapon_type: WeaponType::Dagger,
            damage_dice: "1d4".to_string(),
            damage_type: DamageType::Piercing,
            damage_bonus: 0,
            attack_bonus: 2,
            two_handed: false,
            ranged: false,
            range: None,
        }
    }
    
    pub fn short_bow() -> Self {
        Weapon {
            name: "Short Bow".to_string(),
            weapon_type: WeaponType::Bow,
            damage_dice: "1d6".to_string(),
            damage_type: DamageType::Piercing,
            damage_bonus: 0,
            attack_bonus: 1,
            two_handed: true,
            ranged: true,
            range: Some(150),
        }
    }
    
    pub fn crossbow() -> Self {
        Weapon {
            name: "Crossbow".to_string(),
            weapon_type: WeaponType::Crossbow,
            damage_dice: "1d8".to_string(),
            damage_type: DamageType::Piercing,
            damage_bonus: 1,
            attack_bonus: 0,
            two_handed: true,
            ranged: true,
            range: Some(120),
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
        // Use equipped items from character, with optional weapon override
        let equipped_weapon = weapon.or_else(|| character.equipment.weapon.clone());
        
        CombatParticipant {
            name: character.name.clone(),
            combat_stats: character.combat_stats.clone(),
            weapon: equipped_weapon,
            armor: character.equipment.armor.clone(),
            shield: character.equipment.shield.clone(),
            initiative: 0,
            is_player: true,
            ai_personality: None, // Players don't have AI personalities
            magic: character.magic.clone(),
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
            ai_personality: Some(AIPersonality::default()), // Default AI for enemies
            magic: super::magic::MagicSystem::new(5), // Basic magic system for enemies
        }
    }
    
    pub fn create_enemy_with_ai(name: &str, hp: u32, attack: u8, defense: u8, weapon: Option<Weapon>, ai_personality: AIPersonality) -> Self {
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
            ai_personality: Some(ai_personality),
            magic: super::magic::MagicSystem::new(5), // Basic magic system for enemies
        }
    }
    
    pub fn get_health_percentage(&self) -> u8 {
        if self.combat_stats.hit_points.max == 0 {
            0
        } else {
            ((self.combat_stats.hit_points.current as f32 / self.combat_stats.hit_points.max as f32) * 100.0) as u8
        }
    }
    
    pub fn is_desperate(&self) -> bool {
        if let Some(personality) = &self.ai_personality {
            self.get_health_percentage() <= personality.health_threshold
        } else {
            false
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
    
    // Enhanced AI decision-making system
    pub fn make_ai_decision(&self, ai_index: usize) -> CombatAction {
        if let Some(ai_participant) = self.participants.get(ai_index) {
            if let Some(personality) = &ai_participant.ai_personality {
                self.make_personality_based_decision(ai_index, personality)
            } else {
                self.make_basic_ai_decision(ai_index)
            }
        } else {
            CombatAction::Defend
        }
    }
    
    fn make_personality_based_decision(&self, ai_index: usize, personality: &AIPersonality) -> CombatAction {
        let mut rng = rand::thread_rng();
        let ai_participant = &self.participants[ai_index];
        
        // Check if AI is desperate (low health)
        let is_desperate = ai_participant.is_desperate();
        
        // Find potential targets
        let targets = self.find_potential_targets(ai_index);
        if targets.is_empty() {
            return CombatAction::Defend;
        }
        
        // Adjust behavior based on personality and health status
        let effective_aggression = if is_desperate {
            match personality.behavior_type {
                AIBehaviorType::Coward => 0,
                AIBehaviorType::Berserker => 10,
                _ => personality.aggression.saturating_sub(personality.self_preservation),
            }
        } else {
            personality.aggression
        };
        
        // Decision making based on personality
        match personality.behavior_type {
            AIBehaviorType::Aggressive | AIBehaviorType::Berserker => {
                // Always try to attack the most vulnerable target
                if let Some(target) = self.find_most_vulnerable_target(&targets) {
                    CombatAction::Attack { target_index: target }
                } else {
                    CombatAction::Attack { target_index: targets[0] }
                }
            }
            AIBehaviorType::Defensive => {
                // Defend more often, only attack when safe
                let defend_chance = if is_desperate { 30 } else { 60 };
                if rng.gen_range(1..=100) <= defend_chance {
                    CombatAction::Defend
                } else {
                    // Attack the weakest target
                    if let Some(target) = self.find_weakest_target(&targets) {
                        CombatAction::Attack { target_index: target }
                    } else {
                        CombatAction::Defend
                    }
                }
            }
            AIBehaviorType::Coward => {
                if is_desperate || ai_participant.get_health_percentage() < 50 {
                    CombatAction::Flee
                } else if rng.gen_range(1..=10) <= 3 {
                    // Cowards occasionally attack when feeling safe
                    if let Some(target) = self.find_weakest_target(&targets) {
                        CombatAction::Attack { target_index: target }
                    } else {
                        CombatAction::Defend
                    }
                } else {
                    CombatAction::Defend
                }
            }
            AIBehaviorType::Tactical => {
                // Tactical AI considers threat levels and positioning
                if let Some(target) = self.find_tactical_target(&targets, personality.intelligence) {
                    CombatAction::Attack { target_index: target }
                } else if rng.gen_range(1..=10) <= 4 {
                    CombatAction::Defend
                } else {
                    CombatAction::Attack { target_index: targets[0] }
                }
            }
            AIBehaviorType::Balanced => {
                // Balanced approach based on aggression level
                if rng.gen_range(1..=10) <= effective_aggression {
                    // Attack
                    if let Some(target) = self.find_best_target(&targets) {
                        CombatAction::Attack { target_index: target }
                    } else {
                        CombatAction::Attack { target_index: targets[0] }
                    }
                } else {
                    CombatAction::Defend
                }
            }
            AIBehaviorType::Support => {
                // Support AI would help allies, but in basic combat just defends
                if rng.gen_range(1..=10) <= 3 {
                    if let Some(target) = self.find_weakest_target(&targets) {
                        CombatAction::Attack { target_index: target }
                    } else {
                        CombatAction::Defend
                    }
                } else {
                    CombatAction::Defend
                }
            }
            AIBehaviorType::Spellcaster => {
                // Prefer magic attacks when available, otherwise defend
                // For now, just attack since spell system is separate
                if rng.gen_range(1..=10) <= 6 {
                    if let Some(target) = self.find_best_target(&targets) {
                        CombatAction::Attack { target_index: target }
                    } else {
                        CombatAction::Attack { target_index: targets[0] }
                    }
                } else {
                    CombatAction::Defend
                }
            }
        }
    }
    
    fn make_basic_ai_decision(&self, ai_index: usize) -> CombatAction {
        // Fallback to simple AI behavior
        let targets = self.find_potential_targets(ai_index);
        if !targets.is_empty() {
            CombatAction::Attack { target_index: targets[0] }
        } else {
            CombatAction::Defend
        }
    }
    
    fn find_potential_targets(&self, ai_index: usize) -> Vec<usize> {
        let ai_participant = &self.participants[ai_index];
        self.participants
            .iter()
            .enumerate()
            .filter(|(i, p)| *i != ai_index && p.is_player != ai_participant.is_player && p.is_alive())
            .map(|(i, _)| i)
            .collect()
    }
    
    fn find_most_vulnerable_target(&self, targets: &[usize]) -> Option<usize> {
        targets.iter()
            .min_by_key(|&&i| {
                let participant = &self.participants[i];
                participant.get_total_defense_value() + (participant.get_health_percentage() / 10)
            })
            .copied()
    }
    
    fn find_weakest_target(&self, targets: &[usize]) -> Option<usize> {
        targets.iter()
            .min_by_key(|&&i| self.participants[i].combat_stats.hit_points.current)
            .copied()
    }
    
    fn find_tactical_target(&self, targets: &[usize], intelligence: u8) -> Option<usize> {
        // Higher intelligence considers more factors
        if intelligence >= 7 {
            // Smart AI targets based on threat level and vulnerability
            targets.iter()
                .min_by_key(|&&i| {
                    let participant = &self.participants[i];
                    let threat_level = participant.get_total_attack_value() as u32;
                    let defense = participant.get_total_defense_value() as u32;
                    let health_pct = participant.get_health_percentage() as u32;
                    
                    // Score: higher threat + lower defense + lower health = better target
                    (defense * health_pct) / (threat_level + 1)
                })
                .copied()
        } else {
            self.find_most_vulnerable_target(targets)
        }
    }
    
    fn find_best_target(&self, targets: &[usize]) -> Option<usize> {
        // General purpose "best" target selection
        targets.iter()
            .min_by_key(|&&i| {
                let participant = &self.participants[i];
                // Simple scoring: prefer low health, low defense targets
                participant.combat_stats.hit_points.current + (participant.get_total_defense_value() as u32 * 2)
            })
            .copied()
    }
}

// Tactical Combat System
// =====================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BattlefieldPosition {
    pub x: i32,
    pub y: i32,
}

impl BattlefieldPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    pub fn distance_to(&self, other: &BattlefieldPosition) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
    
    pub fn manhattan_distance_to(&self, other: &BattlefieldPosition) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
    
    pub fn is_adjacent_to(&self, other: &BattlefieldPosition) -> bool {
        self.manhattan_distance_to(other) == 1
    }
    
    pub fn get_adjacent_positions(&self) -> Vec<BattlefieldPosition> {
        vec![
            BattlefieldPosition::new(self.x, self.y - 1), // North
            BattlefieldPosition::new(self.x + 1, self.y), // East
            BattlefieldPosition::new(self.x, self.y + 1), // South
            BattlefieldPosition::new(self.x - 1, self.y), // West
        ]
    }
    
    pub fn get_line_to(&self, target: &BattlefieldPosition) -> Vec<BattlefieldPosition> {
        let mut line = Vec::new();
        let dx = target.x - self.x;
        let dy = target.y - self.y;
        let steps = dx.abs().max(dy.abs());
        
        if steps == 0 {
            return vec![*self];
        }
        
        for i in 0..=steps {
            let x = self.x + (dx * i / steps);
            let y = self.y + (dy * i / steps);
            line.push(BattlefieldPosition::new(x, y));
        }
        
        line
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerrainFeature {
    Open,                    // No obstruction
    Obstacle,               // Blocks movement and line of sight
    DifficultTerrain,       // Costs extra movement
    Cover,                  // Provides defensive bonus
    Hazard,                 // Damages units that enter
    Elevation,              // Higher ground, tactical advantage
    Water,                  // May block some units, affect spells
    Altar,                  // Interactive environmental element
    Pillar,                 // Partial cover, blocks movement
    Pit,                    // Damages units, may trap them
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlefieldTile {
    pub position: BattlefieldPosition,
    pub terrain: TerrainFeature,
    pub movement_cost: u32,     // 1 = normal, 2+ = difficult terrain
    pub blocks_movement: bool,
    pub blocks_line_of_sight: bool,
    pub cover_bonus: i8,        // Defensive bonus for units in this tile
    pub on_enter_effect: Option<String>, // Effect when unit enters (damage, buffs, etc.)
}

impl BattlefieldTile {
    pub fn new(position: BattlefieldPosition, terrain: TerrainFeature) -> Self {
        let (movement_cost, blocks_movement, blocks_line_of_sight, cover_bonus, on_enter_effect) = 
            match terrain {
                TerrainFeature::Open => (1, false, false, 0, None),
                TerrainFeature::Obstacle => (1, true, true, 0, None),
                TerrainFeature::DifficultTerrain => (2, false, false, 0, None),
                TerrainFeature::Cover => (1, false, false, 2, None),
                TerrainFeature::Hazard => (1, false, false, 0, Some("1d4 fire damage".to_string())),
                TerrainFeature::Elevation => (1, false, false, 1, None),
                TerrainFeature::Water => (2, false, false, 0, None),
                TerrainFeature::Altar => (1, true, false, 0, Some("heal 1d4".to_string())),
                TerrainFeature::Pillar => (1, true, true, 3, None),
                TerrainFeature::Pit => (1, false, false, 0, Some("1d6 fall damage".to_string())),
            };
        
        Self {
            position,
            terrain,
            movement_cost,
            blocks_movement,
            blocks_line_of_sight,
            cover_bonus,
            on_enter_effect,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalBattlefield {
    pub width: u32,
    pub height: u32,
    pub tiles: std::collections::HashMap<BattlefieldPosition, BattlefieldTile>,
    pub participant_positions: std::collections::HashMap<usize, BattlefieldPosition>, // participant_id -> position
    pub environmental_features: Vec<EnvironmentalFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalFeature {
    pub name: String,
    pub position: BattlefieldPosition,
    pub feature_type: EnvironmentalFeatureType,
    pub activatable: bool,
    pub activation_effect: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentalFeatureType {
    Lever,          // Activates traps, opens doors
    Trap,           // Triggered by movement or activation
    MagicCircle,    // Enhances spells cast from it
    Brazier,        // Can be lit for area effects
    Statue,         // May have hidden mechanisms
    Well,           // Source of water, may have effects
    Portal,         // Transportation between points
}

impl TacticalBattlefield {
    // Enhanced AI utility functions for tactical combat
    pub fn find_best_tactical_position(&self, ai_id: usize, participants: &[TacticalCombatParticipant], 
                                       movement_speed: u32) -> Option<BattlefieldPosition> {
        if let Some(ai_pos) = self.get_participant_position(ai_id) {
            let mut best_position = ai_pos;
            let mut best_score = self.evaluate_position(&ai_pos, ai_id, participants);
            
            // Check all positions within movement range
            for x in (ai_pos.x - movement_speed as i32)..=(ai_pos.x + movement_speed as i32) {
                for y in (ai_pos.y - movement_speed as i32)..=(ai_pos.y + movement_speed as i32) {
                    let candidate_pos = BattlefieldPosition::new(x, y);
                    
                    if self.is_position_passable(&candidate_pos) {
                        let distance_cost = ai_pos.manhattan_distance_to(&candidate_pos) as u32;
                        if distance_cost <= movement_speed {
                            let score = self.evaluate_position(&candidate_pos, ai_id, participants);
                            if score > best_score {
                                best_score = score;
                                best_position = candidate_pos;
                            }
                        }
                    }
                }
            }
            
            if best_position != ai_pos {
                Some(best_position)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn evaluate_position(&self, pos: &BattlefieldPosition, ai_id: usize, participants: &[TacticalCombatParticipant]) -> i32 {
        let mut score = 0i32;
        
        if let Some(ai_participant) = participants.get(ai_id) {
            let ai_personality = &ai_participant.base_participant.ai_personality;
            
            // Base score from terrain benefits
            score += self.get_cover_bonus(pos) as i32 * 5;
            
            // Distance considerations
            for (id, participant) in participants.iter().enumerate() {
                if id != ai_id && participant.base_participant.is_alive() {
                    let distance = pos.manhattan_distance_to(&participant.position);
                    
                    if participant.base_participant.is_player {
                        // Scoring based on AI personality
                        if let Some(personality) = ai_personality {
                            match personality.behavior_type {
                                AIBehaviorType::Aggressive | AIBehaviorType::Berserker => {
                                    // Prefer being close to enemies
                                    score += (10 - distance.min(10)) * 3;
                                }
                                AIBehaviorType::Defensive | AIBehaviorType::Coward => {
                                    // Prefer distance from enemies, especially if low health
                                    let health_pct = ai_participant.base_participant.get_health_percentage();
                                    if health_pct < personality.health_threshold {
                                        score += distance * 5; // Really want distance when hurt
                                    } else {
                                        score += distance * 2;
                                    }
                                }
                                AIBehaviorType::Tactical => {
                                    // Prefer optimal attack range
                                    let optimal_range = if let Some(weapon) = &ai_participant.base_participant.weapon {
                                        if weapon.ranged { 3 } else { 1 }
                                    } else { 1 };
                                    
                                    let range_diff = (distance - optimal_range).abs();
                                    score += (5 - range_diff.min(5)) * 3;
                                }
                                _ => {
                                    // Default: prefer medium range
                                    score += (7 - distance.min(7)) * 2;
                                }
                            }
                        }
                    }
                }
            }
            
            // Line of sight considerations
            let enemies_in_sight = participants.iter()
                .filter(|p| p.base_participant.is_player && p.base_participant.is_alive())
                .filter(|p| self.has_line_of_sight(pos, &p.position))
                .count();
            
            if let Some(personality) = ai_personality {
                match personality.behavior_type {
                    AIBehaviorType::Aggressive | AIBehaviorType::Berserker => {
                        score += enemies_in_sight as i32 * 3; // Want to see enemies
                    }
                    AIBehaviorType::Defensive | AIBehaviorType::Coward => {
                        score -= enemies_in_sight as i32 * 2; // Don't want to be seen
                    }
                    _ => {
                        score += enemies_in_sight as i32; // Moderate preference
                    }
                }
            }
        }
        
        score
    }
    
    pub fn find_best_attack_target(&self, ai_id: usize, participants: &[TacticalCombatParticipant]) -> Option<usize> {
        if let Some(ai_participant) = participants.get(ai_id) {
            let ai_pos = ai_participant.position;
            let ai_personality = &ai_participant.base_participant.ai_personality;
            
            let mut best_target: Option<usize> = None;
            let mut best_score = 0i32;
            
            for (id, participant) in participants.iter().enumerate() {
                if id != ai_id && participant.base_participant.is_player && participant.base_participant.is_alive() {
                    let target_pos = participant.position;
                    let distance = ai_pos.manhattan_distance_to(&target_pos);
                    
                    // Check if target is in range
                    let max_range = if let Some(weapon) = &ai_participant.base_participant.weapon {
                        if weapon.ranged { 
                            weapon.range.unwrap_or(5) / 5 // Convert feet to tiles
                        } else { 
                            1 
                        }
                    } else { 
                        1 
                    };
                    
                    if distance <= max_range as i32 {
                        let mut score = 100; // Base score for being in range
                        
                        // Personality-based target selection
                        if let Some(personality) = ai_personality {
                            match personality.behavior_type {
                                AIBehaviorType::Tactical => {
                                    // Smart targeting: threat level vs vulnerability
                                    let threat = participant.base_participant.get_total_attack_value() as i32;
                                    let vulnerability = 100 - participant.base_participant.get_health_percentage() as i32;
                                    let defense = participant.base_participant.get_total_defense_value() as i32;
                                    
                                    score += threat * 2; // Higher priority for dangerous targets
                                    score += vulnerability; // Prefer wounded targets
                                    score -= defense; // Prefer low-defense targets
                                }
                                AIBehaviorType::Aggressive | AIBehaviorType::Berserker => {
                                    // Attack strongest/closest target
                                    let threat = participant.base_participant.get_total_attack_value() as i32;
                                    score += threat * 3;
                                    score += (10 - distance.min(10)) * 2; // Prefer close targets
                                }
                                AIBehaviorType::Coward => {
                                    // Attack weakest target
                                    let vulnerability = 100 - participant.base_participant.get_health_percentage() as i32;
                                    score += vulnerability * 3;
                                    score -= participant.base_participant.get_total_attack_value() as i32; // Avoid dangerous foes
                                }
                                _ => {
                                    // Balanced approach
                                    let vulnerability = 100 - participant.base_participant.get_health_percentage() as i32;
                                    score += vulnerability;
                                    score += 5 - distance.min(5); // Slight preference for closer targets
                                }
                            }
                        }
                        
                        // Line of sight bonus
                        if self.has_line_of_sight(&ai_pos, &target_pos) {
                            score += 10;
                        }
                        
                        if score > best_score {
                            best_score = score;
                            best_target = Some(id);
                        }
                    }
                }
            }
            
            best_target
        } else {
            None
        }
    }
    pub fn new(width: u32, height: u32) -> Self {
        let mut tiles = std::collections::HashMap::new();
        
        // Initialize all tiles as open terrain
        for x in 0..width as i32 {
            for y in 0..height as i32 {
                let pos = BattlefieldPosition::new(x, y);
                tiles.insert(pos, BattlefieldTile::new(pos, TerrainFeature::Open));
            }
        }
        
        Self {
            width,
            height,
            tiles,
            participant_positions: std::collections::HashMap::new(),
            environmental_features: Vec::new(),
        }
    }
    
    pub fn generate_battlefield(width: u32, height: u32, complexity: BattlefieldComplexity) -> Self {
        let mut battlefield = Self::new(width, height);
        let mut rng = rand::thread_rng();
        
        match complexity {
            BattlefieldComplexity::Simple => {
                // Add a few obstacles and cover points
                let obstacle_count = rng.gen_range(2..=4);
                for _ in 0..obstacle_count {
                    let x = rng.gen_range(0..width as i32);
                    let y = rng.gen_range(0..height as i32);
                    let pos = BattlefieldPosition::new(x, y);
                    
                    if let Some(tile) = battlefield.tiles.get_mut(&pos) {
                        tile.terrain = if rng.gen_bool(0.6) {
                            TerrainFeature::Cover
                        } else {
                            TerrainFeature::Obstacle
                        };
                        *tile = BattlefieldTile::new(pos, tile.terrain.clone());
                    }
                }
            }
            BattlefieldComplexity::Moderate => {
                // Add more varied terrain and some environmental features
                let feature_count = rng.gen_range(4..=8);
                for _ in 0..feature_count {
                    let x = rng.gen_range(0..width as i32);
                    let y = rng.gen_range(0..height as i32);
                    let pos = BattlefieldPosition::new(x, y);
                    
                    if let Some(tile) = battlefield.tiles.get_mut(&pos) {
                        tile.terrain = match rng.gen_range(0..6) {
                            0 => TerrainFeature::Obstacle,
                            1 => TerrainFeature::Cover,
                            2 => TerrainFeature::DifficultTerrain,
                            3 => TerrainFeature::Elevation,
                            4 => TerrainFeature::Pillar,
                            _ => TerrainFeature::Water,
                        };
                        *tile = BattlefieldTile::new(pos, tile.terrain.clone());
                    }
                }
                
                // Add environmental features
                if rng.gen_bool(0.3) {
                    let feature_pos = BattlefieldPosition::new(
                        rng.gen_range(0..width as i32),
                        rng.gen_range(0..height as i32)
                    );
                    battlefield.environmental_features.push(EnvironmentalFeature {
                        name: "Ancient Lever".to_string(),
                        position: feature_pos,
                        feature_type: EnvironmentalFeatureType::Lever,
                        activatable: true,
                        activation_effect: Some("Activates trap at (3,3)".to_string()),
                    });
                }
            }
            BattlefieldComplexity::Complex => {
                // Create elaborate battlefield with many features
                let feature_count = rng.gen_range(8..=15);
                for _ in 0..feature_count {
                    let x = rng.gen_range(0..width as i32);
                    let y = rng.gen_range(0..height as i32);
                    let pos = BattlefieldPosition::new(x, y);
                    
                    if let Some(tile) = battlefield.tiles.get_mut(&pos) {
                        tile.terrain = match rng.gen_range(0..10) {
                            0..=1 => TerrainFeature::Obstacle,
                            2..=3 => TerrainFeature::Cover,
                            4 => TerrainFeature::DifficultTerrain,
                            5 => TerrainFeature::Elevation,
                            6 => TerrainFeature::Pillar,
                            7 => TerrainFeature::Water,
                            8 => TerrainFeature::Hazard,
                            _ => TerrainFeature::Altar,
                        };
                        *tile = BattlefieldTile::new(pos, tile.terrain.clone());
                    }
                }
                
                // Add multiple environmental features
                let env_feature_count = rng.gen_range(2..=4);
                for i in 0..env_feature_count {
                    let feature_pos = BattlefieldPosition::new(
                        rng.gen_range(0..width as i32),
                        rng.gen_range(0..height as i32)
                    );
                    
                    let (name, feature_type, effect) = match i % 3 {
                        0 => ("Arcane Circle".to_string(), EnvironmentalFeatureType::MagicCircle, 
                              Some("Spells cast from here gain +2 damage".to_string())),
                        1 => ("Flame Brazier".to_string(), EnvironmentalFeatureType::Brazier,
                              Some("When lit, deals 1 fire damage to adjacent enemies each turn".to_string())),
                        _ => ("Hidden Trap".to_string(), EnvironmentalFeatureType::Trap,
                              Some("Triggered by movement, deals 2d4 damage".to_string())),
                    };
                    
                    battlefield.environmental_features.push(EnvironmentalFeature {
                        name,
                        position: feature_pos,
                        feature_type,
                        activatable: true,
                        activation_effect: effect,
                    });
                }
            }
        }
        
        battlefield
    }
    
    pub fn is_position_valid(&self, pos: &BattlefieldPosition) -> bool {
        pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32
    }
    
    pub fn is_position_passable(&self, pos: &BattlefieldPosition) -> bool {
        if !self.is_position_valid(pos) {
            return false;
        }
        
        // Check if tile blocks movement
        if let Some(tile) = self.tiles.get(pos) {
            if tile.blocks_movement {
                return false;
            }
        }
        
        // Check if another participant is already there
        self.participant_positions.values().all(|occupied_pos| occupied_pos != pos)
    }
    
    pub fn get_movement_cost(&self, pos: &BattlefieldPosition) -> u32 {
        if let Some(tile) = self.tiles.get(pos) {
            tile.movement_cost
        } else {
            1
        }
    }
    
    pub fn has_line_of_sight(&self, from: &BattlefieldPosition, to: &BattlefieldPosition) -> bool {
        let line = from.get_line_to(to);
        
        // Check each tile in the line of sight (except the start and end)
        for pos in line.iter().skip(1).take(line.len().saturating_sub(2)) {
            if let Some(tile) = self.tiles.get(pos) {
                if tile.blocks_line_of_sight {
                    return false;
                }
            }
        }
        
        true
    }
    
    pub fn get_cover_bonus(&self, pos: &BattlefieldPosition) -> i8 {
        if let Some(tile) = self.tiles.get(pos) {
            tile.cover_bonus
        } else {
            0
        }
    }
    
    pub fn move_participant(&mut self, participant_id: usize, new_position: BattlefieldPosition) -> Result<(), String> {
        if !self.is_position_passable(&new_position) {
            return Err("Position is not passable".to_string());
        }
        
        self.participant_positions.insert(participant_id, new_position);
        Ok(())
    }
    
    pub fn get_participant_position(&self, participant_id: usize) -> Option<BattlefieldPosition> {
        self.participant_positions.get(&participant_id).copied()
    }
    
    pub fn get_participants_in_range(&self, center: &BattlefieldPosition, range: u32) -> Vec<usize> {
        self.participant_positions
            .iter()
            .filter(|(_, pos)| center.distance_to(pos) <= range as f32)
            .map(|(id, _)| *id)
            .collect()
    }
    
    pub fn get_valid_spell_targets(&self, caster_pos: &BattlefieldPosition, spell: &super::magic::Spell, 
                                   participants: &[TacticalCombatParticipant], caster_id: usize) -> Vec<BattlefieldPosition> {
        let mut valid_targets = Vec::new();
        
        if let Some(tactical_info) = &spell.tactical_info {
            match &spell.target {
                super::magic::SpellTarget::Self_ => {
                    valid_targets.push(*caster_pos);
                }
                super::magic::SpellTarget::SingleEnemy => {
                    // Find all enemy positions within range
                    for (id, pos) in &self.participant_positions {
                        if *id != caster_id {
                            if let Some(participant) = participants.get(*id) {
                                if !participant.base_participant.is_player && participant.base_participant.is_alive() {
                                    let distance = caster_pos.manhattan_distance_to(pos) as u32;
                                    if distance <= tactical_info.range as u32 {
                                        if !tactical_info.requires_line_of_sight || self.has_line_of_sight(caster_pos, pos) {
                                            valid_targets.push(*pos);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                super::magic::SpellTarget::SingleAlly => {
                    // Find all ally positions within range (including self)
                    for (id, pos) in &self.participant_positions {
                        if let Some(participant) = participants.get(*id) {
                            if participant.base_participant.is_player && participant.base_participant.is_alive() {
                                let distance = caster_pos.manhattan_distance_to(pos) as u32;
                                if distance <= tactical_info.range as u32 {
                                    if !tactical_info.requires_line_of_sight || self.has_line_of_sight(caster_pos, pos) {
                                        valid_targets.push(*pos);
                                    }
                                }
                            }
                        }
                    }
                }
                super::magic::SpellTarget::Area(_radius) => {
                    // Get all positions within spell range that can be targeted
                    for x in (caster_pos.x - tactical_info.range as i32)..=(caster_pos.x + tactical_info.range as i32) {
                        for y in (caster_pos.y - tactical_info.range as i32)..=(caster_pos.y + tactical_info.range as i32) {
                            let target_pos = BattlefieldPosition::new(x, y);
                            if self.is_position_valid(&target_pos) {
                                let distance = caster_pos.manhattan_distance_to(&target_pos) as u32;
                                if distance <= tactical_info.range as u32 {
                                    if !tactical_info.requires_line_of_sight || self.has_line_of_sight(caster_pos, &target_pos) {
                                        valid_targets.push(target_pos);
                                    }
                                }
                            }
                        }
                    }
                }
                super::magic::SpellTarget::Line(length) => {
                    // Get positions in cardinal directions up to spell range
                    let directions = vec![
                        (0, -1),  // North
                        (1, 0),   // East
                        (0, 1),   // South
                        (-1, 0),  // West
                    ];
                    
                    for (dx, dy) in directions {
                        for i in 1..=tactical_info.range.min(*length) {
                            let target_pos = BattlefieldPosition::new(
                                caster_pos.x + dx * i as i32,
                                caster_pos.y + dy * i as i32
                            );
                            if self.is_position_valid(&target_pos) {
                                valid_targets.push(target_pos);
                            }
                        }
                    }
                }
                super::magic::SpellTarget::AllEnemies | super::magic::SpellTarget::AllAllies => {
                    // For "all" targets, just return caster position as the spell affects all valid targets automatically
                    valid_targets.push(*caster_pos);
                }
                _ => {} // Handle other target types as needed
            }
        }
        
        valid_targets
    }
    
    pub fn get_affected_positions(&self, spell: &super::magic::Spell, target_pos: &BattlefieldPosition) -> Vec<BattlefieldPosition> {
        let mut affected = Vec::new();
        
        match &spell.target {
            super::magic::SpellTarget::Area(radius) => {
                // Get all positions within the area effect radius
                for x in (target_pos.x - *radius as i32)..=(target_pos.x + *radius as i32) {
                    for y in (target_pos.y - *radius as i32)..=(target_pos.y + *radius as i32) {
                        let pos = BattlefieldPosition::new(x, y);
                        if self.is_position_valid(&pos) {
                            let distance = target_pos.distance_to(&pos);
                            if distance <= *radius as f32 {
                                affected.push(pos);
                            }
                        }
                    }
                }
            }
            super::magic::SpellTarget::Line(_length) => {
                // For line spells, determine direction from caster to target
                // This would need the caster position to be passed in for proper implementation
                affected.push(*target_pos); // Simplified for now
            }
            super::magic::SpellTarget::Cone(_range) => {
                // Cone spells would affect a triangular area
                // This would need more complex calculation
                affected.push(*target_pos); // Simplified for now
            }
            _ => {
                // Single target spells
                affected.push(*target_pos);
            }
        }
        
        affected
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattlefieldComplexity {
    Simple,     // Few obstacles, basic terrain
    Moderate,   // Mixed terrain, some environmental features
    Complex,    // Many features, hazards, interactive elements
}

// Movement and Action Point System
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementCapabilities {
    pub movement_speed: u32,    // How many tiles can move per turn
    pub can_fly: bool,          // Ignores some terrain restrictions
    pub can_swim: bool,         // Can move through water without penalty
    pub can_climb: bool,        // Can scale elevation without penalty
}

impl Default for MovementCapabilities {
    fn default() -> Self {
        Self {
            movement_speed: 3,  // Default 3 tiles per turn
            can_fly: false,
            can_swim: false,
            can_climb: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalCombatParticipant {
    pub base_participant: CombatParticipant,
    pub position: BattlefieldPosition,
    pub movement_capabilities: MovementCapabilities,
    pub movement_remaining: u32,
    pub has_acted: bool,
    pub declared_action: Option<TacticalCombatAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TacticalCombatAction {
    Move { target_position: BattlefieldPosition },
    Attack { target_id: usize },
    CastSpell { spell_name: String, target_position: Option<BattlefieldPosition>, target_id: Option<usize> },
    UseItem { item_name: String, target_id: Option<usize> },
    Defend,
    Interact { feature_position: BattlefieldPosition },
    Wait,
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
    CombatParticipant::create_enemy_with_ai(
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
        }),
        AIPersonality::aggressive() // Wild animals are aggressive
    )
}

pub fn create_wolf() -> CombatParticipant {
    CombatParticipant::create_enemy_with_ai(
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
        }),
        AIPersonality::tactical() // Wolves are pack hunters, tactical
    )
}

pub fn create_goblin() -> CombatParticipant {
    let mut goblin = CombatParticipant::create_enemy_with_ai(
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
        }),
        AIPersonality::coward() // Goblins are cowardly
    );
    goblin.armor = Some(Armor::leather());
    goblin
}

pub fn create_bandit() -> CombatParticipant {
    let mut bandit = CombatParticipant::create_enemy_with_ai(
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
        }),
        AIPersonality::balanced() // Bandits are balanced fighters
    );
    bandit.armor = Some(Armor::studded_leather());
    bandit.shield = Some(Armor::small_shield());
    bandit
}

pub fn create_orc() -> CombatParticipant {
    let mut orc = CombatParticipant::create_enemy_with_ai(
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
        }),
        AIPersonality::berserker() // Orcs are berserkers
    );
    orc.armor = Some(Armor::chain_mail());
    orc
}

pub fn create_giant_spider() -> CombatParticipant {
    CombatParticipant::create_enemy_with_ai(
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
        }),
        AIPersonality::tactical() // Spiders are ambush predators, tactical
    )
}

pub fn create_mountain_lion() -> CombatParticipant {
    CombatParticipant::create_enemy_with_ai(
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
        }),
        AIPersonality::aggressive() // Big cats are aggressive predators
    )
}

pub fn create_skeleton() -> CombatParticipant {
    let mut skeleton = CombatParticipant::create_enemy_with_ai(
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
        }),
        AIPersonality::defensive() // Undead are methodical, defensive
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
    CombatParticipant::create_enemy_with_ai(
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
        }),
        AIPersonality {
            behavior_type: AIBehaviorType::Aggressive,
            aggression: 7,
            intelligence: 1, // Zombies are mindless
            self_preservation: 0, // Don't care about survival
            spell_preference: 0,
            ranged_preference: 0,
            health_threshold: 0, // Never flee
        }
    )
}
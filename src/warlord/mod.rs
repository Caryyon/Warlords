use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Warlord Progression System
/// Players advance through four tiers as they gain power, followers, and territory

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarlordTier {
    Adventurer,     // Tier 1: Solo or small party, clearing dungeons
    Captain,        // Tier 2: Can recruit followers (1-10 people), lead mercenaries
    Warlord,        // Tier 3: Control territory, maintain army (50-500), spy network
    Lord,           // Tier 4: Multiple cities/castles, noble title, vassals
}

impl WarlordTier {
    pub fn name(&self) -> &'static str {
        match self {
            WarlordTier::Adventurer => "Adventurer",
            WarlordTier::Captain => "Captain",
            WarlordTier::Warlord => "Warlord",
            WarlordTier::Lord => "Lord",
        }
    }

    pub fn title_options(&self) -> Vec<&'static str> {
        match self {
            WarlordTier::Adventurer => vec!["Adventurer", "Wanderer", "Sellsword", "Treasure Hunter"],
            WarlordTier::Captain => vec!["Captain", "Band Leader", "Sergeant", "War Chief"],
            WarlordTier::Warlord => vec!["Warlord", "Chieftain", "Commander", "War Marshal"],
            WarlordTier::Lord => vec!["Lord", "Baron", "Count", "Duke"],
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            WarlordTier::Adventurer => "A wandering hero, seeking fortune and glory in the wilds.",
            WarlordTier::Captain => "Leader of a small band of loyal followers.",
            WarlordTier::Warlord => "Commander of armies, ruler of territories.",
            WarlordTier::Lord => "A noble power, master of cities and castles.",
        }
    }

    pub fn max_followers(&self) -> usize {
        match self {
            WarlordTier::Adventurer => 0,
            WarlordTier::Captain => 10,
            WarlordTier::Warlord => 500,
            WarlordTier::Lord => 5000,
        }
    }

    pub fn next_tier(&self) -> Option<WarlordTier> {
        match self {
            WarlordTier::Adventurer => Some(WarlordTier::Captain),
            WarlordTier::Captain => Some(WarlordTier::Warlord),
            WarlordTier::Warlord => Some(WarlordTier::Lord),
            WarlordTier::Lord => None,
        }
    }
}

/// Requirements to advance to the next tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRequirements {
    pub reputation_needed: i32,
    pub gold_needed: u32,
    pub quests_completed: usize,
    pub followers_recruited: usize,
    pub territories_controlled: usize,
    pub special_requirement: Option<String>,
}

impl TierRequirements {
    pub fn for_tier(tier: WarlordTier) -> Option<Self> {
        match tier {
            WarlordTier::Adventurer => None, // Starting tier
            WarlordTier::Captain => Some(TierRequirements {
                reputation_needed: 100,
                gold_needed: 500,
                quests_completed: 5,
                followers_recruited: 0,
                territories_controlled: 0,
                special_requirement: Some("Establish a base camp".to_string()),
            }),
            WarlordTier::Warlord => Some(TierRequirements {
                reputation_needed: 500,
                gold_needed: 5000,
                quests_completed: 20,
                followers_recruited: 10,
                territories_controlled: 1,
                special_requirement: Some("Capture or claim a village".to_string()),
            }),
            WarlordTier::Lord => Some(TierRequirements {
                reputation_needed: 2000,
                gold_needed: 50000,
                quests_completed: 50,
                followers_recruited: 100,
                territories_controlled: 3,
                special_requirement: Some("Take a castle and receive noble recognition".to_string()),
            }),
        }
    }
}

/// A follower in the player's warband
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Follower {
    pub id: String,
    pub name: String,
    pub follower_type: FollowerType,
    pub level: u8,
    pub loyalty: u8, // 0-100
    pub morale: u8,  // 0-100
    pub health: u32,
    pub max_health: u32,
    pub equipment_quality: EquipmentQuality,
    pub skills: Vec<String>,
    pub monthly_cost: u32, // Gold per month
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FollowerType {
    Peasant,
    Militia,
    Soldier,
    Veteran,
    Elite,
    // Specialists
    Scout,
    Spy,
    Healer,
    Mage,
    Archer,
    Cavalry,
}

impl FollowerType {
    pub fn name(&self) -> &'static str {
        match self {
            FollowerType::Peasant => "Peasant Levy",
            FollowerType::Militia => "Militia",
            FollowerType::Soldier => "Soldier",
            FollowerType::Veteran => "Veteran",
            FollowerType::Elite => "Elite Guard",
            FollowerType::Scout => "Scout",
            FollowerType::Spy => "Spy",
            FollowerType::Healer => "Healer",
            FollowerType::Mage => "War Mage",
            FollowerType::Archer => "Archer",
            FollowerType::Cavalry => "Cavalry",
        }
    }

    pub fn base_cost(&self) -> u32 {
        match self {
            FollowerType::Peasant => 1,
            FollowerType::Militia => 2,
            FollowerType::Soldier => 5,
            FollowerType::Veteran => 10,
            FollowerType::Elite => 25,
            FollowerType::Scout => 8,
            FollowerType::Spy => 15,
            FollowerType::Healer => 12,
            FollowerType::Mage => 30,
            FollowerType::Archer => 7,
            FollowerType::Cavalry => 15,
        }
    }

    pub fn combat_power(&self) -> u32 {
        match self {
            FollowerType::Peasant => 1,
            FollowerType::Militia => 2,
            FollowerType::Soldier => 4,
            FollowerType::Veteran => 7,
            FollowerType::Elite => 12,
            FollowerType::Scout => 3,
            FollowerType::Spy => 2,
            FollowerType::Healer => 1,
            FollowerType::Mage => 10,
            FollowerType::Archer => 5,
            FollowerType::Cavalry => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentQuality {
    Poor,
    Standard,
    Good,
    Excellent,
    Masterwork,
}

impl EquipmentQuality {
    pub fn combat_modifier(&self) -> f32 {
        match self {
            EquipmentQuality::Poor => 0.75,
            EquipmentQuality::Standard => 1.0,
            EquipmentQuality::Good => 1.25,
            EquipmentQuality::Excellent => 1.5,
            EquipmentQuality::Masterwork => 2.0,
        }
    }
}

/// Territory controlled by the player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Territory {
    pub id: String,
    pub name: String,
    pub territory_type: TerritoryType,
    pub position: (i32, i32), // World coordinates
    pub population: u32,
    pub prosperity: u8, // 0-100
    pub defense_level: u8, // 0-100
    pub monthly_income: u32,
    pub monthly_expenses: u32,
    pub resources: HashMap<ResourceType, u32>,
    pub buildings: Vec<Building>,
    pub garrison: Vec<String>, // Follower IDs stationed here
    pub loyalty: u8, // 0-100 - loyalty to the player
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerritoryType {
    Camp,       // Temporary base
    Village,
    Town,
    City,
    Castle,
    Fortress,
}

impl TerritoryType {
    pub fn name(&self) -> &'static str {
        match self {
            TerritoryType::Camp => "Camp",
            TerritoryType::Village => "Village",
            TerritoryType::Town => "Town",
            TerritoryType::City => "City",
            TerritoryType::Castle => "Castle",
            TerritoryType::Fortress => "Fortress",
        }
    }

    pub fn base_income(&self) -> u32 {
        match self {
            TerritoryType::Camp => 0,
            TerritoryType::Village => 50,
            TerritoryType::Town => 200,
            TerritoryType::City => 1000,
            TerritoryType::Castle => 500,
            TerritoryType::Fortress => 300,
        }
    }

    pub fn max_garrison(&self) -> usize {
        match self {
            TerritoryType::Camp => 10,
            TerritoryType::Village => 25,
            TerritoryType::Town => 100,
            TerritoryType::City => 500,
            TerritoryType::Castle => 250,
            TerritoryType::Fortress => 400,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Food,
    Gold,
    Iron,
    Wood,
    Stone,
    Horses,
    Weapons,
    Armor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub building_type: BuildingType,
    pub level: u8,
    pub construction_progress: u8, // 0-100, 100 = complete
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    Barracks,
    Blacksmith,
    Stable,
    Market,
    Walls,
    Tower,
    Temple,
    Tavern,
    Farm,
    Mine,
    Lumber,
}

impl BuildingType {
    pub fn name(&self) -> &'static str {
        match self {
            BuildingType::Barracks => "Barracks",
            BuildingType::Blacksmith => "Blacksmith",
            BuildingType::Stable => "Stable",
            BuildingType::Market => "Market",
            BuildingType::Walls => "Walls",
            BuildingType::Tower => "Watchtower",
            BuildingType::Temple => "Temple",
            BuildingType::Tavern => "Tavern",
            BuildingType::Farm => "Farm",
            BuildingType::Mine => "Mine",
            BuildingType::Lumber => "Lumber Mill",
        }
    }

    pub fn build_cost(&self, level: u8) -> HashMap<ResourceType, u32> {
        let base_multiplier = (level as u32 + 1) * 10;
        let mut cost = HashMap::new();

        match self {
            BuildingType::Barracks => {
                cost.insert(ResourceType::Gold, 100 * base_multiplier);
                cost.insert(ResourceType::Wood, 50 * base_multiplier);
                cost.insert(ResourceType::Stone, 30 * base_multiplier);
            }
            BuildingType::Blacksmith => {
                cost.insert(ResourceType::Gold, 150 * base_multiplier);
                cost.insert(ResourceType::Iron, 40 * base_multiplier);
                cost.insert(ResourceType::Stone, 20 * base_multiplier);
            }
            BuildingType::Stable => {
                cost.insert(ResourceType::Gold, 80 * base_multiplier);
                cost.insert(ResourceType::Wood, 60 * base_multiplier);
                cost.insert(ResourceType::Horses, 5 * base_multiplier);
            }
            BuildingType::Market => {
                cost.insert(ResourceType::Gold, 200 * base_multiplier);
                cost.insert(ResourceType::Wood, 40 * base_multiplier);
            }
            BuildingType::Walls => {
                cost.insert(ResourceType::Gold, 300 * base_multiplier);
                cost.insert(ResourceType::Stone, 100 * base_multiplier);
            }
            BuildingType::Tower => {
                cost.insert(ResourceType::Gold, 150 * base_multiplier);
                cost.insert(ResourceType::Stone, 50 * base_multiplier);
            }
            BuildingType::Temple => {
                cost.insert(ResourceType::Gold, 250 * base_multiplier);
                cost.insert(ResourceType::Stone, 60 * base_multiplier);
            }
            BuildingType::Tavern => {
                cost.insert(ResourceType::Gold, 100 * base_multiplier);
                cost.insert(ResourceType::Wood, 40 * base_multiplier);
            }
            BuildingType::Farm => {
                cost.insert(ResourceType::Gold, 50 * base_multiplier);
                cost.insert(ResourceType::Wood, 20 * base_multiplier);
            }
            BuildingType::Mine => {
                cost.insert(ResourceType::Gold, 200 * base_multiplier);
                cost.insert(ResourceType::Wood, 30 * base_multiplier);
            }
            BuildingType::Lumber => {
                cost.insert(ResourceType::Gold, 75 * base_multiplier);
                cost.insert(ResourceType::Wood, 10 * base_multiplier);
            }
        }

        cost
    }
}

/// Spy network operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpyNetwork {
    pub agents: Vec<SpyAgent>,
    pub intel: Vec<IntelReport>,
    pub active_operations: Vec<SpyOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpyAgent {
    pub id: String,
    pub name: String,
    pub skill: u8, // 0-100
    pub location: Option<String>, // Territory or faction where stationed
    pub cover: String, // Their cover identity
    pub status: AgentStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Infiltrating,
    Gathering,
    Sabotaging,
    Returning,
    Captured,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelReport {
    pub report_type: IntelType,
    pub source: String,
    pub content: String,
    pub reliability: u8, // 0-100
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub expires: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntelType {
    TroopMovement,
    TerritoryWeakness,
    PoliticalIntrigue,
    EconomicInfo,
    MilitaryStrength,
    SecretPassage,
    Betrayal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpyOperation {
    pub operation_type: OperationType,
    pub target: String,
    pub agent_id: String,
    pub progress: u8, // 0-100
    pub risk_level: u8, // 0-100
    pub estimated_completion: u64, // Game turns
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    GatherIntel,
    Sabotage,
    Assassination,
    Bribery,
    Counterintelligence,
    EstablishNetwork,
}

/// The player's complete warlord state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarlordState {
    pub tier: WarlordTier,
    pub title: String,
    pub reputation: i32,
    pub infamy: i32, // Negative reputation from evil acts
    pub followers: Vec<Follower>,
    pub territories: Vec<Territory>,
    pub spy_network: Option<SpyNetwork>,
    pub resources: HashMap<ResourceType, u32>,
    pub monthly_income: i32,
    pub monthly_expenses: i32,
    pub wars: Vec<War>,
    pub alliances: Vec<Alliance>,
    pub quests_completed: usize,
    pub enemies_defeated: u32,
    pub dungeons_cleared: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct War {
    pub enemy_faction: String,
    pub start_turn: u64,
    pub war_score: i32, // Positive = winning, negative = losing
    pub battles_won: u32,
    pub battles_lost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alliance {
    pub faction: String,
    pub alliance_type: AllianceType,
    pub strength: u8, // 0-100
    pub start_turn: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllianceType {
    NonAggression,
    DefensivePact,
    MilitaryAlliance,
    Vassalage,
    Overlord,
}

impl WarlordState {
    pub fn new() -> Self {
        WarlordState {
            tier: WarlordTier::Adventurer,
            title: "Adventurer".to_string(),
            reputation: 0,
            infamy: 0,
            followers: Vec::new(),
            territories: Vec::new(),
            spy_network: None,
            resources: HashMap::new(),
            monthly_income: 0,
            monthly_expenses: 0,
            wars: Vec::new(),
            alliances: Vec::new(),
            quests_completed: 0,
            enemies_defeated: 0,
            dungeons_cleared: 0,
        }
    }

    pub fn can_advance(&self) -> bool {
        if let Some(next_tier) = self.tier.next_tier() {
            if let Some(requirements) = TierRequirements::for_tier(next_tier) {
                return self.reputation >= requirements.reputation_needed
                    && self.resources.get(&ResourceType::Gold).copied().unwrap_or(0) >= requirements.gold_needed
                    && self.quests_completed >= requirements.quests_completed
                    && self.followers.len() >= requirements.followers_recruited
                    && self.territories.len() >= requirements.territories_controlled;
            }
        }
        false
    }

    pub fn advance_tier(&mut self) -> Result<WarlordTier, &'static str> {
        if !self.can_advance() {
            return Err("Requirements not met for advancement");
        }

        if let Some(next_tier) = self.tier.next_tier() {
            self.tier = next_tier;
            self.title = next_tier.title_options()[0].to_string();

            // Unlock spy network at Warlord tier
            if next_tier == WarlordTier::Warlord && self.spy_network.is_none() {
                self.spy_network = Some(SpyNetwork {
                    agents: Vec::new(),
                    intel: Vec::new(),
                    active_operations: Vec::new(),
                });
            }

            Ok(next_tier)
        } else {
            Err("Already at maximum tier")
        }
    }

    pub fn total_army_power(&self) -> u32 {
        self.followers.iter()
            .map(|f| {
                let base = f.follower_type.combat_power();
                let morale_mod = f.morale as f32 / 100.0;
                let equip_mod = f.equipment_quality.combat_modifier();
                ((base as f32) * morale_mod * equip_mod) as u32
            })
            .sum()
    }

    pub fn total_monthly_costs(&self) -> u32 {
        let follower_costs: u32 = self.followers.iter()
            .map(|f| f.monthly_cost)
            .sum();

        let territory_costs: u32 = self.territories.iter()
            .map(|t| t.monthly_expenses)
            .sum();

        follower_costs + territory_costs
    }

    pub fn total_monthly_income(&self) -> u32 {
        self.territories.iter()
            .map(|t| t.monthly_income)
            .sum()
    }

    pub fn add_follower(&mut self, follower: Follower) -> Result<(), &'static str> {
        if self.followers.len() >= self.tier.max_followers() {
            return Err("Maximum followers reached for current tier");
        }
        self.followers.push(follower);
        Ok(())
    }

    pub fn add_reputation(&mut self, amount: i32) {
        self.reputation = (self.reputation + amount).max(0);
    }

    pub fn add_infamy(&mut self, amount: i32) {
        self.infamy = (self.infamy + amount).max(0);
    }
}

impl Default for WarlordState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_progression() {
        let state = WarlordState::new();
        assert_eq!(state.tier, WarlordTier::Adventurer);
        assert_eq!(state.tier.next_tier(), Some(WarlordTier::Captain));
    }

    #[test]
    fn test_max_followers() {
        assert_eq!(WarlordTier::Adventurer.max_followers(), 0);
        assert_eq!(WarlordTier::Captain.max_followers(), 10);
        assert_eq!(WarlordTier::Warlord.max_followers(), 500);
        assert_eq!(WarlordTier::Lord.max_followers(), 5000);
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::{Persistable, PersistenceBackend};

/// Combat state - represents an active tactical combat encounter
/// In multiplayer, this needs to handle turn-based synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    /// Unique identifier for this combat instance
    pub combat_id: String,

    /// Where this combat is taking place
    pub location: CombatLocation,

    /// Combat metadata
    pub metadata: CombatMetadata,

    /// All combatants (players and enemies)
    pub combatants: Vec<Combatant>,

    /// Initiative order
    pub initiative_order: Vec<String>, // Combatant IDs in turn order

    /// Current turn index
    pub current_turn: usize,

    /// Combat map/grid
    pub grid: CombatGrid,

    /// Combat log/history
    pub combat_log: Vec<CombatLogEntry>,

    /// Combat status
    pub status: CombatStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatLocation {
    /// Combat in the world
    World {
        zone_x: i32,
        zone_y: i32,
        local_x: i32,
        local_y: i32,
    },
    /// Combat in a dungeon
    Dungeon {
        instance_id: String,
        floor: usize,
        x: i32,
        y: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatMetadata {
    /// When combat started
    pub started_at: chrono::DateTime<chrono::Utc>,

    /// Combat type/encounter name
    pub encounter_name: String,

    /// Difficulty rating
    pub difficulty: u32,

    /// Is this combat optional (can flee)
    pub can_flee: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combatant {
    /// Unique ID for this combatant
    pub id: String,

    /// Display name
    pub name: String,

    /// Type of combatant
    pub combatant_type: CombatantType,

    /// Position on the grid
    pub position: GridPosition,

    /// Current stats
    pub stats: CombatantStats,

    /// Active status effects
    pub status_effects: Vec<StatusEffect>,

    /// Actions available this turn
    pub actions_remaining: u32,

    /// Has this combatant acted this turn
    pub has_acted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatantType {
    /// Player character
    PlayerCharacter { character_id: String },

    /// AI-controlled enemy
    Enemy {
        enemy_type: String,
        behavior: EnemyBehavior,
    },

    /// Ally/companion
    Ally { ally_type: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnemyBehavior {
    Aggressive,
    Defensive,
    Cautious,
    Berserker,
    Tactical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatantStats {
    pub health: i32,
    pub max_health: i32,
    pub armor: i32,
    pub toughness: i32,
    pub parry: i32,

    /// Movement speed (tiles per turn)
    pub speed: u32,

    /// Derived stats for quick access
    pub strength: i32,
    pub agility: i32,
    pub spirit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEffect {
    pub effect_type: StatusEffectType,
    pub duration: i32, // Turns remaining (-1 for permanent)
    pub strength: i32, // Magnitude of effect
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusEffectType {
    Stunned,
    Bleeding,
    Poisoned,
    Burning,
    Frozen,
    Blessed,
    Cursed,
    Shaken,
    Vulnerable,
    Strengthened,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &GridPosition) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatGrid {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Vec<GridTile>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridTile {
    pub tile_type: GridTileType,
    pub elevation: i32,
    pub occupied_by: Option<String>, // Combatant ID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridTileType {
    Normal,
    Difficult, // Costs extra movement
    Wall,
    Water,
    Pit,
    Cover, // Provides defensive bonus
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatLogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub turn: usize,
    pub actor: String, // Combatant ID
    pub action: CombatAction,
    pub result: ActionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatAction {
    Move { from: GridPosition, to: GridPosition },
    Attack { target: String, weapon: String },
    Cast { spell: String, targets: Vec<String> },
    UseItem { item: String },
    Defend,
    Flee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    Success { description: String },
    Failure { reason: String },
    Damage { target: String, amount: i32 },
    Healing { target: String, amount: i32 },
    StatusApplied { target: String, effect: StatusEffectType },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CombatStatus {
    Active,
    Victory,
    Defeat,
    Fled,
}

impl CombatState {
    pub fn new(combat_id: String, location: CombatLocation, metadata: CombatMetadata) -> Self {
        Self {
            combat_id,
            location,
            metadata,
            combatants: vec![],
            initiative_order: vec![],
            current_turn: 0,
            grid: CombatGrid {
                width: 20,
                height: 20,
                tiles: vec![],
            },
            combat_log: vec![],
            status: CombatStatus::Active,
        }
    }

    /// Get the combatant whose turn it is
    pub fn current_combatant(&self) -> Option<&Combatant> {
        let id = self.initiative_order.get(self.current_turn)?;
        self.combatants.iter().find(|c| &c.id == id)
    }

    /// Get mutable current combatant
    pub fn current_combatant_mut(&mut self) -> Option<&mut Combatant> {
        let id = self.initiative_order.get(self.current_turn)?.clone();
        self.combatants.iter_mut().find(|c| c.id == id)
    }

    /// Advance to next turn
    pub fn next_turn(&mut self) {
        self.current_turn = (self.current_turn + 1) % self.initiative_order.len();

        // Reset actions for new combatant
        if let Some(combatant) = self.current_combatant_mut() {
            combatant.actions_remaining = 1; // Standard Savage Worlds: 1 action per turn
            combatant.has_acted = false;
        }

        // Process status effects
        self.process_status_effects();
    }

    /// Process status effects at start of turn
    fn process_status_effects(&mut self) {
        // Process ongoing effects and decrement durations
        for combatant in &mut self.combatants {
            combatant.status_effects.retain_mut(|effect| {
                if effect.duration > 0 {
                    effect.duration -= 1;
                    true
                } else if effect.duration == -1 {
                    true // Permanent effect
                } else {
                    false // Effect expired
                }
            });
        }
    }

    /// Add a combatant to the battle
    pub fn add_combatant(&mut self, combatant: Combatant) {
        self.initiative_order.push(combatant.id.clone());
        self.combatants.push(combatant);
    }

    /// Check if combat is over
    pub fn is_over(&self) -> bool {
        self.status != CombatStatus::Active
    }

    /// Add entry to combat log
    pub fn log_action(&mut self, actor: String, action: CombatAction, result: ActionResult) {
        self.combat_log.push(CombatLogEntry {
            timestamp: chrono::Utc::now(),
            turn: self.current_turn,
            actor,
            action,
            result,
        });
    }
}

impl Persistable for CombatState {
    fn state_type() -> &'static str {
        "combat"
    }

    fn save(&self, backend: &dyn PersistenceBackend) -> Result<()> {
        let key = format!("combat_{}", self.combat_id);
        let data = serde_json::to_vec_pretty(self)?;
        backend.save(&key, &data)
    }

    fn load(backend: &dyn PersistenceBackend, id: &str) -> Result<Self> {
        let key = format!("combat_{}", id);
        let data = backend.load(&key)?;
        let state = serde_json::from_slice(&data)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_state() {
        let location = CombatLocation::World {
            zone_x: 0,
            zone_y: 0,
            local_x: 10,
            local_y: 10,
        };

        let metadata = CombatMetadata {
            started_at: chrono::Utc::now(),
            encounter_name: "Bandit Ambush".to_string(),
            difficulty: 3,
            can_flee: true,
        };

        let combat = CombatState::new("test_combat".to_string(), location, metadata);

        assert_eq!(combat.status, CombatStatus::Active);
        assert!(!combat.is_over());
    }

    #[test]
    fn test_grid_position_distance() {
        let pos1 = GridPosition::new(0, 0);
        let pos2 = GridPosition::new(3, 4);

        assert_eq!(pos1.distance_to(&pos2), 5.0);
    }
}

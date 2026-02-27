use serde::{Deserialize, Serialize};
use crate::account::Occupation;

/// Tutorial system: "The Call to Adventure"
/// A story-driven introduction that teaches game mechanics while immersing
/// the player in their character's origin story.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TutorialStage {
    /// The calm before the storm - player's daily life
    DailyLife,
    /// Something is wrong - first hints of trouble
    Unease,
    /// The inciting incident - bandits/travelers/disaster
    Incident,
    /// Learning basic movement
    MovementLesson,
    /// First NPC interaction
    NPCInteraction,
    /// The call - something compels them to leave
    TheCall,
    /// First combat encounter
    FirstCombat,
    /// Saying goodbye
    Farewell,
    /// Entering the wider world
    Departure,
    /// Tutorial complete
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialState {
    pub stage: TutorialStage,
    pub occupation: Occupation,
    pub village_name: String,
    pub dialogue_index: usize,
    pub has_moved: bool,
    pub has_talked: bool,
    pub has_fought: bool,
    pub has_searched: bool,
    pub has_used_inventory: bool,
    pub mentor_name: String,
    pub family_member: String,
    pub inciting_event: IncitingEvent,
    pub messages: Vec<TutorialMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialMessage {
    pub speaker: Option<String>,
    pub text: String,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Narration,
    Dialogue,
    Thought,
    Tutorial,
    Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncitingEvent {
    BanditRaid,
    StrangeTraveler,
    FamilyDebt,
    ProphecyFulfilled,
    MysteriousArtifact,
    VillageDestroyed,
    DreamVision,
    RelicDiscovered,
}

impl IncitingEvent {
    pub fn for_occupation(occupation: &Occupation) -> Self {
        match occupation {
            Occupation::Farmer | Occupation::Shepherd | Occupation::Miller => {
                IncitingEvent::BanditRaid
            }
            Occupation::Hunter | Occupation::Fisher => {
                IncitingEvent::MysteriousArtifact
            }
            Occupation::Blacksmith | Occupation::Miner | Occupation::Carpenter => {
                IncitingEvent::RelicDiscovered
            }
            Occupation::Barkeep | Occupation::Merchant | Occupation::StableHand => {
                IncitingEvent::StrangeTraveler
            }
            Occupation::Herbalist => {
                IncitingEvent::DreamVision
            }
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            IncitingEvent::BanditRaid => "Bandits attack the village",
            IncitingEvent::StrangeTraveler => "A mysterious stranger arrives",
            IncitingEvent::FamilyDebt => "Your family owes a debt to dangerous people",
            IncitingEvent::ProphecyFulfilled => "An ancient prophecy speaks of you",
            IncitingEvent::MysteriousArtifact => "You discover something strange",
            IncitingEvent::VillageDestroyed => "Your home is destroyed",
            IncitingEvent::DreamVision => "Visions haunt your sleep",
            IncitingEvent::RelicDiscovered => "You uncover an ancient relic",
        }
    }
}

impl TutorialState {
    pub fn new(occupation: Occupation) -> Self {
        let village_name = occupation.village_name().to_string();
        let inciting_event = IncitingEvent::for_occupation(&occupation);

        let (mentor_name, family_member) = match &occupation {
            Occupation::Farmer => ("Old Barley".to_string(), "Ma".to_string()),
            Occupation::Blacksmith => ("Master Erikson".to_string(), "Da".to_string()),
            Occupation::Barkeep => ("Innkeeper Mira".to_string(), "your sister Lena".to_string()),
            Occupation::Merchant => ("Trader Hakon".to_string(), "your uncle".to_string()),
            Occupation::Fisher => ("Weathered Jonas".to_string(), "your grandfather".to_string()),
            Occupation::StableHand => ("Stable Master Bran".to_string(), "your brother".to_string()),
            Occupation::Miller => ("The Miller".to_string(), "your mother".to_string()),
            Occupation::Hunter => ("Old Tracker Finn".to_string(), "your father".to_string()),
            Occupation::Shepherd => ("Wise Shepherd Karl".to_string(), "your grandmother".to_string()),
            Occupation::Carpenter => ("Master Builder Tor".to_string(), "your cousin".to_string()),
            Occupation::Miner => ("Foreman Grist".to_string(), "your brother Kol".to_string()),
            Occupation::Herbalist => ("Elder Sage Mora".to_string(), "your mother".to_string()),
        };

        TutorialState {
            stage: TutorialStage::DailyLife,
            occupation,
            village_name,
            dialogue_index: 0,
            has_moved: false,
            has_talked: false,
            has_fought: false,
            has_searched: false,
            has_used_inventory: false,
            mentor_name,
            family_member,
            inciting_event,
            messages: Vec::new(),
        }
    }

    pub fn get_current_dialogue(&self) -> Vec<TutorialMessage> {
        match self.stage {
            TutorialStage::DailyLife => self.daily_life_dialogue(),
            TutorialStage::Unease => self.unease_dialogue(),
            TutorialStage::Incident => self.incident_dialogue(),
            TutorialStage::MovementLesson => self.movement_dialogue(),
            TutorialStage::NPCInteraction => self.npc_dialogue(),
            TutorialStage::TheCall => self.call_dialogue(),
            TutorialStage::FirstCombat => self.combat_dialogue(),
            TutorialStage::Farewell => self.farewell_dialogue(),
            TutorialStage::Departure => self.departure_dialogue(),
            TutorialStage::Complete => vec![],
        }
    }

    fn daily_life_dialogue(&self) -> Vec<TutorialMessage> {
        vec![
            TutorialMessage {
                speaker: None,
                text: self.occupation.tutorial_intro().to_string(),
                message_type: MessageType::Narration,
            },
            TutorialMessage {
                speaker: None,
                text: format!("The village of {} has been your home for as long as you can remember.", self.village_name),
                message_type: MessageType::Narration,
            },
            TutorialMessage {
                speaker: None,
                text: format!("You've worked as a {} here, day after day, season after season.", self.occupation.name().to_lowercase()),
                message_type: MessageType::Narration,
            },
            TutorialMessage {
                speaker: None,
                text: "It's been a simple life. Perhaps too simple.".to_string(),
                message_type: MessageType::Thought,
            },
        ]
    }

    fn unease_dialogue(&self) -> Vec<TutorialMessage> {
        match &self.inciting_event {
            IncitingEvent::BanditRaid => vec![
                TutorialMessage {
                    speaker: None,
                    text: "Smoke rises from the direction of the north road.".to_string(),
                    message_type: MessageType::Narration,
                },
                TutorialMessage {
                    speaker: Some(self.mentor_name.clone()),
                    text: "That's no cooking fire. Stay alert.".to_string(),
                    message_type: MessageType::Dialogue,
                },
            ],
            IncitingEvent::StrangeTraveler => vec![
                TutorialMessage {
                    speaker: None,
                    text: "A cloaked figure enters the village, moving with purpose.".to_string(),
                    message_type: MessageType::Narration,
                },
                TutorialMessage {
                    speaker: None,
                    text: "Something about them makes your skin prickle.".to_string(),
                    message_type: MessageType::Thought,
                },
            ],
            IncitingEvent::MysteriousArtifact => vec![
                TutorialMessage {
                    speaker: None,
                    text: "The strange markings you found still glow faintly in your mind.".to_string(),
                    message_type: MessageType::Narration,
                },
                TutorialMessage {
                    speaker: None,
                    text: "You can almost hear them... whispering.".to_string(),
                    message_type: MessageType::Thought,
                },
            ],
            IncitingEvent::RelicDiscovered => vec![
                TutorialMessage {
                    speaker: None,
                    text: "The ancient relic pulses with a soft, warm light.".to_string(),
                    message_type: MessageType::Narration,
                },
                TutorialMessage {
                    speaker: Some(self.mentor_name.clone()),
                    text: "By the gods... do you know what you've found?".to_string(),
                    message_type: MessageType::Dialogue,
                },
            ],
            IncitingEvent::DreamVision => vec![
                TutorialMessage {
                    speaker: None,
                    text: "The dreams came again last night. Clearer this time.".to_string(),
                    message_type: MessageType::Narration,
                },
                TutorialMessage {
                    speaker: None,
                    text: "A voice calling your name. A path through darkness. A choice.".to_string(),
                    message_type: MessageType::Thought,
                },
            ],
            _ => vec![
                TutorialMessage {
                    speaker: None,
                    text: "Something has changed. You can feel it.".to_string(),
                    message_type: MessageType::Narration,
                },
            ],
        }
    }

    fn incident_dialogue(&self) -> Vec<TutorialMessage> {
        match &self.inciting_event {
            IncitingEvent::BanditRaid => vec![
                TutorialMessage {
                    speaker: None,
                    text: "BANDITS! The cry goes up from the watchtower.".to_string(),
                    message_type: MessageType::Action,
                },
                TutorialMessage {
                    speaker: None,
                    text: "Armed figures pour out of the treeline, their intentions clear.".to_string(),
                    message_type: MessageType::Narration,
                },
                TutorialMessage {
                    speaker: Some(self.mentor_name.clone()),
                    text: format!("Get to safety! Protect {}!", self.family_member),
                    message_type: MessageType::Dialogue,
                },
            ],
            IncitingEvent::StrangeTraveler => vec![
                TutorialMessage {
                    speaker: Some("Stranger".to_string()),
                    text: "I seek the one marked by fate. The dreams led me here.".to_string(),
                    message_type: MessageType::Dialogue,
                },
                TutorialMessage {
                    speaker: None,
                    text: "The stranger's eyes find yours. You feel a shock of recognition.".to_string(),
                    message_type: MessageType::Narration,
                },
                TutorialMessage {
                    speaker: Some("Stranger".to_string()),
                    text: "Ah. There you are. I've traveled far to find you.".to_string(),
                    message_type: MessageType::Dialogue,
                },
            ],
            _ => vec![
                TutorialMessage {
                    speaker: None,
                    text: "Everything changes in an instant.".to_string(),
                    message_type: MessageType::Narration,
                },
            ],
        }
    }

    fn movement_dialogue(&self) -> Vec<TutorialMessage> {
        vec![
            TutorialMessage {
                speaker: None,
                text: "Use WASD or arrow keys to move around.".to_string(),
                message_type: MessageType::Tutorial,
            },
            TutorialMessage {
                speaker: None,
                text: "Try moving to explore the area.".to_string(),
                message_type: MessageType::Tutorial,
            },
        ]
    }

    fn npc_dialogue(&self) -> Vec<TutorialMessage> {
        vec![
            TutorialMessage {
                speaker: None,
                text: "Press ENTER when standing next to someone to talk to them.".to_string(),
                message_type: MessageType::Tutorial,
            },
            TutorialMessage {
                speaker: Some(self.mentor_name.clone()),
                text: "Come here, child. We need to talk about what's happening.".to_string(),
                message_type: MessageType::Dialogue,
            },
        ]
    }

    fn call_dialogue(&self) -> Vec<TutorialMessage> {
        vec![
            TutorialMessage {
                speaker: Some(self.mentor_name.clone()),
                text: format!("The village cannot offer you what you seek. {} will be fine here.", self.family_member),
                message_type: MessageType::Dialogue,
            },
            TutorialMessage {
                speaker: Some(self.mentor_name.clone()),
                text: "But you... you were meant for something more.".to_string(),
                message_type: MessageType::Dialogue,
            },
            TutorialMessage {
                speaker: None,
                text: "Your hands tremble. Not from fear, but anticipation.".to_string(),
                message_type: MessageType::Narration,
            },
            TutorialMessage {
                speaker: None,
                text: "This is it. The moment everything changes.".to_string(),
                message_type: MessageType::Thought,
            },
        ]
    }

    fn combat_dialogue(&self) -> Vec<TutorialMessage> {
        vec![
            TutorialMessage {
                speaker: None,
                text: "Combat begins! Select your action with the arrow keys.".to_string(),
                message_type: MessageType::Tutorial,
            },
            TutorialMessage {
                speaker: None,
                text: "Press ENTER to confirm your choice.".to_string(),
                message_type: MessageType::Tutorial,
            },
            TutorialMessage {
                speaker: None,
                text: "A hostile figure blocks your path!".to_string(),
                message_type: MessageType::Action,
            },
        ]
    }

    fn farewell_dialogue(&self) -> Vec<TutorialMessage> {
        vec![
            TutorialMessage {
                speaker: Some(self.family_member.clone()),
                text: "Must you go?".to_string(),
                message_type: MessageType::Dialogue,
            },
            TutorialMessage {
                speaker: None,
                text: "You look back at your home one last time.".to_string(),
                message_type: MessageType::Narration,
            },
            TutorialMessage {
                speaker: Some(self.family_member.clone()),
                text: "Then go. And come back to us someday, a hero.".to_string(),
                message_type: MessageType::Dialogue,
            },
            TutorialMessage {
                speaker: Some(self.mentor_name.clone()),
                text: "Take this. It belonged to my adventuring days.".to_string(),
                message_type: MessageType::Dialogue,
            },
            TutorialMessage {
                speaker: None,
                text: "You receive a worn but serviceable weapon!".to_string(),
                message_type: MessageType::Action,
            },
        ]
    }

    fn departure_dialogue(&self) -> Vec<TutorialMessage> {
        vec![
            TutorialMessage {
                speaker: None,
                text: format!("You walk down the road, leaving {} behind.", self.village_name),
                message_type: MessageType::Narration,
            },
            TutorialMessage {
                speaker: None,
                text: "The road stretches before you, full of possibility.".to_string(),
                message_type: MessageType::Narration,
            },
            TutorialMessage {
                speaker: None,
                text: "Your adventure begins NOW.".to_string(),
                message_type: MessageType::Action,
            },
            TutorialMessage {
                speaker: None,
                text: "Tutorial complete! You are now free to explore the world.".to_string(),
                message_type: MessageType::Tutorial,
            },
        ]
    }

    pub fn advance_stage(&mut self) -> bool {
        self.dialogue_index = 0;
        self.messages.clear();

        let next_stage = match self.stage {
            TutorialStage::DailyLife => Some(TutorialStage::Unease),
            TutorialStage::Unease => Some(TutorialStage::Incident),
            TutorialStage::Incident => Some(TutorialStage::MovementLesson),
            TutorialStage::MovementLesson if self.has_moved => Some(TutorialStage::NPCInteraction),
            TutorialStage::NPCInteraction if self.has_talked => Some(TutorialStage::TheCall),
            TutorialStage::TheCall => Some(TutorialStage::FirstCombat),
            TutorialStage::FirstCombat if self.has_fought => Some(TutorialStage::Farewell),
            TutorialStage::Farewell => Some(TutorialStage::Departure),
            TutorialStage::Departure => Some(TutorialStage::Complete),
            _ => None,
        };

        if let Some(stage) = next_stage {
            self.stage = stage;
            true
        } else {
            false
        }
    }

    pub fn is_complete(&self) -> bool {
        self.stage == TutorialStage::Complete
    }

    pub fn mark_moved(&mut self) {
        self.has_moved = true;
    }

    pub fn mark_talked(&mut self) {
        self.has_talked = true;
    }

    pub fn mark_fought(&mut self) {
        self.has_fought = true;
    }

    pub fn mark_searched(&mut self) {
        self.has_searched = true;
    }

    pub fn mark_used_inventory(&mut self) {
        self.has_used_inventory = true;
    }

    pub fn get_tutorial_hint(&self) -> Option<&'static str> {
        match self.stage {
            TutorialStage::MovementLesson if !self.has_moved => {
                Some("Use WASD or arrow keys to move around")
            }
            TutorialStage::NPCInteraction if !self.has_talked => {
                Some("Press ENTER near an NPC to talk")
            }
            TutorialStage::FirstCombat if !self.has_fought => {
                Some("Select Attack and press ENTER to fight")
            }
            _ => None,
        }
    }
}

/// Generates the tutorial village map
#[derive(Debug, Clone)]
pub struct TutorialVillage {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<TutorialTile>>,
    pub npcs: Vec<TutorialNPC>,
    pub player_start: (usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TutorialTile {
    Grass,
    Path,
    Wall,
    Door,
    Water,
    Tree,
    Building,
    WorkArea, // Where the player's job is
}

impl TutorialTile {
    pub fn char(&self) -> char {
        match self {
            TutorialTile::Grass => '.',
            TutorialTile::Path => '#',
            TutorialTile::Wall => '█',
            TutorialTile::Door => '+',
            TutorialTile::Water => '~',
            TutorialTile::Tree => 'T',
            TutorialTile::Building => '▓',
            TutorialTile::WorkArea => '○',
        }
    }

    pub fn is_walkable(&self) -> bool {
        match self {
            TutorialTile::Grass | TutorialTile::Path | TutorialTile::Door | TutorialTile::WorkArea => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TutorialNPC {
    pub name: String,
    pub role: String,
    pub position: (usize, usize),
    pub dialogue: Vec<String>,
    pub is_mentor: bool,
    pub is_family: bool,
}

impl TutorialVillage {
    pub fn generate(occupation: &Occupation) -> Self {
        let width = 40;
        let height = 25;
        let mut tiles = vec![vec![TutorialTile::Grass; width]; height];

        // Create village paths
        for x in 10..30 {
            tiles[12][x] = TutorialTile::Path; // Main road
        }
        for y in 5..20 {
            tiles[y][20] = TutorialTile::Path; // Cross road
        }

        // Add buildings
        Self::add_building(&mut tiles, 5, 8, 6, 5);  // Home
        Self::add_building(&mut tiles, 22, 8, 8, 6); // Tavern
        Self::add_building(&mut tiles, 15, 15, 5, 4); // Shop
        Self::add_building(&mut tiles, 28, 14, 6, 5); // Workshop

        // Add work area based on occupation
        let work_pos = match occupation {
            Occupation::Farmer => (3, 18),
            Occupation::Blacksmith => (30, 16),
            Occupation::Barkeep => (24, 10),
            Occupation::Fisher => (35, 22),
            _ => (20, 20),
        };
        tiles[work_pos.1][work_pos.0] = TutorialTile::WorkArea;

        // Add trees
        for _ in 0..15 {
            let x = rand::random::<usize>() % width;
            let y = rand::random::<usize>() % height;
            if tiles[y][x] == TutorialTile::Grass {
                tiles[y][x] = TutorialTile::Tree;
            }
        }

        // Add water (small pond)
        for y in 20..23 {
            for x in 2..6 {
                tiles[y][x] = TutorialTile::Water;
            }
        }

        let player_start = (work_pos.0, work_pos.1.saturating_sub(1));

        let npcs = vec![
            TutorialNPC {
                name: "Village Elder".to_string(),
                role: "Mentor".to_string(),
                position: (16, 16),
                dialogue: vec!["The world is changing, child.".to_string()],
                is_mentor: true,
                is_family: false,
            },
            TutorialNPC {
                name: "Family Member".to_string(),
                role: "Family".to_string(),
                position: (7, 10),
                dialogue: vec!["Be safe out there.".to_string()],
                is_mentor: false,
                is_family: true,
            },
            TutorialNPC {
                name: "Villager".to_string(),
                role: "Villager".to_string(),
                position: (25, 11),
                dialogue: vec!["Strange times, these.".to_string()],
                is_mentor: false,
                is_family: false,
            },
        ];

        TutorialVillage {
            width,
            height,
            tiles,
            npcs,
            player_start,
        }
    }

    fn add_building(tiles: &mut Vec<Vec<TutorialTile>>, x: usize, y: usize, w: usize, h: usize) {
        for dy in 0..h {
            for dx in 0..w {
                let ty = y + dy;
                let tx = x + dx;
                if ty < tiles.len() && tx < tiles[0].len() {
                    if dy == 0 || dy == h - 1 || dx == 0 || dx == w - 1 {
                        tiles[ty][tx] = TutorialTile::Wall;
                    } else {
                        tiles[ty][tx] = TutorialTile::Building;
                    }
                }
            }
        }
        // Add door
        let door_y = y + h / 2;
        let door_x = x + w / 2;
        if door_y < tiles.len() && door_x < tiles[0].len() {
            tiles[door_y][door_x] = TutorialTile::Door;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tutorial_creation() {
        let state = TutorialState::new(Occupation::Farmer);
        assert_eq!(state.stage, TutorialStage::DailyLife);
        assert_eq!(state.village_name, "Millbrook");
    }

    #[test]
    fn test_stage_advancement() {
        let mut state = TutorialState::new(Occupation::Farmer);
        assert!(state.advance_stage());
        assert_eq!(state.stage, TutorialStage::Unease);
    }

    #[test]
    fn test_village_generation() {
        let village = TutorialVillage::generate(&Occupation::Farmer);
        assert_eq!(village.width, 40);
        assert_eq!(village.height, 25);
        assert!(!village.npcs.is_empty());
    }
}

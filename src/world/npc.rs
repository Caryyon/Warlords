use serde::{Deserialize, Serialize};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use super::{LocalCoord, TerrainMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPC {
    pub name: String,
    pub npc_type: NPCType,
    pub position: LocalCoord,
    pub dialogue: Vec<String>,
    pub disposition: NPCDisposition,
    pub inventory: Vec<String>,
    pub services: Vec<NPCService>,
    pub level: u8,
    pub faction: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NPCType {
    Merchant,
    Guard,
    Traveler,
    Hermit,
    Scholar,
    Warrior,
    Thief,
    Farmer,
    Noble,
    Blacksmith,
    Innkeeper,
    Priest,
    Ranger,
    Bandit,
    Explorer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NPCDisposition {
    Friendly,
    Neutral,
    Wary,
    Hostile,
    Fearful,
    Greedy,
    Helpful,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NPCService {
    Trade,
    Information,
    Quests,
    Healing,
    Training,
    Repair,
    Rest,
    Storage,
}

pub struct NPCGenerator {
    names: Vec<&'static str>,
    surnames: Vec<&'static str>,
}

impl NPCGenerator {
    pub fn new() -> Self {
        Self {
            names: vec![
                "Aldric", "Brina", "Cael", "Dara", "Ewan", "Fira", "Gareth", "Hilda", "Ivan", "Jora",
                "Kael", "Lyra", "Magnus", "Nira", "Osric", "Petra", "Quinn", "Raven", "Soren", "Tara",
                "Ulric", "Vera", "Willem", "Xara", "Yorick", "Zara", "Bjorn", "Cora", "Dain", "Eira",
                "Finn", "Gilda", "Hakon", "Inga", "Jarl", "Kira", "Lars", "Mira", "Norn", "Olga",
            ],
            surnames: vec![
                "Ironforge", "Stormwind", "Goldleaf", "Shadowbane", "Brightblade", "Darkwood", "Swiftarrow",
                "Stoneheart", "Flamestrike", "Iceshield", "Earthshaker", "Windwalker", "Bloodfang", "Silvermoon",
                "Dragonborn", "Wolfbane", "Eagleeye", "Bearclaw", "Lionheart", "Foxglove", "Ravenwood",
                "Thornfield", "Rosehip", "Oakenhand", "Willowbend", "Ashfall", "Cinderspark", "Frostbite",
                "Sunburst", "Moonglow", "Starfire", "Nightfall", "Dawnbreaker", "Twilight", "Stormcrow",
            ],
        }
    }

    pub fn generate_npcs_for_zone(&self, terrain: &TerrainMap, settlement_count: usize, rng: &mut ChaCha8Rng) -> Vec<NPC> {
        let mut npcs = Vec::new();
        
        // Base NPC count on settlement presence and terrain variety
        let base_npc_count = match settlement_count {
            0 => rng.gen_range(1..=3),      // Wilderness encounters
            1 => rng.gen_range(3..=6),      // Small settlement
            2 => rng.gen_range(5..=10),     // Multiple settlements
            _ => rng.gen_range(8..=15),     // Major trade area
        };

        for _ in 0..base_npc_count {
            if let Some(npc) = self.generate_random_npc(terrain, rng) {
                npcs.push(npc);
            }
        }

        npcs
    }

    fn generate_random_npc(&self, terrain: &TerrainMap, rng: &mut ChaCha8Rng) -> Option<NPC> {
        // Find a suitable spawn location
        let mut attempts = 0;
        while attempts < 50 {
            let x = rng.gen_range(1..(terrain.width - 1));
            let y = rng.gen_range(1..(terrain.height - 1));
            let position = LocalCoord::new(x, y);
            let tile = terrain.get_tile(position);

            // Don't spawn in water, mountains, or other impassable terrain
            if matches!(tile.terrain_type, 
                crate::world::TerrainType::Ocean | 
                crate::world::TerrainType::Lake | 
                crate::world::TerrainType::Mountain
            ) {
                attempts += 1;
                continue;
            }

            let name = self.generate_name(rng);
            let npc_type = self.determine_npc_type(&tile.terrain_type, rng);
            let disposition = self.generate_disposition(&npc_type, rng);
            let dialogue = self.generate_dialogue(&npc_type, &disposition);
            let inventory = self.generate_inventory(&npc_type, rng);
            let services = self.generate_services(&npc_type);
            let level = rng.gen_range(1..=10);
            let faction = self.determine_faction(&npc_type, rng);

            return Some(NPC {
                name,
                npc_type,
                position,
                dialogue,
                disposition,
                inventory,
                services,
                level,
                faction,
            });
        }
        None
    }

    fn generate_name(&self, rng: &mut ChaCha8Rng) -> String {
        let first_name = self.names[rng.gen_range(0..self.names.len())];
        let surname = self.surnames[rng.gen_range(0..self.surnames.len())];
        format!("{} {}", first_name, surname)
    }

    fn determine_npc_type(&self, terrain_type: &crate::world::TerrainType, rng: &mut ChaCha8Rng) -> NPCType {
        match terrain_type {
            crate::world::TerrainType::Forest => {
                match rng.gen_range(0..4) {
                    0 => NPCType::Ranger,
                    1 => NPCType::Hermit,
                    2 => NPCType::Traveler,
                    _ => NPCType::Explorer,
                }
            }
            crate::world::TerrainType::Plains | crate::world::TerrainType::Grassland => {
                match rng.gen_range(0..6) {
                    0 => NPCType::Farmer,
                    1 => NPCType::Merchant,
                    2 => NPCType::Traveler,
                    3 => NPCType::Guard,
                    4 => NPCType::Warrior,
                    _ => NPCType::Noble,
                }
            }
            crate::world::TerrainType::Hill => {
                match rng.gen_range(0..4) {
                    0 => NPCType::Blacksmith,
                    1 => NPCType::Explorer,
                    2 => NPCType::Warrior,
                    _ => NPCType::Hermit,
                }
            }
            crate::world::TerrainType::Desert => {
                match rng.gen_range(0..4) {
                    0 => NPCType::Traveler,
                    1 => NPCType::Scholar,
                    2 => NPCType::Bandit,
                    _ => NPCType::Explorer,
                }
            }
            crate::world::TerrainType::Swamp => {
                match rng.gen_range(0..3) {
                    0 => NPCType::Hermit,
                    1 => NPCType::Thief,
                    _ => NPCType::Scholar,
                }
            }
            _ => {
                match rng.gen_range(0..6) {
                    0 => NPCType::Traveler,
                    1 => NPCType::Explorer,
                    2 => NPCType::Merchant,
                    3 => NPCType::Scholar,
                    4 => NPCType::Warrior,
                    _ => NPCType::Hermit,
                }
            }
        }
    }

    fn generate_disposition(&self, npc_type: &NPCType, rng: &mut ChaCha8Rng) -> NPCDisposition {
        match npc_type {
            NPCType::Merchant => match rng.gen_range(0..3) {
                0 => NPCDisposition::Friendly,
                1 => NPCDisposition::Greedy,
                _ => NPCDisposition::Neutral,
            },
            NPCType::Guard => match rng.gen_range(0..3) {
                0 => NPCDisposition::Wary,
                1 => NPCDisposition::Neutral,
                _ => NPCDisposition::Helpful,
            },
            NPCType::Bandit => match rng.gen_range(0..3) {
                0 => NPCDisposition::Hostile,
                1 => NPCDisposition::Wary,
                _ => NPCDisposition::Greedy,
            },
            NPCType::Priest => match rng.gen_range(0..2) {
                0 => NPCDisposition::Helpful,
                _ => NPCDisposition::Friendly,
            },
            NPCType::Hermit => match rng.gen_range(0..3) {
                0 => NPCDisposition::Wary,
                1 => NPCDisposition::Neutral,
                _ => NPCDisposition::Fearful,
            },
            _ => match rng.gen_range(0..4) {
                0 => NPCDisposition::Friendly,
                1 => NPCDisposition::Neutral,
                2 => NPCDisposition::Wary,
                _ => NPCDisposition::Helpful,
            },
        }
    }

    fn generate_dialogue(&self, npc_type: &NPCType, disposition: &NPCDisposition) -> Vec<String> {
        let mut dialogue = Vec::new();
        
        // Greeting based on disposition
        let greeting = match disposition {
            NPCDisposition::Friendly => "Greetings, traveler! How may I help you?",
            NPCDisposition::Hostile => "What do you want? State your business quickly!",
            NPCDisposition::Wary => "Who goes there? What brings you to these parts?",
            NPCDisposition::Fearful => "P-please don't hurt me! I don't have much!",
            NPCDisposition::Helpful => "Welcome, friend! I'm always happy to assist fellow travelers.",
            NPCDisposition::Greedy => "Ah, a customer! I have many fine wares to offer... for the right price.",
            NPCDisposition::Neutral => "Good day. Is there something you need?",
        };
        dialogue.push(greeting.to_string());

        // NPC type-specific dialogue
        match npc_type {
            NPCType::Merchant => {
                dialogue.push("I travel these roads trading goods between settlements.".to_string());
                dialogue.push("Perhaps you'd be interested in my wares?".to_string());
            }
            NPCType::Guard => {
                dialogue.push("I keep watch over this area for bandits and monsters.".to_string());
                dialogue.push("The roads have been dangerous lately. Stay vigilant.".to_string());
            }
            NPCType::Scholar => {
                dialogue.push("I'm researching the ancient history of this region.".to_string());
                dialogue.push("Have you seen any old ruins or artifacts in your travels?".to_string());
            }
            NPCType::Hermit => {
                dialogue.push("I live alone in these wilderness, far from the troubles of civilization.".to_string());
                dialogue.push("The land speaks to those who know how to listen.".to_string());
            }
            NPCType::Ranger => {
                dialogue.push("I know these lands like the back of my hand.".to_string());
                dialogue.push("The wildlife has been restless lately. Something's stirring.".to_string());
            }
            NPCType::Bandit => {
                dialogue.push("Your coin or your life, stranger!".to_string());
                dialogue.push("These roads are under our protection... for a fee.".to_string());
            }
            _ => {
                dialogue.push("Life in these parts isn't easy, but we make do.".to_string());
                dialogue.push("Safe travels, stranger.".to_string());
            }
        }

        dialogue
    }

    fn generate_inventory(&self, npc_type: &NPCType, rng: &mut ChaCha8Rng) -> Vec<String> {
        let mut inventory = Vec::new();
        
        match npc_type {
            NPCType::Merchant => {
                inventory.extend([
                    "Health Potion".to_string(),
                    "Iron Sword".to_string(),
                    "Leather Armor".to_string(),
                    "Rations".to_string(),
                    "Map".to_string(),
                ]);
                if rng.gen_bool(0.3) {
                    inventory.push("Magic Amulet".to_string());
                }
            }
            NPCType::Blacksmith => {
                inventory.extend([
                    "Iron Sword".to_string(),
                    "Steel Hammer".to_string(),
                    "Chain Mail".to_string(),
                    "Iron Ingot".to_string(),
                ]);
            }
            NPCType::Priest => {
                inventory.extend([
                    "Blessing Scroll".to_string(),
                    "Holy Water".to_string(),
                    "Health Potion".to_string(),
                ]);
            }
            NPCType::Ranger => {
                inventory.extend([
                    "Hunting Bow".to_string(),
                    "Arrows".to_string(),
                    "Herbal Remedy".to_string(),
                    "Tracking Guide".to_string(),
                ]);
            }
            NPCType::Bandit => {
                inventory.extend([
                    "Rusty Sword".to_string(),
                    "Stolen Goods".to_string(),
                    "Lockpicks".to_string(),
                ]);
            }
            _ => {
                if rng.gen_bool(0.5) {
                    inventory.push("Rations".to_string());
                }
                if rng.gen_bool(0.3) {
                    inventory.push("Coin Purse".to_string());
                }
            }
        }
        
        inventory
    }

    fn generate_services(&self, npc_type: &NPCType) -> Vec<NPCService> {
        match npc_type {
            NPCType::Merchant => vec![NPCService::Trade, NPCService::Information],
            NPCType::Blacksmith => vec![NPCService::Trade, NPCService::Repair],
            NPCType::Priest => vec![NPCService::Healing, NPCService::Quests],
            NPCType::Innkeeper => vec![NPCService::Rest, NPCService::Storage, NPCService::Information],
            NPCType::Scholar => vec![NPCService::Information, NPCService::Training],
            NPCType::Guard => vec![NPCService::Information, NPCService::Quests],
            NPCType::Ranger => vec![NPCService::Information, NPCService::Training],
            _ => vec![NPCService::Information],
        }
    }

    fn determine_faction(&self, npc_type: &NPCType, rng: &mut ChaCha8Rng) -> String {
        let factions = match npc_type {
            NPCType::Guard | NPCType::Noble => vec!["Royal Guard", "City Watch", "Local Militia"],
            NPCType::Bandit | NPCType::Thief => vec!["Shadowclaw Gang", "Red Daggers", "Freelance"],
            NPCType::Priest => vec!["Temple of Light", "Order of the Sun", "Independent"],
            NPCType::Merchant => vec!["Merchant Guild", "Traveling Traders", "Independent"],
            NPCType::Scholar => vec!["Academy of Lore", "Royal Library", "Independent Scholar"],
            _ => vec!["Independent", "Local Folk", "Neutral"],
        };
        
        factions[rng.gen_range(0..factions.len())].to_string()
    }
}

impl NPCType {
    pub fn get_ascii_char(&self) -> char {
        match self {
            NPCType::Merchant => 'M',
            NPCType::Guard => 'G',
            NPCType::Traveler => 'T',
            NPCType::Hermit => 'H',
            NPCType::Scholar => 'S',
            NPCType::Warrior => 'W',
            NPCType::Thief => 't',
            NPCType::Farmer => 'F',
            NPCType::Noble => 'N',
            NPCType::Blacksmith => 'B',
            NPCType::Innkeeper => 'I',
            NPCType::Priest => 'P',
            NPCType::Ranger => 'R',
            NPCType::Bandit => '!',
            NPCType::Explorer => 'E',
        }
    }

    pub fn get_color(&self) -> &'static str {
        match self {
            NPCType::Merchant => "yellow",
            NPCType::Guard => "blue",
            NPCType::Traveler => "green",
            NPCType::Hermit => "dark_gray",
            NPCType::Scholar => "cyan",
            NPCType::Warrior => "red",
            NPCType::Thief => "dark_green",
            NPCType::Farmer => "brown",
            NPCType::Noble => "magenta",
            NPCType::Blacksmith => "gray",
            NPCType::Innkeeper => "yellow",
            NPCType::Priest => "white",
            NPCType::Ranger => "green",
            NPCType::Bandit => "red",
            NPCType::Explorer => "cyan",
        }
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use noise::Perlin;

pub mod terrain;
pub mod settlement;
pub mod road;
pub mod river;
pub mod npc;
pub mod persistence;
pub mod display;
pub mod dungeon;

pub use terrain::*;
pub use settlement::*;
pub use road::*;
pub use river::*;
pub use npc::*;
pub use persistence::*;
pub use display::*;
pub use dungeon::*;

/// World coordinates - each zone is ZONE_SIZE x ZONE_SIZE tiles
pub const ZONE_SIZE: i32 = 64;  // Reduced from 512 for better performance
pub const TILE_SIZE: f64 = 1.0; // meters per tile

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZoneCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalCoord {
    pub x: i32,
    pub y: i32,
}

impl ZoneCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    pub fn from_world(world_coord: WorldCoord) -> Self {
        Self {
            x: world_coord.x.div_euclid(ZONE_SIZE),
            y: world_coord.y.div_euclid(ZONE_SIZE),
        }
    }
    
    pub fn adjacent_zones(&self) -> Vec<ZoneCoord> {
        vec![
            ZoneCoord::new(self.x - 1, self.y - 1),
            ZoneCoord::new(self.x, self.y - 1),
            ZoneCoord::new(self.x + 1, self.y - 1),
            ZoneCoord::new(self.x - 1, self.y),
            ZoneCoord::new(self.x + 1, self.y),
            ZoneCoord::new(self.x - 1, self.y + 1),
            ZoneCoord::new(self.x, self.y + 1),
            ZoneCoord::new(self.x + 1, self.y + 1),
        ]
    }
}

impl WorldCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    pub fn to_zone(&self) -> ZoneCoord {
        ZoneCoord::from_world(*self)
    }
    
    pub fn to_local(&self) -> LocalCoord {
        LocalCoord {
            x: self.x.rem_euclid(ZONE_SIZE),
            y: self.y.rem_euclid(ZONE_SIZE),
        }
    }
    
    pub fn from_zone_local(zone: ZoneCoord, local: LocalCoord) -> Self {
        Self {
            x: zone.x * ZONE_SIZE + local.x,
            y: zone.y * ZONE_SIZE + local.y,
        }
    }
    
    pub fn distance(&self, other: &WorldCoord) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

impl LocalCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldZone {
    pub coord: ZoneCoord,
    pub terrain: TerrainMap,
    pub settlements: Vec<Settlement>,
    pub roads: RoadNetwork,
    pub rivers: Vec<River>,
    pub npcs: Vec<NPC>,
    pub points_of_interest: Vec<PointOfInterest>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub last_visited: Option<chrono::DateTime<chrono::Utc>>,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointOfInterest {
    pub position: LocalCoord,
    pub poi_type: PoiType,
    pub name: String,
    pub description: String,
    pub explored: bool,
    pub treasure: Option<Treasure>,
    pub encounter: Option<Encounter>,
    pub difficulty: u8, // 1-10 difficulty rating
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treasure {
    pub items: Vec<String>,
    pub gold: u32,
    pub experience: u32,
    pub hidden: bool, // Requires searching to find
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encounter {
    pub encounter_type: EncounterType,
    pub description: String,
    pub challenge_rating: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncounterType {
    Combat(Vec<String>), // Enemy types
    Puzzle(String),      // Puzzle description
    Trap(String),        // Trap type
    Discovery(String),   // Something to discover
    NPC(String),         // Special NPC encounter
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoiType {
    AncientRuins,
    Cave,
    AbandonedTower,
    MysticShrine,
    Bridge,
    Ford,
    AbandonedMine,
    Quarry,
    Battlefield,
    Cemetery,
    DragonLair,
    BanditCamp,
    WizardTower,
    Temple,
    Crypt,
    Library,
    Laboratory,
    TreasureVault,
}

pub struct WorldGenerator {
    master_seed: u64,
    terrain_noise: Perlin,
    moisture_noise: Perlin,
    temperature_noise: Perlin,
}

impl WorldGenerator {
    pub fn new(master_seed: u64) -> Self {
        Self {
            master_seed,
            terrain_noise: Perlin::new(master_seed as u32),
            moisture_noise: Perlin::new((master_seed.wrapping_add(1)) as u32),
            temperature_noise: Perlin::new((master_seed.wrapping_add(2)) as u32),
        }
    }
    
    pub fn generate_zone(&self, coord: ZoneCoord, adjacent_zones: &HashMap<ZoneCoord, WorldZone>) -> WorldZone {
        let zone_seed = self.calculate_zone_seed(coord);
        let mut rng = ChaCha8Rng::seed_from_u64(zone_seed);
        
        // Generate terrain first
        let terrain = self.generate_terrain(coord, &mut rng);
        
        // Generate settlements based on terrain
        let settlements = self.generate_settlements(coord, &terrain, &mut rng);
        
        // Generate road network connecting settlements and adjacent zones
        let roads = self.generate_roads(coord, &settlements, adjacent_zones, &terrain, &mut rng);
        
        // Generate rivers
        let rivers = self.generate_rivers(coord, &terrain, &mut rng);
        
        // Generate points of interest
        let points_of_interest = self.generate_pois(coord, &terrain, &settlements, &mut rng);
        
        // Generate NPCs
        let npcs = self.generate_npcs(&terrain, &settlements, &mut rng);
        
        WorldZone {
            coord,
            terrain,
            settlements,
            roads,
            rivers,
            npcs,
            points_of_interest,
            generated_at: chrono::Utc::now(),
            last_visited: None,
            seed: zone_seed,
        }
    }
    
    fn calculate_zone_seed(&self, coord: ZoneCoord) -> u64 {
        // Use a hash function to create deterministic but pseudo-random seeds
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        
        self.master_seed.hash(&mut hasher);
        coord.x.hash(&mut hasher);
        coord.y.hash(&mut hasher);
        
        hasher.finish()
    }
    
    fn generate_terrain(&self, coord: ZoneCoord, rng: &mut ChaCha8Rng) -> TerrainMap {
        TerrainGenerator::new(&self.terrain_noise, &self.moisture_noise, &self.temperature_noise)
            .generate(coord, rng)
    }
    
    fn generate_settlements(&self, coord: ZoneCoord, terrain: &TerrainMap, rng: &mut ChaCha8Rng) -> Vec<Settlement> {
        SettlementGenerator::new().generate(coord, terrain, rng)
    }
    
    fn generate_roads(&self, coord: ZoneCoord, settlements: &[Settlement], adjacent_zones: &HashMap<ZoneCoord, WorldZone>, terrain: &TerrainMap, rng: &mut ChaCha8Rng) -> RoadNetwork {
        RoadGenerator::new().generate(coord, settlements, adjacent_zones, terrain, rng)
    }
    
    fn generate_rivers(&self, coord: ZoneCoord, terrain: &TerrainMap, rng: &mut ChaCha8Rng) -> Vec<River> {
        RiverGenerator::new().generate(coord, terrain, rng)
    }
    
    fn generate_pois(&self, coord: ZoneCoord, terrain: &TerrainMap, settlements: &[Settlement], rng: &mut ChaCha8Rng) -> Vec<PointOfInterest> {
        PoiGenerator::new().generate(coord, terrain, settlements, rng)
    }
    
    fn generate_npcs(&self, terrain: &TerrainMap, settlements: &[Settlement], rng: &mut ChaCha8Rng) -> Vec<NPC> {
        NPCGenerator::new().generate_npcs_for_zone(terrain, settlements.len(), rng)
    }
}

pub struct PoiGenerator;

impl PoiGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate(&self, _coord: ZoneCoord, terrain: &TerrainMap, settlements: &[Settlement], rng: &mut ChaCha8Rng) -> Vec<PointOfInterest> {
        let mut pois = Vec::new();
        
        // Generate 2-6 POIs per zone
        let poi_count = rng.gen_range(2..=6);
        
        for _ in 0..poi_count {
            let x = rng.gen_range(0..ZONE_SIZE);
            let y = rng.gen_range(0..ZONE_SIZE);
            let position = LocalCoord::new(x, y);
            
            // Don't place POIs too close to settlements
            let too_close = settlements.iter().any(|settlement| {
                let dx = (settlement.position.x - position.x).abs();
                let dy = (settlement.position.y - position.y).abs();
                dx < 10 && dy < 10  // Reduced from 50 for smaller zones
            });
            
            if too_close {
                continue;
            }
            
            let tile = terrain.get_tile(position);
            let poi_type = self.select_poi_type(tile, rng);
            let name = self.generate_poi_name(&poi_type, rng);
            let description = self.generate_poi_description(&poi_type, &name);
            let difficulty = self.calculate_difficulty(&poi_type, rng);
            let treasure = self.generate_treasure(&poi_type, difficulty, rng);
            let encounter = self.generate_encounter(&poi_type, difficulty, rng);
            
            pois.push(PointOfInterest {
                position,
                poi_type,
                name,
                description,
                explored: false,
                treasure,
                encounter,
                difficulty,
            });
        }
        
        pois
    }
    
    fn select_poi_type(&self, tile: &TerrainTile, rng: &mut ChaCha8Rng) -> PoiType {
        match tile.terrain_type {
            TerrainType::Mountain => {
                let options = [PoiType::Cave, PoiType::AbandonedMine, PoiType::AncientRuins, PoiType::AbandonedTower, PoiType::DragonLair];
                options[rng.gen_range(0..options.len())].clone()
            }
            TerrainType::Hill => {
                let options = [PoiType::AbandonedTower, PoiType::AncientRuins, PoiType::Quarry, PoiType::MysticShrine, PoiType::WizardTower];
                options[rng.gen_range(0..options.len())].clone()
            }
            TerrainType::Forest => {
                let options = [PoiType::AncientRuins, PoiType::MysticShrine, PoiType::Cave, PoiType::BanditCamp, PoiType::Temple];
                options[rng.gen_range(0..options.len())].clone()
            }
            TerrainType::Plains => {
                let options = [PoiType::Battlefield, PoiType::Cemetery, PoiType::AncientRuins, PoiType::Temple];
                options[rng.gen_range(0..options.len())].clone()
            }
            TerrainType::River | TerrainType::Lake => {
                let options = [PoiType::Bridge, PoiType::Ford, PoiType::MysticShrine, PoiType::Temple];
                options[rng.gen_range(0..options.len())].clone()
            }
            TerrainType::Desert => {
                let options = [PoiType::AncientRuins, PoiType::TreasureVault, PoiType::Crypt, PoiType::Library];
                options[rng.gen_range(0..options.len())].clone()
            }
            TerrainType::Swamp => {
                let options = [PoiType::Crypt, PoiType::Laboratory, PoiType::Cave, PoiType::Temple];
                options[rng.gen_range(0..options.len())].clone()
            }
            _ => {
                let options = [PoiType::AncientRuins, PoiType::MysticShrine, PoiType::Cemetery, PoiType::Temple];
                options[rng.gen_range(0..options.len())].clone()
            }
        }
    }
    
    fn generate_poi_name(&self, poi_type: &PoiType, rng: &mut ChaCha8Rng) -> String {
        let adjectives = ["Ancient", "Forgotten", "Lost", "Hidden", "Ruined", "Sacred", "Dark", "Old", "Mysterious", "Abandoned"];
        let adjective = adjectives[rng.gen_range(0..adjectives.len())];
        
        match poi_type {
            PoiType::AncientRuins => format!("{} Ruins", adjective),
            PoiType::Cave => format!("{} Cave", adjective),
            PoiType::AbandonedTower => format!("{} Tower", adjective),
            PoiType::MysticShrine => format!("{} Shrine", adjective),
            PoiType::Bridge => format!("{} Bridge", adjective),
            PoiType::Ford => format!("{} Ford", adjective),
            PoiType::AbandonedMine => format!("{} Mine", adjective),
            PoiType::Quarry => format!("{} Quarry", adjective),
            PoiType::Battlefield => format!("{} Battlefield", adjective),
            PoiType::Cemetery => format!("{} Cemetery", adjective),
            PoiType::DragonLair => format!("{} Dragon's Lair", adjective),
            PoiType::BanditCamp => format!("{} Bandit Camp", adjective),
            PoiType::WizardTower => format!("{} Wizard's Tower", adjective),
            PoiType::Temple => format!("{} Temple", adjective),
            PoiType::Crypt => format!("{} Crypt", adjective),
            PoiType::Library => format!("{} Library", adjective),
            PoiType::Laboratory => format!("{} Laboratory", adjective),
            PoiType::TreasureVault => format!("{} Treasure Vault", adjective),
        }
    }
    
    fn generate_poi_description(&self, poi_type: &PoiType, name: &str) -> String {
        match poi_type {
            PoiType::AncientRuins => format!("{} stands as a testament to a forgotten civilization, its crumbling walls hiding ancient secrets.", name),
            PoiType::Cave => format!("{} descends deep into the earth, echoing with unknown sounds and promising hidden treasures.", name),
            PoiType::AbandonedTower => format!("{} rises above the landscape, its abandoned halls may still contain valuable artifacts.", name),
            PoiType::MysticShrine => format!("{} emanates magical energy, blessed by ancient powers that may aid the worthy.", name),
            PoiType::Bridge => format!("{} spans the waters, worn by countless travelers seeking passage.", name),
            PoiType::Ford => format!("{} provides a shallow crossing for those who know its secrets.", name),
            PoiType::AbandonedMine => format!("{} was once rich with precious metals and gems, perhaps some still remain.", name),
            PoiType::Quarry => format!("{} provided stone for great constructions, its depths may hide forgotten tools and treasures.", name),
            PoiType::Battlefield => format!("{} is scarred by ancient conflicts, weapons and armor may still lie buried here.", name),
            PoiType::Cemetery => format!("{} holds the remains of those who came before, and perhaps their earthly possessions.", name),
            PoiType::DragonLair => format!("{} was once home to a mighty dragon, its hoard may still be hidden within.", name),
            PoiType::BanditCamp => format!("{} was used by brigands as a hideout, their stolen treasures may still be cached here.", name),
            PoiType::WizardTower => format!("{} belonged to a powerful mage, magical artifacts and knowledge await discovery.", name),
            PoiType::Temple => format!("{} was a place of worship, sacred relics and offerings may remain.", name),
            PoiType::Crypt => format!("{} houses the dead and their burial goods, riches await the brave.", name),
            PoiType::Library => format!("{} contains ancient knowledge and rare books, treasures of wisdom and magic.", name),
            PoiType::Laboratory => format!("{} was used for mysterious experiments, alchemical treasures may be found.", name),
            PoiType::TreasureVault => format!("{} was built specifically to hide great wealth and artifacts.", name),
        }
    }
    
    fn calculate_difficulty(&self, poi_type: &PoiType, rng: &mut ChaCha8Rng) -> u8 {
        let base_difficulty = match poi_type {
            PoiType::MysticShrine | PoiType::Bridge | PoiType::Ford => 1,
            PoiType::Cemetery | PoiType::Quarry => 2,
            PoiType::AncientRuins | PoiType::Cave | PoiType::AbandonedMine => rng.gen_range(2..=5),
            PoiType::AbandonedTower | PoiType::Temple | PoiType::BanditCamp => rng.gen_range(3..=6),
            PoiType::Battlefield | PoiType::Crypt | PoiType::Library => rng.gen_range(4..=7),
            PoiType::WizardTower | PoiType::Laboratory => rng.gen_range(5..=8),
            PoiType::DragonLair | PoiType::TreasureVault => rng.gen_range(7..=10),
        };
        base_difficulty
    }
    
    fn generate_treasure(&self, poi_type: &PoiType, difficulty: u8, rng: &mut ChaCha8Rng) -> Option<Treasure> {
        // Not all POIs have treasure
        let treasure_chance = match poi_type {
            PoiType::Bridge | PoiType::Ford => 0.1,
            PoiType::Cemetery | PoiType::Battlefield => 0.3,
            PoiType::MysticShrine | PoiType::Temple => 0.4,
            PoiType::AncientRuins | PoiType::Cave | PoiType::AbandonedMine => 0.6,
            PoiType::AbandonedTower | PoiType::BanditCamp | PoiType::Quarry => 0.7,
            PoiType::Crypt | PoiType::Library | PoiType::Laboratory => 0.8,
            PoiType::WizardTower => 0.9,
            PoiType::DragonLair | PoiType::TreasureVault => 1.0,
        };
        
        if !rng.gen_bool(treasure_chance) {
            return None;
        }
        
        let mut items = Vec::new();
        let gold = rng.gen_range(0..=difficulty as u32 * 50);
        let experience = difficulty as u32 * 25;
        let hidden = rng.gen_bool(0.4); // 40% of treasures are hidden
        
        // Generate items based on POI type and difficulty
        match poi_type {
            PoiType::WizardTower | PoiType::Library | PoiType::Laboratory => {
                items.extend(["Spell Scroll".to_string(), "Magic Potion".to_string(), "Ancient Tome".to_string()]);
                if difficulty >= 5 {
                    items.push("Staff of Power".to_string());
                }
            }
            PoiType::DragonLair => {
                items.extend(["Dragon Scale".to_string(), "Enchanted Weapon".to_string(), "Precious Gem".to_string()]);
                if difficulty >= 8 {
                    items.push("Dragon Egg".to_string());
                }
            }
            PoiType::AbandonedMine | PoiType::Quarry => {
                items.extend(["Iron Ore".to_string(), "Precious Gems".to_string(), "Mining Tools".to_string()]);
            }
            PoiType::Temple | PoiType::MysticShrine => {
                items.extend(["Holy Symbol".to_string(), "Blessed Water".to_string(), "Prayer Book".to_string()]);
            }
            PoiType::BanditCamp => {
                items.extend(["Stolen Goods".to_string(), "Weapons Cache".to_string(), "Lockpicks".to_string()]);
            }
            PoiType::Battlefield => {
                items.extend(["Ancient Weapon".to_string(), "Battle Standard".to_string(), "Armor Piece".to_string()]);
            }
            PoiType::Crypt | PoiType::Cemetery => {
                items.extend(["Burial Goods".to_string(), "Ancient Jewelry".to_string(), "Bone Artifacts".to_string()]);
            }
            _ => {
                items.extend(["Health Potion".to_string(), "Rations".to_string(), "Coin Purse".to_string()]);
            }
        }
        
        // Add more items for higher difficulty
        for _ in 0..(difficulty / 3) {
            if rng.gen_bool(0.5) {
                items.push("Rare Artifact".to_string());
            }
        }
        
        Some(Treasure {
            items,
            gold,
            experience,
            hidden,
        })
    }
    
    fn generate_encounter(&self, poi_type: &PoiType, difficulty: u8, rng: &mut ChaCha8Rng) -> Option<Encounter> {
        let encounter_chance = match poi_type {
            PoiType::Bridge | PoiType::Ford | PoiType::MysticShrine => 0.2,
            PoiType::Cemetery | PoiType::Temple => 0.3,
            PoiType::AncientRuins | PoiType::Cave => 0.5,
            PoiType::AbandonedTower | PoiType::BanditCamp => 0.7,
            PoiType::Crypt | PoiType::WizardTower | PoiType::Laboratory => 0.8,
            PoiType::DragonLair | PoiType::Battlefield => 0.9,
            _ => 0.4,
        };
        
        if !rng.gen_bool(encounter_chance) {
            return None;
        }
        
        let encounter_type = match poi_type {
            PoiType::DragonLair => {
                EncounterType::Combat(vec!["Young Dragon".to_string(), "Drake".to_string()])
            }
            PoiType::BanditCamp => {
                EncounterType::Combat(vec!["Bandit Leader".to_string(), "Bandits".to_string()])
            }
            PoiType::Crypt | PoiType::Cemetery => {
                EncounterType::Combat(vec!["Skeleton Warrior".to_string(), "Undead".to_string()])
            }
            PoiType::Cave => {
                if rng.gen_bool(0.5) {
                    EncounterType::Combat(vec!["Cave Bear".to_string(), "Goblins".to_string()])
                } else {
                    EncounterType::Discovery("Hidden cave paintings revealing ancient secrets".to_string())
                }
            }
            PoiType::WizardTower | PoiType::Laboratory => {
                if rng.gen_bool(0.4) {
                    EncounterType::Puzzle("Ancient magical riddle that guards powerful secrets".to_string())
                } else {
                    EncounterType::Trap("Magical ward that activates when disturbed".to_string())
                }
            }
            PoiType::AncientRuins => {
                match rng.gen_range(0..3) {
                    0 => EncounterType::Puzzle("Stone mechanism requiring ancient knowledge".to_string()),
                    1 => EncounterType::Trap("Collapsing floor concealing a hidden chamber".to_string()),
                    _ => EncounterType::Discovery("Hieroglyphs telling the story of the lost civilization".to_string()),
                }
            }
            PoiType::Temple | PoiType::MysticShrine => {
                if rng.gen_bool(0.6) {
                    EncounterType::NPC("Ancient Guardian Spirit".to_string())
                } else {
                    EncounterType::Discovery("Sacred blessing that enhances your abilities".to_string())
                }
            }
            _ => {
                match rng.gen_range(0..3) {
                    0 => EncounterType::Combat(vec!["Wild Animals".to_string()]),
                    1 => EncounterType::Discovery("Useful information about the area".to_string()),
                    _ => EncounterType::NPC("Mysterious Stranger".to_string()),
                }
            }
        };
        
        let description = match &encounter_type {
            EncounterType::Combat(enemies) => format!("You encounter hostile {}!", enemies.join(" and ")),
            EncounterType::Puzzle(desc) => format!("You discover a puzzle: {}", desc),
            EncounterType::Trap(trap) => format!("You trigger a trap: {}", trap),
            EncounterType::Discovery(discovery) => format!("You make a discovery: {}", discovery),
            EncounterType::NPC(npc) => format!("You encounter {}", npc),
        };
        
        Some(Encounter {
            encounter_type,
            description,
            challenge_rating: difficulty,
        })
    }
}

impl WorldZone {
    pub fn mark_visited(&mut self) {
        self.last_visited = Some(chrono::Utc::now());
    }
    
    pub fn get_settlement_at(&self, position: LocalCoord) -> Option<&Settlement> {
        self.settlements.iter().find(|settlement| {
            let dx = (settlement.position.x - position.x).abs();
            let dy = (settlement.position.y - position.y).abs();
            dx < settlement.size as i32 && dy < settlement.size as i32
        })
    }
    
    pub fn get_poi_at(&self, position: LocalCoord) -> Option<&PointOfInterest> {
        self.points_of_interest.iter().find(|poi| {
            let dx = (poi.position.x - position.x).abs();
            let dy = (poi.position.y - position.y).abs();
            dx < 3 && dy < 3 // POIs have small interaction radius
        })
    }
}
use serde::{Deserialize, Serialize};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use crate::world::{LocalCoord, PoiType};

pub const DUNGEON_WIDTH: i32 = 40;
pub const DUNGEON_HEIGHT: i32 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonLayout {
    pub poi_type: PoiType,
    pub name: String,
    pub current_floor: i32,
    pub floors: HashMap<i32, DungeonFloor>,
    pub entrance_pos: LocalCoord, // Where player enters
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonFloor {
    pub floor_number: i32,
    pub tiles: Vec<Vec<DungeonTile>>,
    pub rooms: Vec<DungeonRoom>,
    pub corridors: Vec<Corridor>,
    pub stairs: Vec<Staircase>,
    pub creatures: Vec<DungeonCreature>,
    pub features: Vec<DungeonFeature>,
    pub corpses: Vec<DungeonCorpse>,
    pub loot_piles: Vec<LootPile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonTile {
    pub tile_type: DungeonTileType,
    pub visible: bool,
    pub explored: bool,
    pub light_level: u8, // 0-10, 0=pitch black, 10=bright
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DungeonTileType {
    Wall,
    Floor,
    Door(DoorState),
    Stairs(StairType),
    Water,
    Pit,
    Rubble,
    Altar,
    Chest,
    Pillar,
    Window,
    Torch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DoorState {
    Open,
    Closed,
    Locked,
    Secret, // Hidden door
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StairType {
    Up,
    Down,
    UpDown, // Spiral staircase
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonRoom {
    pub room_type: RoomType,
    pub top_left: LocalCoord,
    pub width: i32,
    pub height: i32,
    pub description: String,
    pub treasure_chest: Option<LocalCoord>,
    pub special_features: Vec<LocalCoord>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RoomType {
    EntryHall,
    GreatHall,
    Chamber,
    Library,
    Laboratory,
    Treasury,
    Armory,
    Kitchen,
    Bedroom,
    Throne,
    Crypt,
    Chapel,
    Study,
    Storage,
    GuardRoom,
    Dungeon, // Prison cells
    Cave,
    Cavern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corridor {
    pub points: Vec<LocalCoord>,
    pub width: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Staircase {
    pub position: LocalCoord,
    pub stair_type: StairType,
    pub connects_to_floor: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonCreature {
    pub position: LocalCoord,
    pub creature_type: CreatureType,
    pub name: String,
    pub health: u32,
    pub patrol_route: Vec<LocalCoord>,
    pub current_patrol_index: usize,
    pub aggro_radius: i32,
    pub movement_cooldown: u32,
    pub last_move_time: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreatureType {
    Skeleton,
    Zombie,
    Ghost,
    Rat,
    Bat,
    Spider,
    Goblin,
    Orc,
    Bandit,
    GuardianSpirit,
    WildAnimal,
    Construct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonFeature {
    pub position: LocalCoord,
    pub feature_type: FeatureType,
    pub interactable: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureType {
    Bookshelf,
    WeaponRack,
    ArmorStand,
    Cauldron,
    Crystal,
    Statue,
    Fountain,
    Lever,
    Button,
    PressurePlate,
    Trap(TrapType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrapType {
    Dart,
    Pit,
    Fire,
    Poison,
    Magic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonCorpse {
    pub position: LocalCoord,
    pub creature_type: CreatureType,
    pub name: String,
    pub decay_level: u8, // 0=fresh, 10=skeleton
    pub interactions: Vec<CorpseInteraction>,
    pub loot_generated: bool, // Prevent duplicate loot generation
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CorpseInteraction {
    Loot,           // Basic looting
    Skin,           // Animals - get hide/meat
    Harvest,        // Magical creatures - get components
    RaiseSkeleton,  // Necromancy on humanoids
    RaiseZombie,    // Necromancy on fresh corpses
    Examine,        // Learn about creature
    Burn,          // Destroy corpse
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootPile {
    pub position: LocalCoord,
    pub items: Vec<LootItem>,
    pub source: String, // What dropped it
    pub discovered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootItem {
    pub name: String,
    pub item_type: LootItemType,
    pub quantity: u32,
    pub value: u32, // In gold pieces
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LootItemType {
    Weapon,
    Armor,
    Gold,
    Gem,
    Potion,
    Scroll,
    Food,
    Hide,           // Animal skins
    Meat,           // Food from animals
    Bone,           // Skeleton/bone crafting
    SpellComponent, // Magical reagents
    Tool,
    Trinket,
}

pub struct DungeonGenerator;

impl DungeonGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate_dungeon(&self, poi_type: PoiType, poi_name: String, seed: u64) -> DungeonLayout {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let floor_count = self.determine_floor_count(&poi_type, &mut rng);
        let mut floors = HashMap::new();
        
        for floor_num in 0..floor_count {
            let floor = self.generate_floor(floor_num, &poi_type, &mut rng);
            floors.insert(floor_num, floor);
        }
        
        DungeonLayout {
            poi_type: poi_type.clone(),
            name: poi_name,
            current_floor: 0,
            floors,
            entrance_pos: LocalCoord::new(DUNGEON_WIDTH / 2, DUNGEON_HEIGHT - 2),
            seed,
        }
    }
    
    fn determine_floor_count(&self, poi_type: &PoiType, rng: &mut ChaCha8Rng) -> i32 {
        match poi_type {
            PoiType::AbandonedTower | PoiType::WizardTower => rng.gen_range(3..=7),
            PoiType::Cave | PoiType::AbandonedMine => rng.gen_range(2..=4),
            PoiType::Crypt | PoiType::TreasureVault => rng.gen_range(2..=3),
            PoiType::AncientRuins | PoiType::Temple => rng.gen_range(1..=3),
            PoiType::DragonLair => rng.gen_range(1..=2),
            _ => 1,
        }
    }
    
    fn generate_floor(&self, floor_number: i32, poi_type: &PoiType, rng: &mut ChaCha8Rng) -> DungeonFloor {
        // Initialize empty floor with walls
        let mut tiles = vec![vec![DungeonTile {
            tile_type: DungeonTileType::Wall,
            visible: false,
            explored: false,
            light_level: 0,
        }; DUNGEON_WIDTH as usize]; DUNGEON_HEIGHT as usize];
        
        let (rooms, corridors, stairs) = match poi_type {
            PoiType::AbandonedTower | PoiType::WizardTower => {
                self.generate_tower_layout(floor_number, &mut tiles, rng)
            },
            PoiType::Cave | PoiType::AbandonedMine => {
                self.generate_cave_layout(floor_number, &mut tiles, rng)
            },
            PoiType::Crypt | PoiType::TreasureVault => {
                self.generate_crypt_layout(floor_number, &mut tiles, rng)
            },
            _ => {
                self.generate_ruins_layout(floor_number, &mut tiles, rng)
            }
        };
        
        let creatures = self.generate_creatures(poi_type, &rooms, &tiles, rng);
        let features = self.generate_features(poi_type, &rooms, &tiles, rng);
        
        DungeonFloor {
            floor_number,
            tiles,
            rooms,
            corridors,
            stairs,
            creatures,
            features,
            corpses: Vec::new(), // Initially no corpses
            loot_piles: Vec::new(), // Initially no loot
        }
    }
    
    fn generate_tower_layout(&self, floor_number: i32, tiles: &mut Vec<Vec<DungeonTile>>, rng: &mut ChaCha8Rng) -> (Vec<DungeonRoom>, Vec<Corridor>, Vec<Staircase>) {
        let mut rooms = Vec::new();
        let corridors = Vec::new();
        let mut stairs = Vec::new();
        
        // Create circular tower floor
        let center_x = DUNGEON_WIDTH / 2;
        let center_y = DUNGEON_HEIGHT / 2;
        let radius = 12;
        
        // Carve out circular room
        for y in 0..DUNGEON_HEIGHT {
            for x in 0..DUNGEON_WIDTH {
                let dx = x - center_x;
                let dy = y - center_y;
                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                
                if distance <= radius as f32 {
                    tiles[y as usize][x as usize].tile_type = DungeonTileType::Floor;
                    tiles[y as usize][x as usize].light_level = 2; // Dim light
                }
            }
        }
        
        // Add room based on floor
        let room_type = match floor_number {
            0 => RoomType::EntryHall,
            1 => RoomType::GreatHall,
            2 => RoomType::Library,
            3 => RoomType::Laboratory,
            4 => RoomType::Study,
            5 => RoomType::Treasury,
            _ => RoomType::Chamber,
        };
        
        rooms.push(DungeonRoom {
            room_type,
            top_left: LocalCoord::new(center_x - radius, center_y - radius),
            width: radius * 2,
            height: radius * 2,
            description: format!("A circular chamber on floor {}", floor_number + 1),
            treasure_chest: if matches!(room_type, RoomType::Treasury) {
                Some(LocalCoord::new(center_x - 3, center_y))
            } else {
                None
            },
            special_features: Vec::new(),
        });
        
        // Add spiral staircase in center
        if floor_number > 0 {
            tiles[center_y as usize][center_x as usize].tile_type = DungeonTileType::Stairs(StairType::UpDown);
            stairs.push(Staircase {
                position: LocalCoord::new(center_x, center_y),
                stair_type: StairType::UpDown,
                connects_to_floor: Some(floor_number - 1),
            });
        }
        
        // Add entrance door on ground floor
        if floor_number == 0 {
            tiles[(DUNGEON_HEIGHT - 2) as usize][center_x as usize].tile_type = DungeonTileType::Door(DoorState::Open);
        }
        
        // Add some pillars for decoration
        for _ in 0..rng.gen_range(2..=4) {
            let pillar_x = rng.gen_range((center_x - radius + 3)..(center_x + radius - 3));
            let pillar_y = rng.gen_range((center_y - radius + 3)..(center_y + radius - 3));
            
            if tiles[pillar_y as usize][pillar_x as usize].tile_type == DungeonTileType::Floor {
                tiles[pillar_y as usize][pillar_x as usize].tile_type = DungeonTileType::Pillar;
            }
        }
        
        (rooms, corridors, stairs)
    }
    
    fn generate_cave_layout(&self, floor_number: i32, tiles: &mut Vec<Vec<DungeonTile>>, rng: &mut ChaCha8Rng) -> (Vec<DungeonRoom>, Vec<Corridor>, Vec<Staircase>) {
        let mut rooms = Vec::new();
        let mut corridors = Vec::new();
        let mut stairs = Vec::new();
        
        // Generate organic cave system
        let cavern_count = rng.gen_range(3..=6);
        
        for i in 0..cavern_count {
            let center_x = rng.gen_range(8..(DUNGEON_WIDTH - 8));
            let center_y = rng.gen_range(6..(DUNGEON_HEIGHT - 6));
            let radius_x = rng.gen_range(4..=8);
            let radius_y = rng.gen_range(3..=6);
            
            // Create irregular cavern
            for y in (center_y - radius_y)..(center_y + radius_y) {
                for x in (center_x - radius_x)..(center_x + radius_x) {
                    if y >= 0 && y < DUNGEON_HEIGHT && x >= 0 && x < DUNGEON_WIDTH {
                        let dx = (x - center_x) as f32 / radius_x as f32;
                        let dy = (y - center_y) as f32 / radius_y as f32;
                        let noise = rng.gen::<f32>() * 0.3;
                        
                        if (dx * dx + dy * dy) <= (1.0 + noise) {
                            tiles[y as usize][x as usize].tile_type = DungeonTileType::Floor;
                            tiles[y as usize][x as usize].light_level = 1; // Very dim
                        }
                    }
                }
            }
            
            let room_type = if i == 0 { RoomType::Cave } else { RoomType::Cavern };
            rooms.push(DungeonRoom {
                room_type,
                top_left: LocalCoord::new(center_x - radius_x, center_y - radius_y),
                width: radius_x * 2,
                height: radius_y * 2,
                description: format!("A natural cavern carved by water and time"),
                treasure_chest: if rng.gen_bool(0.3) {
                    Some(LocalCoord::new(center_x, center_y))
                } else {
                    None
                },
                special_features: Vec::new(),
            });
        }
        
        // Connect caverns with tunnels
        for i in 0..(rooms.len() - 1) {
            let start_x = rooms[i].top_left.x + rooms[i].width / 2;
            let start_y = rooms[i].top_left.y + rooms[i].height / 2;
            let end_x = rooms[i + 1].top_left.x + rooms[i + 1].width / 2;
            let end_y = rooms[i + 1].top_left.y + rooms[i + 1].height / 2;
            
            self.carve_tunnel(tiles, start_x, start_y, end_x, end_y);
            
            corridors.push(Corridor {
                points: vec![
                    LocalCoord::new(start_x, start_y),
                    LocalCoord::new(end_x, end_y),
                ],
                width: 2,
            });
        }
        
        // Add entrance
        tiles[(DUNGEON_HEIGHT - 2) as usize][(DUNGEON_WIDTH / 2) as usize].tile_type = DungeonTileType::Floor;
        
        // Add stairs down if not bottom floor
        if rng.gen_bool(0.7) {
            let room_idx = rng.gen_range(1..rooms.len());
            let stair_x = rooms[room_idx].top_left.x + rooms[room_idx].width / 2;
            let stair_y = rooms[room_idx].top_left.y + rooms[room_idx].height / 2;
            
            tiles[stair_y as usize][stair_x as usize].tile_type = DungeonTileType::Stairs(StairType::Down);
            stairs.push(Staircase {
                position: LocalCoord::new(stair_x, stair_y),
                stair_type: StairType::Down,
                connects_to_floor: Some(floor_number + 1),
            });
        }
        
        (rooms, corridors, stairs)
    }
    
    fn generate_crypt_layout(&self, _floor_number: i32, tiles: &mut Vec<Vec<DungeonTile>>, rng: &mut ChaCha8Rng) -> (Vec<DungeonRoom>, Vec<Corridor>, Vec<Staircase>) {
        let mut rooms = Vec::new();
        let corridors = Vec::new();
        let stairs = Vec::new();
        
        // Create main burial chamber
        let main_width = 20;
        let main_height = 12;
        let main_x = (DUNGEON_WIDTH - main_width) / 2;
        let main_y = (DUNGEON_HEIGHT - main_height) / 2;
        
        for y in main_y..(main_y + main_height) {
            for x in main_x..(main_x + main_width) {
                tiles[y as usize][x as usize].tile_type = DungeonTileType::Floor;
                tiles[y as usize][x as usize].light_level = 1;
            }
        }
        
        rooms.push(DungeonRoom {
            room_type: RoomType::Crypt,
            top_left: LocalCoord::new(main_x, main_y),
            width: main_width,
            height: main_height,
            description: "The main burial chamber, filled with ancient sarcophagi".to_string(),
            treasure_chest: Some(LocalCoord::new(main_x + main_width / 2, main_y + main_height / 2)),
            special_features: Vec::new(),
        });
        
        // Add smaller burial alcoves
        for i in 0..4 {
            let alcove_width = 6;
            let alcove_height = 4;
            let (alcove_x, alcove_y) = match i {
                0 => (main_x - alcove_width - 2, main_y + 2),
                1 => (main_x + main_width + 2, main_y + 2),
                2 => (main_x - alcove_width - 2, main_y + main_height - alcove_height - 2),
                _ => (main_x + main_width + 2, main_y + main_height - alcove_height - 2),
            };
            
            for y in alcove_y..(alcove_y + alcove_height) {
                for x in alcove_x..(alcove_x + alcove_width) {
                    if x >= 0 && x < DUNGEON_WIDTH && y >= 0 && y < DUNGEON_HEIGHT {
                        tiles[y as usize][x as usize].tile_type = DungeonTileType::Floor;
                        tiles[y as usize][x as usize].light_level = 1;
                    }
                }
            }
            
            // Connect to main chamber
            let connect_x = if i % 2 == 0 { main_x } else { main_x + main_width - 1 };
            let connect_y = alcove_y + alcove_height / 2;
            tiles[connect_y as usize][connect_x as usize].tile_type = DungeonTileType::Door(DoorState::Closed);
            
            rooms.push(DungeonRoom {
                room_type: RoomType::Crypt,
                top_left: LocalCoord::new(alcove_x, alcove_y),
                width: alcove_width,
                height: alcove_height,
                description: format!("A small burial alcove containing ancient remains"),
                treasure_chest: if rng.gen_bool(0.4) {
                    Some(LocalCoord::new(alcove_x + alcove_width / 2, alcove_y + alcove_height / 2))
                } else {
                    None
                },
                special_features: Vec::new(),
            });
        }
        
        // Add entrance
        tiles[(DUNGEON_HEIGHT - 2) as usize][(DUNGEON_WIDTH / 2) as usize].tile_type = DungeonTileType::Door(DoorState::Open);
        
        (rooms, corridors, stairs)
    }
    
    fn generate_ruins_layout(&self, _floor_number: i32, tiles: &mut Vec<Vec<DungeonTile>>, rng: &mut ChaCha8Rng) -> (Vec<DungeonRoom>, Vec<Corridor>, Vec<Staircase>) {
        let mut rooms = Vec::new();
        let corridors = Vec::new();
        let stairs = Vec::new();
        
        // Create rectangular halls and chambers
        let room_count = rng.gen_range(3..=5);
        
        for i in 0..room_count {
            let room_width = rng.gen_range(8..=12);
            let room_height = rng.gen_range(6..=8);
            let room_x = rng.gen_range(2..(DUNGEON_WIDTH - room_width - 2));
            let room_y = rng.gen_range(2..(DUNGEON_HEIGHT - room_height - 2));
            
            for y in room_y..(room_y + room_height) {
                for x in room_x..(room_x + room_width) {
                    tiles[y as usize][x as usize].tile_type = DungeonTileType::Floor;
                    tiles[y as usize][x as usize].light_level = 2;
                }
            }
            
            let room_type = match i {
                0 => RoomType::EntryHall,
                1 => RoomType::GreatHall,
                2 => RoomType::Chamber,
                _ => RoomType::Study,
            };
            
            rooms.push(DungeonRoom {
                room_type,
                top_left: LocalCoord::new(room_x, room_y),
                width: room_width,
                height: room_height,
                description: format!("An ancient stone chamber from a forgotten civilization"),
                treasure_chest: if rng.gen_bool(0.3) {
                    Some(LocalCoord::new(room_x + room_width / 2, room_y + room_height / 2))
                } else {
                    None
                },
                special_features: Vec::new(),
            });
            
            // Add some rubble for atmosphere
            for _ in 0..rng.gen_range(1..=3) {
                let rubble_x = rng.gen_range(room_x..(room_x + room_width));
                let rubble_y = rng.gen_range(room_y..(room_y + room_height));
                if rng.gen_bool(0.3) {
                    tiles[rubble_y as usize][rubble_x as usize].tile_type = DungeonTileType::Rubble;
                }
            }
        }
        
        // Connect rooms with corridors
        for i in 0..(rooms.len() - 1) {
            let start_x = rooms[i].top_left.x + rooms[i].width / 2;
            let start_y = rooms[i].top_left.y + rooms[i].height / 2;
            let end_x = rooms[i + 1].top_left.x + rooms[i + 1].width / 2;
            let end_y = rooms[i + 1].top_left.y + rooms[i + 1].height / 2;
            
            self.carve_corridor(tiles, start_x, start_y, end_x, end_y);
        }
        
        // Add entrance
        tiles[(DUNGEON_HEIGHT - 2) as usize][(DUNGEON_WIDTH / 2) as usize].tile_type = DungeonTileType::Door(DoorState::Open);
        
        (rooms, corridors, stairs)
    }
    
    fn carve_tunnel(&self, tiles: &mut Vec<Vec<DungeonTile>>, start_x: i32, start_y: i32, end_x: i32, end_y: i32) {
        let mut x = start_x;
        let mut y = start_y;
        
        while x != end_x || y != end_y {
            if x < end_x { x += 1; }
            else if x > end_x { x -= 1; }
            
            if y < end_y { y += 1; }
            else if y > end_y { y -= 1; }
            
            if x >= 0 && x < DUNGEON_WIDTH && y >= 0 && y < DUNGEON_HEIGHT {
                tiles[y as usize][x as usize].tile_type = DungeonTileType::Floor;
                tiles[y as usize][x as usize].light_level = 1;
            }
        }
    }
    
    fn carve_corridor(&self, tiles: &mut Vec<Vec<DungeonTile>>, start_x: i32, start_y: i32, end_x: i32, end_y: i32) {
        // L-shaped corridor (horizontal then vertical)
        for x in start_x.min(end_x)..=start_x.max(end_x) {
            if x >= 0 && x < DUNGEON_WIDTH && start_y >= 0 && start_y < DUNGEON_HEIGHT {
                tiles[start_y as usize][x as usize].tile_type = DungeonTileType::Floor;
                tiles[start_y as usize][x as usize].light_level = 1;
            }
        }
        
        for y in start_y.min(end_y)..=start_y.max(end_y) {
            if end_x >= 0 && end_x < DUNGEON_WIDTH && y >= 0 && y < DUNGEON_HEIGHT {
                tiles[y as usize][end_x as usize].tile_type = DungeonTileType::Floor;
                tiles[y as usize][end_x as usize].light_level = 1;
            }
        }
    }
    
    fn generate_creatures(&self, poi_type: &PoiType, rooms: &[DungeonRoom], tiles: &[Vec<DungeonTile>], rng: &mut ChaCha8Rng) -> Vec<DungeonCreature> {
        let mut creatures = Vec::new();
        let creature_count = rng.gen_range(2..=6);
        
        for _ in 0..creature_count {
            if let Some(room) = rooms.get(rng.gen_range(0..rooms.len())) {
                let x = rng.gen_range(room.top_left.x..(room.top_left.x + room.width));
                let y = rng.gen_range(room.top_left.y..(room.top_left.y + room.height));
                
                if tiles.get(y as usize).and_then(|row| row.get(x as usize))
                    .map(|tile| matches!(tile.tile_type, DungeonTileType::Floor))
                    .unwrap_or(false) {
                    
                    let creature_type = self.select_creature_type(poi_type, rng);
                    let name = self.generate_creature_name(&creature_type, rng);
                    
                    // Simple patrol route within the room
                    let patrol_route = vec![
                        LocalCoord::new(x, y),
                        LocalCoord::new(room.top_left.x + 1, room.top_left.y + 1),
                        LocalCoord::new(room.top_left.x + room.width - 2, room.top_left.y + 1),
                        LocalCoord::new(room.top_left.x + room.width - 2, room.top_left.y + room.height - 2),
                        LocalCoord::new(room.top_left.x + 1, room.top_left.y + room.height - 2),
                    ];
                    
                    creatures.push(DungeonCreature {
                        position: LocalCoord::new(x, y),
                        creature_type,
                        name,
                        health: rng.gen_range(10..=30),
                        patrol_route,
                        current_patrol_index: 0,
                        aggro_radius: rng.gen_range(3..=6),
                        movement_cooldown: rng.gen_range(3..=7),
                        last_move_time: 0,
                    });
                }
            }
        }
        
        creatures
    }
    
    fn select_creature_type(&self, poi_type: &PoiType, rng: &mut ChaCha8Rng) -> CreatureType {
        match poi_type {
            PoiType::Crypt | PoiType::Cemetery => {
                let options = [CreatureType::Skeleton, CreatureType::Zombie, CreatureType::Ghost];
                options[rng.gen_range(0..options.len())].clone()
            },
            PoiType::Cave | PoiType::AbandonedMine => {
                let options = [CreatureType::Bat, CreatureType::Spider, CreatureType::Rat, CreatureType::Goblin];
                options[rng.gen_range(0..options.len())].clone()
            },
            PoiType::BanditCamp => {
                let options = [CreatureType::Bandit, CreatureType::Goblin];
                options[rng.gen_range(0..options.len())].clone()
            },
            PoiType::WizardTower | PoiType::Laboratory => {
                let options = [CreatureType::Construct, CreatureType::GuardianSpirit];
                options[rng.gen_range(0..options.len())].clone()
            },
            PoiType::Temple | PoiType::MysticShrine => {
                let options = [CreatureType::GuardianSpirit, CreatureType::Construct];
                options[rng.gen_range(0..options.len())].clone()
            },
            _ => {
                let options = [CreatureType::Rat, CreatureType::Bat, CreatureType::Spider, CreatureType::WildAnimal];
                options[rng.gen_range(0..options.len())].clone()
            }
        }
    }
    
    fn generate_creature_name(&self, creature_type: &CreatureType, rng: &mut ChaCha8Rng) -> String {
        let adjectives = ["Ancient", "Restless", "Fierce", "Hungry", "Shadowy", "Mad"];
        let adjective = adjectives[rng.gen_range(0..adjectives.len())];
        
        match creature_type {
            CreatureType::Skeleton => format!("{} Skeleton", adjective),
            CreatureType::Zombie => format!("{} Zombie", adjective),
            CreatureType::Ghost => format!("{} Ghost", adjective),
            CreatureType::Rat => format!("{} Rat", adjective),
            CreatureType::Bat => format!("{} Bat", adjective),
            CreatureType::Spider => format!("{} Spider", adjective),
            CreatureType::Goblin => format!("{} Goblin", adjective),
            CreatureType::Orc => format!("{} Orc", adjective),
            CreatureType::Bandit => format!("{} Bandit", adjective),
            CreatureType::GuardianSpirit => format!("{} Guardian", adjective),
            CreatureType::WildAnimal => format!("{} Beast", adjective),
            CreatureType::Construct => format!("{} Golem", adjective),
        }
    }
    
    fn generate_features(&self, poi_type: &PoiType, rooms: &[DungeonRoom], tiles: &[Vec<DungeonTile>], rng: &mut ChaCha8Rng) -> Vec<DungeonFeature> {
        let mut features = Vec::new();
        
        for room in rooms {
            // Add 1-3 features per room
            let feature_count = rng.gen_range(1..=3);
            
            for _ in 0..feature_count {
                let x = rng.gen_range(room.top_left.x..(room.top_left.x + room.width));
                let y = rng.gen_range(room.top_left.y..(room.top_left.y + room.height));
                
                if tiles.get(y as usize).and_then(|row| row.get(x as usize))
                    .map(|tile| matches!(tile.tile_type, DungeonTileType::Floor))
                    .unwrap_or(false) {
                    
                    let feature_type = self.select_feature_type(poi_type, &room.room_type, rng);
                    let description = self.generate_feature_description(&feature_type);
                    
                    features.push(DungeonFeature {
                        position: LocalCoord::new(x, y),
                        feature_type,
                        interactable: true,
                        description,
                    });
                }
            }
        }
        
        features
    }
    
    fn select_feature_type(&self, poi_type: &PoiType, room_type: &RoomType, rng: &mut ChaCha8Rng) -> FeatureType {
        match (poi_type, room_type) {
            (_, RoomType::Library) => {
                let options = [FeatureType::Bookshelf, FeatureType::Statue];
                options[rng.gen_range(0..options.len())].clone()
            },
            (_, RoomType::Laboratory) => {
                let options = [FeatureType::Cauldron, FeatureType::Crystal];
                options[rng.gen_range(0..options.len())].clone()
            },
            (_, RoomType::Armory) => {
                let options = [FeatureType::WeaponRack, FeatureType::ArmorStand];
                options[rng.gen_range(0..options.len())].clone()
            },
            (PoiType::Temple | PoiType::MysticShrine, _) => {
                let options = [FeatureType::Statue, FeatureType::Fountain];
                options[rng.gen_range(0..options.len())].clone()
            },
            _ => {
                let options = [FeatureType::Statue, FeatureType::Lever, FeatureType::Crystal];
                options[rng.gen_range(0..options.len())].clone()
            }
        }
    }
    
    fn generate_feature_description(&self, feature_type: &FeatureType) -> String {
        match feature_type {
            FeatureType::Bookshelf => "A dusty bookshelf filled with ancient tomes".to_string(),
            FeatureType::WeaponRack => "A wooden rack holding various weapons".to_string(),
            FeatureType::ArmorStand => "A metal stand displaying pieces of armor".to_string(),
            FeatureType::Cauldron => "A large iron cauldron used for brewing".to_string(),
            FeatureType::Crystal => "A glowing crystal emanating magical energy".to_string(),
            FeatureType::Statue => "An ancient stone statue depicting a forgotten figure".to_string(),
            FeatureType::Fountain => "A stone fountain with crystal clear water".to_string(),
            FeatureType::Lever => "A mechanical lever built into the wall".to_string(),
            FeatureType::Button => "A stone button recessed into the floor".to_string(),
            FeatureType::PressurePlate => "A pressure-sensitive stone plate".to_string(),
            FeatureType::Trap(_) => "Something seems suspicious about this area".to_string(),
        }
    }
}

impl DungeonLayout {
    pub fn get_current_floor(&self) -> Option<&DungeonFloor> {
        self.floors.get(&self.current_floor)
    }
    
    pub fn get_current_floor_mut(&mut self) -> Option<&mut DungeonFloor> {
        self.floors.get_mut(&self.current_floor)
    }
    
    pub fn change_floor(&mut self, new_floor: i32) -> bool {
        if self.floors.contains_key(&new_floor) {
            self.current_floor = new_floor;
            true
        } else {
            false
        }
    }
    
    pub fn get_tile_at(&self, pos: LocalCoord) -> Option<&DungeonTile> {
        self.get_current_floor()?
            .tiles
            .get(pos.y as usize)?
            .get(pos.x as usize)
    }
    
    pub fn get_tile_at_mut(&mut self, pos: LocalCoord) -> Option<&mut DungeonTile> {
        self.get_current_floor_mut()?
            .tiles
            .get_mut(pos.y as usize)?
            .get_mut(pos.x as usize)
    }
    
    pub fn add_corpse(&mut self, corpse: DungeonCorpse) {
        if let Some(floor) = self.get_current_floor_mut() {
            floor.corpses.push(corpse);
        }
    }
    
    pub fn add_loot_pile(&mut self, loot_pile: LootPile) {
        if let Some(floor) = self.get_current_floor_mut() {
            floor.loot_piles.push(loot_pile);
        }
    }
}

impl DungeonCorpse {
    pub fn new(position: LocalCoord, creature_type: CreatureType, name: String) -> Self {
        let interactions = Self::get_interactions_for_creature(&creature_type);
        
        Self {
            position,
            creature_type,
            name,
            decay_level: 0, // Fresh corpse
            interactions,
            loot_generated: false,
        }
    }
    
    fn get_interactions_for_creature(creature_type: &CreatureType) -> Vec<CorpseInteraction> {
        let mut interactions = vec![CorpseInteraction::Examine, CorpseInteraction::Loot];
        
        match creature_type {
            CreatureType::Rat | CreatureType::Bat | CreatureType::WildAnimal => {
                interactions.push(CorpseInteraction::Skin);
            }
            CreatureType::Skeleton => {
                interactions.push(CorpseInteraction::Harvest); // Bone collection
            }
            CreatureType::Zombie => {
                interactions.push(CorpseInteraction::RaiseSkeleton);
                interactions.push(CorpseInteraction::Burn);
            }
            CreatureType::Ghost => {
                interactions.push(CorpseInteraction::Harvest); // Ectoplasm
            }
            CreatureType::Spider => {
                interactions.push(CorpseInteraction::Harvest); // Venom sacs
                interactions.push(CorpseInteraction::Skin); // Chitin
            }
            CreatureType::Goblin | CreatureType::Orc | CreatureType::Bandit => {
                interactions.push(CorpseInteraction::RaiseSkeleton);
                interactions.push(CorpseInteraction::RaiseZombie);
            }
            CreatureType::Construct => {
                interactions.push(CorpseInteraction::Harvest); // Magical components
            }
            _ => {}
        }
        
        interactions.push(CorpseInteraction::Burn); // Can always burn corpses
        interactions
    }
    
    pub fn generate_loot(&self) -> Vec<LootItem> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut loot = Vec::new();
        
        // Base gold drop
        let gold_amount = match self.creature_type {
            CreatureType::Bandit => rng.gen_range(5..15),
            CreatureType::Orc => rng.gen_range(3..8),
            CreatureType::Goblin => rng.gen_range(1..5),
            _ => rng.gen_range(0..3),
        };
        
        if gold_amount > 0 {
            loot.push(LootItem {
                name: "Gold Coins".to_string(),
                item_type: LootItemType::Gold,
                quantity: gold_amount,
                value: gold_amount,
                description: "Shiny gold coins".to_string(),
            });
        }
        
        // Creature-specific loot
        match self.creature_type {
            CreatureType::Rat => {
                if rng.gen_bool(0.1) { // 10% chance
                    loot.push(LootItem {
                        name: "Rat Tail".to_string(),
                        item_type: LootItemType::SpellComponent,
                        quantity: 1,
                        value: 2,
                        description: "A rat's tail, useful for certain potions".to_string(),
                    });
                }
            }
            CreatureType::Spider => {
                if rng.gen_bool(0.3) { // 30% chance
                    loot.push(LootItem {
                        name: "Spider Silk".to_string(),
                        item_type: LootItemType::SpellComponent,
                        quantity: rng.gen_range(1..4),
                        value: 5,
                        description: "Strong spider silk for crafting".to_string(),
                    });
                }
                if rng.gen_bool(0.2) { // 20% chance
                    loot.push(LootItem {
                        name: "Venom Sac".to_string(),
                        item_type: LootItemType::SpellComponent,
                        quantity: 1,
                        value: 10,
                        description: "Spider venom for poison crafting".to_string(),
                    });
                }
            }
            CreatureType::Skeleton => {
                if rng.gen_bool(0.4) { // 40% chance
                    loot.push(LootItem {
                        name: "Ancient Bone".to_string(),
                        item_type: LootItemType::Bone,
                        quantity: rng.gen_range(1..3),
                        value: 3,
                        description: "Well-preserved bone suitable for necromancy".to_string(),
                    });
                }
            }
            CreatureType::Bandit => {
                if rng.gen_bool(0.6) { // 60% chance for weapon
                    let weapons = vec!["Rusty Sword", "Wooden Club", "Iron Dagger"];
                    let weapon = weapons[rng.gen_range(0..weapons.len())];
                    loot.push(LootItem {
                        name: weapon.to_string(),
                        item_type: LootItemType::Weapon,
                        quantity: 1,
                        value: rng.gen_range(10..25),
                        description: format!("A {}", weapon.to_lowercase()),
                    });
                }
                if rng.gen_bool(0.3) { // 30% chance for armor
                    loot.push(LootItem {
                        name: "Leather Armor".to_string(),
                        item_type: LootItemType::Armor,
                        quantity: 1,
                        value: rng.gen_range(15..30),
                        description: "Worn leather armor".to_string(),
                    });
                }
            }
            _ => {}
        }
        
        loot
    }
}
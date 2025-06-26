#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use super::{ZoneCoord, LocalCoord, TerrainMap, ZONE_SIZE};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub name: String,
    pub position: LocalCoord,
    pub settlement_type: SettlementType,
    pub size: u32,
    pub population: u32,
    pub prosperity: f32,
    pub specializations: Vec<SettlementSpecialization>,
    pub buildings: Vec<Building>,
    pub established_year: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SettlementType {
    Outpost,      // 10-50 people
    Village,      // 50-300 people  
    Town,         // 300-2000 people
    City,         // 2000-10000 people
    Capital,      // 10000+ people
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettlementSpecialization {
    Farming,
    Mining,
    Logging,
    Fishing,
    Trading,
    Crafting,
    Military,
    Religious,
    Magical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub name: String,
    pub building_type: BuildingType,
    pub size: u32,
    pub condition: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildingType {
    Houses,
    Inn,
    Tavern,
    Shop,
    Blacksmith,
    Temple,
    Barracks,
    Walls,
    Tower,
    Market,
    Warehouse,
    Farm,
    Mill,
    Mine,
    Dock,
}

pub struct SettlementGenerator {
    name_prefixes: Vec<&'static str>,
    name_suffixes: Vec<&'static str>,
}

impl SettlementGenerator {
    pub fn new() -> Self {
        Self {
            name_prefixes: vec![
                "Green", "Stone", "Iron", "Gold", "Silver", "Red", "Blue", "White", "Black", "Grey",
                "North", "South", "East", "West", "High", "Low", "Old", "New", "Fair", "Dark",
                "Bright", "Deep", "Swift", "Still", "Cold", "Warm", "Rich", "Poor", "Grand", "Small",
                "Elder", "Young", "Ancient", "Hidden", "Lost", "Found", "Sacred", "Blessed", "Cursed", "Free"
            ],
            name_suffixes: vec![
                "ford", "bridge", "haven", "town", "burg", "shire", "field", "wood", "hill", "dale",
                "brook", "creek", "river", "lake", "mount", "ridge", "vale", "glen", "hollow", "grove",
                "mill", "well", "spring", "falls", "rapids", "crossing", "bend", "point", "rock", "stone",
                "gate", "wall", "keep", "hold", "watch", "guard", "rest", "end", "start", "way"
            ],
        }
    }
    
    pub fn generate(&self, _zone_coord: ZoneCoord, terrain: &TerrainMap, rng: &mut ChaCha8Rng) -> Vec<Settlement> {
        let mut settlements = Vec::new();
        
        // Simple settlement generation - just place 1-2 settlements randomly
        let settlement_count = rng.gen_range(0..=2);
        
        for i in 0..settlement_count {
            // Find a random suitable location (simplified)
            let mut attempts = 0;
            while attempts < 10 {
                let x = rng.gen_range(10..(ZONE_SIZE - 10));
                let y = rng.gen_range(10..(ZONE_SIZE - 10));
                let location = LocalCoord::new(x, y);
                let tile = terrain.get_tile(location);
                
                // Simple suitability check
                if tile.fertility > 0.3 && !matches!(tile.terrain_type, super::TerrainType::Ocean | super::TerrainType::Lake | super::TerrainType::Mountain) {
                    let settlement = self.generate_settlement(location, i == 0, terrain, rng);
                    settlements.push(settlement);
                    break;
                }
                attempts += 1;
            }
        }
        
        settlements
    }
    
    fn calculate_base_settlement_count(&self, _zone_coord: &ZoneCoord, terrain: &TerrainMap, rng: &mut ChaCha8Rng) -> usize {
        // Calculate average fertility and accessibility
        let mut total_fertility = 0.0;
        let mut accessible_tiles = 0;
        
        for x in 0..terrain.width {
            for y in 0..terrain.height {
                let tile = terrain.get_tile(LocalCoord::new(x, y));
                if tile.traversal_cost < 5.0 {
                    total_fertility += tile.fertility;
                    accessible_tiles += 1;
                }
            }
        }
        
        let avg_fertility = if accessible_tiles > 0 {
            total_fertility / accessible_tiles as f32
        } else {
            0.0
        };
        
        // Base count depends on how hospitable the terrain is
        let base_count = if avg_fertility > 0.7 {
            rng.gen_range(2..5)
        } else if avg_fertility > 0.5 {
            rng.gen_range(1..3)
        } else if avg_fertility > 0.3 {
            rng.gen_range(0..2)
        } else {
            0
        };
        
        base_count
    }
    
    fn find_best_location(&self, candidates: &[LocalCoord], existing: &[LocalCoord], terrain: &TerrainMap) -> Option<LocalCoord> {
        let mut best_location = None;
        let mut best_score = f32::NEG_INFINITY;
        
        for &candidate in candidates {
            // Skip if too close to existing settlements
            let too_close = existing.iter().any(|&existing_pos| {
                let dx = candidate.x - existing_pos.x;
                let dy = candidate.y - existing_pos.y;
                (dx * dx + dy * dy) < 20 * 20  // Reduced from 80 for smaller zones
            });
            
            if too_close {
                continue;
            }
            
            let score = self.calculate_location_score(candidate, terrain);
            if score > best_score {
                best_score = score;
                best_location = Some(candidate);
            }
        }
        
        best_location
    }
    
    fn calculate_location_score(&self, location: LocalCoord, terrain: &TerrainMap) -> f32 {
        let tile = terrain.get_tile(location);
        let mut score = tile.fertility * 100.0;
        
        // Prefer locations near water
        let water_nearby = terrain.get_neighbors(location).iter().any(|&neighbor| {
            let neighbor_tile = terrain.get_tile(neighbor);
            matches!(neighbor_tile.terrain_type, super::TerrainType::River | super::TerrainType::Lake)
        });
        
        if water_nearby {
            score += 50.0;
        }
        
        // Prefer defensible positions (hills)
        if matches!(tile.terrain_type, super::TerrainType::Hill) {
            score += 30.0;
        }
        
        // Avoid extreme locations
        if location.x < 5 || location.x > ZONE_SIZE - 5 || location.y < 5 || location.y > ZONE_SIZE - 5 {
            score -= 20.0;
        }
        
        score
    }
    
    fn generate_settlement(&self, location: LocalCoord, is_primary: bool, terrain: &TerrainMap, rng: &mut ChaCha8Rng) -> Settlement {
        let tile = terrain.get_tile(location);
        
        // Determine settlement type and size
        let (settlement_type, population) = if is_primary {
            // Primary settlement is larger
            match rng.gen_range(0..100) {
                0..20 => (SettlementType::Town, rng.gen_range(300..1000)),
                20..40 => (SettlementType::City, rng.gen_range(1000..3000)),
                _ => (SettlementType::Village, rng.gen_range(100..400)),
            }
        } else {
            // Secondary settlements are smaller
            match rng.gen_range(0..100) {
                0..60 => (SettlementType::Village, rng.gen_range(50..200)),
                60..85 => (SettlementType::Town, rng.gen_range(200..600)),
                _ => (SettlementType::Outpost, rng.gen_range(10..80)),
            }
        };
        
        let size = self.calculate_settlement_size(&settlement_type, population);
        let name = self.generate_settlement_name(rng);
        let prosperity = tile.fertility * 0.8 + rng.gen_range(0.0..0.4);
        
        // Determine specializations based on terrain and resources
        let specializations = self.determine_specializations(location, terrain, &settlement_type, rng);
        
        // Generate buildings
        let buildings = self.generate_buildings(&settlement_type, &specializations, prosperity, rng);
        
        Settlement {
            name,
            position: location,
            settlement_type,
            size,
            population,
            prosperity,
            specializations,
            buildings,
            established_year: rng.gen_range(800..1200), // Arbitrary fantasy years
        }
    }
    
    fn calculate_settlement_size(&self, settlement_type: &SettlementType, population: u32) -> u32 {
        match settlement_type {
            SettlementType::Outpost => (population / 10).max(3),
            SettlementType::Village => (population / 8).max(5),
            SettlementType::Town => (population / 6).max(8),
            SettlementType::City => (population / 4).max(12),
            SettlementType::Capital => (population / 3).max(20),
        }
    }
    
    fn generate_settlement_name(&self, rng: &mut ChaCha8Rng) -> String {
        let prefix = self.name_prefixes[rng.gen_range(0..self.name_prefixes.len())];
        let suffix = self.name_suffixes[rng.gen_range(0..self.name_suffixes.len())];
        format!("{}{}", prefix, suffix)
    }
    
    fn determine_specializations(&self, location: LocalCoord, terrain: &TerrainMap, settlement_type: &SettlementType, rng: &mut ChaCha8Rng) -> Vec<SettlementSpecialization> {
        let mut specializations = Vec::new();
        let tile = terrain.get_tile(location);
        
        // Always have farming if fertility is decent
        if tile.fertility > 0.4 {
            specializations.push(SettlementSpecialization::Farming);
        }
        
        // Check nearby terrain for resources
        let nearby_water = terrain.get_neighbors(location).iter().any(|&neighbor| {
            let neighbor_tile = terrain.get_tile(neighbor);
            matches!(neighbor_tile.terrain_type, super::TerrainType::River | super::TerrainType::Lake)
        });
        
        if nearby_water {
            specializations.push(SettlementSpecialization::Fishing);
        }
        
        // Terrain-based specializations
        match tile.terrain_type {
            super::TerrainType::Mountain | super::TerrainType::Hill => {
                if rng.gen_bool(0.6) {
                    specializations.push(SettlementSpecialization::Mining);
                }
            }
            super::TerrainType::Forest => {
                if rng.gen_bool(0.7) {
                    specializations.push(SettlementSpecialization::Logging);
                }
            }
            _ => {}
        }
        
        // Size-based specializations
        match settlement_type {
            SettlementType::Town | SettlementType::City | SettlementType::Capital => {
                if rng.gen_bool(0.8) {
                    specializations.push(SettlementSpecialization::Trading);
                }
                if rng.gen_bool(0.6) {
                    specializations.push(SettlementSpecialization::Crafting);
                }
                if rng.gen_bool(0.4) {
                    specializations.push(SettlementSpecialization::Military);
                }
            }
            _ => {}
        }
        
        // Religious centers
        if rng.gen_bool(0.3) {
            specializations.push(SettlementSpecialization::Religious);
        }
        
        // Magical centers (rare)
        if matches!(settlement_type, SettlementType::City | SettlementType::Capital) && rng.gen_bool(0.15) {
            specializations.push(SettlementSpecialization::Magical);
        }
        
        specializations
    }
    
    fn generate_buildings(&self, settlement_type: &SettlementType, specializations: &[SettlementSpecialization], prosperity: f32, rng: &mut ChaCha8Rng) -> Vec<Building> {
        let mut buildings = Vec::new();
        
        // Basic buildings every settlement has
        buildings.push(Building {
            name: "Houses".to_string(),
            building_type: BuildingType::Houses,
            size: match settlement_type {
                SettlementType::Outpost => rng.gen_range(3..8),
                SettlementType::Village => rng.gen_range(8..20),
                SettlementType::Town => rng.gen_range(20..50),
                SettlementType::City => rng.gen_range(50..120),
                SettlementType::Capital => rng.gen_range(120..300),
            },
            condition: rng.gen_range(0.6..1.0),
        });
        
        // Specialization-based buildings
        for specialization in specializations {
            match specialization {
                SettlementSpecialization::Farming => {
                    buildings.push(Building {
                        name: "Farms".to_string(),
                        building_type: BuildingType::Farm,
                        size: rng.gen_range(2..8),
                        condition: rng.gen_range(0.7..1.0),
                    });
                    if rng.gen_bool(0.7) {
                        buildings.push(Building {
                            name: "Mill".to_string(),
                            building_type: BuildingType::Mill,
                            size: 1,
                            condition: rng.gen_range(0.6..0.9),
                        });
                    }
                }
                SettlementSpecialization::Mining => {
                    buildings.push(Building {
                        name: "Mine".to_string(),
                        building_type: BuildingType::Mine,
                        size: rng.gen_range(1..3),
                        condition: rng.gen_range(0.5..0.8),
                    });
                }
                SettlementSpecialization::Trading => {
                    buildings.push(Building {
                        name: "Market".to_string(),
                        building_type: BuildingType::Market,
                        size: 1,
                        condition: rng.gen_range(0.7..1.0),
                    });
                    buildings.push(Building {
                        name: "Warehouse".to_string(),
                        building_type: BuildingType::Warehouse,
                        size: rng.gen_range(1..4),
                        condition: rng.gen_range(0.6..0.9),
                    });
                }
                SettlementSpecialization::Crafting => {
                    buildings.push(Building {
                        name: "Blacksmith".to_string(),
                        building_type: BuildingType::Blacksmith,
                        size: 1,
                        condition: rng.gen_range(0.7..1.0),
                    });
                    buildings.push(Building {
                        name: "Shops".to_string(),
                        building_type: BuildingType::Shop,
                        size: rng.gen_range(2..6),
                        condition: rng.gen_range(0.6..0.9),
                    });
                }
                SettlementSpecialization::Military => {
                    buildings.push(Building {
                        name: "Barracks".to_string(),
                        building_type: BuildingType::Barracks,
                        size: 1,
                        condition: rng.gen_range(0.8..1.0),
                    });
                    if matches!(settlement_type, SettlementType::Town | SettlementType::City | SettlementType::Capital) {
                        buildings.push(Building {
                            name: "Walls".to_string(),
                            building_type: BuildingType::Walls,
                            size: 1,
                            condition: rng.gen_range(0.6..0.9),
                        });
                    }
                }
                SettlementSpecialization::Religious => {
                    buildings.push(Building {
                        name: "Temple".to_string(),
                        building_type: BuildingType::Temple,
                        size: 1,
                        condition: rng.gen_range(0.8..1.0),
                    });
                }
                SettlementSpecialization::Fishing => {
                    buildings.push(Building {
                        name: "Dock".to_string(),
                        building_type: BuildingType::Dock,
                        size: 1,
                        condition: rng.gen_range(0.7..0.9),
                    });
                }
                _ => {}
            }
        }
        
        // Size and prosperity-based buildings
        if matches!(settlement_type, SettlementType::Village | SettlementType::Town | SettlementType::City | SettlementType::Capital) {
            buildings.push(Building {
                name: "Inn".to_string(),
                building_type: BuildingType::Inn,
                size: 1,
                condition: rng.gen_range(0.6..0.9),
            });
            
            if prosperity > 0.5 {
                buildings.push(Building {
                    name: "Tavern".to_string(),
                    building_type: BuildingType::Tavern,
                    size: 1,
                    condition: rng.gen_range(0.7..1.0),
                });
            }
        }
        
        buildings
    }
    
    fn establish_settlement_relationships(&self, settlements: &mut [Settlement], _terrain: &TerrainMap) {
        // This would establish trade relationships, political connections, etc.
        // For now, we'll keep it simple and just ensure they're aware of each other
        let settlement_count = settlements.len();
        for settlement in settlements.iter_mut() {
            settlement.prosperity += 0.1 * (settlement_count - 1) as f32 * 0.05;
            settlement.prosperity = settlement.prosperity.clamp(0.0, 1.0);
        }
    }
}

impl Settlement {
    pub fn get_display_info(&self) -> Vec<String> {
        vec![
            format!("{} ({})", self.name, self.settlement_type.get_name()),
            format!("Population: {}", self.population),
            format!("Prosperity: {:.1}%", self.prosperity * 100.0),
            format!("Established: {}", self.established_year),
            format!("Specializations: {}", 
                self.specializations.iter()
                    .map(|s| s.get_name())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ]
    }
}

impl SettlementType {
    pub fn get_name(&self) -> &'static str {
        match self {
            SettlementType::Outpost => "Outpost",
            SettlementType::Village => "Village",
            SettlementType::Town => "Town",
            SettlementType::City => "City",
            SettlementType::Capital => "Capital",
        }
    }
    
    pub fn get_ascii_char(&self) -> char {
        match self {
            SettlementType::Outpost => '•',
            SettlementType::Village => '○',
            SettlementType::Town => '●',
            SettlementType::City => '◉',
            SettlementType::Capital => '⬟',
        }
    }
}

impl SettlementSpecialization {
    pub fn get_name(&self) -> &'static str {
        match self {
            SettlementSpecialization::Farming => "Farming",
            SettlementSpecialization::Mining => "Mining",
            SettlementSpecialization::Logging => "Logging",
            SettlementSpecialization::Fishing => "Fishing",
            SettlementSpecialization::Trading => "Trading",
            SettlementSpecialization::Crafting => "Crafting",
            SettlementSpecialization::Military => "Military",
            SettlementSpecialization::Religious => "Religious",
            SettlementSpecialization::Magical => "Magical",
        }
    }
}
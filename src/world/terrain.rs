use serde::{Deserialize, Serialize};
use noise::{NoiseFn, Perlin};
use rand_chacha::ChaCha8Rng;
use rand::Rng;
use super::{ZoneCoord, LocalCoord, ZONE_SIZE};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainMap {
    pub tiles: Vec<Vec<TerrainTile>>,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainTile {
    pub terrain_type: TerrainType,
    pub elevation: f32,
    pub moisture: f32,
    pub temperature: f32,
    pub fertility: f32,
    pub traversal_cost: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainType {
    Ocean,
    Lake,
    River,
    Swamp,
    Desert,
    Plains,
    Grassland,
    Forest,
    Hill,
    Mountain,
    Snow,
    Tundra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiomeType {
    Arctic,
    Subarctic,
    Temperate,
    Subtropical,
    Tropical,
    Desert,
}

pub struct TerrainGenerator<'a> {
    terrain_noise: &'a Perlin,
    moisture_noise: &'a Perlin,
    temperature_noise: &'a Perlin,
}

impl<'a> TerrainGenerator<'a> {
    pub fn new(terrain_noise: &'a Perlin, moisture_noise: &'a Perlin, temperature_noise: &'a Perlin) -> Self {
        Self {
            terrain_noise,
            moisture_noise,
            temperature_noise,
        }
    }
    
    pub fn generate(&self, zone_coord: ZoneCoord, _rng: &mut ChaCha8Rng) -> TerrainMap {
        let mut tiles = vec![vec![TerrainTile::default(); ZONE_SIZE as usize]; ZONE_SIZE as usize];
        
        // Generate base terrain using noise
        for x in 0..ZONE_SIZE {
            for y in 0..ZONE_SIZE {
                let world_x = zone_coord.x * ZONE_SIZE + x;
                let world_y = zone_coord.y * ZONE_SIZE + y;
                
                // Sample noise at multiple scales for detail
                let elevation = self.sample_elevation(world_x as f64, world_y as f64);
                let moisture = self.sample_moisture(world_x as f64, world_y as f64);
                let temperature = self.sample_temperature(world_x as f64, world_y as f64);
                
                let terrain_type = self.determine_terrain_type(elevation, moisture, temperature);
                let fertility = self.calculate_fertility(&terrain_type, moisture, temperature);
                let traversal_cost = self.calculate_traversal_cost(&terrain_type);
                
                tiles[x as usize][y as usize] = TerrainTile {
                    terrain_type,
                    elevation,
                    moisture,
                    temperature,
                    fertility,
                    traversal_cost,
                };
            }
        }
        
        // Post-process to add features like rivers, lakes, etc.
        // Skip expensive post-processing for faster generation
        // self.add_water_features(&mut tiles, rng);
        // self.smooth_terrain(&mut tiles);
        
        TerrainMap {
            tiles,
            width: ZONE_SIZE,
            height: ZONE_SIZE,
        }
    }
    
    fn sample_elevation(&self, x: f64, y: f64) -> f32 {
        let scale1 = 0.01;  // Large features
        let scale2 = 0.05;  // Medium features
        let scale3 = 0.1;   // Small features
        
        let noise1 = self.terrain_noise.get([x * scale1, y * scale1]) * 0.6;
        let noise2 = self.terrain_noise.get([x * scale2, y * scale2]) * 0.3;
        let noise3 = self.terrain_noise.get([x * scale3, y * scale3]) * 0.1;
        
        ((noise1 + noise2 + noise3 + 1.0) / 2.0) as f32
    }
    
    fn sample_moisture(&self, x: f64, y: f64) -> f32 {
        let scale = 0.02;
        ((self.moisture_noise.get([x * scale, y * scale]) + 1.0) / 2.0) as f32
    }
    
    fn sample_temperature(&self, x: f64, y: f64) -> f32 {
        let scale = 0.015;
        let base_temp = ((self.temperature_noise.get([x * scale, y * scale]) + 1.0) / 2.0) as f32;
        
        // Latitude effect (assuming y=0 is equator)
        let latitude_factor = 1.0 - (y.abs() / 10000.0) as f32;
        
        (base_temp * 0.7 + latitude_factor * 0.3).clamp(0.0, 1.0)
    }
    
    fn determine_terrain_type(&self, elevation: f32, moisture: f32, temperature: f32) -> TerrainType {
        // Water bodies
        if elevation < 0.2 {
            return TerrainType::Ocean;
        }
        
        if elevation < 0.25 && moisture > 0.8 {
            return TerrainType::Lake;
        }
        
        // High elevation
        if elevation > 0.8 {
            if temperature < 0.3 {
                return TerrainType::Snow;
            } else {
                return TerrainType::Mountain;
            }
        }
        
        if elevation > 0.6 {
            return TerrainType::Hill;
        }
        
        // Temperature-based biomes
        if temperature < 0.2 {
            return TerrainType::Tundra;
        }
        
        // Moisture-based terrain
        if moisture < 0.2 {
            return TerrainType::Desert;
        }
        
        if moisture > 0.8 {
            if elevation < 0.3 {
                return TerrainType::Swamp;
            } else {
                return TerrainType::Forest;
            }
        }
        
        if moisture > 0.5 {
            return TerrainType::Forest;
        }
        
        if moisture > 0.3 {
            return TerrainType::Grassland;
        }
        
        TerrainType::Plains
    }
    
    fn calculate_fertility(&self, terrain_type: &TerrainType, moisture: f32, temperature: f32) -> f32 {
        let base_fertility = match terrain_type {
            TerrainType::Grassland => 0.8,
            TerrainType::Plains => 0.7,
            TerrainType::Forest => 0.6,
            TerrainType::Swamp => 0.4,
            TerrainType::Hill => 0.5,
            TerrainType::River => 0.9,
            TerrainType::Lake => 0.3,
            TerrainType::Desert => 0.1,
            TerrainType::Mountain => 0.2,
            TerrainType::Snow => 0.0,
            TerrainType::Tundra => 0.1,
            TerrainType::Ocean => 0.0,
        };
        
        // Modify based on moisture and temperature
        let moisture_factor = (moisture * 2.0 - 1.0).abs(); // Prefer moderate moisture
        let temp_factor = (temperature * 2.0 - 1.0).abs(); // Prefer moderate temperature
        
        (base_fertility * (1.0 - moisture_factor * 0.3) * (1.0 - temp_factor * 0.3)).clamp(0.0, 1.0)
    }
    
    fn calculate_traversal_cost(&self, terrain_type: &TerrainType) -> f32 {
        match terrain_type {
            TerrainType::Plains => 1.0,
            TerrainType::Grassland => 1.1,
            TerrainType::Forest => 1.5,
            TerrainType::Hill => 2.0,
            TerrainType::Mountain => 4.0,
            TerrainType::Swamp => 3.0,
            TerrainType::Desert => 1.8,
            TerrainType::Snow => 2.5,
            TerrainType::Tundra => 1.8,
            TerrainType::River => 0.8, // Rivers can be followed
            TerrainType::Lake => 10.0, // Very hard to cross
            TerrainType::Ocean => 100.0, // Nearly impossible without boats
        }
    }
    
    #[allow(dead_code)]
    fn add_water_features(&self, tiles: &mut Vec<Vec<TerrainTile>>, rng: &mut ChaCha8Rng) {
        // Add small lakes
        let lake_count = rng.gen_range(0..3);
        for _ in 0..lake_count {
            let center_x = rng.gen_range(5..(ZONE_SIZE - 5)) as usize;
            let center_y = rng.gen_range(5..(ZONE_SIZE - 5)) as usize;
            let radius = rng.gen_range(5..15);
            
            for dx in -(radius as i32)..=(radius as i32) {
                for dy in -(radius as i32)..=(radius as i32) {
                    let x = center_x as i32 + dx;
                    let y = center_y as i32 + dy;
                    
                    if x >= 0 && x < ZONE_SIZE && y >= 0 && y < ZONE_SIZE {
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        if distance <= radius as f32 {
                            let tile = &mut tiles[x as usize][y as usize];
                            if tile.elevation > 0.2 {
                                tile.terrain_type = TerrainType::Lake;
                                tile.elevation = 0.25;
                                tile.moisture = 1.0;
                            }
                        }
                    }
                }
            }
        }
    }
    
    #[allow(dead_code)]
    fn smooth_terrain(&self, tiles: &mut Vec<Vec<TerrainTile>>) {
        // Simple smoothing pass to reduce isolated tiles
        for x in 1..(ZONE_SIZE - 1) as usize {
            for y in 1..(ZONE_SIZE - 1) as usize {
                let neighbors = [
                    &tiles[x-1][y], &tiles[x+1][y],
                    &tiles[x][y-1], &tiles[x][y+1],
                ];
                
                // Count neighbor terrain types
                let mut terrain_counts = std::collections::HashMap::new();
                for neighbor in &neighbors {
                    *terrain_counts.entry(&neighbor.terrain_type).or_insert(0) += 1;
                }
                
                // If current tile is isolated, change it to most common neighbor
                if let Some((most_common, count)) = terrain_counts.iter().max_by_key(|(_, &count)| count) {
                    if *count >= 3 && tiles[x][y].terrain_type != **most_common {
                        // Only change if it makes sense (similar elevation)
                        let avg_elevation: f32 = neighbors.iter().map(|n| n.elevation).sum::<f32>() / neighbors.len() as f32;
                        if (tiles[x][y].elevation - avg_elevation).abs() < 0.2 {
                            tiles[x][y].terrain_type = (*most_common).clone();
                        }
                    }
                }
            }
        }
    }
}

impl TerrainMap {
    pub fn get_tile(&self, coord: LocalCoord) -> &TerrainTile {
        &self.tiles[coord.x as usize][coord.y as usize]
    }
    
    pub fn get_tile_mut(&mut self, coord: LocalCoord) -> &mut TerrainTile {
        &mut self.tiles[coord.x as usize][coord.y as usize]
    }
    
    pub fn is_valid_coord(&self, coord: LocalCoord) -> bool {
        coord.x >= 0 && coord.x < self.width && coord.y >= 0 && coord.y < self.height
    }
    
    pub fn get_neighbors(&self, coord: LocalCoord) -> Vec<LocalCoord> {
        let mut neighbors = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let new_coord = LocalCoord::new(coord.x + dx, coord.y + dy);
                if self.is_valid_coord(new_coord) {
                    neighbors.push(new_coord);
                }
            }
        }
        neighbors
    }
    
    pub fn find_suitable_settlement_locations(&self, min_fertility: f32, min_distance: i32) -> Vec<LocalCoord> {
        let mut locations = Vec::new();
        
        for x in 0..self.width {
            for y in 0..self.height {
                let coord = LocalCoord::new(x, y);
                let tile = self.get_tile(coord);
                
                // Check if location is suitable
                if tile.fertility >= min_fertility && 
                   !matches!(tile.terrain_type, TerrainType::Ocean | TerrainType::Lake | TerrainType::Mountain | TerrainType::Snow) {
                    
                    // Check distance from existing locations
                    let far_enough = locations.iter().all(|existing: &LocalCoord| {
                        let dx = coord.x - existing.x;
                        let dy = coord.y - existing.y;
                        (dx * dx + dy * dy) >= min_distance * min_distance
                    });
                    
                    if far_enough {
                        locations.push(coord);
                    }
                }
            }
        }
        
        locations
    }
}

impl Default for TerrainTile {
    fn default() -> Self {
        Self {
            terrain_type: TerrainType::Plains,
            elevation: 0.5,
            moisture: 0.5,
            temperature: 0.5,
            fertility: 0.5,
            traversal_cost: 1.0,
        }
    }
}

impl TerrainType {
    pub fn get_ascii_char(&self) -> char {
        match self {
            TerrainType::Ocean => '~',
            TerrainType::Lake => '○',
            TerrainType::River => '≈',
            TerrainType::Swamp => '♠',
            TerrainType::Desert => '░',
            TerrainType::Plains => '.',
            TerrainType::Grassland => ',',
            TerrainType::Forest => '♣',
            TerrainType::Hill => '∩',
            TerrainType::Mountain => '▲',
            TerrainType::Snow => '*',
            TerrainType::Tundra => '·',
        }
    }
    
    pub fn get_color(&self) -> &'static str {
        match self {
            TerrainType::Ocean => "blue",
            TerrainType::Lake => "cyan",
            TerrainType::River => "cyan",
            TerrainType::Swamp => "dark_green",
            TerrainType::Desert => "yellow",
            TerrainType::Plains => "green",
            TerrainType::Grassland => "bright_green",
            TerrainType::Forest => "dark_green",
            TerrainType::Hill => "brown",
            TerrainType::Mountain => "gray",
            TerrainType::Snow => "white",
            TerrainType::Tundra => "dark_gray",
        }
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use super::{WorldZone, ZoneCoord, WorldGenerator};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldDatabase {
    pub master_seed: u64,
    pub zones: HashMap<ZoneCoord, WorldZone>,
    pub metadata: WorldMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub total_zones_generated: usize,
    pub world_name: String,
    pub version: String,
}

pub struct WorldManager {
    database: WorldDatabase,
    save_path: PathBuf,
    generator: WorldGenerator,
    dirty_zones: std::collections::HashSet<ZoneCoord>,
}

impl WorldManager {
    pub fn new(world_name: &str, master_seed: u64, save_directory: &Path) -> Result<Self> {
        let save_path = save_directory.join(format!("{}_world.json", world_name));
        
        let database = if save_path.exists() {
            Self::load_database(&save_path)?
        } else {
            WorldDatabase {
                master_seed,
                zones: HashMap::new(),
                metadata: WorldMetadata {
                    created_at: chrono::Utc::now(),
                    last_accessed: chrono::Utc::now(),
                    total_zones_generated: 0,
                    world_name: world_name.to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            }
        };
        
        let generator = WorldGenerator::new(database.master_seed);
        
        Ok(WorldManager {
            database,
            save_path,
            generator,
            dirty_zones: std::collections::HashSet::new(),
        })
    }
    
    pub fn get_zone(&mut self, coord: ZoneCoord) -> Result<&WorldZone> {
        if !self.database.zones.contains_key(&coord) {
            self.generate_zone(coord)?;
        }
        
        // Mark zone as visited
        if let Some(zone) = self.database.zones.get_mut(&coord) {
            zone.mark_visited();
            self.dirty_zones.insert(coord);
        }
        
        Ok(self.database.zones.get(&coord).unwrap())
    }
    
    pub fn get_zone_if_exists(&self, coord: ZoneCoord) -> Option<&WorldZone> {
        self.database.zones.get(&coord)
    }
    
    pub fn generate_zone(&mut self, coord: ZoneCoord) -> Result<()> {
        if self.database.zones.contains_key(&coord) {
            return Ok(()); // Already exists
        }
        
        // Get adjacent zones for context
        let adjacent_zones: HashMap<ZoneCoord, WorldZone> = coord.adjacent_zones()
            .into_iter()
            .filter_map(|adj_coord| {
                self.database.zones.get(&adj_coord).map(|zone| (adj_coord, zone.clone()))
            })
            .collect();
        
        // Generate the new zone
        let zone = self.generator.generate_zone(coord, &adjacent_zones);
        
        // Store the zone
        self.database.zones.insert(coord, zone);
        self.database.metadata.total_zones_generated += 1;
        self.database.metadata.last_accessed = chrono::Utc::now();
        self.dirty_zones.insert(coord);
        
        Ok(())
    }
    
    pub fn save(&mut self) -> Result<()> {
        self.save_database()?;
        self.dirty_zones.clear();
        Ok(())
    }
    
    pub fn save_if_dirty(&mut self) -> Result<()> {
        if !self.dirty_zones.is_empty() {
            self.save()?;
        }
        Ok(())
    }
    
    pub fn get_generated_zone_coords(&self) -> Vec<ZoneCoord> {
        self.database.zones.keys().copied().collect()
    }
    
    pub fn get_world_info(&self) -> &WorldMetadata {
        &self.database.metadata
    }
    
    pub fn pregenerate_area(&mut self, center: ZoneCoord, radius: i32) -> Result<Vec<ZoneCoord>> {
        let mut generated = Vec::new();
        
        for x in (center.x - radius)..=(center.x + radius) {
            for y in (center.y - radius)..=(center.y + radius) {
                let coord = ZoneCoord::new(x, y);
                if !self.database.zones.contains_key(&coord) {
                    self.generate_zone(coord)?;
                    generated.push(coord);
                }
            }
        }
        
        if !generated.is_empty() {
            self.save()?;
        }
        
        Ok(generated)
    }
    
    pub fn cleanup_distant_zones(&mut self, player_position: ZoneCoord, max_distance: i32) -> Result<usize> {
        let zones_to_remove: Vec<ZoneCoord> = self.database.zones.keys()
            .filter(|&&coord| {
                let dx = (coord.x - player_position.x).abs();
                let dy = (coord.y - player_position.y).abs();
                dx > max_distance || dy > max_distance
            })
            .copied()
            .collect();
        
        let removed_count = zones_to_remove.len();
        
        for coord in zones_to_remove {
            self.database.zones.remove(&coord);
            self.dirty_zones.remove(&coord);
        }
        
        if removed_count > 0 {
            self.save()?;
        }
        
        Ok(removed_count)
    }
    
    pub fn get_zone_summary(&self, coord: ZoneCoord) -> Option<ZoneSummary> {
        self.database.zones.get(&coord).map(|zone| ZoneSummary {
            coord,
            settlement_count: zone.settlements.len(),
            largest_settlement: zone.settlements.iter()
                .max_by_key(|s| s.population)
                .map(|s| s.name.clone()),
            road_count: zone.roads.roads.len(),
            river_count: zone.rivers.len(),
            poi_count: zone.points_of_interest.len(),
            generated_at: zone.generated_at,
            last_visited: zone.last_visited,
        })
    }
    
    fn load_database(path: &Path) -> Result<WorldDatabase> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read world database from {}", path.display()))?;
        
        let mut database: WorldDatabase = serde_json::from_str(&content)
            .with_context(|| "Failed to parse world database JSON")?;
        
        // Update last accessed time
        database.metadata.last_accessed = chrono::Utc::now();
        
        Ok(database)
    }
    
    fn save_database(&self) -> Result<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = self.save_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        
        // Serialize to JSON with pretty printing
        let content = serde_json::to_string_pretty(&self.database)
            .with_context(|| "Failed to serialize world database")?;
        
        // Write to temporary file first, then rename (atomic operation)
        let temp_path = self.save_path.with_extension("tmp");
        fs::write(&temp_path, content)
            .with_context(|| format!("Failed to write world database to {}", temp_path.display()))?;
        
        fs::rename(&temp_path, &self.save_path)
            .with_context(|| format!("Failed to rename {} to {}", temp_path.display(), self.save_path.display()))?;
        
        Ok(())
    }
    
    pub fn export_zone_map(&self, center: ZoneCoord, radius: i32) -> Result<String> {
        let mut map_lines = Vec::new();
        
        for y in (center.y - radius)..=(center.y + radius) {
            let mut line = String::new();
            for x in (center.x - radius)..=(center.x + radius) {
                let coord = ZoneCoord::new(x, y);
                let char = if coord == center {
                    '@' // Player position
                } else if let Some(zone) = self.database.zones.get(&coord) {
                    if zone.settlements.is_empty() {
                        '·' // Generated but empty
                    } else {
                        let largest = zone.settlements.iter()
                            .max_by_key(|s| s.population)
                            .unwrap();
                        largest.settlement_type.get_ascii_char()
                    }
                } else {
                    ' ' // Not generated
                };
                line.push(char);
            }
            map_lines.push(line);
        }
        
        Ok(map_lines.join("\n"))
    }
    
    pub fn get_statistics(&self) -> WorldStatistics {
        let mut stats = WorldStatistics::default();
        
        for zone in self.database.zones.values() {
            stats.total_settlements += zone.settlements.len();
            stats.total_roads += zone.roads.roads.len();
            stats.total_rivers += zone.rivers.len();
            stats.total_pois += zone.points_of_interest.len();
            
            for settlement in &zone.settlements {
                stats.total_population += settlement.population as u64;
                
                match settlement.settlement_type {
                    super::SettlementType::Outpost => stats.outposts += 1,
                    super::SettlementType::Village => stats.villages += 1,
                    super::SettlementType::Town => stats.towns += 1,
                    super::SettlementType::City => stats.cities += 1,
                    super::SettlementType::Capital => stats.capitals += 1,
                }
            }
        }
        
        stats.zones_generated = self.database.zones.len();
        stats
    }
}

#[derive(Debug, Clone)]
pub struct ZoneSummary {
    pub coord: ZoneCoord,
    pub settlement_count: usize,
    pub largest_settlement: Option<String>,
    pub road_count: usize,
    pub river_count: usize,
    pub poi_count: usize,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub last_visited: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Default)]
pub struct WorldStatistics {
    pub zones_generated: usize,
    pub total_settlements: usize,
    pub total_population: u64,
    pub total_roads: usize,
    pub total_rivers: usize,
    pub total_pois: usize,
    pub outposts: usize,
    pub villages: usize,
    pub towns: usize,
    pub cities: usize,
    pub capitals: usize,
}

// Utility functions for world management
impl WorldManager {
    pub fn backup_world(&self, backup_path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.database)
            .with_context(|| "Failed to serialize world database for backup")?;
        
        fs::write(backup_path, content)
            .with_context(|| format!("Failed to write backup to {}", backup_path.display()))?;
        
        Ok(())
    }
    
    pub fn get_zones_in_area(&self, center: ZoneCoord, radius: i32) -> Vec<(ZoneCoord, Option<&WorldZone>)> {
        let mut zones = Vec::new();
        
        for x in (center.x - radius)..=(center.x + radius) {
            for y in (center.y - radius)..=(center.y + radius) {
                let coord = ZoneCoord::new(x, y);
                let zone = self.database.zones.get(&coord);
                zones.push((coord, zone));
            }
        }
        
        zones
    }
    
    pub fn find_nearest_settlement(&self, center: ZoneCoord, max_search_radius: i32) -> Option<(ZoneCoord, &super::Settlement)> {
        for radius in 0..=max_search_radius {
            for x in (center.x - radius)..=(center.x + radius) {
                for y in (center.y - radius)..=(center.y + radius) {
                    let coord = ZoneCoord::new(x, y);
                    if let Some(zone) = self.database.zones.get(&coord) {
                        if let Some(settlement) = zone.settlements.iter().max_by_key(|s| s.population) {
                            return Some((coord, settlement));
                        }
                    }
                }
            }
        }
        None
    }
}

impl Drop for WorldManager {
    fn drop(&mut self) {
        // Auto-save when the manager is dropped
        let _ = self.save_if_dirty();
    }
}
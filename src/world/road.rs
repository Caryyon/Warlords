#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use super::{ZoneCoord, LocalCoord, WorldCoord, WorldZone, Settlement, TerrainMap, ZONE_SIZE};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadNetwork {
    pub roads: Vec<Road>,
    pub connections: HashMap<LocalCoord, Vec<LocalCoord>>,
    pub zone_exits: Vec<ZoneExit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Road {
    pub path: Vec<LocalCoord>,
    pub road_type: RoadType,
    pub condition: f32,
    pub width: u32,
    pub traffic_level: TrafficLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoadType {
    Trail,        // Foot traffic only
    Path,         // Light cart traffic
    Road,         // Regular cart and wagon traffic
    Highway,      // Major trade route
    Imperial,     // Major imperial road
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrafficLevel {
    Abandoned,
    Light,
    Moderate,
    Heavy,
    Major,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneExit {
    pub position: LocalCoord,
    pub direction: ExitDirection,
    pub connects_to: Option<ZoneCoord>,
    pub road_type: RoadType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExitDirection {
    North,
    South,
    East,
    West,
    Northeast,
    Northwest,
    Southeast,
    Southwest,
}

#[derive(Debug, Clone)]
struct PathNode {
    position: LocalCoord,
    cost: f32,
    heuristic: f32,
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        (other.cost + other.heuristic).partial_cmp(&(self.cost + self.heuristic)).unwrap_or(Ordering::Equal)
    }
}

pub struct RoadGenerator;

impl RoadGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate(&self, _zone_coord: ZoneCoord, settlements: &[Settlement], _adjacent_zones: &HashMap<ZoneCoord, WorldZone>, _terrain: &TerrainMap, _rng: &mut ChaCha8Rng) -> RoadNetwork {
        let mut roads = Vec::new();
        let connections = HashMap::new();
        let zone_exits = Vec::new();
        
        // Simplified road generation - just create simple straight line roads between settlements
        if settlements.len() >= 2 {
            for i in 0..(settlements.len() - 1) {
                let start = settlements[i].position;
                let end = settlements[i + 1].position;
                
                // Create a simple straight-line path
                let mut path = vec![start];
                let dx = (end.x - start.x).signum();
                let dy = (end.y - start.y).signum();
                
                let mut current = start;
                while current != end {
                    if current.x != end.x {
                        current.x += dx;
                    } else if current.y != end.y {
                        current.y += dy;
                    }
                    path.push(current);
                }
                
                roads.push(Road {
                    path,
                    road_type: RoadType::Path,
                    condition: 0.8,
                    width: 2,
                    traffic_level: TrafficLevel::Light,
                });
            }
        }
        
        RoadNetwork { roads, connections, zone_exits }
    }
    
    fn generate_internal_roads(&self, settlements: &[Settlement], terrain: &TerrainMap, rng: &mut ChaCha8Rng) -> Vec<Road> {
        let mut roads = Vec::new();
        
        if settlements.len() < 2 {
            return roads;
        }
        
        // Connect all settlements with a minimum spanning tree approach
        let mut connected = HashSet::new();
        let mut unconnected: HashSet<usize> = (0..settlements.len()).collect();
        
        // Start with the largest settlement
        let start_idx = settlements.iter()
            .enumerate()
            .max_by_key(|(_, s)| s.population)
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        connected.insert(start_idx);
        unconnected.remove(&start_idx);
        
        while !unconnected.is_empty() {
            let mut best_connection = None;
            let mut best_cost = f32::INFINITY;
            
            // Find the cheapest connection from connected to unconnected settlements
            for &connected_idx in &connected {
                for &unconnected_idx in &unconnected {
                    let start = settlements[connected_idx].position;
                    let end = settlements[unconnected_idx].position;
                    let cost = self.estimate_path_cost(start, end, terrain);
                    
                    if cost < best_cost {
                        best_cost = cost;
                        best_connection = Some((connected_idx, unconnected_idx));
                    }
                }
            }
            
            if let Some((from_idx, to_idx)) = best_connection {
                let start = settlements[from_idx].position;
                let end = settlements[to_idx].position;
                
                if let Some(path) = self.find_path(start, end, terrain) {
                    let road_type = self.determine_road_type(&settlements[from_idx], &settlements[to_idx], rng);
                    let condition = rng.gen_range(0.6..1.0);
                    let width = self.calculate_road_width(&road_type);
                    let traffic_level = self.determine_traffic_level(&road_type, &settlements[from_idx], &settlements[to_idx]);
                    
                    roads.push(Road {
                        path,
                        road_type,
                        condition,
                        width,
                        traffic_level,
                    });
                }
                
                connected.insert(to_idx);
                unconnected.remove(&to_idx);
            } else {
                break; // No more connections possible
            }
        }
        
        // Add some additional roads for major settlements
        self.add_additional_roads(&mut roads, settlements, terrain, rng);
        
        roads
    }
    
    fn generate_external_connections(&self, zone_coord: ZoneCoord, settlements: &[Settlement], adjacent_zones: &HashMap<ZoneCoord, WorldZone>, terrain: &TerrainMap, rng: &mut ChaCha8Rng) -> (Vec<Road>, Vec<ZoneExit>) {
        let mut roads = Vec::new();
        let mut exits = Vec::new();
        
        if settlements.is_empty() {
            return (roads, exits);
        }
        
        // For each adjacent zone, try to create connections
        for adjacent_coord in zone_coord.adjacent_zones() {
            if let Some(adjacent_zone) = adjacent_zones.get(&adjacent_coord) {
                if let Some((road, exit)) = self.create_zone_connection(zone_coord, adjacent_coord, settlements, adjacent_zone, terrain, rng) {
                    roads.push(road);
                    exits.push(exit);
                }
            } else {
                // Adjacent zone not generated yet, create potential exit points
                if let Some(exit) = self.create_potential_exit(zone_coord, adjacent_coord, settlements, terrain, rng) {
                    exits.push(exit);
                }
            }
        }
        
        (roads, exits)
    }
    
    fn create_zone_connection(&self, _current_zone: ZoneCoord, adjacent_coord: ZoneCoord, settlements: &[Settlement], adjacent_zone: &WorldZone, terrain: &TerrainMap, rng: &mut ChaCha8Rng) -> Option<(Road, ZoneExit)> {
        // Find the best settlement in current zone to connect from
        let source_settlement = settlements.iter()
            .max_by_key(|s| s.population)?;
        
        // Find the best settlement in adjacent zone to connect to
        let target_settlement = adjacent_zone.settlements.iter()
            .max_by_key(|s| s.population)?;
        
        // Convert target settlement position to world coordinates, then to border position
        let target_world = WorldCoord::from_zone_local(adjacent_coord, target_settlement.position);
        let border_position = self.find_border_position(_current_zone, adjacent_coord, source_settlement.position, target_world)?;
        
        // Create path from source settlement to border
        let path = self.find_path(source_settlement.position, border_position, terrain)?;
        
        let direction = self.get_exit_direction(_current_zone, adjacent_coord);
        let road_type = self.determine_inter_zone_road_type(source_settlement, target_settlement, rng);
        let condition = rng.gen_range(0.5..0.9);
        let width = self.calculate_road_width(&road_type);
        let traffic_level = self.determine_traffic_level(&road_type, source_settlement, target_settlement);
        
        let road = Road {
            path,
            road_type: road_type.clone(),
            condition,
            width,
            traffic_level,
        };
        
        let exit = ZoneExit {
            position: border_position,
            direction,
            connects_to: Some(adjacent_coord),
            road_type,
        };
        
        Some((road, exit))
    }
    
    fn create_potential_exit(&self, current_zone: ZoneCoord, adjacent_coord: ZoneCoord, settlements: &[Settlement], terrain: &TerrainMap, rng: &mut ChaCha8Rng) -> Option<ZoneExit> {
        if settlements.is_empty() {
            return None;
        }
        
        let source_settlement = settlements.iter()
            .max_by_key(|s| s.population)?;
        
        // Create a potential exit point on the border
        let border_position = self.find_suitable_border_position(current_zone, adjacent_coord, source_settlement.position, terrain)?;
        let direction = self.get_exit_direction(current_zone, adjacent_coord);
        let road_type = if rng.gen_bool(0.3) { RoadType::Road } else { RoadType::Path };
        
        Some(ZoneExit {
            position: border_position,
            direction,
            connects_to: None,
            road_type,
        })
    }
    
    fn find_path(&self, start: LocalCoord, end: LocalCoord, terrain: &TerrainMap) -> Option<Vec<LocalCoord>> {
        let mut open_set = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        
        let start_node = PathNode {
            position: start,
            cost: 0.0,
            heuristic: self.heuristic(start, end),
        };
        
        open_set.push(start_node);
        g_score.insert(start, 0.0);
        
        while let Some(current) = open_set.pop() {
            if current.position == end {
                return Some(self.reconstruct_path(&came_from, current.position));
            }
            
            for neighbor in terrain.get_neighbors(current.position) {
                let neighbor_tile = terrain.get_tile(neighbor);
                let movement_cost = neighbor_tile.traversal_cost;
                
                // Skip impassable terrain
                if movement_cost > 10.0 {
                    continue;
                }
                
                let tentative_g_score = g_score[&current.position] + movement_cost;
                
                if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&f32::INFINITY) {
                    came_from.insert(neighbor, current.position);
                    g_score.insert(neighbor, tentative_g_score);
                    
                    let neighbor_node = PathNode {
                        position: neighbor,
                        cost: tentative_g_score,
                        heuristic: self.heuristic(neighbor, end),
                    };
                    
                    open_set.push(neighbor_node);
                }
            }
        }
        
        None // No path found
    }
    
    fn heuristic(&self, from: LocalCoord, to: LocalCoord) -> f32 {
        let dx = (from.x - to.x).abs() as f32;
        let dy = (from.y - to.y).abs() as f32;
        dx + dy // Manhattan distance
    }
    
    fn reconstruct_path(&self, came_from: &HashMap<LocalCoord, LocalCoord>, mut current: LocalCoord) -> Vec<LocalCoord> {
        let mut path = vec![current];
        
        while let Some(&prev) = came_from.get(&current) {
            current = prev;
            path.push(current);
        }
        
        path.reverse();
        path
    }
    
    fn estimate_path_cost(&self, start: LocalCoord, end: LocalCoord, _terrain: &TerrainMap) -> f32 {
        // Simple estimate based on distance and average terrain cost
        let distance = self.heuristic(start, end);
        let avg_traversal_cost = 1.5; // Reasonable average
        distance * avg_traversal_cost
    }
    
    fn determine_road_type(&self, settlement_a: &Settlement, settlement_b: &Settlement, rng: &mut ChaCha8Rng) -> RoadType {
        let total_pop = settlement_a.population + settlement_b.population;
        let distance = {
            let dx = (settlement_a.position.x - settlement_b.position.x).abs();
            let dy = (settlement_a.position.y - settlement_b.position.y).abs();
            ((dx * dx + dy * dy) as f32).sqrt()
        };
        
        match total_pop {
            0..500 => {
                if distance > 200.0 || rng.gen_bool(0.3) {
                    RoadType::Trail
                } else {
                    RoadType::Path
                }
            }
            500..2000 => {
                if rng.gen_bool(0.7) {
                    RoadType::Path
                } else {
                    RoadType::Road
                }
            }
            2000..5000 => {
                if rng.gen_bool(0.6) {
                    RoadType::Road
                } else {
                    RoadType::Highway
                }
            }
            _ => {
                if rng.gen_bool(0.4) {
                    RoadType::Highway
                } else {
                    RoadType::Imperial
                }
            }
        }
    }
    
    fn determine_inter_zone_road_type(&self, settlement_a: &Settlement, settlement_b: &Settlement, rng: &mut ChaCha8Rng) -> RoadType {
        // Inter-zone roads tend to be more important
        let base_type = self.determine_road_type(settlement_a, settlement_b, rng);
        match base_type {
            RoadType::Trail => RoadType::Path,
            RoadType::Path => if rng.gen_bool(0.5) { RoadType::Road } else { RoadType::Path },
            other => other,
        }
    }
    
    fn calculate_road_width(&self, road_type: &RoadType) -> u32 {
        match road_type {
            RoadType::Trail => 1,
            RoadType::Path => 2,
            RoadType::Road => 3,
            RoadType::Highway => 4,
            RoadType::Imperial => 6,
        }
    }
    
    fn determine_traffic_level(&self, road_type: &RoadType, settlement_a: &Settlement, settlement_b: &Settlement) -> TrafficLevel {
        let total_pop = settlement_a.population + settlement_b.population;
        let base_level = match road_type {
            RoadType::Trail => TrafficLevel::Light,
            RoadType::Path => TrafficLevel::Light,
            RoadType::Road => TrafficLevel::Moderate,
            RoadType::Highway => TrafficLevel::Heavy,
            RoadType::Imperial => TrafficLevel::Major,
        };
        
        // Adjust based on population
        match (base_level, total_pop) {
            (TrafficLevel::Light, 0..200) => TrafficLevel::Light,
            (TrafficLevel::Light, _) => TrafficLevel::Moderate,
            (TrafficLevel::Moderate, 0..1000) => TrafficLevel::Moderate,
            (TrafficLevel::Moderate, _) => TrafficLevel::Heavy,
            (other, _) => other,
        }
    }
    
    fn add_additional_roads(&self, roads: &mut Vec<Road>, settlements: &[Settlement], terrain: &TerrainMap, rng: &mut ChaCha8Rng) {
        // Add some additional roads for major settlements (redundant connections)
        for (i, settlement) in settlements.iter().enumerate() {
            if settlement.population > 1000 {
                // Try to connect to another major settlement
                for (j, other_settlement) in settlements.iter().enumerate() {
                    if i != j && other_settlement.population > 500 && rng.gen_bool(0.3) {
                        // Check if already connected
                        let already_connected = roads.iter().any(|road| {
                            (road.path.first() == Some(&settlement.position) && road.path.last() == Some(&other_settlement.position)) ||
                            (road.path.first() == Some(&other_settlement.position) && road.path.last() == Some(&settlement.position))
                        });
                        
                        if !already_connected {
                            if let Some(path) = self.find_path(settlement.position, other_settlement.position, terrain) {
                                let road_type = self.determine_road_type(settlement, other_settlement, rng);
                                roads.push(Road {
                                    path,
                                    road_type: road_type.clone(),
                                    condition: rng.gen_range(0.6..0.9),
                                    width: self.calculate_road_width(&road_type),
                                    traffic_level: self.determine_traffic_level(&road_type, settlement, other_settlement),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn find_border_position(&self, _current_zone: ZoneCoord, adjacent_zone: ZoneCoord, _source: LocalCoord, _target_world: WorldCoord) -> Option<LocalCoord> {
        // Simplified: just find a position on the appropriate border
        let zone_diff_x = adjacent_zone.x - _current_zone.x;
        let zone_diff_y = adjacent_zone.y - _current_zone.y;
        
        let border_x = if zone_diff_x > 0 {
            ZONE_SIZE - 1
        } else if zone_diff_x < 0 {
            0
        } else {
            ZONE_SIZE / 2
        };
        
        let border_y = if zone_diff_y > 0 {
            ZONE_SIZE - 1
        } else if zone_diff_y < 0 {
            0
        } else {
            ZONE_SIZE / 2
        };
        
        Some(LocalCoord::new(border_x, border_y))
    }
    
    fn find_suitable_border_position(&self, current_zone: ZoneCoord, adjacent_zone: ZoneCoord, _source: LocalCoord, _terrain: &TerrainMap) -> Option<LocalCoord> {
        // For now, just use the same logic as find_border_position
        self.find_border_position(current_zone, adjacent_zone, _source, WorldCoord::new(0, 0))
    }
    
    fn get_exit_direction(&self, current_zone: ZoneCoord, adjacent_zone: ZoneCoord) -> ExitDirection {
        let dx = adjacent_zone.x - current_zone.x;
        let dy = adjacent_zone.y - current_zone.y;
        
        match (dx, dy) {
            (0, 1) => ExitDirection::North,
            (0, -1) => ExitDirection::South,
            (1, 0) => ExitDirection::East,
            (-1, 0) => ExitDirection::West,
            (1, 1) => ExitDirection::Northeast,
            (-1, 1) => ExitDirection::Northwest,
            (1, -1) => ExitDirection::Southeast,
            (-1, -1) => ExitDirection::Southwest,
            _ => ExitDirection::North, // Fallback
        }
    }
}

impl RoadNetwork {
    pub fn get_road_at(&self, position: LocalCoord) -> Option<&Road> {
        self.roads.iter().find(|road| road.path.contains(&position))
    }
    
    pub fn find_route(&self, start: LocalCoord, end: LocalCoord) -> Option<Vec<LocalCoord>> {
        // Simple pathfinding using the road network
        let mut queue = std::collections::VecDeque::new();
        let mut visited = HashSet::new();
        let mut came_from = HashMap::new();
        
        queue.push_back(start);
        visited.insert(start);
        
        while let Some(current) = queue.pop_front() {
            if current == end {
                let mut path = vec![current];
                let mut pos = current;
                while let Some(&prev) = came_from.get(&pos) {
                    path.push(prev);
                    pos = prev;
                }
                path.reverse();
                return Some(path);
            }
            
            if let Some(neighbors) = self.connections.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        came_from.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        
        None
    }
}

impl RoadType {
    pub fn get_ascii_char(&self) -> char {
        match self {
            RoadType::Trail => '·',
            RoadType::Path => '-',
            RoadType::Road => '=',
            RoadType::Highway => '≡',
            RoadType::Imperial => '━',
        }
    }
    
    pub fn get_name(&self) -> &'static str {
        match self {
            RoadType::Trail => "Trail",
            RoadType::Path => "Path", 
            RoadType::Road => "Road",
            RoadType::Highway => "Highway",
            RoadType::Imperial => "Imperial Road",
        }
    }
}
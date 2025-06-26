use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use super::{WorldZone, LocalCoord, ZoneCoord, TerrainType, ZONE_SIZE};

pub struct WorldRenderer {
    pub viewport_width: i32,
    pub viewport_height: i32,
}

impl WorldRenderer {
    pub fn new(viewport_width: i32, viewport_height: i32) -> Self {
        Self {
            viewport_width,
            viewport_height,
        }
    }
    
    pub fn render_zone_view(&self, zone: &WorldZone, player_pos: LocalCoord, view_distance: i32) -> Vec<Line> {
        let mut lines = Vec::new();
        
        let start_x = (player_pos.x - view_distance).max(0);
        let end_x = (player_pos.x + view_distance).min(ZONE_SIZE - 1);
        let start_y = (player_pos.y - view_distance).max(0);
        let end_y = (player_pos.y + view_distance).min(ZONE_SIZE - 1);
        
        for y in start_y..=end_y {
            let mut spans = Vec::new();
            
            for x in start_x..=end_x {
                let coord = LocalCoord::new(x, y);
                let char_and_color = self.get_tile_representation(zone, coord, player_pos);
                
                spans.push(Span::styled(
                    char_and_color.0.to_string(),
                    Style::default().fg(char_and_color.1)
                ));
            }
            
            lines.push(Line::from(spans));
        }
        
        lines
    }
    
    pub fn render_minimap(&self, zone: &WorldZone, player_pos: LocalCoord) -> Vec<Line> {
        let mut lines = Vec::new();
        let scale = 8i32; // Show every 8th tile
        
        for y in (0..ZONE_SIZE).step_by(scale as usize) {
            let mut spans = Vec::new();
            
            for x in (0..ZONE_SIZE).step_by(scale as usize) {
                let coord = LocalCoord::new(x, y);
                let (char, color) = if coord.x == (player_pos.x / scale) * scale && coord.y == (player_pos.y / scale) * scale {
                    ('@', Color::Yellow) // Player position
                } else {
                    self.get_minimap_representation(zone, coord)
                };
                
                spans.push(Span::styled(char.to_string(), Style::default().fg(color)));
            }
            
            lines.push(Line::from(spans));
        }
        
        lines
    }
    
    fn get_tile_representation(&self, zone: &WorldZone, coord: LocalCoord, player_pos: LocalCoord) -> (char, Color) {
        // Player position
        if coord == player_pos {
            return ('@', Color::Yellow);
        }
        
        // Check for settlements
        if let Some(settlement) = zone.get_settlement_at(coord) {
            return (settlement.settlement_type.get_ascii_char(), Color::White);
        }
        
        // Check for points of interest
        if let Some(_poi) = zone.get_poi_at(coord) {
            return ('?', Color::Magenta);
        }
        
        // Check for roads
        if let Some(road) = zone.roads.get_road_at(coord) {
            return (road.road_type.get_ascii_char(), Color::Gray);
        }
        
        // Check for rivers
        for river in &zone.rivers {
            if river.contains_position(coord) {
                return (river.river_type.get_ascii_char(), Color::Blue);
            }
        }
        
        // Default to terrain
        let tile = zone.terrain.get_tile(coord);
        (tile.terrain_type.get_ascii_char(), self.get_terrain_color(&tile.terrain_type))
    }
    
    fn get_minimap_representation(&self, zone: &WorldZone, coord: LocalCoord) -> (char, Color) {
        // Check for settlements first (highest priority)
        if let Some(settlement) = zone.get_settlement_at(coord) {
            return (settlement.settlement_type.get_ascii_char(), Color::White);
        }
        
        // Check for major roads
        if let Some(road) = zone.roads.get_road_at(coord) {
            if matches!(road.road_type, super::RoadType::Highway | super::RoadType::Imperial) {
                return ('═', Color::Gray);
            }
        }
        
        // Check for major rivers
        for river in &zone.rivers {
            if river.contains_position(coord) {
                if matches!(river.river_type, super::RiverType::River | super::RiverType::MajorRiver) {
                    return ('~', Color::Blue);
                }
            }
        }
        
        // Terrain with reduced detail
        let tile = zone.terrain.get_tile(coord);
        match tile.terrain_type {
            TerrainType::Mountain => ('▲', Color::Gray),
            TerrainType::Hill => ('∩', Color::DarkGray),
            TerrainType::Forest => ('♣', Color::Green),
            TerrainType::Desert => ('░', Color::Yellow),
            TerrainType::Ocean => ('~', Color::Blue),
            TerrainType::Lake => ('○', Color::Cyan),
            _ => ('.', Color::DarkGray),
        }
    }
    
    fn get_terrain_color(&self, terrain_type: &TerrainType) -> Color {
        match terrain_type {
            TerrainType::Ocean => Color::Blue,
            TerrainType::Lake => Color::Cyan,
            TerrainType::River => Color::Cyan,
            TerrainType::Swamp => Color::Green,
            TerrainType::Desert => Color::Yellow,
            TerrainType::Plains => Color::Green,
            TerrainType::Grassland => Color::LightGreen,
            TerrainType::Forest => Color::Green,
            TerrainType::Hill => Color::Rgb(139, 69, 19), // Brown
            TerrainType::Mountain => Color::Gray,
            TerrainType::Snow => Color::White,
            TerrainType::Tundra => Color::DarkGray,
        }
    }
    
    pub fn render_location_info(&self, zone: &WorldZone, player_pos: LocalCoord) -> Vec<Line> {
        let mut lines = vec![
            Line::from(Span::styled("Location Info", Style::default().fg(Color::Yellow))),
            Line::from(""),
        ];
        
        // Current position
        lines.push(Line::from(format!("Position: ({}, {})", player_pos.x, player_pos.y)));
        
        // Current terrain
        let tile = zone.terrain.get_tile(player_pos);
        lines.push(Line::from(format!("Terrain: {:?}", tile.terrain_type)));
        lines.push(Line::from(format!("Elevation: {:.1}m", tile.elevation * 1000.0)));
        
        // Nearby features
        if let Some(settlement) = zone.get_settlement_at(player_pos) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Settlement:", Style::default().fg(Color::White))));
            lines.extend(settlement.get_display_info().into_iter().map(Line::from));
        }
        
        if let Some(poi) = zone.get_poi_at(player_pos) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Point of Interest:", Style::default().fg(Color::Magenta))));
            lines.push(Line::from(poi.name.clone()));
            lines.push(Line::from(poi.description.clone()));
        }
        
        if let Some(road) = zone.roads.get_road_at(player_pos) {
            lines.push(Line::from(""));
            lines.push(Line::from(format!("Road: {} (Condition: {:.0}%)", 
                road.road_type.get_name(), road.condition * 100.0)));
        }
        
        for river in &zone.rivers {
            if river.contains_position(player_pos) {
                lines.push(Line::from(""));
                lines.push(Line::from(format!("River: {} (Width: {}m)", 
                    river.river_type.get_name(), 
                    river.get_width_at(player_pos).unwrap_or(1))));
                break;
            }
        }
        
        lines
    }
    
    pub fn render_zone_overview(&self, zone: &WorldZone) -> Vec<Line> {
        let mut lines = vec![
            Line::from(Span::styled("Zone Overview", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(format!("Zone: ({}, {})", zone.coord.x, zone.coord.y)),
            Line::from(format!("Generated: {}", zone.generated_at.format("%Y-%m-%d %H:%M UTC"))),
        ];
        
        if let Some(last_visited) = zone.last_visited {
            lines.push(Line::from(format!("Last Visited: {}", last_visited.format("%Y-%m-%d %H:%M UTC"))));
        }
        
        lines.push(Line::from(""));
        lines.push(Line::from("Features:"));
        lines.push(Line::from(format!("  Settlements: {}", zone.settlements.len())));
        lines.push(Line::from(format!("  Roads: {}", zone.roads.roads.len())));
        lines.push(Line::from(format!("  Rivers: {}", zone.rivers.len())));
        lines.push(Line::from(format!("  Points of Interest: {}", zone.points_of_interest.len())));
        
        if !zone.settlements.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from("Settlements:"));
            for settlement in &zone.settlements {
                lines.push(Line::from(format!("  {} ({}) - Pop: {}", 
                    settlement.name,
                    settlement.settlement_type.get_name(),
                    settlement.population
                )));
            }
        }
        
        lines
    }
    
    pub fn render_world_map(&self, zones: &[(ZoneCoord, Option<&WorldZone>)], player_zone: ZoneCoord) -> Vec<Line> {
        let mut lines = Vec::new();
        
        // Find bounds
        let min_x = zones.iter().map(|(coord, _)| coord.x).min().unwrap_or(0);
        let max_x = zones.iter().map(|(coord, _)| coord.x).max().unwrap_or(0);
        let min_y = zones.iter().map(|(coord, _)| coord.y).min().unwrap_or(0);
        let max_y = zones.iter().map(|(coord, _)| coord.y).max().unwrap_or(0);
        
        for y in min_y..=max_y {
            let mut spans = Vec::new();
            
            for x in min_x..=max_x {
                let coord = ZoneCoord::new(x, y);
                let (char, color) = if coord == player_zone {
                    ('@', Color::Yellow)
                } else if let Some((_, Some(zone))) = zones.iter().find(|(c, _)| *c == coord) {
                    if zone.settlements.is_empty() {
                        ('·', Color::DarkGray)
                    } else {
                        let largest = zone.settlements.iter().max_by_key(|s| s.population).unwrap();
                        (largest.settlement_type.get_ascii_char(), Color::White)
                    }
                } else {
                    (' ', Color::Black)
                };
                
                spans.push(Span::styled(char.to_string(), Style::default().fg(color)));
            }
            
            lines.push(Line::from(spans));
        }
        
        lines
    }
}

pub fn render_world_ui(f: &mut Frame, zone: &WorldZone, player_pos: LocalCoord, viewport: ratatui::layout::Rect) {
    let renderer = WorldRenderer::new(viewport.width as i32, viewport.height as i32);
    
    // Split the area into main view and info panel
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(70),
            ratatui::layout::Constraint::Percentage(30),
        ])
        .split(viewport);
    
    let left_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage(80),
            ratatui::layout::Constraint::Percentage(20),
        ])
        .split(chunks[0]);
    
    // Main world view
    let view_distance = 20;
    let world_lines = renderer.render_zone_view(zone, player_pos, view_distance);
    let world_view = Paragraph::new(world_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("World View"))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(world_view, left_chunks[0]);
    
    // Minimap
    let minimap_lines = renderer.render_minimap(zone, player_pos);
    let minimap = Paragraph::new(minimap_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Minimap"))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(minimap, left_chunks[1]);
    
    // Info panel
    let info_lines = renderer.render_location_info(zone, player_pos);
    let info_panel = Paragraph::new(info_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Location"))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(info_panel, chunks[1]);
}

// Helper function to create a legend for terrain symbols
pub fn create_terrain_legend() -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled("Terrain Legend:", Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from("~ Ocean/Sea"),
        Line::from("○ Lake"),
        Line::from("≈ River"),
        Line::from("♠ Swamp"),
        Line::from("░ Desert"),
        Line::from(". Plains"),
        Line::from(", Grassland"),
        Line::from("♣ Forest"),
        Line::from("∩ Hills"),
        Line::from("▲ Mountains"),
        Line::from("* Snow"),
        Line::from("· Tundra"),
        Line::from(""),
        Line::from("Settlements:"),
        Line::from("• Outpost"),
        Line::from("○ Village"),
        Line::from("● Town"),
        Line::from("◉ City"),
        Line::from("⬟ Capital"),
        Line::from(""),
        Line::from("Roads:"),
        Line::from("· Trail"),
        Line::from("- Path"),
        Line::from("= Road"),
        Line::from("≡ Highway"),
        Line::from("━ Imperial Road"),
        Line::from(""),
        Line::from("? Point of Interest"),
        Line::from("@ You"),
    ]
}
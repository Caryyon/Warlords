use serde::{Deserialize, Serialize};
use rand_chacha::ChaCha8Rng;
use super::{ZoneCoord, LocalCoord, TerrainMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct River {
    pub segments: Vec<RiverSegment>,
    pub river_type: RiverType,
    pub flow_direction: FlowDirection,
    pub source: Option<LocalCoord>,
    pub mouth: Option<LocalCoord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiverSegment {
    pub path: Vec<LocalCoord>,
    pub width: u32,
    pub depth: f32,
    pub flow_rate: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiverType {
    Stream,      // Small mountain stream
    Creek,       // Larger than stream, permanent water
    River,       // Main waterway
    MajorRiver,  // Large river system
    Tributary,   // Feeds into larger river
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowDirection {
    North,
    South,
    East,
    West,
    Northeast,
    Northwest,
    Southeast,
    Southwest,
}

pub struct RiverGenerator;

impl RiverGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate(&self, _coord: ZoneCoord, _terrain: &TerrainMap, _rng: &mut ChaCha8Rng) -> Vec<River> {
        // Simplified - no rivers for now to avoid complexity
        Vec::new()
    }
}

impl River {
    pub fn contains_position(&self, _coord: LocalCoord) -> bool {
        // Simplified - no rivers contain any positions
        false
    }
    
    pub fn get_width_at(&self, _coord: LocalCoord) -> Option<u32> {
        // Simplified - no rivers have width
        None
    }
}

impl RiverType {
    pub fn get_ascii_char(&self) -> char {
        match self {
            RiverType::Stream => '~',
            RiverType::Creek => '≈',
            RiverType::River => '≈',
            RiverType::MajorRiver => '≈',
            RiverType::Tributary => '~',
        }
    }
    
    pub fn get_name(&self) -> &'static str {
        match self {
            RiverType::Stream => "Stream",
            RiverType::Creek => "Creek",
            RiverType::River => "River",
            RiverType::MajorRiver => "Major River",
            RiverType::Tributary => "Tributary",
        }
    }
}
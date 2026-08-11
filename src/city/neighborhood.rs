use macroquad_toolkit::rng;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

/// The type of neighborhood - each has distinct characteristics affecting gameplay
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NeighborhoodType {
    /// High rent, high turnover, noisy, good for professionals
    Downtown,
    /// Families, stability, lower margins, quiet
    Suburbs,
    /// Students, artists, gentrification pressure, affordable
    Industrial,
    /// Elderly, regulations, preservation requirements, historic charm
    Historic,
}

impl NeighborhoodType {
    pub fn name(&self) -> &'static str {
        match self {
            NeighborhoodType::Downtown => "Downtown",
            NeighborhoodType::Suburbs => "Suburbs",
            NeighborhoodType::Industrial => "Industrial District",
            NeighborhoodType::Historic => "Historic Quarter",
        }
    }

    /// Color for UI display
    pub fn color(&self) -> macroquad::color::Color {
        use macroquad::color::Color;
        match self {
            NeighborhoodType::Downtown => Color::from_rgba(100, 149, 237, 255),
            NeighborhoodType::Suburbs => Color::from_rgba(144, 238, 144, 255),
            NeighborhoodType::Industrial => Color::from_rgba(255, 165, 79, 255),
            NeighborhoodType::Historic => Color::from_rgba(221, 160, 221, 255),
        }
    }
}

/// Dynamic stats for a neighborhood that change over time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeighborhoodStats {
    /// Crime level (0-100, higher = worse)
    pub crime_level: i32,
    /// Transit access score (0-100)
    pub transit_access: i32,
    /// Walkability score (0-100)
    pub walkability: i32,
    /// School quality (0-100) - matters for families
    pub school_quality: i32,
    /// Local services (shops, cafes, etc.) (0-100)
    pub services: i32,
    /// Current rent demand (affects application rate)
    pub rent_demand: f32,
    /// Gentrification pressure (0-100)
    pub gentrification: i32,
}

impl NeighborhoodStats {
    pub fn for_type(neighborhood_type: &NeighborhoodType) -> Self {
        // Load config (lazy/cached would be better but this is only called at startup)
        let config_map = load_neighborhood_config();

        let type_key = match neighborhood_type {
            NeighborhoodType::Downtown => "Downtown",
            NeighborhoodType::Suburbs => "Suburbs",
            NeighborhoodType::Industrial => "Industrial",
            NeighborhoodType::Historic => "Historic",
        };

        if let Some(stats) = config_map.get(type_key) {
            stats.clone()
        } else {
            // Fallback defaults
            match neighborhood_type {
                NeighborhoodType::Downtown => Self {
                    crime_level: 40,
                    transit_access: 95,
                    walkability: 90,
                    school_quality: 50,
                    services: 95,
                    rent_demand: 1.2,
                    gentrification: 80,
                },
                NeighborhoodType::Suburbs => Self {
                    crime_level: 15,
                    transit_access: 40,
                    walkability: 30,
                    school_quality: 85,
                    services: 60,
                    rent_demand: 1.0,
                    gentrification: 20,
                },
                NeighborhoodType::Industrial => Self {
                    crime_level: 50,
                    transit_access: 60,
                    walkability: 50,
                    school_quality: 35,
                    services: 45,
                    rent_demand: 0.9,
                    gentrification: 60,
                },
                NeighborhoodType::Historic => Self {
                    crime_level: 25,
                    transit_access: 70,
                    walkability: 75,
                    school_quality: 65,
                    services: 80,
                    rent_demand: 1.1,
                    gentrification: 40,
                },
            }
        }
    }

    /// Apply monthly changes to neighborhood (gentrification, crime changes, etc.)
    pub fn tick(&mut self, neighborhood_type: &NeighborhoodType) {
        // Gentrification slowly increases in industrial areas
        if matches!(neighborhood_type, NeighborhoodType::Industrial)
            && self.gentrification < 100
            && rng::gen_range(0, 100) < 10
        {
            self.gentrification = (self.gentrification + 1).min(100);
            // Gentrification increases rent demand but pushes out long-term residents
            self.rent_demand = (self.rent_demand + 0.01).min(1.5);
        }

        // Crime fluctuates slightly
        let crime_change = rng::gen_range(-2, 3);
        self.crime_level = (self.crime_level + crime_change).clamp(5, 95);

        // Rent demand fluctuates
        let demand_change = rng::gen_range(-5, 6) as f32 / 100.0;
        self.rent_demand = (self.rent_demand + demand_change).clamp(0.5, 2.0);
    }
}

/// A neighborhood in the city containing multiple building slots
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Neighborhood {
    pub id: u32,
    pub neighborhood_type: NeighborhoodType,
    pub name: String,
    pub stats: NeighborhoodStats,
    /// IDs of buildings in this neighborhood (player-owned)
    pub building_ids: Vec<u32>,
    /// Number of available property slots
    pub available_slots: u32,
    /// Player's reputation in this neighborhood (0-100)
    pub reputation: i32,
}

impl Neighborhood {
    pub fn new(id: u32, neighborhood_type: NeighborhoodType, name: &str) -> Self {
        let stats = NeighborhoodStats::for_type(&neighborhood_type);
        Self {
            id,
            neighborhood_type,
            name: name.to_string(),
            stats,
            building_ids: Vec::new(),
            available_slots: 3, // Can acquire up to 3 buildings per neighborhood
            reputation: 50,
        }
    }

    /// Add a building to this neighborhood
    pub fn add_building(&mut self, building_id: u32) {
        if !self.building_ids.contains(&building_id) {
            self.building_ids.push(building_id);
        }
    }

    /// Check if we can add more buildings
    pub fn can_add_building(&self) -> bool {
        (self.building_ids.len() as u32) < self.available_slots
    }

    /// Historic quarters carry preservation regulations (stricter inspections).
    pub fn is_historic(&self) -> bool {
        matches!(self.neighborhood_type, NeighborhoodType::Historic)
    }

    /// Apply monthly tick
    pub fn tick(&mut self) {
        self.stats.tick(&self.neighborhood_type);
    }
}

fn load_neighborhood_config() -> HashMap<String, NeighborhoodStats> {
    #[cfg(target_arch = "wasm32")]
    let json = macroquad_toolkit::include_json_str!("../../assets/neighborhoods.json");

    #[cfg(not(target_arch = "wasm32"))]
    let json = std::fs::read_to_string("assets/neighborhoods.json").unwrap_or_else(|_| {
        macroquad_toolkit::include_json_str!("../../assets/neighborhoods.json").to_string()
    });

    serde_json::from_str(&json).unwrap_or_default()
}

#[cfg(test)]
mod tests;

use crate::building::{ApartmentSize, DesignType, NoiseLevel};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingTemplates {
    pub templates: Vec<BuildingTemplate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingTemplate {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub unlock_order: u32,
    #[serde(default)]
    pub difficulty: String,
    /// Which city neighborhood this building sits in (0=Downtown, 1=Suburbs,
    /// 2=Industrial, 3=Historic). Drives the building's campaign identity —
    /// e.g. a Historic-quarter building activates preservation regulations.
    #[serde(default)]
    pub neighborhood_id: u32,
    #[serde(default)]
    pub description: String,
    pub floors: u32,
    pub units_per_floor: u32,
    pub hallway_condition: i32,
    pub apartments: Vec<ApartmentTemplate>,
    pub initial_tenant: Option<InitialTenantData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApartmentTemplate {
    pub unit_number: String,
    pub floor: u32,
    #[serde(rename = "size")]
    pub size_str: String,
    #[serde(rename = "base_noise")]
    pub base_noise_str: String,
    pub initial_condition: i32,
    pub initial_design: String,
    pub initial_rent: i32,
}

impl ApartmentTemplate {
    pub fn size(&self) -> ApartmentSize {
        match self.size_str.to_lowercase().as_str() {
            "small" => ApartmentSize::Small,
            "medium" => ApartmentSize::Medium,
            "large" => ApartmentSize::Large,
            "penthouse" => ApartmentSize::Penthouse,
            _ => ApartmentSize::Medium,
        }
    }

    pub fn base_noise(&self) -> NoiseLevel {
        match self.base_noise_str.to_lowercase().as_str() {
            "low" => NoiseLevel::Low,
            "high" => NoiseLevel::High,
            _ => NoiseLevel::Low, // Default to Low for unknown values
        }
    }

    pub fn initial_design(&self) -> DesignType {
        match self.initial_design.to_lowercase().as_str() {
            "bare" => DesignType::Bare,
            "practical" => DesignType::Practical,
            "cozy" => DesignType::Cozy,
            "luxury" => DesignType::Luxury,
            "opulent" => DesignType::Opulent,
            _ => DesignType::Bare,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitialTenantData {
    pub apartment_unit: String,
    pub archetype: String,
    pub name: String,
}

pub fn load_templates() -> Option<BuildingTemplates> {
    // For WASM, embed at compile time
    #[cfg(target_arch = "wasm32")]
    let json = macroquad_toolkit::include_json_str!("../../assets/building_templates.json");

    #[cfg(not(target_arch = "wasm32"))]
    let json = match fs::read_to_string("assets/building_templates.json") {
        Ok(s) => s,
        Err(_) => {
            macroquad_toolkit::include_json_str!("../../assets/building_templates.json").to_string()
        }
    };

    match serde_json::from_str::<BuildingTemplates>(&json) {
        Ok(templates) => Some(templates),
        Err(e) => {
            eprintln!("Failed to parse building_templates.json: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests;

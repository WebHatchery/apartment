use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TenantEventsConfig {
    pub requests: HashMap<String, Vec<RequestTemplate>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RequestTemplate {
    Pet {
        options: Vec<String>,
        weight: u32,
    },
    Sublease {
        weight: u32,
    },
    HomeBusiness {
        options: Vec<String>,
        weight: u32,
    },
    Modification {
        options: Vec<String>,
        weight: u32,
    },
    TemporaryGuest {
        options: Vec<String>,
        duration_min: u32,
        duration_max: u32,
        weight: u32,
    },
    None {
        weight: u32,
    },
}

pub fn load_events_config() -> TenantEventsConfig {
    #[cfg(target_arch = "wasm32")]
    let json = macroquad_toolkit::include_json_str!("../../assets/tenant_events.json");

    #[cfg(not(target_arch = "wasm32"))]
    let json = std::fs::read_to_string("assets/tenant_events.json").unwrap_or_else(|_| {
        macroquad_toolkit::include_json_str!("../../assets/tenant_events.json").to_string()
    });

    serde_json::from_str(&json).unwrap_or_else(|e| {
        eprintln!("Failed to parse tenant_events.json: {}", e);
        TenantEventsConfig::default()
    })
}

#[cfg(test)]
mod tests;

use crate::narrative::events::NarrativeEffect;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RelationshipEventsConfig {
    #[serde(default)]
    pub hostile: Vec<RelationshipEventTemplate>,
    #[serde(default)]
    pub friendly: Vec<RelationshipEventTemplate>,
    #[serde(default)]
    pub romance: Vec<RelationshipEventTemplate>,
    /// Emergent "high-rent tenant vs. unhappy neighbors" dilemmas.
    /// Placeholders: {tenant}/{apt}/{rent}/{victim_count}/{victims} in text;
    /// in effects, tenant_id 0 = the disruptor, 1 = each affected neighbor.
    #[serde(default)]
    pub dilemma: Vec<RelationshipEventTemplate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipEventTemplate {
    pub id: String,
    pub trigger_strength_min: Option<i32>,
    pub trigger_strength_max: Option<i32>,
    pub probability: u32,
    pub headline: String,
    pub description: String,
    #[serde(default)]
    pub choices: Vec<RelationshipChoiceTemplate>,
    #[serde(default)]
    pub default_effect: Option<NarrativeEffect>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipChoiceTemplate {
    pub label: String,
    pub description: String,
    pub effect: NarrativeEffect,
    #[serde(default)]
    pub reputation_change: i32,
}

pub fn load_relationship_config() -> RelationshipEventsConfig {
    #[cfg(target_arch = "wasm32")]
    let json = macroquad_toolkit::include_json_str!("../../assets/relationship_events.json");

    #[cfg(not(target_arch = "wasm32"))]
    let json = std::fs::read_to_string("assets/relationship_events.json").unwrap_or_else(|_| {
        macroquad_toolkit::include_json_str!("../../assets/relationship_events.json").to_string()
    });

    serde_json::from_str(&json).unwrap_or_else(|e| {
        eprintln!("Failed to parse relationship_events.json: {}", e);
        RelationshipEventsConfig::default()
    })
}

#[cfg(test)]
mod tests;

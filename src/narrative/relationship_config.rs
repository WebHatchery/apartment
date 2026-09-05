use crate::narrative::events::NarrativeEffect;
use macroquad_toolkit::data_loader::{load_json_file_with_fallback_sync, JsonFallbackPolicy};
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
    let json_result = load_json_file_with_fallback_sync(
        "assets/relationship_events.json",
        macroquad_toolkit::include_json_str!("../../assets/relationship_events.json"),
        JsonFallbackPolicy::ReadError,
    );

    json_result.unwrap_or_else(|e| {
        eprintln!("Failed to parse relationship_events.json: {}", e);
        RelationshipEventsConfig::default()
    })
}

#[cfg(test)]
mod tests;

//! Tenant archetypes and their housing preferences.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TenantArchetype {
    Student,
    Professional,
    Artist,
    Family,
    Elderly,
}

impl TenantArchetype {
    pub fn name(&self) -> &'static str {
        match self {
            TenantArchetype::Student => "Student",
            TenantArchetype::Professional => "Professional",
            TenantArchetype::Artist => "Artist",
            TenantArchetype::Family => "Family",
            TenantArchetype::Elderly => "Elderly",
        }
    }

    /// Get archetype from string ID
    pub fn from_id(id: &str) -> Option<Self> {
        match id.to_lowercase().as_str() {
            "student" => Some(TenantArchetype::Student),
            "professional" => Some(TenantArchetype::Professional),
            "artist" => Some(TenantArchetype::Artist),
            "family" => Some(TenantArchetype::Family),
            "elderly" => Some(TenantArchetype::Elderly),
            _ => None,
        }
    }

    /// Get the ID used in JSON files
    pub fn id(&self) -> &'static str {
        match self {
            TenantArchetype::Student => "student",
            TenantArchetype::Professional => "professional",
            TenantArchetype::Artist => "artist",
            TenantArchetype::Family => "family",
            TenantArchetype::Elderly => "elderly",
        }
    }

    /// Preferences from `tenant_archetypes.json`, the only source of them;
    /// the registry is validated to define every archetype when it loads.
    pub fn preferences(&self) -> ArchetypePreferences {
        let definition = crate::data::archetypes::archetypes()
            .get(self.id())
            .expect("tenant_archetypes.json defines every archetype");
        crate::data::archetypes::ArchetypeRegistry::to_preferences(&definition.preferences)
    }
}

#[derive(Clone, Debug)]
pub struct ArchetypePreferences {
    // Sensitivity weights (0.0 - 1.0, higher = more affected)
    pub rent_sensitivity: f32,
    pub condition_sensitivity: f32,
    pub noise_sensitivity: f32,
    pub design_sensitivity: f32,

    // Thresholds
    pub ideal_rent_max: i32,
    pub min_acceptable_condition: i32,
    pub prefers_quiet: bool,

    // Design preferences
    pub preferred_design: Option<crate::building::DesignType>,
    pub hates_design: Option<crate::building::DesignType>,
}

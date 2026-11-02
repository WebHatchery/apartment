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

    /// Get the preferences for this archetype
    /// Attempts to load from JSON registry first, falls back to hardcoded defaults
    pub fn preferences(&self) -> ArchetypePreferences {
        // Try to load from JSON registry
        let registry = crate::data::archetypes::archetypes();
        if let Some(definition) = registry.get(self.id()) {
            return crate::data::archetypes::ArchetypeRegistry::to_preferences(
                &definition.preferences,
            );
        }

        // Fallback to hardcoded values
        self.default_preferences()
    }

    /// Hardcoded default preferences (fallback if JSON fails to load)
    fn default_preferences(&self) -> ArchetypePreferences {
        match self {
            TenantArchetype::Student => ArchetypePreferences {
                rent_sensitivity: 0.9,      // Very price sensitive
                condition_sensitivity: 0.3, // Low - tolerates some wear
                noise_sensitivity: 0.4,     // Low - can deal with noise
                design_sensitivity: 0.2,    // Doesn't care much

                ideal_rent_max: 750,
                min_acceptable_condition: 30,
                prefers_quiet: false,
                preferred_design: None,
                hates_design: None,
            },
            TenantArchetype::Professional => ArchetypePreferences {
                rent_sensitivity: 0.4,      // Can afford more
                condition_sensitivity: 0.8, // Values good condition
                noise_sensitivity: 0.9,     // Hates noise
                design_sensitivity: 0.5,    // Moderate

                ideal_rent_max: 1200,
                min_acceptable_condition: 60,
                prefers_quiet: true,
                preferred_design: None,
                hates_design: None,
            },
            TenantArchetype::Artist => ArchetypePreferences {
                rent_sensitivity: 0.6,      // Moderate budget
                condition_sensitivity: 0.5, // Moderate
                noise_sensitivity: 0.5,     // Moderate
                design_sensitivity: 0.95,   // Very design focused

                ideal_rent_max: 900,
                min_acceptable_condition: 40,
                prefers_quiet: false,
                preferred_design: Some(crate::building::DesignType::Cozy),
                hates_design: Some(crate::building::DesignType::Bare),
            },
            TenantArchetype::Family => ArchetypePreferences {
                rent_sensitivity: 0.7,      // Moderate-High (kids are expensive)
                condition_sensitivity: 0.7, // Needs decent condition
                noise_sensitivity: 1.0,     // Hates noise (kids sleeping)
                design_sensitivity: 0.4,    // Moderate

                ideal_rent_max: 1100,
                min_acceptable_condition: 50,
                prefers_quiet: true,
                preferred_design: Some(crate::building::DesignType::Practical),
                hates_design: None,
            },
            TenantArchetype::Elderly => ArchetypePreferences {
                rent_sensitivity: 0.8,      // Fixed income
                condition_sensitivity: 0.6, // Moderate
                noise_sensitivity: 0.9,     // Hates noise
                design_sensitivity: 0.3,    // Low

                ideal_rent_max: 800,
                min_acceptable_condition: 45,
                prefers_quiet: true,
                preferred_design: None,
                hates_design: Some(crate::building::DesignType::Bare), // Wants some comfort
            },
        }
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

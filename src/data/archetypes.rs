//! Tenant archetype definitions loaded from `tenant_archetypes.json`.

use crate::building::DesignType;
use macroquad_toolkit::data_loader::{load_json_file_with_fallback_sync, JsonFallbackPolicy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Loaded archetype data from JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchetypeData {
    pub archetypes: Vec<ArchetypeDefinition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchetypeDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub spawn_weight: u32,
    pub preferences: ArchetypePreferencesData,
    pub name_pool: NamePool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchetypePreferencesData {
    pub rent_sensitivity: f32,
    pub condition_sensitivity: f32,
    pub noise_sensitivity: f32,
    pub design_sensitivity: f32,
    pub ideal_rent_max: i32,
    pub min_acceptable_condition: i32,
    pub prefers_quiet: bool,
    pub preferred_design: Option<String>,
    pub hates_design: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NamePool {
    pub first_names: Vec<String>,
    pub last_initials: Vec<String>,
}

/// Global archetype registry loaded from JSON
#[derive(Default)]
pub struct ArchetypeRegistry {
    pub definitions: HashMap<String, ArchetypeDefinition>,
}

impl ArchetypeRegistry {
    /// Load archetypes from JSON. A local file that fails to parse or omits
    /// an archetype is reported and the embedded copy is used instead, so the
    /// game never runs with a partial roster.
    pub fn load() -> Self {
        let embedded = macroquad_toolkit::include_json_str!("../../assets/tenant_archetypes.json");
        let loaded = load_json_file_with_fallback_sync::<ArchetypeData>(
            "assets/tenant_archetypes.json",
            embedded,
            JsonFallbackPolicy::ReadError,
        )
        .map_err(|error| error.to_string())
        .map(Self::from_data)
        .and_then(|registry| registry.validate().map(|()| registry));
        loaded.unwrap_or_else(|error| {
            eprintln!("Invalid tenant_archetypes.json ({error}); using the built-in roster");
            let data: ArchetypeData =
                serde_json::from_str(embedded).expect("the embedded tenant_archetypes.json parses");
            Self::from_data(data)
        })
    }

    fn from_data(data: ArchetypeData) -> Self {
        let definitions = data
            .archetypes
            .into_iter()
            .map(|archetype| (archetype.id.clone(), archetype))
            .collect();
        Self { definitions }
    }

    /// Every archetype the game can generate must be defined with names.
    pub fn validate(&self) -> Result<(), String> {
        for id in ["student", "professional", "artist", "family", "elderly"] {
            let definition = self
                .definitions
                .get(id)
                .ok_or_else(|| format!("missing archetype '{id}'"))?;
            if definition.name_pool.first_names.is_empty()
                || definition.name_pool.last_initials.is_empty()
            {
                return Err(format!("archetype '{id}' has an empty name pool"));
            }
        }
        Ok(())
    }

    /// Get archetype definition by ID
    pub fn get(&self, id: &str) -> Option<&ArchetypeDefinition> {
        self.definitions.get(id)
    }

    /// Convert preferences data to the runtime preferences struct
    pub fn to_preferences(prefs: &ArchetypePreferencesData) -> crate::tenant::ArchetypePreferences {
        crate::tenant::ArchetypePreferences {
            rent_sensitivity: prefs.rent_sensitivity,
            condition_sensitivity: prefs.condition_sensitivity,
            noise_sensitivity: prefs.noise_sensitivity,
            design_sensitivity: prefs.design_sensitivity,
            ideal_rent_max: prefs.ideal_rent_max,
            min_acceptable_condition: prefs.min_acceptable_condition,
            prefers_quiet: prefs.prefers_quiet,
            preferred_design: prefs
                .preferred_design
                .as_ref()
                .and_then(|s| Self::parse_design(s)),
            hates_design: prefs
                .hates_design
                .as_ref()
                .and_then(|s| Self::parse_design(s)),
        }
    }

    fn parse_design(design_str: &str) -> Option<DesignType> {
        match design_str.to_lowercase().as_str() {
            "bare" => Some(DesignType::Bare),
            "practical" => Some(DesignType::Practical),
            "cozy" => Some(DesignType::Cozy),
            _ => None,
        }
    }
}

/// Lazy-loaded global registry
use std::sync::OnceLock;
static ARCHETYPE_REGISTRY: OnceLock<ArchetypeRegistry> = OnceLock::new();

/// Get the global archetype registry
pub fn archetypes() -> &'static ArchetypeRegistry {
    ARCHETYPE_REGISTRY.get_or_init(ArchetypeRegistry::load)
}

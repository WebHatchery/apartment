//! The root game configuration and its loader. The individual tuning structs
//! live in `config/` grouped by the system they tune, and are re-exported here
//! so callers keep using `crate::data::config::<Thing>`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

mod apartment;
mod consequences;
mod difficulty;
mod presentation;
mod rules;
mod social;
mod tenants;
mod upgrades;

pub use apartment::ApartmentPropertiesConfig;
pub use consequences::{
    CriticalFailureConfig, GentrificationConfig, PortfolioConfig, RegulationsConfig,
};
pub use difficulty::DifficultyModifiers;
pub use presentation::{LayoutConfig, ThemeConfig, UiThresholdsConfig};
pub use rules::{
    ApplicationConfig, DecayConfig, EconomyConfig, HappinessConfig, OperatingCostsConfig,
    StartingConditions, ThresholdsConfig, WinConditions,
};
pub use social::{CohesionConfig, DilemmaConfig, RelationshipsConfig};
pub use tenants::{
    LeaseAcceptanceConfig, LeaseDefaultsConfig, LifeEventsConfig, MarketingConfig, MatchingConfig,
    StaffEffectsConfig, TenantRiskConfig, VettingConfig,
};
pub use upgrades::{UiConfig, UpgradeDefinition, UpgradeEffect, UpgradeRequirement, UpgradeTarget};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConfig {
    pub version: String,
    pub starting_conditions: StartingConditions,
    pub economy: EconomyConfig,
    pub decay: DecayConfig,
    pub happiness: HappinessConfig,
    pub win_conditions: WinConditions,
    pub applications: ApplicationConfig,
    pub ui: UiConfig,
    #[serde(default)]
    pub upgrades: HashMap<String, UpgradeDefinition>,
    #[serde(default)]
    pub matching: MatchingConfig,
    #[serde(default)]
    pub thresholds: ThresholdsConfig,
    #[serde(default)]
    pub operating_costs: OperatingCostsConfig,
    #[serde(default)]
    pub staff_effects: StaffEffectsConfig,
    #[serde(default)]
    pub tenant_risk: TenantRiskConfig,
    #[serde(default)]
    pub vetting: VettingConfig,
    #[serde(default)]
    pub marketing: MarketingConfig,
    #[serde(default)]
    pub relationships: RelationshipsConfig,
    #[serde(default)]
    pub cohesion: CohesionConfig,
    #[serde(default)]
    pub gentrification: GentrificationConfig,
    #[serde(default)]
    pub regulations: RegulationsConfig,
    #[serde(default)]
    pub life_events: LifeEventsConfig,
    #[serde(default)]
    pub critical_failures: CriticalFailureConfig,
    #[serde(default)]
    pub portfolio: PortfolioConfig,
    /// Per-difficulty rule modifiers, keyed by the building template's
    /// `difficulty` ("Easy"/"Medium"/"Hard"). Empty map → no adjustment.
    #[serde(default)]
    pub difficulty: HashMap<String, DifficultyModifiers>,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub ui_thresholds: UiThresholdsConfig,
    #[serde(default)]
    pub apartment: ApartmentPropertiesConfig,
}

/// Process-wide "currently loaded" config, consulted by call sites that would
/// otherwise need config threaded through many constructors/free functions
/// (presentation: theme colors, layout metrics, UI thresholds; and a few
/// enum-keyed scoring helpers on `Apartment`/`DesignType`/etc.). Refreshed
/// every time [`load_config`] runs. Falls back to `GameConfig::default()`
/// until the first load, so early/headless callers never see a panic.
static ACTIVE_CONFIG: OnceLock<RwLock<GameConfig>> = OnceLock::new();

fn active_cell() -> &'static RwLock<GameConfig> {
    ACTIVE_CONFIG.get_or_init(|| RwLock::new(GameConfig::default()))
}

fn set_active(config: &GameConfig) {
    *active_cell().write().unwrap() = config.clone();
}

/// The most recently loaded [`GameConfig`] (or the default, if none has
/// loaded yet). Cloned out so callers don't hold the lock.
pub fn active() -> GameConfig {
    active_cell().read().unwrap().clone()
}

pub fn load_config() -> GameConfig {
    // For WASM, embed configs at compile time
    #[cfg(target_arch = "wasm32")]
    let config_json = macroquad_toolkit::include_json_str!("../../assets/config.json");

    #[cfg(not(target_arch = "wasm32"))]
    let config_json = std::fs::read_to_string("assets/config.json").unwrap_or_else(|_| {
        macroquad_toolkit::include_json_str!("../../assets/config.json").to_string()
    });

    let mut config: GameConfig = serde_json::from_str(&config_json).unwrap_or_else(|e| {
        eprintln!("Failed to parse config.json: {}", e);
        GameConfig::default()
    });

    // Load upgrades from separate file
    #[cfg(target_arch = "wasm32")]
    let upgrades_json = macroquad_toolkit::include_json_str!("../../assets/upgrades.json");

    #[cfg(not(target_arch = "wasm32"))]
    let upgrades_json = std::fs::read_to_string("assets/upgrades.json").unwrap_or_else(|_| {
        macroquad_toolkit::include_json_str!("../../assets/upgrades.json").to_string()
    });

    if let Ok(upgrades) = serde_json::from_str::<HashMap<String, UpgradeDefinition>>(&upgrades_json)
    {
        config.upgrades = upgrades;
    }

    set_active(&config);
    config
}

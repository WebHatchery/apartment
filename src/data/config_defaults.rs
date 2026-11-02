//! `GameConfig::default()` is the shipped configuration itself: the embedded
//! `config.json` and `upgrades.json`. There is no second copy of the numbers
//! in Rust; per-block `Default` impls remain only as schema fallbacks for
//! blocks a JSON file omits.

use super::config::*;
use std::collections::HashMap;

impl Default for GameConfig {
    fn default() -> Self {
        let mut config: GameConfig = serde_json::from_str(macroquad_toolkit::include_json_str!(
            "../../assets/config.json"
        ))
        .expect("the embedded assets/config.json matches GameConfig");
        config.upgrades = serde_json::from_str::<HashMap<String, UpgradeDefinition>>(
            macroquad_toolkit::include_json_str!("../../assets/upgrades.json"),
        )
        .expect("the embedded assets/upgrades.json matches UpgradeDefinition");
        config
    }
}

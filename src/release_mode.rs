//! Compile-time separation between the full career and the itch.io demo.

use crate::data::config::GameConfig;

/// Three monthly turns represent roughly 90 in-world days.
pub const DEMO_DURATION_MONTHS: u32 = 3;
pub const DEMO_DURATION_DAYS: u32 = 90;

pub const fn is_demo() -> bool {
    cfg!(feature = "demo")
}

pub const fn save_namespace() -> &'static str {
    if is_demo() {
        "apartment_manager_demo"
    } else {
        "apartment_manager"
    }
}

pub fn apply_config(config: &mut GameConfig) {
    if is_demo() {
        config.win_conditions.game_duration_ticks = Some(DEMO_DURATION_MONTHS);
        config.win_conditions.min_ticks_for_victory = config
            .win_conditions
            .min_ticks_for_victory
            .min(DEMO_DURATION_MONTHS);
    }
}

#[cfg(test)]
mod tests;

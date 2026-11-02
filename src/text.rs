//! Player-facing text from `assets/text_strings.json`. Each template is
//! looked up by `section.key` and filled from named `{placeholders}`; the
//! file is checked on load so every key the game uses exists and carries the
//! placeholders the game fills in.

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::OnceLock;

use macroquad_toolkit::data_loader::{load_json_file_with_fallback_sync, JsonFallbackPolicy};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct TextStrings {
    events: HashMap<String, String>,
    status: HashMap<String, String>,
    month: HashMap<String, String>,
    inspector: HashMap<String, String>,
}

/// Every key the game looks up, with the placeholders its template must use.
pub const REQUIRED: &[(&str, &[&str])] = &[
    ("events.rent_paid", &["amount", "tenant"]),
    ("events.rent_missed", &["tenant"]),
    ("events.tenant_unhappy", &["tenant", "happiness"]),
    ("events.new_application", &["tenant", "archetype", "unit"]),
    ("events.tenant_moved_in", &["tenant", "unit"]),
    ("events.noise_complaint", &["tenant"]),
    ("events.tenant_damage", &["tenant", "unit", "damage"]),
    ("events.condition_complaint", &["tenant", "unit"]),
    ("events.poor_condition", &["unit", "condition"]),
    ("events.critical_condition", &["unit", "condition"]),
    ("events.hallway_deteriorating", &["condition"]),
    ("events.upgrade_completed", &["description", "cost"]),
    (
        "events.insufficient_funds",
        &["action", "needed", "available"],
    ),
    (
        "events.month_end",
        &["month", "income", "expenses", "balance"],
    ),
    ("events.victory", &[]),
    ("events.bankruptcy", &[]),
    ("events.all_left", &[]),
    ("events.heatwave", &["months"]),
    ("events.pipe_burst", &["unit", "damage"]),
    ("events.gentrification", &["effect", "months"]),
    ("events.inspection_failed", &["result", "fine"]),
    ("events.inspection_passed", &["result"]),
    ("events.boiler_failure", &["cost"]),
    ("events.structural_issue", &["description", "cost"]),
    ("events.staff_action", &["role", "action"]),
    ("status.critical_units", &["count"]),
    ("status.overdrawn", &[]),
    ("status.requests_waiting", &["count"]),
    ("status.unlisted_vacancies", &["count"]),
    ("status.needs_repair", &["count"]),
    ("status.activity", &[]),
    ("status.view_history", &[]),
    ("status.hide_history", &[]),
    ("status.history_title", &["month"]),
    ("status.earlier_activity", &[]),
    ("month.title", &["month"]),
    ("month.rent", &[]),
    ("month.other_income", &[]),
    ("month.running_costs", &[]),
    ("month.repairs", &[]),
    ("month.upgrades", &[]),
    ("month.net", &[]),
    ("month.balance", &[]),
    ("month.rent_paid", &["count"]),
    ("month.missed_rent", &["names"]),
    ("month.applications", &["count"]),
    ("month.more_lines", &["count"]),
    ("month.overflow", &[]),
    ("month.continue", &[]),
    ("inspector.condition", &[]),
    ("inspector.rent", &[]),
    ("inspector.asking_rent", &[]),
    ("inspector.rent_with_limit", &["rent", "limit"]),
    ("inspector.rent_monthly", &["rent"]),
    ("inspector.happiness", &[]),
    ("inspector.condition_critical", &["condition"]),
    ("inspector.condition_poor", &["condition"]),
    ("inspector.condition_worn", &["condition"]),
    ("inspector.condition_good", &["condition"]),
    ("inspector.vacant_listed", &[]),
    ("inspector.vacant_unlisted", &[]),
    ("inspector.priced", &["action", "cost"]),
    ("inspector.priced_short", &["action", "cost", "shortfall"]),
    ("inspector.lobby_title", &[]),
    ("inspector.building_appeal", &[]),
    ("inspector.lobby_note", &[]),
];

impl TextStrings {
    /// The template for `section.key`, if the file defines it.
    pub fn template(&self, key: &str) -> Option<&str> {
        let (section, name) = key.split_once('.')?;
        let table = match section {
            "events" => &self.events,
            "status" => &self.status,
            "month" => &self.month,
            "inspector" => &self.inspector,
            _ => return None,
        };
        table.get(name).map(String::as_str)
    }

    /// Every required key is present, non-empty and uses its placeholders.
    pub fn validate(&self) -> Result<(), String> {
        for (key, placeholders) in REQUIRED {
            let template = self
                .template(key)
                .filter(|template| !template.trim().is_empty())
                .ok_or_else(|| format!("missing text '{key}'"))?;
            if let Some(missing) = placeholders
                .iter()
                .find(|name| !template.contains(&format!("{{{name}}}")))
            {
                return Err(format!("text '{key}' does not use {{{missing}}}"));
            }
        }
        Ok(())
    }
}

/// Parse and validate a text table (used on load and by the data tests).
pub fn parse(json: &str) -> Result<TextStrings, String> {
    let strings: TextStrings = serde_json::from_str(json).map_err(|error| error.to_string())?;
    strings.validate()?;
    Ok(strings)
}

const EMBEDDED: &str = macroquad_toolkit::include_json_str!("../assets/text_strings.json");

fn strings() -> &'static TextStrings {
    static STRINGS: OnceLock<TextStrings> = OnceLock::new();
    STRINGS.get_or_init(|| {
        let loaded = load_json_file_with_fallback_sync::<TextStrings>(
            "assets/text_strings.json",
            EMBEDDED,
            JsonFallbackPolicy::ReadError,
        )
        .map_err(|error| error.to_string())
        .and_then(|strings| strings.validate().map(|()| strings));
        loaded.unwrap_or_else(|error| {
            eprintln!("Invalid text_strings.json ({error}); using the built-in text");
            parse(EMBEDDED).unwrap_or_default()
        })
    })
}

/// The text for `key` with each `{name}` replaced by its value.
pub fn tr(key: &str, args: &[(&str, &dyn Display)]) -> String {
    let Some(template) = strings().template(key) else {
        return format!("[{key}]");
    };
    let mut text = template.to_string();
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), &value.to_string());
    }
    text
}

/// The fixed text for `key` (no placeholders).
pub fn t(key: &str) -> String {
    tr(key, &[])
}

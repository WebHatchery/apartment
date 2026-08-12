use macroquad_toolkit::rng;
// Game notification system for relationship changes and contextual hints
// Uses pop-up modals similar to the tutorial system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A notification message to display to the player
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameNotification {
    /// Icon to display (emoji)
    pub icon: String,
    /// Main message text
    pub message: String,
    /// Optional additional context
    pub description: Option<String>,
    /// Category for styling
    pub category: NotificationCategory,
}

/// Categories of notifications affect visual styling
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NotificationCategory {
    /// Positive event (green tint)
    Positive,
    /// Warning or conflict (yellow/orange tint)  
    Warning,
    /// Neutral information (blue tint)
    Info,
    /// Hint or tip (gray tint)
    Hint,
}

impl GameNotification {
    pub fn positive(icon: &str, message: &str) -> Self {
        Self {
            icon: icon.to_string(),
            message: message.to_string(),
            description: None,
            category: NotificationCategory::Positive,
        }
    }

    pub fn warning(icon: &str, message: &str) -> Self {
        Self {
            icon: icon.to_string(),
            message: message.to_string(),
            description: None,
            category: NotificationCategory::Warning,
        }
    }

    pub fn hint(message: &str) -> Self {
        Self {
            icon: "i".to_string(),
            message: message.to_string(),
            description: None,
            category: NotificationCategory::Hint,
        }
    }
}

/// Relationship change event returned from tick
#[derive(Clone, Debug)]
pub enum RelationshipChange {
    /// New relationship formed
    NewRelationship {
        tenant_a_name: String,
        tenant_b_name: String,
        relationship_type: String,
        is_positive: bool,
    },
}

impl RelationshipChange {
    /// Convert to a game notification
    pub fn to_notification(&self, config: &HintsConfig) -> GameNotification {
        match self {
            RelationshipChange::NewRelationship {
                tenant_a_name,
                tenant_b_name,
                relationship_type,
                is_positive,
            } => {
                let key = if *is_positive {
                    "new_friendly"
                } else {
                    "new_hostile"
                };
                if let Some(template) = config.relationship_notifications.get(key) {
                    let message = template
                        .template
                        .replace("{tenant_a}", tenant_a_name)
                        .replace("{tenant_b}", tenant_b_name);

                    let mut notif = if *is_positive {
                        GameNotification::positive(&template.icon, &message)
                    } else {
                        GameNotification::warning(&template.icon, &message)
                    };

                    notif.description = Some(template.description.clone());
                    notif
                } else {
                    let msg = format!(
                        "{} and {} formed a {} relationship",
                        tenant_a_name, tenant_b_name, relationship_type
                    );
                    if *is_positive {
                        GameNotification::positive("+", &msg)
                    } else {
                        GameNotification::warning("!", &msg)
                    }
                }
            }
        }
    }
}

/// Configuration loaded from hints.json
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HintsConfig {
    pub context_hints: HashMap<String, ContextHint>,
    pub relationship_notifications: HashMap<String, RelationshipNotificationTemplate>,
    pub relationship_icons: HashMap<String, String>,
    pub thresholds: HintThresholds,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextHint {
    pub priority: i32,
    pub messages: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipNotificationTemplate {
    pub icon: String,
    pub template: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HintThresholds {
    pub low_condition: i32,
    pub high_vacancy_percent: i32,
    pub unhappy_happiness: i32,
    pub low_funds: i32,
    pub high_funds: i32,
    pub hint_cooldown_months: u32,
}

impl Default for HintsConfig {
    fn default() -> Self {
        let mut context_hints = HashMap::new();
        context_hints.insert(
            "full_occupancy".to_string(),
            ContextHint {
                priority: 1,
                messages: vec!["All units occupied! Focus on keeping tenants happy.".to_string()],
            },
        );

        let mut relationship_notifications = HashMap::new();
        relationship_notifications.insert(
            "new_friendly".to_string(),
            RelationshipNotificationTemplate {
                icon: "+".to_string(),
                template: "{tenant_a} and {tenant_b} have become friends!".to_string(),
                description: "Friendly neighbors boost each other's happiness.".to_string(),
            },
        );
        relationship_notifications.insert(
            "new_hostile".to_string(),
            RelationshipNotificationTemplate {
                icon: "!".to_string(),
                template: "Conflict brewing between {tenant_a} and {tenant_b}!".to_string(),
                description: "Hostile relationships reduce happiness.".to_string(),
            },
        );

        let mut relationship_icons = HashMap::new();
        relationship_icons.insert("friendly".to_string(), "+".to_string());
        relationship_icons.insert("neutral".to_string(), "=".to_string());
        relationship_icons.insert("hostile".to_string(), "!".to_string());
        relationship_icons.insert("romantic".to_string(), "+".to_string());
        relationship_icons.insert("family".to_string(), "+".to_string());

        Self {
            context_hints,
            relationship_notifications,
            relationship_icons,
            thresholds: HintThresholds::default(),
        }
    }
}

impl Default for HintThresholds {
    fn default() -> Self {
        Self {
            low_condition: 50,
            high_vacancy_percent: 50,
            unhappy_happiness: 30,
            low_funds: 500,
            high_funds: 10000,
            hint_cooldown_months: 3,
        }
    }
}

/// Load hints config from JSON file
pub fn load_hints_config() -> HintsConfig {
    #[cfg(target_arch = "wasm32")]
    let json = macroquad_toolkit::include_json_str!("../../assets/hints.json");

    #[cfg(not(target_arch = "wasm32"))]
    let json = std::fs::read_to_string("assets/hints.json").unwrap_or_else(|_| {
        macroquad_toolkit::include_json_str!("../../assets/hints.json").to_string()
    });

    serde_json::from_str(&json).unwrap_or_else(|e| {
        eprintln!("Failed to parse hints.json: {}", e);
        HintsConfig::default()
    })
}

/// Manages pending game notifications
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NotificationManager {
    pub pending: Vec<GameNotification>,
    pub last_hint_month: u32,
    #[serde(skip)]
    pub hints_config: Option<HintsConfig>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            last_hint_month: 0,
            hints_config: Some(load_hints_config()),
        }
    }

    /// Add relationship changes as notifications
    pub fn add_relationship_changes(&mut self, changes: Vec<RelationshipChange>) {
        let config = self.hints_config.clone().unwrap_or_default();
        for change in changes {
            self.pending.push(change.to_notification(&config));
        }
    }

    /// Check game state and add contextual hints if appropriate
    pub fn check_context_hints(
        &mut self,
        current_month: u32,
        vacancy_count: usize,
        total_units: usize,
        avg_condition: i32,
        funds: i32,
        any_unhappy: bool,
    ) {
        let config = self.hints_config.clone().unwrap_or_default();

        // Respect cooldown
        if current_month < self.last_hint_month + config.thresholds.hint_cooldown_months {
            return;
        }

        // Find the highest priority applicable hint
        let mut best_hint: Option<(&str, i32)> = None;

        // Full occupancy check
        if vacancy_count == 0 {
            if let Some(hint) = config.context_hints.get("full_occupancy") {
                if best_hint.is_none_or(|(_, p)| hint.priority < p) {
                    best_hint = Some(("full_occupancy", hint.priority));
                }
            }
        }

        // Low condition check
        if avg_condition < config.thresholds.low_condition {
            if let Some(hint) = config.context_hints.get("low_condition") {
                if best_hint.is_none_or(|(_, p)| hint.priority < p) {
                    best_hint = Some(("low_condition", hint.priority));
                }
            }
        }

        // High vacancy check
        let vacancy_percent = vacancy_count
            .saturating_mul(100)
            .checked_div(total_units)
            .unwrap_or(0);
        if vacancy_percent >= config.thresholds.high_vacancy_percent as usize {
            if let Some(hint) = config.context_hints.get("high_vacancy") {
                if best_hint.is_none_or(|(_, p)| hint.priority < p) {
                    best_hint = Some(("high_vacancy", hint.priority));
                }
            }
        }

        // Unhappy tenant check
        if any_unhappy {
            if let Some(hint) = config.context_hints.get("tenant_unhappy") {
                if best_hint.is_none_or(|(_, p)| hint.priority < p) {
                    best_hint = Some(("tenant_unhappy", hint.priority));
                }
            }
        }

        // Funds check
        if funds < config.thresholds.low_funds {
            if let Some(hint) = config.context_hints.get("low_funds") {
                if best_hint.is_none_or(|(_, p)| hint.priority < p) {
                    best_hint = Some(("low_funds", hint.priority));
                }
            }
        } else if funds > config.thresholds.high_funds {
            if let Some(hint) = config.context_hints.get("high_funds") {
                if best_hint.is_none_or(|(_, p)| hint.priority < p) {
                    best_hint = Some(("high_funds", hint.priority));
                }
            }
        }

        // Generate the hint if we found one
        if let Some((hint_key, _)) = best_hint {
            if let Some(hint) = config.context_hints.get(hint_key) {
                if !hint.messages.is_empty() {
                    let idx = rng::gen_range(0, hint.messages.len());
                    self.pending
                        .push(GameNotification::hint(&hint.messages[idx]));
                    self.last_hint_month = current_month;
                }
            }
        }
    }

    /// Get the next pending notification (if any)
    pub fn pop(&mut self) -> Option<GameNotification> {
        if self.pending.is_empty() {
            None
        } else {
            Some(self.pending.remove(0))
        }
    }

    /// Check if there are pending notifications
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
}

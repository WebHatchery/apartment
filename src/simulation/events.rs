//! Simulation events, their severity and the event log.

use super::GameOutcome;
use serde::{Deserialize, Serialize};

/// Significant events that happen during simulation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameEvent {
    // Generic
    Notification {
        message: String,
        level: NotificationLevel,
    },

    // Economy
    RentPaid {
        tenant_name: String,
        amount: i32,
    },
    RentMissed {
        tenant_name: String,
        amount: i32,
    },
    UpgradeCompleted {
        description: String,
        cost: i32,
    },
    InsufficientFunds {
        action: String,
        needed: i32,
        available: i32,
    },

    // Tenant events
    TenantUnhappy {
        tenant_name: String,
        happiness: i32,
    },
    TenantMovedOut {
        message: String,
    },
    NewApplication {
        tenant_name: String,
        archetype: String,
        apartment_unit: String,
    },
    TenantMovedIn {
        tenant_name: String,
        apartment_unit: String,
    },

    // Complaint events
    NoiseComplaint {
        tenant_name: String,
    },
    TenantDamage {
        tenant_name: String,
        apartment_unit: String,
        damage: i32,
    },
    ConditionComplaint {
        tenant_name: String,
        apartment_unit: String,
    },

    // Building events
    PoorCondition {
        apartment_unit: String,
        condition: i32,
    },
    CriticalCondition {
        apartment_unit: String,
        condition: i32,
    },
    HallwayDeteriorating {
        condition: i32,
    },

    // Time events
    MonthEnd {
        tick: u32,
        income: i32,
        expenses: i32,
        balance: i32,
    },

    // Game state events
    GameEnded {
        outcome: GameOutcome,
    },

    // Random Events
    Heatwave {
        tick_duration: u32,
    },
    PipeBurst {
        apartment_unit: String,
        damage: i32,
    },
    Gentrification {
        tick_duration: u32,
        effect_desc: String,
    },
    Inspection {
        result: String,
        fine: i32,
    },

    // Critical Failures
    BoilerFailure {
        cost: i32,
    },
    StructuralIssue {
        cost: i32,
        description: String,
    },

    // Staff Events
    StaffAction {
        role: String,
        action: String,
    },
}

impl GameEvent {
    /// Get a short message for display
    /// The event as the player reads it, from `text_strings.json`.
    pub fn message(&self) -> String {
        use crate::text::{t, tr};
        match self {
            GameEvent::RentPaid {
                tenant_name,
                amount,
            } => tr(
                "events.rent_paid",
                &[("amount", amount), ("tenant", tenant_name)],
            ),
            GameEvent::RentMissed { tenant_name, .. } => {
                tr("events.rent_missed", &[("tenant", tenant_name)])
            }
            GameEvent::TenantUnhappy {
                tenant_name,
                happiness,
            } => tr(
                "events.tenant_unhappy",
                &[("tenant", tenant_name), ("happiness", happiness)],
            ),
            GameEvent::TenantMovedOut { message } => message.clone(),
            GameEvent::NewApplication {
                tenant_name,
                archetype,
                apartment_unit,
            } => tr(
                "events.new_application",
                &[
                    ("tenant", tenant_name),
                    ("archetype", archetype),
                    ("unit", apartment_unit),
                ],
            ),
            GameEvent::TenantMovedIn {
                tenant_name,
                apartment_unit,
            } => tr(
                "events.tenant_moved_in",
                &[("tenant", tenant_name), ("unit", apartment_unit)],
            ),
            GameEvent::NoiseComplaint { tenant_name } => {
                tr("events.noise_complaint", &[("tenant", tenant_name)])
            }
            GameEvent::TenantDamage {
                tenant_name,
                apartment_unit,
                damage,
            } => tr(
                "events.tenant_damage",
                &[
                    ("tenant", tenant_name),
                    ("unit", apartment_unit),
                    ("damage", damage),
                ],
            ),
            GameEvent::ConditionComplaint {
                tenant_name,
                apartment_unit,
            } => tr(
                "events.condition_complaint",
                &[("tenant", tenant_name), ("unit", apartment_unit)],
            ),
            GameEvent::PoorCondition {
                apartment_unit,
                condition,
            } => tr(
                "events.poor_condition",
                &[("unit", apartment_unit), ("condition", condition)],
            ),
            GameEvent::CriticalCondition {
                apartment_unit,
                condition,
            } => tr(
                "events.critical_condition",
                &[("unit", apartment_unit), ("condition", condition)],
            ),
            GameEvent::HallwayDeteriorating { condition } => {
                tr("events.hallway_deteriorating", &[("condition", condition)])
            }
            GameEvent::UpgradeCompleted { description, cost } => tr(
                "events.upgrade_completed",
                &[("description", description), ("cost", cost)],
            ),
            GameEvent::InsufficientFunds {
                action,
                needed,
                available,
            } => tr(
                "events.insufficient_funds",
                &[
                    ("action", action),
                    ("needed", needed),
                    ("available", available),
                ],
            ),
            GameEvent::MonthEnd {
                tick,
                income,
                expenses,
                balance,
            } => tr(
                "events.month_end",
                &[
                    ("month", tick),
                    ("income", income),
                    ("expenses", expenses),
                    ("balance", balance),
                ],
            ),
            GameEvent::GameEnded { outcome } => match outcome {
                GameOutcome::Victory { .. } => t("events.victory"),
                GameOutcome::Bankruptcy { .. } => t("events.bankruptcy"),
                GameOutcome::AllTenantsLeft => t("events.all_left"),
            },
            GameEvent::Heatwave { tick_duration } => {
                tr("events.heatwave", &[("months", tick_duration)])
            }
            GameEvent::PipeBurst {
                apartment_unit,
                damage,
            } => tr(
                "events.pipe_burst",
                &[("unit", apartment_unit), ("damage", damage)],
            ),
            GameEvent::Gentrification {
                tick_duration,
                effect_desc,
            } => tr(
                "events.gentrification",
                &[("effect", effect_desc), ("months", tick_duration)],
            ),
            GameEvent::Inspection { result, fine } if *fine > 0 => tr(
                "events.inspection_failed",
                &[("result", result), ("fine", fine)],
            ),
            GameEvent::Inspection { result, .. } => {
                tr("events.inspection_passed", &[("result", result)])
            }
            GameEvent::BoilerFailure { cost } => tr("events.boiler_failure", &[("cost", cost)]),
            GameEvent::StructuralIssue { cost, description } => tr(
                "events.structural_issue",
                &[("description", description), ("cost", cost)],
            ),
            GameEvent::StaffAction { role, action } => {
                tr("events.staff_action", &[("role", role), ("action", action)])
            }
            GameEvent::Notification { message, .. } => message.clone(),
        }
    }

    /// Get event severity for UI coloring
    pub fn severity(&self) -> EventSeverity {
        match self {
            GameEvent::Notification { level, .. } => match level {
                NotificationLevel::Info => EventSeverity::Info,
                NotificationLevel::Warning => EventSeverity::Warning,
                NotificationLevel::Critical => EventSeverity::Negative,
            },
            GameEvent::RentPaid { .. } => EventSeverity::Positive,
            GameEvent::TenantMovedIn { .. } => EventSeverity::Positive,
            GameEvent::UpgradeCompleted { .. } => EventSeverity::Positive,
            GameEvent::NewApplication { .. } => EventSeverity::Info,
            GameEvent::MonthEnd { .. } => EventSeverity::Info,
            GameEvent::RentMissed { .. } => EventSeverity::Warning,
            GameEvent::TenantUnhappy { .. } => EventSeverity::Warning,
            GameEvent::NoiseComplaint { .. } => EventSeverity::Warning,
            GameEvent::TenantDamage { .. } => EventSeverity::Negative,
            GameEvent::ConditionComplaint { .. } => EventSeverity::Warning,
            GameEvent::PoorCondition { .. } => EventSeverity::Warning,
            GameEvent::HallwayDeteriorating { .. } => EventSeverity::Warning,
            GameEvent::InsufficientFunds { .. } => EventSeverity::Negative,
            GameEvent::TenantMovedOut { .. } => EventSeverity::Negative,
            GameEvent::CriticalCondition { .. } => EventSeverity::Negative,
            GameEvent::GameEnded { outcome } => match outcome {
                GameOutcome::Victory { .. } => EventSeverity::Positive,
                _ => EventSeverity::Negative,
            },
            GameEvent::Heatwave { .. } => EventSeverity::Warning,
            GameEvent::PipeBurst { .. } => EventSeverity::Negative,
            GameEvent::Gentrification { .. } => EventSeverity::Positive,
            GameEvent::Inspection { fine, .. } => {
                if *fine > 0 {
                    EventSeverity::Negative
                } else {
                    EventSeverity::Positive
                }
            }
            GameEvent::BoilerFailure { .. } => EventSeverity::Negative,
            GameEvent::StructuralIssue { .. } => EventSeverity::Negative,
            GameEvent::StaffAction { .. } => EventSeverity::Info,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventSeverity {
    Positive,
    Info,
    Warning,
    Negative,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActiveWorldEventKind {
    Heatwave,
    Gentrification,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveWorldEvent {
    pub kind: ActiveWorldEventKind,
    pub remaining_ticks: u32,
}

impl ActiveWorldEvent {
    pub fn new(kind: ActiveWorldEventKind, remaining_ticks: u32) -> Self {
        Self {
            kind,
            remaining_ticks,
        }
    }

    pub fn tick(&mut self) {
        self.remaining_ticks = self.remaining_ticks.saturating_sub(1);
    }
}

/// Log of all game events
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventLog {
    events: Vec<(u32, GameEvent)>, // (tick, event)
}

impl EventLog {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn log(&mut self, event: GameEvent, tick: u32) {
        self.events.push((tick, event));
    }

    /// Everything logged for one month, oldest first.
    pub fn events_at(&self, tick: u32) -> impl Iterator<Item = &GameEvent> {
        self.events
            .iter()
            .filter(move |(at, _)| *at == tick)
            .map(|(_, event)| event)
    }

    pub fn recent_events(&self, count: usize) -> Vec<&GameEvent> {
        self.events
            .iter()
            .rev()
            .take(count)
            .map(|(_, e)| e)
            .collect()
    }
}

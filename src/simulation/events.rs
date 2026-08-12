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
    pub fn message(&self) -> String {
        match self {
            GameEvent::RentPaid {
                tenant_name,
                amount,
            } => {
                format!("Received ${} rent from {}", amount, tenant_name)
            }
            GameEvent::RentMissed { tenant_name, .. } => {
                format!("{} missed rent payment", tenant_name)
            }
            GameEvent::TenantUnhappy {
                tenant_name,
                happiness,
            } => {
                format!("{} is unhappy ({}%)", tenant_name, happiness)
            }
            GameEvent::TenantMovedOut { message } => message.clone(),
            GameEvent::NewApplication {
                tenant_name,
                archetype,
                apartment_unit,
            } => {
                format!(
                    "{} ({}) applied for Unit {}",
                    tenant_name, archetype, apartment_unit
                )
            }
            GameEvent::TenantMovedIn {
                tenant_name,
                apartment_unit,
            } => {
                format!("{} moved into Unit {}", tenant_name, apartment_unit)
            }
            GameEvent::NoiseComplaint { tenant_name } => {
                format!("Noise complaint from {}", tenant_name)
            }
            GameEvent::TenantDamage {
                tenant_name,
                apartment_unit,
                damage,
            } => {
                format!(
                    "{} damaged Unit {} (-{} condition)",
                    tenant_name, apartment_unit, damage
                )
            }
            GameEvent::ConditionComplaint {
                tenant_name,
                apartment_unit,
            } => {
                format!(
                    "{} complained about Unit {} condition",
                    tenant_name, apartment_unit
                )
            }
            GameEvent::PoorCondition {
                apartment_unit,
                condition,
            } => {
                format!("Unit {} in poor condition ({}%)", apartment_unit, condition)
            }
            GameEvent::CriticalCondition {
                apartment_unit,
                condition,
            } => {
                format!("Unit {} CRITICAL ({}%)", apartment_unit, condition)
            }
            GameEvent::HallwayDeteriorating { condition } => {
                format!("Hallway deteriorating ({}%)", condition)
            }
            GameEvent::UpgradeCompleted { description, cost } => {
                format!("{} (-${})", description, cost)
            }
            GameEvent::InsufficientFunds {
                action,
                needed,
                available,
            } => {
                format!(
                    "Cannot afford {} (need ${}, have ${})",
                    action, needed, available
                )
            }
            GameEvent::MonthEnd {
                tick,
                income,
                expenses,
                balance,
            } => {
                format!(
                    "Month {} ended: +${} -${} = ${}",
                    tick, income, expenses, balance
                )
            }
            GameEvent::GameEnded { outcome } => match outcome {
                GameOutcome::Victory { .. } => "Career complete!".to_string(),
                GameOutcome::Bankruptcy { .. } => "Bankruptcy declared!".to_string(),
                GameOutcome::AllTenantsLeft => "All tenants have left!".to_string(),
            },
            GameEvent::Heatwave { tick_duration } => {
                format!("Heatwave! (Duration: {} months)", tick_duration)
            }
            GameEvent::PipeBurst {
                apartment_unit,
                damage,
            } => {
                format!(
                    "Pipe burst in Unit {}! (-{} condition)",
                    apartment_unit, damage
                )
            }
            GameEvent::Gentrification {
                tick_duration,
                effect_desc,
            } => {
                format!(
                    "Neighborhood improving! {} (Duration: {})",
                    effect_desc, tick_duration
                )
            }
            GameEvent::Inspection { result, fine } => {
                if *fine > 0 {
                    format!("Inspection failed: {} (Fine: -${})", result, fine)
                } else {
                    format!("Inspection passed: {}", result)
                }
            }
            GameEvent::BoilerFailure { cost } => {
                format!("Boiler failure! (-${} repair)", cost)
            }
            GameEvent::StructuralIssue { cost, description } => {
                format!("Structural issue: {} (-${})", description, cost)
            }
            GameEvent::StaffAction { role, action } => {
                format!("{}: {}", role, action)
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

    pub fn recent_events(&self, count: usize) -> Vec<&GameEvent> {
        self.events
            .iter()
            .rev()
            .take(count)
            .map(|(_, e)| e)
            .collect()
    }
}

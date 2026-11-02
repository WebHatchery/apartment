//! # Simulation Module
//!
//! The heartbeat of the game. Handles time and state progression:
//! - `Tick`: The central game loop processing logic.
//! - `Decay`: Entropy and maintenance mechanics.
//! - `Win Conditions`: Victory and failure state checks.
//! - `Events`: Random events and lucky/unlucky occurrences.

mod decay;
mod events;
mod random_events;
mod tick;
mod win_condition;

pub use tick::{advance_tick, TickResult};
// pub use decay::apply_decay;
pub use events::{
    ActiveWorldEvent, ActiveWorldEventKind, EventLog, EventSeverity, GameEvent, NotificationLevel,
};
pub use win_condition::GameOutcome;

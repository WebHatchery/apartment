//! # State Module
//!
//! Manages the global application state machine:
//! - `GameState`: The top-level enum for game modes (Menu, Gameplay, Results).
//! - Transitions between these high-level states.
//! - Specific state structs for each mode.

mod gameplay;
mod gameplay_actions; // UI action dispatch and city action handling
mod gameplay_awards; // Tax breaks, annual awards, tenant council
mod gameplay_effects; // Narrative event effect application
mod gameplay_input; // Input dispatch, pause, and modal ownership
mod gameplay_inspections; // Building inspections and regulatory fines
mod gameplay_life_events; // Emergent tenant life events
mod gameplay_narrative_turn; // Monthly narrative, mail, dialogue, requests
mod gameplay_neighborhood; // Neighborhood reputation and market conditions
mod gameplay_ownership; // Safe condo sale and buyback transactions
mod gameplay_policies; // Rent, marketing, and operating-policy changes
mod gameplay_turn; // Monthly turn advancement
mod gameplay_views; // Drawing functions (draw, draw_building_mode, etc.)
mod menu;
pub mod mission_system;
pub mod tutorial_system; // Tutorial logic // Mission logic

pub use gameplay::GameplayState;
pub(crate) use gameplay::ViewMode;
pub use menu::MenuState;

pub enum GameState {
    Menu(MenuState),
    Gameplay(GameplayState),
}

pub enum StateTransition {
    ToMenu,
    ToGameplay(GameplayState),
}

//! Second Story as a library: every game module lives here so the binary is
//! only the window and frame loop, and integration tests in `tests/` reach
//! the game through its public seams.

pub mod assets;
pub mod building;
pub mod city;
pub mod consequences;
pub mod data;
pub mod economy;
pub mod game;
pub mod narrative;
pub mod release_mode;
pub mod save;
pub mod screen_layout;
pub mod simulation;
pub mod state;
pub mod tenant;
pub mod text;
pub mod ui;

// Headless balance-simulation harness (test-only).
#[cfg(test)]
mod sim_harness;

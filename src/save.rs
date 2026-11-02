//! Saving and loading games, settings and cross-run player progress.

pub mod manager;

pub use manager::{
    has_save_game, load_game, load_player_progress, load_settings, save_game, save_settings,
    update_player_progress, PlayerProgress,
};

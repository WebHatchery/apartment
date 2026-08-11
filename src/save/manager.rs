use crate::state::GameplayState;
use macroquad_toolkit::persistence::{json_key_exists, load_json_key, save_json_key};
use serde::{Deserialize, Serialize};

const GAME_NAME: &str = "apartment_manager";
const SAVE_FILE_NAME: &str = "savegame.json";
const PROGRESS_FILE_NAME: &str = "player_progress.json";

/// Player progress - persists across game sessions
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlayerProgress {
    pub unlocked_buildings: Vec<String>,
    pub completed_buildings: Vec<String>,
}

impl PlayerProgress {
    pub fn new() -> Self {
        Self {
            unlocked_buildings: vec!["mvp_default".to_string()], // First building unlocked by default
            completed_buildings: Vec::new(),
        }
    }

    pub fn is_unlocked(&self, building_id: &str) -> bool {
        self.unlocked_buildings.contains(&building_id.to_string())
    }

    pub fn unlock_building(&mut self, building_id: &str) {
        if building_id.is_empty() {
            return;
        }
        if !self.unlocked_buildings.contains(&building_id.to_string()) {
            self.unlocked_buildings.push(building_id.to_string());
        }
    }

    pub fn mark_completed(&mut self, building_id: &str) {
        // Guard against an empty id (a pre-backfill save could leave
        // current_building_id blank), which previously polluted progress with an
        // empty "" entry that renders as a phantom completed building.
        if building_id.is_empty() {
            return;
        }
        if !self.completed_buildings.contains(&building_id.to_string()) {
            self.completed_buildings.push(building_id.to_string());
        }
    }

    /// Drop any empty/blank ids that older saves may have accumulated.
    fn sanitize(&mut self) {
        self.unlocked_buildings.retain(|id| !id.is_empty());
        self.completed_buildings.retain(|id| !id.is_empty());
    }
}

/// Save the current game state to disk
pub fn save_game(state: &GameplayState) -> std::io::Result<()> {
    save_json_key(GAME_NAME, SAVE_FILE_NAME, state).map_err(std::io::Error::other)
}

/// Load the game state from disk
pub fn load_game() -> std::io::Result<GameplayState> {
    let mut state: GameplayState =
        load_json_key(GAME_NAME, SAVE_FILE_NAME).map_err(std::io::Error::other)?;

    // Restore non-serialized fields and repair older save shapes.
    state.post_load();

    Ok(state)
}

/// Check if a save file exists
pub fn has_save_game() -> bool {
    json_key_exists(GAME_NAME, SAVE_FILE_NAME)
}

/// Load player progress (persistent unlock state)
pub fn load_player_progress() -> PlayerProgress {
    let mut progress: PlayerProgress =
        load_json_key(GAME_NAME, PROGRESS_FILE_NAME).unwrap_or_else(|_| PlayerProgress::new());
    progress.sanitize();
    progress
}

/// Save player progress (persistent unlock state)
pub fn save_player_progress(progress: &PlayerProgress) -> std::io::Result<()> {
    save_json_key(GAME_NAME, PROGRESS_FILE_NAME, progress).map_err(std::io::Error::other)
}

#[cfg(test)]
mod tests;

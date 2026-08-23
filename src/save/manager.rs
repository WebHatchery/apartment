use crate::state::GameplayState;
use macroquad_toolkit::persistence::{json_key_exists, load_json_key, save_json_key};
use serde::{Deserialize, Serialize};

const GAME_NAME: &str = "apartment_manager";
const SAVE_FILE_NAME: &str = "savegame.json";
const PROGRESS_FILE_NAME: &str = "player_progress.json";
const CURRENT_SAVE_FORMAT_VERSION: u32 = 1;

/// The persistent save envelope. Keeping the version outside gameplay state
/// lets the loader distinguish a future file format from ordinary game data.
#[derive(Deserialize)]
struct VersionedSave {
    state: GameplayState,
}

#[derive(Serialize)]
struct CurrentSave<'a> {
    save_format_version: u32,
    state: &'a GameplayState,
}

fn current_save(state: &GameplayState) -> CurrentSave<'_> {
    CurrentSave {
        save_format_version: CURRENT_SAVE_FORMAT_VERSION,
        state,
    }
}

/// Decode either the current envelope or the unwrapped gameplay state used by
/// pre-versioned releases. An envelope is identified before deserialization so
/// an unsupported newer save can never fall through to the legacy path.
fn decode_save(value: serde_json::Value) -> std::io::Result<GameplayState> {
    if value.get("save_format_version").is_none() && value.get("state").is_none() {
        return serde_json::from_value(value).map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Legacy save could not be decoded: {error}"),
            )
        });
    }

    let version = value
        .get("save_format_version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Versioned save has no numeric format version",
            )
        })?;

    if version > u64::from(CURRENT_SAVE_FORMAT_VERSION) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!(
                "Save format {version} is newer than supported format {CURRENT_SAVE_FORMAT_VERSION}"
            ),
        ));
    }
    if version < u64::from(CURRENT_SAVE_FORMAT_VERSION) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Save format {version} cannot be migrated to format {CURRENT_SAVE_FORMAT_VERSION}"
            ),
        ));
    }

    let save: VersionedSave = serde_json::from_value(value).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Versioned save could not be decoded: {error}"),
        )
    })?;

    Ok(save.state)
}

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
    save_json_key(GAME_NAME, SAVE_FILE_NAME, &current_save(state)).map_err(std::io::Error::other)
}

/// Load the game state from disk
pub fn load_game() -> std::io::Result<GameplayState> {
    let value: serde_json::Value =
        load_json_key(GAME_NAME, SAVE_FILE_NAME).map_err(std::io::Error::other)?;
    let mut state = decode_save(value)?;

    // Restore non-serialized fields and repair legacy state shapes.
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

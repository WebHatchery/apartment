//! Top-level game: owns the active screen state, configuration and assets.

use crate::assets::AssetManager;
use crate::data::config::{load_config, GameConfig};
use crate::state::{GameState, MenuState, StateTransition};

mod capture;
mod capture_seeds;

pub struct Game {
    pub state: GameState,
    pub config: GameConfig,
    pub assets: AssetManager,
    capture_resident_gallery: bool,
}

impl Game {
    pub async fn new() -> Self {
        let mut assets = AssetManager::new();
        assets.load_assets().await;

        let mut config = load_config();
        crate::release_mode::apply_config(&mut config);

        Self {
            state: GameState::Menu(Box::new(MenuState::new())),
            config,
            assets,
            capture_resident_gallery: false,
        }
    }

    pub fn update(&mut self) {
        if self.capture_resident_gallery {
            return;
        }
        let transition = match &mut self.state {
            GameState::Menu(s) => s.update(&self.config),
            GameState::Gameplay(s) => s.update(&self.assets),
        };

        if let Some(t) = transition {
            self.transition(t);
        }
    }

    pub fn draw(&mut self) {
        if self.capture_resident_gallery {
            crate::ui::draw_resident_sprite_gallery(&self.assets);
            return;
        }
        match &mut self.state {
            GameState::Menu(s) => s.draw(&self.assets),
            GameState::Gameplay(s) => s.draw(&self.assets),
        }
    }

    fn transition(&mut self, transition: StateTransition) {
        self.state = match transition {
            StateTransition::ToMenu => GameState::Menu(Box::new(MenuState::new())),
            StateTransition::ToGameplay(mut s) => {
                s.apply_settings(&crate::save::load_settings());
                GameState::Gameplay(s)
            }
        };
    }
}

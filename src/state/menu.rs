//! Title screen: a home menu (Continue / New Game / Settings / Exit Game)
//! with the building picker and the settings page as sub-screens.

use super::{GameplayState, StateTransition};
use crate::assets::AssetManager;
use crate::data::templates::{load_templates, BuildingTemplate};
use crate::save::{has_save_game, load_game, load_player_progress, PlayerProgress};
use crate::ui::theme::{color, Tone};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width};

mod building_select;

use super::settings_screen::SettingsScreen;

const HOME_BUTTON_W: f32 = 280.0;
const HOME_BUTTON_H: f32 = 48.0;
const HOME_BUTTON_GAP: f32 = 12.0;

/// Which page of the title screen is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuScreen {
    Home,
    NewGame,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HomeOption {
    Continue,
    NewGame,
    Settings,
    /// Native only — a browser tab has nothing to exit, and
    /// `std::process::exit` is unsupported on wasm.
    #[cfg(not(target_arch = "wasm32"))]
    Exit,
}

#[cfg(not(target_arch = "wasm32"))]
const HOME_OPTIONS: &[HomeOption] = &[
    HomeOption::Continue,
    HomeOption::NewGame,
    HomeOption::Settings,
    HomeOption::Exit,
];
#[cfg(target_arch = "wasm32")]
const HOME_OPTIONS: &[HomeOption] = &[
    HomeOption::Continue,
    HomeOption::NewGame,
    HomeOption::Settings,
];

impl HomeOption {
    fn label(self) -> &'static str {
        match self {
            HomeOption::Continue => "Continue",
            HomeOption::NewGame => "New Game",
            HomeOption::Settings => "Settings",
            #[cfg(not(target_arch = "wasm32"))]
            HomeOption::Exit => "Exit Game",
        }
    }

    fn tone(self) -> Tone {
        match self {
            HomeOption::Continue => Tone::Positive,
            HomeOption::NewGame => Tone::Primary,
            HomeOption::Settings => Tone::Secondary,
            #[cfg(not(target_arch = "wasm32"))]
            HomeOption::Exit => Tone::Danger,
        }
    }
}

/// Logo size and top edge `(w, h, y)` for the current screen height.
/// Shared by the logo itself and everything laid out beneath it.
fn logo_layout() -> (f32, f32, f32) {
    if screen_height() < 520.0 {
        (280.0, 126.0, 4.0)
    } else if screen_height() < 660.0 {
        (300.0, 135.0, 8.0)
    } else {
        (400.0, 180.0, 40.0)
    }
}

/// Top of the first content block below the logo (home buttons, settings).
fn content_top() -> f32 {
    let (_, logo_h, logo_y) = logo_layout();
    let gap = if screen_height() < 660.0 { 20.0 } else { 44.0 };
    logo_y + logo_h + gap
}

fn home_button_rect(index: usize) -> Rect {
    Rect::new(
        screen_width() / 2.0 - HOME_BUTTON_W / 2.0,
        content_top() + index as f32 * (HOME_BUTTON_H + HOME_BUTTON_GAP),
        HOME_BUTTON_W,
        HOME_BUTTON_H,
    )
}

pub struct MenuState {
    screen: MenuScreen,
    has_save: bool,
    progress: PlayerProgress,
    templates: Vec<BuildingTemplate>,
    load_error: Option<String>,
    settings: SettingsScreen,
}

impl MenuState {
    pub fn new() -> Self {
        let templates = load_templates().map(|t| t.templates).unwrap_or_default();

        // Unreadable progress still lets the player start the first building;
        // the file is left untouched and the problem is shown on the menu.
        let (progress, load_error) = match load_player_progress() {
            Ok(progress) => (progress, None),
            Err(error) => (PlayerProgress::new(), Some(error)),
        };

        Self {
            screen: MenuScreen::Home,
            has_save: has_save_game(),
            progress,
            templates,
            load_error,
            settings: SettingsScreen::new(content_top),
        }
    }

    /// Open straight onto a sub-screen (screenshot harness).
    pub fn for_capture(scene: &str) -> Self {
        let screen = if scene.starts_with("menu_new_game") {
            MenuScreen::NewGame
        } else if scene.starts_with("menu_settings") {
            MenuScreen::Settings
        } else {
            MenuScreen::Home
        };
        Self {
            screen,
            ..Self::new()
        }
    }

    pub fn update(&mut self, config: &crate::data::config::GameConfig) -> Option<StateTransition> {
        if self.screen != MenuScreen::Home && is_key_pressed(KeyCode::Escape) {
            self.screen = MenuScreen::Home;
            return None;
        }

        let pointer = vec2(mouse_position().0, mouse_position().1);
        // Release, like every themed button: dragging off cancels a choice.
        let clicked = is_mouse_button_released(MouseButton::Left);

        match self.screen {
            MenuScreen::Home => {
                if clicked {
                    return self.update_home(pointer);
                }
            }
            MenuScreen::NewGame => {
                if !clicked {
                    return None;
                }
                if building_select::back_rect().contains(pointer) {
                    self.screen = MenuScreen::Home;
                    return None;
                }
                if let Some(template) =
                    building_select::clicked_template(&self.templates, &self.progress, pointer)
                {
                    let state = GameplayState::new_with_template(config.clone(), template.clone());
                    return Some(StateTransition::ToGameplay(Box::new(state)));
                }
            }
            MenuScreen::Settings => {
                if clicked && self.settings.update(pointer) {
                    self.screen = MenuScreen::Home;
                }
            }
        }
        None
    }

    fn update_home(&mut self, pointer: Vec2) -> Option<StateTransition> {
        let option = HOME_OPTIONS
            .iter()
            .enumerate()
            .find(|(i, _)| home_button_rect(*i).contains(pointer))
            .map(|(_, option)| *option)?;

        match option {
            HomeOption::Continue => {
                if !self.has_save {
                    return None;
                }
                match load_game() {
                    Ok(state) => return Some(StateTransition::ToGameplay(Box::new(state))),
                    Err(_) => {
                        self.load_error = Some(
                            "That saved game could not be opened. Choose New Game to start again."
                                .to_string(),
                        );
                    }
                }
            }
            HomeOption::NewGame => self.screen = MenuScreen::NewGame,
            HomeOption::Settings => {
                self.settings = SettingsScreen::new(content_top);
                self.screen = MenuScreen::Settings;
            }
            #[cfg(not(target_arch = "wasm32"))]
            HomeOption::Exit => std::process::exit(0),
        }
        None
    }

    pub fn draw(&self, assets: &AssetManager) {
        draw_background(assets);

        match self.screen {
            MenuScreen::Home => self.draw_home(),
            MenuScreen::NewGame => building_select::draw(
                &self.templates,
                &self.progress,
                self.has_save,
                logo_layout().0,
                assets,
            ),
            MenuScreen::Settings => self.settings.draw(),
        }

        // Textured branding is intentionally the final layer. Toolkit card
        // surfaces may batch after earlier textures even when their bounds do
        // not overlap, so this preserves the logo across every card layout.
        draw_title_logo(assets);
    }

    fn draw_home(&self) {
        if crate::release_mode::is_demo() {
            let label = format!("{}-DAY DEMO", crate::release_mode::DEMO_DURATION_DAYS);
            let size = 16.0;
            let width = measure_ui_text(&label, None, size as u16, 1.0).width;
            draw_ui_text(
                &label,
                screen_width() / 2.0 - width / 2.0,
                content_top() - 6.0,
                size,
                color::TEXT_BRIGHT(),
            );
        }

        for (i, option) in HOME_OPTIONS.iter().enumerate() {
            let enabled = *option != HomeOption::Continue || self.has_save;
            crate::ui::widgets::button_at(
                home_button_rect(i),
                option.label(),
                enabled,
                option.tone(),
            );
        }

        let last = home_button_rect(HOME_OPTIONS.len() - 1);
        let mut note_y = last.bottom() + 16.0;
        if !self.has_save {
            let note = "No saved game yet — start one with New Game.";
            let size = 13.0;
            let width = measure_ui_text(note, None, size as u16, 1.0).width;
            draw_ui_text(
                note,
                screen_width() / 2.0 - width / 2.0,
                note_y + 10.0,
                size,
                color::TEXT_DIM(),
            );
            note_y += 24.0;
        }

        if let Some(error) = &self.load_error {
            let error_w = (screen_width() - 32.0).min(620.0);
            let error_x = (screen_width() - error_w) / 2.0;
            crate::ui::widgets::draw_card(Rect::new(error_x, note_y, error_w, 34.0), true);
            draw_ui_text(
                &truncate_text_to_width(error, error_w - 24.0, 14.0),
                error_x + 12.0,
                note_y + 22.0,
                14.0,
                color::NEGATIVE(),
            );
        }
    }
}

fn draw_background(assets: &AssetManager) {
    if let Some(bg) = assets.get_texture("title_background") {
        draw_texture_ex(
            bg,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(screen_width(), screen_height())),
                ..Default::default()
            },
        );
    } else {
        clear_background(Color::from_rgba(25, 25, 30, 255));
    }
}

fn draw_title_logo(assets: &AssetManager) {
    if let Some(logo) = assets.get_texture("title_logo") {
        let (logo_w, logo_h, logo_y) = logo_layout();
        draw_texture_ex(
            logo,
            screen_width() / 2.0 - logo_w / 2.0,
            logo_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(logo_w, logo_h)),
                ..Default::default()
            },
        );
    } else {
        let title = "SECOND STORY";
        let title_size = 60.0;
        let title_width = measure_ui_text(title, None, title_size as u16, 1.0).width;
        draw_ui_text(
            title,
            screen_width() / 2.0 - title_width / 2.0,
            120.0,
            title_size,
            WHITE,
        );
    }
}

#[cfg(test)]
mod tests;

impl Default for MenuState {
    fn default() -> Self {
        Self::new()
    }
}

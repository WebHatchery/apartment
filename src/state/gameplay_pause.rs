//! Pause menu (Escape or the Menu button): resume, save, load, settings, and
//! exit. Opening it is allowed even under blocking overlays.

use crate::save::{has_save_game, load_game, save_game};
use crate::ui::theme::Tone;
use crate::ui::{colors, widgets::button_at};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

use super::gameplay::GameplayState;
use super::settings_screen::{centered_top, SettingsScreen};
use super::StateTransition;

const PANEL_W: f32 = 300.0;
const BUTTON_W: f32 = 220.0;
const BUTTON_H: f32 = 40.0;
const BUTTON_STEP: f32 = 48.0;
const TITLE_H: f32 = 64.0;
const FOOTER_H: f32 = 34.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PauseOption {
    Resume,
    Save,
    Load,
    Settings,
    Tips,
    ExitToTitle,
    /// Native only — a browser tab has nothing to exit, and
    /// `std::process::exit` is unsupported on wasm.
    #[cfg(not(target_arch = "wasm32"))]
    ExitGame,
}

#[cfg(not(target_arch = "wasm32"))]
const PAUSE_OPTIONS: &[PauseOption] = &[
    PauseOption::Resume,
    PauseOption::Save,
    PauseOption::Load,
    PauseOption::Settings,
    PauseOption::Tips,
    PauseOption::ExitToTitle,
    PauseOption::ExitGame,
];
#[cfg(target_arch = "wasm32")]
const PAUSE_OPTIONS: &[PauseOption] = &[
    PauseOption::Resume,
    PauseOption::Save,
    PauseOption::Load,
    PauseOption::Settings,
    PauseOption::Tips,
    PauseOption::ExitToTitle,
];

impl PauseOption {
    fn label(self) -> &'static str {
        match self {
            PauseOption::Resume => "Resume",
            PauseOption::Save => "Save Game",
            PauseOption::Load => "Load Saved Game",
            PauseOption::Settings => "Settings",
            PauseOption::Tips => "Tips",
            PauseOption::ExitToTitle => "Exit to Title",
            #[cfg(not(target_arch = "wasm32"))]
            PauseOption::ExitGame => "Exit Game",
        }
    }

    fn tone(self) -> Tone {
        match self {
            PauseOption::Resume => Tone::Positive,
            PauseOption::Save | PauseOption::Load => Tone::Primary,
            PauseOption::Settings | PauseOption::Tips | PauseOption::ExitToTitle => Tone::Secondary,
            #[cfg(not(target_arch = "wasm32"))]
            PauseOption::ExitGame => Tone::Danger,
        }
    }
}

fn panel_rect() -> Rect {
    let h = TITLE_H + PAUSE_OPTIONS.len() as f32 * BUTTON_STEP + FOOTER_H;
    Rect::new(
        (screen_width() - PANEL_W) / 2.0,
        ((screen_height() - h) / 2.0).max(8.0),
        PANEL_W,
        h,
    )
}

fn button_rect(panel: Rect, index: usize) -> Rect {
    Rect::new(
        panel.x + (panel.w - BUTTON_W) / 2.0,
        panel.y + TITLE_H + index as f32 * BUTTON_STEP,
        BUTTON_W,
        BUTTON_H,
    )
}

impl GameplayState {
    /// Per-frame pause logic that may leave gameplay (loading a save) or
    /// route clicks to the settings page.
    pub(super) fn update_pause_menu(&mut self) -> Option<StateTransition> {
        if let Some(settings) = self.pause_settings.as_mut() {
            let pointer = vec2(mouse_position().0, mouse_position().1);
            if is_mouse_button_pressed(MouseButton::Left) && settings.update(pointer) {
                self.close_pause_settings();
            }
        }

        if std::mem::take(&mut self.pending_load_game) {
            match load_game() {
                Ok(state) => return Some(StateTransition::ToGameplay(Box::new(state))),
                Err(error) => {
                    eprintln!("Failed to load game: {error}");
                    self.floating_texts.spawn(
                        "That saved game could not be opened",
                        vec2(screen_width() / 2.0, screen_height() / 2.0),
                        colors::NEGATIVE(),
                    );
                }
            }
        }
        None
    }

    /// Leave the settings page, adopting whatever the player changed there.
    pub(super) fn close_pause_settings(&mut self) {
        if let Some(settings) = self.pause_settings.take() {
            self.apply_settings(settings.settings());
        }
    }

    /// Draw the pause menu overlay (called from draw()).
    pub(super) fn draw_pause_menu_overlay(&mut self) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.7),
        );

        if let Some(settings) = &self.pause_settings {
            settings.draw();
            return;
        }

        let panel = panel_rect();
        draw_rectangle(panel.x, panel.y, panel.w, panel.h, colors::SURFACE());
        draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 2.0, colors::ACCENT());

        let title = "PAUSED";
        let title_width = measure_ui_text(title, None, 32, 1.0).width;
        draw_ui_text(
            title,
            panel.x + (panel.w - title_width) / 2.0,
            panel.y + 44.0,
            32.0,
            colors::TEXT_BRIGHT(),
        );

        let has_save = has_save_game();
        for (i, option) in PAUSE_OPTIONS.iter().enumerate() {
            let enabled = *option != PauseOption::Load || has_save;
            let label = match option {
                PauseOption::Tips if self.tutorial.active && self.coach_hidden => "Show tips",
                PauseOption::Tips if self.tutorial.active => "Hide tips",
                PauseOption::Tips => "How to play",
                _ => option.label(),
            };
            if button_at(button_rect(panel, i), label, enabled, option.tone()) && enabled {
                self.choose_pause_option(*option);
            }
        }

        // Touch-first continuation hint; keyboard shortcuts remain supplemental.
        let hint = "Tap RESUME or press Esc to continue";
        let hint_w = measure_ui_text(hint, None, 14, 1.0).width;
        draw_ui_text(
            hint,
            panel.x + (panel.w - hint_w) / 2.0,
            panel.bottom() - 14.0,
            14.0,
            colors::TEXT_DIM(),
        );
    }

    fn choose_pause_option(&mut self, option: PauseOption) {
        match option {
            PauseOption::Resume => self.show_pause_menu = false,
            PauseOption::Save => {
                let (message, message_color) = if save_game(self).is_ok() {
                    ("Game saved", colors::POSITIVE())
                } else {
                    ("Save failed — please try again", colors::NEGATIVE())
                };
                self.floating_texts.spawn(
                    message,
                    vec2(screen_width() / 2.0, screen_height() / 2.0),
                    message_color,
                );
                self.show_pause_menu = false;
            }
            PauseOption::Load => self.pending_load_game = true,
            PauseOption::Settings => {
                self.pause_settings = Some(SettingsScreen::new(centered_top));
            }
            // Guidance only: tutorial progress and saves are untouched.
            PauseOption::Tips => {
                if self.tutorial.active {
                    self.coach_hidden = !self.coach_hidden;
                } else {
                    self.help_open = true;
                }
                self.show_pause_menu = false;
            }
            PauseOption::ExitToTitle => self.pending_quit_to_menu = true,
            #[cfg(not(target_arch = "wasm32"))]
            PauseOption::ExitGame => std::process::exit(0),
        }
    }
}

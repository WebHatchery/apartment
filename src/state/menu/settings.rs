//! Settings page: player preferences stored with the toolkit's shared
//! `GameSettings`. Every toggle applies and saves immediately.

use crate::ui::theme::{color, space, Tone};
use macroquad::prelude::*;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

const PANEL_MAX_W: f32 = 520.0;
const PANEL_H: f32 = 256.0;
const HEADER_H: f32 = 38.0;
const ROW_H: f32 = 64.0;
const TOGGLE_W: f32 = 96.0;
const TOGGLE_H: f32 = 40.0;
const BACK_W: f32 = 160.0;
const BACK_H: f32 = 44.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Setting {
    Fullscreen,
    Autosave,
}

const SETTINGS: [Setting; 2] = [Setting::Fullscreen, Setting::Autosave];

impl Setting {
    fn label(self) -> &'static str {
        match self {
            Setting::Fullscreen => "Fullscreen",
            Setting::Autosave => "Monthly autosave",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Setting::Fullscreen => "Fill the whole screen with the game.",
            Setting::Autosave => "Save automatically when each month closes.",
        }
    }

    fn value(self, settings: &GameSettings) -> bool {
        match self {
            Setting::Fullscreen => settings.fullscreen,
            Setting::Autosave => settings.autosave_enabled,
        }
    }
}

fn panel_rect() -> Rect {
    let w = (screen_width() - 32.0).min(PANEL_MAX_W);
    let y = super::content_top()
        .min(screen_height() - PANEL_H - 12.0)
        .max(12.0);
    Rect::new((screen_width() - w) / 2.0, y, w, PANEL_H)
}

fn row_rect(index: usize) -> Rect {
    let panel = panel_rect();
    Rect::new(
        panel.x + space::PAD,
        panel.y + HEADER_H + space::SM + index as f32 * ROW_H,
        panel.w - space::PAD * 2.0,
        ROW_H,
    )
}

fn toggle_rect(index: usize) -> Rect {
    let row = row_rect(index);
    Rect::new(
        row.right() - TOGGLE_W,
        row.y + (ROW_H - TOGGLE_H) / 2.0,
        TOGGLE_W,
        TOGGLE_H,
    )
}

fn back_rect() -> Rect {
    let panel = panel_rect();
    Rect::new(
        panel.x + (panel.w - BACK_W) / 2.0,
        panel.bottom() - space::PAD - BACK_H,
        BACK_W,
        BACK_H,
    )
}

pub(super) struct SettingsScreen {
    settings: GameSettings,
    save_error: Option<String>,
}

impl SettingsScreen {
    pub(super) fn new() -> Self {
        Self {
            settings: crate::save::load_settings(),
            save_error: None,
        }
    }

    /// Handle a click. Returns true when the player asked to go back.
    pub(super) fn update(&mut self, pointer: Vec2) -> bool {
        if back_rect().contains(pointer) {
            return true;
        }
        if let Some(setting) = SETTINGS
            .iter()
            .enumerate()
            .find(|(i, _)| toggle_rect(*i).contains(pointer))
            .map(|(_, setting)| *setting)
        {
            self.toggle(setting);
        }
        false
    }

    fn toggle(&mut self, setting: Setting) {
        match setting {
            Setting::Fullscreen => self.settings.toggle_fullscreen(),
            Setting::Autosave => {
                self.settings.autosave_enabled = !self.settings.autosave_enabled;
            }
        }
        self.save_error = crate::save::save_settings(&self.settings)
            .err()
            .map(|error| format!("Settings could not be saved: {error}"));
    }

    pub(super) fn draw(&self) {
        crate::ui::widgets::draw_panel(panel_rect(), "Settings");

        for (i, setting) in SETTINGS.iter().enumerate() {
            let row = row_rect(i);
            let text_w = row.w - TOGGLE_W - space::MD;
            draw_ui_text(
                &truncate_text_to_width(setting.label(), text_w, 18.0),
                row.x,
                row.y + 26.0,
                18.0,
                color::TEXT_BRIGHT(),
            );
            draw_ui_text(
                &truncate_text_to_width(setting.description(), text_w, 13.0),
                row.x,
                row.y + 46.0,
                13.0,
                color::TEXT_DIM(),
            );

            let on = setting.value(&self.settings);
            crate::ui::widgets::button_at(
                toggle_rect(i),
                if on { "On" } else { "Off" },
                true,
                if on { Tone::Positive } else { Tone::Secondary },
            );
        }

        if let Some(error) = &self.save_error {
            let panel = panel_rect();
            draw_ui_text(
                &truncate_text_to_width(error, panel.w - space::PAD * 2.0, 13.0),
                panel.x + space::PAD,
                back_rect().y - space::SM,
                13.0,
                color::NEGATIVE(),
            );
        }

        crate::ui::widgets::button_at(back_rect(), "Back", true, Tone::Secondary);
    }
}

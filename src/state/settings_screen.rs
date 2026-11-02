//! Settings page shared by the title screen and the pause menu: player
//! preferences stored with the toolkit's shared `GameSettings`. Every toggle
//! applies and saves immediately.

use crate::ui::theme::{color, space, Tone};
use macroquad::prelude::*;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

const PANEL_MAX_W: f32 = 520.0;
const PANEL_H: f32 = 320.0;
const HEADER_H: f32 = 38.0;
const ROW_H: f32 = 64.0;
const TOGGLE_W: f32 = 96.0;
const TOGGLE_H: f32 = 44.0;
const BACK_W: f32 = 160.0;
const BACK_H: f32 = 44.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Setting {
    Fullscreen,
    Autosave,
    ReducedMotion,
}

const SETTINGS: [Setting; 3] = [
    Setting::Fullscreen,
    Setting::Autosave,
    Setting::ReducedMotion,
];

impl Setting {
    fn label(self) -> &'static str {
        match self {
            Setting::Fullscreen => "Fullscreen",
            Setting::Autosave => "Monthly autosave",
            Setting::ReducedMotion => "Reduced motion",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Setting::Fullscreen => "Fill the whole screen with the game.",
            Setting::Autosave => "Save automatically when each month closes.",
            Setting::ReducedMotion => "Hold residents, steam and floating text still.",
        }
    }

    fn value(self, settings: &GameSettings) -> bool {
        match self {
            Setting::Fullscreen => settings.fullscreen,
            Setting::Autosave => settings.autosave_enabled,
            Setting::ReducedMotion => settings.reduced_motion,
        }
    }
}

/// Vertically centered placement, for use over gameplay.
pub(crate) fn centered_top() -> f32 {
    (screen_height() - PANEL_H) / 2.0
}

fn row_rect(panel: Rect, index: usize) -> Rect {
    Rect::new(
        panel.x + space::PAD,
        panel.y + HEADER_H + space::SM + index as f32 * ROW_H,
        panel.w - space::PAD * 2.0,
        ROW_H,
    )
}

fn toggle_rect(panel: Rect, index: usize) -> Rect {
    let row = row_rect(panel, index);
    Rect::new(
        row.right() - TOGGLE_W,
        row.y + (ROW_H - TOGGLE_H) / 2.0,
        TOGGLE_W,
        TOGGLE_H,
    )
}

fn back_rect(panel: Rect) -> Rect {
    Rect::new(
        panel.x + (panel.w - BACK_W) / 2.0,
        panel.bottom() - space::PAD - BACK_H,
        BACK_W,
        BACK_H,
    )
}

pub(crate) struct SettingsScreen {
    settings: GameSettings,
    save_error: Option<String>,
    /// Top edge of the panel for the current screen size; the caller decides
    /// where it sits (below the title logo, or centered over gameplay).
    top: fn() -> f32,
}

impl SettingsScreen {
    pub(crate) fn new(top: fn() -> f32) -> Self {
        Self {
            settings: crate::save::load_settings(),
            save_error: None,
            top,
        }
    }

    pub(crate) fn settings(&self) -> &GameSettings {
        &self.settings
    }

    fn panel_rect(&self) -> Rect {
        let w = (screen_width() - 32.0).min(PANEL_MAX_W);
        let y = (self.top)().min(screen_height() - PANEL_H - 12.0).max(12.0);
        Rect::new((screen_width() - w) / 2.0, y, w, PANEL_H)
    }

    /// Handle a click. Returns true when the player asked to go back.
    pub(crate) fn update(&mut self, pointer: Vec2) -> bool {
        let panel = self.panel_rect();
        if back_rect(panel).contains(pointer) {
            return true;
        }
        if let Some(setting) = SETTINGS
            .iter()
            .enumerate()
            .find(|(i, _)| toggle_rect(panel, *i).contains(pointer))
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
            Setting::ReducedMotion => {
                self.settings.reduced_motion = !self.settings.reduced_motion;
                self.settings.apply_effects();
            }
        }
        self.save_error = crate::save::save_settings(&self.settings)
            .err()
            .map(|error| format!("Settings could not be saved: {error}"));
    }

    pub(crate) fn draw(&self) {
        let panel = self.panel_rect();
        crate::ui::widgets::draw_panel(panel, "Settings");

        for (i, setting) in SETTINGS.iter().enumerate() {
            let row = row_rect(panel, i);
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
                toggle_rect(panel, i),
                if on { "On" } else { "Off" },
                true,
                if on { Tone::Positive } else { Tone::Secondary },
            );
        }

        if let Some(error) = &self.save_error {
            draw_ui_text(
                &truncate_text_to_width(error, panel.w - space::PAD * 2.0, 13.0),
                panel.x + space::PAD,
                back_rect(panel).y - space::SM,
                13.0,
                color::NEGATIVE(),
            );
        }

        crate::ui::widgets::button_at(back_rect(panel), "Back", true, Tone::Secondary);
    }
}

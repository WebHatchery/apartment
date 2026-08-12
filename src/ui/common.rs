use macroquad::prelude::*;

pub use macroquad_toolkit::input::{is_hovered, was_clicked};

/// Color palette — single source of truth lives in [`crate::ui::theme::color`].
/// Re-exported here so existing `colors::NAME()` references keep working while
/// the whole UI picks up the restyle.
pub mod colors {
    pub use crate::ui::theme::color::*;
}

use crate::tenant::TenantArchetype;

/// Get color for tenant archetype
pub fn archetype_color(archetype: &TenantArchetype) -> macroquad::prelude::Color {
    match archetype {
        TenantArchetype::Student => colors::STUDENT(),
        TenantArchetype::Professional => colors::PROFESSIONAL(),
        TenantArchetype::Artist => colors::ARTIST(),
        TenantArchetype::Family => colors::FAMILY(),
        TenantArchetype::Elderly => colors::ELDERLY(),
    }
}

/// Layout metrics, read from the active config's `layout` block. Functions
/// keep the SCREAMING_CASE names call sites already use (they used to be
/// consts); `non_snake_case` is allowed module-wide for that reason.
#[allow(non_snake_case)]
pub mod layout {
    fn layout() -> crate::data::config::LayoutConfig {
        crate::data::config::active().layout
    }

    pub fn HEADER_HEIGHT() -> f32 {
        // 64 px status bar + 56 px primary navigation. Keeping this stable
        // makes every workspace share one predictable management shell.
        120.0
    }
    pub fn FOOTER_HEIGHT() -> f32 {
        // The old 100 px permanent event log starved short viewports. Events
        // now live in a collapsible activity drawer with a compact handle.
        44.0
    }
    pub fn PANEL_SPLIT() -> f32 {
        match macroquad::prelude::screen_width() {
            width if width >= 1180.0 => 0.70,
            width if width >= 960.0 => 0.68,
            _ => 0.62,
        }
    }
    pub fn PADDING() -> f32 {
        if macroquad::prelude::screen_width() < 900.0 {
            8.0
        } else {
            layout().padding
        }
    }
    pub fn UNIT_WIDTH() -> f32 {
        layout().unit_width
    }
    pub fn UNIT_HEIGHT() -> f32 {
        layout().unit_height
    }
    pub fn UNIT_GAP() -> f32 {
        layout().unit_gap
    }
    pub fn FLOOR_HEIGHT() -> f32 {
        layout().floor_height
    }
}

/// Draw a standard secondary button (returns true on click). Restyled to the
/// theme tone system so every call site picks up the new look at once.
pub fn button(x: f32, y: f32, w: f32, h: f32, text: &str, enabled: bool) -> bool {
    crate::ui::widgets::button_at(
        Rect::new(x, y, w, h),
        text,
        enabled,
        crate::ui::theme::Tone::Secondary,
    )
}

/// Draw a titled panel using the theme's card + header style.
pub fn panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    crate::ui::widgets::draw_panel(Rect::new(x, y, w, h), title);
}

/// Get color for condition value, using the active config's `ui_thresholds`.
pub fn condition_color(condition: i32) -> Color {
    let t = crate::data::config::active().ui_thresholds;
    if condition >= t.condition_good {
        colors::POSITIVE()
    } else if condition >= t.condition_fair {
        colors::ACCENT()
    } else if condition >= t.condition_poor {
        colors::WARNING()
    } else {
        colors::NEGATIVE()
    }
}

/// Get color for happiness value, using the active config's `ui_thresholds`.
/// Reuses the happy/neutral/unhappy breakpoints (there's no distinct
/// "ecstatic" tone — `happiness_ecstatic` is reserved for a future label-only
/// tier and doesn't affect color).
pub fn happiness_color(happiness: i32) -> Color {
    let t = crate::data::config::active().ui_thresholds;
    if happiness >= t.happiness_happy {
        colors::POSITIVE()
    } else if happiness >= t.happiness_neutral {
        colors::ACCENT()
    } else if happiness >= t.happiness_unhappy {
        colors::WARNING()
    } else {
        colors::NEGATIVE()
    }
}

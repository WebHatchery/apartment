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

/// Runtime adapters for responsive geometry and the configured unit gap.
/// Uppercase wrappers retain existing call-site conventions.
#[allow(non_snake_case)]
pub mod layout {
    fn layout() -> crate::data::config::LayoutConfig {
        crate::data::config::active().layout
    }

    pub fn HEADER_HEIGHT() -> f32 {
        apartment::screen_layout::header_height(macroquad::prelude::screen_width())
    }
    pub fn FOOTER_HEIGHT() -> f32 {
        44.0
    }
    pub fn building(selected: bool) -> apartment::screen_layout::BuildingLayout {
        apartment::screen_layout::BuildingLayout::new(
            macroquad::prelude::screen_width(),
            macroquad::prelude::screen_height(),
            selected,
        )
    }
    pub fn context_panel() -> macroquad::prelude::Rect {
        building(true).panel
    }
    pub fn UNIT_GAP() -> f32 {
        layout().unit_gap
    }
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

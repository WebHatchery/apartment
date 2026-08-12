//! # Theme
//!
//! Single source of truth for the game's visual design tokens: palette, type
//! scale, spacing scale, and reusable style builders. Panels compose these
//! instead of hand-picking colors and font sizes, which keeps the look
//! consistent and eliminates the ad-hoc magic numbers that caused overlaps.
//!
//! The palette is loaded from the `theme` block in `assets/config.json`
//! (`ThemeConfig`, via `crate::data::config::active()`); the values below are
//! only the compile-time fallback used before a config has loaded (or if one
//! fails to parse), so they're kept in sync with `ThemeConfig::default()`.

use macroquad::prelude::Color;
use macroquad_toolkit::ui::{ButtonStyle, SurfaceStyle};

fn c(v: [f32; 4]) -> Color {
    Color::new(v[0], v[1], v[2], v[3])
}

/// Bolder restyle palette: deep desaturated indigo base with a warm amber
/// primary, plus clear semantic accents. Each function reads the active
/// config's `theme` block.
///
/// Functions keep the SCREAMING_CASE names call sites already use (they used
/// to be consts); `non_snake_case` is allowed module-wide for that reason.
#[allow(non_snake_case)]
pub mod color {
    use super::c;
    use macroquad::prelude::Color;

    fn theme() -> crate::data::config::ThemeConfig {
        crate::data::config::active().theme
    }

    // Base surfaces (dark -> light)
    pub fn BACKGROUND() -> Color {
        c(theme().background)
    }
    pub fn SURFACE() -> Color {
        c(theme().surface)
    }
    pub fn SURFACE_ALT() -> Color {
        c(theme().surface_alt)
    }
    pub fn SURFACE_HEADER() -> Color {
        c(theme().surface_header)
    }

    // Hairlines / outlines
    pub fn BORDER() -> Color {
        c(theme().border)
    }
    pub fn BORDER_STRONG() -> Color {
        c(theme().border_strong)
    }

    // Text
    pub fn TEXT() -> Color {
        c(theme().text)
    }
    pub fn TEXT_BRIGHT() -> Color {
        c(theme().text_bright)
    }
    pub fn TEXT_DIM() -> Color {
        c(theme().text_dim)
    }

    // Primary (warm amber) — the headline accent for this restyle.
    pub fn PRIMARY() -> Color {
        c(theme().primary)
    }
    pub fn PRIMARY_HOVER() -> Color {
        c(theme().primary_hover)
    }
    pub fn PRIMARY_PRESSED() -> Color {
        c(theme().primary_pressed)
    }

    // Secondary accent (cool teal) — selection glow / info highlights.
    pub fn ACCENT() -> Color {
        c(theme().accent)
    }

    // Semantic status colors
    pub fn POSITIVE() -> Color {
        c(theme().positive)
    }
    pub fn WARNING() -> Color {
        c(theme().warning)
    }
    pub fn NEGATIVE() -> Color {
        c(theme().negative)
    }

    // Apartment-unit states
    pub fn VACANT() -> Color {
        c(theme().vacant)
    }
    pub fn OCCUPIED() -> Color {
        c(theme().occupied)
    }
    pub fn HOVERED() -> Color {
        c(theme().hovered)
    }

    // Tenant archetype accents
    pub fn STUDENT() -> Color {
        c(theme().student)
    }
    pub fn PROFESSIONAL() -> Color {
        c(theme().professional)
    }
    pub fn ARTIST() -> Color {
        c(theme().artist)
    }
    pub fn FAMILY() -> Color {
        c(theme().family)
    }
    pub fn ELDERLY() -> Color {
        c(theme().elderly)
    }

    /// A translucent shadow used under raised surfaces.
    pub fn SHADOW() -> Color {
        c(theme().shadow)
    }
}

/// Type scale (font sizes in logical px). Replaces the ad-hoc 11..32 sizes.
pub mod scale {
    pub const TITLE: f32 = 23.0;
    pub const HEADING: f32 = 19.0;
    pub const BODY: f32 = 16.0;
    pub const LABEL: f32 = 14.0;
    pub const CAPTION: f32 = 13.0;
}

/// Spacing scale (logical px). Replaces magic 10/20/25/... offsets.
pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    /// Default inner padding for panels/cards.
    pub const PAD: f32 = 16.0;
}

// --- Surface style builders ----------------------------------------------

/// A raised card: filled surface with a soft drop shadow and hairline border.
pub fn card_style() -> SurfaceStyle {
    SurfaceStyle::new(color::SURFACE())
        .with_shadow(macroquad::prelude::vec2(0.0, 3.0), color::SHADOW())
        .with_border(1.0, color::BORDER())
}

/// A selected/active card variant with the primary accent edge.
pub fn card_selected_style() -> SurfaceStyle {
    SurfaceStyle::new(color::SURFACE_ALT())
        .with_shadow(macroquad::prelude::vec2(0.0, 3.0), color::SHADOW())
        .with_border(2.0, color::PRIMARY())
        .with_left_accent(4.0, color::PRIMARY())
}

/// A titled panel surface (header strip + divider).
pub fn panel_style() -> SurfaceStyle {
    SurfaceStyle::new(color::SURFACE())
        .with_shadow(macroquad::prelude::vec2(0.0, 3.0), color::SHADOW())
        .with_border(1.0, color::BORDER())
        .with_header(38.0, color::SURFACE_HEADER())
        .with_header_divider(1.0, color::BORDER_STRONG())
}

// --- Button style builders ------------------------------------------------

/// Semantic button tone drawn with the theme palette (not the toolkit's
/// default blue), so buttons match the restyle.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Primary,
    Secondary,
    Positive,
    Danger,
}

fn dim(c: Color, f: f32) -> Color {
    Color::new(c.r * f, c.g * f, c.b * f, c.a)
}

fn lift(c: Color, f: f32) -> Color {
    Color::new(
        (c.r * f).min(1.0),
        (c.g * f).min(1.0),
        (c.b * f).min(1.0),
        c.a,
    )
}

/// Build a `ButtonStyle` for a tone from the theme palette.
pub fn button_style(tone: Tone) -> ButtonStyle {
    match tone {
        Tone::Primary => ButtonStyle {
            normal: color::PRIMARY(),
            hovered: color::PRIMARY_HOVER(),
            pressed: color::PRIMARY_PRESSED(),
            border: color::PRIMARY_HOVER(),
            text_color: Color::new(0.10, 0.08, 0.04, 1.0),
            disabled: Color::new(0.18, 0.16, 0.12, 1.0),
        },
        Tone::Secondary => ButtonStyle {
            normal: color::SURFACE_ALT(),
            hovered: color::HOVERED(),
            pressed: dim(color::SURFACE_ALT(), 0.8),
            border: color::BORDER_STRONG(),
            text_color: color::TEXT(),
            disabled: Color::new(0.12, 0.12, 0.14, 1.0),
        },
        Tone::Positive => tone_style(color::POSITIVE()),
        Tone::Danger => tone_style(color::NEGATIVE()),
    }
}

fn tone_style(base: Color) -> ButtonStyle {
    ButtonStyle {
        normal: base,
        hovered: lift(base, 1.15),
        pressed: dim(base, 0.75),
        border: lift(base, 1.2),
        text_color: Color::new(0.08, 0.09, 0.06, 1.0),
        disabled: dim(base, 0.3),
    }
}

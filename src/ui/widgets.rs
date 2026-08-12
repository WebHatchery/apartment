//! # Widgets
//!
//! Game-semantic UI components composed from `macroquad-toolkit`, plus a
//! `Column` layout cursor that lays panels out by *measured* content height so
//! nothing overlaps or overflows. Panels use these instead of hand-drawing
//! rectangles at magic offsets.

use macroquad::prelude::*;
use macroquad_toolkit::ui::{
    button_rect_enabled_styled_ex, draw_surface, draw_ui_text, measure_ui_text,
    truncate_text_to_width, wrap_text_ex, ButtonTrigger, TextStyle,
};

use super::theme::{self, color, scale, space, Tone};

/// Approximate line height for a given font size.
pub fn line_height(size: f32) -> f32 {
    size * 1.35
}

/// Small dim uppercase section label. Returns the height consumed.
pub fn section_label(x: f32, y: f32, text: &str) -> f32 {
    draw_ui_text(text, x, y + scale::LABEL, scale::LABEL, color::TEXT_DIM());
    line_height(scale::LABEL) + space::XS
}

/// A key/value row: dim key on the left, value right-aligned to `width`.
/// Right-aligning the value (instead of a fixed x-offset) is what stops the
/// label text and value/icon from overlapping. Returns height consumed.
pub fn kv_row(x: f32, y: f32, width: f32, key: &str, value: &str, value_color: Color) -> f32 {
    let size = scale::BODY;
    draw_ui_text(key, x, y + size, size, color::TEXT_DIM());
    let vw = measure_ui_text(value, None, size as u16, 1.0).width;
    draw_ui_text(value, x + width - vw, y + size, size, value_color);
    line_height(size) + 3.0
}

/// A full-width labeled meter with the percentage right-aligned after the bar.
/// Returns height consumed.
pub fn stat_meter(x: f32, y: f32, width: f32, value: i32, max: i32, fill: Color) -> f32 {
    use macroquad_toolkit::ui::progress_bar;
    let pct = format!("{}%", value);
    let pct_w = measure_ui_text(&pct, None, scale::LABEL as u16, 1.0).width;
    let bar_w = (width - pct_w - space::SM).max(20.0);
    let bar_h = 16.0;
    progress_bar(x, y, bar_w, bar_h, value as f32, max as f32, fill);
    draw_ui_text(
        &pct,
        x + width - pct_w,
        y + bar_h - 3.0,
        scale::LABEL,
        color::TEXT(),
    );
    bar_h + space::SM
}

/// Wrap text to a pixel width using the shared UI font (measured, not by char
/// count). Thin wrapper over the toolkit so panels have one entry point.
pub fn wrap(text: &str, max_width: f32, size: f32) -> Vec<String> {
    wrap_text_ex(text, max_width, None, size)
}

/// Draw a raised card surface (optionally selected/active).
pub fn draw_card(rect: Rect, selected: bool) {
    let style = if selected {
        theme::card_selected_style()
    } else {
        theme::card_style()
    };
    draw_surface(rect, &style);
}

/// Draw a titled panel: shadowed surface + header strip with a left-aligned
/// title. Returns the inner content rect (inside padding, below the header).
pub fn draw_panel(rect: Rect, title: &str) -> Rect {
    draw_surface(rect, &theme::panel_style());
    draw_panel_header(rect, title);
    let header_h = 38.0;
    Rect::new(
        rect.x + space::PAD,
        rect.y + header_h + space::SM,
        rect.w - space::PAD * 2.0,
        rect.h - header_h - space::SM - space::PAD,
    )
}

/// Redraw a panel's fixed header after scrollable content, masking any row
/// that straddled the upper content boundary.
pub fn draw_panel_header(rect: Rect, title: &str) {
    let header_h = 38.0;
    draw_rectangle(rect.x, rect.y, rect.w, header_h, color::SURFACE_HEADER());
    draw_line(
        rect.x,
        rect.y + header_h,
        rect.right(),
        rect.y + header_h,
        1.0,
        color::BORDER_STRONG(),
    );
    // Left-aligned title, vertically centered in the header strip.
    let baseline = rect.y + header_h / 2.0 + scale::HEADING / 2.0 - 2.0;
    let title = truncate_text_to_width(title, rect.w - space::PAD * 2.0, scale::HEADING);
    draw_ui_text(
        &title,
        rect.x + space::PAD,
        baseline,
        scale::HEADING,
        color::TEXT_BRIGHT(),
    );
}

/// Natural button width for a label at the given height.
pub fn button_width(text: &str, height: f32) -> f32 {
    let w = measure_ui_text(text, None, scale::LABEL as u16, 1.0).width;
    (w + space::MD * 2.0).max(height * 1.6)
}

/// Draw a tone button in a rect. Triggers on release. Returns true if clicked.
pub fn button_at(rect: Rect, text: &str, enabled: bool, tone: Tone) -> bool {
    let style = theme::button_style(tone);
    let text_style = TextStyle::new(scale::LABEL, style.text_color);
    button_rect_enabled_styled_ex(
        rect,
        text,
        enabled,
        &style,
        text_style,
        ButtonTrigger::Release,
    )
}

/// Fixed, touch-friendly controls for content that extends below a panel.
/// Returns the updated offset in logical pixels.
pub fn scroll_controls(rect: Rect, current: f32, max: f32) -> f32 {
    if max <= 1.0 {
        return current;
    }
    let gap = space::SM;
    let button_w = (rect.w - gap) / 2.0;
    let mut next = current;
    draw_rectangle(
        rect.x - space::XS,
        rect.y - space::XS,
        rect.w + space::XS * 2.0,
        rect.h + space::XS * 2.0,
        color::SURFACE(),
    );
    draw_line(
        rect.x,
        rect.y - space::XS,
        rect.right(),
        rect.y - space::XS,
        1.0,
        color::BORDER(),
    );
    if button_at(
        Rect::new(rect.x, rect.y, button_w, rect.h),
        "Earlier",
        current > 1.0,
        Tone::Secondary,
    ) {
        next = (current - 120.0).max(0.0);
    }
    if button_at(
        Rect::new(rect.x + button_w + gap, rect.y, button_w, rect.h),
        "More",
        current + 1.0 < max,
        Tone::Primary,
    ) {
        next = (current + 120.0).min(max);
    }
    next
}

/// Draw a compact badge/chip with a leading label. Returns its width so
/// callers can flow badges left-to-right without overlap.
pub fn draw_badge(x: f32, y: f32, height: f32, text: &str, fill: Color, text_color: Color) -> f32 {
    let text_w = measure_ui_text(text, None, scale::LABEL as u16, 1.0).width;
    let w = text_w + space::MD * 2.0;
    let style = macroquad_toolkit::ui::SurfaceStyle::new(fill)
        .with_border(1.0, Color::new(1.0, 1.0, 1.0, 0.12));
    draw_surface(Rect::new(x, y, w, height), &style);
    let baseline = y + height / 2.0 + scale::LABEL / 2.0 - 1.0;
    draw_ui_text(text, x + space::MD, baseline, scale::LABEL, text_color);
    w
}

/// Category for the shared bottom toast.
#[derive(Clone, Copy)]
pub enum ToastKind {
    Info,
    Positive,
    Warning,
    Hint,
}

impl ToastKind {
    fn accent(self) -> Color {
        match self {
            ToastKind::Info => color::ACCENT(),
            ToastKind::Positive => color::POSITIVE(),
            ToastKind::Warning => color::WARNING(),
            ToastKind::Hint => color::TEXT_DIM(),
        }
    }
}

/// Draw a single bottom-center toast (used for tutorial + notifications), with
/// measured word-wrap and an optional action button. Returns true if the
/// action button was clicked. `icon` is drawn to the left when non-empty.
pub fn draw_toast(
    icon: &str,
    title: &str,
    body: &str,
    kind: ToastKind,
    action_label: &str,
) -> bool {
    let accent = kind.accent();
    let panel_w = (screen_width() * 0.6).clamp(420.0, 680.0);
    let text_x_pad = if icon.is_empty() { space::LG } else { 74.0 };
    let content_w = panel_w - text_x_pad - space::LG;

    // Measure required height from wrapped content.
    let mut lines = 0usize;
    if !title.is_empty() {
        lines += 1;
    }
    let body_lines = wrap(body, content_w, scale::BODY);
    lines += body_lines.len();
    let text_h = lines as f32 * line_height(scale::BODY) + space::SM;
    let panel_h = (text_h + space::LG * 2.0 + 44.0).max(120.0);

    let panel_x = (screen_width() - panel_w) / 2.0;
    let footer_y = screen_height() - crate::ui::layout::FOOTER_HEIGHT();
    let panel_y =
        (footer_y - panel_h - space::SM).max(crate::ui::layout::HEADER_HEIGHT() + space::SM);

    let style = macroquad_toolkit::ui::SurfaceStyle::new(color::SURFACE())
        .with_shadow(vec2(0.0, 4.0), color::SHADOW())
        .with_border(1.0, color::BORDER())
        .with_left_accent(5.0, accent);
    draw_surface(Rect::new(panel_x, panel_y, panel_w, panel_h), &style);

    if !icon.is_empty() {
        draw_ui_text(icon, panel_x + space::LG, panel_y + 46.0, 34.0, accent);
    }

    let mut y = panel_y + space::LG;
    if !title.is_empty() {
        draw_ui_text(
            title,
            panel_x + text_x_pad,
            y + scale::HEADING,
            scale::HEADING,
            color::TEXT_BRIGHT(),
        );
        y += line_height(scale::HEADING);
    }
    for l in &body_lines {
        draw_ui_text(
            l,
            panel_x + text_x_pad,
            y + scale::BODY,
            scale::BODY,
            color::TEXT(),
        );
        y += line_height(scale::BODY);
    }

    // Action button, bottom-right.
    if action_label.is_empty() {
        return false;
    }
    let btn_h = 40.0;
    let btn_w = button_width(action_label, btn_h).max(96.0);
    let btn_x = panel_x + panel_w - btn_w - space::LG;
    let btn_y = panel_y + panel_h - btn_h - space::MD;
    button_at(
        Rect::new(btn_x, btn_y, btn_w, btn_h),
        action_label,
        true,
        Tone::Primary,
    )
}

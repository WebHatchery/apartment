//! Compact global status and time controls. Navigation shares the desktop row.
use super::theme::{color, Tone};
use super::widgets::button_at;
use super::UiAction;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, format_money, truncate_text_to_width};

pub fn draw_header(
    money: i32,
    tick: u32,
    occupancy: usize,
    total_units: usize,
) -> Option<UiAction> {
    let width = screen_width();
    draw_rectangle(
        0.0,
        0.0,
        width,
        super::layout::HEADER_HEIGHT(),
        color::SURFACE_HEADER(),
    );
    draw_line(
        0.0,
        super::layout::HEADER_HEIGHT(),
        width,
        super::layout::HEADER_HEIGHT(),
        1.0,
        color::BORDER(),
    );
    let mut action = None;
    let end_month_x = if width >= 1180.0 {
        width - 198.0
    } else {
        width - 182.0
    };
    if button_at(
        Rect::new(end_month_x, 8.0, 108.0, 40.0),
        "End Month",
        true,
        Tone::Primary,
    ) {
        action = Some(UiAction::EndTurn);
    }
    if width < 760.0
        && button_at(
            Rect::new(width - 68.0, 52.0, 60.0, 40.0),
            "Menu",
            true,
            Tone::Secondary,
        )
    {
        action = Some(UiAction::OpenPauseMenu);
    }
    let label = format!(
        "{}   Month {}   {}/{} leased",
        format_money(money as i64),
        tick,
        occupancy,
        total_units
    );
    let (x, y, available) = if width < 760.0 {
        // Portrait status uses three quiet rows: title, time control, and
        // compact operating status. Menu is placed in its own utility row.
        (12.0, 92.0, width - 24.0)
    } else {
        ((width - 510.0).max(12.0), 34.0, 312.0)
    };
    draw_ui_text(
        &truncate_text_to_width(&label, available, 18.0),
        x,
        y,
        18.0,
        if money < 0 {
            color::NEGATIVE()
        } else {
            color::TEXT()
        },
    );
    if width < 800.0 || (800.0..1180.0).contains(&width) {
        draw_ui_text("Second Story", 12.0, 34.0, 22.0, color::TEXT_BRIGHT());
    }
    action
}

//! The closed month at a glance: ledger totals and what changed, dismissed
//! with one Continue. Also drawn (without the button) in the history drawer.

use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, format_money};

use super::common::*;
use super::theme::{scale, space, Tone};
use super::widgets::{button_at, draw_panel, kv_row, line_height, wrap};
use super::UiAction;
use crate::simulation::EventSeverity;
use crate::state::MonthSummary;
use crate::text::{t, tr};

pub(crate) fn severity_color(severity: &EventSeverity) -> Color {
    match severity {
        EventSeverity::Positive => colors::POSITIVE(),
        EventSeverity::Info => colors::TEXT_DIM(),
        EventSeverity::Warning => colors::WARNING(),
        EventSeverity::Negative => colors::NEGATIVE(),
    }
}

fn money(value: i32) -> String {
    format_money(value as i64)
}

fn signed(value: i32) -> String {
    if value >= 0 {
        format!("+{}", money(value))
    } else {
        format!("-{}", money(value.abs()))
    }
}

/// Ledger rows worth showing: zero buckets are left out.
fn ledger_rows(summary: &MonthSummary) -> Vec<(String, String, Color)> {
    let report = &summary.report;
    let mut rows = vec![(
        t("month.rent"),
        money(report.rent_income),
        colors::POSITIVE(),
    )];
    if report.other_income > 0 {
        rows.push((
            t("month.other_income"),
            money(report.other_income),
            colors::POSITIVE(),
        ));
    }
    if report.operating_costs > 0 {
        rows.push((
            t("month.running_costs"),
            money(report.operating_costs),
            colors::TEXT(),
        ));
    }
    if report.repair_costs > 0 {
        rows.push((
            t("month.repairs"),
            money(report.repair_costs),
            colors::TEXT(),
        ));
    }
    if report.upgrade_costs > 0 {
        rows.push((
            t("month.upgrades"),
            money(report.upgrade_costs),
            colors::TEXT(),
        ));
    }
    rows.push((
        t("month.net"),
        signed(report.net),
        if report.net >= 0 {
            colors::POSITIVE()
        } else {
            colors::NEGATIVE()
        },
    ));
    rows.push((
        t("month.balance"),
        money(report.ending_balance),
        colors::TEXT_BRIGHT(),
    ));
    rows
}

/// Height `draw_summary_body` needs at `width`, so cards fit their content.
fn summary_body_height(summary: &MonthSummary, width: f32) -> f32 {
    let rows = ledger_rows(summary).len() as f32 * (line_height(scale::BODY) + 3.0);
    let lines: usize = summary
        .lines
        .iter()
        .map(|line| wrap(&line.text, width, scale::LABEL).len())
        .sum();
    rows + space::SM + lines as f32 * line_height(scale::LABEL)
}

/// Draw the summary's rows into `rect`, returning the y after the last row.
/// When the lines do not all fit, the last row says where the rest are.
pub(crate) fn draw_summary_body(summary: &MonthSummary, rect: Rect) -> f32 {
    let row_h = line_height(scale::BODY) + 3.0;
    let line_h = line_height(scale::LABEL);
    let overflow = summary_body_height(summary, rect.w) > rect.h;
    let lines_bottom = if overflow {
        rect.bottom() - line_h
    } else {
        rect.bottom()
    };
    let mut y = rect.y;
    for (key, value, value_color) in ledger_rows(summary) {
        if y + row_h > rect.bottom() {
            return y;
        }
        kv_row(rect.x, y, rect.w, &key, &value, value_color);
        y += row_h;
    }
    y += space::SM;
    'lines: for line in &summary.lines {
        for text in wrap(&line.text, rect.w, scale::LABEL) {
            if y + line_h > lines_bottom {
                break 'lines;
            }
            draw_ui_text(
                &text,
                rect.x,
                y + scale::LABEL,
                scale::LABEL,
                severity_color(&line.severity),
            );
            y += line_h;
        }
    }
    if overflow && y + line_h <= rect.bottom() {
        draw_ui_text(
            &t("month.overflow"),
            rect.x,
            y + scale::LABEL,
            scale::LABEL,
            colors::TEXT_DIM(),
        );
        y += line_h;
    }
    y
}

/// The modal card shown right after End Month. Returns Dismiss on Continue.
pub fn draw_month_summary(summary: &MonthSummary) -> Option<UiAction> {
    let top = layout::HEADER_HEIGHT();
    let available = screen_height() - top - layout::FOOTER_HEIGHT() - space::LG;
    draw_rectangle(
        0.0,
        top,
        screen_width(),
        screen_height() - top,
        Color::new(0.0, 0.0, 0.0, 0.55),
    );
    let w = (screen_width() - space::LG * 2.0).min(460.0);
    // Panel chrome: header strip, padding, the Continue button and its gap.
    let chrome = 38.0 + space::SM + space::PAD + 44.0 + space::SM;
    let body = summary_body_height(summary, w - space::PAD * 2.0);
    let h = (body + chrome + space::SM).min(available);
    let card = Rect::new(
        (screen_width() - w) * 0.5,
        top + (available - h) * 0.5 + space::SM,
        w,
        h,
    );
    let inner = draw_panel(card, &tr("month.title", &[("month", &summary.month)]));
    let button = Rect::new(inner.x, inner.bottom() - 44.0, inner.w, 44.0);
    draw_summary_body(
        summary,
        Rect::new(inner.x, inner.y, inner.w, button.y - inner.y - space::SM),
    );
    button_at(button, &t("month.continue"), true, Tone::Primary)
        .then_some(UiAction::DismissMonthSummary)
}

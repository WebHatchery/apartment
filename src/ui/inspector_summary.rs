//! Default building inspector shown before the player selects a specific unit.

use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, format_money, truncate_text_to_width};

use crate::building::Building;
use crate::tenant::Tenant;

use super::common::layout;
use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, draw_card, draw_panel, kv_row, line_height, section_label};
use super::UiAction;

pub fn draw_building_summary(
    building: &Building,
    tenants: &[Tenant],
    applications: usize,
    monthly_net: i32,
) -> Option<UiAction> {
    let padding = layout::PADDING();
    let panel_x = screen_width() * layout::PANEL_SPLIT() + padding;
    let panel_y = layout::HEADER_HEIGHT() + padding;
    let panel_w = screen_width() - panel_x - padding;
    let panel_h = screen_height() - panel_y - layout::FOOTER_HEIGHT() - padding;
    let inner = draw_panel(
        Rect::new(panel_x, panel_y, panel_w, panel_h),
        "Building inspector",
    );

    let units = building.rental_unit_count();
    let occupied = building.occupancy_count();
    let average_condition = if building.apartments.is_empty() {
        0
    } else {
        building
            .apartments
            .iter()
            .map(|unit| unit.condition)
            .sum::<i32>()
            / building.apartments.len() as i32
    };
    let average_happiness = if tenants.is_empty() {
        None
    } else {
        Some(tenants.iter().map(|tenant| tenant.happiness).sum::<i32>() / tenants.len() as i32)
    };
    let net_label = if monthly_net > 0 {
        format!("+{}", format_money(monthly_net as i64))
    } else {
        format_money(monthly_net as i64)
    };
    let dense = inner.h < 300.0;

    let mut y = inner.y;
    y += section_label(inner.x, y, "AT A GLANCE");
    y += kv_row(
        inner.x,
        y,
        inner.w,
        "Occupancy",
        &format!("{} / {}", occupied, units),
        if occupied == units {
            color::POSITIVE()
        } else {
            color::TEXT()
        },
    );
    if !dense {
        y += kv_row(
            inner.x,
            y,
            inner.w,
            "Unit condition",
            &format!("{}% avg", average_condition),
            super::common::condition_color(average_condition),
        );
        y += kv_row(
            inner.x,
            y,
            inner.w,
            "Resident happiness",
            &average_happiness.map_or_else(
                || "No residents".to_string(),
                |value| format!("{}% avg", value),
            ),
            average_happiness.map_or(color::TEXT_DIM(), super::common::happiness_color),
        );
    }
    y += kv_row(
        inner.x,
        y,
        inner.w,
        "Last monthly net",
        &net_label,
        if monthly_net < 0 {
            color::NEGATIVE()
        } else {
            color::POSITIVE()
        },
    );
    y += if dense { space::XS } else { space::SM };

    let critical = building
        .apartments
        .iter()
        .filter(|unit| unit.condition < 40)
        .count();
    let vacant = building
        .apartments
        .iter()
        .filter(|unit| unit.is_vacant())
        .count();
    let (priority, detail, priority_color) = if building.hallway_condition < 80 {
        (
            "Hallway needs attention",
            format!(
                "Condition is {}%. Repairing shared space protects every lease.",
                building.hallway_condition
            ),
            color::WARNING(),
        )
    } else if critical > 0 {
        (
            "Repairs are due",
            format!(
                "{} unit{} below 40% condition.",
                critical,
                if critical == 1 { " is" } else { "s are" }
            ),
            color::NEGATIVE(),
        )
    } else if vacant > 0 && applications > 0 {
        (
            "Applicants are waiting",
            format!(
                "{} applicant{} for {} vacant unit{}.",
                applications,
                if applications == 1 { "" } else { "s" },
                vacant,
                if vacant == 1 { "" } else { "s" }
            ),
            color::ACCENT(),
        )
    } else if vacant > 0 {
        (
            "Vacancies to fill",
            format!(
                "{} vacant unit{}. Select a unit to prepare a lease.",
                vacant,
                if vacant == 1 { "" } else { "s" }
            ),
            color::TEXT(),
        )
    } else {
        (
            "Building is stable",
            "Review finances or plan the next improvement.".to_string(),
            color::POSITIVE(),
        )
    };

    let priority_h = if dense { 42.0 } else { 70.0 };
    draw_card(Rect::new(inner.x, y, inner.w, priority_h), false);
    draw_ui_text(
        priority,
        inner.x + space::MD,
        y + if dense { 27.0 } else { 23.0 },
        scale::BODY,
        priority_color,
    );
    if !dense {
        draw_ui_text(
            &truncate_text_to_width(&detail, inner.w - space::MD * 2.0, scale::CAPTION),
            inner.x + space::MD,
            y + 48.0,
            scale::CAPTION,
            color::TEXT_DIM(),
        );
    }
    y += priority_h + if dense { space::XS } else { space::MD };

    y += section_label(inner.x, y, "QUICK ACTIONS");
    let button_h = if dense { 38.0 } else { 42.0 };
    let mut action = None;
    let applications_label = format!("Review applications ({})", applications);
    let button_gap = space::XS;
    let compact_w = (inner.w - button_gap) * 0.5;
    if button_at(
        Rect::new(
            inner.x,
            y,
            if dense { compact_w } else { inner.w },
            button_h,
        ),
        &applications_label,
        true,
        if applications > 0 {
            Tone::Primary
        } else {
            Tone::Secondary
        },
    ) {
        action = Some(UiAction::SelectApplications(None));
    }
    if button_at(
        Rect::new(
            if dense {
                inner.x + compact_w + button_gap
            } else {
                inner.x
            },
            if dense { y } else { y + button_h + space::SM },
            if dense { compact_w } else { inner.w },
            button_h,
        ),
        "Inspect hallway",
        true,
        Tone::Secondary,
    ) {
        action = Some(UiAction::SelectHallway);
    }
    y += button_h + space::SM;
    if !dense {
        y += button_h + space::SM;
    }
    if y + button_h <= inner.bottom() + line_height(scale::CAPTION)
        && button_at(
            Rect::new(inner.x, y, inner.w, button_h),
            "Ownership options",
            true,
            Tone::Secondary,
        )
    {
        action = Some(UiAction::SelectOwnership);
    }
    action
}

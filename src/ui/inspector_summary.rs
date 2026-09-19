//! Building details and secondary management routes, shown only on selection.
use super::theme::{color, Tone};
use super::widgets::{button_at, draw_panel, kv_row};
use super::UiAction;
use crate::building::Building;
use crate::tenant::Tenant;
use macroquad::prelude::*;
use macroquad_toolkit::ui::format_money;

pub fn draw_building_summary(
    building: &Building,
    tenants: &[Tenant],
    applications: usize,
    monthly_net: i32,
) -> Option<UiAction> {
    let inner = draw_panel(super::layout::context_panel(), "Building");
    let mut y = inner.y;
    y += kv_row(
        inner.x,
        y,
        inner.w,
        "Occupancy",
        &format!(
            "{} / {}",
            building.occupancy_count(),
            building.rental_unit_count()
        ),
        color::TEXT(),
    );
    y += kv_row(
        inner.x,
        y,
        inner.w,
        "Condition",
        &format!("{}%", building.average_condition()),
        super::condition_color(building.average_condition()),
    );
    y += kv_row(
        inner.x,
        y,
        inner.w,
        "Hallway",
        &format!("{}%", building.hallway_condition),
        super::condition_color(building.hallway_condition),
    );
    if inner.h > 330.0 {
        let happiness = if tenants.is_empty() {
            "No residents".to_string()
        } else {
            format!(
                "{}%",
                tenants.iter().map(|tenant| tenant.happiness).sum::<i32>() / tenants.len() as i32
            )
        };
        y += kv_row(inner.x, y, inner.w, "Happiness", &happiness, color::TEXT());
        y += kv_row(
            inner.x,
            y,
            inner.w,
            "Last monthly net",
            &format_money(monthly_net as i64),
            color::TEXT(),
        );
    }
    let labels = [
        (
            format!("Applications ({})", applications),
            UiAction::SelectApplications(None),
        ),
        ("Shared spaces".to_string(), UiAction::SelectHallway),
        ("Ownership".to_string(), UiAction::SelectOwnership),
        ("Finances".to_string(), UiAction::OpenFinances),
    ];
    let width = (inner.w - 8.0) / 2.0;
    for (index, (label, intent)) in labels.into_iter().enumerate() {
        let rect = Rect::new(
            inner.x + (index % 2) as f32 * (width + 8.0),
            y + 8.0 + (index / 2) as f32 * 48.0,
            width,
            40.0,
        );
        if button_at(rect, &label, true, Tone::Secondary) {
            return Some(intent);
        }
    }
    None
}

//! Activity footer (current issues) and the history drawer.

use super::common::*;
use super::month_summary::{draw_summary_body, severity_color};
use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, line_height, wrap};
use super::UiAction;
use crate::assets::AssetManager;
use crate::simulation::{EventLog, GameEvent};
use crate::state::{MonthSummary, SummaryLine};
use apartment::text::{t, tr};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

/// Height of the history drawer above the footer; also the area whose clicks
/// the drawer owns.
pub fn drawer_height() -> f32 {
    let footer_y = screen_height() - layout::FOOTER_HEIGHT();
    360.0_f32.min(footer_y - layout::HEADER_HEIGHT() - space::SM)
}

/// The footer: the most pressing issue that is still true (or nothing when
/// the building is calm) and a quiet control for the history drawer.
pub fn draw_notifications(
    event_log: &EventLog,
    issue: Option<&SummaryLine>,
    latest_month: Option<&MonthSummary>,
    assets: &AssetManager,
    expanded: bool,
) -> Option<UiAction> {
    let y = screen_height() - layout::FOOTER_HEIGHT();
    let w = screen_width();
    let h = layout::FOOTER_HEIGHT();

    if expanded {
        draw_activity_drawer(event_log, latest_month, y, assets);
    }

    draw_rectangle(0.0, y, w, h, colors::SURFACE_HEADER());
    draw_line(0.0, y, w, y, 1.0, colors::BORDER_STRONG());

    let button_w = 116.0;
    let text_w = w - button_w - space::LG * 3.0;
    let (label, size, label_color) = match issue {
        Some(issue) => (
            truncate_text_to_width(&issue.text, text_w, scale::BODY),
            scale::BODY,
            severity_color(&issue.severity),
        ),
        None => (t("status.activity"), scale::LABEL, colors::TEXT_DIM()),
    };
    draw_ui_text(&label, space::LG, y + 26.0, size, label_color);

    if button_at(
        Rect::new(w - button_w - space::LG, y + 2.0, button_w, 40.0),
        &t(if expanded {
            "status.hide_history"
        } else {
            "status.view_history"
        }),
        true,
        Tone::Secondary,
    ) {
        Some(UiAction::ToggleActivityDrawer)
    } else {
        None
    }
}

/// Last month's result first, then earlier activity, wrapped to the width.
fn draw_activity_drawer(
    event_log: &EventLog,
    latest_month: Option<&MonthSummary>,
    footer_y: f32,
    assets: &AssetManager,
) {
    let drawer_h = drawer_height();
    let y = footer_y - drawer_h;
    draw_rectangle(0.0, y, screen_width(), drawer_h, color::SURFACE());
    draw_line(0.0, y, screen_width(), y, 1.0, color::BORDER_STRONG());
    let x = space::LG;
    let max_w = (screen_width() - space::LG * 2.0).min(720.0);
    let mut event_y = y + space::MD;
    if let Some(summary) = latest_month {
        draw_ui_text(
            &tr("status.history_title", &[("month", &summary.month)]),
            x,
            event_y + scale::HEADING,
            scale::HEADING,
            color::TEXT_BRIGHT(),
        );
        event_y += line_height(scale::HEADING) + space::SM;
        let body = Rect::new(x, event_y, max_w, footer_y - event_y - space::SM);
        event_y = draw_summary_body(summary, body) + space::MD;
    }
    if event_y + line_height(scale::HEADING) > footer_y {
        return;
    }
    draw_ui_text(
        &t("status.earlier_activity"),
        x,
        event_y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    event_y += line_height(scale::LABEL) + space::XS;
    for event in event_log
        .recent_events(40)
        .into_iter()
        .filter(|event| !matches!(event, GameEvent::MonthEnd { .. }))
    {
        let icon_rect = Rect::new(x, event_y, 18.0, 18.0);
        let text_x = if draw_event_icon(event, icon_rect, assets) {
            icon_rect.right() + space::SM
        } else {
            x
        };
        for line in wrap(&event.message(), max_w - (text_x - x), scale::LABEL) {
            if event_y + line_height(scale::LABEL) > footer_y - space::XS {
                return;
            }
            draw_ui_text(
                &line,
                text_x,
                event_y + scale::LABEL,
                scale::LABEL,
                severity_color(&event.severity()),
            );
            event_y += line_height(scale::LABEL);
        }
        event_y += space::XS;
    }
}

fn draw_event_icon(event: &GameEvent, rect: Rect, assets: &AssetManager) -> bool {
    let id = match event {
        GameEvent::RentPaid { .. } | GameEvent::RentMissed { .. } => "event_rent_collected",
        GameEvent::TenantMovedIn { .. } => "event_tenant_moved_in",
        GameEvent::TenantMovedOut { .. } => "event_tenant_moved_out",
        GameEvent::NoiseComplaint { .. } => "event_noise_complaint",
        GameEvent::PipeBurst { .. }
        | GameEvent::BoilerFailure { .. }
        | GameEvent::StructuralIssue { .. } => "event_pipe_burst",
        GameEvent::Inspection { .. } => "event_inspection",
        GameEvent::Heatwave { .. } => "event_heatwave",
        GameEvent::NewApplication { .. } => "icon_application",
        GameEvent::UpgradeCompleted { .. } => "icon_upgrade",
        GameEvent::ConditionComplaint { .. }
        | GameEvent::PoorCondition { .. }
        | GameEvent::CriticalCondition { .. }
        | GameEvent::HallwayDeteriorating { .. }
        | GameEvent::TenantDamage { .. } => "icon_repair",
        GameEvent::MonthEnd { .. } => "icon_calendar",
        _ => return false,
    };
    if id.starts_with("event_") {
        crate::ui::event_art::draw_event_art(id, rect, assets)
    } else if let Some(texture) = assets.get_texture(id) {
        draw_texture_ex(
            texture,
            rect.x,
            rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(rect.w, rect.h)),
                ..Default::default()
            },
        );
        true
    } else {
        false
    }
}

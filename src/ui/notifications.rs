use super::common::*;
use super::theme::{color, scale, space, Tone};
use super::widgets::button_at;
use super::UiAction;
use crate::assets::AssetManager;
use crate::simulation::{EventLog, EventSeverity, GameEvent};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

pub fn draw_notifications(
    event_log: &EventLog,
    _current_tick: u32,
    assets: &AssetManager,
    expanded: bool,
) -> Option<UiAction> {
    let y = screen_height() - layout::FOOTER_HEIGHT();
    let w = screen_width();
    let h = layout::FOOTER_HEIGHT();

    if expanded {
        draw_activity_drawer(event_log, y, assets);
    }

    // Background
    draw_rectangle(0.0, y, w, h, colors::SURFACE_HEADER());
    draw_line(0.0, y, w, y, 1.0, colors::BORDER_STRONG());

    // Compact activity handle. The latest event remains visible without
    // permanently reserving the old 100 px footer.
    draw_ui_text(
        if expanded {
            "ACTIVITY · OPEN"
        } else {
            "ACTIVITY"
        },
        space::LG,
        y + 25.0,
        scale::LABEL,
        colors::TEXT_DIM(),
    );

    if let Some(event) = event_log.recent_events(1).first() {
        let color = match event.severity() {
            EventSeverity::Positive => colors::POSITIVE(),
            EventSeverity::Info => colors::TEXT_DIM(),
            EventSeverity::Warning => colors::WARNING(),
            EventSeverity::Negative => colors::NEGATIVE(),
        };

        let text_x = if draw_event_icon(event, Rect::new(112.0, y + 11.0, 24.0, 24.0), assets) {
            144.0
        } else {
            132.0
        };
        let display_msg = truncate_text_to_width(
            &event.message(),
            (w - text_x - 148.0).max(120.0),
            scale::BODY,
        );
        draw_ui_text(&display_msg, text_x, y + 26.0, scale::BODY, color);
    }

    if button_at(
        Rect::new(w - 132.0, y + 2.0, 116.0, 40.0),
        if expanded {
            "Hide history"
        } else {
            "View history"
        },
        true,
        Tone::Secondary,
    ) {
        Some(UiAction::ToggleActivityDrawer)
    } else {
        None
    }
}

fn draw_activity_drawer(event_log: &EventLog, footer_y: f32, assets: &AssetManager) {
    let drawer_h = 168.0_f32.min(footer_y - layout::HEADER_HEIGHT() - space::SM);
    let y = footer_y - drawer_h;
    draw_rectangle(0.0, y, screen_width(), drawer_h, color::SURFACE());
    draw_line(0.0, y, screen_width(), y, 1.0, color::BORDER_STRONG());
    draw_ui_text(
        "Recent building activity",
        space::LG,
        y + 28.0,
        scale::HEADING,
        color::TEXT_BRIGHT(),
    );

    let mut event_y = y + 56.0;
    let max_w = screen_width() - space::LG * 2.0;
    for event in event_log.recent_events(5) {
        let event_color = match event.severity() {
            EventSeverity::Positive => color::POSITIVE(),
            EventSeverity::Info => color::TEXT_DIM(),
            EventSeverity::Warning => color::WARNING(),
            EventSeverity::Negative => color::NEGATIVE(),
        };
        let icon_rect = Rect::new(space::LG, event_y - 16.0, 20.0, 20.0);
        let text_x = if draw_event_icon(event, icon_rect, assets) {
            icon_rect.right() + space::SM
        } else {
            space::LG
        };
        let display_msg = truncate_text_to_width(&event.message(), max_w - text_x, scale::BODY);
        draw_ui_text(&display_msg, text_x, event_y, scale::BODY, event_color);
        event_y += scale::BODY + space::SM;
        if event_y > footer_y - space::SM {
            break;
        }
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
    let Some(texture) = assets.get_texture(id) else {
        return false;
    };
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
}

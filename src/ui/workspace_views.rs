//! Tenants workspace and the shared workspace page frame.
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

use crate::assets::AssetManager;
use crate::state::GameplayState;

use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, draw_card, draw_panel, kv_row, line_height};
use super::UiAction;

pub(super) fn content_rect() -> Rect {
    Rect::new(
        space::LG,
        crate::ui::layout::HEADER_HEIGHT() + space::LG,
        screen_width() - space::LG * 2.0,
        screen_height()
            - crate::ui::layout::HEADER_HEIGHT()
            - crate::ui::layout::FOOTER_HEIGHT()
            - space::LG * 2.0,
    )
}

fn draw_title(rect: Rect, title: &str, subtitle: &str) -> f32 {
    draw_ui_text(
        title,
        rect.x,
        rect.y + scale::TITLE,
        scale::TITLE,
        color::TEXT_BRIGHT(),
    );
    draw_ui_text(
        subtitle,
        rect.x,
        rect.y + scale::TITLE + line_height(scale::BODY),
        scale::BODY,
        color::TEXT_DIM(),
    );
    rect.y + scale::TITLE + line_height(scale::BODY) + space::LG
}

pub fn draw_tenants_view(state: &GameplayState, assets: &AssetManager) -> Option<UiAction> {
    let rect = content_rect();
    let active_id = state.active_building_id();
    let tenants: Vec<_> = state
        .tenants
        .iter()
        .filter(|tenant| tenant.building_id == active_id)
        .collect();
    let applications = state
        .applications
        .iter()
        .filter(|application| application.building_id == active_id)
        .count();
    let pending_requests = tenants
        .iter()
        .filter(|tenant| {
            state
                .tenant_stories
                .get(&tenant.id)
                .is_some_and(|story| story.pending_request.is_some())
        })
        .count();
    let average = if tenants.is_empty() {
        0
    } else {
        tenants.iter().map(|tenant| tenant.happiness).sum::<i32>() / tenants.len() as i32
    };
    if rect.w < 760.0 {
        return super::tenants_view::draw_tenants_narrow(
            state,
            assets,
            rect,
            tenants,
            applications,
            pending_requests,
            average,
        );
    }
    let body_y = draw_title(
        rect,
        "Tenants",
        &format!("{} residents at {}", tenants.len(), state.building.name),
    );
    let gap = space::LG;
    let side_w = (rect.w * 0.30).clamp(220.0, 330.0);
    let roster = Rect::new(
        rect.x,
        body_y,
        rect.w - side_w - gap,
        rect.bottom() - body_y,
    );
    let side = Rect::new(roster.right() + gap, body_y, side_w, roster.h);
    let roster_inner = draw_panel(roster, "Resident roster");
    let mut y = roster_inner.y;

    if tenants.is_empty() {
        draw_ui_text(
            "No one currently lives in this building.",
            roster_inner.x,
            y + scale::BODY,
            scale::BODY,
            color::TEXT_DIM(),
        );
    }
    let row_h = 64.0;
    let controls_h = 48.0;
    let page_size = (((roster_inner.h - controls_h) / row_h).floor() as usize).max(1);
    let page_count = tenants.len().div_ceil(page_size).max(1);
    let page = state.tenants_page.min(page_count - 1);
    for tenant in tenants.iter().skip(page * page_size).take(page_size) {
        let row = Rect::new(roster_inner.x, y, roster_inner.w, row_h - space::XS);
        draw_card(row, false);
        let portrait = Rect::new(row.x + space::SM, row.y + space::SM, 44.0, 44.0);
        super::resident_sprite::draw_face_portrait(tenant, portrait, assets);
        let text_x = portrait.right() + space::SM;
        let unit = tenant
            .apartment_id
            .and_then(|id| state.building.get_apartment(id))
            .map(|apartment| apartment.unit_number.as_str())
            .unwrap_or("—");
        let name = truncate_text_to_width(&tenant.name, row.w * 0.34, scale::BODY);
        draw_ui_text(
            &name,
            text_x,
            row.y + 21.0,
            scale::BODY,
            color::TEXT_BRIGHT(),
        );
        draw_ui_text(
            &format!("{} · Unit {}", tenant.archetype.name(), unit),
            text_x,
            row.y + 42.0,
            scale::LABEL,
            color::TEXT_DIM(),
        );
        let rent = tenant
            .apartment_id
            .and_then(|id| state.building.get_apartment(id))
            .map(|apartment| apartment.rent_price)
            .unwrap_or(0);
        let status = format!(
            "{} happy · ${}/${}",
            tenant.happiness, rent, tenant.rent_tolerance
        );
        let status_w =
            macroquad_toolkit::ui::measure_ui_text(&status, None, scale::LABEL as u16, 1.0).width;
        draw_ui_text(
            &status,
            row.right() - status_w - space::MD,
            row.y + 31.0,
            scale::LABEL,
            if rent > tenant.rent_tolerance {
                color::WARNING()
            } else {
                color::TEXT()
            },
        );
        y += row_h;
    }
    if page_count > 1 {
        let button_y = roster_inner.bottom() - 40.0;
        let gap = space::XS;
        let button_w = (roster_inner.w - gap) * 0.5;
        if page > 0
            && button_at(
                Rect::new(roster_inner.x, button_y, button_w, 40.0),
                "Previous resident",
                true,
                Tone::Secondary,
            )
        {
            return Some(UiAction::SetTenantsPage { page: page - 1 });
        }
        if page + 1 < page_count
            && button_at(
                Rect::new(roster_inner.x + button_w + gap, button_y, button_w, 40.0),
                "Next resident",
                true,
                Tone::Secondary,
            )
        {
            return Some(UiAction::SetTenantsPage { page: page + 1 });
        }
    }

    let side_inner = draw_panel(side, "Leasing desk");
    let dense_side = side_inner.h < 220.0;
    let mut sy = side_inner.y;
    sy += kv_row(
        side_inner.x,
        sy,
        side_inner.w,
        "Occupancy",
        &format!(
            "{}/{}",
            state.building.occupancy_count(),
            state.building.rental_unit_count()
        ),
        color::TEXT(),
    );
    if !dense_side {
        sy += kv_row(
            side_inner.x,
            sy,
            side_inner.w,
            "Average happiness",
            &format!("{}%", average),
            if average >= 60 {
                color::POSITIVE()
            } else {
                color::WARNING()
            },
        );
    }
    sy += kv_row(
        side_inner.x,
        sy,
        side_inner.w,
        "Applications",
        &applications.to_string(),
        if applications > 0 {
            color::ACCENT()
        } else {
            color::TEXT_DIM()
        },
    );
    sy += kv_row(
        side_inner.x,
        sy,
        side_inner.w,
        "Pending requests",
        &pending_requests.to_string(),
        if pending_requests > 0 {
            color::WARNING()
        } else {
            color::TEXT_DIM()
        },
    );
    sy += if dense_side { space::SM } else { space::LG };
    if button_at(
        Rect::new(side_inner.x, sy, side_inner.w, 40.0),
        "Review applications",
        applications > 0,
        Tone::Primary,
    ) {
        return Some(UiAction::SelectApplications(None));
    }
    sy += 46.0;
    if button_at(
        Rect::new(side_inner.x, sy, side_inner.w, 40.0),
        "Open requests & tasks",
        true,
        Tone::Secondary,
    ) {
        return Some(UiAction::OpenTasks);
    }
    None
}

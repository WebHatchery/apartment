//! Responsive tenant roster and resident detail view.

use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

use crate::assets::AssetManager;
use crate::state::GameplayState;

use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, draw_card, draw_panel, line_height};
use super::UiAction;

pub(super) fn draw_tenants_narrow(
    state: &GameplayState,
    assets: &AssetManager,
    rect: Rect,
    tenants: Vec<&crate::tenant::Tenant>,
    applications: usize,
    pending_requests: usize,
    _average: i32,
) -> Option<UiAction> {
    draw_ui_text(
        "Tenants",
        rect.x,
        rect.y + scale::TITLE,
        scale::TITLE,
        color::TEXT_BRIGHT(),
    );
    let subtitle = format!(
        "{} residents · {} applications · {} requests",
        tenants.len(),
        applications,
        pending_requests
    );
    draw_ui_text(
        &subtitle,
        rect.x,
        rect.y + scale::TITLE + line_height(scale::LABEL),
        scale::LABEL,
        color::TEXT_DIM(),
    );
    let action_y = rect.y + scale::TITLE + line_height(scale::LABEL) + space::SM;
    let has_detail = state.selected_tenant_id.is_some();
    if !has_detail
        && applications > 0
        && button_at(
            Rect::new(rect.right() - 180.0, action_y, 180.0, 40.0),
            "Review applications",
            true,
            Tone::Primary,
        )
    {
        return Some(UiAction::SelectApplications(None));
    }
    let body_y = if has_detail {
        action_y
    } else {
        action_y + 48.0
    };
    let panel = Rect::new(rect.x, body_y, rect.w, rect.bottom() - body_y);
    if let Some(selected_id) = state.selected_tenant_id {
        let Some(tenant) = tenants.iter().find(|tenant| tenant.id == selected_id) else {
            return Some(UiAction::ClearTenantSelection);
        };
        let inner = draw_panel(panel, "Resident details");
        let portrait = Rect::new(inner.x, inner.y, 56.0, 56.0);
        super::resident_sprite::draw_face_portrait(tenant, portrait, assets);
        let unit = tenant
            .apartment_id
            .and_then(|id| state.building.get_apartment(id));
        let text_x = portrait.right() + space::MD;
        draw_ui_text(
            &format!(
                "{}{}",
                tenant.name,
                unit.map_or(String::new(), |apartment| {
                    format!(" · Unit {}", apartment.unit_number)
                })
            ),
            text_x,
            inner.y + scale::HEADING,
            scale::HEADING,
            color::TEXT_BRIGHT(),
        );
        draw_ui_text(
            &format!("{} · {}% happy", tenant.archetype.name(), tenant.happiness),
            text_x,
            inner.y + scale::HEADING + line_height(scale::LABEL),
            scale::LABEL,
            color::TEXT_DIM(),
        );
        if let Some(apartment) = unit {
            let back_y = inner.bottom() - 40.0;
            if button_at(
                Rect::new(inner.x, back_y - 48.0, inner.w, 40.0),
                &format!("Open unit {}", apartment.unit_number),
                true,
                Tone::Primary,
            ) {
                return Some(UiAction::OpenTenantUnit(apartment.id));
            }
            if button_at(
                Rect::new(inner.x, back_y, inner.w, 40.0),
                "Back to resident roster",
                true,
                Tone::Secondary,
            ) {
                return Some(UiAction::ClearTenantSelection);
            }
        } else if button_at(
            Rect::new(inner.x, inner.bottom() - 40.0, inner.w, 40.0),
            "Back to resident roster",
            true,
            Tone::Secondary,
        ) {
            return Some(UiAction::ClearTenantSelection);
        }
        return None;
    }

    let inner = draw_panel(panel, "Resident roster");
    let row_h = 68.0;
    let pager_h = 48.0;
    let page_size = (((inner.h - pager_h) / row_h).floor() as usize).max(1);
    let page_count = tenants.len().max(1).div_ceil(page_size);
    let page = state.tenants_page.min(page_count - 1);
    let mut y = inner.y;
    for tenant in tenants.iter().skip(page * page_size).take(page_size) {
        let row = Rect::new(inner.x, y, inner.w, row_h - space::XS);
        draw_card(row, false);
        let portrait = Rect::new(row.x + space::SM, row.y + space::SM, 48.0, 48.0);
        super::resident_sprite::draw_face_portrait(tenant, portrait, assets);
        let text_x = portrait.right() + space::SM;
        draw_ui_text(
            &tenant.name,
            text_x,
            row.y + 22.0,
            scale::BODY,
            color::TEXT_BRIGHT(),
        );
        let unit = tenant
            .apartment_id
            .and_then(|id| state.building.get_apartment(id))
            .map_or("No unit", |apartment| apartment.unit_number.as_str());
        draw_ui_text(
            &format!(
                "{} · Unit {} · {}% happy",
                tenant.archetype.name(),
                unit,
                tenant.happiness
            ),
            text_x,
            row.y + 45.0,
            scale::LABEL,
            color::TEXT_DIM(),
        );
        if row.contains(Vec2::from(mouse_position())) && is_mouse_button_released(MouseButton::Left)
        {
            return Some(UiAction::SelectTenant(tenant.id));
        }
        y += row_h;
    }
    if page_count > 1 {
        let button_y = inner.bottom() - 40.0;
        let button_w = (inner.w - space::SM) / 2.0;
        if button_at(
            Rect::new(inner.x, button_y, button_w, 40.0),
            "Previous resident",
            page > 0,
            Tone::Secondary,
        ) {
            return Some(UiAction::SetTenantsPage { page: page - 1 });
        }
        if button_at(
            Rect::new(inner.right() - button_w, button_y, button_w, 40.0),
            "Next resident",
            page + 1 < page_count,
            Tone::Primary,
        ) {
            return Some(UiAction::SetTenantsPage { page: page + 1 });
        }
    }
    None
}

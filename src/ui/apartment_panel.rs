use crate::assets::AssetManager;
use crate::building::{Apartment, Building};
use crate::consequences::TenantNetwork;
use crate::narrative::TenantStory;
use crate::tenant::Tenant;
use macroquad::prelude::*;
use std::collections::HashMap;

use super::apartment_panel_sections::{draw_apartment_stats, draw_sold_condo_panel, draw_upgrades};
use super::tenant_panel::draw_tenant_info;
use super::{common::*, UiAction};

pub fn draw_apartment_panel(
    apt: &Apartment,
    building: &Building,
    tenants: &[Tenant],
    money: i32,
    offset_x: f32,
    scroll_offset: f32,
    assets: &AssetManager,
    config: &crate::data::config::GameConfig,
    tenant_network: &TenantNetwork,
    stories: &HashMap<u32, TenantStory>,
) -> (Option<UiAction>, f32) {
    let mut action = None;
    let mut new_scroll = scroll_offset;

    let bounds = layout::context_panel();
    let panel_x = bounds.x + offset_x;
    let panel_y = bounds.y;
    let panel_w = bounds.w;
    let panel_h = bounds.h;

    if building.is_unit_sold(apt.id) {
        if let Some(act) =
            draw_sold_condo_panel(apt, building, money, panel_x, panel_y, panel_w, panel_h)
        {
            action = Some(act);
        }
        return (action, new_scroll);
    }

    panel(
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        &format!("Unit {}", apt.unit_number),
    );

    let mouse = mouse_position();
    let is_hovering = mouse.0 >= panel_x
        && mouse.0 <= panel_x + panel_w
        && mouse.1 >= panel_y
        && mouse.1 <= panel_y + panel_h;

    if is_hovering {
        let wheel = mouse_wheel();
        new_scroll -= wheel.1 * 30.0;
        new_scroll = new_scroll.max(0.0);
    }

    let content_x = panel_x + 15.0;
    let mut y = panel_y + 50.0 - new_scroll;
    let content_top = panel_y + 35.0;
    let content_bottom = panel_y + panel_h - 58.0;

    // Keep the existing repair reachable immediately after tapping visible wear.
    if let Some(repair) =
        crate::building::upgrades::available_apartment_upgrades(apt, &config.upgrades)
            .into_iter()
            .find(|upgrade| {
                matches!(
                    upgrade,
                    crate::building::UpgradeAction::RepairApartment { .. }
                )
            })
    {
        if let Some(cost) = repair.cost(building, &config.economy, &config.upgrades) {
            let label = format!(
                "{} (${})",
                repair.label(building, &config.ui, &config.upgrades),
                cost
            );
            if y >= content_top
                && y + 40.0 <= content_bottom
                && button(content_x, y, panel_w - 30.0, 40.0, &label, money >= cost)
            {
                action = Some(UiAction::UpgradeAction(repair));
            }
            y += 48.0;
        }
    }

    draw_apartment_stats(
        apt,
        building,
        assets,
        content_x,
        &mut y,
        panel_w,
        content_top,
        content_bottom,
    );

    if let Some(act) = draw_tenant_info(
        apt,
        tenants,
        assets,
        content_x,
        &mut y,
        panel_w,
        content_top,
        content_bottom,
        tenant_network,
        stories,
    ) {
        action = Some(act);
    }

    let (upgrade_action, scroll_result, max_scroll) = draw_upgrades(
        apt,
        building,
        money,
        content_x,
        &mut y,
        panel_w,
        content_top,
        content_bottom,
        new_scroll,
        config,
        assets,
    );
    if let Some(act) = upgrade_action {
        action = Some(act);
    }
    new_scroll = scroll_result;
    new_scroll = super::widgets::scroll_controls(
        Rect::new(content_x, panel_y + panel_h - 48.0, panel_w - 30.0, 40.0),
        new_scroll,
        max_scroll,
        "Unit overview",
        "Tenant & actions",
    );
    super::widgets::draw_panel_header(
        Rect::new(panel_x, panel_y, panel_w, panel_h),
        &format!("Unit {}", apt.unit_number),
    );

    (action, new_scroll)
}

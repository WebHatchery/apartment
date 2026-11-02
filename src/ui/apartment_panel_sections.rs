//! Unit inspector sections: sold-condo buyback, unit facts and improvements.

use crate::assets::AssetManager;
use crate::building::{Apartment, ApartmentSize, Building, NoiseLevel, UpgradeAction};
use macroquad::prelude::*;

use super::apartment_panel::UnitPanelContext;
use super::panel_flow::{PanelFlow, BUTTON_H};
use super::theme::{scale, Tone};
use super::{common::*, Disclosure, UiAction};
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

/// A sold condo can only be bought back; everything else is read-only.
pub(super) fn draw_sold_condo_panel(
    apt: &Apartment,
    building: &Building,
    money: i32,
    bounds: Rect,
) -> Option<UiAction> {
    let title = format!("Unit {} · sold", apt.unit_number);
    panel(bounds.x, bounds.y, bounds.w, bounds.h, &title);
    let mut flow = PanelFlow::new(bounds, 0.0);
    let mut action = None;
    flow.line("Privately owned condo", scale::HEADING, colors::WARNING());
    flow.gap();
    if let Some((owner_name, purchase_price)) = building.get_condo_info(apt.id) {
        flow.kv("Owner", &owner_name, colors::TEXT());
        flow.kv(
            "Sold for",
            &format!("${purchase_price}"),
            colors::TEXT_DIM(),
        );
        let buyback_price = (purchase_price as f32 * 1.1) as i32;
        let label = if money >= buyback_price {
            format!("Buy back · ${buyback_price}")
        } else {
            format!(
                "Buy back · ${buyback_price} (need ${} more)",
                buyback_price - money
            )
        };
        if flow.button(&label, money >= buyback_price, Tone::Primary) {
            action = Some(UiAction::BuybackCondo {
                apartment_id: apt.id,
            });
        }
    }
    flow.paragraph(
        "You cannot modify or rent this unit while it is privately owned.",
        scale::LABEL,
        colors::TEXT_DIM(),
    );
    super::widgets::draw_panel_header(bounds, &title);
    action
}

/// Fixed facts about the room, shown on request.
pub(super) fn draw_unit_details(flow: &mut PanelFlow, apt: &Apartment, building: &Building) {
    let size = match apt.size {
        ApartmentSize::Small => "Small",
        ApartmentSize::Medium => "Medium",
        ApartmentSize::Large => "Large",
        ApartmentSize::Penthouse => "Penthouse",
    };
    flow.kv("Design", &format!("{:?}", apt.design), colors::TEXT());
    flow.kv("Size", size, colors::TEXT());
    let (noise, noise_color) = match apt.effective_noise() {
        NoiseLevel::Low => ("Quiet", colors::POSITIVE()),
        NoiseLevel::High => ("Noisy", colors::WARNING()),
    };
    flow.kv("Noise", noise, noise_color);
    if apt.has_soundproofing {
        flow.kv("Soundproofing", "Yes", colors::POSITIVE());
    }
    if apt.kitchen_level > 0 {
        let kitchen = if apt.kitchen_level >= 2 {
            "Renovated"
        } else {
            "Improved"
        };
        flow.kv("Kitchen", kitchen, colors::POSITIVE());
    }
    if apt.flags.contains("has_better_lighting") {
        flow.kv("Lighting", "Upgraded", colors::POSITIVE());
    }
    if building.has_laundry {
        flow.kv("Laundry", "On site", colors::POSITIVE());
    }
    flow.kv(
        "Quality score",
        &apt.quality_score().to_string(),
        colors::TEXT_DIM(),
    );
}

/// Optional improvements (never the repair, which leads the panel) behind
/// one disclosure that names how many are available.
pub(super) fn draw_improvements(
    flow: &mut PanelFlow,
    apt: &Apartment,
    ctx: &UnitPanelContext,
) -> Option<UiAction> {
    let options: Vec<(UpgradeAction, i32)> =
        crate::building::upgrades::available_apartment_upgrades(apt, &ctx.config.upgrades)
            .into_iter()
            .filter(|upgrade| !matches!(upgrade, UpgradeAction::RepairApartment { .. }))
            .filter_map(|upgrade| {
                let cost = upgrade.cost(&ctx.config.economy, &ctx.config.upgrades)?;
                Some((upgrade, cost))
            })
            .collect();
    if options.is_empty() {
        return None;
    }
    let open = ctx.disclosure.is_open(Disclosure::Improvements);
    let mut action = None;
    if flow.disclosure(
        "Improvements",
        &format!("{} available", options.len()),
        open,
    ) {
        action = Some(UiAction::ToggleDisclosure(Disclosure::Improvements));
    }
    if !open {
        return action;
    }
    for (upgrade, cost) in options {
        let label = format!(
            "{} · ${}",
            upgrade.label(&ctx.config.ui, &ctx.config.upgrades),
            cost
        );
        let rect = Rect::new(flow.x, flow.y, flow.w, BUTTON_H);
        if flow.fits(BUTTON_H)
            && upgrade_button(rect, &label, ctx.money >= cost, &upgrade, ctx.assets)
        {
            action = Some(UiAction::UpgradeAction(upgrade));
        }
        flow.advance(BUTTON_H + 6.0);
    }
    action
}

fn upgrade_icon_tile(action: &UpgradeAction) -> (f32, f32) {
    match action {
        UpgradeAction::RepairApartment { .. } | UpgradeAction::RepairHallway { .. } => (0.0, 0.0),
        UpgradeAction::Apply { upgrade_id, .. } if upgrade_id == "soundproofing" => (1.0, 0.0),
        UpgradeAction::Apply { upgrade_id, .. } if upgrade_id == "kitchen_renovation" => (2.0, 0.0),
        UpgradeAction::Apply { upgrade_id, .. } if upgrade_id == "lighting_upgrade" => (0.0, 1.0),
        UpgradeAction::Apply { upgrade_id, .. } if upgrade_id == "install_laundry" => (1.0, 1.0),
        UpgradeAction::Apply { .. } => (2.0, 1.0),
    }
}

fn upgrade_button(
    rect: Rect,
    label: &str,
    enabled: bool,
    action: &UpgradeAction,
    assets: &AssetManager,
) -> bool {
    let clicked =
        crate::ui::widgets::button_at(rect, "", enabled, crate::ui::theme::Tone::Secondary);
    let icon_size = 28.0;
    let icon_rect = Rect::new(
        rect.x + 7.0,
        rect.y + (rect.h - icon_size) * 0.5,
        icon_size,
        icon_size,
    );
    let text_x = icon_rect.right() + 7.0;
    let text_w = rect.right() - text_x - 7.0;
    draw_upgrade_icon(action, icon_rect, enabled, assets);
    draw_ui_text(
        &truncate_text_to_width(label, text_w, crate::ui::theme::scale::LABEL),
        text_x,
        rect.y + rect.h * 0.5 + crate::ui::theme::scale::LABEL * 0.5 - 1.0,
        crate::ui::theme::scale::LABEL,
        if enabled {
            colors::TEXT()
        } else {
            colors::TEXT_DIM()
        },
    );
    clicked
}

fn draw_upgrade_icon(
    action: &UpgradeAction,
    rect: Rect,
    enabled: bool,
    assets: &AssetManager,
) -> bool {
    let Some(texture) = assets.get_texture("upgrade_icons") else {
        return false;
    };
    let (column, row) = upgrade_icon_tile(action);
    let tile_w = texture.width() / 3.0;
    let tile_h = texture.height() * 0.5;
    let inset_x = tile_w * 0.08;
    let inset_y = tile_h * 0.08;
    let source_w = tile_w - inset_x * 2.0;
    let source_h = tile_h - inset_y * 2.0;
    let fit_scale = (rect.w / source_w).min(rect.h / source_h);
    let fitted_w = source_w * fit_scale;
    let fitted_h = source_h * fit_scale;
    draw_texture_ex(
        texture,
        rect.x + (rect.w - fitted_w) * 0.5,
        rect.y + (rect.h - fitted_h) * 0.5,
        if enabled {
            WHITE
        } else {
            Color::new(0.45, 0.43, 0.4, 0.7)
        },
        DrawTextureParams {
            dest_size: Some(vec2(fitted_w, fitted_h)),
            source: Some(Rect::new(
                column * tile_w + inset_x,
                row * tile_h + inset_y,
                tile_w - inset_x * 2.0,
                tile_h - inset_y * 2.0,
            )),
            ..Default::default()
        },
    );
    true
}

#[cfg(test)]
mod tests;

use crate::assets::AssetManager;
use crate::building::{Apartment, ApartmentSize, Building, DesignType, NoiseLevel, UpgradeAction};
use macroquad::prelude::*;

use super::{common::*, UiAction};
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width};

pub(super) fn draw_sold_condo_panel(
    apt: &Apartment,
    building: &Building,
    money: i32,
    panel_x: f32,
    panel_y: f32,
    panel_w: f32,
    panel_h: f32,
) -> Option<UiAction> {
    panel(
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        &format!("Unit {} - SOLD", apt.unit_number),
    );

    let content_x = panel_x + 15.0;
    let mut y = panel_y + 60.0;

    draw_ui_text(
        "CONDO - PRIVATELY OWNED",
        content_x,
        y,
        20.0,
        colors::WARNING(),
    );
    y += 35.0;

    if let Some((owner_name, purchase_price)) = building.get_condo_info(apt.id) {
        draw_ui_text(
            &format!("Owner: {}", owner_name),
            content_x,
            y,
            18.0,
            colors::TEXT(),
        );
        y += 25.0;
        draw_ui_text(
            &format!("Purchased for: ${}", purchase_price),
            content_x,
            y,
            16.0,
            colors::TEXT_DIM(),
        );
        y += 35.0;

        let buyback_price = (purchase_price as f32 * 1.1) as i32;
        draw_ui_text("Buyback Option:", content_x, y, 16.0, colors::ACCENT());
        y += 25.0;

        let can_afford = money >= buyback_price;
        let btn_label = format!("Buy Back (${})", buyback_price);

        if button(content_x, y, panel_w - 30.0, 40.0, &btn_label, can_afford) {
            return Some(UiAction::BuybackCondo {
                apartment_id: apt.id,
            });
        }
        y += 45.0;

        if !can_afford {
            draw_ui_text("Insufficient funds", content_x, y, 14.0, colors::NEGATIVE());
        }
    }

    y += 30.0;
    draw_ui_text(
        "You cannot modify or rent this unit",
        content_x,
        y,
        14.0,
        colors::TEXT_DIM(),
    );
    y += 20.0;
    draw_ui_text(
        "while it is privately owned.",
        content_x,
        y,
        14.0,
        colors::TEXT_DIM(),
    );

    None
}

pub(super) fn draw_apartment_stats(
    apt: &Apartment,
    building: &Building,
    _assets: &AssetManager,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
) {
    use crate::ui::widgets::{kv_row, section_label, stat_meter};
    let w = panel_w - 30.0;
    let vis = |yy: f32| yy >= content_top && yy + 22.0 <= content_bottom;

    if vis(*y) {
        section_label(content_x, *y, "CONDITION");
    }
    *y += 22.0;
    if vis(*y) {
        stat_meter(
            content_x,
            *y,
            w,
            apt.condition,
            100,
            condition_color(apt.condition),
        );
    }
    *y += 28.0;

    let design_text = match apt.design {
        DesignType::Bare => "Bare",
        DesignType::Practical => "Practical",
        DesignType::Cozy => "Cozy",
        DesignType::Luxury => "Luxury",
        DesignType::Opulent => "Opulent",
    };
    if vis(*y) {
        kv_row(content_x, *y, w, "Design", design_text, colors::TEXT());
    }
    *y += 24.0;

    let size_text = match apt.size {
        ApartmentSize::Small => "Small",
        ApartmentSize::Medium => "Medium",
        ApartmentSize::Large => "Large",
        ApartmentSize::Penthouse => "Penthouse",
    };
    if vis(*y) {
        kv_row(content_x, *y, w, "Size", size_text, colors::TEXT());
    }
    *y += 24.0;

    let (noise_text, noise_color) = match apt.effective_noise() {
        NoiseLevel::Low => ("Quiet", colors::POSITIVE()),
        NoiseLevel::High => ("Noisy", colors::WARNING()),
    };
    if vis(*y) {
        kv_row(content_x, *y, w, "Noise", noise_text, noise_color);
    }
    *y += 24.0;

    if apt.has_soundproofing {
        if vis(*y) {
            kv_row(content_x, *y, w, "Soundproofing", "Yes", colors::POSITIVE());
        }
        *y += 24.0;
    }

    if apt.kitchen_level > 0 {
        if vis(*y) {
            kv_row(
                content_x,
                *y,
                w,
                "Kitchen",
                if apt.kitchen_level >= 2 {
                    "Renovated"
                } else {
                    "Improved"
                },
                colors::POSITIVE(),
            );
        }
        *y += 24.0;
    }

    if apt.flags.contains("has_better_lighting") {
        if vis(*y) {
            kv_row(content_x, *y, w, "Lighting", "Upgraded", colors::POSITIVE());
        }
        *y += 24.0;
    }

    if building.has_laundry {
        if vis(*y) {
            kv_row(content_x, *y, w, "Laundry", "On site", colors::POSITIVE());
        }
        *y += 24.0;
    }

    if vis(*y) {
        kv_row(
            content_x,
            *y,
            w,
            "Rent",
            &format!("${}/mo", apt.rent_price),
            colors::PRIMARY(),
        );
    }
    *y += 24.0;

    if vis(*y) {
        kv_row(
            content_x,
            *y,
            w,
            "Quality Score",
            &apt.quality_score().to_string(),
            colors::TEXT_DIM(),
        );
    }
    *y += 30.0;
}

pub(super) fn draw_upgrades(
    apt: &Apartment,
    building: &Building,
    money: i32,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
    current_scroll: f32,
    config: &crate::data::config::GameConfig,
    assets: &AssetManager,
) -> (Option<UiAction>, f32, f32) {
    let w = panel_w - 30.0;
    if *y > content_top && *y < content_bottom {
        draw_line(content_x, *y, content_x + w, *y, 1.0, colors::BORDER());
    }
    *y += 14.0;

    // Section header is drawn after the buttons (so the scroll hint can reflect
    // overflow), but reserve its row here.
    let header_y = *y;
    *y += 24.0;

    let btn_w = w;
    let btn_h = 40.0;
    let available = crate::building::upgrades::available_apartment_upgrades(apt, &config.upgrades);

    let upgrades_start_y = *y;
    let mut total_upgrade_height = 0.0;
    for upgrade in &available {
        if upgrade
            .cost(building, &config.economy, &config.upgrades)
            .is_some()
        {
            total_upgrade_height += btn_h + 8.0;
        }
    }

    let max_scroll =
        (upgrades_start_y + total_upgrade_height - content_bottom + current_scroll).max(0.0);
    let final_scroll = current_scroll.min(max_scroll);
    let mut action = None;

    for upgrade in available {
        if let Some(cost) = upgrade.cost(building, &config.economy, &config.upgrades) {
            let can_afford = money >= cost;
            let label = format!(
                "{} — ${}",
                upgrade.label(building, &config.ui, &config.upgrades),
                cost
            );

            if *y >= content_top
                && *y + btn_h <= content_bottom
                && upgrade_button(
                    Rect::new(content_x, *y, btn_w, btn_h),
                    &label,
                    can_afford,
                    &upgrade,
                    assets,
                )
            {
                action = Some(UiAction::UpgradeAction(upgrade));
            }
            *y += btn_h + 8.0;
        }
    }

    // UPGRADES header + right-aligned scroll hint (no longer overlaps buttons).
    if header_y >= content_top && header_y + 20.0 <= content_bottom {
        crate::ui::widgets::section_label(content_x, header_y, "UPGRADES");
        if max_scroll > final_scroll + 5.0 {
            let hint = "MORE BELOW";
            let hw = measure_ui_text(hint, None, 13, 1.0).width;
            draw_ui_text(
                hint,
                content_x + w - hw,
                header_y + 13.0,
                13.0,
                colors::TEXT_DIM(),
            );
        }
    }

    (action, final_scroll, max_scroll)
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
    draw_texture_ex(
        texture,
        rect.x,
        rect.y,
        if enabled {
            WHITE
        } else {
            Color::new(0.45, 0.43, 0.4, 0.7)
        },
        DrawTextureParams {
            dest_size: Some(vec2(rect.w, rect.h)),
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

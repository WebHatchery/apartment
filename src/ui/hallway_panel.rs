use crate::assets::AssetManager;
use crate::building::Building;
use macroquad::prelude::*;

use super::{common::*, UiAction};
use macroquad_toolkit::ui::draw_ui_text;

pub fn draw_hallway_panel(
    building: &Building,
    money: i32,
    offset_x: f32,
    scroll_offset: f32,
    assets: &AssetManager,
    config: &crate::data::config::GameConfig,
) -> (Option<UiAction>, f32) {
    let mut action = None;
    let mut new_scroll = scroll_offset;

    let panel_x = screen_width() * layout::PANEL_SPLIT() + layout::PADDING() + offset_x;
    let panel_y = layout::HEADER_HEIGHT() + layout::PADDING();
    let panel_w = screen_width() * (1.0 - layout::PANEL_SPLIT()) - layout::PADDING() * 2.0;

    if panel_x > screen_width() {
        return (None, scroll_offset);
    }

    let panel_h = screen_height()
        - layout::HEADER_HEIGHT()
        - layout::FOOTER_HEIGHT()
        - layout::PADDING() * 2.0;

    panel(panel_x, panel_y, panel_w, panel_h, "Hallway");

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
    let content_w = panel_w - 30.0;
    let content_top = panel_y + 40.0;
    let content_bottom = panel_y + panel_h - 58.0;
    let mut y = panel_y + 50.0 - new_scroll;

    if y >= content_top && y + 20.0 <= content_bottom {
        crate::ui::widgets::section_label(content_x, y, "CONDITION");
    }
    y += 22.0;

    if y >= content_top && y + 20.0 <= content_bottom {
        crate::ui::widgets::stat_meter(
            content_x,
            y,
            content_w,
            building.hallway_condition,
            100,
            condition_color(building.hallway_condition),
        );
    }
    y += 30.0;

    if y >= content_top && y + 14.0 <= content_bottom {
        draw_ui_text(
            "Affects overall building appeal",
            content_x,
            y,
            14.0,
            colors::TEXT_DIM(),
        );
    }
    y += 20.0;

    if y >= content_top && y + 18.0 <= content_bottom {
        let appeal = building.building_appeal();
        draw_ui_text(
            &format!("Building Appeal: {}", appeal),
            content_x,
            y,
            18.0,
            colors::ACCENT(),
        );
    }
    y += 50.0;

    if y >= content_top && y + 14.0 <= content_bottom {
        draw_ui_text("STAFF", content_x, y, 14.0, colors::TEXT_DIM());
    }
    y += 25.0;

    let mut staff_count = 0;
    let mut staff_types: Vec<_> = config.economy.staff_costs.keys().collect();
    staff_types.sort();

    for staff_type in staff_types {
        let Some(cost) = config.economy.staff_costs.get(staff_type) else {
            continue;
        };
        let flag = format!("staff_{}", staff_type);

        if building.flags.contains(&flag) {
            let mut chars = staff_type.chars();
            let label = match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            };

            let portrait_rect = Rect::new(content_x, y - 7.0, 38.0, 38.0);
            let has_portrait = draw_staff_portrait(staff_type, portrait_rect, assets);
            let staff_x = if has_portrait {
                portrait_rect.right() + 8.0
            } else {
                content_x
            };
            if y >= content_top && y + 16.0 <= content_bottom {
                draw_ui_text(
                    &format!("{} (${}/mo)", label, cost),
                    staff_x,
                    y,
                    16.0,
                    colors::TEXT(),
                );
            }
            y += 20.0;
            let benefit = match staff_type.as_str() {
                "janitor" => format!(
                    "Maintains {} units against monthly wear",
                    config.staff_effects.janitor_units_maintained
                ),
                "security" => format!(
                    "+{} happiness; {}% fewer emergencies",
                    config.staff_effects.security_happiness_bonus,
                    config.staff_effects.security_failure_reduction_percent
                ),
                "manager" => format!(
                    "+{} happiness; handles resident requests",
                    config.staff_effects.manager_happiness_bonus
                ),
                "receptionist" => format!(
                    "{:.2}× applicant volume",
                    config.staff_effects.receptionist_application_multiplier
                ),
                _ => String::new(),
            };
            if !benefit.is_empty() {
                if y >= content_top && y + 13.0 <= content_bottom {
                    draw_ui_text(&benefit, staff_x, y, 13.0, colors::TEXT_DIM());
                }
                y += 20.0;
            }
            if has_portrait {
                y = y.max(portrait_rect.bottom() + 6.0);
            }
            staff_count += 1;
        }
    }

    if staff_count == 0 {
        if y >= content_top && y + 16.0 <= content_bottom {
            draw_ui_text("None hired", content_x, y, 16.0, colors::TEXT_DIM());
        }
        y += 25.0;
    }

    y += 25.0;

    let available =
        crate::building::upgrades::available_building_upgrades(building, &config.upgrades);

    let mut staff_actions = Vec::new();
    let mut other_actions = Vec::new();

    for upgrade in available {
        let is_staff = match &upgrade {
            crate::building::upgrades::UpgradeAction::Apply { upgrade_id, .. } => {
                if let Some(def) = config.upgrades.get(upgrade_id) {
                    def.effects.iter().any(|effect| match effect {
                        crate::data::config::UpgradeEffect::SetFlag(flag)
                        | crate::data::config::UpgradeEffect::RemoveFlag(flag) => {
                            flag.starts_with("staff_")
                        }
                        _ => false,
                    })
                } else {
                    false
                }
            }
            _ => false,
        };

        if is_staff {
            staff_actions.push(upgrade);
        } else {
            other_actions.push(upgrade);
        }
    }

    staff_actions.sort_by(|a, b| {
        let a_id = match a {
            crate::building::upgrades::UpgradeAction::Apply { upgrade_id, .. } => upgrade_id,
            _ => "",
        };
        let b_id = match b {
            crate::building::upgrades::UpgradeAction::Apply { upgrade_id, .. } => upgrade_id,
            _ => "",
        };

        let a_is_fire = a_id.contains("fire");
        let b_is_fire = b_id.contains("fire");

        if a_is_fire && !b_is_fire {
            std::cmp::Ordering::Greater
        } else if !a_is_fire && b_is_fire {
            std::cmp::Ordering::Less
        } else {
            a_id.cmp(b_id)
        }
    });

    let btn_w = panel_w - 30.0;

    for upgrade in staff_actions {
        if let Some(cost) = upgrade.cost(building, &config.economy, &config.upgrades) {
            let can_afford = money >= cost;
            let action_label = upgrade.label(building, &config.ui, &config.upgrades);
            let monthly_cost = match &upgrade {
                crate::building::upgrades::UpgradeAction::Apply { upgrade_id, .. } => upgrade_id
                    .strip_prefix("hire_")
                    .and_then(|role| config.economy.staff_costs.get(role))
                    .copied(),
                _ => None,
            };
            let label = monthly_cost.map_or_else(
                || action_label.clone(),
                |monthly| format!("{} (${} / month)", action_label, monthly),
            );

            if y >= content_top
                && y + 40.0 <= content_bottom
                && button(content_x, y, btn_w, 40.0, &label, can_afford)
            {
                action = Some(UiAction::UpgradeAction(upgrade));
            }
            y += 44.0;
        }
    }

    if y >= content_top && y + 14.0 <= content_bottom {
        draw_ui_text("UPGRADES", content_x, y, 14.0, colors::TEXT_DIM());
    }
    y += 25.0;

    for upgrade in other_actions {
        if let Some(cost) = upgrade.cost(building, &config.economy, &config.upgrades) {
            let can_afford = money >= cost;
            let label = format!(
                "{} (${})",
                upgrade.label(building, &config.ui, &config.upgrades),
                cost
            );

            if y >= content_top
                && y + 40.0 <= content_bottom
                && button(content_x, y, btn_w, 40.0, &label, can_afford)
            {
                action = Some(UiAction::UpgradeAction(upgrade));
            }
            y += 44.0;
        }
    }

    let content_height = (y + new_scroll) - (panel_y + 50.0);
    let visible_height = content_bottom - (panel_y + 50.0);
    let max_scroll = (content_height - visible_height).max(0.0);
    new_scroll = new_scroll.min(max_scroll);
    if max_scroll > 0.0 {
        new_scroll = super::widgets::scroll_controls(
            Rect::new(content_x, panel_y + panel_h - 48.0, content_w, 40.0),
            new_scroll,
            max_scroll,
            "Hallway summary",
            "Staff & upgrades",
        );
    }
    super::widgets::draw_panel_header(Rect::new(panel_x, panel_y, panel_w, panel_h), "Hallway");

    (action, new_scroll)
}

fn draw_staff_portrait(role: &str, rect: Rect, assets: &AssetManager) -> bool {
    let Some(texture) = assets.get_texture("staff_portraits") else {
        return false;
    };
    let (col, row) = match role {
        "janitor" => (0.0, 0.0),
        "security" => (1.0, 0.0),
        "manager" => (0.0, 1.0),
        "receptionist" => (1.0, 1.0),
        _ => return false,
    };
    let tile_w = texture.width() * 0.5;
    let tile_h = texture.height() * 0.5;
    draw_texture_ex(
        texture,
        rect.x,
        rect.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(rect.w, rect.h)),
            source: Some(Rect::new(col * tile_w, row * tile_h, tile_w, tile_h)),
            ..Default::default()
        },
    );
    true
}

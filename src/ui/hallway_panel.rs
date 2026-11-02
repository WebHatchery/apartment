//! Lobby inspector: condition, appeal and the lobby repair first; building
//! upgrades and staff behind explicit disclosure.

use crate::assets::AssetManager;
use crate::building::upgrades::UpgradeAction;
use crate::building::Building;
use crate::data::config::{GameConfig, UpgradeEffect};
use macroquad::prelude::*;

use super::panel_flow::{PanelFlow, BUTTON_H};
use super::theme::{scale, Tone};
use super::{common::*, Disclosure, PanelDisclosure, UiAction};
use apartment::text::t;
use macroquad_toolkit::ui::draw_ui_text;

pub fn draw_hallway_panel(
    building: &Building,
    money: i32,
    scroll_offset: f32,
    assets: &AssetManager,
    config: &GameConfig,
    disclosure: PanelDisclosure,
) -> (Option<UiAction>, f32) {
    let bounds = layout::context_panel();
    let title = t("inspector.lobby_title");
    panel(bounds.x, bounds.y, bounds.w, bounds.h, &title);
    let mut scroll = scroll_offset;
    if bounds.contains(mouse_position().into()) {
        scroll = (scroll - mouse_wheel().1 * 30.0).max(0.0);
    }
    let mut flow = PanelFlow::new(bounds, scroll);
    let mut action = None;

    let condition = building.hallway_condition;
    flow.kv(
        &t("inspector.condition"),
        &format!("{condition}%"),
        condition_color(condition),
    );
    if flow.roomy() {
        flow.meter(condition, 100, condition_color(condition));
    }
    flow.kv(
        &t("inspector.building_appeal"),
        &building.building_appeal().to_string(),
        colors::ACCENT(),
    );
    flow.paragraph(&t("inspector.lobby_note"), scale::LABEL, colors::TEXT_DIM());

    let (staff_actions, mut other_actions) = split_staff_actions(building, config);
    if let Some(index) = other_actions
        .iter()
        .position(|upgrade| matches!(upgrade, UpgradeAction::RepairHallway { .. }))
    {
        let repair = other_actions.remove(index);
        if let Some(cost) = repair.cost(&config.economy, &config.upgrades) {
            let name = repair.label(&config.ui, &config.upgrades);
            let label = super::apartment_panel::priced_label(&name, cost, money);
            let tone = if condition < config.thresholds.poor_condition {
                Tone::Primary
            } else {
                Tone::Secondary
            };
            if flow.button(&label, money >= cost, tone) {
                action = Some(UiAction::UpgradeAction(repair));
            }
        }
    }
    flow.gap();

    if let Some(upgrade) =
        draw_building_upgrades(&mut flow, money, config, disclosure, other_actions)
    {
        action = Some(upgrade);
    }
    if let Some(staff) = draw_staff(
        &mut flow,
        building,
        money,
        config,
        assets,
        disclosure,
        staff_actions,
    ) {
        action = Some(staff);
    }

    let max_scroll = flow.max_scroll(scroll);
    scroll = scroll.min(max_scroll);
    if max_scroll > 0.0 {
        scroll = super::widgets::scroll_controls(
            Rect::new(flow.x, bounds.bottom() - 48.0, flow.w, 40.0),
            scroll,
            max_scroll,
            "Lobby summary",
            "Upgrades & staff",
        );
    }
    super::widgets::draw_panel_header(bounds, &title);
    (action, scroll)
}

fn draw_building_upgrades(
    flow: &mut PanelFlow,
    money: i32,
    config: &GameConfig,
    disclosure: PanelDisclosure,
    actions: Vec<UpgradeAction>,
) -> Option<UiAction> {
    let upgrades: Vec<_> = actions
        .into_iter()
        .filter_map(|upgrade| {
            let cost = upgrade.cost(&config.economy, &config.upgrades)?;
            Some((upgrade, cost))
        })
        .collect();
    if upgrades.is_empty() {
        return None;
    }
    let open = disclosure.is_open(Disclosure::Improvements);
    let mut action = None;
    if flow.disclosure(
        "Building upgrades",
        &format!("{} available", upgrades.len()),
        open,
    ) {
        action = Some(UiAction::ToggleDisclosure(Disclosure::Improvements));
    }
    if open {
        for (upgrade, cost) in upgrades {
            let label = format!("{} · ${cost}", upgrade.label(&config.ui, &config.upgrades));
            if flow.button(&label, money >= cost, Tone::Secondary) {
                action = Some(UiAction::UpgradeAction(upgrade));
            }
        }
    }
    action
}

fn draw_staff(
    flow: &mut PanelFlow,
    building: &Building,
    money: i32,
    config: &GameConfig,
    assets: &AssetManager,
    disclosure: PanelDisclosure,
    actions: Vec<UpgradeAction>,
) -> Option<UiAction> {
    let hired = hired_staff(building, config);
    let open = disclosure.is_open(Disclosure::Staff);
    let summary = if hired.is_empty() {
        "none hired".to_string()
    } else {
        format!("{} hired", hired.len())
    };
    let mut action = None;
    if flow.disclosure("Staff", &summary, open) {
        action = Some(UiAction::ToggleDisclosure(Disclosure::Staff));
    }
    if !open {
        return action;
    }
    for (role, cost) in &hired {
        draw_staff_row(flow, role, *cost, config, assets);
    }
    for upgrade in actions {
        if let Some(cost) = upgrade.cost(&config.economy, &config.upgrades) {
            let label = staff_label(&upgrade, config);
            if flow.button(&label, money >= cost, Tone::Secondary) {
                action = Some(UiAction::UpgradeAction(upgrade));
            }
        }
    }
    action
}

/// Hiring and firing apart from everything else; hires sort before fires.
fn split_staff_actions(
    building: &Building,
    config: &GameConfig,
) -> (Vec<UpgradeAction>, Vec<UpgradeAction>) {
    let (mut staff, other): (Vec<_>, Vec<_>) =
        crate::building::upgrades::available_building_upgrades(building, &config.upgrades)
            .into_iter()
            .partition(|upgrade| is_staff_action(upgrade, config));
    staff.sort_by_key(|upgrade| {
        let id = upgrade_id(upgrade);
        (id.contains("fire"), id.to_string())
    });
    (staff, other)
}

fn upgrade_id(upgrade: &UpgradeAction) -> &str {
    match upgrade {
        UpgradeAction::Apply { upgrade_id, .. } => upgrade_id,
        _ => "",
    }
}

fn is_staff_action(upgrade: &UpgradeAction, config: &GameConfig) -> bool {
    config.upgrades.get(upgrade_id(upgrade)).is_some_and(|def| {
        def.effects.iter().any(|effect| match effect {
            UpgradeEffect::SetFlag(flag) | UpgradeEffect::RemoveFlag(flag) => {
                flag.starts_with("staff_")
            }
            _ => false,
        })
    })
}

fn hired_staff(building: &Building, config: &GameConfig) -> Vec<(String, i32)> {
    let mut hired: Vec<_> = config
        .economy
        .staff_costs
        .iter()
        .filter(|(role, _)| building.flags.contains(&format!("staff_{role}")))
        .map(|(role, cost)| (role.clone(), *cost))
        .collect();
    hired.sort();
    hired
}

fn staff_label(upgrade: &UpgradeAction, config: &GameConfig) -> String {
    let name = upgrade.label(&config.ui, &config.upgrades);
    match upgrade_id(upgrade)
        .strip_prefix("hire_")
        .and_then(|role| config.economy.staff_costs.get(role))
    {
        Some(monthly) => format!("{name} · ${monthly}/month"),
        None => name,
    }
}

fn draw_staff_row(
    flow: &mut PanelFlow,
    role: &str,
    cost: i32,
    config: &GameConfig,
    assets: &AssetManager,
) {
    if flow.fits(BUTTON_H) {
        let portrait = Rect::new(flow.x, flow.y + 2.0, 38.0, 38.0);
        let text_x = if draw_staff_portrait(role, portrait, assets) {
            portrait.right() + 8.0
        } else {
            flow.x
        };
        let mut name = role.to_string();
        if let Some(first) = name.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        draw_ui_text(
            &format!("{name} · ${cost}/mo"),
            text_x,
            flow.y + 16.0,
            scale::BODY,
            colors::TEXT(),
        );
        draw_ui_text(
            &staff_benefit(role, config),
            text_x,
            flow.y + 34.0,
            scale::CAPTION,
            colors::TEXT_DIM(),
        );
    }
    flow.advance(BUTTON_H + 4.0);
}

fn staff_benefit(role: &str, config: &GameConfig) -> String {
    let effects = &config.staff_effects;
    match role {
        "janitor" => format!(
            "Maintains {} units against monthly wear",
            effects.janitor_units_maintained
        ),
        "security" => format!(
            "+{} happiness; {}% fewer emergencies",
            effects.security_happiness_bonus, effects.security_failure_reduction_percent
        ),
        "manager" => format!(
            "+{} happiness; handles resident requests",
            effects.manager_happiness_bonus
        ),
        "receptionist" => format!(
            "{:.2}x applicant volume",
            effects.receptionist_application_multiplier
        ),
        _ => String::new(),
    }
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

//! Unit inspector: identity, condition, rent and the current decision first;
//! resident detail, unit facts and improvements behind explicit disclosure.

use crate::assets::AssetManager;
use crate::building::{Apartment, Building, UpgradeAction};
use crate::consequences::TenantNetwork;
use crate::data::config::GameConfig;
use crate::narrative::TenantStory;
use crate::tenant::Tenant;
use macroquad::prelude::*;
use std::collections::HashMap;

use super::apartment_panel_sections::{
    draw_improvements, draw_sold_condo_panel, draw_unit_details,
};
use super::panel_flow::PanelFlow;
use super::tenant_panel::{
    draw_pending_request, draw_resident_details, draw_resident_identity, draw_vacancy_actions,
};
use super::theme::{scale, Tone};
use super::{common::*, Disclosure, PanelDisclosure, UiAction};

/// Everything the unit inspector reads besides the unit itself.
pub struct UnitPanelContext<'a> {
    pub building: &'a Building,
    pub tenants: &'a [Tenant],
    pub money: i32,
    pub assets: &'a AssetManager,
    pub config: &'a GameConfig,
    pub network: &'a TenantNetwork,
    pub stories: &'a HashMap<u32, TenantStory>,
    pub disclosure: PanelDisclosure,
}

pub fn draw_apartment_panel(
    apt: &Apartment,
    ctx: &UnitPanelContext,
    scroll_offset: f32,
) -> (Option<UiAction>, f32) {
    let bounds = layout::context_panel();
    if ctx.building.is_unit_sold(apt.id) {
        return (
            draw_sold_condo_panel(apt, ctx.building, ctx.money, bounds),
            0.0,
        );
    }

    let title = format!("Unit {}", apt.unit_number);
    panel(bounds.x, bounds.y, bounds.w, bounds.h, &title);
    let mut scroll = scroll_offset;
    if bounds.contains(mouse_position().into()) {
        scroll = (scroll - mouse_wheel().1 * 30.0).max(0.0);
    }

    let mut flow = PanelFlow::new(bounds, scroll);
    let mut action = None;
    let tenant = apt
        .tenant_id
        .and_then(|id| ctx.tenants.iter().find(|tenant| tenant.id == id));

    // Who (or nobody) lives here, then the room's state and price.
    match tenant {
        Some(tenant) => draw_resident_identity(&mut flow, tenant, ctx.network, ctx.assets),
        None => flow.line(
            if apt.is_listed_for_lease {
                "Vacant · listed for lease"
            } else {
                "Vacant · off the market"
            },
            scale::HEADING,
            colors::WARNING(),
        ),
    }
    flow.gap();
    flow.kv(
        "Condition",
        &condition_issue(apt.condition, ctx.config),
        condition_color(apt.condition),
    );
    if flow.roomy() {
        flow.meter(apt.condition, 100, condition_color(apt.condition));
    }
    match tenant {
        Some(tenant) => flow.kv(
            "Rent",
            &format!("${}/mo · limit ${}", apt.rent_price, tenant.rent_tolerance),
            if apt.rent_price <= tenant.rent_tolerance {
                colors::POSITIVE()
            } else {
                colors::WARNING()
            },
        ),
        None => flow.kv(
            "Asking rent",
            &format!("${}/mo", apt.rent_price),
            colors::PRIMARY(),
        ),
    }

    // A poor room's repair is the decision; otherwise the resident's request
    // or the vacancy controls come first and routine repair follows quietly.
    let urgent = apt.condition < ctx.config.thresholds.poor_condition;
    if urgent {
        action = action.or(draw_repair(&mut flow, apt, ctx, Tone::Primary));
    }
    let decision = match tenant {
        Some(tenant) => {
            flow.kv(
                "Happiness",
                &format!("{}%", tenant.happiness),
                happiness_color(tenant.happiness),
            );
            draw_pending_request(&mut flow, tenant, ctx.stories)
        }
        None => draw_vacancy_actions(&mut flow, apt),
    };
    action = action.or(decision);
    if !urgent {
        action = action.or(draw_repair(&mut flow, apt, ctx, Tone::Secondary));
    }

    flow.gap();
    if let Some(tenant) = tenant {
        let open = ctx.disclosure.is_open(Disclosure::ResidentDetails);
        if flow.disclosure("Resident", tenant.archetype.name(), open) {
            action = Some(UiAction::ToggleDisclosure(Disclosure::ResidentDetails));
        }
        if open {
            draw_resident_details(&mut flow, tenant, ctx.stories);
        }
    }
    let open = ctx.disclosure.is_open(Disclosure::UnitDetails);
    if flow.disclosure("Unit details", &format!("{:?}", apt.design), open) {
        action = Some(UiAction::ToggleDisclosure(Disclosure::UnitDetails));
    }
    if open {
        draw_unit_details(&mut flow, apt, ctx.building);
    }
    if let Some(upgrade) = draw_improvements(&mut flow, apt, ctx) {
        action = Some(upgrade);
    }

    let max_scroll = flow.max_scroll(scroll);
    scroll = scroll.min(max_scroll);
    if max_scroll > 0.0 {
        scroll = super::widgets::scroll_controls(
            Rect::new(flow.x, bounds.bottom() - 48.0, flow.w, 40.0),
            scroll,
            max_scroll,
            "Unit overview",
            "More details",
        );
    }
    super::widgets::draw_panel_header(bounds, &title);
    (action, scroll)
}

/// Condition plus the plain consequence of letting it slide.
fn condition_issue(condition: i32, config: &GameConfig) -> String {
    let issue = if condition < config.thresholds.critical_condition {
        "critical, residents may leave"
    } else if condition < config.thresholds.poor_condition {
        "poor, needs repair"
    } else if condition < config.ui_thresholds.condition_good {
        "worn"
    } else {
        "good"
    };
    format!("{condition}% · {issue}")
}

fn repair_option(apt: &Apartment, ctx: &UnitPanelContext) -> Option<(UpgradeAction, i32)> {
    crate::building::upgrades::available_apartment_upgrades(apt, &ctx.config.upgrades)
        .into_iter()
        .find(|upgrade| matches!(upgrade, UpgradeAction::RepairApartment { .. }))
        .and_then(|repair| {
            let cost = repair.cost(&ctx.config.economy, &ctx.config.upgrades)?;
            Some((repair, cost))
        })
}

/// The repair lives here only: the one action wear on this room asks for.
fn draw_repair(
    flow: &mut PanelFlow,
    apt: &Apartment,
    ctx: &UnitPanelContext,
    tone: Tone,
) -> Option<UiAction> {
    let (repair, cost) = repair_option(apt, ctx)?;
    let label = repair_label(&repair, cost, ctx);
    flow.button(&label, ctx.money >= cost, tone)
        .then_some(UiAction::UpgradeAction(repair))
}

/// Price and, when short, exactly how much more is needed.
fn repair_label(repair: &UpgradeAction, cost: i32, ctx: &UnitPanelContext) -> String {
    let name = repair.label(&ctx.config.ui, &ctx.config.upgrades);
    if ctx.money >= cost {
        format!("{name} · ${cost}")
    } else {
        format!("{name} · ${cost} (need ${} more)", cost - ctx.money)
    }
}

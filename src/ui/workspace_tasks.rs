//! Mission and resident-request workspace.

use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, format_money, truncate_text_to_width};

use crate::assets::AssetManager;
use crate::narrative::{MissionGoal, MissionReward, MissionStatus, TenantRequest};
use crate::state::GameplayState;

use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, draw_card, draw_panel, line_height};
use super::UiAction;

fn content_rect() -> Rect {
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

pub fn draw_tasks_view(state: &GameplayState, assets: &AssetManager) -> Option<UiAction> {
    let rect = content_rect();
    let requests: Vec<_> = state
        .tenant_stories
        .iter()
        .filter_map(|(tenant_id, story)| {
            story
                .pending_request
                .as_ref()
                .map(|request| (*tenant_id, request))
        })
        .collect();
    let available = state
        .missions
        .missions
        .iter()
        .filter(|mission| mission.status == MissionStatus::Available)
        .count();
    let active = state
        .missions
        .missions
        .iter()
        .filter(|mission| mission.status == MissionStatus::Active)
        .count();
    draw_ui_text(
        "Tasks",
        rect.x,
        rect.y + scale::TITLE,
        scale::TITLE,
        color::TEXT_BRIGHT(),
    );
    draw_ui_text(
        &format!(
            "{} active · {} available · {} tenant requests",
            active,
            available,
            requests.len()
        ),
        rect.x,
        rect.y + scale::TITLE + line_height(scale::BODY),
        scale::BODY,
        color::TEXT_DIM(),
    );
    let body_y = rect.y + scale::TITLE + line_height(scale::BODY) + space::LG;
    let gap = space::LG;
    let mission_rect = Rect::new(rect.x, body_y, rect.w * 0.62, rect.bottom() - body_y);
    let request_rect = Rect::new(
        mission_rect.right() + gap,
        body_y,
        rect.right() - mission_rect.right() - gap,
        mission_rect.h,
    );
    let missions = draw_panel(mission_rect, "Missions");
    let mut y = missions.y;
    for mission in state
        .missions
        .missions
        .iter()
        .filter(|mission| {
            matches!(
                mission.status,
                MissionStatus::Active | MissionStatus::Available
            )
        })
        .take(5)
    {
        let card_h = 76.0;
        draw_card(
            Rect::new(missions.x, y, missions.w, card_h),
            mission.status == MissionStatus::Active,
        );
        draw_ui_text(
            &truncate_text_to_width(&mission.title, missions.w - 130.0, scale::BODY),
            missions.x + space::MD,
            y + 22.0,
            scale::BODY,
            color::TEXT_BRIGHT(),
        );
        let progress = mission_progress(state, &mission.goal);
        let reward = reward_text(&mission.reward);
        draw_ui_text(
            &truncate_text_to_width(
                &format!("{} · Reward: {}", progress, reward),
                missions.w - space::MD * 2.0,
                scale::LABEL,
            ),
            missions.x + space::MD,
            y + 45.0,
            scale::LABEL,
            color::TEXT_DIM(),
        );
        draw_ui_text(
            &truncate_text_to_width(&mission.description, missions.w - 130.0, scale::CAPTION),
            missions.x + space::MD,
            y + 64.0,
            scale::CAPTION,
            color::TEXT_DIM(),
        );
        if mission.status == MissionStatus::Available
            && button_at(
                Rect::new(missions.right() - 104.0, y + 18.0, 92.0, 40.0),
                "Accept",
                true,
                Tone::Primary,
            )
        {
            return Some(UiAction::AcceptMission {
                mission_id: mission.id,
            });
        }
        y += card_h + space::SM;
    }
    let request_panel = draw_panel(request_rect, "Tenant requests");
    let mut ry = request_panel.y;
    if requests.is_empty() {
        draw_ui_text(
            "No requests are waiting.",
            request_panel.x,
            ry + scale::BODY,
            scale::BODY,
            color::TEXT_DIM(),
        );
    }
    for (tenant_id, request) in requests.iter().take(4) {
        let tenant = state.tenants.iter().find(|tenant| tenant.id == *tenant_id);
        let tenant_name = tenant.map_or("Former tenant", |tenant| tenant.name.as_str());
        let portrait_size = 44.0;
        let text_x = if let Some(tenant) = tenant {
            super::resident_sprite::draw_face_portrait(
                tenant,
                Rect::new(request_panel.x, ry, portrait_size, portrait_size),
                assets,
            );
            request_panel.x + portrait_size + space::SM
        } else {
            request_panel.x
        };
        draw_ui_text(
            &truncate_text_to_width(tenant_name, request_panel.right() - text_x, scale::BODY),
            text_x,
            ry + scale::BODY,
            scale::BODY,
            color::TEXT_BRIGHT(),
        );
        ry += line_height(scale::BODY);
        draw_ui_text(
            &truncate_text_to_width(
                &request_text(request),
                request_panel.right() - text_x,
                scale::LABEL,
            ),
            text_x,
            ry + scale::LABEL,
            scale::LABEL,
            color::TEXT_DIM(),
        );
        ry = (ry + line_height(scale::LABEL) + space::SM)
            .max(request_panel.y + portrait_size + space::SM);
        let bw = (request_panel.w - space::SM) / 2.0;
        if button_at(
            Rect::new(request_panel.x, ry, bw, 40.0),
            "Approve",
            true,
            Tone::Positive,
        ) {
            return Some(UiAction::ApproveRequest {
                tenant_id: *tenant_id,
            });
        }
        if button_at(
            Rect::new(request_panel.x + bw + space::SM, ry, bw, 40.0),
            "Deny",
            true,
            Tone::Danger,
        ) {
            return Some(UiAction::DenyRequest {
                tenant_id: *tenant_id,
            });
        }
        ry += 44.0;
    }
    None
}

fn mission_progress(state: &GameplayState, goal: &MissionGoal) -> String {
    match goal {
        MissionGoal::HouseTenants { count, archetype } => {
            let current = state
                .tenants
                .iter()
                .filter(|tenant| {
                    tenant.building_id == state.active_building_id()
                        && archetype
                            .as_ref()
                            .is_none_or(|kind| tenant.archetype.name() == kind)
                })
                .count();
            format!("Residents {}/{}", current, count)
        }
        MissionGoal::ReachOccupancy { percentage } => {
            let occupancy = if state.building.rental_unit_count() == 0 {
                0.0
            } else {
                state.building.occupancy_count() as f32 / state.building.rental_unit_count() as f32
                    * 100.0
            };
            format!("Occupancy {:.0}% / {:.0}%", occupancy, percentage * 100.0)
        }
        MissionGoal::MaintainHappiness {
            threshold,
            months,
            current_months,
        } => format!(
            "Happiness ≥{:.0}: {}/{} months",
            threshold, current_months, months
        ),
        MissionGoal::PerfectCollection {
            months,
            current_months,
        } => format!("Perfect collection {}/{} months", current_months, months),
        MissionGoal::FullRepair { building_id } => {
            format!("Repair building #{} to 90+", building_id + 1)
        }
        MissionGoal::AcquireBuilding => format!("Buildings owned {}/2", state.city.buildings.len()),
    }
}

fn reward_text(reward: &MissionReward) -> String {
    match reward {
        MissionReward::Money(amount) => format_money(*amount as i64),
        MissionReward::TaxBreak { months, percentage } => {
            format!("{:.0}% tax break for {}m", percentage * 100.0, months)
        }
        MissionReward::Reputation(amount) => format!("{:+} reputation", amount),
        MissionReward::UnlockBuilding(_) => "new property".to_string(),
    }
}

fn request_text(request: &TenantRequest) -> String {
    match request {
        TenantRequest::Pet { pet_type } => format!("Permission to keep a {}", pet_type),
        TenantRequest::TemporaryGuest {
            guest_name,
            duration_months,
        } => format!("{} staying for {} month(s)", guest_name, duration_months),
        TenantRequest::HomeBusiness { business_type } => {
            format!("Run a {} from home", business_type)
        }
        TenantRequest::Modification { description } => format!("Modify the unit: {}", description),
        TenantRequest::Sublease => "Permission to sublease".to_string(),
    }
}

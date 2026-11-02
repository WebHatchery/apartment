//! Single-column Tasks layout for narrow and portrait canvases.

use super::*;

pub(super) fn draw_tasks_narrow(
    state: &mut GameplayState,
    assets: &AssetManager,
    rect: Rect,
) -> Option<UiAction> {
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
    let missions: Vec<_> = state
        .missions
        .missions
        .iter()
        .filter(|mission| {
            matches!(
                mission.status,
                MissionStatus::Active | MissionStatus::Available
            )
        })
        .collect();
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
            missions
                .iter()
                .filter(|mission| mission.status == MissionStatus::Active)
                .count(),
            missions
                .iter()
                .filter(|mission| mission.status == MissionStatus::Available)
                .count(),
            requests.len()
        ),
        rect.x,
        rect.y + scale::TITLE + line_height(scale::LABEL),
        scale::LABEL,
        color::TEXT_DIM(),
    );
    let body_y = rect.y + scale::TITLE + line_height(scale::LABEL) + space::LG;
    let body = Rect::new(rect.x, body_y, rect.w, (rect.bottom() - body_y).max(1.0));
    let gap = space::LG;
    let mission_h = 340.0;
    let request_h = 290.0;
    let requests_y = body.y + mission_h + gap;
    let content_h = mission_h + gap + request_h;
    let controls_h = 46.0;
    let viewport_h = (body.h - if content_h > body.h { controls_h } else { 0.0 }).max(1.0);
    let viewport = Rect::new(body.x, body.y, body.w, viewport_h);
    let max_scroll = (content_h - viewport.h).max(0.0);
    let mut scroll = state.tasks_scroll_offset.clamp(0.0, max_scroll);
    if viewport.contains(vec2(mouse_position().0, mouse_position().1)) {
        scroll = (scroll - mouse_wheel().1 * 80.0).clamp(0.0, max_scroll);
    }
    let mut action = None;
    let ui = macroquad_toolkit::ui::VirtualUi::new(screen_width(), screen_height());
    ui.with_clip(viewport, || {
        if let Some(next) = draw_narrow_missions(
            state,
            missions.as_slice(),
            Rect::new(body.x, body.y - scroll, body.w, mission_h),
        ) {
            action = Some(next);
        }
        if action.is_none() {
            action = draw_narrow_requests(
                state,
                assets,
                requests.as_slice(),
                Rect::new(body.x, requests_y - scroll, body.w, request_h),
            );
        }
    });
    if max_scroll > 0.0 {
        state.tasks_scroll_offset = crate::ui::widgets::scroll_controls(
            Rect::new(body.x, body.bottom() - controls_h + space::XS, body.w, 38.0),
            scroll,
            max_scroll,
            "Earlier",
            "Later",
        );
    }
    action
}

fn draw_narrow_missions(
    state: &GameplayState,
    missions: &[&crate::narrative::missions::Mission],
    rect: Rect,
) -> Option<UiAction> {
    let inner = draw_panel(rect, "Missions · accept or track");
    let card_h = 120.0;
    let gap = space::SM;
    let pager_h = 44.0;
    let page_size = (((inner.h - pager_h) / (card_h + gap)).floor() as usize).max(1);
    let page_count = missions.len().max(1).div_ceil(page_size);
    let page = state.tasks_page.min(page_count - 1);
    let mut y = inner.y;
    for mission in missions.iter().skip(page * page_size).take(page_size) {
        let card = Rect::new(inner.x, y, inner.w, card_h);
        draw_card(card, mission.status == MissionStatus::Active);
        let text_w = (card.w - 116.0).max(80.0);
        let mut text_y = card.y + space::SM;
        for line in wrap(&mission.title, text_w, scale::BODY).iter().take(2) {
            draw_ui_text(
                line,
                card.x + space::MD,
                text_y + scale::BODY,
                scale::BODY,
                color::TEXT_BRIGHT(),
            );
            text_y += line_height(scale::BODY);
        }
        let progress = format!(
            "{} · Reward: {}",
            mission_progress(state, &mission.goal),
            reward_text(&mission.reward)
        );
        draw_ui_text(
            &progress,
            card.x + space::MD,
            card.y + 66.0,
            scale::LABEL,
            color::TEXT_DIM(),
        );
        for (index, line) in wrap(&mission.description, text_w, scale::CAPTION)
            .iter()
            .take(2)
            .enumerate()
        {
            draw_ui_text(
                line,
                card.x + space::MD,
                card.y + 88.0 + index as f32 * line_height(scale::CAPTION),
                scale::CAPTION,
                color::TEXT_DIM(),
            );
        }
        if mission.status == MissionStatus::Available
            && button_at(
                Rect::new(card.right() - 104.0, card.y + 16.0, 92.0, 40.0),
                "Accept",
                true,
                Tone::Primary,
            )
        {
            return Some(UiAction::AcceptMission {
                mission_id: mission.id,
            });
        }
        y += card_h + gap;
    }
    if page_count > 1 {
        let y = inner.bottom() - 40.0;
        let button_w = (inner.w - gap) / 2.0;
        if button_at(
            Rect::new(inner.x, y, button_w, 40.0),
            "Previous mission",
            page > 0,
            Tone::Secondary,
        ) {
            return Some(UiAction::SetTasksPage { page: page - 1 });
        }
        if button_at(
            Rect::new(inner.right() - button_w, y, button_w, 40.0),
            "Next mission",
            page + 1 < page_count,
            Tone::Primary,
        ) {
            return Some(UiAction::SetTasksPage { page: page + 1 });
        }
    }
    None
}

fn draw_narrow_requests(
    state: &GameplayState,
    assets: &AssetManager,
    requests: &[(u32, &TenantRequest)],
    rect: Rect,
) -> Option<UiAction> {
    let inner = draw_panel(rect, "Tenant requests · decide");
    if requests.is_empty() {
        draw_ui_text(
            "No requests are waiting.",
            inner.x,
            inner.y + scale::BODY,
            scale::BODY,
            color::TEXT_DIM(),
        );
        return None;
    }
    let page_count = requests.len();
    let page = state.requests_page.min(page_count - 1);
    let (tenant_id, request) = requests[page];
    let tenant = state.tenants.iter().find(|tenant| tenant.id == tenant_id);
    let mut y = inner.y;
    if let Some(tenant) = tenant {
        crate::ui::resident_sprite::draw_face_portrait(
            tenant,
            Rect::new(inner.x, y, 52.0, 52.0),
            assets,
        );
        draw_ui_text(
            &tenant.name,
            inner.x + 64.0,
            y + scale::BODY,
            scale::BODY,
            color::TEXT_BRIGHT(),
        );
        y += 60.0;
    }
    for line in wrap(&request_text(request), inner.w, scale::BODY) {
        draw_ui_text(&line, inner.x, y + scale::BODY, scale::BODY, color::TEXT());
        y += line_height(scale::BODY);
    }
    for line in wrap(&request_outcomes(request), inner.w, scale::LABEL) {
        draw_ui_text(
            &line,
            inner.x,
            y + scale::LABEL,
            scale::LABEL,
            color::WARNING(),
        );
        y += line_height(scale::LABEL);
    }
    let button_y = inner.bottom() - 40.0;
    let button_w = (inner.w - space::SM) / 2.0;
    if button_at(
        Rect::new(inner.x, button_y, button_w, 40.0),
        "Approve",
        true,
        Tone::Positive,
    ) {
        return Some(UiAction::ApproveRequest { tenant_id });
    }
    if button_at(
        Rect::new(inner.right() - button_w, button_y, button_w, 40.0),
        "Deny",
        true,
        Tone::Danger,
    ) {
        return Some(UiAction::DenyRequest { tenant_id });
    }
    if page_count > 1 {
        let pager_y = button_y - 48.0;
        if button_at(
            Rect::new(inner.x, pager_y, button_w, 40.0),
            "Previous request",
            page > 0,
            Tone::Secondary,
        ) {
            return Some(UiAction::SetRequestsPage { page: page - 1 });
        }
        if button_at(
            Rect::new(inner.right() - button_w, pager_y, button_w, 40.0),
            "Next request",
            page + 1 < page_count,
            Tone::Primary,
        ) {
            return Some(UiAction::SetRequestsPage { page: page + 1 });
        }
    }
    None
}

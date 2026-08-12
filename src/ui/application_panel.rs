use super::{common::*, UiAction};
use crate::assets::AssetManager;
use crate::building::Building;
use crate::tenant::TenantApplication;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub fn draw_application_panel(
    applications: &[TenantApplication],
    building_id: u32,
    building: &Building,
    filter_apartment_id: Option<u32>,
    requested_page: usize,
    offset_x: f32,
    assets: &AssetManager,
) -> Option<UiAction> {
    let panel_rect = application_panel_rect(offset_x)?;
    panel(
        panel_rect.x,
        panel_rect.y,
        panel_rect.w,
        panel_rect.h,
        "Applications",
    );

    let content_x = panel_rect.x + 15.0;
    let mut y = panel_rect.y + 50.0;
    let filtered_apps: Vec<(usize, &TenantApplication)> = applications
        .iter()
        .enumerate()
        .filter(|(_, app)| {
            app.building_id == building_id
                && filter_apartment_id.is_none_or(|id| app.apartment_id == id)
        })
        .collect();

    if filtered_apps.is_empty() {
        draw_empty_applications(content_x, y, filter_apartment_id);
        return None;
    }

    draw_ui_text(
        &format!("{} pending", filtered_apps.len()),
        content_x,
        y,
        16.0,
        colors::TEXT_DIM(),
    );
    y += 25.0;

    let available_h = panel_rect.bottom() - y - 50.0;
    let action_width = panel_rect.w - 46.0;
    let columns = if action_width >= 4.0 * 54.0 + 3.0 * 6.0 {
        4
    } else {
        2
    };
    let card_h = 88.0 + 4_usize.div_ceil(columns) as f32 * 46.0 + 4.0;
    let page_size = ((available_h / (card_h + 12.0)).floor() as usize).max(1);
    let page_count = filtered_apps.len().div_ceil(page_size).max(1);
    let page = requested_page.min(page_count - 1);
    let mut action = None;
    for (index, application) in filtered_apps
        .iter()
        .skip(page * page_size)
        .take(page_size)
        .copied()
    {
        let (card_action, card_h) = draw_application_card(
            index,
            application,
            building,
            content_x,
            y,
            panel_rect.w - 30.0,
            assets,
        );
        if card_action.is_some() {
            action = card_action;
        }
        y += card_h + 12.0;
    }
    if page_count > 1 {
        use crate::ui::theme::Tone;
        use crate::ui::widgets::button_at;
        let controls_y = panel_rect.bottom() - 48.0;
        let gap = 8.0;
        let button_w = (panel_rect.w - 30.0 - gap) * 0.5;
        if page > 0
            && button_at(
                Rect::new(content_x, controls_y, button_w, 40.0),
                "Previous applicant",
                true,
                Tone::Secondary,
            )
        {
            action = Some(UiAction::SetApplicationPage { page: page - 1 });
        }
        if page + 1 < page_count
            && button_at(
                Rect::new(content_x + button_w + gap, controls_y, button_w, 40.0),
                "Next applicant",
                true,
                Tone::Primary,
            )
        {
            action = Some(UiAction::SetApplicationPage { page: page + 1 });
        }
    }

    action
}

fn application_panel_rect(offset_x: f32) -> Option<Rect> {
    let panel_x = screen_width() * layout::PANEL_SPLIT() + layout::PADDING() + offset_x;
    if panel_x > screen_width() {
        return None;
    }

    Some(Rect::new(
        panel_x,
        layout::HEADER_HEIGHT() + layout::PADDING(),
        screen_width() * (1.0 - layout::PANEL_SPLIT()) - layout::PADDING() * 2.0,
        screen_height()
            - layout::HEADER_HEIGHT()
            - layout::FOOTER_HEIGHT()
            - layout::PADDING() * 2.0,
    ))
}

fn draw_empty_applications(content_x: f32, y: f32, filter_apartment_id: Option<u32>) {
    if filter_apartment_id.is_some() {
        draw_ui_text(
            "No applications for this unit",
            content_x,
            y,
            18.0,
            colors::TEXT_DIM(),
        );
        return;
    }

    draw_ui_text(
        "No pending applications",
        content_x,
        y,
        18.0,
        colors::TEXT_DIM(),
    );
    draw_ui_text(
        "List apartments for lease, then End Month!",
        content_x,
        y + 25.0,
        14.0,
        colors::TEXT_DIM(),
    );
}

/// Draw one application card. Returns the chosen action (if any) and the card
/// height, which grows when the action buttons wrap to a second row on narrow
/// panels — so cards never overlap.
fn draw_application_card(
    index: usize,
    application: &TenantApplication,
    building: &Building,
    x: f32,
    y: f32,
    width: f32,
    assets: &AssetManager,
) -> (Option<UiAction>, f32) {
    use crate::ui::theme::Tone;
    use crate::ui::widgets::button_at;

    let compact = width < 260.0;
    let portrait_size = if compact { 60.0 } else { 68.0 };
    let text_x = x + portrait_size + if compact { 16.0 } else { 20.0 };

    let btn_y = y + 88.0;
    let bh = 40.0;
    let gap = 6.0;
    let button_left = x + 8.0;
    let right = x + width - 8.0;

    // Actions use the full card width rather than starting after the portrait.
    // That keeps two applicant cards fully reachable at the 800x600 target.
    let cols = if right - button_left >= 4.0 * 54.0 + 3.0 * gap {
        4
    } else {
        2
    };
    let rows = 4_usize.div_ceil(cols);
    let bw = ((right - button_left) - (cols - 1) as f32 * gap) / cols as f32;
    let card_h = 88.0 + rows as f32 * (bh + gap) + 4.0;

    // Card frame (sized to fit the buttons), then portrait + content on top.
    crate::ui::widgets::draw_card(Rect::new(x, y, width, card_h), false);
    if !super::resident_sprite::draw_face_portrait(
        &application.tenant,
        Rect::new(x + 8.0, y + 8.0, portrait_size, portrait_size),
        assets,
    ) {
        draw_rectangle(
            x + 8.0,
            y + 8.0,
            4.0,
            portrait_size,
            archetype_color(&application.tenant.archetype),
        );
    }
    draw_application_text(
        application,
        building,
        text_x,
        y,
        x + width - 8.0 - text_x,
        compact,
    );

    let specs: [(&str, bool, Tone, UiAction); 4] = [
        (
            "Accept",
            true,
            Tone::Positive,
            UiAction::AcceptApplication {
                application_index: index,
            },
        ),
        (
            "Reject",
            true,
            Tone::Danger,
            UiAction::RejectApplication {
                application_index: index,
            },
        ),
        (
            if application.revealed_reliability {
                "Credit done"
            } else {
                "Check credit"
            },
            !application.revealed_reliability,
            Tone::Secondary,
            UiAction::CreditCheck {
                application_index: index,
            },
        ),
        (
            if application.revealed_behavior {
                "History done"
            } else {
                "Check history"
            },
            !application.revealed_behavior,
            Tone::Secondary,
            UiAction::BackgroundCheck {
                application_index: index,
            },
        ),
    ];

    let mut action = None;
    for (i, (label, enabled, tone, act)) in specs.into_iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let bx = button_left + col as f32 * (bw + gap);
        let by = btn_y + row as f32 * (bh + gap);
        if button_at(Rect::new(bx, by, bw, bh), label, enabled, tone) {
            action = Some(act);
        }
    }

    (action, card_h)
}

fn draw_application_text(
    application: &TenantApplication,
    building: &Building,
    text_x: f32,
    y: f32,
    text_width: f32,
    compact: bool,
) {
    use macroquad_toolkit::ui::{measure_ui_text, truncate_text_to_width};

    let unit = building
        .get_apartment(application.apartment_id)
        .map(|apartment| format!("Unit {}", apartment.unit_number))
        .unwrap_or_else(|| "Unknown unit".to_string());
    let unit_w = measure_ui_text(&unit, None, 14, 1.0).width;
    if compact {
        draw_ui_text(
            &truncate_text_to_width(&application.tenant.name, text_width, 18.0),
            text_x,
            y + 22.0,
            18.0,
            colors::TEXT(),
        );
        draw_ui_text(&unit, text_x, y + 43.0, 14.0, colors::ACCENT());
        let fit_text = if application.match_result.meets_minimum {
            format!("Qualified · {}%", application.match_result.score)
        } else {
            format!("Stretch · {}%", application.match_result.score)
        };
        let score_color = if application.match_result.score >= 70 {
            colors::POSITIVE()
        } else if application.match_result.score >= 50 {
            colors::ACCENT()
        } else {
            colors::WARNING()
        };
        draw_ui_text(
            &truncate_text_to_width(&fit_text, text_width, 14.0),
            text_x,
            y + 65.0,
            14.0,
            score_color,
        );
        return;
    }
    draw_ui_text(
        &truncate_text_to_width(
            &application.tenant.name,
            (text_width - unit_w - 8.0).max(48.0),
            18.0,
        ),
        text_x,
        y + 22.0,
        18.0,
        colors::TEXT(),
    );
    draw_ui_text(
        &unit,
        text_x + text_width - unit_w,
        y + 21.0,
        14.0,
        colors::ACCENT(),
    );

    let score_color = if application.match_result.score >= 70 {
        colors::POSITIVE()
    } else if application.match_result.score >= 50 {
        colors::ACCENT()
    } else {
        colors::WARNING()
    };
    let fit_text = if application.match_result.meets_minimum {
        format!("Qualified · {}%", application.match_result.score)
    } else {
        format!("Stretch · {}%", application.match_result.score)
    };
    let fit_w = measure_ui_text(&fit_text, None, 14, 1.0).width;
    draw_ui_text(
        application.tenant.archetype.name(),
        text_x,
        y + 43.0,
        14.0,
        colors::TEXT_DIM(),
    );
    draw_ui_text(
        &fit_text,
        text_x + text_width - fit_w,
        y + 43.0,
        14.0,
        score_color,
    );

    let credit_text = if application.revealed_reliability {
        format!("Credit: {}", application.tenant.rent_reliability)
    } else {
        "Credit: ?".to_string()
    };
    draw_ui_text(&credit_text, text_x, y + 67.0, 14.0, colors::TEXT_DIM());

    let background_text = if application.revealed_behavior {
        format!("Behavior: {}", application.tenant.behavior_score)
    } else {
        "Behavior: ?".to_string()
    };
    draw_ui_text(
        &background_text,
        text_x + text_width - measure_ui_text(&background_text, None, 14, 1.0).width,
        y + 67.0,
        14.0,
        colors::TEXT_DIM(),
    );
}

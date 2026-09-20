use crate::assets::AssetManager;
use crate::narrative::events::{NarrativeEvent, NarrativeEventType};
use crate::ui::theme::{color, scale, space, Tone};
use crate::ui::widgets::{self, button_at, draw_panel, line_height, scroll_controls, wrap};
use crate::ui::UiAction;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, VirtualUi};

pub fn draw_event_modal(
    event: &NarrativeEvent,
    assets: &AssetManager,
    scroll_offset: f32,
) -> (Option<UiAction>, f32) {
    let screen_w = screen_width();
    let screen_h = screen_height();

    draw_rectangle(0., 0., screen_w, screen_h, Color::new(0., 0., 0., 0.6));

    let modal_w = (screen_w - space::MD * 2.0).min(680.0);
    let illustration_id = event_texture_id(&event.event_type);
    let has_illustration = crate::ui::event_art::has_event_art(illustration_id, assets);
    let art_size = if has_illustration && modal_w >= 500.0 {
        104.0
    } else if has_illustration && modal_w >= 420.0 {
        72.0
    } else {
        0.0
    };
    let art_gap = if art_size > 0.0 { space::LG } else { 0.0 };
    let content_w = (modal_w - space::PAD * 2.0 - art_size - art_gap).max(80.0);

    let body_lines = wrap(&event.description, content_w, scale::BODY);
    let body_h = (body_lines.len() as f32 * line_height(scale::BODY)).max(art_size);

    let btn_h = 44.0;
    let choice_heights: Vec<(Vec<String>, f32)> = event
        .choices
        .iter()
        .map(|choice| {
            let lines = wrap(
                &choice.description,
                modal_w - space::PAD * 2.0,
                scale::CAPTION,
            );
            let height = lines.len() as f32 * line_height(scale::CAPTION) + space::XS + btn_h;
            (lines, height)
        })
        .collect();
    let choices_h = choice_heights
        .iter()
        .map(|(_, height)| *height)
        .sum::<f32>()
        + space::SM * choice_heights.len().saturating_sub(1) as f32;
    let content_natural_h = if event.choices.is_empty() {
        body_h
    } else {
        body_h + space::LG + choices_h + space::LG
    };

    let header_h = 38.0;
    let static_footer_h = if event.choices.is_empty() {
        btn_h + space::MD
    } else {
        0.0
    };
    let natural_h = header_h + space::SM + content_natural_h + static_footer_h + space::PAD;
    let max_h = (screen_h - space::MD * 2.0).max(160.0);
    let needs_scroll = natural_h > max_h;
    let scroll_footer_h = if needs_scroll { 46.0 } else { 0.0 };
    let footer_h = static_footer_h + scroll_footer_h;
    let viewport_h = if needs_scroll {
        (max_h - header_h - space::SM - footer_h).max(112.0)
    } else {
        content_natural_h
    };
    let modal_h = header_h + space::SM + viewport_h + footer_h;

    let x = (screen_w - modal_w) / 2.0;
    let y = ((screen_h - modal_h) / 2.0).clamp(space::SM, screen_h - modal_h - space::SM);

    let content = draw_panel(Rect::new(x, y, modal_w, modal_h), &event.headline);
    let view_rect = Rect::new(
        content.x,
        content.y,
        content.w,
        (content.h - footer_h).max(1.0),
    );
    let max_scroll = (content_natural_h - view_rect.h).max(0.0);
    let mut next_scroll = scroll_offset.clamp(0.0, max_scroll);
    if view_rect.contains(vec2(mouse_position().0, mouse_position().1)) {
        next_scroll = (next_scroll - mouse_wheel().1 * 80.0).clamp(0.0, max_scroll);
    }

    let mut action = None;
    let ui = VirtualUi::new(screen_w, screen_h);
    ui.with_clip(view_rect, || {
        let origin_y = content.y - next_scroll;
        if art_size > 0.0 {
            crate::ui::event_art::draw_event_art(
                illustration_id,
                Rect::new(content.x, origin_y, art_size, art_size),
                assets,
            );
            draw_rectangle_lines(
                content.x,
                origin_y,
                art_size,
                art_size,
                1.0,
                color::BORDER_STRONG(),
            );
        }
        let text_x = content.x + art_size + art_gap;
        let mut text_y = origin_y;
        for line in &body_lines {
            draw_ui_text(
                line,
                text_x,
                text_y + scale::BODY,
                scale::BODY,
                color::TEXT(),
            );
            text_y += line_height(scale::BODY);
        }

        if event.choices.is_empty() {
            return;
        }

        let mut choice_y = origin_y + body_h + space::LG;
        for (i, choice) in event.choices.iter().enumerate() {
            let (description_lines, _) = &choice_heights[i];
            for line in description_lines {
                draw_ui_text(
                    line,
                    content.x,
                    choice_y + scale::CAPTION,
                    scale::CAPTION,
                    color::TEXT_DIM(),
                );
                choice_y += line_height(scale::CAPTION);
            }
            let rect = Rect::new(content.x, choice_y + space::XS, content.w, btn_h);
            let fully_visible = rect.y >= view_rect.y && rect.bottom() <= view_rect.bottom();
            let label = if choice.reputation_change != 0 {
                format!("{}   (Rep {:+})", choice.label, choice.reputation_change)
            } else {
                choice.label.clone()
            };
            let tone = if choice.reputation_change > 0 {
                Tone::Positive
            } else if choice.reputation_change < 0 {
                Tone::Danger
            } else {
                Tone::Secondary
            };
            if fully_visible && button_at(rect, &label, true, tone) {
                action = Some(UiAction::ResolveEventChoice {
                    event_id: event.id,
                    choice_index: i,
                });
            }
            choice_y = rect.bottom() + space::SM;
        }
    });

    if needs_scroll {
        next_scroll = scroll_controls(
            Rect::new(content.x, content.bottom() - footer_h, content.w, 38.0),
            next_scroll,
            max_scroll,
            "Earlier",
            "Later",
        );
    }

    if event.choices.is_empty() {
        let w = widgets::button_width("Continue", btn_h).max(120.0);
        let rect = Rect::new(content.right() - w, content.bottom() - btn_h, w, btn_h);
        if button_at(rect, "Continue", true, Tone::Primary) {
            action = Some(UiAction::ResolveEventChoice {
                event_id: event.id,
                choice_index: 0,
            });
        }
    }

    (action, next_scroll)
}

fn event_texture_id(event_type: &NarrativeEventType) -> &'static str {
    match event_type {
        NarrativeEventType::NeighborhoodNews => "event_new_business",
        NarrativeEventType::CityEvent | NarrativeEventType::SeasonalEvent => "event_heatwave",
        NarrativeEventType::TenantStory { .. } => "event_tenant_moved_in",
        NarrativeEventType::BuildingMilestone => "event_rent_collected",
        NarrativeEventType::CharacterEncounter => "event_inspection",
        NarrativeEventType::ExternalOffer => "event_developer_offer",
        NarrativeEventType::RelationshipEvent => "event_noise_complaint",
    }
}

#[cfg(test)]
mod tests;

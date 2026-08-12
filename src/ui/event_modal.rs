use crate::assets::AssetManager;
use crate::narrative::events::{NarrativeEvent, NarrativeEventType};
use crate::ui::theme::{color, scale, space, Tone};
use crate::ui::widgets::{self, button_at, draw_panel, line_height, wrap};
use crate::ui::UiAction;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

pub fn draw_event_modal(event: &NarrativeEvent, assets: &AssetManager) -> Option<UiAction> {
    let screen_w = screen_width();
    let screen_h = screen_height();

    draw_rectangle(0., 0., screen_w, screen_h, Color::new(0., 0., 0., 0.6));

    let modal_w = (screen_w * 0.55).clamp(480.0, 680.0);
    let illustration = assets.get_texture(event_texture_id(&event.event_type));
    let art_size = if illustration.is_some() { 104.0 } else { 0.0 };
    let art_gap = if illustration.is_some() {
        space::LG
    } else {
        0.0
    };
    let content_w = modal_w - space::PAD * 2.0 - art_size - art_gap;

    let body_lines = wrap(&event.description, content_w, scale::BODY);
    let body_h = (body_lines.len() as f32 * line_height(scale::BODY)).max(art_size);

    let btn_h = 44.0;
    let choice_h = line_height(scale::CAPTION) + space::XS + btn_h;
    let btn_count = event.choices.len().max(1) as f32;
    let buttons_h = if event.choices.is_empty() {
        btn_h
    } else {
        btn_count * choice_h + (btn_count - 1.0).max(0.0) * space::SM
    };

    let header_h = 38.0;
    let modal_h = header_h + space::SM + body_h + space::LG + buttons_h + space::MD;

    let x = (screen_w - modal_w) / 2.0;
    let y = ((screen_h - modal_h) / 2.0).max(space::XL);

    let content = draw_panel(Rect::new(x, y, modal_w, modal_h), &event.headline);

    if let Some(texture) = illustration {
        draw_texture_ex(
            texture,
            content.x,
            content.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(art_size, art_size)),
                ..Default::default()
            },
        );
        draw_rectangle_lines(
            content.x,
            content.y,
            art_size,
            art_size,
            1.0,
            color::BORDER_STRONG(),
        );
    }
    let text_x = content.x + art_size + art_gap;
    let mut text_y = content.y;
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

    let btn_x = content.x;
    let btn_w = content.w;

    if event.choices.is_empty() {
        let w = widgets::button_width("Continue", btn_h).max(120.0);
        let rect = Rect::new(btn_x + btn_w - w, y + modal_h - space::MD - btn_h, w, btn_h);
        if button_at(rect, "Continue", true, Tone::Primary) {
            return Some(UiAction::ResolveEventChoice {
                event_id: event.id,
                choice_index: 0,
            });
        }
        return None;
    }

    let mut choice_y = content.y + body_h + space::LG;
    for (i, choice) in event.choices.iter().enumerate() {
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

        draw_ui_text(
            &truncate_text_to_width(&choice.description, btn_w, scale::CAPTION),
            btn_x,
            choice_y + scale::CAPTION,
            scale::CAPTION,
            color::TEXT_DIM(),
        );
        let rect = Rect::new(
            btn_x,
            choice_y + line_height(scale::CAPTION) + space::XS,
            btn_w,
            btn_h,
        );
        if button_at(rect, &label, true, tone) {
            return Some(UiAction::ResolveEventChoice {
                event_id: event.id,
                choice_index: i,
            });
        }
        choice_y += choice_h + space::SM;
    }

    None
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

//! Shared 3x3 event-atlas rendering for modals and the activity ledger.

use macroquad::prelude::*;

use crate::assets::AssetManager;

fn event_tile(id: &str) -> Option<(f32, f32)> {
    Some(match id {
        "event_rent_collected" => (0.0, 0.0),
        "event_tenant_moved_in" => (1.0, 0.0),
        "event_tenant_moved_out" => (2.0, 0.0),
        "event_noise_complaint" => (0.0, 1.0),
        "event_pipe_burst" => (1.0, 1.0),
        "event_inspection" => (2.0, 1.0),
        "event_heatwave" => (0.0, 2.0),
        "event_new_business" => (1.0, 2.0),
        "event_developer_offer" => (2.0, 2.0),
        _ => return None,
    })
}

pub fn has_event_art(id: &str, assets: &AssetManager) -> bool {
    (assets.get_texture("event_atlas").is_some() && event_tile(id).is_some())
        || assets.get_texture(id).is_some()
}

pub fn draw_event_art(id: &str, rect: Rect, assets: &AssetManager) -> bool {
    if let (Some(texture), Some((column, row))) =
        (assets.get_texture("event_atlas"), event_tile(id))
    {
        let tile_w = texture.width() / 3.0;
        let tile_h = texture.height() / 3.0;
        draw_texture_ex(
            texture,
            rect.x,
            rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(rect.w, rect.h)),
                source: Some(Rect::new(column * tile_w, row * tile_h, tile_w, tile_h)),
                ..Default::default()
            },
        );
        true
    } else if let Some(texture) = assets.get_texture(id) {
        draw_texture_ex(
            texture,
            rect.x,
            rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(rect.w, rect.h)),
                ..Default::default()
            },
        );
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests;

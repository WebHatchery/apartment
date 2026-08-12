//! Composes an in-room resident from independent pose and emotion atlases.

use crate::assets::AssetManager;
use crate::building::{Apartment, DesignType};
use crate::tenant::Tenant;
use macroquad::prelude::*;

const BODY_COLUMNS: f32 = 3.0;
const FACE_COLUMNS: f32 = 5.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResidentPose {
    Standing,
    Sitting,
    Cooking,
}

impl ResidentPose {
    fn column(self) -> f32 {
        match self {
            Self::Standing => 0.0,
            Self::Sitting => 1.0,
            Self::Cooking => 2.0,
        }
    }
}

fn pose_for(apartment: &Apartment) -> ResidentPose {
    if apartment.kitchen_level > 0 || matches!(apartment.design, DesignType::Practical) {
        ResidentPose::Cooking
    } else if matches!(
        apartment.design,
        DesignType::Cozy | DesignType::Luxury | DesignType::Opulent
    ) {
        ResidentPose::Sitting
    } else {
        ResidentPose::Standing
    }
}

fn emotion_column(happiness: i32) -> f32 {
    match happiness {
        85.. => 0.0,
        70..=84 => 1.0,
        40..=69 => 2.0,
        20..=39 => 3.0,
        _ => 4.0,
    }
}

pub(super) fn draw_resident(
    apartment: &Apartment,
    tenant: &Tenant,
    room: Rect,
    assets: &AssetManager,
) {
    let (Some(bodies), Some(faces)) = (
        assets.get_texture("tenant_body_poses"),
        assets.get_texture("tenant_face_emotions"),
    ) else {
        return;
    };

    let pose = pose_for(apartment);
    let body_column_w = bodies.width() / BODY_COLUMNS;
    let face_column_w = faces.width() / FACE_COLUMNS;
    let sprite_h = (room.h * 0.68).clamp(58.0, 112.0);
    let body_w = sprite_h * 0.72;
    let body_x = room.x + room.w * 0.58 - body_w / 2.0;
    let body_y = room.bottom() - sprite_h - 4.0;

    draw_texture_ex(
        bodies,
        body_x,
        body_y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(body_w, sprite_h)),
            source: Some(Rect::new(
                pose.column() * body_column_w,
                bodies.height() * 0.12,
                body_column_w,
                bodies.height() * 0.76,
            )),
            ..Default::default()
        },
    );

    let head_h = sprite_h * 0.48;
    let head_w = head_h * 0.83;
    let seated_drop = if pose == ResidentPose::Sitting {
        sprite_h * 0.10
    } else {
        0.0
    };
    let head_x = body_x + (body_w - head_w) / 2.0;
    let head_y = body_y - head_h * 0.42 + seated_drop;
    draw_texture_ex(
        faces,
        head_x,
        head_y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(head_w, head_h)),
            source: Some(Rect::new(
                emotion_column(tenant.happiness) * face_column_w,
                faces.height() * 0.24,
                face_column_w,
                faces.height() * 0.48,
            )),
            ..Default::default()
        },
    );
}

#[cfg(test)]
mod tests;

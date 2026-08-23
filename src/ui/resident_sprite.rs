//! Shared modular resident rendering for rooms, profiles, and applicant cards.

use super::common::archetype_color;
use crate::assets::AssetManager;
use crate::building::Apartment;
#[cfg(test)]
use crate::building::DesignType;
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

#[cfg(test)]
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

fn animated_pose(apartment: &Apartment, tenant_id: u32, elapsed_seconds: f64) -> ResidentPose {
    let phase = (elapsed_seconds * 0.45 + f64::from(apartment.id + tenant_id) * 0.37) % 3.0;
    match phase as u32 {
        0 => ResidentPose::Standing,
        1 => ResidentPose::Sitting,
        _ => ResidentPose::Cooking,
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

fn head_drop_for_pose(pose: ResidentPose) -> f32 {
    match pose {
        ResidentPose::Standing => 0.04,
        ResidentPose::Sitting => 0.15,
        ResidentPose::Cooking => 0.06,
    }
}

fn face_texture<'a>(tenant: &Tenant, assets: &'a AssetManager) -> Option<&'a Texture2D> {
    let face_id = if tenant.id.is_multiple_of(2) {
        "tenant_face_emotions"
    } else {
        "tenant_face_emotions_alt"
    };
    assets
        .get_texture(face_id)
        .or_else(|| assets.get_texture("tenant_face_emotions"))
}

pub(super) fn draw_face_portrait(tenant: &Tenant, rect: Rect, assets: &AssetManager) -> bool {
    let Some(faces) = face_texture(tenant, assets) else {
        return false;
    };
    let accent = archetype_color(&tenant.archetype);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(accent.r * 0.28, accent.g * 0.25, accent.b * 0.22, 1.0),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::new(accent.r, accent.g, accent.b, 0.70),
    );
    let face_column_w = faces.width() / FACE_COLUMNS;
    draw_texture_ex(
        faces,
        rect.x + rect.w * 0.06,
        rect.y + rect.h * 0.02,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(rect.w * 0.88, rect.h * 0.96)),
            source: Some(Rect::new(
                emotion_column(tenant.happiness) * face_column_w,
                faces.height() * 0.20,
                face_column_w,
                faces.height() * 0.55,
            )),
            ..Default::default()
        },
    );
    true
}

pub(super) fn draw_resident(
    apartment: &Apartment,
    tenant: &Tenant,
    room: Rect,
    assets: &AssetManager,
) {
    let body_id = if tenant.id.is_multiple_of(2) {
        "tenant_body_poses"
    } else {
        "tenant_body_poses_alt"
    };
    let Some(bodies) = assets
        .get_texture(body_id)
        .or_else(|| assets.get_texture("tenant_body_poses"))
    else {
        return;
    };
    let Some(faces) = face_texture(tenant, assets) else {
        return;
    };

    let pose = animated_pose(apartment, tenant.id, get_time());
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
    let head_x = body_x + (body_w - head_w) / 2.0;
    let head_y = body_y - head_h * 0.42 + sprite_h * head_drop_for_pose(pose);
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

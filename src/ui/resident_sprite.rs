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

#[derive(Clone, Copy, Debug, PartialEq)]
struct ResidentStyle {
    name: &'static str,
    body_id: &'static str,
    face_id: &'static str,
    body_source_top: f32,
    body_source_height: f32,
    body_width_ratio: f32,
    head_height_ratio: f32,
}

const RESIDENT_STYLES: [ResidentStyle; 4] = [
    ResidentStyle {
        name: "Rust jacket",
        body_id: "tenant_body_poses",
        face_id: "tenant_face_emotions",
        body_source_top: 0.12,
        body_source_height: 0.76,
        body_width_ratio: 0.72,
        head_height_ratio: 0.48,
    },
    ResidentStyle {
        name: "Teal shirt",
        body_id: "tenant_body_poses_alt",
        face_id: "tenant_face_emotions_alt",
        body_source_top: 0.0,
        body_source_height: 1.0,
        body_width_ratio: 0.54,
        head_height_ratio: 0.46,
    },
    ResidentStyle {
        name: "Moss cardigan",
        body_id: "tenant_body_poses_senior",
        face_id: "tenant_face_emotions_senior",
        body_source_top: 0.10,
        body_source_height: 0.80,
        body_width_ratio: 0.70,
        head_height_ratio: 0.48,
    },
    ResidentStyle {
        name: "Indigo overshirt",
        body_id: "tenant_body_poses_indigo",
        face_id: "tenant_face_emotions_indigo",
        body_source_top: 0.0,
        body_source_height: 1.0,
        body_width_ratio: 0.52,
        head_height_ratio: 0.46,
    },
];

fn style_for(tenant_id: u32) -> ResidentStyle {
    RESIDENT_STYLES[tenant_id as usize % RESIDENT_STYLES.len()]
}

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
    let face_id = style_for(tenant.id).face_id;
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
    let style = style_for(tenant.id);
    let pose = animated_pose(apartment, tenant.id, get_time());
    draw_resident_layers(style, pose, tenant.happiness, room, assets);
}

fn draw_resident_layers(
    style: ResidentStyle,
    pose: ResidentPose,
    happiness: i32,
    room: Rect,
    assets: &AssetManager,
) {
    let Some(bodies) = assets
        .get_texture(style.body_id)
        .or_else(|| assets.get_texture("tenant_body_poses"))
    else {
        return;
    };
    let Some(faces) = assets
        .get_texture(style.face_id)
        .or_else(|| assets.get_texture("tenant_face_emotions"))
    else {
        return;
    };

    let body_column_w = bodies.width() / BODY_COLUMNS;
    let face_column_w = faces.width() / FACE_COLUMNS;
    let sprite_h = (room.h * 0.68).clamp(58.0, 112.0);
    let body_w = sprite_h * style.body_width_ratio;
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
                bodies.height() * style.body_source_top,
                body_column_w,
                bodies.height() * style.body_source_height,
            )),
            ..Default::default()
        },
    );

    let head_h = sprite_h * style.head_height_ratio;
    let head_w = head_h * 0.72;
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
                emotion_column(happiness) * face_column_w,
                faces.height() * 0.24,
                face_column_w,
                faces.height() * 0.48,
            )),
            ..Default::default()
        },
    );
}

pub(crate) fn draw_resident_sprite_gallery(assets: &AssetManager) {
    clear_background(Color::from_rgba(27, 29, 36, 255));
    draw_text("RESIDENT SPRITE FIT CHECK", 34.0, 42.0, 30.0, WHITE);
    draw_text(
        "Every matched head/body set across standing, sitting, cooking, and all five moods",
        34.0,
        68.0,
        18.0,
        Color::from_rgba(198, 203, 214, 255),
    );

    let scale = (screen_width() / 1280.0).min(screen_height() / 720.0);
    let origin_x = (screen_width() - 1220.0 * scale) * 0.5;
    let origin_y = 88.0 * scale;
    let row_h = 148.0 * scale;
    let labels = [
        "STAND",
        "SIT",
        "COOK",
        "JOY",
        "HAPPY",
        "OK",
        "LOW",
        "MISERABLE",
    ];
    for (index, label) in labels.iter().enumerate() {
        let x = origin_x + (190.0 + index as f32 * 124.0) * scale;
        draw_text(label, x, origin_y - 8.0 * scale, 14.0 * scale, GRAY);
    }

    for (style_index, style) in RESIDENT_STYLES.iter().copied().enumerate() {
        let y = origin_y + style_index as f32 * row_h;
        draw_text(style.name, origin_x, y + 72.0 * scale, 18.0 * scale, WHITE);
        draw_line(
            origin_x,
            y + row_h - 5.0 * scale,
            origin_x + 1220.0 * scale,
            y + row_h - 5.0 * scale,
            scale,
            Color::from_rgba(65, 69, 82, 255),
        );

        for (pose_index, pose) in [
            ResidentPose::Standing,
            ResidentPose::Sitting,
            ResidentPose::Cooking,
        ]
        .into_iter()
        .enumerate()
        {
            let room = Rect::new(
                origin_x + (174.0 + pose_index as f32 * 124.0) * scale,
                y,
                92.0 * scale,
                142.0 * scale,
            );
            draw_rectangle(
                room.x,
                room.y,
                room.w,
                room.h,
                Color::from_rgba(42, 45, 54, 255),
            );
            draw_resident_layers(style, pose, 55, room, assets);
        }

        for (emotion_index, happiness) in [95, 75, 55, 30, 5].into_iter().enumerate() {
            let rect = Rect::new(
                origin_x + (560.0 + emotion_index as f32 * 124.0) * scale,
                y + 28.0 * scale,
                78.0 * scale,
                92.0 * scale,
            );
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::from_rgba(42, 45, 54, 255),
            );
            let Some(faces) = assets.get_texture(style.face_id) else {
                continue;
            };
            let face_column_w = faces.width() / FACE_COLUMNS;
            draw_texture_ex(
                faces,
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(rect.size()),
                    source: Some(Rect::new(
                        emotion_column(happiness) * face_column_w,
                        faces.height() * 0.20,
                        face_column_w,
                        faces.height() * 0.55,
                    )),
                    ..Default::default()
                },
            );
        }
    }
}

#[cfg(test)]
mod tests;

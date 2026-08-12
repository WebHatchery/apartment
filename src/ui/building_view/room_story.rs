//! Visual state layers for everything the player can change inside a unit.

use crate::assets::AssetManager;
use crate::building::Apartment;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RoomFeatures {
    damaged: bool,
    severely_damaged: bool,
    cared_for: bool,
    soundproofed: bool,
    upgraded_lighting: bool,
    renovated_kitchen: bool,
}

fn room_features(apartment: &Apartment) -> RoomFeatures {
    RoomFeatures {
        damaged: apartment.condition < 45,
        severely_damaged: apartment.condition < 25,
        cared_for: apartment.condition >= 75,
        soundproofed: apartment.has_soundproofing,
        upgraded_lighting: apartment.flags.contains("has_better_lighting"),
        renovated_kitchen: apartment.kitchen_level > 0
            || apartment.flags.contains("has_renovated_kitchen"),
    }
}

pub(super) fn draw_room_story(apartment: &Apartment, room: Rect, assets: &AssetManager) {
    let features = room_features(apartment);
    if features.upgraded_lighting {
        draw_lighting(room);
    }
    if features.soundproofed {
        draw_acoustic_panels(room);
    }
    if features.renovated_kitchen {
        draw_kitchen(room, apartment.kitchen_level);
    }
    if features.damaged {
        draw_damage(room, apartment.id, features.severely_damaged);
    } else if features.cared_for {
        draw_plant(room, assets);
    }
}

fn draw_lighting(room: Rect) {
    let x = room.x + room.w * 0.52;
    let y = room.y + 8.0;
    draw_circle(x, y + 6.0, room.h * 0.34, Color::new(1.0, 0.72, 0.31, 0.10));
    draw_line(
        x,
        room.y,
        x,
        y + 4.0,
        1.5,
        Color::new(0.20, 0.15, 0.10, 0.9),
    );
    draw_circle(x, y + 7.0, 4.0, Color::new(1.0, 0.78, 0.38, 1.0));
}

fn draw_acoustic_panels(room: Rect) {
    let panel_w = (room.w * 0.035).clamp(5.0, 10.0);
    let panel_h = (room.h * 0.23).clamp(16.0, 31.0);
    let start_x = room.right() - panel_w * 4.6;
    let y = room.y + room.h * 0.25;
    for index in 0..3 {
        let x = start_x + index as f32 * panel_w * 1.35;
        draw_rectangle(x, y, panel_w, panel_h, Color::new(0.12, 0.28, 0.29, 0.90));
        draw_rectangle_lines(
            x,
            y,
            panel_w,
            panel_h,
            1.0,
            Color::new(0.38, 0.58, 0.55, 0.9),
        );
    }
}

fn draw_kitchen(room: Rect, level: i32) {
    let width = (room.w * 0.28).clamp(48.0, 92.0);
    let height = (room.h * 0.25).clamp(18.0, 34.0);
    let x = room.right() - width - 4.0;
    let y = room.bottom() - height - 3.0;
    let cabinet = if level >= 2 {
        Color::new(0.31, 0.48, 0.45, 0.96)
    } else {
        Color::new(0.47, 0.30, 0.18, 0.96)
    };
    draw_rectangle(x, y, width, height, cabinet);
    draw_rectangle(
        x - 2.0,
        y - 3.0,
        width + 4.0,
        4.0,
        Color::new(0.83, 0.73, 0.58, 1.0),
    );
    draw_line(
        x + width * 0.5,
        y,
        x + width * 0.5,
        y + height,
        1.0,
        Color::new(0.12, 0.10, 0.08, 0.7),
    );
    draw_circle(
        x + width * 0.42,
        y + 5.0,
        1.5,
        Color::new(0.93, 0.74, 0.36, 1.0),
    );
    draw_circle(
        x + width * 0.58,
        y + 5.0,
        1.5,
        Color::new(0.93, 0.74, 0.36, 1.0),
    );
}

fn draw_damage(room: Rect, apartment_id: u32, severe: bool) {
    draw_rectangle(
        room.x,
        room.y,
        room.w,
        room.h,
        Color::new(0.20, 0.16, 0.11, 0.18),
    );
    let base_x = room.x + room.w * (0.22 + (apartment_id % 4) as f32 * 0.11);
    let base_y = room.y + room.h * 0.34;
    let crack = Color::new(0.18, 0.12, 0.09, 0.8);
    draw_line(base_x, base_y, base_x + 8.0, base_y + 12.0, 1.5, crack);
    draw_line(
        base_x + 8.0,
        base_y + 12.0,
        base_x + 2.0,
        base_y + 23.0,
        1.5,
        crack,
    );
    draw_line(
        base_x + 7.0,
        base_y + 12.0,
        base_x + 15.0,
        base_y + 17.0,
        1.0,
        crack,
    );
    if severe {
        let bag_y = room.bottom() - 7.0;
        draw_circle(
            room.x + room.w * 0.20,
            bag_y,
            7.0,
            Color::new(0.16, 0.14, 0.11, 0.95),
        );
        draw_circle(
            room.x + room.w * 0.25,
            bag_y + 1.0,
            5.0,
            Color::new(0.22, 0.19, 0.14, 0.95),
        );
        draw_line(
            room.x + room.w * 0.20,
            bag_y - 7.0,
            room.x + room.w * 0.20,
            bag_y - 10.0,
            2.0,
            crack,
        );
    }
}

fn draw_plant(room: Rect, assets: &AssetManager) {
    let Some(plant) = assets.get_texture("decoration_plant") else {
        return;
    };
    let size = room.h.min(room.w) * 0.24;
    draw_texture_ex(
        plant,
        room.right() - size - 5.0,
        room.bottom() - size,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            ..Default::default()
        },
    );
}

#[cfg(test)]
mod tests;

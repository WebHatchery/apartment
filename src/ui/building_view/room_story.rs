//! Visual state layers for everything the player can change inside a unit.

use crate::building::Apartment;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RoomFeatures {
    cared_for: bool,
    soundproofed: bool,
    upgraded_lighting: bool,
    renovated_kitchen: bool,
}

fn room_features(apartment: &Apartment) -> RoomFeatures {
    RoomFeatures {
        cared_for: apartment.condition >= 75,
        soundproofed: apartment.has_soundproofing,
        upgraded_lighting: apartment.flags.contains("has_better_lighting"),
        renovated_kitchen: apartment.kitchen_level > 0
            || apartment.flags.contains("has_renovated_kitchen"),
    }
}

pub(super) fn draw_room_story(apartment: &Apartment, room: Rect) {
    let features = room_features(apartment);
    super::condition_art::draw_condition(room, apartment.condition, apartment.id);
    if features.upgraded_lighting {
        draw_lighting(room);
    }
    if features.soundproofed {
        draw_acoustic_panels(room);
    }
    if features.renovated_kitchen {
        draw_kitchen(room, apartment.kitchen_level);
    }
    if features.cared_for {
        super::condition_art::draw_plant(room);
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

#[cfg(test)]
mod tests;

//! Environmental wear follows existing condition, without cosmetic RNG or saved state.
use macroquad::prelude::*;

pub(super) fn draw_condition(room: Rect, condition: i32, identity: u32) {
    if condition >= 75 {
        // Warm paint catches the light after repairs.
        draw_rectangle(
            room.x,
            room.y,
            room.w,
            room.h,
            Color::new(1.0, 0.84, 0.59, 0.09),
        );
        return;
    }
    let severe = condition < 25;
    let damaged = condition < 45;
    let dirt = Color::new(0.25, 0.21, 0.13, if damaged { 0.42 } else { 0.22 });
    // Stable, stepped patches read as stains and peeling paint, not warning UI.
    let x = room.x + room.w * (0.36 + (identity % 3) as f32 * 0.09);
    let y = room.y + room.h * 0.19;
    let patch_w = room.w * if severe { 0.19 } else { 0.12 };
    draw_rectangle(x, y, patch_w, room.h * 0.10, dirt);
    draw_rectangle(
        x + 4.0,
        y + room.h * 0.10,
        patch_w * 0.65,
        room.h * 0.08,
        dirt,
    );
    for index in 0..if severe { 5 } else { 3 } {
        let floor_x = room.x + room.w * (0.12 + index as f32 * 0.16);
        let floor_y = room.y + room.h * (0.80 + (index % 2) as f32 * 0.06);
        draw_rectangle(floor_x, floor_y, room.w * 0.07, 2.0, dirt);
    }
    if damaged {
        draw_crack(room, x + patch_w * 0.5, y + 8.0);
        draw_rectangle(
            room.x + room.w * 0.68,
            room.y + room.h * 0.70,
            room.w * 0.14,
            4.0,
            Color::new(0.22, 0.14, 0.09, 0.75),
        );
    }
    if severe {
        draw_rectangle(
            room.x,
            room.y,
            room.w,
            room.h,
            Color::new(0.20, 0.16, 0.11, 0.13),
        );
        draw_crack(room, room.x + room.w * 0.81, room.y + room.h * 0.25);
        for index in 0..4 {
            draw_rectangle(
                room.x + room.w * 0.62 + index as f32 * 6.0,
                room.y + room.h * 0.88 - (index % 2) as f32 * 3.0,
                5.0,
                3.0,
                Color::new(0.55, 0.43, 0.28, 0.9),
            );
        }
    }
}

fn draw_crack(room: Rect, x: f32, y: f32) {
    let step = (room.h * 0.07).min(9.0);
    let ink = Color::new(0.19, 0.13, 0.09, 0.85);
    for (dx, dy, width, height) in [
        (0.0, 0.0, 2.0, step),
        (0.0, step, step, 2.0),
        (step, step, 2.0, step),
        (step * 0.5, step * 2.0, step * 0.5 + 2.0, 2.0),
        (step * 0.5, step * 2.0, 2.0, step),
    ] {
        draw_rectangle(x + dx, y + dy, width, height, ink);
    }
}

pub(super) fn draw_lobby_condition(room: Rect, condition: i32) {
    draw_condition(room, condition, 0);
    let floor_y = room.y + room.h * 0.81;
    if condition >= 75 {
        draw_rectangle(
            room.x + 3.0,
            floor_y,
            room.w - 6.0,
            room.bottom() - floor_y,
            Color::new(0.43, 0.32, 0.22, 1.0),
        );
        for index in 1..12 {
            let x = room.x + room.w * index as f32 / 12.0;
            draw_line(
                x,
                floor_y,
                x - 5.0,
                room.bottom(),
                1.0,
                Color::new(0.55, 0.43, 0.29, 1.0),
            );
        }
        draw_plant(Rect::new(room.x + 12.0, room.y, 32.0, room.h));
        draw_plant(Rect::new(room.right() - 44.0, room.y, 32.0, room.h));
    } else if condition < 45 {
        for index in 0..5 {
            draw_rectangle(
                room.x + room.w * (0.18 + index as f32 * 0.13),
                floor_y + (index % 2) as f32 * 5.0,
                9.0,
                4.0,
                Color::new(0.72, 0.66, 0.48, 0.9),
            );
        }
    }
    // These fixtures represent shared condition; selecting any part opens the
    // hallway's existing repair action, never a fictional fixture repair.
    for fraction in [0.28, 0.72] {
        let x = room.x + room.w * fraction;
        let y = room.y + 9.0;
        let lit = condition >= 45 || fraction < 0.5;
        if lit {
            draw_rectangle(
                x - 13.0,
                y + 5.0,
                26.0,
                room.h * 0.44,
                Color::new(1.0, 0.77, 0.40, if condition >= 75 { 0.14 } else { 0.06 }),
            );
        }
        draw_rectangle(x - 8.0, y, 16.0, 4.0, Color::new(0.24, 0.19, 0.13, 1.0));
        draw_rectangle(
            x - 6.0,
            y + 4.0,
            12.0,
            4.0,
            if lit {
                Color::new(0.96, 0.80, 0.48, 1.0)
            } else {
                Color::new(0.32, 0.29, 0.24, 1.0)
            },
        );
    }
}

pub(super) fn draw_plant(room: Rect) {
    let pixel = (room.h.min(room.w) * 0.035).clamp(2.0, 4.0);
    let x = room.right() - pixel * 5.0 - 6.0;
    let y = room.bottom() - pixel * 3.0;
    draw_rectangle(
        x,
        y,
        pixel * 4.0,
        pixel * 3.0,
        Color::new(0.56, 0.29, 0.15, 1.0),
    );
    draw_rectangle(
        x - pixel,
        y,
        pixel * 6.0,
        pixel,
        Color::new(0.73, 0.42, 0.23, 1.0),
    );
    draw_rectangle(
        x + pixel,
        y - pixel * 5.0,
        pixel,
        pixel * 5.0,
        Color::new(0.22, 0.37, 0.12, 1.0),
    );
    for (dx, dy) in [(-2.0, -3.0), (1.0, -5.0), (2.0, -2.0), (-1.0, -6.0)] {
        draw_rectangle(
            x + dx * pixel,
            y + dy * pixel,
            pixel * 3.0,
            pixel * 2.0,
            Color::new(0.34, 0.52, 0.18, 1.0),
        );
    }
}

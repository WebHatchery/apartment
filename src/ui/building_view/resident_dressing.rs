//! Restrained per-resident dressing drawn over the room art: one keepsake for
//! the resident's archetype, a framed photo once they have stayed a year,
//! moving boxes in their first month, and a single slow ambient touch. Every
//! choice comes from the resident's id through a decorative hash, never the
//! simulation RNG, so it is stable across frames and changes no results.

use crate::tenant::{Tenant, TenantArchetype};
use macroquad::prelude::*;
use macroquad_toolkit::settings::reduced_motion_enabled;

/// Stable decorative variation from an id (Knuth multiplicative hash).
fn decor_hash(id: u32) -> u32 {
    id.wrapping_mul(2_654_435_761) >> 16
}

/// Wall and floor anchors on the side of the room away from the kitchen
/// and acoustic panels, which sit on the right.
struct Spot {
    wall: Vec2,
    floor: Vec2,
    unit: f32,
}

fn spot(room: Rect, id: u32) -> Spot {
    let nudge = (decor_hash(id) % 5) as f32 * 0.02;
    let unit = (room.h * 0.11).clamp(7.0, 16.0);
    Spot {
        wall: vec2(room.x + room.w * (0.10 + nudge), room.y + room.h * 0.24),
        floor: vec2(room.x + room.w * (0.16 + nudge), room.bottom() - 4.0),
        unit,
    }
}

pub(super) fn draw_resident_dressing(room: Rect, tenant: &Tenant, elapsed_seconds: f64) {
    let spot = spot(room, tenant.id);
    if tenant.months_residing == 0 {
        draw_moving_boxes(spot.floor, spot.unit);
    } else {
        draw_keepsake(&tenant.archetype, &spot, tenant.id, elapsed_seconds);
    }
    if tenant.months_residing >= 12 {
        draw_photo(
            vec2(spot.wall.x + spot.unit * 3.2, spot.wall.y - spot.unit * 0.4),
            spot.unit,
        );
    }
}

fn draw_moving_boxes(floor: Vec2, unit: f32) {
    let cardboard = Color::new(0.66, 0.50, 0.32, 1.0);
    let tape = Color::new(0.86, 0.79, 0.62, 1.0);
    for (dx, dy, w, h) in [
        (0.0_f32, 0.0_f32, 2.4_f32, 1.8_f32),
        (2.6, 0.0, 2.0, 1.5),
        (0.4, -1.8, 1.8, 1.4),
    ] {
        let rect = Rect::new(
            floor.x + dx * unit,
            floor.y - (dy.abs() + h) * unit,
            w * unit,
            h * unit,
        );
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, cardboard);
        draw_rectangle(rect.x + rect.w * 0.45, rect.y, rect.w * 0.1, rect.h, tape);
    }
}

fn draw_keepsake(archetype: &TenantArchetype, spot: &Spot, id: u32, elapsed: f64) {
    let u = spot.unit;
    let (wall, floor) = (spot.wall, spot.floor);
    let accent = [
        Color::new(0.80, 0.33, 0.26, 1.0),
        Color::new(0.27, 0.49, 0.67, 1.0),
        Color::new(0.38, 0.62, 0.35, 1.0),
    ][(decor_hash(id) % 3) as usize];
    match archetype {
        TenantArchetype::Student => {
            // A poster and a stack of course books.
            draw_rectangle(wall.x, wall.y, u * 1.6, u * 2.2, accent);
            draw_rectangle_lines(wall.x, wall.y, u * 1.6, u * 2.2, 1.0, BLACK);
            for (index, shade) in [0.9, 0.6, 0.75].iter().enumerate() {
                draw_rectangle(
                    floor.x,
                    floor.y - (index as f32 + 1.0) * u * 0.45,
                    u * 1.8,
                    u * 0.4,
                    Color::new(accent.r * shade, accent.g * shade, accent.b * shade, 1.0),
                );
            }
        }
        TenantArchetype::Professional => {
            // A framed certificate and a mug with a slow curl of steam.
            draw_rectangle(
                wall.x,
                wall.y,
                u * 2.0,
                u * 1.4,
                Color::new(0.95, 0.92, 0.84, 1.0),
            );
            draw_rectangle_lines(
                wall.x,
                wall.y,
                u * 2.0,
                u * 1.4,
                2.0,
                Color::new(0.35, 0.25, 0.15, 1.0),
            );
            draw_rectangle(
                floor.x,
                floor.y - u,
                u * 0.8,
                u,
                Color::new(0.92, 0.92, 0.90, 1.0),
            );
            draw_steam(vec2(floor.x + u * 0.4, floor.y - u), u, id, elapsed);
        }
        TenantArchetype::Artist => {
            // An easel with a painting in progress.
            let leg = Color::new(0.45, 0.30, 0.18, 1.0);
            draw_line(floor.x, floor.y, floor.x + u, floor.y - u * 3.6, 2.0, leg);
            draw_line(
                floor.x + u * 2.0,
                floor.y,
                floor.x + u,
                floor.y - u * 3.6,
                2.0,
                leg,
            );
            let canvas = Rect::new(floor.x - u * 0.1, floor.y - u * 3.4, u * 2.2, u * 1.7);
            draw_rectangle(
                canvas.x,
                canvas.y,
                canvas.w,
                canvas.h,
                Color::new(0.96, 0.94, 0.88, 1.0),
            );
            draw_circle(
                canvas.x + canvas.w * 0.35,
                canvas.y + canvas.h * 0.45,
                u * 0.35,
                accent,
            );
            draw_rectangle(
                canvas.x + canvas.w * 0.55,
                canvas.y + canvas.h * 0.6,
                canvas.w * 0.3,
                canvas.h * 0.2,
                Color::new(0.93, 0.70, 0.24, 1.0),
            );
        }
        TenantArchetype::Family => {
            // A child's drawing on the wall and toy blocks on the floor.
            draw_rectangle(wall.x, wall.y, u * 1.8, u * 1.3, WHITE);
            draw_line(
                wall.x + u * 0.2,
                wall.y + u,
                wall.x + u * 0.9,
                wall.y + u * 0.3,
                2.0,
                accent,
            );
            draw_line(
                wall.x + u * 0.9,
                wall.y + u * 0.3,
                wall.x + u * 1.6,
                wall.y + u,
                2.0,
                accent,
            );
            draw_circle(
                wall.x + u * 1.4,
                wall.y + u * 0.3,
                u * 0.2,
                Color::new(0.98, 0.80, 0.25, 1.0),
            );
            for (index, color) in [
                accent,
                Color::new(0.98, 0.80, 0.25, 1.0),
                Color::new(0.27, 0.49, 0.67, 1.0),
            ]
            .iter()
            .enumerate()
            {
                draw_rectangle(
                    floor.x + index as f32 * u * 0.75,
                    floor.y - u * 0.7,
                    u * 0.65,
                    u * 0.65,
                    *color,
                );
            }
        }
        TenantArchetype::Elderly => {
            // A small bookshelf and a knitting basket with a steaming cup.
            let wood = Color::new(0.42, 0.27, 0.16, 1.0);
            draw_rectangle(wall.x, wall.y, u * 2.4, u * 0.25, wood);
            for index in 0..5 {
                let shade = 0.55 + (index % 3) as f32 * 0.15;
                draw_rectangle(
                    wall.x + u * 0.15 + index as f32 * u * 0.42,
                    wall.y - u * 0.9,
                    u * 0.32,
                    u * 0.9,
                    Color::new(accent.r * shade, accent.g * shade, accent.b * shade, 1.0),
                );
            }
            draw_rectangle(
                floor.x,
                floor.y - u * 0.9,
                u * 1.4,
                u * 0.9,
                Color::new(0.62, 0.45, 0.26, 1.0),
            );
            draw_circle(floor.x + u * 0.5, floor.y - u * 1.0, u * 0.35, accent);
            draw_steam(vec2(floor.x + u * 1.9, floor.y - u * 0.8), u, id, elapsed);
        }
    }
}

/// A photo frame: the resident has made the place theirs.
fn draw_photo(at: Vec2, u: f32) {
    draw_rectangle(
        at.x,
        at.y,
        u * 1.3,
        u * 1.0,
        Color::new(0.35, 0.24, 0.14, 1.0),
    );
    draw_rectangle(
        at.x + 2.0,
        at.y + 2.0,
        u * 1.3 - 4.0,
        u - 4.0,
        Color::new(0.70, 0.80, 0.86, 1.0),
    );
    draw_circle(
        at.x + u * 0.65,
        at.y + u * 0.45,
        u * 0.18,
        Color::new(0.87, 0.67, 0.52, 1.0),
    );
}

/// Two faint wisps rising from a cup; still under reduced motion.
fn draw_steam(at: Vec2, u: f32, id: u32, elapsed: f64) {
    let phase = if reduced_motion_enabled() {
        0.0
    } else {
        (elapsed * 0.8 + f64::from(decor_hash(id) % 7)) as f32
    };
    for wisp in 0..2 {
        let t = (phase + wisp as f32 * 0.5).fract();
        let x = at.x + (t * std::f32::consts::TAU).sin() * u * 0.15 + wisp as f32 * u * 0.2;
        let y = at.y - t * u * 1.2;
        draw_circle(x, y, u * 0.12, Color::new(1.0, 1.0, 1.0, 0.35 * (1.0 - t)));
    }
}

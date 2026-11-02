//! The sky and skyline behind the tower, following the campaign's seasons:
//! each three months the light, sky and a small seasonal detail change, so
//! Month 1 and later months look like different times of year.

use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// The campaign opens in spring; seasons turn every three months.
    pub(super) fn for_month(month: u32) -> Self {
        match (month.max(1) - 1) / 3 % 4 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }

    /// Sky gradient (top, horizon) and sun glow.
    fn palette(self) -> (Color, Color, Color) {
        match self {
            Season::Spring => (
                Color::new(0.47, 0.62, 0.78, 1.0),
                Color::new(0.93, 0.80, 0.68, 1.0),
                Color::new(1.0, 0.90, 0.62, 0.55),
            ),
            Season::Summer => (
                Color::new(0.33, 0.53, 0.84, 1.0),
                Color::new(0.98, 0.76, 0.46, 1.0),
                Color::new(1.0, 0.86, 0.45, 0.70),
            ),
            Season::Autumn => (
                Color::new(0.38, 0.42, 0.55, 1.0),
                Color::new(0.88, 0.58, 0.39, 1.0),
                Color::new(1.0, 0.72, 0.42, 0.58),
            ),
            Season::Winter => (
                Color::new(0.30, 0.34, 0.46, 1.0),
                Color::new(0.68, 0.70, 0.78, 1.0),
                Color::new(0.92, 0.94, 1.0, 0.40),
            ),
        }
    }
}

pub(super) fn draw_backdrop(view: Rect, month: u32) {
    let season = Season::for_month(month);
    let (top, bottom, sun) = season.palette();
    for index in 0..24 {
        let t = index as f32 / 23.0;
        draw_rectangle(
            view.x,
            view.y + index as f32 * view.h / 24.0,
            view.w,
            view.h / 24.0 + 1.0,
            Color::new(
                top.r + (bottom.r - top.r) * t,
                top.g + (bottom.g - top.g) * t,
                top.b + (bottom.b - top.b) * t,
                1.0,
            ),
        );
    }
    draw_circle(view.x + view.w * 0.12, view.y + 82.0, 25.0, sun);

    let skyline_y = view.bottom() - 86.0;
    let width = view.w / 9.0;
    for index in 0..10 {
        let height = 30.0 + ((index * 29) % 52) as f32;
        let x = view.x + index as f32 * width - 8.0;
        let roof = skyline_y - height;
        draw_rectangle(
            x,
            roof,
            width - 5.0,
            height + 86.0,
            Color::new(0.20, 0.20, 0.25, 0.42),
        );
        match season {
            Season::Winter => {
                draw_rectangle(x, roof, width - 5.0, 4.0, Color::new(0.95, 0.96, 1.0, 0.85));
            }
            Season::Spring | Season::Summer => {
                // A street tree in leaf in front of every other building.
                if index % 2 == 0 {
                    let leaf = if season == Season::Spring {
                        Color::new(0.52, 0.72, 0.42, 0.55)
                    } else {
                        Color::new(0.30, 0.55, 0.30, 0.55)
                    };
                    draw_circle(x + width * 0.5, view.bottom() - 30.0, 18.0, leaf);
                }
            }
            Season::Autumn => {
                if index % 2 == 0 {
                    draw_circle(
                        x + width * 0.5,
                        view.bottom() - 30.0,
                        16.0,
                        Color::new(0.85, 0.48, 0.20, 0.50),
                    );
                }
            }
        }
    }
    if season == Season::Winter {
        // Fixed flakes: the season reads at a glance without motion.
        for index in 0..40u32 {
            let fx = ((index * 73) % 997) as f32 / 997.0;
            let fy = ((index * 151) % 991) as f32 / 991.0;
            draw_circle(
                view.x + fx * view.w,
                view.y + fy * (view.h - 90.0),
                1.6,
                Color::new(1.0, 1.0, 1.0, 0.55),
            );
        }
    }
}

//! Shared geometry for building composition, contextual panels, and input bounds.
use macroquad::prelude::Rect;

pub fn status_height(width: f32) -> f32 {
    if width < 760.0 {
        100.0
    } else {
        56.0
    }
}

pub fn header_height(width: f32) -> f32 {
    status_height(width) + if width < 1180.0 { 48.0 } else { 0.0 }
}

#[derive(Clone, Copy, Debug)]
pub struct BuildingLayout {
    pub world: Rect,
    pub panel: Rect,
}

impl BuildingLayout {
    pub fn new(width: f32, height: f32, selected: bool) -> Self {
        let top = header_height(width);
        let available = (height - top - 44.0).max(1.0);
        let mut world = Rect::new(0.0, top, width, available);
        let panel = if width < 760.0 {
            let panel_height = (available * 0.62).max(280.0).min(available * 0.8);
            if selected {
                world.h = available - panel_height - 8.0;
            }
            Rect::new(
                8.0,
                top + available - panel_height,
                width - 16.0,
                panel_height - 8.0,
            )
        } else {
            let panel_width = (width * 0.28).clamp(340.0, 400.0);
            if selected {
                world.w = width - panel_width - 16.0;
            }
            Rect::new(
                width - panel_width - 8.0,
                top + 8.0,
                panel_width,
                available - 16.0,
            )
        };
        Self { world, panel }
    }
}

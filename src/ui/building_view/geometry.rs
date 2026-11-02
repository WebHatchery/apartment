//! Tower cutaway geometry shared by drawing, hit-testing and framing, so a
//! focused room is revealed exactly where it is drawn and clicked.

use super::{Selection, MIN_FLOOR_HEIGHT};
use crate::building::{Apartment, ApartmentSize, Building};
use crate::ui::common::layout;
use crate::ui::theme::space;
use macroquad::prelude::*;

/// Height reserved above the cutaway for the building name and controls.
pub(super) const CONTROLS_H: f32 = 58.0;
/// Height of the Up/Down strip shown when the tower overflows its viewport.
const SCROLL_STRIP_H: f32 = 54.0;

#[derive(Clone, Copy, Debug)]
pub(super) struct CutawayMetrics {
    pub unit_w: f32,
    pub unit_h: f32,
    pub unit_gap: f32,
    pub floor_step: f32,
    pub units_left: f32,
    pub units_width: f32,
    pub hallway_y: f32,
    pub hallway_h: f32,
}

impl CutawayMetrics {
    /// Top edge of the rooms on `floor` (1-based).
    pub fn floor_y(&self, floor: u32) -> f32 {
        self.hallway_y
            - self.unit_gap
            - self.unit_h
            - floor.saturating_sub(1) as f32 * self.floor_step
    }
}

pub(super) fn cutaway_metrics(view: Rect, floors: usize, max_slots: usize) -> CutawayMetrics {
    let edge = if view.w < 500.0 { space::SM } else { space::XL };
    let unit_gap = layout::UNIT_GAP().clamp(8.0, 12.0);
    let hallway_h = 96.0;
    let hallway_y = view.bottom() - hallway_h - space::MD;
    let units_top = view.y + CONTROLS_H;
    let vertical_room = (hallway_y - unit_gap - units_top).max(80.0);
    let floor_step = vertical_room / floors.max(1) as f32;
    let unit_h = (floor_step - unit_gap).max(46.0);
    let horizontal_room = (view.w - edge * 2.0).max(120.0);
    let slots = max_slots.max(1) as f32;
    let unit_w = ((horizontal_room - unit_gap * (slots - 1.0)) / slots)
        .clamp(68.0, (unit_h * 3.2).max(320.0));
    let units_width = unit_w * slots + unit_gap * (slots - 1.0);
    let units_left = view.x + (view.w - units_width) / 2.0;
    CutawayMetrics {
        unit_w,
        unit_h,
        unit_gap,
        floor_step,
        units_left,
        units_width,
        hallway_y,
        hallway_h,
    }
}

pub(super) fn unit_width(apartment: &Apartment, metrics: CutawayMetrics) -> f32 {
    if matches!(apartment.size, ApartmentSize::Penthouse) {
        metrics.unit_w * 2.0 + metrics.unit_gap
    } else {
        metrics.unit_w
    }
}

/// The scrollable tower inside the building workspace for the current canvas.
#[derive(Clone, Copy, Debug)]
pub(super) struct TowerGeometry {
    pub view: Rect,
    pub viewport: Rect,
    pub overflow: bool,
    pub content_height: f32,
    pub max_scroll: f32,
    pub max_floor: u32,
    pub max_floor_slots: usize,
}

impl TowerGeometry {
    pub fn new(building: &Building, selection: &Selection) -> Self {
        let view = layout::building(*selection != Selection::None).world;
        let max_floor = building
            .apartments
            .iter()
            .map(|apartment| apartment.floor)
            .max()
            .unwrap_or(1);
        let max_floor_slots = (1..=max_floor)
            .map(|floor| {
                building
                    .apartments
                    .iter()
                    .filter(|apartment| apartment.floor == floor)
                    .map(|apartment| {
                        usize::from(matches!(apartment.size, ApartmentSize::Penthouse)) + 1
                    })
                    .sum::<usize>()
            })
            .max()
            .unwrap_or(1);
        // Keep rooms readable and scroll the tower when its natural height
        // exceeds the workspace. The controls and contextual panel stay fixed.
        let natural_height = CONTROLS_H + max_floor as f32 * MIN_FLOOR_HEIGHT + 96.0 + space::MD;
        let overflow = natural_height > view.h;
        let viewport = Rect::new(
            view.x,
            view.y + CONTROLS_H,
            view.w,
            (view.h - CONTROLS_H - if overflow { SCROLL_STRIP_H } else { 0.0 }).max(1.0),
        );
        let content_height = natural_height.max(viewport.h + CONTROLS_H);
        let max_scroll = (content_height - CONTROLS_H - viewport.h).max(0.0);
        Self {
            view,
            viewport,
            overflow,
            content_height,
            max_scroll,
            max_floor,
            max_floor_slots,
        }
    }

    /// The cutaway content rectangle when scrolled by `scroll`.
    pub fn content(&self, scroll: f32) -> Rect {
        Rect::new(
            self.view.x,
            self.view.y - scroll,
            self.view.w,
            self.content_height,
        )
    }

    pub fn metrics(&self, scroll: f32) -> CutawayMetrics {
        cutaway_metrics(
            self.content(scroll),
            self.max_floor as usize,
            self.max_floor_slots,
        )
    }

    /// Scroll that centres the selected room or lobby in the viewport, or
    /// `None` when the selection is not a place in the tower.
    pub fn reveal_scroll(&self, building: &Building, selection: &Selection) -> Option<f32> {
        let metrics = self.metrics(0.0);
        let (top, bottom) = match selection {
            Selection::Apartment(id) => {
                let floor = building.get_apartment(*id)?.floor;
                let top = metrics.floor_y(floor);
                (top, top + metrics.unit_h)
            }
            Selection::Hallway => (metrics.hallway_y, metrics.hallway_y + metrics.hallway_h),
            _ => return None,
        };
        let target_center = (top + bottom) * 0.5;
        let viewport_center = self.viewport.y + self.viewport.h * 0.5;
        Some((target_center - viewport_center).clamp(0.0, self.max_scroll))
    }
}

/// Reframe the tower so the selected room or lobby is visible; other
/// selections keep the current scroll.
pub fn reveal_selection(building: &Building, selection: &Selection, scroll: f32) -> f32 {
    TowerGeometry::new(building, selection)
        .reveal_scroll(building, selection)
        .unwrap_or(scroll)
}

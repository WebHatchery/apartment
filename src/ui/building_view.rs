//! Scrollable tower cutaway and fixed building navigation.
mod condition_art;
mod geometry;
mod room_story;

#[cfg(test)]
use geometry::cutaway_metrics;
pub use geometry::reveal_selection;
use geometry::{unit_width, CutawayMetrics, TowerGeometry};

use super::theme::{color, scale, space, Tone};
use super::widgets::button_at;
use super::{common::*, Selection, UiAction};
use crate::assets::AssetManager;
use crate::building::{Apartment, ApartmentSize, Building, DesignType, NoiseLevel};
use crate::tenant::Tenant;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width};

const MIN_FLOOR_HEIGHT: f32 = 112.0;

const MASONRY: Color = Color::new(0.24, 0.15, 0.11, 1.0);
const BEAM: Color = Color::new(0.11, 0.075, 0.055, 1.0);
const MORTAR: Color = Color::new(0.42, 0.29, 0.20, 1.0);

pub fn draw_building_view(
    building: &Building,
    tenants: &[Tenant],
    selection: &Selection,
    assets: &AssetManager,
    scroll_offset: f32,
) -> (Option<UiAction>, f32) {
    let geometry = TowerGeometry::new(building, selection);
    let view = geometry.view;
    draw_sunset_backdrop(view);
    let viewport = geometry.viewport;
    let max_scroll = geometry.max_scroll;
    let mouse = mouse_position();
    let over_tower = viewport.contains(vec2(mouse.0, mouse.1));
    let mut next_scroll = scroll_offset.clamp(0.0, max_scroll);
    if over_tower {
        next_scroll = (next_scroll - mouse_wheel().1 * 40.0).clamp(0.0, max_scroll);
    }
    let content = geometry.content(next_scroll);
    let metrics = geometry.metrics(next_scroll);
    let ui = macroquad_toolkit::ui::VirtualUi::new(screen_width(), screen_height());
    let mut action = ui
        .with_clip(viewport, || {
            draw_cutaway(content, metrics, building, tenants, selection, assets)
        })
        .filter(|_| over_tower);
    if geometry.overflow {
        next_scroll = super::widgets::scroll_controls(
            Rect::new(
                view.x + space::MD,
                view.bottom() - 46.0,
                view.w - space::MD * 2.0,
                38.0,
            ),
            next_scroll,
            max_scroll,
            "Up",
            "Down",
        );
    }
    if let Some(control_action) = draw_building_controls(view, &building.name) {
        action = Some(control_action);
    }
    if action.is_none() && over_tower && was_clicked(viewport.x, viewport.y, viewport.w, viewport.h)
    {
        action = Some(UiAction::ClearSelection);
    }
    (action, next_scroll)
}

fn draw_cutaway(
    content: Rect,
    metrics: CutawayMetrics,
    building: &Building,
    tenants: &[Tenant],
    selection: &Selection,
    assets: &AssetManager,
) -> Option<UiAction> {
    let max_floor = building
        .apartments
        .iter()
        .map(|unit| unit.floor)
        .max()
        .unwrap_or(1);
    draw_building_shell(content, metrics, max_floor);
    let mut action = None;
    for floor in 1..=max_floor {
        let floor_y = metrics.floor_y(floor);
        let floor_apartments: Vec<_> = building
            .apartments
            .iter()
            .filter(|apartment| apartment.floor == floor)
            .collect();
        let floor_width = floor_apartments
            .iter()
            .map(|apartment| unit_width(apartment, metrics) + metrics.unit_gap)
            .sum::<f32>()
            - metrics.unit_gap;
        let mut x = metrics.units_left + (metrics.units_width - floor_width) / 2.0;

        for apartment in floor_apartments {
            let width = unit_width(apartment, metrics);
            if let Some(unit_action) = draw_apartment(
                apartment,
                building.is_unit_sold(apartment.id),
                tenants,
                Rect::new(x, floor_y, width, metrics.unit_h),
                selection,
                assets,
            ) {
                action = Some(unit_action);
            }
            x += width + metrics.unit_gap;
        }
    }

    if let Some(hallway_action) = draw_lobby(building, metrics, selection, assets) {
        action = Some(hallway_action);
    }
    action
}

fn draw_sunset_backdrop(view: Rect) {
    let top = Color::new(0.38, 0.42, 0.55, 1.0);
    let bottom = Color::new(0.88, 0.58, 0.39, 1.0);
    for index in 0..24 {
        let t = index as f32 / 23.0;
        let band_h = view.h / 24.0 + 1.0;
        draw_rectangle(
            view.x,
            view.y + index as f32 * view.h / 24.0,
            view.w,
            band_h,
            Color::new(
                top.r + (bottom.r - top.r) * t,
                top.g + (bottom.g - top.g) * t,
                top.b + (bottom.b - top.b) * t,
                1.0,
            ),
        );
    }

    let skyline_y = view.bottom() - 86.0;
    for index in 0..10 {
        let width = view.w / 9.0;
        let height = 30.0 + ((index * 29) % 52) as f32;
        let x = view.x + index as f32 * width - 8.0;
        draw_rectangle(
            x,
            skyline_y - height,
            width - 5.0,
            height + 86.0,
            Color::new(0.20, 0.20, 0.25, 0.42),
        );
    }
    draw_circle(
        view.w * 0.12,
        view.y + 82.0,
        25.0,
        Color::new(1.0, 0.72, 0.42, 0.58),
    );
}

fn draw_building_shell(view: Rect, metrics: CutawayMetrics, floors: u32) {
    let top_y = metrics.floor_y(floors);
    let shell = Rect::new(
        metrics.units_left - 14.0,
        top_y - 12.0,
        metrics.units_width + 28.0,
        metrics.hallway_y + metrics.hallway_h - top_y + 22.0,
    );
    draw_rectangle(
        shell.x - 5.0,
        shell.y + 7.0,
        shell.w + 10.0,
        shell.h,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
    draw_rectangle(shell.x, shell.y, shell.w, shell.h, MASONRY);

    for row in 0..14 {
        let y = shell.y + 8.0 + row as f32 * 16.0;
        let offset = if row % 2 == 0 { 0.0 } else { 18.0 };
        let mut x = shell.x + offset;
        while x < shell.right() {
            draw_line(x, y, (x + 28.0).min(shell.right()), y, 1.0, MORTAR);
            x += 36.0;
        }
    }

    draw_triangle(
        vec2(shell.x - 10.0, shell.y + 1.0),
        vec2(shell.right() + 10.0, shell.y + 1.0),
        vec2(shell.right() - 28.0, shell.y - 18.0),
        Color::new(0.10, 0.09, 0.085, 1.0),
    );
    draw_rectangle(shell.x - 10.0, shell.y - 4.0, shell.w + 20.0, 8.0, BEAM);
    draw_rectangle_lines(
        shell.x,
        shell.y,
        shell.w,
        shell.h,
        2.0,
        Color::new(0.08, 0.055, 0.04, 1.0),
    );
    draw_rectangle(
        view.x,
        view.bottom() - 12.0,
        view.w,
        12.0,
        Color::new(0.12, 0.11, 0.10, 1.0),
    );
}

fn draw_apartment(
    apartment: &Apartment,
    sold: bool,
    tenants: &[Tenant],
    room: Rect,
    selection: &Selection,
    assets: &AssetManager,
) -> Option<UiAction> {
    let selected = matches!(selection, Selection::Apartment(id) if *id == apartment.id);
    let hovered = is_hovered(room.x, room.y, room.w, room.h);
    let design_id = match apartment.design {
        DesignType::Bare => "design_bare",
        DesignType::Practical => "design_practical",
        DesignType::Cozy | DesignType::Luxury | DesignType::Opulent => "design_cozy",
    };

    draw_rectangle(room.x - 4.0, room.y - 4.0, room.w + 8.0, room.h + 8.0, BEAM);
    if let Some(texture) = assets.get_texture(design_id) {
        draw_texture_ex(
            texture,
            room.x,
            room.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(room.size()),
                ..Default::default()
            },
        );
    } else {
        draw_rectangle(
            room.x,
            room.y,
            room.w,
            room.h,
            if apartment.is_vacant() {
                color::VACANT()
            } else {
                color::OCCUPIED()
            },
        );
    }
    room_story::draw_room_story(apartment, room);

    if let Some(tenant_id) = apartment.tenant_id {
        if let Some(tenant) = tenants.iter().find(|tenant| tenant.id == tenant_id) {
            super::resident_sprite::draw_resident(apartment, tenant, room, assets);
            let name = truncate_text_to_width(&tenant.name, room.w * 0.42, scale::CAPTION);
            let name_metrics = measure_text(&name, None, scale::CAPTION as u16, 1.0);
            draw_rectangle(
                room.x + space::XS,
                room.bottom() - 23.0,
                name_metrics.width + space::SM * 2.0,
                19.0,
                Color::new(0.04, 0.04, 0.05, 0.76),
            );
            draw_ui_text(
                &name,
                room.x + space::SM,
                room.bottom() - 8.0,
                scale::CAPTION,
                color::TEXT_BRIGHT(),
            );
        }
    } else {
        draw_vacancy(apartment, sold, room, assets);
    }

    if selected {
        draw_rectangle(
            room.x,
            room.y,
            room.w,
            room.h,
            Color::new(0.91, 0.56, 0.18, 0.11),
        );
    } else if hovered {
        draw_rectangle(
            room.x,
            room.y,
            room.w,
            room.h,
            Color::new(1.0, 0.92, 0.72, 0.08),
        );
    }
    draw_rectangle_lines(
        room.x,
        room.y,
        room.w,
        room.h,
        if selected { 3.0 } else { 1.0 },
        if selected {
            color::PRIMARY()
        } else {
            Color::new(0.12, 0.08, 0.05, 1.0)
        },
    );
    draw_unit_plaque(apartment, room);

    was_clicked(room.x, room.y, room.w, room.h).then_some(UiAction::SelectApartment(apartment.id))
}

fn draw_unit_plaque(apartment: &Apartment, room: Rect) {
    let size = match apartment.size {
        ApartmentSize::Small => "S",
        ApartmentSize::Medium => "M",
        ApartmentSize::Large => "L",
        ApartmentSize::Penthouse => "PH",
    };
    let plaque_w = if room.w < 150.0 { 50.0 } else { 64.0 };
    draw_rectangle(
        room.x + 6.0,
        room.y + 6.0,
        plaque_w,
        23.0,
        Color::new(0.08, 0.07, 0.055, 0.88),
    );
    draw_ui_text(
        &apartment.unit_number,
        room.x + 11.0,
        room.y + 22.0,
        scale::BODY,
        color::TEXT_BRIGHT(),
    );
    draw_ui_text(
        size,
        room.x + plaque_w - 11.0,
        room.y + 21.0,
        scale::CAPTION,
        color::TEXT_DIM(),
    );
}

/// An empty room: for rent at its asking price, or a privately owned condo.
fn draw_vacancy(apartment: &Apartment, sold: bool, room: Rect, assets: &AssetManager) {
    let window_id = if matches!(apartment.effective_noise(), NoiseLevel::High) {
        "window_street"
    } else {
        "window_quiet"
    };
    if let Some(window) = assets.get_texture(window_id) {
        let size = (room.h * 0.38).clamp(28.0, 54.0);
        draw_texture_ex(
            window,
            room.x + (room.w - size) / 2.0,
            room.y + (room.h - size) / 2.0,
            Color::new(1.0, 1.0, 1.0, 0.72),
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
    }
    draw_ui_text(
        if sold { "CONDO · OWNED" } else { "VACANT" },
        room.x + space::SM,
        room.bottom() - 8.0,
        scale::CAPTION,
        color::TEXT_DIM(),
    );
    if sold {
        return;
    }
    let rent = format!("${}", apartment.rent_price);
    let rent_w = measure_ui_text(&rent, None, scale::CAPTION as u16, 1.0).width;
    draw_ui_text(
        &rent,
        room.right() - rent_w - space::SM,
        room.bottom() - 8.0,
        scale::CAPTION,
        color::PRIMARY(),
    );
}

fn draw_lobby(
    building: &Building,
    metrics: CutawayMetrics,
    selection: &Selection,
    assets: &AssetManager,
) -> Option<UiAction> {
    let room = Rect::new(
        metrics.units_left,
        metrics.hallway_y,
        metrics.units_width,
        metrics.hallway_h,
    );
    draw_rectangle(room.x - 4.0, room.y - 4.0, room.w + 8.0, room.h + 8.0, BEAM);
    if let Some(texture) = assets.get_texture("hallway") {
        draw_texture_ex(
            texture,
            room.x,
            room.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(room.size()),
                ..Default::default()
            },
        );
    } else {
        draw_rectangle(room.x, room.y, room.w, room.h, color::SURFACE_ALT());
    }
    condition_art::draw_lobby_condition(room, building.hallway_condition);
    draw_lobby_amenities(building, room);
    let selected = matches!(selection, Selection::Hallway);
    draw_rectangle_lines(
        room.x,
        room.y,
        room.w,
        room.h,
        if selected { 3.0 } else { 1.0 },
        if selected { color::PRIMARY() } else { BEAM },
    );
    draw_rectangle(
        room.x + 8.0,
        room.y + 8.0,
        100.0,
        24.0,
        Color::new(0.08, 0.07, 0.055, 0.86),
    );
    draw_ui_text(
        "LOBBY",
        room.x + 16.0,
        room.y + 25.0,
        scale::LABEL,
        color::TEXT_BRIGHT(),
    );
    was_clicked(room.x, room.y, room.w, room.h).then_some(UiAction::SelectHallway)
}

fn draw_lobby_amenities(building: &Building, room: Rect) {
    if building.has_laundry || building.flags.contains("has_laundry") {
        let size = (room.h * 0.52).clamp(20.0, 31.0);
        for index in 0..2 {
            let x = room.x + room.w * 0.43 + index as f32 * (size + 3.0);
            let y = room.bottom() - size - 3.0;
            draw_rectangle(x, y, size, size, Color::new(0.72, 0.69, 0.61, 0.96));
            draw_circle(
                x + size / 2.0,
                y + size * 0.60,
                size * 0.28,
                Color::new(0.14, 0.22, 0.24, 1.0),
            );
            draw_circle(x + size * 0.20, y + size * 0.16, 1.5, color::POSITIVE());
        }
    }
    if building.flags.contains("staff_receptionist") {
        let desk_w = (room.w * 0.18).clamp(55.0, 96.0);
        draw_rectangle(
            room.x + room.w * 0.68,
            room.bottom() - 19.0,
            desk_w,
            16.0,
            Color::new(0.34, 0.20, 0.12, 1.0),
        );
        draw_rectangle(
            room.x + room.w * 0.68 - 2.0,
            room.bottom() - 22.0,
            desk_w + 4.0,
            4.0,
            Color::new(0.68, 0.47, 0.25, 1.0),
        );
    }
    if building.flags.contains("staff_security") {
        let x = room.right() - 25.0;
        let y = room.y + 7.0;
        draw_rectangle(x, y, 12.0, 7.0, Color::new(0.12, 0.13, 0.13, 1.0));
        draw_circle(x + 3.0, y + 3.5, 2.0, color::NEGATIVE());
        draw_line(
            x + 12.0,
            y + 3.0,
            x + 18.0,
            y - 1.0,
            2.0,
            Color::new(0.14, 0.14, 0.13, 1.0),
        );
    }
}

fn draw_building_controls(view: Rect, name: &str) -> Option<UiAction> {
    let width = (view.w - 24.0).min(360.0);
    let label = truncate_text_to_width(&format!("{} - Details", name), width - 24.0, scale::LABEL);
    button_at(
        Rect::new(view.x + (view.w - width) / 2.0, view.y + 6.0, width, 40.0),
        &label,
        true,
        Tone::Secondary,
    )
    .then_some(UiAction::SelectBuilding)
}

#[cfg(test)]
#[path = "../../tests/ui/building_view.rs"]
mod tests;

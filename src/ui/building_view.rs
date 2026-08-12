mod room_story;
mod tenant_sprite;

use super::theme::{color, scale, space, Tone};
use super::widgets::button_at;
use super::{common::*, Selection, UiAction};
use crate::assets::AssetManager;
use crate::building::{Apartment, ApartmentSize, Building, DesignType, NoiseLevel};
use crate::tenant::Tenant;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width};

const MASONRY: Color = Color::new(0.24, 0.15, 0.11, 1.0);
const BEAM: Color = Color::new(0.11, 0.075, 0.055, 1.0);
const MORTAR: Color = Color::new(0.42, 0.29, 0.20, 1.0);

pub fn draw_building_view(
    building: &Building,
    tenants: &[Tenant],
    selection: &Selection,
    assets: &AssetManager,
) -> Option<UiAction> {
    let view = Rect::new(
        0.0,
        layout::HEADER_HEIGHT(),
        screen_width() * layout::PANEL_SPLIT(),
        screen_height() - layout::HEADER_HEIGHT() - layout::FOOTER_HEIGHT(),
    );
    draw_sunset_backdrop(view);

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
    let metrics = cutaway_metrics(view, max_floor as usize, max_floor_slots);
    draw_building_shell(view, metrics, max_floor);

    let mut action = None;
    for floor in 1..=max_floor {
        let floor_y = metrics.hallway_y
            - metrics.unit_gap
            - metrics.unit_h
            - floor.saturating_sub(1) as f32 * metrics.floor_step;
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
    if let Some(control_action) = draw_building_controls(view) {
        action = Some(control_action);
    }
    action
}

fn unit_width(apartment: &Apartment, metrics: CutawayMetrics) -> f32 {
    if matches!(apartment.size, ApartmentSize::Penthouse) {
        metrics.unit_w * 2.0 + metrics.unit_gap
    } else {
        metrics.unit_w
    }
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
    let top_y = metrics.hallway_y
        - metrics.unit_gap
        - metrics.unit_h
        - floors.saturating_sub(1) as f32 * metrics.floor_step;
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
    room_story::draw_room_story(apartment, room, assets);

    if let Some(tenant_id) = apartment.tenant_id {
        if let Some(tenant) = tenants.iter().find(|tenant| tenant.id == tenant_id) {
            tenant_sprite::draw_resident(apartment, tenant, room, assets);
            let name = truncate_text_to_width(&tenant.name, room.w * 0.42, scale::CAPTION);
            draw_ui_text(
                &name,
                room.x + space::SM,
                room.bottom() - 8.0,
                scale::CAPTION,
                color::TEXT_BRIGHT(),
            );
        }
    } else {
        draw_vacancy(apartment, room, assets);
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
    let meter_x = room.right() - 48.0;
    draw_rectangle(
        meter_x,
        room.y + 11.0,
        39.0,
        8.0,
        Color::new(0.04, 0.04, 0.035, 0.72),
    );
    draw_rectangle(
        meter_x + 1.0,
        room.y + 12.0,
        37.0 * apartment.condition.clamp(0, 100) as f32 / 100.0,
        6.0,
        condition_color(apartment.condition),
    );
}

fn draw_vacancy(apartment: &Apartment, room: Rect, assets: &AssetManager) {
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
        "VACANT",
        room.x + space::SM,
        room.bottom() - 8.0,
        scale::CAPTION,
        color::TEXT_DIM(),
    );
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
    let bar_w = 72.0;
    draw_rectangle(
        room.right() - bar_w - 12.0,
        room.y + 15.0,
        bar_w,
        9.0,
        Color::new(0.05, 0.05, 0.04, 0.75),
    );
    draw_rectangle(
        room.right() - bar_w - 11.0,
        room.y + 16.0,
        (bar_w - 2.0) * building.hallway_condition as f32 / 100.0,
        7.0,
        condition_color(building.hallway_condition),
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

fn draw_building_controls(view: Rect) -> Option<UiAction> {
    let y = view.y + space::SM;
    let width = (view.w - space::LG * 2.0 - space::SM).min(330.0);
    let x = view.x + (view.w - width) / 2.0;
    let applications_w = width * 0.56;
    if button_at(
        Rect::new(x, y, applications_w, 38.0),
        "Applications",
        true,
        Tone::Primary,
    ) {
        return Some(UiAction::SelectApplications(None));
    }
    if button_at(
        Rect::new(
            x + applications_w + space::SM,
            y,
            width - applications_w - space::SM,
            38.0,
        ),
        "Ownership",
        true,
        Tone::Secondary,
    ) {
        return Some(UiAction::SelectOwnership);
    }
    None
}

#[derive(Clone, Copy, Debug)]
struct CutawayMetrics {
    unit_w: f32,
    unit_h: f32,
    unit_gap: f32,
    floor_step: f32,
    units_left: f32,
    units_width: f32,
    hallway_y: f32,
    hallway_h: f32,
}

fn cutaway_metrics(view: Rect, floors: usize, max_slots: usize) -> CutawayMetrics {
    let edge = if view.w < 500.0 { space::SM } else { space::XL };
    let unit_gap = layout::UNIT_GAP().clamp(8.0, 12.0);
    let hallway_h = if view.h < 430.0 { 42.0 } else { 54.0 };
    let hallway_y = view.bottom() - hallway_h - space::MD;
    let units_top = view.y + 58.0;
    let vertical_room = (hallway_y - unit_gap - units_top).max(80.0);
    let floor_step = (vertical_room / floors.max(1) as f32).min(layout::FLOOR_HEIGHT());
    let unit_h = (floor_step - unit_gap).clamp(46.0, layout::UNIT_HEIGHT());
    let horizontal_room = (view.w - edge * 2.0).max(120.0);
    let slots = max_slots.max(1) as f32;
    let unit_w =
        ((horizontal_room - unit_gap * (slots - 1.0)) / slots).clamp(80.0, layout::UNIT_WIDTH());
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

#[cfg(test)]
mod tests;

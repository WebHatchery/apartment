use super::theme::{color, scale, space, Tone};
use super::widgets::button_at;
use super::{common::*, Selection, UiAction};
use crate::assets::AssetManager;
use crate::building::{Apartment, ApartmentSize, Building, DesignType, NoiseLevel};
use crate::tenant::Tenant;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub fn draw_building_view(
    building: &Building,
    tenants: &[Tenant],
    selection: &Selection,
    assets: &AssetManager,
) -> Option<UiAction> {
    let mut action = None;

    let view_width = screen_width() * layout::PANEL_SPLIT();
    let view_height = screen_height() - layout::HEADER_HEIGHT() - layout::FOOTER_HEIGHT();
    let view_x = 0.0;
    let view_y = layout::HEADER_HEIGHT();

    // Background - Building Exterior
    if let Some(tex) = assets.get_texture("building_exterior") {
        draw_texture_ex(
            tex,
            view_x,
            view_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(view_width, view_height)),
                ..Default::default()
            },
        );
    } else {
        draw_rectangle(view_x, view_y, view_width, view_height, color::BACKGROUND());
    }

    // Size the cutaway from the real workspace rectangle. This keeps five-floor
    // / ten-unit campaigns readable at 800x600 instead of pushing the top
    // floors behind the navigation bar.
    let max_floor = building
        .apartments
        .iter()
        .map(|a| a.floor)
        .max()
        .unwrap_or(1);
    let max_floor_slots = (1..=max_floor)
        .map(|floor| {
            building
                .apartments
                .iter()
                .filter(|apartment| apartment.floor == floor)
                .map(|apartment| {
                    if matches!(apartment.size, ApartmentSize::Penthouse) {
                        2
                    } else {
                        1
                    }
                })
                .sum::<usize>()
        })
        .max()
        .unwrap_or(1);
    let metrics = cutaway_metrics(
        Rect::new(view_x, view_y, view_width, view_height),
        max_floor as usize,
        max_floor_slots,
    );

    // Draw floors (bottom to top)
    for floor in 1..=max_floor {
        let floor_y = metrics.hallway_y
            - space::SM
            - metrics.unit_h
            - (floor.saturating_sub(1) as f32 * metrics.floor_step);

        // Floor label
        let floor_label = if view_width < 500.0 {
            format!("F{}", floor)
        } else {
            format!("Floor {}", floor)
        };
        draw_ui_text(
            &floor_label,
            view_x + space::MD,
            floor_y + metrics.unit_h / 2.0 + scale::LABEL / 2.0,
            scale::LABEL,
            color::TEXT_DIM(),
        );

        // Draw units on this floor
        let floor_apartments: Vec<_> = building
            .apartments
            .iter()
            .filter(|a| a.floor == floor)
            .collect();

        // Calculate total floor width (accounting for penthouse double-width)
        let mut floor_total_width = 0.0;
        for apt in &floor_apartments {
            let unit_w = if matches!(apt.size, ApartmentSize::Penthouse) {
                (metrics.unit_w * 2.0) + metrics.unit_gap
            } else {
                metrics.unit_w
            };
            floor_total_width += unit_w + metrics.unit_gap;
        }
        floor_total_width -= metrics.unit_gap;

        // Center this floor's units
        let floor_start_x = metrics.units_left + (metrics.units_width - floor_total_width) / 2.0;

        let mut current_x = floor_start_x;
        for apt in floor_apartments.iter() {
            let unit_w = if matches!(apt.size, ApartmentSize::Penthouse) {
                (metrics.unit_w * 2.0) + metrics.unit_gap
            } else {
                metrics.unit_w
            };

            if let Some(apt_action) = draw_apartment_unit_sized(
                apt,
                tenants,
                current_x,
                floor_y,
                unit_w,
                metrics.unit_h,
                selection,
                assets,
            ) {
                action = Some(apt_action);
            }

            current_x += unit_w + metrics.unit_gap;
        }
    }

    // Draw hallway at bottom
    let hallway_y = metrics.hallway_y;
    let hallway_width = metrics.units_width;
    let hallway_h = metrics.hallway_h;
    let start_x = metrics.units_left;

    let hallway_selected = matches!(selection, Selection::Hallway);
    let hallway_hovered = is_hovered(start_x, hallway_y, hallway_width, hallway_h);

    let hallway_color = if hallway_selected {
        color::SELECTED()
    } else if hallway_hovered {
        color::HOVERED()
    } else {
        color::SURFACE_ALT()
    };

    // Use texture for hallway if available
    let drawn_texture = if let Some(tex) = assets.get_texture("hallway") {
        draw_texture_ex(
            tex,
            start_x,
            hallway_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(hallway_width, hallway_h)),
                ..Default::default()
            },
        );
        true
    } else {
        draw_rectangle(start_x, hallway_y, hallway_width, hallway_h, hallway_color);
        false
    };

    let hallway_border = if hallway_selected {
        color::PRIMARY()
    } else {
        color::BORDER()
    };
    draw_rectangle_lines(
        start_x,
        hallway_y,
        hallway_width,
        hallway_h,
        if hallway_selected || !drawn_texture {
            2.0
        } else {
            1.0
        },
        hallway_border,
    );

    // Hallway label and condition
    draw_ui_text(
        "HALLWAY",
        start_x + space::MD,
        hallway_y + hallway_h / 2.0 + scale::LABEL / 2.0,
        scale::LABEL,
        color::TEXT_BRIGHT(),
    );

    let cond_color = condition_color(building.hallway_condition);
    progress_bar(
        start_x + hallway_width - 110.0,
        hallway_y + (hallway_h - 14.0) / 2.0,
        100.0,
        14.0,
        building.hallway_condition as f32,
        100.0,
        cond_color,
    );

    if was_clicked(start_x, hallway_y, hallway_width, hallway_h) {
        action = Some(UiAction::SelectHallway);
    }

    // Top action buttons (clear of the header band).
    let btn_y = view_y + space::SM;
    let btn_h = 40.0;
    let controls_w = (view_width - space::LG * 2.0 - space::SM).min(300.0);
    let app_w = (controls_w * 0.54).max(112.0);
    let owner_w = controls_w - app_w - space::SM;
    let controls_x = view_x + (view_width - controls_w) / 2.0;
    if button_at(
        Rect::new(controls_x, btn_y, app_w, btn_h),
        "Applications",
        true,
        Tone::Secondary,
    ) {
        action = Some(UiAction::SelectApplications(None));
    }
    if button_at(
        Rect::new(controls_x + app_w + space::SM, btn_y, owner_w, btn_h),
        "Ownership",
        true,
        Tone::Secondary,
    ) {
        action = Some(UiAction::SelectOwnership);
    }

    action
}

fn draw_apartment_unit_sized(
    apt: &Apartment,
    tenants: &[Tenant],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    selection: &Selection,
    assets: &AssetManager,
) -> Option<UiAction> {
    let is_selected = matches!(selection, Selection::Apartment(id) if *id == apt.id);
    let unit_hovered = is_hovered(x, y, w, h);

    // Background color (fallback when no design texture)
    let bg_color = if apt.is_vacant() {
        color::VACANT()
    } else {
        color::OCCUPIED()
    };

    // Draw Design Texture as background
    let design_id = match apt.design {
        DesignType::Bare => "design_bare",
        DesignType::Practical => "design_practical",
        DesignType::Cozy => "design_cozy",
        DesignType::Luxury => "design_luxury",
        DesignType::Opulent => "design_opulent",
    };

    if let Some(tex) = assets.get_texture(design_id) {
        draw_texture_ex(
            tex,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(w, h)),
                ..Default::default()
            },
        );
    } else {
        draw_rectangle(x, y, w, h, bg_color);
    }

    // Selection / hover tint
    if is_selected {
        draw_rectangle(
            x,
            y,
            w,
            h,
            Color::new(
                color::PRIMARY().r,
                color::PRIMARY().g,
                color::PRIMARY().b,
                0.16,
            ),
        );
    } else if unit_hovered {
        draw_rectangle(x, y, w, h, Color::new(1.0, 1.0, 1.0, 0.08));
    }

    // Legibility strip behind the unit number / size.
    draw_rectangle(x, y, w, 22.0, Color::new(0.0, 0.0, 0.0, 0.45));

    // Border
    let (border_w, border_color) = if is_selected {
        (2.0, color::PRIMARY())
    } else if unit_hovered {
        (1.0, color::BORDER_STRONG())
    } else {
        (1.0, color::BORDER())
    };
    draw_rectangle_lines(x, y, w, h, border_w, border_color);

    // Unit number + size
    draw_ui_text(
        &apt.unit_number,
        x + space::SM,
        y + 16.0,
        scale::BODY,
        color::TEXT_BRIGHT(),
    );
    let size_text = match apt.size {
        ApartmentSize::Small => "S",
        ApartmentSize::Medium => "M",
        ApartmentSize::Large => "L",
        ApartmentSize::Penthouse => "PH",
    };
    let size_w = measure_ui_text(size_text, None, scale::LABEL as u16, 1.0).width;
    draw_ui_text(
        size_text,
        x + w - size_w - space::SM,
        y + 16.0,
        scale::LABEL,
        color::TEXT_DIM(),
    );

    // Condition meter
    let cond_color = condition_color(apt.condition);
    progress_bar(
        x + space::SM,
        y + 27.0,
        w - space::SM * 2.0,
        6.0,
        apt.condition as f32,
        100.0,
        cond_color,
    );

    // Noise indicator (if high)
    if matches!(apt.effective_noise(), NoiseLevel::High) {
        if let Some(icon) = assets.get_texture("icon_noise") {
            draw_texture_ex(
                icon,
                x + space::SM,
                y + 38.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(18.0, 18.0)),
                    ..Default::default()
                },
            );
        } else {
            draw_ui_text("!", x + space::SM, y + 50.0, scale::LABEL, color::WARNING());
        }
    }

    // Soundproofing indicator
    if apt.has_soundproofing {
        if let Some(icon) = assets.get_texture("icon_soundproofing") {
            draw_texture_ex(
                icon,
                x + 30.0,
                y + 38.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(18.0, 18.0)),
                    ..Default::default()
                },
            );
        } else {
            draw_ui_text("S", x + 30.0, y + 50.0, scale::LABEL, color::POSITIVE());
        }
    }

    // Low Condition Warning
    if apt.condition < 40 {
        draw_ui_text(
            "!",
            x + w - 16.0,
            y + 50.0,
            scale::HEADING,
            color::NEGATIVE(),
        );
    }

    // Tenant / vacant content
    if let Some(tenant_id) = apt.tenant_id {
        if let Some(tenant) = tenants.iter().find(|t| t.id == tenant_id) {
            let portrait_id = format!("tenant_{}", tenant.archetype.name().to_lowercase());
            if h >= 64.0 {
                if let Some(tex) = assets.get_texture(&portrait_id) {
                    let portrait_size = (h - 38.0).clamp(20.0, 40.0);
                    draw_texture_ex(
                        tex,
                        x + (w - portrait_size) / 2.0,
                        y + 36.0,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(Vec2::new(portrait_size, portrait_size)),
                            ..Default::default()
                        },
                    );
                } else {
                    draw_rectangle(
                        x + space::SM,
                        y + h - 16.0,
                        3.0,
                        12.0,
                        archetype_color(&tenant.archetype),
                    );
                }
            } else {
                draw_circle(
                    x + space::MD,
                    y + h - space::SM,
                    5.0,
                    archetype_color(&tenant.archetype),
                );
            }

            let happiness_level = if tenant.happiness >= 90 {
                "happiness_ecstatic"
            } else if tenant.happiness >= 70 {
                "happiness_happy"
            } else if tenant.happiness >= 40 {
                "happiness_neutral"
            } else if tenant.happiness >= 20 {
                "happiness_unhappy"
            } else {
                "happiness_miserable"
            };

            if let Some(icon) = assets.get_texture(happiness_level) {
                draw_texture_ex(
                    icon,
                    x + w - 24.0,
                    y + h - 24.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(20.0, 20.0)),
                        ..Default::default()
                    },
                );
            } else {
                // Colored happiness dot fallback.
                draw_circle(
                    x + w - 12.0,
                    y + h - 12.0,
                    6.0,
                    happiness_color(tenant.happiness),
                );
            }
        }
    } else {
        let window_tex = if matches!(apt.effective_noise(), NoiseLevel::High) {
            "window_street"
        } else {
            "window_quiet"
        };
        if h >= 64.0 {
            if let Some(tex) = assets.get_texture(window_tex) {
                let window_size = (h - 38.0).clamp(20.0, 40.0);
                draw_texture_ex(
                    tex,
                    x + (w - window_size) / 2.0,
                    y + 36.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(window_size, window_size)),
                        ..Default::default()
                    },
                );
            }
        }

        draw_ui_text(
            "VACANT",
            x + space::SM,
            y + h - 8.0,
            scale::CAPTION,
            color::TEXT_DIM(),
        );
        let rent = format!("${}", apt.rent_price);
        let rent_w = measure_ui_text(&rent, None, scale::CAPTION as u16, 1.0).width;
        draw_ui_text(
            &rent,
            x + w - rent_w - space::SM,
            y + h - 8.0,
            scale::CAPTION,
            color::PRIMARY(),
        );
    }

    // Handle click
    if was_clicked(x, y, w, h) {
        return Some(UiAction::SelectApartment(apt.id));
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
    let compact = view.w < 500.0;
    let edge = if compact { space::SM } else { space::LG };
    let label_w = if compact { 34.0 } else { 62.0 };
    let unit_gap = if compact {
        8.0
    } else {
        layout::UNIT_GAP().min(12.0)
    };
    let hallway_h = 44.0;
    let hallway_y = view.bottom() - hallway_h - space::SM;
    let units_top = view.y + 40.0 + space::XL;
    let vertical_room = (hallway_y - space::SM - units_top).max(80.0);
    let floor_step = (vertical_room / floors.max(1) as f32).min(layout::FLOOR_HEIGHT());
    let unit_h = (floor_step - unit_gap).clamp(42.0, layout::UNIT_HEIGHT());

    let horizontal_room = (view.w - edge * 2.0 - label_w).max(120.0);
    let slots = max_slots.max(1) as f32;
    let unit_w =
        ((horizontal_room - unit_gap * (slots - 1.0)) / slots).clamp(68.0, layout::UNIT_WIDTH());
    let units_width = unit_w * slots + unit_gap * (slots - 1.0);
    let units_left = view.x + label_w + (horizontal_room - units_width) / 2.0;

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

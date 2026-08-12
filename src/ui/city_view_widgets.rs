use crate::assets::AssetManager;
use crate::city::{Neighborhood, NeighborhoodType, PropertyListing};
use crate::ui::colors;
use crate::ui::theme::{self, scale, Tone};
use macroquad::prelude::*;

use super::city_view::CityMapAction;
use macroquad_toolkit::ui::{draw_surface, draw_ui_text_ex, truncate_text_to_width, SurfaceStyle};

pub(super) fn draw_listing_card(
    listing: &PropertyListing,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    neighborhoods: &[Neighborhood],
    player_funds: i32,
    assets: &AssetManager,
) -> Option<CityMapAction> {
    let mouse = mouse_position();
    let hovered = mouse.0 >= x && mouse.0 <= x + width && mouse.1 >= y && mouse.1 <= y + height;
    let neighborhood = neighborhoods
        .iter()
        .find(|n| n.id == listing.neighborhood_id);

    draw_listing_background(x, y, width, height, hovered, neighborhood);
    draw_listing_preview(neighborhood, x, y, width, assets);
    draw_listing_text(listing, neighborhood, x, y, width);
    draw_listing_purchase(listing, x, y, width, height, player_funds)
}

pub(super) fn draw_progress_bar(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    progress: f32,
    color: Color,
) {
    macroquad_toolkit::ui::progress_bar(x, y, width, height, progress, 1.0, color);
}

pub(super) fn draw_button_icon(label: &str, x: f32, y: f32, width: f32, height: f32) -> bool {
    let style = theme::button_style(Tone::Secondary);
    macroquad_toolkit::ui::button_rect_enabled_styled_ex(
        Rect::new(x, y, width, height),
        label,
        true,
        &style,
        macroquad_toolkit::ui::TextStyle::new(scale::LABEL, style.text_color),
        macroquad_toolkit::ui::ButtonTrigger::Press,
    )
}

pub(super) fn draw_button_mini(label: &str, x: f32, y: f32, width: f32, height: f32) -> bool {
    let style = theme::button_style(Tone::Positive);
    macroquad_toolkit::ui::button_rect_enabled_styled_ex(
        Rect::new(x, y, width, height),
        label,
        true,
        &style,
        macroquad_toolkit::ui::TextStyle::new(scale::CAPTION, style.text_color),
        macroquad_toolkit::ui::ButtonTrigger::Press,
    )
}

fn draw_listing_background(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    hovered: bool,
    neighborhood: Option<&Neighborhood>,
) {
    let bg_color = if hovered {
        colors::SURFACE_ALT()
    } else {
        colors::SURFACE()
    };
    let neighborhood_color = neighborhood
        .map(|n| n.neighborhood_type.color())
        .unwrap_or(colors::BORDER_STRONG());

    let style = SurfaceStyle::new(bg_color)
        .with_border(1.0, colors::BORDER())
        .with_left_accent(5.0, neighborhood_color);
    draw_surface(Rect::new(x, y, width, height), &style);
}

fn draw_listing_preview(
    neighborhood: Option<&Neighborhood>,
    x: f32,
    y: f32,
    width: f32,
    assets: &AssetManager,
) {
    let Some(neighborhood) = neighborhood else {
        return;
    };

    if assets.get_texture("property_facades").is_some() {
        draw_property_facade(
            &neighborhood.neighborhood_type,
            Rect::new(x + width - 100.0, y + 10.0, 90.0, 60.0),
            assets,
        );
    } else {
        let texture_id = match neighborhood.neighborhood_type {
            NeighborhoodType::Downtown => "neighborhood_downtown",
            NeighborhoodType::Industrial => "neighborhood_industrial",
            NeighborhoodType::Suburbs => "neighborhood_suburbs",
            NeighborhoodType::Historic => "neighborhood_historic",
        };
        let Some(texture) = assets.get_texture(texture_id) else {
            return;
        };
        draw_texture_ex(
            texture,
            x + width - 100.0,
            y + 10.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(90.0, 60.0)),
                ..Default::default()
            },
        );
    }
}

pub(super) fn draw_property_facade(
    neighborhood_type: &NeighborhoodType,
    rect: Rect,
    assets: &AssetManager,
) {
    let Some(texture) = assets.get_texture("property_facades") else {
        return;
    };
    let (col, row) = match neighborhood_type {
        NeighborhoodType::Downtown => (0.0, 0.0),
        NeighborhoodType::Industrial => (1.0, 0.0),
        NeighborhoodType::Historic => (0.0, 1.0),
        NeighborhoodType::Suburbs => (1.0, 1.0),
    };
    let tile_w = texture.width() * 0.5;
    let tile_h = texture.height() * 0.5;
    draw_texture_ex(
        texture,
        rect.x,
        rect.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(rect.w, rect.h)),
            source: Some(Rect::new(col * tile_w, row * tile_h, tile_w, tile_h)),
            ..Default::default()
        },
    );
}

fn draw_listing_text(
    listing: &PropertyListing,
    neighborhood: Option<&Neighborhood>,
    x: f32,
    y: f32,
    width: f32,
) {
    let text_width = (width - 125.0).max(90.0);
    draw_ui_text_ex(
        &truncate_text_to_width(&listing.name, text_width, scale::HEADING),
        x + 15.0,
        y + 22.0,
        text_params(scale::HEADING as u16, colors::TEXT_BRIGHT()),
    );

    let location_name = neighborhood.map(|n| n.name.as_str()).unwrap_or("Unknown");
    draw_ui_text_ex(
        &truncate_text_to_width(location_name, text_width, scale::LABEL),
        x + 15.0,
        y + 40.0,
        text_params(scale::LABEL as u16, colors::TEXT_DIM()),
    );
    draw_ui_text_ex(
        &truncate_text_to_width(
            &format!(
                "{} floors, {} units | {} condition",
                listing.num_floors,
                listing.total_units(),
                listing.condition.name()
            ),
            text_width,
            scale::CAPTION,
        ),
        x + 15.0,
        y + 58.0,
        text_params(scale::CAPTION as u16, colors::TEXT_DIM()),
    );

    if listing.existing_tenants > 0 {
        draw_ui_text_ex(
            &format!("{} existing tenants", listing.existing_tenants),
            x + 15.0,
            y + 73.0,
            text_params(scale::CAPTION as u16, colors::WARNING()),
        );
    }
}

fn draw_listing_purchase(
    listing: &PropertyListing,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    player_funds: i32,
) -> Option<CityMapAction> {
    let can_afford = player_funds >= listing.asking_price;
    let btn_width = 80.0;
    let btn_x = x + width - btn_width - 10.0;
    let btn_y = y + height - 46.0;

    draw_ui_text_ex(
        &format!("${}", listing.asking_price),
        x + 15.0,
        y + height - 12.0,
        text_params(scale::HEADING as u16, price_color(listing, player_funds)),
    );

    if can_afford && draw_button_mini("Buy", btn_x, btn_y, btn_width, 40.0) {
        return Some(CityMapAction::PurchaseBuilding(listing.id));
    }

    if !can_afford {
        draw_ui_text_ex(
            "Can't afford",
            btn_x,
            btn_y + 15.0,
            text_params(scale::CAPTION as u16, colors::TEXT_DIM()),
        );
    }

    None
}

fn price_color(listing: &PropertyListing, player_funds: i32) -> Color {
    if player_funds >= listing.asking_price {
        colors::POSITIVE()
    } else {
        colors::NEGATIVE()
    }
}

fn text_params(font_size: u16, color: Color) -> TextParams<'static> {
    TextParams {
        font_size,
        color,
        ..Default::default()
    }
}

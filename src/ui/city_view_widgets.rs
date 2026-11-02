//! Property listing card and district facade art shared by the City views.

use crate::assets::AssetManager;
use crate::city::{Neighborhood, NeighborhoodType, PropertyListing};
use crate::ui::colors;
use crate::ui::theme::{scale, space, Tone};
use crate::ui::widgets::{button_at, draw_card};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, format_money, truncate_text_to_width};

use super::city_view::CityMapAction;
use super::panel_flow::BUTTON_H;

/// One listing with everything needed to compare it: where, how big, how
/// full, the price, and what buying it leaves you with.
pub(super) fn draw_listing_card(
    listing: &PropertyListing,
    card: Rect,
    neighborhoods: &[Neighborhood],
    player_funds: i32,
    assets: &AssetManager,
) -> Option<CityMapAction> {
    draw_card(card, false);
    let neighborhood = neighborhoods
        .iter()
        .find(|n| n.id == listing.neighborhood_id);
    let pad = space::MD;
    let thumb_w = if card.w >= 300.0 { 84.0 } else { 0.0 };
    if let (Some(neighborhood), true) = (neighborhood, thumb_w > 0.0) {
        draw_property_facade(
            &neighborhood.neighborhood_type,
            Rect::new(card.right() - pad - thumb_w, card.y + pad, thumb_w, 66.0),
            assets,
        );
    }
    let x = card.x + pad;
    let text_w = card.w - pad * 2.0 - if thumb_w > 0.0 { thumb_w + pad } else { 0.0 };
    let line = |text: &str, y: f32, size: f32, color: Color| {
        draw_ui_text(
            &truncate_text_to_width(text, text_w, size),
            x,
            card.y + y,
            size,
            color,
        );
    };
    line(&listing.name, 24.0, scale::HEADING, colors::TEXT_BRIGHT());
    let location = neighborhood.map_or("Unknown district", |n| n.name.as_str());
    line(location, 44.0, scale::LABEL, colors::TEXT_DIM());
    line(
        &format!(
            "{} floors · {} units · {} condition",
            listing.num_floors,
            listing.total_units(),
            listing.condition.name()
        ),
        64.0,
        scale::LABEL,
        colors::TEXT(),
    );
    line(
        &format!(
            "{} of {} units already leased",
            listing.existing_tenants,
            listing.total_units()
        ),
        84.0,
        scale::LABEL,
        if listing.existing_tenants > 0 {
            colors::POSITIVE()
        } else {
            colors::TEXT_DIM()
        },
    );

    let can_afford = player_funds >= listing.asking_price;
    let button_w = 96.0;
    let button = Rect::new(
        card.right() - pad - button_w,
        card.bottom() - pad - BUTTON_H,
        button_w,
        BUTTON_H,
    );
    let price_w = button.x - x - space::SM;
    draw_ui_text(
        &format_money((listing.asking_price) as i64),
        x,
        card.bottom() - pad - 24.0,
        scale::HEADING,
        if can_afford {
            colors::POSITIVE()
        } else {
            colors::NEGATIVE()
        },
    );
    let consequence = if can_afford {
        format!(
            "leaves {}",
            format_money((player_funds - listing.asking_price) as i64)
        )
    } else {
        format!(
            "need {} more",
            format_money((listing.asking_price - player_funds) as i64)
        )
    };
    draw_ui_text(
        &truncate_text_to_width(&consequence, price_w, scale::CAPTION),
        x,
        card.bottom() - pad - 4.0,
        scale::CAPTION,
        colors::TEXT_DIM(),
    );
    button_at(button, "Review", can_afford, Tone::Positive)
        .then_some(CityMapAction::ReviewPurchase(listing.id))
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

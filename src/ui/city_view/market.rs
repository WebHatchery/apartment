//! Property market: comparable listing cards with complete purchase terms,
//! and the purchase review that states the consequence before confirming.

use super::*;
use crate::city::PropertyListing;
use crate::ui::city_view_widgets::draw_listing_card;
use macroquad_toolkit::ui::format_money;

const CARD_H: f32 = 150.0;

/// Draw property market listings
pub fn draw_market_panel(
    listings: &[&PropertyListing],
    neighborhoods: &[Neighborhood],
    player_funds: i32,
    requested_page: usize,
    selected_neighborhood_id: Option<u32>,
    pending_purchase_id: Option<u32>,
    assets: &AssetManager,
) -> Option<CityMapAction> {
    let bounds = workspace_area();
    let selected_name = selected_neighborhood_id.and_then(|id| {
        neighborhoods
            .iter()
            .find(|neighborhood| neighborhood.id == id)
            .map(|neighborhood| neighborhood.name.as_str())
    });
    let title = selected_name.map_or_else(
        || "Property market".to_string(),
        |name| format!("Market · {name}"),
    );
    let content = draw_panel(bounds, &title);
    let filtered: Vec<_> = listings
        .iter()
        .copied()
        .filter(|listing| selected_neighborhood_id.is_none_or(|id| listing.neighborhood_id == id))
        .collect();

    let budget = format!("Budget {}", format_money((player_funds) as i64));
    let budget_w =
        macroquad_toolkit::ui::measure_ui_text(&budget, None, scale::LABEL as u16, 1.0).width;
    if budget_w + 220.0 < bounds.w {
        draw_ui_text(
            &budget,
            content.right() - budget_w,
            bounds.y + 24.0,
            scale::LABEL,
            colors::POSITIVE(),
        );
    }

    // Controls wrap to two rows on narrow canvases so every label fits.
    let narrow = content.w < 560.0;
    let controls_h = if narrow {
        BUTTON_H * 2.0 + space::SM
    } else {
        BUTTON_H
    };
    let list = Rect::new(
        content.x,
        content.y,
        content.w,
        content.h - controls_h - space::SM,
    );
    let columns = if list.w >= 700.0 { 2 } else { 1 };
    let card_w = (list.w - space::LG * (columns as f32 - 1.0)) / columns as f32;
    let rows = ((list.h + space::MD) / (CARD_H + space::MD))
        .floor()
        .max(1.0) as usize;
    let page_size = rows * columns;
    let page_count = filtered.len().div_ceil(page_size).max(1);
    let page = requested_page.min(page_count - 1);
    let mut action = None;

    if filtered.is_empty() {
        draw_ui_text(
            "No listings here right now. New properties come to market each month.",
            list.x,
            list.y + scale::BODY,
            scale::BODY,
            colors::TEXT_DIM(),
        );
    }
    for (i, listing) in filtered
        .iter()
        .skip(page * page_size)
        .take(page_size)
        .enumerate()
    {
        let card = Rect::new(
            list.x + (i % columns) as f32 * (card_w + space::LG),
            list.y + (i / columns) as f32 * (CARD_H + space::MD),
            card_w,
            CARD_H,
        );
        if let Some(card_action) =
            draw_listing_card(listing, card, neighborhoods, player_funds, assets)
        {
            action = Some(card_action);
        }
    }

    let controls_y = content.bottom() - controls_h;
    let half = (content.w - space::SM) * 0.5;
    let (nav_y, pager_x, pager_w) = if narrow {
        (controls_y, content.x, content.w)
    } else {
        let pager_w = (content.w * 0.4).min(320.0);
        (controls_y, content.right() - pager_w, pager_w)
    };
    let nav_w = if narrow { half } else { 160.0 };
    if button_at(
        Rect::new(content.x, nav_y, nav_w, BUTTON_H),
        "Back to city",
        true,
        Tone::Secondary,
    ) {
        action = Some(CityMapAction::CloseMarket);
    }
    if selected_neighborhood_id.is_some()
        && button_at(
            Rect::new(content.x + nav_w + space::SM, nav_y, nav_w, BUTTON_H),
            "All districts",
            true,
            Tone::Secondary,
        )
    {
        action = Some(CityMapAction::ClearNeighborhood);
    }
    if page_count > 1 {
        let pager_y = if narrow {
            controls_y + BUTTON_H + space::SM
        } else {
            controls_y
        };
        let button_w = (pager_w - space::SM) * 0.5;
        if button_at(
            Rect::new(pager_x, pager_y, button_w, BUTTON_H),
            "Previous",
            page > 0,
            Tone::Secondary,
        ) {
            action = Some(CityMapAction::SetMarketPage(page - 1));
        }
        if button_at(
            Rect::new(pager_x + button_w + space::SM, pager_y, button_w, BUTTON_H),
            &format!("Next ({}/{})", page + 1, page_count),
            page + 1 < page_count,
            Tone::Secondary,
        ) {
            action = Some(CityMapAction::SetMarketPage(page + 1));
        }
    }

    if let Some(listing) =
        pending_purchase_id.and_then(|id| listings.iter().copied().find(|listing| listing.id == id))
    {
        // The review owns input while open: nothing beneath may act.
        action = draw_purchase_confirmation(listing, neighborhoods, player_funds, assets);
    }

    action
}

fn draw_purchase_confirmation(
    listing: &PropertyListing,
    neighborhoods: &[Neighborhood],
    player_funds: i32,
    assets: &AssetManager,
) -> Option<CityMapAction> {
    draw_rectangle(
        0.0,
        layout::HEADER_HEIGHT(),
        screen_width(),
        screen_height() - layout::HEADER_HEIGHT(),
        Color::new(0.0, 0.0, 0.0, 0.72),
    );
    // Fit between the header and the activity footer so the decision
    // buttons are never covered.
    let top = layout::HEADER_HEIGHT();
    let available = screen_height() - top - layout::FOOTER_HEIGHT() - space::LG;
    let modal_w = (screen_width() - 32.0).min(520.0);
    let modal_h = 290.0_f32.min(available);
    let modal = Rect::new(
        (screen_width() - modal_w) * 0.5,
        top + (available - modal_h) * 0.5 + space::SM,
        modal_w,
        modal_h,
    );
    let inner = draw_panel(modal, "Review property purchase");
    let neighborhood = neighborhoods
        .iter()
        .find(|neighborhood| neighborhood.id == listing.neighborhood_id);
    let thumb = if let Some(neighborhood) = neighborhood {
        draw_property_facade(
            &neighborhood.neighborhood_type,
            Rect::new(inner.x, inner.y, 88.0, 76.0),
            assets,
        );
        100.0
    } else {
        0.0
    };
    let text_x = inner.x + thumb;
    let text_w = inner.w - thumb;
    draw_ui_text(
        &truncate_text_to_width(&listing.name, text_w, scale::HEADING),
        text_x,
        inner.y + 20.0,
        scale::HEADING,
        colors::TEXT_BRIGHT(),
    );
    let location = neighborhood.map_or("Unknown district", |n| n.name.as_str());
    draw_ui_text(
        &truncate_text_to_width(location, text_w, scale::LABEL),
        text_x,
        inner.y + 42.0,
        scale::LABEL,
        colors::TEXT_DIM(),
    );
    draw_ui_text(
        &truncate_text_to_width(
            &format!(
                "{} units · {} condition",
                listing.total_units(),
                listing.condition.name()
            ),
            text_w,
            scale::LABEL,
        ),
        text_x,
        inner.y + 62.0,
        scale::LABEL,
        colors::TEXT_DIM(),
    );

    let after = player_funds - listing.asking_price;
    let rows = [
        (
            "Price",
            format_money((listing.asking_price) as i64),
            colors::WARNING(),
        ),
        (
            "Residents included",
            format!("{} of {}", listing.existing_tenants, listing.total_units()),
            colors::TEXT(),
        ),
        (
            "Balance after",
            format_money((after) as i64),
            if after >= 0 {
                colors::POSITIVE()
            } else {
                colors::NEGATIVE()
            },
        ),
    ];
    let mut y = inner.y + 82.0;
    for (key, value, value_color) in rows {
        crate::ui::widgets::kv_row(inner.x, y, inner.w, key, &value, value_color);
        y += 24.0;
    }

    let button_y = inner.bottom() - BUTTON_H;
    let button_w = (inner.w - space::SM) * 0.5;
    if button_at(
        Rect::new(inner.x, button_y, button_w, BUTTON_H),
        "Cancel",
        true,
        Tone::Secondary,
    ) {
        return Some(CityMapAction::CancelPurchase);
    }
    if button_at(
        Rect::new(inner.x + button_w + space::SM, button_y, button_w, BUTTON_H),
        "Confirm purchase",
        after >= 0,
        Tone::Positive,
    ) {
        return Some(CityMapAction::ConfirmPurchase(listing.id));
    }
    None
}

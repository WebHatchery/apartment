//! Property market listings and the purchase review dialog.

use super::*;

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
    let panel_x = 20.0;
    let panel_y = layout::HEADER_HEIGHT() + 16.0;
    let panel_width = screen_width() - 40.0;
    let panel_height = screen_height() - panel_y - layout::FOOTER_HEIGHT() - 16.0;

    let selected_name = selected_neighborhood_id.and_then(|id| {
        neighborhoods
            .iter()
            .find(|neighborhood| neighborhood.id == id)
            .map(|neighborhood| neighborhood.name.as_str())
    });
    let title = selected_name
        .map(|name| format!("Market · {}", name))
        .unwrap_or_else(|| "Property Market".to_string());
    let content = draw_panel(
        Rect::new(panel_x, panel_y, panel_width, panel_height),
        &title,
    );
    let filtered: Vec<_> = listings
        .iter()
        .copied()
        .filter(|listing| selected_neighborhood_id.is_none_or(|id| listing.neighborhood_id == id))
        .collect();

    // Budget display
    let budget_text = format!("Your Budget: ${}", player_funds);
    let budget_w =
        macroquad_toolkit::ui::measure_ui_text(&budget_text, None, scale::LABEL as u16, 1.0).width;
    draw_ui_text_ex(
        &budget_text,
        content.x + content.w - budget_w,
        panel_y + 28.0,
        text_params(scale::LABEL, colors::POSITIVE()),
    );

    let mut action = None;
    let start_y = content.y;
    let listing_height = 120.0;
    let listing_width = (content.w - 20.0) / 2.0;
    let controls_y = content.bottom() - 40.0;
    let rows_per_page =
        (((controls_y - start_y - 10.0) / (listing_height + 15.0)).floor() as usize).max(1);
    let page_size = rows_per_page * 2;
    let page_count = filtered.len().div_ceil(page_size).max(1);
    let page = requested_page.min(page_count - 1);

    for (i, listing) in filtered
        .iter()
        .skip(page * page_size)
        .take(page_size)
        .enumerate()
    {
        let col = i % 2;
        let row = i / 2;

        let x = content.x + col as f32 * (listing_width + 20.0);
        let y = start_y + row as f32 * (listing_height + 15.0);

        if let Some(a) = draw_listing_card(
            listing,
            Rect::new(x, y, listing_width, listing_height),
            neighborhoods,
            player_funds,
            assets,
        ) {
            action = Some(a);
        }
    }

    // Back button
    if draw_button_icon("Back to city", content.x, controls_y, 150.0, 40.0) {
        action = Some(CityMapAction::CloseMarket);
    }
    if selected_neighborhood_id.is_some()
        && draw_button_mini("All districts", content.x + 158.0, controls_y, 130.0, 40.0)
    {
        action = Some(CityMapAction::ClearNeighborhood);
    }
    if page_count > 1 {
        let pager_w = (content.w - 170.0).min(250.0);
        let button_w = (pager_w - 8.0) * 0.5;
        let pager_x = content.right() - pager_w;
        if page > 0 && draw_button_mini("Prev listings", pager_x, controls_y, button_w, 40.0) {
            action = Some(CityMapAction::SetMarketPage(page - 1));
        }
        if page + 1 < page_count
            && draw_button_mini(
                "Next listings",
                pager_x + button_w + 8.0,
                controls_y,
                button_w,
                40.0,
            )
        {
            action = Some(CityMapAction::SetMarketPage(page + 1));
        }
    }

    if let Some(listing) =
        pending_purchase_id.and_then(|id| listings.iter().copied().find(|listing| listing.id == id))
    {
        if let Some(modal_action) =
            draw_purchase_confirmation(listing, neighborhoods, player_funds, assets)
        {
            action = Some(modal_action);
        }
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
    let modal_w = (screen_width() - 32.0).min(520.0);
    let modal_h = 242.0;
    let modal = Rect::new(
        (screen_width() - modal_w) * 0.5,
        (screen_height() - modal_h) * 0.5,
        modal_w,
        modal_h,
    );
    let inner = draw_panel(modal, "Review property purchase");
    let neighborhood = neighborhoods
        .iter()
        .find(|neighborhood| neighborhood.id == listing.neighborhood_id);
    if let Some(neighborhood) = neighborhood {
        draw_property_facade(
            &neighborhood.neighborhood_type,
            Rect::new(inner.x, inner.y, 104.0, 92.0),
            assets,
        );
    }
    let text_x = inner.x + 120.0;
    draw_ui_text_ex(
        &truncate_text_to_width(&listing.name, inner.w - 120.0, scale::HEADING),
        text_x,
        inner.y + 24.0,
        text_params(scale::HEADING, colors::TEXT_BRIGHT()),
    );
    let location = neighborhood
        .map(|n| n.name.as_str())
        .unwrap_or("Unknown district");
    draw_ui_text_ex(
        location,
        text_x,
        inner.y + 46.0,
        text_params(scale::LABEL, colors::TEXT_DIM()),
    );
    draw_ui_text_ex(
        &format!(
            "{} units · {} condition · {} existing tenants",
            listing.total_units(),
            listing.condition.name(),
            listing.existing_tenants
        ),
        text_x,
        inner.y + 68.0,
        text_params(scale::CAPTION, colors::TEXT_DIM()),
    );
    draw_ui_text_ex(
        &format!("Purchase price: ${}", listing.asking_price),
        inner.x,
        inner.y + 101.0,
        text_params(scale::BODY, colors::WARNING()),
    );
    draw_ui_text_ex(
        &format!(
            "Balance after purchase: ${}",
            player_funds - listing.asking_price
        ),
        inner.x,
        inner.y + 125.0,
        text_params(scale::LABEL, colors::TEXT()),
    );
    let button_y = inner.bottom() - 40.0;
    let gap = 8.0;
    let button_w = (inner.w - gap) * 0.5;
    if draw_button_icon("Cancel", inner.x, button_y, button_w, 40.0) {
        return Some(CityMapAction::CancelPurchase);
    }
    if draw_button_mini(
        "Confirm purchase",
        inner.x + button_w + gap,
        button_y,
        button_w,
        40.0,
    ) {
        return Some(CityMapAction::ConfirmPurchase(listing.id));
    }
    None
}

use super::city_view_widgets::{
    draw_button_icon, draw_button_mini, draw_listing_card, draw_progress_bar, draw_property_facade,
};
use crate::assets::AssetManager;
use crate::city::{City, Neighborhood, NeighborhoodType, PropertyListing};
use crate::narrative::NarrativeEventSystem;
use crate::ui::colors;
use crate::ui::layout;
use crate::ui::theme::scale;
use crate::ui::widgets::{draw_card, draw_panel};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, draw_ui_text_ex, truncate_text_to_width};

fn text_params(font_size: f32, color: Color) -> TextParams<'static> {
    TextParams {
        font_size: font_size as u16,
        color,
        ..Default::default()
    }
}

/// Draw the city map showing all neighborhoods
pub fn draw_city_map(
    city: &City,
    selected_neighborhood_id: Option<u32>,
    assets: &AssetManager,
    narrative: &NarrativeEventSystem,
) -> Option<CityMapAction> {
    let map_x = 20.0;
    let map_y = layout::HEADER_HEIGHT() + 16.0;
    let map_width = screen_width() * 0.5 - 40.0;
    let map_height = screen_height() - map_y - layout::FOOTER_HEIGHT() - 16.0;

    let content = draw_panel(Rect::new(map_x, map_y, map_width, map_height), &city.name);

    // Draw neighborhoods as a 2x2 grid
    let grid_x = content.x;
    let grid_y = content.y;
    let cell_width = (content.w - 20.0) / 2.0;
    let cell_height = (content.h - 20.0) / 2.0;
    let padding = 10.0;

    let mut action = None;

    for (i, neighborhood) in city.neighborhoods.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;

        let x = grid_x + col as f32 * (cell_width + padding);
        let y = grid_y + row as f32 * (cell_height + padding);

        if let Some(a) = draw_neighborhood_cell(
            neighborhood,
            selected_neighborhood_id == Some(neighborhood.id),
            x,
            y,
            cell_width,
            cell_height,
            city,
            assets,
            narrative,
        ) {
            action = Some(a);
        }
    }

    action
}

/// Draw a single neighborhood cell
fn draw_neighborhood_cell(
    neighborhood: &Neighborhood,
    selected: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    _city: &City,
    assets: &AssetManager,
    narrative: &NarrativeEventSystem,
) -> Option<CityMapAction> {
    let mouse = mouse_position();
    let hovered = mouse.0 >= x && mouse.0 <= x + width && mouse.1 >= y && mouse.1 <= y + height;

    // Background with neighborhood color (fallback or tint)
    let base_color = neighborhood.neighborhood_type.color();
    let bg_color = if hovered {
        Color::from_rgba(
            ((base_color.r * 255.0) + 20.0).min(255.0) as u8,
            ((base_color.g * 255.0) + 20.0).min(255.0) as u8,
            ((base_color.b * 255.0) + 20.0).min(255.0) as u8,
            200,
        )
    } else {
        Color::from_rgba(
            (base_color.r * 255.0 * 0.6) as u8,
            (base_color.g * 255.0 * 0.6) as u8,
            (base_color.b * 255.0 * 0.6) as u8,
            180,
        )
    };

    draw_rectangle(x, y, width, height, bg_color);

    // Draw Neighborhood Texture
    let texture_id = match neighborhood.neighborhood_type {
        NeighborhoodType::Downtown => "neighborhood_downtown",
        NeighborhoodType::Industrial => "neighborhood_industrial",
        NeighborhoodType::Suburbs => "neighborhood_suburbs",
        NeighborhoodType::Historic => "neighborhood_historic",
    };

    if let Some(tex) = assets.get_texture(texture_id) {
        // Draw with some transparency or multiply to blend with selection?
        // Or just draw it fully opaque and draw selection border/overlay.
        draw_texture_ex(
            tex,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(width, height)),
                ..Default::default()
            },
        );

        // Darken it a bit to make text readable
        draw_rectangle(x, y, width, height, Color::new(0.0, 0.0, 0.0, 0.5));
    }

    draw_rectangle_lines(
        x,
        y,
        width,
        height,
        if selected { 5.0 } else { 2.0 },
        if selected {
            colors::ACCENT()
        } else {
            base_color
        },
    );
    if selected {
        draw_rectangle(x + width - 78.0, y + 8.0, 70.0, 22.0, colors::ACCENT());
        draw_ui_text_ex(
            "FILTERED",
            x + width - 71.0,
            y + 23.0,
            text_params(scale::CAPTION, colors::TEXT_BRIGHT()),
        );
    }

    // Neighborhood name
    draw_ui_text_ex(
        &truncate_text_to_width(&neighborhood.name, width - 16.0, scale::HEADING),
        x + 8.0,
        y + 22.0,
        text_params(scale::HEADING, colors::TEXT_BRIGHT()),
    );

    // Neighborhood type
    draw_ui_text_ex(
        neighborhood.neighborhood_type.name(),
        x + 8.0,
        y + 40.0,
        text_params(scale::LABEL, colors::TEXT_DIM()),
    );

    // Building count. Compact cells retain name/type/reputation and elide the
    // less important rows instead of drawing them on top of one another.
    let building_count = neighborhood.building_ids.len();
    if height >= 105.0 {
        let slot_text = format!(
            "Buildings: {}/{}",
            building_count, neighborhood.available_slots
        );
        draw_ui_text_ex(
            &slot_text,
            x + 8.0,
            y + 60.0,
            text_params(
                scale::LABEL,
                if building_count > 0 {
                    colors::POSITIVE()
                } else {
                    colors::TEXT_DIM()
                },
            ),
        );
    }

    // Stats preview
    let stats = &neighborhood.stats;
    if height >= 135.0 {
        draw_ui_text_ex(
            &format!(
                "Crime: {} | Transit: {}",
                stats.crime_level, stats.transit_access
            ),
            x + 8.0,
            y + 80.0,
            text_params(scale::CAPTION, colors::TEXT_DIM()),
        );
    }

    // Reputation bar
    let bar_y = y + height - 25.0;
    let bar_width = width - 16.0;
    draw_ui_text_ex(
        &format!("Rep: {}", neighborhood.reputation),
        x + 8.0,
        bar_y - 3.0,
        text_params(scale::CAPTION, colors::TEXT_DIM()),
    );
    draw_progress_bar(
        x + 8.0,
        bar_y,
        bar_width,
        8.0,
        neighborhood.reputation as f32 / 100.0,
        colors::POSITIVE(),
    );

    // Event indicator
    let has_event = narrative
        .events
        .iter()
        .any(|e| !e.read && e.related_neighborhood_id == Some(neighborhood.id));

    if has_event {
        let icon_x = x + width - 30.0;
        let icon_y = y + 30.0;
        draw_circle(icon_x, icon_y, 12.0, colors::ACCENT());
        draw_ui_text(
            "!",
            icon_x - 3.0,
            icon_y + 5.0,
            scale::HEADING,
            colors::TEXT_BRIGHT(),
        );
    }

    // Button area
    if hovered && is_mouse_button_pressed(MouseButton::Left) {
        return Some(CityMapAction::SelectNeighborhood(neighborhood.id));
    }

    None
}

/// Draw the portfolio panel showing all player buildings
pub fn draw_portfolio_panel(
    city: &City,
    selected_building: usize,
    requested_page: usize,
    selected_neighborhood_id: Option<u32>,
    assets: &AssetManager,
) -> Option<CityMapAction> {
    let panel_x = screen_width() * 0.5 + 10.0;
    let panel_y = layout::HEADER_HEIGHT() + 16.0;
    let panel_width = screen_width() * 0.5 - 30.0;
    let panel_height = screen_height() - panel_y - layout::FOOTER_HEIGHT() - 16.0;

    let selected_name = selected_neighborhood_id.and_then(|id| {
        city.neighborhoods
            .iter()
            .find(|neighborhood| neighborhood.id == id)
            .map(|neighborhood| neighborhood.name.as_str())
    });
    let title = selected_name
        .map(|name| format!("Properties · {}", name))
        .unwrap_or_else(|| "Your Properties".to_string());
    let content = draw_panel(
        Rect::new(panel_x, panel_y, panel_width, panel_height),
        &title,
    );

    let mut action = None;
    let item_height = 80.0;
    let properties: Vec<_> = city
        .buildings_with_info()
        .into_iter()
        .filter(|(index, _, _)| {
            selected_neighborhood_id.is_none_or(|id| {
                city.neighborhood_for_building(*index)
                    .is_some_and(|neighborhood| neighborhood.id == id)
            })
        })
        .collect();
    let controls_height = 94.0;
    let page_size = (((content.h - controls_height) / item_height).floor() as usize).max(1);
    let page_count = properties.len().div_ceil(page_size).max(1);
    let page = requested_page.min(page_count - 1);
    let mut y = content.y;

    if properties.is_empty() {
        draw_ui_text_ex(
            "No properties here yet.",
            content.x,
            y + 24.0,
            text_params(scale::BODY, colors::TEXT_DIM()),
        );
    }

    for (index, building, neighborhood_name) in properties
        .iter()
        .skip(page * page_size)
        .take(page_size)
        .cloned()
    {
        let is_selected = index == selected_building;

        let item_width = content.w;
        let item_x = content.x;

        draw_card(
            Rect::new(item_x, y, item_width, item_height - 5.0),
            is_selected,
        );

        let neighborhood = city.neighborhood_for_building(index);
        let thumbnail_w = if assets.get_texture("property_facades").is_some()
            || assets.get_texture("building_exterior").is_some()
        {
            64.0
        } else {
            0.0
        };
        if let Some(neighborhood) = neighborhood {
            draw_property_facade(
                &neighborhood.neighborhood_type,
                Rect::new(item_x + 8.0, y + 8.0, 56.0, 59.0),
                assets,
            );
        } else if let Some(texture) = assets.get_texture("building_exterior") {
            draw_texture_ex(
                texture,
                item_x + 8.0,
                y + 8.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(56.0, 59.0)),
                    ..Default::default()
                },
            );
        }

        if is_selected {
            // Enter Button
            if draw_button_mini("Enter", item_x + item_width - 78.0, y + 18.0, 68.0, 40.0) {
                action = Some(CityMapAction::EnterBuilding(index));
            }
        }

        // Building name
        let text_x = item_x + 10.0 + thumbnail_w;
        draw_ui_text_ex(
            &truncate_text_to_width(
                &building.name,
                item_width - thumbnail_w - if is_selected { 100.0 } else { 20.0 },
                scale::HEADING,
            ),
            text_x,
            y + 22.0,
            text_params(
                scale::HEADING,
                if is_selected {
                    colors::ACCENT()
                } else {
                    colors::TEXT_BRIGHT()
                },
            ),
        );

        // Location
        draw_ui_text_ex(
            &truncate_text_to_width(
                &neighborhood_name,
                item_width - thumbnail_w - if is_selected { 100.0 } else { 20.0 },
                scale::LABEL,
            ),
            text_x,
            y + 40.0,
            text_params(scale::LABEL, colors::TEXT_DIM()),
        );

        // Stats
        let occupancy = building.occupancy_count();
        let total = building.apartments.len();
        let appeal = building.building_appeal();

        let stats = format!("Occupancy: {}/{} · Appeal: {}", occupancy, total, appeal);
        draw_ui_text_ex(
            &truncate_text_to_width(
                &stats,
                item_width - thumbnail_w - if is_selected { 100.0 } else { 20.0 },
                scale::LABEL,
            ),
            text_x,
            y + 58.0,
            text_params(
                scale::LABEL,
                if occupancy == total {
                    colors::POSITIVE()
                } else {
                    colors::TEXT_DIM()
                },
            ),
        );

        // Click to select
        let mouse = mouse_position();
        let hovered = mouse.0 >= item_x
            && mouse.0 <= item_x + item_width
            && mouse.1 >= y
            && mouse.1 <= y + item_height - 5.0;

        if action.is_none() && hovered && is_mouse_button_pressed(MouseButton::Left) {
            action = Some(CityMapAction::SelectBuilding(index));
        }

        y += item_height;
    }

    let pager_y = content.bottom() - 40.0;
    let acquire_y = if page_count > 1 || selected_neighborhood_id.is_some() {
        pager_y - 48.0
    } else {
        pager_y
    };
    if draw_button_icon(
        "+ Acquire New Building",
        content.x,
        acquire_y,
        content.w,
        40.0,
    ) {
        action = Some(CityMapAction::OpenMarket);
    }
    if selected_neighborhood_id.is_some() {
        if draw_button_mini("Show all properties", content.x, pager_y, content.w, 40.0) {
            action = Some(CityMapAction::ClearNeighborhood);
        }
    } else if page_count > 1 {
        let gap = 8.0;
        let button_w = (content.w - gap) * 0.5;
        if page > 0 && draw_button_mini("Back", content.x, pager_y, button_w, 40.0) {
            action = Some(CityMapAction::SetPortfolioPage(page - 1));
        }
        if page + 1 < page_count
            && draw_button_mini("Next", content.x + button_w + gap, pager_y, button_w, 40.0)
        {
            action = Some(CityMapAction::SetPortfolioPage(page + 1));
        }
    }

    action
}

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
            x,
            y,
            listing_width,
            listing_height,
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
        if page > 0 && draw_button_mini("Back", pager_x, controls_y, button_w, 40.0) {
            action = Some(CityMapAction::SetMarketPage(page - 1));
        }
        if page + 1 < page_count
            && draw_button_mini("Next", pager_x + button_w + 8.0, controls_y, button_w, 40.0)
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

/// Actions from the city map UI
#[derive(Clone, Debug)]
pub enum CityMapAction {
    SelectNeighborhood(u32),
    ClearNeighborhood,
    SelectBuilding(usize),
    OpenMarket,
    CloseMarket,
    ReviewPurchase(u32),
    CancelPurchase,
    ConfirmPurchase(u32),
    EnterBuilding(usize),
    SetPortfolioPage(usize),
    SetMarketPage(usize),
}

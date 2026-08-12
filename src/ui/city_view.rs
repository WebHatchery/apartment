use super::city_view_widgets::{
    draw_button_icon, draw_button_mini, draw_listing_card, draw_progress_bar,
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

    draw_rectangle_lines(x, y, width, height, 2.0, base_color);

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
    assets: &AssetManager,
) -> Option<CityMapAction> {
    let panel_x = screen_width() * 0.5 + 10.0;
    let panel_y = layout::HEADER_HEIGHT() + 16.0;
    let panel_width = screen_width() * 0.5 - 30.0;
    let panel_height = screen_height() - panel_y - layout::FOOTER_HEIGHT() - 16.0;

    let content = draw_panel(
        Rect::new(panel_x, panel_y, panel_width, panel_height),
        "Your Properties",
    );

    let mut action = None;
    let mut y = content.y;
    let item_height = 80.0;

    for (index, building, neighborhood_name) in city.buildings_with_info() {
        let is_selected = index == selected_building;

        let item_width = content.w;
        let item_x = content.x;

        draw_card(
            Rect::new(item_x, y, item_width, item_height - 5.0),
            is_selected,
        );

        // Building Icon/Thumbnail?
        // Maybe just use a generic icon for now or small building exterior
        if let Some(_tex) = assets.get_texture("icon_building") { // Assuming we have one, or reuse building_exterior
             // If we don't have icon_building, we can use building_exterior scaled down?
             // But building_exterior is large. Let's just skip for now or use rectangle.
        }

        if is_selected {
            // Enter Button
            if draw_button_mini("Enter", item_x + item_width - 78.0, y + 18.0, 68.0, 40.0) {
                action = Some(CityMapAction::EnterBuilding(index));
            }
        }

        // Building name
        draw_ui_text_ex(
            &truncate_text_to_width(
                &building.name,
                item_width - if is_selected { 100.0 } else { 20.0 },
                scale::HEADING,
            ),
            item_x + 10.0,
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
            &neighborhood_name,
            item_x + 10.0,
            y + 40.0,
            text_params(scale::LABEL, colors::TEXT_DIM()),
        );

        // Stats
        let occupancy = building.occupancy_count();
        let total = building.apartments.len();
        let appeal = building.building_appeal();

        draw_ui_text_ex(
            &format!("Occupancy: {}/{} | Appeal: {}", occupancy, total, appeal),
            item_x + 10.0,
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

        if y > content.y + content.h - item_height {
            break;
        }
    }

    // "Add Building" button if there's space
    if y < content.y + content.h - 50.0 {
        let btn_width = content.w;
        let btn_x = content.x;

        if draw_button_icon("+ Acquire New Building", btn_x, y + 8.0, btn_width, 40.0) {
            action = Some(CityMapAction::OpenMarket);
        }
    }

    action
}

/// Draw property market listings
pub fn draw_market_panel(
    listings: &[&PropertyListing],
    neighborhoods: &[Neighborhood],
    player_funds: i32,
    assets: &AssetManager,
) -> Option<CityMapAction> {
    let panel_x = 20.0;
    let panel_y = layout::HEADER_HEIGHT() + 16.0;
    let panel_width = screen_width() - 40.0;
    let panel_height = screen_height() - panel_y - layout::FOOTER_HEIGHT() - 16.0;

    let content = draw_panel(
        Rect::new(panel_x, panel_y, panel_width, panel_height),
        "Property Market",
    );

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

    for (i, listing) in listings.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;

        let x = content.x + col as f32 * (listing_width + 20.0);
        let y = start_y + row as f32 * (listing_height + 15.0);

        if y + listing_height > content.y + content.h - 20.0 {
            break;
        }

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
    if draw_button_icon(
        "← Back to Map",
        content.x,
        panel_y + panel_height - 60.0,
        150.0,
        40.0,
    ) {
        action = Some(CityMapAction::CloseMarket);
    }

    action
}

/// Actions from the city map UI
#[derive(Clone, Debug)]
pub enum CityMapAction {
    SelectNeighborhood(u32),
    SelectBuilding(usize),
    OpenMarket,
    CloseMarket,
    PurchaseBuilding(u32),
    EnterBuilding(usize),
}

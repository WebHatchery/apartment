//! City workspace: pick a district first, then read its statistics and the
//! properties you own there. District tiles carry only art and a name; the
//! facts live in the detail panel so nothing is layered over busy artwork.

use super::city_view_widgets::draw_property_facade;
use super::panel_flow::{PanelFlow, BUTTON_H};
use crate::assets::AssetManager;
use crate::city::{City, Neighborhood, NeighborhoodType};
use crate::narrative::NarrativeEventSystem;
use crate::ui::colors;
use crate::ui::layout;
use crate::ui::theme::{scale, space, Tone};
use crate::ui::widgets::{button_at, draw_panel, draw_panel_header};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

/// Below this width the district grid stacks above the detail panel.
const SIDE_BY_SIDE_MIN_W: f32 = 820.0;

/// Workspace area between the header and the activity footer.
pub(super) fn workspace_area() -> Rect {
    let top = layout::HEADER_HEIGHT() + space::MD;
    Rect::new(
        space::LG,
        top,
        screen_width() - space::LG * 2.0,
        screen_height() - top - layout::FOOTER_HEIGHT() - space::MD,
    )
}

/// District grid and detail panel rectangles for the current canvas.
fn city_layout(area: Rect) -> (Rect, Rect) {
    if area.w >= SIDE_BY_SIDE_MIN_W {
        let grid_w = (area.w * 0.48).floor();
        (
            Rect::new(area.x, area.y, grid_w, area.h),
            Rect::new(
                area.x + grid_w + space::LG,
                area.y,
                area.w - grid_w - space::LG,
                area.h,
            ),
        )
    } else {
        // Two rows of short tiles: enough to recognise and pick a district
        // while the detail panel keeps most of a narrow screen.
        let grid_h = (area.h * 0.34).clamp(150.0, 230.0);
        (
            Rect::new(area.x, area.y, area.w, grid_h),
            Rect::new(
                area.x,
                area.y + grid_h + space::SM,
                area.w,
                area.h - grid_h - space::SM,
            ),
        )
    }
}

/// Draw the City workspace. Returns the player's intent and the detail
/// panel's updated scroll offset.
pub fn draw_city_view(
    city: &City,
    selected_neighborhood_id: Option<u32>,
    scroll: f32,
    assets: &AssetManager,
    narrative: &NarrativeEventSystem,
) -> (Option<CityMapAction>, f32) {
    let (grid, detail) = city_layout(workspace_area());
    let mut action = draw_district_grid(city, selected_neighborhood_id, grid, assets, narrative);
    let (detail_action, scroll) = draw_district_detail(
        city,
        selected_neighborhood_id,
        detail,
        scroll,
        assets,
        narrative,
    );
    action = action.or(detail_action);
    (action, scroll)
}

fn draw_district_grid(
    city: &City,
    selected_neighborhood_id: Option<u32>,
    grid: Rect,
    assets: &AssetManager,
    narrative: &NarrativeEventSystem,
) -> Option<CityMapAction> {
    let content = draw_panel(grid, &city.name);
    let gap = space::SM;
    let columns = 2;
    let rows = city.neighborhoods.len().div_ceil(columns).max(1);
    let cell_w = (content.w - gap) / columns as f32;
    let cell_h = (content.h - gap * (rows as f32 - 1.0)) / rows as f32;
    let mut action = None;
    for (index, neighborhood) in city.neighborhoods.iter().enumerate() {
        let cell = Rect::new(
            content.x + (index % columns) as f32 * (cell_w + gap),
            content.y + (index / columns) as f32 * (cell_h + gap),
            cell_w,
            cell_h,
        );
        let selected = selected_neighborhood_id == Some(neighborhood.id);
        let has_news = narrative
            .events
            .iter()
            .any(|event| !event.read && event.related_neighborhood_id == Some(neighborhood.id));
        if draw_district_tile(neighborhood, selected, has_news, cell, assets) {
            action = Some(CityMapAction::SelectNeighborhood(neighborhood.id));
        }
    }
    action
}

fn district_texture(kind: &NeighborhoodType) -> &'static str {
    match kind {
        NeighborhoodType::Downtown => "neighborhood_downtown",
        NeighborhoodType::Industrial => "neighborhood_industrial",
        NeighborhoodType::Suburbs => "neighborhood_suburbs",
        NeighborhoodType::Historic => "neighborhood_historic",
    }
}

/// A district tile: art, name and selection only. Returns true on release.
fn draw_district_tile(
    neighborhood: &Neighborhood,
    selected: bool,
    has_news: bool,
    cell: Rect,
    assets: &AssetManager,
) -> bool {
    let clicked = button_at(cell, "", true, Tone::Secondary);
    if let Some(texture) = assets.get_texture(district_texture(&neighborhood.neighborhood_type)) {
        draw_texture_ex(
            texture,
            cell.x,
            cell.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(cell.size()),
                ..Default::default()
            },
        );
    } else {
        draw_rectangle(
            cell.x,
            cell.y,
            cell.w,
            cell.h,
            neighborhood.neighborhood_type.color(),
        );
    }
    // A solid label strip keeps the name readable on any artwork.
    let strip_h = (scale::HEADING + space::MD).min(cell.h);
    draw_rectangle(
        cell.x,
        cell.bottom() - strip_h,
        cell.w,
        strip_h,
        Color::new(0.05, 0.05, 0.08, 0.82),
    );
    draw_ui_text(
        &truncate_text_to_width(&neighborhood.name, cell.w - space::MD * 2.0, scale::BODY),
        cell.x + space::SM,
        cell.bottom() - strip_h * 0.5 + scale::BODY * 0.35,
        scale::BODY,
        colors::TEXT_BRIGHT(),
    );
    if has_news {
        draw_circle(cell.right() - 14.0, cell.y + 14.0, 9.0, colors::ACCENT());
        draw_ui_text(
            "!",
            cell.right() - 16.5,
            cell.y + 19.0,
            scale::LABEL,
            colors::TEXT_BRIGHT(),
        );
    }
    let (thickness, outline) = if selected {
        (4.0, colors::ACCENT())
    } else {
        (1.0, colors::BORDER())
    };
    draw_rectangle_lines(cell.x, cell.y, cell.w, cell.h, thickness, outline);
    clicked
}

/// Crime reads better as a word than as a bare score.
fn crime_label(level: i32) -> &'static str {
    match level {
        ..=20 => "low",
        21..=45 => "moderate",
        _ => "high",
    }
}

fn draw_district_detail(
    city: &City,
    selected_neighborhood_id: Option<u32>,
    bounds: Rect,
    scroll: f32,
    assets: &AssetManager,
    narrative: &NarrativeEventSystem,
) -> (Option<CityMapAction>, f32) {
    let selected =
        selected_neighborhood_id.and_then(|id| city.neighborhoods.iter().find(|n| n.id == id));
    let title = selected.map_or("Your properties", |n| n.name.as_str());
    draw_panel(bounds, title);
    let mut scroll = scroll;
    if bounds.contains(mouse_position().into()) {
        scroll = (scroll - mouse_wheel().1 * 30.0).max(0.0);
    }
    let mut flow = PanelFlow::new(bounds, scroll);
    let mut action = None;

    match selected {
        Some(neighborhood) => {
            draw_district_facts(&mut flow, neighborhood, narrative);
            if flow.button("Browse listings here", true, Tone::Primary) {
                action = Some(CityMapAction::OpenMarket);
            }
            if flow.button("Show all districts", true, Tone::Secondary) {
                action = Some(CityMapAction::ClearNeighborhood);
            }
            flow.divider();
            flow.section_label("YOUR PROPERTIES HERE");
        }
        None => {
            flow.paragraph(
                "Pick a district to compare its statistics and listings.",
                scale::LABEL,
                colors::TEXT_DIM(),
            );
            if flow.button("Browse the property market", true, Tone::Primary) {
                action = Some(CityMapAction::OpenMarket);
            }
            flow.divider();
        }
    }

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
    if properties.is_empty() {
        flow.paragraph(
            "You do not own a property here yet.",
            scale::BODY,
            colors::TEXT_DIM(),
        );
    }
    for (index, building, neighborhood_name) in properties {
        let active = index == city.active_building_index;
        let kind = city
            .neighborhood_for_building(index)
            .map(|neighborhood| &neighborhood.neighborhood_type);
        let summary = format!(
            "{} · {}/{} leased · appeal {}",
            neighborhood_name,
            building.occupancy_count(),
            building.apartments.len(),
            building.building_appeal()
        );
        if draw_property_row(&mut flow, &building.name, &summary, kind, active, assets) {
            action = Some(CityMapAction::EnterBuilding(index));
        }
    }

    let max_scroll = flow.max_scroll(scroll);
    scroll = scroll.min(max_scroll);
    if max_scroll > 0.0 {
        scroll = super::widgets::scroll_controls(
            Rect::new(flow.x, bounds.bottom() - 48.0, flow.w, 40.0),
            scroll,
            max_scroll,
            "Up",
            "More",
        );
    }
    draw_panel_header(bounds, title);
    (action, scroll)
}

fn draw_district_facts(
    flow: &mut PanelFlow,
    neighborhood: &Neighborhood,
    narrative: &NarrativeEventSystem,
) {
    let stats = &neighborhood.stats;
    flow.kv(
        "District",
        neighborhood.neighborhood_type.name(),
        colors::TEXT(),
    );
    flow.kv(
        "Your buildings",
        &format!(
            "{} of {} lots",
            neighborhood.building_ids.len(),
            neighborhood.available_slots
        ),
        colors::TEXT(),
    );
    flow.kv(
        "Crime",
        &format!("{} ({})", crime_label(stats.crime_level), stats.crime_level),
        if stats.crime_level > 45 {
            colors::WARNING()
        } else {
            colors::TEXT()
        },
    );
    flow.kv("Transit", &stats.transit_access.to_string(), colors::TEXT());
    flow.kv(
        "Reputation",
        &neighborhood.reputation.to_string(),
        colors::POSITIVE(),
    );
    if let Some(event) = narrative
        .events
        .iter()
        .find(|event| !event.read && event.related_neighborhood_id == Some(neighborhood.id))
    {
        flow.paragraph(
            &format!("News: {}", event.headline),
            scale::LABEL,
            colors::ACCENT(),
        );
    }
    flow.gap();
}

/// One owned property with its enter action. Returns true on Enter.
fn draw_property_row(
    flow: &mut PanelFlow,
    name: &str,
    summary: &str,
    kind: Option<&NeighborhoodType>,
    active: bool,
    assets: &AssetManager,
) -> bool {
    let row_h = BUTTON_H + space::SM;
    let mut entered = false;
    if flow.fits(row_h) {
        let (x, y, w) = (flow.x, flow.y, flow.w);
        let thumb = if let Some(kind) = kind {
            draw_property_facade(kind, Rect::new(x, y + 2.0, 44.0, 44.0), assets);
            48.0
        } else {
            0.0
        };
        let enter_w = 84.0;
        let text_w = w - thumb - enter_w - space::SM;
        draw_ui_text(
            &truncate_text_to_width(name, text_w, scale::BODY),
            x + thumb,
            y + 18.0,
            scale::BODY,
            if active {
                colors::ACCENT()
            } else {
                colors::TEXT_BRIGHT()
            },
        );
        draw_ui_text(
            &truncate_text_to_width(summary, text_w, scale::CAPTION),
            x + thumb,
            y + 38.0,
            scale::CAPTION,
            colors::TEXT_DIM(),
        );
        entered = button_at(
            Rect::new(x + w - enter_w, y, enter_w, BUTTON_H),
            if active { "Open" } else { "Enter" },
            true,
            Tone::Positive,
        );
    }
    flow.advance(row_h);
    entered
}

/// Actions from the city map UI
#[derive(Clone, Debug)]
pub enum CityMapAction {
    SelectNeighborhood(u32),
    ClearNeighborhood,
    OpenMarket,
    CloseMarket,
    ReviewPurchase(u32),
    CancelPurchase,
    ConfirmPurchase(u32),
    EnterBuilding(usize),
    SetMarketPage(usize),
}

mod market;
pub use market::draw_market_panel;

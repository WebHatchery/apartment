//! New Game page: the grid of campaign buildings to start from.

use crate::assets::AssetManager;
use crate::data::templates::BuildingTemplate;
use crate::save::PlayerProgress;
use crate::ui::theme::{color, Tone};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width};

const CARD_W: f32 = 280.0;
const CARD_SPACING: f32 = 20.0;
const GRID_EDGE_MARGIN: f32 = 24.0;

fn grid_top() -> f32 {
    if screen_height() < 520.0 {
        142.0
    } else if screen_height() < 660.0 {
        180.0
    } else {
        screen_height() * 0.39
    }
}

fn card_spacing() -> f32 {
    if screen_height() < 660.0 {
        8.0
    } else {
        CARD_SPACING
    }
}

fn card_height(count: usize) -> f32 {
    let rows = count.div_ceil(grid_columns(count)).max(1);
    let spacing = card_spacing();
    let available = (screen_height() - grid_top() - 8.0).max(180.0);
    let minimum = if screen_height() < 520.0 { 72.0 } else { 82.0 };
    ((available - spacing * (rows.saturating_sub(1)) as f32) / rows as f32).clamp(minimum, 120.0)
}

fn grid_columns(count: usize) -> usize {
    let fit = ((screen_width() - GRID_EDGE_MARGIN * 2.0 + CARD_SPACING) / (CARD_W + CARD_SPACING))
        .floor() as usize;
    fit.clamp(1, count.max(1))
}

/// Rect for building card `i`; rows wrap to fit the screen width and each row
/// is centered. Shared by hit-testing and rendering so they can't drift apart.
fn card_rect(i: usize, count: usize) -> Rect {
    let columns = grid_columns(count);
    let row = i / columns;
    let col = i % columns;
    let cards_in_row = (count - row * columns).min(columns);
    let spacing = card_spacing();
    let card_h = card_height(count);
    let row_width = cards_in_row as f32 * (CARD_W + spacing) - spacing;
    Rect::new(
        (screen_width() - row_width) / 2.0 + col as f32 * (CARD_W + spacing),
        grid_top() + row as f32 * (card_h + spacing),
        CARD_W,
        card_h,
    )
}

/// Return to the home menu. Top-left stays clear of the centered logo at
/// every supported size.
pub(super) fn back_rect() -> Rect {
    Rect::new(12.0, 12.0, 110.0, 40.0)
}

/// The unlocked building under `pointer`, if any.
pub(super) fn clicked_template<'a>(
    templates: &'a [BuildingTemplate],
    progress: &PlayerProgress,
    pointer: Vec2,
) -> Option<&'a BuildingTemplate> {
    let count = templates.len();
    templates.iter().enumerate().find_map(|(i, template)| {
        (progress.is_unlocked(&template.id) && card_rect(i, count).contains(pointer))
            .then_some(template)
    })
}

pub(super) fn draw(
    templates: &[BuildingTemplate],
    progress: &PlayerProgress,
    has_save: bool,
    logo_w: f32,
    assets: &AssetManager,
) {
    crate::ui::widgets::button_at(back_rect(), "Back", true, Tone::Secondary);

    if has_save {
        draw_overwrite_notice(logo_w);
    }

    // On very short screens the logo and cards communicate the same
    // instruction without competing for the narrow strip between them.
    if screen_height() >= 520.0 {
        let section_title = "Choose a building";
        let section_size = 28.0;
        let section_width = measure_ui_text(section_title, None, section_size as u16, 1.0).width;
        draw_ui_text(
            section_title,
            screen_width() / 2.0 - section_width / 2.0,
            grid_top() - 8.0,
            section_size,
            color::TEXT_BRIGHT(),
        );
    }

    let (mx, my) = mouse_position();
    let count = templates.len();
    for (i, template) in templates.iter().enumerate() {
        let rect = card_rect(i, count);
        let is_unlocked = progress.is_unlocked(&template.id);
        let is_completed = progress.completed_buildings.contains(&template.id);
        let is_hovered = rect.contains(vec2(mx, my));
        draw_building_card(
            rect,
            template,
            is_unlocked,
            is_completed,
            is_hovered,
            assets,
        );
    }
}

/// Starting a building replaces the single save slot at the next autosave,
/// so say so before the player commits. Sits top-right, beside the logo.
fn draw_overwrite_notice(logo_w: f32) {
    let size = 13.0;
    let line_h = 17.0;
    let max_w = ((screen_width() - logo_w) / 2.0 - 40.0).clamp(110.0, 240.0);
    let lines = crate::ui::widgets::wrap(
        "Starting a new building replaces your saved game.",
        max_w,
        size,
    );
    let text_w = lines
        .iter()
        .map(|line| measure_ui_text(line, None, size as u16, 1.0).width)
        .fold(0.0, f32::max);
    let card = Rect::new(
        screen_width() - 12.0 - text_w - 20.0,
        12.0,
        text_w + 20.0,
        lines.len() as f32 * line_h + 14.0,
    );
    crate::ui::widgets::draw_card(card, false);
    for (i, line) in lines.iter().enumerate() {
        draw_ui_text(
            line,
            card.x + 10.0,
            card.y + 20.0 + i as f32 * line_h,
            size,
            color::WARNING(),
        );
    }
}

fn draw_building_card(
    rect: Rect,
    template: &BuildingTemplate,
    is_unlocked: bool,
    is_completed: bool,
    is_hovered: bool,
    assets: &AssetManager,
) {
    let (x, y) = (rect.x, rect.y);
    let (card_w, card_h) = (rect.w, rect.h);

    crate::ui::widgets::draw_card(rect, is_unlocked && is_hovered);
    if !is_unlocked {
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.02, 0.02, 0.02, 0.38),
        );
    }

    let art_w = if card_h < 90.0 { 64.0 } else { 78.0 };
    let art_rect = Rect::new(x + card_w - art_w - 8.0, y + 8.0, art_w, card_h - 16.0);
    let has_facade = draw_campaign_facade(template.neighborhood_id, art_rect, is_unlocked, assets);
    let text_w = if has_facade {
        card_w - art_w - 38.0
    } else {
        card_w - 30.0
    };

    // Border color based on difficulty
    let border_color = match template.difficulty.as_str() {
        "Easy" => Color::from_rgba(80, 180, 80, 255),
        "Medium" => Color::from_rgba(200, 180, 60, 255),
        "Hard" => Color::from_rgba(200, 80, 80, 255),
        _ => Color::from_rgba(100, 100, 100, 255),
    };
    draw_rectangle(x, y, 4.0, card_h, border_color);

    // Building name
    let name_color = if is_unlocked {
        WHITE
    } else {
        Color::from_rgba(100, 100, 100, 255)
    };
    let compact_card = card_h < 105.0;
    let name_size = if compact_card { 19.0 } else { 22.0 };
    draw_ui_text(
        &truncate_text_to_width(&template.name, text_w, name_size),
        x + 15.0,
        y + if compact_card { 24.0 } else { 30.0 },
        name_size,
        name_color,
    );

    // Difficulty badge
    draw_ui_text(
        &template.difficulty,
        x + 15.0,
        y + if compact_card { 43.0 } else { 52.0 },
        14.0,
        border_color,
    );

    let desc_color = if is_unlocked {
        color::TEXT_DIM()
    } else {
        Color::new(0.42, 0.40, 0.37, 1.0)
    };
    if !compact_card {
        draw_ui_text(
            &truncate_text_to_width(&template.description, text_w, 13.0),
            x + 15.0,
            y + 75.0,
            13.0,
            desc_color,
        );
    }

    // Locked cards use the final line for the concrete progression
    // requirement; unlocked cards retain the unit-count summary.
    let footer_text = if is_unlocked {
        format!("{} units", template.apartments.len())
    } else if template.unlock_order == 0 {
        "Complete the previous property".to_string()
    } else {
        format!("Complete property {}", template.unlock_order)
    };
    draw_ui_text(
        &truncate_text_to_width(&footer_text, text_w, 14.0),
        x + 15.0,
        y + card_h - 12.0,
        14.0,
        desc_color,
    );

    if !is_unlocked {
        draw_ui_text(
            "LOCKED",
            art_rect.x + 6.0,
            art_rect.bottom() - 6.0,
            13.0,
            Color::from_rgba(150, 100, 100, 255),
        );
    }

    if is_completed {
        draw_ui_text(
            "DONE",
            x + card_w - 30.0,
            y + 30.0,
            14.0,
            Color::from_rgba(80, 200, 80, 255),
        );
    }
}

pub(super) fn campaign_facade_tile(neighborhood_id: u32) -> (f32, f32) {
    match neighborhood_id {
        0 => (0.0, 0.0), // Downtown
        2 => (1.0, 0.0), // Industrial
        3 => (0.0, 1.0), // Historic
        _ => (1.0, 1.0), // Suburbs and safe fallback
    }
}

fn draw_campaign_facade(
    neighborhood_id: u32,
    rect: Rect,
    unlocked: bool,
    assets: &AssetManager,
) -> bool {
    let Some(texture) = assets.get_texture("property_facades") else {
        return false;
    };
    let (column, row) = campaign_facade_tile(neighborhood_id);
    let tile_w = texture.width() * 0.5;
    let tile_h = texture.height() * 0.5;
    draw_texture_ex(
        texture,
        rect.x,
        rect.y,
        if unlocked {
            WHITE
        } else {
            Color::new(0.42, 0.4, 0.38, 0.72)
        },
        DrawTextureParams {
            dest_size: Some(vec2(rect.w, rect.h)),
            source: Some(Rect::new(column * tile_w, row * tile_h, tile_w, tile_h)),
            ..Default::default()
        },
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::new(0.8, 0.7, 0.55, 0.45),
    );
    true
}

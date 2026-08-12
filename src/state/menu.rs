use super::{GameplayState, StateTransition};
use crate::assets::AssetManager;
use crate::data::templates::{load_templates, BuildingTemplate};
use crate::save::{has_save_game, load_game, load_player_progress, PlayerProgress};
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

fn grid_bottom(count: usize) -> f32 {
    if count == 0 {
        return grid_top();
    }
    let rows = count.div_ceil(grid_columns(count));
    let spacing = card_spacing();
    grid_top() + rows as f32 * (card_height(count) + spacing) - spacing
}

fn continue_rect(count: usize) -> Rect {
    let w = 200.0;
    let h = 44.0;
    if screen_height() < 660.0 {
        Rect::new(screen_width() - w - 12.0, 12.0, w, h)
    } else {
        Rect::new(
            screen_width() / 2.0 - w / 2.0,
            grid_bottom(count) + 24.0,
            w,
            h,
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn quit_rect() -> Rect {
    let w = 150.0;
    let h = 40.0;
    if screen_height() < 660.0 {
        Rect::new(12.0, 12.0, w, h)
    } else {
        Rect::new(screen_width() / 2.0 - w / 2.0, screen_height() - 80.0, w, h)
    }
}

pub struct MenuState {
    has_save: bool,
    progress: PlayerProgress,
    templates: Vec<BuildingTemplate>,
    load_error: Option<String>,
}

impl MenuState {
    pub fn new() -> Self {
        let templates = load_templates().map(|t| t.templates).unwrap_or_default();

        Self {
            has_save: has_save_game(),
            progress: load_player_progress(),
            templates,
            load_error: None,
        }
    }

    pub fn update(
        &mut self,
        _assets: &AssetManager,
        config: &crate::data::config::GameConfig,
    ) -> Option<StateTransition> {
        let (mx, my) = mouse_position();
        let clicked = is_mouse_button_pressed(MouseButton::Left);

        // Building cards
        let count = self.templates.len();
        for (i, template) in self.templates.iter().enumerate() {
            let rect = card_rect(i, count);
            let is_unlocked = self.progress.is_unlocked(&template.id);

            if is_unlocked && clicked && rect.contains(vec2(mx, my)) {
                // Start game with this building template
                let state = GameplayState::new_with_template(config.clone(), template.clone());
                return Some(StateTransition::ToGameplay(state));
            }
        }

        // Continue button (if save exists)
        if self.has_save {
            let rect = continue_rect(count);

            if clicked && rect.contains(vec2(mx, my)) {
                if let Ok(state) = load_game() {
                    return Some(StateTransition::ToGameplay(state));
                } else {
                    self.load_error = Some(
                        "That saved game could not be opened. Tap a building to start again."
                            .to_string(),
                    );
                }
            }
        }

        // Quit button (native only — a browser tab has nothing to exit, and
        // std::process::exit is a no-op/unsupported on wasm).
        #[cfg(not(target_arch = "wasm32"))]
        {
            if clicked && quit_rect().contains(vec2(mx, my)) {
                std::process::exit(0);
            }
        }

        None
    }

    pub fn draw(&self, assets: &AssetManager) {
        // Background
        if let Some(bg) = assets.get_texture("title_background") {
            draw_texture_ex(
                bg,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(screen_width(), screen_height())),
                    ..Default::default()
                },
            );
        } else {
            clear_background(Color::from_rgba(25, 25, 30, 255));
        }

        // On very short screens the logo and cards communicate the same
        // instruction without competing for the narrow strip between them.
        if screen_height() >= 520.0 {
            let section_title = "Tap a building to begin";
            let section_size = 28.0;
            let section_width =
                measure_ui_text(section_title, None, section_size as u16, 1.0).width;
            draw_ui_text(
                section_title,
                screen_width() / 2.0 - section_width / 2.0,
                grid_top() - 8.0,
                section_size,
                crate::ui::theme::color::TEXT_BRIGHT(),
            );
        }

        let (mx, my) = mouse_position();

        if let Some(error) = &self.load_error {
            let error_w = (screen_width() - 32.0).min(620.0);
            let error_x = (screen_width() - error_w) / 2.0;
            crate::ui::widgets::draw_card(
                Rect::new(error_x, grid_top() - 44.0, error_w, 34.0),
                true,
            );
            draw_ui_text(
                &truncate_text_to_width(error, error_w - 24.0, 14.0),
                error_x + 12.0,
                grid_top() - 21.0,
                14.0,
                crate::ui::theme::color::NEGATIVE(),
            );
        }

        // Draw building cards
        let count = self.templates.len();
        for (i, template) in self.templates.iter().enumerate() {
            let rect = card_rect(i, count);
            let (x, y) = (rect.x, rect.y);
            let (card_w, card_h) = (rect.w, rect.h);

            let is_unlocked = self.progress.is_unlocked(&template.id);
            let is_completed = self.progress.completed_buildings.contains(&template.id);
            let is_hovered = rect.contains(vec2(mx, my));

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
            draw_ui_text(
                &template.name,
                x + 15.0,
                y + if compact_card { 24.0 } else { 30.0 },
                if compact_card { 19.0 } else { 22.0 },
                name_color,
            );

            // Difficulty badge
            let diff_color = border_color;
            draw_ui_text(
                &template.difficulty,
                x + 15.0,
                y + if compact_card { 43.0 } else { 52.0 },
                14.0,
                diff_color,
            );

            let desc_color = if is_unlocked {
                crate::ui::theme::color::TEXT_DIM()
            } else {
                Color::new(0.42, 0.40, 0.37, 1.0)
            };
            if !compact_card {
                draw_ui_text(
                    &truncate_text_to_width(&template.description, card_w - 30.0, 13.0),
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
                &truncate_text_to_width(&footer_text, card_w - 30.0, 14.0),
                x + 15.0,
                y + card_h - 12.0,
                14.0,
                desc_color,
            );

            // Locked overlay
            if !is_unlocked {
                draw_ui_text(
                    "LOCKED",
                    x + card_w - 90.0,
                    y + if compact_card { 24.0 } else { 30.0 },
                    16.0,
                    Color::from_rgba(150, 100, 100, 255),
                );
            }

            // Completed checkmark
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

        // Continue button (if save exists)
        if self.has_save {
            crate::ui::widgets::button_at(
                continue_rect(count),
                "Continue saved game",
                true,
                crate::ui::theme::Tone::Positive,
            );
        }

        // Quit button — native only (see update()).
        #[cfg(not(target_arch = "wasm32"))]
        {
            let rect = quit_rect();
            let quit_btn_w = rect.w;
            let quit_btn_h = rect.h;
            let quit_btn_x = rect.x;
            let quit_btn_y = rect.y;

            crate::ui::widgets::button_at(
                Rect::new(quit_btn_x, quit_btn_y, quit_btn_w, quit_btn_h),
                "Quit",
                true,
                crate::ui::theme::Tone::Danger,
            );
        }

        // Textured branding is intentionally the final layer. Toolkit card
        // surfaces may batch after earlier textures even when their bounds do
        // not overlap, so this preserves the logo across every card layout.
        draw_title_logo(assets);
    }
}

fn draw_title_logo(assets: &AssetManager) {
    if let Some(logo) = assets.get_texture("title_logo") {
        let (logo_w, logo_h, logo_y) = if screen_height() < 520.0 {
            (280.0, 126.0, 4.0)
        } else if screen_height() < 660.0 {
            (300.0, 135.0, 8.0)
        } else {
            (400.0, 180.0, 40.0)
        };
        draw_texture_ex(
            logo,
            screen_width() / 2.0 - logo_w / 2.0,
            logo_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(logo_w, logo_h)),
                ..Default::default()
            },
        );
    } else {
        let title = "SECOND STORY";
        let title_size = 60.0;
        let title_width = measure_ui_text(title, None, title_size as u16, 1.0).width;
        draw_ui_text(
            title,
            screen_width() / 2.0 - title_width / 2.0,
            120.0,
            title_size,
            WHITE,
        );
    }
}

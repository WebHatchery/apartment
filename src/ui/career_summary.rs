//! Responsive end-of-career report.

use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, format_money, progress_bar, truncate_text_to_width};

use crate::assets::AssetManager;
use crate::narrative::achievements::{Achievement, AchievementCondition};
use crate::state::GameplayState;

use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, draw_card, line_height};
use super::UiAction;

pub fn draw_career_summary(state: &GameplayState, assets: &AssetManager) -> Option<UiAction> {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        color::BACKGROUND(),
    );

    let page = Rect::new(
        space::LG,
        space::LG,
        screen_width() - space::LG * 2.0,
        screen_height() - space::LG * 2.0,
    );
    let score = career_score(state);
    let (rank, rank_color) = career_rank(score);

    draw_ui_text(
        "Career summary",
        page.x,
        page.y + scale::TITLE,
        scale::TITLE,
        color::TEXT_BRIGHT(),
    );
    draw_ui_text(
        "The final ledger for everything you built together.",
        page.x,
        page.y + scale::TITLE + line_height(scale::LABEL),
        scale::LABEL,
        color::TEXT_DIM(),
    );

    let summary_y = page.y + 52.0;
    draw_summary_banner(page.x, summary_y, page.w, rank, score, rank_color);

    let stats_y = summary_y + 88.0;
    draw_ui_text(
        "FINAL PORTFOLIO",
        page.x,
        stats_y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    let stats = career_stats(state);
    let stat_y = stats_y + line_height(scale::LABEL) + space::XS;
    draw_stat_cards(page, stat_y, &stats);

    let achievement_y = stat_y + 72.0 + space::LG;
    let button_h = 48.0;
    let button_y = page.bottom() - button_h;
    if let Some(action) =
        draw_achievements(state, page, achievement_y, button_y - space::LG, assets)
    {
        return Some(action);
    }

    let button_w = page.w.min(280.0);
    if button_at(
        Rect::new(page.right() - button_w, button_y, button_w, button_h),
        "Return to menu",
        true,
        Tone::Primary,
    ) {
        return Some(UiAction::ReturnToMenu);
    }
    None
}

fn career_score(state: &GameplayState) -> i32 {
    let happiness = average_happiness(state);
    let reputation = average_reputation(state);
    state.funds.balance
        + happiness * 100
        + reputation * 50
        + state.achievements.unlocked.len() as i32 * 1000
}

fn career_rank(score: i32) -> (&'static str, Color) {
    if score > 50_000 {
        ("Real estate tycoon", color::POSITIVE())
    } else if score > 25_000 {
        ("Successful landlord", color::POSITIVE())
    } else if score > 10_000 {
        ("Property manager", color::ACCENT())
    } else if score > 0 {
        ("Struggling owner", color::WARNING())
    } else {
        ("Building in distress", color::NEGATIVE())
    }
}

fn average_happiness(state: &GameplayState) -> i32 {
    if state.tenants.is_empty() {
        0
    } else {
        state
            .tenants
            .iter()
            .map(|tenant| tenant.happiness)
            .sum::<i32>()
            / state.tenants.len() as i32
    }
}

fn average_reputation(state: &GameplayState) -> i32 {
    state
        .city
        .neighborhoods
        .iter()
        .map(|neighborhood| neighborhood.reputation)
        .sum::<i32>()
        / state.city.neighborhoods.len().max(1) as i32
}

fn draw_summary_banner(x: f32, y: f32, width: f32, rank: &str, score: i32, rank_color: Color) {
    let rect = Rect::new(x, y, width, 72.0);
    draw_card(rect, true);
    draw_ui_text(
        "YOUR LEGACY",
        rect.x + space::PAD,
        rect.y + 22.0,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    draw_ui_text(
        rank,
        rect.x + space::PAD,
        rect.y + 51.0,
        scale::TITLE,
        rank_color,
    );
    let score_text = format!("{score} points");
    let score_w =
        macroquad_toolkit::ui::measure_ui_text(&score_text, None, scale::HEADING as u16, 1.0).width;
    draw_ui_text(
        &score_text,
        rect.right() - space::PAD - score_w,
        rect.y + 45.0,
        scale::HEADING,
        color::TEXT_BRIGHT(),
    );
}

fn career_stats(state: &GameplayState) -> [(String, String, Color); 5] {
    [
        (
            "Cash".to_string(),
            format_money(state.funds.balance as i64),
            color::POSITIVE(),
        ),
        (
            "Happiness".to_string(),
            format!("{}%", average_happiness(state)),
            color::TEXT(),
        ),
        (
            "Reputation".to_string(),
            average_reputation(state).to_string(),
            color::ACCENT(),
        ),
        (
            "Months".to_string(),
            state.current_tick.to_string(),
            color::TEXT(),
        ),
        (
            "Missions".to_string(),
            state.missions.completed_missions().len().to_string(),
            color::PRIMARY(),
        ),
    ]
}

fn draw_stat_cards(page: Rect, y: f32, stats: &[(String, String, Color); 5]) {
    let gap = space::SM;
    let width = (page.w - gap * 4.0) / 5.0;
    for (index, (label, value, value_color)) in stats.iter().enumerate() {
        let rect = Rect::new(page.x + index as f32 * (width + gap), y, width, 64.0);
        draw_card(rect, false);
        draw_ui_text(
            label,
            rect.x + space::MD,
            rect.y + 21.0,
            scale::CAPTION,
            color::TEXT_DIM(),
        );
        draw_ui_text(
            &truncate_text_to_width(value, rect.w - space::MD * 2.0, scale::HEADING),
            rect.x + space::MD,
            rect.y + 49.0,
            scale::HEADING,
            *value_color,
        );
    }
}

fn draw_achievements(
    state: &GameplayState,
    page: Rect,
    y: f32,
    bottom: f32,
    assets: &AssetManager,
) -> Option<UiAction> {
    let unlocked = state.achievements.unlocked.len();
    let total = state.achievements.list.len().max(1);
    draw_ui_text(
        &format!("CAREER BADGES · {unlocked}/{total} UNLOCKED"),
        page.x,
        y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    let meter_w = page.w.min(260.0);
    progress_bar(
        page.right() - meter_w,
        y + 2.0,
        meter_w,
        14.0,
        unlocked as f32,
        total as f32,
        color::PRIMARY(),
    );

    let grid_y = y + line_height(scale::LABEL) + space::SM;
    let columns = if page.w >= 1000.0 { 4 } else { 3 };
    let gap = space::SM;
    let card_h = 68.0;
    let pager_h = 48.0;
    let rows = ((bottom - grid_y - pager_h + gap) / (card_h + gap))
        .floor()
        .max(1.0) as usize;
    let page_size = rows * columns;
    let page_count = total.div_ceil(page_size);
    let badge_page = state.career_badges_page.min(page_count - 1);
    let card_w = (page.w - gap * (columns - 1) as f32) / columns as f32;

    let ordered = state
        .achievements
        .list
        .iter()
        .filter(|achievement| state.achievements.is_unlocked(&achievement.id))
        .chain(
            state
                .achievements
                .list
                .iter()
                .filter(|achievement| !state.achievements.is_unlocked(&achievement.id)),
        );
    for (index, achievement) in ordered
        .skip(badge_page * page_size)
        .take(page_size)
        .enumerate()
    {
        let row = index / columns;
        let column = index % columns;
        draw_achievement_card(
            state,
            achievement,
            Rect::new(
                page.x + column as f32 * (card_w + gap),
                grid_y + row as f32 * (card_h + gap),
                card_w,
                card_h,
            ),
            assets,
        );
    }
    if page_count > 1 {
        let controls_y = bottom - 40.0;
        let controls_w = page.w.min(360.0);
        let gap = space::SM;
        let button_w = (controls_w - gap) * 0.5;
        if button_at(
            Rect::new(page.x, controls_y, button_w, 40.0),
            "Earlier badges",
            badge_page > 0,
            Tone::Secondary,
        ) {
            return Some(UiAction::SetCareerBadgesPage {
                page: badge_page - 1,
            });
        }
        if button_at(
            Rect::new(page.x + button_w + gap, controls_y, button_w, 40.0),
            "More badges",
            badge_page + 1 < page_count,
            Tone::Primary,
        ) {
            return Some(UiAction::SetCareerBadgesPage {
                page: badge_page + 1,
            });
        }
    }
    None
}

fn draw_achievement_card(
    state: &GameplayState,
    achievement: &Achievement,
    rect: Rect,
    assets: &AssetManager,
) {
    let unlocked = state.achievements.is_unlocked(&achievement.id);
    draw_card(rect, unlocked);
    let emblem_rect = Rect::new(rect.x + space::SM, rect.y + 8.0, 52.0, 52.0);
    draw_achievement_emblem(achievement, emblem_rect, unlocked, assets);
    let text_x = emblem_rect.right() + space::SM;
    let text_w = (rect.right() - space::SM - text_x).max(24.0);
    draw_ui_text(
        &truncate_text_to_width(&achievement.name, text_w, scale::BODY),
        text_x,
        rect.y + 24.0,
        scale::BODY,
        if unlocked {
            color::TEXT_BRIGHT()
        } else {
            color::TEXT_DIM()
        },
    );
    draw_ui_text(
        &truncate_text_to_width(&achievement.description, text_w, scale::CAPTION),
        text_x,
        rect.y + 49.0,
        scale::CAPTION,
        color::TEXT_DIM(),
    );
}

fn achievement_emblem_tile(condition: &AchievementCondition) -> (f32, f32) {
    match condition {
        AchievementCondition::TotalTenants { .. } | AchievementCondition::FullOccupancy => {
            (0.0, 0.0)
        }
        AchievementCondition::AvgHappiness { .. }
        | AchievementCondition::HappinessAtLeast { .. }
        | AchievementCondition::MaxReputation { .. } => (1.0, 0.0),
        AchievementCondition::Funds { .. } => (0.0, 1.0),
        AchievementCondition::GameComplete => (1.0, 1.0),
    }
}

fn draw_achievement_emblem(
    achievement: &Achievement,
    rect: Rect,
    unlocked: bool,
    assets: &AssetManager,
) {
    let Some(texture) = assets.get_texture("achievement_emblems") else {
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, color::SURFACE_ALT());
        draw_ui_text(
            if unlocked { "◆" } else { "◇" },
            rect.x + 15.0,
            rect.y + 36.0,
            scale::HEADING,
            if unlocked {
                color::PRIMARY()
            } else {
                color::TEXT_DIM()
            },
        );
        return;
    };
    let (column, row) = achievement_emblem_tile(&achievement.condition);
    let tile_w = texture.width() * 0.5;
    let tile_h = texture.height() * 0.5;
    draw_texture_ex(
        texture,
        rect.x,
        rect.y,
        if unlocked {
            WHITE
        } else {
            Color::new(0.4, 0.38, 0.36, 0.62)
        },
        DrawTextureParams {
            dest_size: Some(vec2(rect.w, rect.h)),
            source: Some(Rect::new(column * tile_w, row * tile_h, tile_w, tile_h)),
            ..Default::default()
        },
    );
}

#[cfg(test)]
mod tests;

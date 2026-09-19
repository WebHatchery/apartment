//! Persistent navigation between the game's management workspaces.

use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

use super::theme::{color, space, Tone};
use super::widgets::button_at;
use super::UiAction;
use crate::assets::AssetManager;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceTab {
    Building,
    Tenants,
    Finances,
    City,
    Inbox,
    Tasks,
}

pub fn draw_workspace_nav(
    active: WorkspaceTab,
    unread_mail: usize,
    pending_tasks: usize,
    assets: &AssetManager,
) -> Option<UiAction> {
    let desktop = screen_width() >= 1180.0;
    let y = if desktop {
        0.0
    } else {
        apartment::screen_layout::status_height(screen_width())
    };
    let h = if desktop { 56.0 } else { 48.0 };
    if desktop {
        draw_ui_text("Second Story", 12.0, 34.0, 22.0, color::TEXT_BRIGHT());
    }
    let labels = [
        (
            WorkspaceTab::Building,
            "Building",
            "icon_key",
            UiAction::OpenBuilding,
        ),
        (
            WorkspaceTab::Tenants,
            "People",
            "icon_application",
            UiAction::OpenTenants,
        ),
        (
            WorkspaceTab::Finances,
            "Finances",
            "icon_money",
            UiAction::OpenFinances,
        ),
        (
            WorkspaceTab::City,
            "City",
            "icon_market",
            UiAction::OpenCityMap,
        ),
        (
            WorkspaceTab::Inbox,
            "Inbox",
            "icon_mail",
            UiAction::OpenMail,
        ),
    ];
    let gap = 4.0;
    let outer = if desktop { 155.0 } else { 8.0 };
    let available = if desktop {
        screen_width() - 520.0 - outer
    } else {
        screen_width() - 16.0
    };
    let button_w = ((available - gap * 4.0) / 5.0).min(96.0);
    let button_h = 40.0;
    let mut x = outer;
    let mut action = None;

    for (tab, base_label, icon_id, intent) in labels {
        let count = match tab {
            WorkspaceTab::Inbox => unread_mail + pending_tasks,
            WorkspaceTab::Tasks => pending_tasks,
            _ => 0,
        };
        let base_label = if screen_width() < 700.0 {
            match tab {
                WorkspaceTab::Building => "Build",
                WorkspaceTab::Tenants => "People",
                WorkspaceTab::Finances => "Money",
                WorkspaceTab::City => "City",
                WorkspaceTab::Inbox => "Inbox",
                WorkspaceTab::Tasks => "Tasks",
            }
        } else {
            base_label
        };
        let label = if count > 0 {
            format!("{} *", base_label)
        } else {
            base_label.to_string()
        };
        let tone = if tab == active || (tab == WorkspaceTab::Inbox && active == WorkspaceTab::Tasks)
        {
            Tone::Primary
        } else {
            Tone::Secondary
        };
        let rect = Rect::new(x, y + (h - button_h) / 2.0, button_w, button_h);
        if button_at(rect, "", true, tone) {
            action = Some(intent);
        }
        draw_nav_content(rect, &label, icon_id, assets, tab == active);
        x += button_w + gap;
    }
    action
}

fn draw_nav_content(rect: Rect, label: &str, icon_id: &str, assets: &AssetManager, active: bool) {
    let font_size = super::theme::scale::LABEL;
    let text_w = measure_ui_text(label, None, font_size as u16, 1.0).width;
    let icon_size = if rect.w < 88.0 { 0.0 } else { 18.0 };
    let has_icon =
        assets.get_texture("workspace_icons").is_some() || assets.get_texture(icon_id).is_some();
    let icon_gap = if has_icon && icon_size > 0.0 {
        space::SM
    } else {
        0.0
    };
    let group_w = text_w + icon_gap + if has_icon { icon_size } else { 0.0 };
    let mut cx = rect.x + (rect.w - group_w) / 2.0;
    if icon_size > 0.0
        && draw_workspace_icon(
            icon_id,
            Rect::new(
                cx,
                rect.y + (rect.h - icon_size) / 2.0,
                icon_size,
                icon_size,
            ),
            active,
            assets,
        )
    {
        cx += icon_size + icon_gap;
    }
    draw_ui_text(
        label,
        cx,
        rect.y + rect.h / 2.0 + font_size / 2.0 - 1.0,
        font_size,
        if active {
            Color::new(0.10, 0.08, 0.04, 1.0)
        } else {
            color::TEXT()
        },
    );
}

fn workspace_icon_tile(id: &str) -> Option<(f32, f32)> {
    Some(match id {
        "icon_key" => (0.0, 0.0),
        "icon_application" => (1.0, 0.0),
        "icon_money" => (2.0, 0.0),
        "icon_market" => (0.0, 1.0),
        "icon_mail" => (1.0, 1.0),
        "icon_inspection" => (2.0, 1.0),
        _ => return None,
    })
}

fn draw_workspace_icon(id: &str, rect: Rect, active: bool, assets: &AssetManager) -> bool {
    if let (Some(texture), Some((column, row))) = (
        assets.get_texture("workspace_icons"),
        workspace_icon_tile(id),
    ) {
        let tile_w = texture.width() / 3.0;
        let tile_h = texture.height() * 0.5;
        let inset_x = tile_w * 0.1;
        let inset_y = tile_h * 0.1;
        draw_texture_ex(
            texture,
            rect.x,
            rect.y,
            if active {
                Color::new(0.35, 0.24, 0.12, 1.0)
            } else {
                WHITE
            },
            DrawTextureParams {
                dest_size: Some(vec2(rect.w, rect.h)),
                source: Some(Rect::new(
                    column * tile_w + inset_x,
                    row * tile_h + inset_y,
                    tile_w - inset_x * 2.0,
                    tile_h - inset_y * 2.0,
                )),
                ..Default::default()
            },
        );
        true
    } else if let Some(texture) = assets.get_texture(id) {
        draw_texture_ex(
            texture,
            rect.x,
            rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(rect.w, rect.h)),
                ..Default::default()
            },
        );
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests;

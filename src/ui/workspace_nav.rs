//! Persistent navigation between the game's management workspaces.

use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_surface, draw_ui_text, measure_ui_text, SurfaceStyle};

use super::theme::{color, space, Tone};
use super::widgets::button_at;
use super::UiAction;
use crate::assets::AssetManager;

pub const STATUS_BAR_HEIGHT: f32 = 64.0;

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
    let y = STATUS_BAR_HEIGHT;
    let h = (crate::ui::layout::HEADER_HEIGHT() - y).max(40.0);
    draw_surface(
        Rect::new(0.0, y, screen_width(), h),
        &SurfaceStyle::new(color::SURFACE_HEADER()).with_border(1.0, color::BORDER_STRONG()),
    );

    let labels = [
        (
            WorkspaceTab::Building,
            "Building",
            "icon_key",
            UiAction::OpenBuilding,
        ),
        (
            WorkspaceTab::Tenants,
            "Tenants",
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
        (
            WorkspaceTab::Tasks,
            "Tasks",
            "icon_inspection",
            UiAction::OpenTasks,
        ),
    ];
    let gap = if screen_width() < 900.0 {
        space::XS
    } else {
        space::SM
    };
    let outer = if screen_width() < 900.0 {
        space::SM
    } else {
        space::LG
    };
    let available = screen_width() - outer * 2.0 - gap * (labels.len() as f32 - 1.0);
    let button_w = available / labels.len() as f32;
    let button_h = h - space::SM * 2.0;
    let mut x = outer;
    let mut action = None;

    for (tab, base_label, icon_id, intent) in labels {
        let count = match tab {
            WorkspaceTab::Inbox => unread_mail,
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
            format!("{} ({})", base_label, count)
        } else {
            base_label.to_string()
        };
        let tone = if tab == active {
            Tone::Primary
        } else {
            Tone::Secondary
        };
        let rect = Rect::new(x, y + space::SM, button_w, button_h);
        if button_at(rect, "", true, tone) {
            action = Some(intent);
        }
        draw_nav_content(rect, &label, assets.get_texture(icon_id), tab == active);
        x += button_w + gap;
    }
    action
}

fn draw_nav_content(rect: Rect, label: &str, icon: Option<&Texture2D>, active: bool) {
    let font_size = super::theme::scale::LABEL;
    let text_w = measure_ui_text(label, None, font_size as u16, 1.0).width;
    let icon_size = if rect.w < 88.0 { 0.0 } else { 18.0 };
    let icon_gap = if icon.is_some() && icon_size > 0.0 {
        space::SM
    } else {
        0.0
    };
    let group_w = text_w + icon_gap + if icon.is_some() { icon_size } else { 0.0 };
    let mut cx = rect.x + (rect.w - group_w) / 2.0;
    if let Some(texture) = icon.filter(|_| icon_size > 0.0) {
        draw_texture_ex(
            texture,
            cx,
            rect.y + (rect.h - icon_size) / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(icon_size, icon_size)),
                ..Default::default()
            },
        );
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

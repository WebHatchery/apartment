use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, button_width};
use super::UiAction;
use crate::assets::AssetManager;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{
    draw_surface, draw_ui_text, measure_ui_text, truncate_text_to_width, SurfaceStyle,
};

/// Draw a stat chip (optional icon + label) at `x`, vertically centered in the
/// header. Returns the chip width so callers can flow chips without overlap.
#[derive(Clone, Copy)]
enum StatusIcon {
    Cash,
    Month,
    Occupancy,
    Net,
}

fn stat_chip(
    x: f32,
    icon: StatusIcon,
    label: &str,
    text_color: Color,
    header_h: f32,
    assets: &AssetManager,
) -> f32 {
    let chip_h = 34.0;
    let chip_y = (header_h - chip_h) / 2.0;
    let compact = screen_width() < 1000.0;
    let icon_size = if compact { 0.0 } else { 20.0 };
    let text_w = measure_ui_text(label, None, scale::BODY as u16, 1.0).width;
    let has_icon = assets.get_texture("status_icons").is_some();
    let icon_w = if has_icon && !compact {
        icon_size + space::XS
    } else {
        0.0
    };
    let pad = if compact { space::SM } else { space::MD };
    let w = pad + icon_w + text_w + pad;

    let style = SurfaceStyle::new(color::SURFACE_ALT()).with_border(1.0, color::BORDER());
    draw_surface(Rect::new(x, chip_y, w, chip_h), &style);

    let mut cx = x + pad;
    if !compact
        && draw_status_icon(
            icon,
            Rect::new(
                cx,
                chip_y + (chip_h - icon_size) / 2.0,
                icon_size,
                icon_size,
            ),
            assets,
        )
    {
        cx += icon_size + space::XS;
    }
    draw_ui_text(
        label,
        cx,
        chip_y + chip_h / 2.0 + scale::BODY / 2.0 - 1.0,
        scale::BODY,
        text_color,
    );
    w
}

fn draw_status_icon(icon: StatusIcon, rect: Rect, assets: &AssetManager) -> bool {
    let Some(texture) = assets.get_texture("status_icons") else {
        return false;
    };
    let (column, row) = match icon {
        StatusIcon::Cash => (0.0, 0.0),
        StatusIcon::Month => (1.0, 0.0),
        StatusIcon::Occupancy => (0.0, 1.0),
        StatusIcon::Net => (1.0, 1.0),
    };
    let tile_w = texture.width() * 0.5;
    let tile_h = texture.height() * 0.5;
    let inset_x = tile_w * 0.08;
    let inset_y = tile_h * 0.08;
    draw_texture_ex(
        texture,
        rect.x,
        rect.y,
        WHITE,
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
}

pub fn draw_header(
    money: i32,
    monthly_net: i32,
    tick: u32,
    building_name: &str,
    occupancy: usize,
    total_units: usize,
    assets: &AssetManager,
) -> Option<UiAction> {
    let mut action = None;
    let w = screen_width();
    let h = super::workspace_nav::STATUS_BAR_HEIGHT;

    // Background + bottom hairline
    draw_rectangle(0.0, 0.0, w, h, color::SURFACE_HEADER());
    draw_line(0.0, h, w, h, 1.0, color::BORDER_STRONG());

    // Keep primary time control and recovery/menu control together so both
    // paths remain reachable with touch or a mouse.
    let btn_h = 40.0;
    let compact = w < 1000.0;
    let end_month_w = button_width("End Month", btn_h).max(if compact { 106.0 } else { 120.0 });
    let menu_w = button_width("Menu", btn_h).max(if compact { 74.0 } else { 84.0 });
    let outer = if compact { space::SM } else { space::LG };
    let end_month_x = w - end_month_w - outer;
    let menu_x = end_month_x - menu_w - space::SM;
    let btn_y = (h - btn_h) / 2.0;
    if button_at(
        Rect::new(end_month_x, btn_y, end_month_w, btn_h),
        "End Month",
        true,
        Tone::Primary,
    ) {
        action = Some(UiAction::EndTurn);
    }
    if button_at(
        Rect::new(menu_x, btn_y, menu_w, btn_h),
        "Menu",
        true,
        Tone::Secondary,
    ) {
        action = Some(UiAction::OpenPauseMenu);
    }
    // Stat cluster: money / month / occupancy chips, flowed right-to-left so
    // they hug the controls and never collide with the building name.
    let money_color = if money < 0 {
        color::NEGATIVE()
    } else if money < 500 {
        color::WARNING()
    } else {
        color::POSITIVE()
    };
    let money_label = macroquad_toolkit::ui::format_money(money as i64);
    let month_label = format!("Month {}", tick);
    let occ_label = format!("{}/{} leased", occupancy, total_units);
    let net_amount = macroquad_toolkit::ui::format_money(monthly_net.unsigned_abs() as i64);
    let net_label = if monthly_net > 0 {
        format!("Net +{}", net_amount)
    } else if monthly_net < 0 {
        format!("Net -{}", net_amount)
    } else {
        "Net $0".to_string()
    };
    let net_color = if monthly_net < 0 {
        color::NEGATIVE()
    } else if monthly_net > 0 {
        color::POSITIVE()
    } else {
        color::TEXT_DIM()
    };

    // Measure chip widths (mirror stat_chip's math) to place them.
    let chip_gap = space::SM;
    let chips: [(StatusIcon, &str, Color); 4] = [
        (StatusIcon::Cash, &money_label, money_color),
        (StatusIcon::Month, &month_label, color::TEXT()),
        (StatusIcon::Occupancy, &occ_label, color::TEXT()),
        (StatusIcon::Net, &net_label, net_color),
    ];
    let widths: Vec<f32> = chips
        .iter()
        .map(|(_, label, _)| {
            let text_w = measure_ui_text(label, None, scale::BODY as u16, 1.0).width;
            let icon_w = if assets.get_texture("status_icons").is_some() && !compact {
                20.0 + space::XS
            } else {
                0.0
            };
            let pad = if compact { space::SM } else { space::MD };
            pad + icon_w + text_w + pad
        })
        .collect();
    let cluster_w: f32 = widths.iter().sum::<f32>() + chip_gap * (chips.len() as f32 - 1.0);
    let cluster_right = menu_x - space::MD;
    let mut cx = (cluster_right - cluster_w).max(0.0);
    let cluster_left = cx;
    for (i, (icon, label, text_color)) in chips.iter().enumerate() {
        stat_chip(cx, *icon, label, *text_color, h, assets);
        cx += widths[i] + chip_gap;
    }

    // Building name, left-aligned, ellipsized to the space before the cluster.
    let name_x = outer;
    let name_avail = (cluster_left - space::MD - name_x).max(40.0);
    let name = truncate_text_to_width(building_name, name_avail, scale::TITLE);
    draw_ui_text(
        &name,
        name_x,
        h / 2.0 + scale::TITLE / 2.0 - 1.0,
        scale::TITLE,
        color::TEXT_BRIGHT(),
    );

    action
}

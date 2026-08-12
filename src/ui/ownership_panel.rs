//! Responsive ownership and condo-conversion panel.

use std::collections::HashSet;

use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, format_money, truncate_text_to_width};

use crate::building::ownership::OwnershipType;
use crate::building::{Apartment, Building};

use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, draw_card, draw_panel, line_height, wrap};
use super::UiAction;

pub fn draw_ownership_panel(
    building: &Building,
    market_multiplier: f32,
    scroll_offset: f32,
) -> (Option<UiAction>, f32) {
    let padding = crate::ui::layout::PADDING();
    let panel_x = screen_width() * crate::ui::layout::PANEL_SPLIT() + padding;
    let panel_y = crate::ui::layout::HEADER_HEIGHT() + padding;
    let panel_w = screen_width() - panel_x - padding;
    let panel_h = screen_height() - panel_y - crate::ui::layout::FOOTER_HEIGHT() - padding;
    let panel = Rect::new(panel_x, panel_y, panel_w, panel_h);
    let inner = draw_panel(panel, "Building ownership");

    let model_name = ownership_name(&building.ownership_model);
    draw_ui_text(
        model_name,
        inner.x,
        inner.y + scale::BODY,
        scale::BODY,
        color::ACCENT(),
    );
    let mut y = inner.y + line_height(scale::BODY) + space::SM;

    let (description, apartments, reserve) = ownership_details(building);
    for line in wrap(description, inner.w, scale::LABEL).iter().take(2) {
        draw_ui_text(
            line,
            inner.x,
            y + scale::LABEL,
            scale::LABEL,
            color::TEXT_DIM(),
        );
        y += line_height(scale::LABEL);
    }
    if let Some(reserve) = reserve {
        draw_ui_text(
            &format!("Board reserve: {}", format_money(reserve as i64)),
            inner.x,
            y + scale::LABEL,
            scale::LABEL,
            color::POSITIVE(),
        );
        y += line_height(scale::LABEL);
    }
    y += space::SM;

    let footer_h = 42.0;
    let footer_y = inner.bottom() - footer_h;
    if apartments.is_empty() {
        draw_ui_text(
            "No units are available for condo conversion.",
            inner.x,
            y + scale::BODY,
            scale::BODY,
            color::TEXT_DIM(),
        );
        let close = button_at(
            Rect::new(inner.x, footer_y, inner.w, footer_h),
            "Close ownership",
            true,
            Tone::Secondary,
        );
        return (close.then_some(UiAction::ClearSelection), 0.0);
    }

    draw_ui_text(
        &format!("UNITS AVAILABLE · {}", apartments.len()),
        inner.x,
        y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    y += line_height(scale::LABEL) + space::SM;

    let row_h = 58.0;
    let visible = (((footer_y - space::SM - y) / row_h).floor() as usize).max(1);
    let (first, max_first) = ownership_page(apartments.len(), visible, scroll_offset, row_h);
    let mut action = None;
    for apartment in apartments.iter().skip(first).take(visible) {
        if draw_unit_row(
            apartment,
            market_multiplier,
            Rect::new(inner.x, y, inner.w, 52.0),
        ) {
            action = Some(UiAction::SellUnitAsCondo {
                apartment_id: apartment.id,
            });
        }
        y += row_h;
    }

    let mut next_offset = first as f32 * row_h;
    let dense = inner.w < 230.0;
    let close_w = inner.w.min(if dense { 92.0 } else { 118.0 });
    if button_at(
        Rect::new(inner.x, footer_y, close_w, footer_h),
        "Close",
        true,
        Tone::Secondary,
    ) {
        action = Some(UiAction::ClearSelection);
    }
    if max_first > 0 {
        let gap = space::SM;
        let pager_x = inner.x + close_w + gap;
        let pager_w = inner.right() - pager_x;
        let button_w = (pager_w - gap) / 2.0;
        if button_at(
            Rect::new(pager_x, footer_y, button_w, footer_h),
            if dense { "Prev" } else { "Earlier" },
            first > 0,
            Tone::Secondary,
        ) {
            next_offset = first.saturating_sub(visible) as f32 * row_h;
        }
        if button_at(
            Rect::new(pager_x + button_w + gap, footer_y, button_w, footer_h),
            if dense { "Next" } else { "More" },
            first < max_first,
            Tone::Primary,
        ) {
            next_offset = (first + visible).min(max_first) as f32 * row_h;
        }
    }
    (action, next_offset)
}

fn ownership_page(len: usize, visible: usize, offset: f32, row_h: f32) -> (usize, usize) {
    let max_first = len.saturating_sub(visible);
    let first = ((offset.max(0.0) / row_h).round() as usize).min(max_first);
    (first, max_first)
}

fn ownership_name(ownership: &OwnershipType) -> &'static str {
    match ownership {
        OwnershipType::FullRental => "Full rental · sole proprietor",
        OwnershipType::MixedOwnership(_) => "Mixed ownership · partial condo",
        OwnershipType::FullCondo(_) => "Full condo association",
        OwnershipType::CooperativeHousing => "Tenant cooperative",
        OwnershipType::SocialHousing => "Social and subsidized housing",
    }
}

fn ownership_details<'a>(
    building: &'a Building,
) -> (&'static str, Vec<&'a Apartment>, Option<i32>) {
    match &building.ownership_model {
        OwnershipType::FullRental => (
            "You own every unit and collect all rent. Selling a unit raises capital but gives up its future rent.",
            building.apartments.iter().collect(),
            None,
        ),
        OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
            let sold: HashSet<u32> = board.units.iter().map(|unit| unit.apartment_id).collect();
            (
                "The condo board shares responsibility for sold units. Remaining units can still be converted.",
                building
                    .apartments
                    .iter()
                    .filter(|apartment| !sold.contains(&apartment.id))
                    .collect(),
                Some(board.reserve_fund),
            )
        }
        OwnershipType::CooperativeHousing | OwnershipType::SocialHousing => (
            "This building is governed externally, so individual condo conversion is unavailable.",
            Vec::new(),
            None,
        ),
    }
}

fn draw_unit_row(apartment: &Apartment, multiplier: f32, rect: Rect) -> bool {
    draw_card(rect, false);
    let button_w = rect.w.min(166.0);
    let text_w = (rect.w - button_w - space::MD * 3.0).max(50.0);
    draw_ui_text(
        &truncate_text_to_width(
            &format!("Unit {}", apartment.unit_number),
            text_w,
            scale::BODY,
        ),
        rect.x + space::MD,
        rect.y + 20.0,
        scale::BODY,
        color::TEXT_BRIGHT(),
    );
    draw_ui_text(
        if apartment.is_vacant() {
            "Vacant"
        } else {
            "Occupied"
        },
        rect.x + space::MD,
        rect.y + 41.0,
        scale::CAPTION,
        if apartment.is_vacant() {
            color::POSITIVE()
        } else {
            color::WARNING()
        },
    );
    let sale_price = (apartment.market_value() as f32 * multiplier) as i32;
    button_at(
        Rect::new(rect.right() - button_w - 6.0, rect.y + 6.0, button_w, 40.0),
        &format!("Sell · {}", format_money(sale_price as i64)),
        true,
        Tone::Positive,
    )
}

#[cfg(test)]
mod tests;

//! Portfolio management workspaces for tenants, finances, and the inbox.

use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, format_money, truncate_text_to_width};

use crate::assets::AssetManager;
use crate::building::MarketingType;
use crate::economy::{OperatingCosts, TransactionType};
use crate::state::GameplayState;

use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, draw_card, draw_panel, kv_row, line_height};
use super::UiAction;

fn content_rect() -> Rect {
    Rect::new(
        space::LG,
        crate::ui::layout::HEADER_HEIGHT() + space::LG,
        screen_width() - space::LG * 2.0,
        screen_height()
            - crate::ui::layout::HEADER_HEIGHT()
            - crate::ui::layout::FOOTER_HEIGHT()
            - space::LG * 2.0,
    )
}

fn draw_title(rect: Rect, title: &str, subtitle: &str) -> f32 {
    draw_ui_text(
        title,
        rect.x,
        rect.y + scale::TITLE,
        scale::TITLE,
        color::TEXT_BRIGHT(),
    );
    draw_ui_text(
        subtitle,
        rect.x,
        rect.y + scale::TITLE + line_height(scale::BODY),
        scale::BODY,
        color::TEXT_DIM(),
    );
    rect.y + scale::TITLE + line_height(scale::BODY) + space::LG
}

pub fn draw_tenants_view(state: &GameplayState, assets: &AssetManager) -> Option<UiAction> {
    let rect = content_rect();
    let active_id = state.active_building_id();
    let tenants: Vec<_> = state
        .tenants
        .iter()
        .filter(|tenant| tenant.building_id == active_id)
        .collect();
    let applications = state
        .applications
        .iter()
        .filter(|application| application.building_id == active_id)
        .count();
    let pending_requests = tenants
        .iter()
        .filter(|tenant| {
            state
                .tenant_stories
                .get(&tenant.id)
                .is_some_and(|story| story.pending_request.is_some())
        })
        .count();
    let average = if tenants.is_empty() {
        0
    } else {
        tenants.iter().map(|tenant| tenant.happiness).sum::<i32>() / tenants.len() as i32
    };
    let body_y = draw_title(
        rect,
        "Tenants",
        &format!("{} residents at {}", tenants.len(), state.building.name),
    );
    let gap = space::LG;
    let side_w = (rect.w * 0.30).clamp(220.0, 330.0);
    let roster = Rect::new(
        rect.x,
        body_y,
        rect.w - side_w - gap,
        rect.bottom() - body_y,
    );
    let side = Rect::new(roster.right() + gap, body_y, side_w, roster.h);
    let roster_inner = draw_panel(roster, "Resident roster");
    let mut y = roster_inner.y;

    if tenants.is_empty() {
        draw_ui_text(
            "No one currently lives in this building.",
            roster_inner.x,
            y + scale::BODY,
            scale::BODY,
            color::TEXT_DIM(),
        );
    }
    let row_h = 64.0;
    let visible = ((roster_inner.h / row_h).floor() as usize).max(1);
    for tenant in tenants.iter().take(visible) {
        let row = Rect::new(roster_inner.x, y, roster_inner.w, row_h - space::XS);
        draw_card(row, false);
        let portrait = Rect::new(row.x + space::SM, row.y + space::SM, 44.0, 44.0);
        super::resident_sprite::draw_face_portrait(tenant, portrait, assets);
        let text_x = portrait.right() + space::SM;
        let unit = tenant
            .apartment_id
            .and_then(|id| state.building.get_apartment(id))
            .map(|apartment| apartment.unit_number.as_str())
            .unwrap_or("—");
        let name = truncate_text_to_width(&tenant.name, row.w * 0.34, scale::BODY);
        draw_ui_text(
            &name,
            text_x,
            row.y + 21.0,
            scale::BODY,
            color::TEXT_BRIGHT(),
        );
        draw_ui_text(
            &format!("{} · Unit {}", tenant.archetype.name(), unit),
            text_x,
            row.y + 42.0,
            scale::LABEL,
            color::TEXT_DIM(),
        );
        let rent = tenant
            .apartment_id
            .and_then(|id| state.building.get_apartment(id))
            .map(|apartment| apartment.rent_price)
            .unwrap_or(0);
        let status = format!(
            "{} happy · ${}/${}",
            tenant.happiness, rent, tenant.rent_tolerance
        );
        let status_w =
            macroquad_toolkit::ui::measure_ui_text(&status, None, scale::LABEL as u16, 1.0).width;
        draw_ui_text(
            &status,
            row.right() - status_w - space::MD,
            row.y + 31.0,
            scale::LABEL,
            if rent > tenant.rent_tolerance {
                color::WARNING()
            } else {
                color::TEXT()
            },
        );
        y += row_h;
    }
    if tenants.len() > visible {
        draw_ui_text(
            &format!("+ {} more residents", tenants.len() - visible),
            roster_inner.x,
            roster_inner.bottom() - space::SM,
            scale::LABEL,
            color::TEXT_DIM(),
        );
    }

    let side_inner = draw_panel(side, "Leasing desk");
    let dense_side = side_inner.h < 220.0;
    let mut sy = side_inner.y;
    sy += kv_row(
        side_inner.x,
        sy,
        side_inner.w,
        "Occupancy",
        &format!(
            "{}/{}",
            state.building.occupancy_count(),
            state.building.rental_unit_count()
        ),
        color::TEXT(),
    );
    if !dense_side {
        sy += kv_row(
            side_inner.x,
            sy,
            side_inner.w,
            "Average happiness",
            &format!("{}%", average),
            if average >= 60 {
                color::POSITIVE()
            } else {
                color::WARNING()
            },
        );
    }
    sy += kv_row(
        side_inner.x,
        sy,
        side_inner.w,
        "Applications",
        &applications.to_string(),
        if applications > 0 {
            color::ACCENT()
        } else {
            color::TEXT_DIM()
        },
    );
    sy += kv_row(
        side_inner.x,
        sy,
        side_inner.w,
        "Pending requests",
        &pending_requests.to_string(),
        if pending_requests > 0 {
            color::WARNING()
        } else {
            color::TEXT_DIM()
        },
    );
    sy += if dense_side { space::SM } else { space::LG };
    if button_at(
        Rect::new(side_inner.x, sy, side_inner.w, 40.0),
        "Review applications",
        applications > 0,
        Tone::Primary,
    ) {
        return Some(UiAction::SelectApplications(None));
    }
    sy += 46.0;
    if button_at(
        Rect::new(side_inner.x, sy, side_inner.w, 40.0),
        "Open requests & tasks",
        true,
        Tone::Secondary,
    ) {
        return Some(UiAction::OpenTasks);
    }
    None
}

pub fn draw_finances_view(state: &GameplayState) -> Option<UiAction> {
    let rect = content_rect();
    let body_y = draw_title(
        rect,
        "Finances & policies",
        "See the ledger and decide how this building operates.",
    );
    let gap = space::LG;
    let ledger_w = (rect.w * 0.48).max(330.0);
    let ledger_rect = Rect::new(rect.x, body_y, ledger_w, rect.bottom() - body_y);
    let policy_rect = Rect::new(
        ledger_rect.right() + gap,
        body_y,
        rect.right() - ledger_rect.right() - gap,
        ledger_rect.h,
    );
    draw_ledger(state, ledger_rect);
    draw_policies(state, policy_rect)
}

fn draw_ledger(state: &GameplayState, rect: Rect) {
    let inner = draw_panel(rect, "Portfolio ledger");
    let mut y = inner.y;
    y += kv_row(
        inner.x,
        y,
        inner.w,
        "Cash balance",
        &format_money(state.funds.balance as i64),
        if state.funds.balance >= 0 {
            color::POSITIVE()
        } else {
            color::NEGATIVE()
        },
    );
    y += kv_row(
        inner.x,
        y,
        inner.w,
        "Lifetime income",
        &format_money(state.funds.total_income as i64),
        color::POSITIVE(),
    );
    y += kv_row(
        inner.x,
        y,
        inner.w,
        "Lifetime expenses",
        &format_money(state.funds.total_expenses as i64),
        color::NEGATIVE(),
    );
    y += space::MD;
    draw_ui_text(
        "RECENT MONTHS",
        inner.x,
        y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    y += line_height(scale::LABEL) + space::SM;
    for report in state.ledger.reports.iter().rev().take(4) {
        let text = format!(
            "Month {}   income {}   costs {}   net {:+}",
            report.tick,
            format_money(report.rent_income as i64),
            format_money((report.repair_costs + report.upgrade_costs) as i64),
            report.net
        );
        draw_ui_text(
            &truncate_text_to_width(&text, inner.w, scale::LABEL),
            inner.x,
            y + scale::LABEL,
            scale::LABEL,
            if report.net >= 0 {
                color::POSITIVE()
            } else {
                color::NEGATIVE()
            },
        );
        y += line_height(scale::LABEL);
    }
    y += space::MD;
    draw_ui_text(
        "LATEST TRANSACTIONS",
        inner.x,
        y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    y += line_height(scale::LABEL) + space::SM;
    let transaction_rows = ((inner.bottom() - y) / line_height(scale::LABEL))
        .floor()
        .max(0.0) as usize;
    for transaction in state
        .funds
        .transactions
        .iter()
        .rev()
        .take(transaction_rows.min(5))
    {
        let kind = transaction_type_name(&transaction.transaction_type);
        let text = format!(
            "M{} · {} · {} {:+}",
            transaction.tick, kind, transaction.description, transaction.amount
        );
        draw_ui_text(
            &truncate_text_to_width(&text, inner.w, scale::LABEL),
            inner.x,
            y + scale::LABEL,
            scale::LABEL,
            if transaction.amount >= 0 {
                color::POSITIVE()
            } else {
                color::TEXT()
            },
        );
        y += line_height(scale::LABEL);
    }
}

fn draw_policies(state: &GameplayState, rect: Rect) -> Option<UiAction> {
    let inner = draw_panel(rect, "Operations");
    let show_explanations = inner.h >= 340.0;
    let dense = inner.h < 280.0;
    let mut y = inner.y;
    draw_ui_text(
        "MARKETING",
        inner.x,
        y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    y += line_height(scale::LABEL);
    let choices = [
        (MarketingType::None, "None", "No monthly cost"),
        (MarketingType::SocialMedia, "Social", "2× Students/Artists"),
        (
            MarketingType::LocalNewspaper,
            "Local",
            "1.5× Families/Elderly",
        ),
        (
            MarketingType::PremiumAgency,
            "Agency",
            "Focused Professional leads",
        ),
    ];
    let button_gap = space::XS;
    let button_w = (inner.w - button_gap * 3.0) / 4.0;
    for (index, (strategy, label, _)) in choices.iter().enumerate() {
        let selected = state.building.marketing_strategy == *strategy;
        if button_at(
            Rect::new(
                inner.x + index as f32 * (button_w + button_gap),
                y,
                button_w,
                40.0,
            ),
            if dense {
                match strategy {
                    MarketingType::None => "Off",
                    MarketingType::SocialMedia => "Social",
                    MarketingType::LocalNewspaper => "Local",
                    MarketingType::PremiumAgency => "Pro",
                }
            } else {
                label
            },
            !selected,
            if selected {
                Tone::Primary
            } else {
                Tone::Secondary
            },
        ) {
            return Some(UiAction::SetMarketing {
                strategy: strategy.clone(),
            });
        }
    }
    y += 46.0;
    let selected = choices
        .iter()
        .find(|(strategy, _, _)| *strategy == state.building.marketing_strategy)
        .map(|(_, _, description)| *description)
        .unwrap_or("");
    let monthly = state
        .building
        .marketing_strategy
        .monthly_cost(&state.config.marketing);
    draw_ui_text(
        &format!("{} · {}/month", selected, format_money(monthly as i64)),
        inner.x,
        y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    y += line_height(scale::LABEL) + if dense { space::XS } else { space::SM };
    let open_house_label = if state.building.open_house_remaining > 0 {
        format!(
            "Open house active · {} month(s)",
            state.building.open_house_remaining
        )
    } else {
        format!(
            "Run open house · {}",
            format_money(state.config.marketing.open_house_cost as i64)
        )
    };
    if button_at(
        Rect::new(inner.x, y, inner.w, 40.0),
        &open_house_label,
        state.building.open_house_remaining == 0,
        Tone::Secondary,
    ) {
        return Some(UiAction::StartOpenHouse);
    }
    y += 45.0;
    if show_explanations {
        draw_ui_text(
            "Doubles applicant volume for the displayed duration.",
            inner.x,
            y + scale::LABEL,
            scale::LABEL,
            color::TEXT_DIM(),
        );
        y += line_height(scale::LABEL) + space::MD;
    } else if !dense {
        y += space::SM;
    }

    draw_ui_text(
        "BUILDING POLICIES",
        inner.x,
        y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    y += line_height(scale::LABEL) + if dense { 0.0 } else { space::XS };
    if dense {
        let gap = space::XS;
        let button_w = (inner.w - gap) * 0.5;
        if button_at(
            Rect::new(inner.x, y, button_w, 40.0),
            &format!("Utilities: {}", on_off(state.building.utilities_included)),
            true,
            if state.building.utilities_included {
                Tone::Positive
            } else {
                Tone::Secondary
            },
        ) {
            return Some(UiAction::SetUtilitiesIncluded {
                included: !state.building.utilities_included,
            });
        }
        if button_at(
            Rect::new(inner.x + button_w + gap, y, button_w, 40.0),
            &format!("Insurance: {}", on_off(state.building.insurance_active)),
            true,
            if state.building.insurance_active {
                Tone::Positive
            } else {
                Tone::Secondary
            },
        ) {
            return Some(UiAction::SetInsuranceActive {
                active: !state.building.insurance_active,
            });
        }
        return None;
    }
    let utility_cost = OperatingCosts::calculate_utilities(
        &policy_preview(&state.building, true, state.building.insurance_active),
        &state.config.operating_costs,
    );
    let utility_label = format!(
        "Utilities included: {} · {}/month",
        on_off(state.building.utilities_included),
        format_money(utility_cost as i64)
    );
    if button_at(
        Rect::new(inner.x, y, inner.w, 40.0),
        &utility_label,
        true,
        if state.building.utilities_included {
            Tone::Positive
        } else {
            Tone::Secondary
        },
    ) {
        return Some(UiAction::SetUtilitiesIncluded {
            included: !state.building.utilities_included,
        });
    }
    y += 45.0;
    if show_explanations {
        draw_ui_text(
            &format!(
                "Included utilities add {} happiness for every resident.",
                state.config.staff_effects.utilities_happiness_bonus
            ),
            inner.x,
            y + scale::LABEL,
            scale::LABEL,
            color::TEXT_DIM(),
        );
        y += line_height(scale::LABEL) + space::SM;
    } else if !dense {
        y += space::SM;
    }
    let insurance_cost = OperatingCosts::calculate_insurance(
        &policy_preview(&state.building, state.building.utilities_included, true),
        &state.config.operating_costs,
    );
    let insurance_label = format!(
        "Insurance: {} · {}/month",
        on_off(state.building.insurance_active),
        format_money(insurance_cost as i64)
    );
    if button_at(
        Rect::new(inner.x, y, inner.w, 40.0),
        &insurance_label,
        true,
        if state.building.insurance_active {
            Tone::Positive
        } else {
            Tone::Secondary
        },
    ) {
        return Some(UiAction::SetInsuranceActive {
            active: !state.building.insurance_active,
        });
    }
    y += 45.0;
    if show_explanations {
        draw_ui_text(
            &format!(
                "Insurance pays {}% of emergency repair bills.",
                state
                    .config
                    .critical_failures
                    .insurance_cost_reduction_percent
            ),
            inner.x,
            y + scale::LABEL,
            scale::LABEL,
            color::TEXT_DIM(),
        );
    }
    None
}

fn policy_preview(
    building: &crate::building::Building,
    utilities: bool,
    insurance: bool,
) -> crate::building::Building {
    let mut preview = building.clone();
    preview.utilities_included = utilities;
    preview.insurance_active = insurance;
    preview
}

fn on_off(value: bool) -> &'static str {
    if value {
        "On"
    } else {
        "Off"
    }
}

fn transaction_type_name(kind: &TransactionType) -> &'static str {
    match kind {
        TransactionType::RentIncome => "Rent",
        TransactionType::RepairCost | TransactionType::HallwayRepair => "Repair",
        TransactionType::UpgradeCost => "Upgrade",
        TransactionType::BuildingPurchase => "Purchase",
        TransactionType::AssetSale => "Sale",
        TransactionType::PropertyTax => "Tax",
        TransactionType::Mortgage => "Overhead",
        TransactionType::Utilities => "Utilities",
        TransactionType::Insurance => "Insurance",
        TransactionType::StaffSalary => "Staff",
        TransactionType::CriticalFailure => "Emergency",
        TransactionType::Marketing => "Marketing",
        TransactionType::Vetting => "Vetting",
        TransactionType::InspectionFine => "Fine",
        TransactionType::Grant => "Grant",
    }
}

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
    let controls_h = 48.0;
    let page_size = (((roster_inner.h - controls_h) / row_h).floor() as usize).max(1);
    let page_count = tenants.len().div_ceil(page_size).max(1);
    let page = state.tenants_page.min(page_count - 1);
    for tenant in tenants.iter().skip(page * page_size).take(page_size) {
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
    if page_count > 1 {
        let button_y = roster_inner.bottom() - 40.0;
        let gap = space::XS;
        let button_w = (roster_inner.w - gap) * 0.5;
        if page > 0
            && button_at(
                Rect::new(roster_inner.x, button_y, button_w, 40.0),
                "Previous resident",
                true,
                Tone::Secondary,
            )
        {
            return Some(UiAction::SetTenantsPage { page: page - 1 });
        }
        if page + 1 < page_count
            && button_at(
                Rect::new(roster_inner.x + button_w + gap, button_y, button_w, 40.0),
                "Next resident",
                true,
                Tone::Primary,
            )
        {
            return Some(UiAction::SetTenantsPage { page: page + 1 });
        }
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

pub fn draw_finances_view(state: &mut GameplayState) -> Option<UiAction> {
    let rect = content_rect();
    draw_ui_text(
        "Finances & policies",
        rect.x,
        rect.y + scale::TITLE,
        scale::TITLE,
        color::TEXT_BRIGHT(),
    );
    let body_y = rect.y + scale::TITLE + space::LG;
    let body = Rect::new(rect.x, body_y, rect.w, (rect.bottom() - body_y).max(1.0));
    let gap = space::LG;
    let summary_h = if rect.w < 760.0 { 96.0 } else { 112.0 };
    let summary = Rect::new(body.x, body.y, body.w, summary_h);
    let narrow = body.w < 760.0;

    if !narrow {
        draw_monthly_summary(state, summary);
        let lower_y = summary.bottom() + gap;
        let lower_h = (body.bottom() - lower_y).max(1.0);
        let ledger_w = (body.w * 0.48).max(330.0);
        let ledger = Rect::new(body.x, lower_y, ledger_w, lower_h);
        let operations = Rect::new(
            ledger.right() + gap,
            lower_y,
            (body.right() - ledger.right() - gap).max(1.0),
            lower_h,
        );
        draw_ledger(state, ledger);
        return draw_policies(state, operations);
    }

    let operations_h = if body.w < 520.0 { 500.0 } else { 430.0 };
    let ledger_h = 480.0;
    let operations_y = summary.bottom() + gap;
    let ledger_y = operations_y + operations_h + gap;
    let content_h = ledger_y - body.y + ledger_h;
    let scroll_h = 46.0;
    let viewport_h = (body.h - if content_h > body.h { scroll_h } else { 0.0 }).max(1.0);
    let viewport = Rect::new(body.x, body.y, body.w, viewport_h);
    let max_scroll = (content_h - viewport.h).max(0.0);
    state.finance_scroll_offset = state.finance_scroll_offset.clamp(0.0, max_scroll);
    if viewport.contains(vec2(mouse_position().0, mouse_position().1)) {
        state.finance_scroll_offset =
            (state.finance_scroll_offset - mouse_wheel().1 * 80.0).clamp(0.0, max_scroll);
    }

    let offset = state.finance_scroll_offset;
    let ui = macroquad_toolkit::ui::VirtualUi::new(screen_width(), screen_height());
    let mut action = None;
    ui.with_clip(viewport, || {
        draw_monthly_summary(state, Rect::new(body.x, body.y - offset, body.w, summary_h));
        if let Some(next) = draw_policies(
            state,
            Rect::new(body.x, operations_y - offset, body.w, operations_h),
        ) {
            action = Some(next);
        }
        draw_ledger(
            state,
            Rect::new(body.x, ledger_y - offset, body.w, ledger_h),
        );
    });
    if max_scroll > 0.0 {
        state.finance_scroll_offset = super::widgets::scroll_controls(
            Rect::new(body.x, body.bottom() - scroll_h + space::XS, body.w, 38.0),
            state.finance_scroll_offset,
            max_scroll,
            "Earlier",
            "Later",
        );
    }
    action
}

fn draw_monthly_summary(state: &GameplayState, rect: Rect) {
    draw_card(rect, false);
    draw_ui_text(
        "LAST CLOSED MONTH",
        rect.x + space::MD,
        rect.y + space::LG,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    let report = state.ledger.reports.last();
    let income = report.map_or(0, |report| report.rent_income);
    let outgoings = report.map_or(0, |report| report.repair_costs + report.upgrade_costs);
    let net = report.map_or(0, |report| report.net);
    let values = [
        ("Income", income, color::POSITIVE()),
        ("Outgoings", outgoings, color::NEGATIVE()),
        (
            "Net",
            net,
            if net >= 0 {
                color::POSITIVE()
            } else {
                color::NEGATIVE()
            },
        ),
    ];
    let compact = rect.h < 100.0;
    let label_y = if compact { 40.0 } else { 48.0 };
    let value_y = if compact { 67.0 } else { 75.0 };
    let gap = space::SM;
    let cell_w = (rect.w - space::MD * 2.0 - gap * 2.0) / 3.0;
    for (index, (label, value, value_color)) in values.into_iter().enumerate() {
        let x = rect.x + space::MD + index as f32 * (cell_w + gap);
        draw_ui_text(label, x, rect.y + label_y, scale::LABEL, color::TEXT_DIM());
        draw_ui_text(
            &format_money(value as i64),
            x,
            rect.y + value_y,
            scale::HEADING,
            value_color,
        );
    }
    let recurring = state
        .building
        .marketing_strategy
        .monthly_cost(&state.config.marketing)
        + OperatingCosts::calculate_utilities(&state.building, &state.config.operating_costs)
        + OperatingCosts::calculate_insurance(&state.building, &state.config.operating_costs);
    draw_ui_text(
        &format!(
            "Policy commitments: {}/month",
            format_money(recurring as i64)
        ),
        rect.right() - 190.0,
        rect.y + 20.0,
        scale::CAPTION,
        color::TEXT_DIM(),
    );
}

fn draw_ledger(state: &GameplayState, rect: Rect) {
    let inner = draw_panel(rect, "Ledger details");
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
    if inner.h >= 360.0 && state.ledger.reports.len() >= 2 {
        let chart_h = 70.0;
        draw_finance_trend(
            &state.ledger.reports,
            Rect::new(inner.x, y, inner.w, chart_h),
        );
        y += chart_h + space::SM;
    }
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
    for transaction in state.funds.transactions.iter().rev().take(8) {
        if y + line_height(scale::LABEL) > inner.bottom() {
            break;
        }
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

fn draw_finance_trend(reports: &[crate::economy::MonthlyReport], rect: Rect) {
    let recent: Vec<_> = reports.iter().rev().take(6).collect();
    let max_value = recent
        .iter()
        .map(|report| {
            report
                .rent_income
                .max(report.repair_costs + report.upgrade_costs)
                .max(report.net.abs())
        })
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color::SURFACE_ALT());
    draw_line(
        rect.x,
        rect.bottom() - 16.0,
        rect.right(),
        rect.bottom() - 16.0,
        1.0,
        color::BORDER(),
    );
    let group_w = rect.w / recent.len() as f32;
    for (index, report) in recent.iter().rev().enumerate() {
        let x = rect.x + index as f32 * group_w + group_w * 0.18;
        let bar_w = (group_w * 0.24).max(3.0);
        let chart_h = rect.h - 25.0;
        let income_h = chart_h * report.rent_income as f32 / max_value;
        let costs = report.repair_costs + report.upgrade_costs;
        let cost_h = chart_h * costs as f32 / max_value;
        draw_rectangle(
            x,
            rect.bottom() - 16.0 - income_h,
            bar_w,
            income_h,
            color::POSITIVE(),
        );
        draw_rectangle(
            x + bar_w + 2.0,
            rect.bottom() - 16.0 - cost_h,
            bar_w,
            cost_h,
            color::NEGATIVE(),
        );
        draw_ui_text(
            &report.tick.to_string(),
            x,
            rect.bottom() - 3.0,
            scale::CAPTION,
            color::TEXT_DIM(),
        );
    }
    draw_ui_text(
        "IN",
        rect.right() - 62.0,
        rect.y + 12.0,
        scale::CAPTION,
        color::POSITIVE(),
    );
    draw_ui_text(
        "OUT",
        rect.right() - 34.0,
        rect.y + 12.0,
        scale::CAPTION,
        color::NEGATIVE(),
    );
}

fn draw_policies(state: &GameplayState, rect: Rect) -> Option<UiAction> {
    let inner = draw_panel(rect, "Operations");
    let mut y = inner.y;
    let mut action = None;
    draw_ui_text(
        "MARKETING · MONTHLY COMMITMENT",
        inner.x,
        y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    y += line_height(scale::LABEL) + space::SM;
    let choices = [
        (MarketingType::None, "None", "No extra leads"),
        (
            MarketingType::SocialMedia,
            "Social",
            "2× student + artist leads",
        ),
        (
            MarketingType::LocalNewspaper,
            "Local",
            "1.5× family + elder leads",
        ),
        (
            MarketingType::PremiumAgency,
            "Premium",
            "Professional leads",
        ),
    ];
    let columns = if inner.w < 520.0 { 2 } else { 4 };
    let button_gap = space::SM;
    let button_w = (inner.w - button_gap * (columns - 1) as f32) / columns as f32;
    let row_h = 70.0;
    for (index, (strategy, label, _)) in choices.iter().enumerate() {
        let row = index / columns;
        let column = index % columns;
        let selected = state.building.marketing_strategy == *strategy;
        let price = strategy.monthly_cost(&state.config.marketing);
        let button = Rect::new(
            inner.x + column as f32 * (button_w + button_gap),
            y + row as f32 * row_h,
            button_w,
            40.0,
        );
        if button_at(
            button,
            &format!("{} {}/mo", label, format_money(price as i64)),
            true,
            if selected {
                Tone::Primary
            } else {
                Tone::Secondary
            },
        ) {
            action = Some(UiAction::SetMarketing {
                strategy: strategy.clone(),
            });
        }
        draw_ui_text(
            match strategy {
                MarketingType::None => "No extra leads",
                MarketingType::SocialMedia => "2× student + artist leads",
                MarketingType::LocalNewspaper => "1.5× family + elder leads",
                MarketingType::PremiumAgency => "Professional leads",
            },
            button.x,
            button.bottom() + scale::CAPTION,
            scale::CAPTION,
            color::TEXT_DIM(),
        );
    }
    y += choices.len().div_ceil(columns) as f32 * row_h + space::SM;
    let open_house_label = if state.building.open_house_remaining > 0 {
        format!(
            "Open house active · {} month(s) left",
            state.building.open_house_remaining
        )
    } else {
        "Run open house".to_string()
    };
    let open_house_cost = state.config.marketing.open_house_cost;
    let open_house_enabled =
        state.building.open_house_remaining == 0 && state.funds.balance >= open_house_cost;
    if button_at(
        Rect::new(inner.x, y, inner.w, 40.0),
        &open_house_label,
        open_house_enabled,
        Tone::Secondary,
    ) {
        action = Some(UiAction::StartOpenHouse);
    }
    draw_ui_text(
        &format!(
            "{} · {} once",
            if state.building.open_house_remaining > 0 {
                "Cost already paid"
            } else {
                "Cost"
            },
            format_money(open_house_cost as i64)
        ),
        inner.x,
        y + 55.0,
        scale::CAPTION,
        color::TEXT_DIM(),
    );
    y += 45.0;
    if !open_house_enabled && state.building.open_house_remaining == 0 {
        draw_ui_text(
            &format!(
                "Need {} cash to run it",
                format_money(open_house_cost as i64)
            ),
            inner.x,
            y + scale::CAPTION,
            scale::CAPTION,
            color::WARNING(),
        );
    }
    y += line_height(scale::CAPTION) + space::SM;

    draw_ui_text(
        "BUILDING POLICIES",
        inner.x,
        y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    y += line_height(scale::LABEL) + space::SM;
    let utility_cost = OperatingCosts::calculate_utilities(
        &policy_preview(&state.building, true, state.building.insurance_active),
        &state.config.operating_costs,
    );
    let utility_label = format!("Utilities: {}", on_off(state.building.utilities_included));
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
        action = Some(UiAction::SetUtilitiesIncluded {
            included: !state.building.utilities_included,
        });
    }
    draw_ui_text(
        &format!(
            "{} /month when included · +{} happiness",
            format_money(utility_cost as i64),
            state.config.staff_effects.utilities_happiness_bonus
        ),
        inner.x,
        y + 55.0,
        scale::CAPTION,
        color::TEXT_DIM(),
    );
    y += 45.0;
    y += line_height(scale::CAPTION) + space::SM;
    let insurance_cost = OperatingCosts::calculate_insurance(
        &policy_preview(&state.building, state.building.utilities_included, true),
        &state.config.operating_costs,
    );
    let insurance_label = format!("Insurance: {}", on_off(state.building.insurance_active));
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
        action = Some(UiAction::SetInsuranceActive {
            active: !state.building.insurance_active,
        });
    }
    draw_ui_text(
        &format!(
            "{} /month · pays {}% of emergency repairs",
            format_money(insurance_cost as i64),
            state
                .config
                .critical_failures
                .insurance_cost_reduction_percent
        ),
        inner.x,
        y + 55.0,
        scale::CAPTION,
        color::TEXT_DIM(),
    );
    action
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

use super::*;
use crate::economy::TransactionType;

#[test]
fn visible_pause_action_opens_the_pause_menu() {
    let mut state = GameplayState::new();

    state.process_action(UiAction::OpenPauseMenu);

    assert!(state.show_pause_menu);
}

#[test]
fn stale_application_is_not_discarded_when_the_unit_became_unavailable() {
    let mut state = GameplayState::new();
    let apartment_id = state.building.apartments[0].id;
    state.building.apartments[0].move_in(777);
    let tenant =
        crate::tenant::Tenant::new(888, "Applicant", crate::tenant::TenantArchetype::Student);
    state
        .applications
        .push(crate::tenant::TenantApplication::new_for_building(
            tenant,
            state.active_building_id(),
            apartment_id,
            crate::tenant::matching::MatchResult {
                score: 50,
                meets_minimum: true,
                reasons: Vec::new(),
            },
            0,
        ));

    state.process_action(UiAction::AcceptApplication {
        application_index: 0,
    });

    assert_eq!(state.applications.len(), 1);
}

#[test]
fn dialogue_money_reward_records_income_transaction() {
    let mut state = GameplayState::new();
    state.current_tick = 7;
    let starting_balance = state.funds.balance;

    state.apply_dialogue_money_change(250);

    assert_eq!(state.funds.balance, starting_balance + 250);
    assert_eq!(state.funds.total_income, 250);
    assert!(state.funds.transactions.iter().any(|transaction| {
        transaction.transaction_type == TransactionType::Grant
            && transaction.amount == 250
            && transaction.description == "Dialogue Reward"
            && transaction.tick == 7
    }));
}

#[test]
fn dialogue_money_cost_records_expense_transaction() {
    let mut state = GameplayState::new();
    state.current_tick = 8;
    let starting_balance = state.funds.balance;

    state.apply_dialogue_money_change(-125);

    assert_eq!(state.funds.balance, starting_balance - 125);
    assert_eq!(state.funds.total_expenses, 125);
    assert!(state.funds.transactions.iter().any(|transaction| {
        transaction.transaction_type == TransactionType::CriticalFailure
            && transaction.amount == -125
            && transaction.description == "Dialogue Cost"
            && transaction.tick == 8
    }));
}

#[test]
fn end_month_taps_are_ignored_while_the_closed_month_is_open() {
    // A full turn touches screen-space feedback, so this exercises the
    // guard rather than the simulation: the summary owns End Month.
    let mut state = GameplayState::new();
    state.month_summary_open = true;
    let start = state.current_tick;

    state.process_action(UiAction::EndTurn);
    state.process_action(UiAction::EndTurn);
    assert_eq!(state.current_tick, start);

    state.process_action(UiAction::DismissMonthSummary);
    assert!(!state.month_summary_open);
}

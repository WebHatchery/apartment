use super::*;

#[test]
fn monthly_mail_uses_current_tick_rent_income() {
    let mut state = GameplayState::new();
    state.current_tick = 2;
    state.mailbox.items.clear();
    state.last_tick_result = Some(TickResult {
        events: Vec::new(),
        rent_collected: 10,
        tenants_moved_out: Vec::new(),
        new_applications: 0,
        outcome: None,
    });

    let result = TickResult {
        events: Vec::new(),
        rent_collected: 1234,
        tenants_moved_out: Vec::new(),
        new_applications: 0,
        outcome: None,
    };
    state.generate_monthly_narrative(&result);

    let statement = state
        .mailbox
        .items
        .iter()
        .find(|item| item.subject == "Monthly Statement - Month 2");
    assert!(statement.is_some(), "expected month 2 financial statement");
    if let Some(statement) = statement {
        assert!(
            statement.body.contains("Total Income: $1234"),
            "statement should use current tick income"
        );
    }
}

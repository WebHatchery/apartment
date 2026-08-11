use super::*;
use crate::narrative::ActiveTaxBreak;

#[test]
fn tax_break_refunds_current_tick_property_tax_and_expires() {
    let mut state = GameplayState::new();
    state.current_tick = 4;
    state.active_tax_breaks = vec![ActiveTaxBreak::new(1, 0.25)];
    state.funds.deduct_expense(Transaction::expense(
        TransactionType::PropertyTax,
        400,
        "Monthly Property Tax",
        state.current_tick,
    ));

    let refund = state.process_active_tax_breaks();

    assert_eq!(refund, 100);
    assert!(state.active_tax_breaks.is_empty());
    assert!(state.funds.transactions.iter().any(|transaction| {
        transaction.transaction_type == TransactionType::Grant
            && transaction.amount == 100
            && transaction.description == "Mission Tax Break Refund"
            && transaction.tick == 4
    }));
}

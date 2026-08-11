use super::*;

#[test]
fn operating_credit_allows_recovery_but_has_a_hard_limit() {
    let mut funds = PlayerFunds::new(0);
    funds.balance = -PlayerFunds::BANKRUPTCY_DEBT_LIMIT;
    assert!(!funds.is_bankrupt());
    funds.balance -= 1;
    assert!(funds.is_bankrupt());
}

use super::*;
use crate::building::Building;
use crate::data::config::GameConfig;
use crate::simulation::TickContext;

fn check(tenants: &[Tenant], tick: u32, ever_occupied: bool) -> Option<GameOutcome> {
    let building = Building::new("Test", 3, 2);
    let funds = PlayerFunds::default(); // 5000, solvent
    let cfg = GameConfig::default(); // all_left_check_tick = 3, duration = 36
    check_win_condition(
        &building,
        tenants,
        &funds,
        tick,
        &TickContext {
            building_id: 0,
            has_ever_had_tenant: ever_occupied,
            reputation_multiplier: 1.0,
            config: &cfg,
        },
    )
}

#[test]
fn never_occupied_building_does_not_trigger_all_tenants_left() {
    // Empty and past the check tick, but no tenant ever moved in: not a loss.
    assert!(check(&[], 5, false).is_none());
}

#[test]
fn previously_occupied_then_empty_triggers_all_tenants_left() {
    assert!(matches!(
        check(&[], 5, true),
        Some(GameOutcome::AllTenantsLeft)
    ));
}

#[test]
fn empty_before_check_tick_is_not_yet_a_loss() {
    // tick 2 <= all_left_check_tick (3): a temporary early vacancy is tolerated.
    assert!(check(&[], 2, true).is_none());
}

#[test]
fn grants_and_asset_sales_do_not_inflate_victory_score() {
    use crate::economy::{Transaction, TransactionType};

    let building = Building::new("Test", 1, 1);
    let cfg = GameConfig::default();
    let mut base = PlayerFunds::new(5_000);
    base.add_income(Transaction::income(
        TransactionType::RentIncome,
        1_000,
        "Rent",
        36,
    ));
    let mut windfall = base.clone();
    windfall.add_income(Transaction::income(
        TransactionType::Grant,
        100_000,
        "Grant",
        36,
    ));
    windfall.add_income(Transaction::income(
        TransactionType::AssetSale,
        100_000,
        "Condo sale",
        36,
    ));

    let score = |funds: &PlayerFunds| match check_win_condition(
        &building,
        &[],
        funds,
        36,
        &TickContext {
            building_id: 0,
            has_ever_had_tenant: false,
            reputation_multiplier: 1.0,
            config: &cfg,
        },
    ) {
        Some(GameOutcome::Victory { score, .. }) => score,
        outcome => panic!("expected victory, got {outcome:?}"),
    };

    assert_eq!(score(&base), score(&windfall));
}

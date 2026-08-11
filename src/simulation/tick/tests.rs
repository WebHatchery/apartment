use super::*;
use crate::building::Building;
use crate::data::config::GameConfig;
use crate::tenant::{Tenant, TenantArchetype};

fn empty_result() -> TickResult {
    TickResult {
        events: Vec::new(),
        rent_collected: 0,
        tenants_moved_out: Vec::new(),
        new_applications: 0,
        outcome: None,
    }
}

#[test]
fn janitor_offsets_decay_on_maintained_units() {
    let mut config = GameConfig::default();
    config.decay.apartment_per_tick = 3;
    config.staff_effects.janitor_units_maintained = 5;

    let mut building = Building::new("Test", 3, 2); // 6 units at condition 50
    building.flags.insert("staff_janitor".to_string());

    building.apply_monthly_decay(3, 1); // every unit -> 47
    let mut result = empty_result();
    GameTick::process_janitor_maintenance(&mut building, &mut result, &config);

    // 5 of 6 units are restored to their pre-decay condition; one is not.
    let restored = building
        .apartments
        .iter()
        .filter(|a| a.condition == 50)
        .count();
    let unmaintained = building
        .apartments
        .iter()
        .filter(|a| a.condition == 47)
        .count();
    assert_eq!(restored, 5);
    assert_eq!(unmaintained, 1);
}

#[test]
fn insurance_reduces_emergency_repair_costs() {
    assert_eq!(emergency_repair_cost(2_000, 400, false, 50), 2_400);
    assert_eq!(emergency_repair_cost(2_000, 400, true, 50), 1_200);
}

#[test]
fn inactive_building_tenant_is_not_advanced() {
    let building = Building::new("Active", 1, 1);
    let mut tenant = Tenant::new(1, "Elsewhere", TenantArchetype::Student);
    tenant.move_into_building(1, 0);
    tenant.happiness = 41;
    let mut tenants = vec![tenant];
    let mut result = empty_result();
    let config = GameConfig::default();

    GameTick::update_tenants(
        0,
        &building,
        &mut tenants,
        &mut result,
        &config.happiness,
        &config.staff_effects,
    );

    assert_eq!(tenants[0].happiness, 41);
    assert_eq!(tenants[0].months_residing, 0);
}

#[test]
fn low_behavior_tenant_damages_property() {
    let mut config = GameConfig::default();
    config.tenant_risk.low_behavior_threshold = 100;
    config.tenant_risk.damage_chance_percent = 100;
    config.tenant_risk.damage_amount = 6;

    let mut building = Building::new("Test", 1, 1);
    let apt_id = building.apartments[0].id;
    let before = building.apartments[0].condition;

    let mut tenant = Tenant::new(1, "Risky", TenantArchetype::Student);
    tenant.behavior_score = 10;
    tenant.apartment_id = Some(apt_id);
    let tenants = vec![tenant];

    let mut result = empty_result();
    GameTick::process_tenant_risk(0, &mut building, &tenants, &config, &mut result);

    assert_eq!(building.apartments[0].condition, before - 6);
    assert!(result
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::TenantDamage { .. })));
}

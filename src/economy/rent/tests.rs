use super::*;
use crate::tenant::{Tenant, TenantArchetype};

#[test]
fn unreliable_tenant_skips_rent() {
    let building = Building::new("Test", 1, 1);
    let apt_id = building.apartments[0].id;
    let mut funds = PlayerFunds::new(1000);

    let mut tenant = Tenant::new(1, "Flaky", TenantArchetype::Student);
    tenant.happiness = 80; // avoid the unhappiness skip branch
    tenant.rent_reliability = 10;
    tenant.apartment_id = Some(apt_id);
    let tenants = vec![tenant];

    let risk = TenantRiskConfig {
        unreliable_threshold: 100,
        skip_rent_chance_percent: 100,
        ..TenantRiskConfig::default()
    };

    let collection = collect_rent(0, &tenants, &building, &mut funds, 1, &risk);
    assert_eq!(collection.total_collected, 0);
    assert_eq!(collection.missed_payments.len(), 1);
}

#[test]
fn reliable_tenant_pays_rent() {
    let building = Building::new("Test", 1, 1);
    let apt_id = building.apartments[0].id;
    let mut funds = PlayerFunds::new(1000);

    let mut tenant = Tenant::new(1, "Solid", TenantArchetype::Professional);
    tenant.happiness = 80;
    tenant.rent_reliability = 95;
    tenant.apartment_id = Some(apt_id);
    let tenants = vec![tenant];

    let collection = collect_rent(
        0,
        &tenants,
        &building,
        &mut funds,
        1,
        &TenantRiskConfig::default(),
    );
    assert_eq!(collection.missed_payments.len(), 0);
    assert!(collection.total_collected > 0);
}

#[test]
fn tenant_in_another_building_does_not_pay_active_building_rent() {
    let building = Building::new("Active", 1, 1);
    let mut funds = PlayerFunds::new(1000);
    let mut tenant = Tenant::new(1, "Elsewhere", TenantArchetype::Professional);
    tenant.move_into_building(1, building.apartments[0].id);

    let collection = collect_rent(
        0,
        &[tenant],
        &building,
        &mut funds,
        1,
        &TenantRiskConfig::default(),
    );

    assert_eq!(collection.total_collected, 0);
    assert_eq!(funds.balance, 1000);
}

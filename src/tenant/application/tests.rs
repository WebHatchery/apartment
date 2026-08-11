use super::*;
use crate::data::config::TenantRiskConfig;

#[test]
fn risky_applicant_gets_a_rent_premium() {
    let cfg = TenantRiskConfig::default();
    let mut tenant = Tenant::new(1, "Risky", TenantArchetype::Student);
    tenant.rent_reliability = 20;
    tenant.behavior_score = 20;
    let base = tenant.rent_tolerance;
    apply_risk_rent_premium(&mut tenant, &cfg);
    assert!(
        tenant.rent_tolerance > base,
        "a risky applicant should tolerate higher rent (tempting to accept)"
    );
}

#[test]
fn safe_applicant_gets_no_premium() {
    let cfg = TenantRiskConfig::default();
    let mut tenant = Tenant::new(1, "Safe", TenantArchetype::Professional);
    tenant.rent_reliability = 90;
    tenant.behavior_score = 90;
    let base = tenant.rent_tolerance;
    apply_risk_rent_premium(&mut tenant, &cfg);
    assert_eq!(tenant.rent_tolerance, base);
}

#[test]
fn receptionist_increases_application_rate() {
    let config = crate::data::config::StaffEffectsConfig::default();
    let mut building = Building::new("Test", 1, 1);
    assert_eq!(application_staff_multiplier(&building, &config), 1.0);

    building.flags.insert("staff_receptionist".to_string());
    assert_eq!(
        application_staff_multiplier(&building, &config),
        config.receptionist_application_multiplier
    );
}

#[test]
fn weighted_archetype_source_has_a_stable_order() {
    let ids: Vec<_> = base_weighted_archetypes()
        .into_iter()
        .map(|(archetype, _)| archetype.id())
        .collect();
    assert!(ids.windows(2).all(|pair| pair[0] <= pair[1]));
}

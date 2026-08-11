use super::*;
use crate::building::DesignType;
use crate::data::config::OperatingCostsConfig;

#[test]
fn base_overhead_scales_with_unit_count() {
    let building = Building::new("Test", 3, 2); // 6 units
    let config = OperatingCostsConfig::default();
    assert_eq!(
        OperatingCosts::calculate_base_overhead(&building, &config),
        6 * config.base_monthly_cost_per_unit
    );
}

#[test]
fn stale_upgrade_action_cannot_bypass_size_requirements_or_charge_funds() {
    let config = crate::data::config::load_config();
    let mut building = Building::new("Small Units", 1, 1);
    building.apartments[0].design = DesignType::Cozy;
    let mut funds = PlayerFunds::new(100_000);
    let before = funds.balance;

    let result = process_upgrade(
        &UpgradeAction::Apply {
            upgrade_id: "upgrade_to_luxury".to_string(),
            target_id: Some(0),
        },
        &mut building,
        &mut funds,
        &config,
        1,
    );

    assert!(result.is_err());
    assert_eq!(funds.balance, before);
    assert_eq!(building.apartments[0].design, DesignType::Cozy);
}

#[test]
fn property_tax_escalates_each_year() {
    let building = Building::new("Test", 1, 1);
    let config = OperatingCostsConfig {
        property_tax_rate: 0.10,
        property_tax_annual_increase: 0.02,
        ..OperatingCostsConfig::default()
    };

    let year0 = OperatingCosts::calculate_property_tax(&building, 1000, &config, 0);
    let year2 = OperatingCosts::calculate_property_tax(&building, 1000, &config, 24);

    assert_eq!(year0, 100); // 10% of 1000
    assert_eq!(year2, 140); // (0.10 + 0.02*2) * 1000
    assert!(year2 > year0);
}

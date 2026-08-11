use super::*;
use crate::data::config::StaffEffectsConfig;

#[test]
fn staff_factor_reflects_security_and_manager() {
    let mut building = Building::new("Test", 1, 1);
    let staff = StaffEffectsConfig::default();

    assert_eq!(calculate_staff_factor(&building, &staff), 0);

    building.flags.insert("staff_security".to_string());
    building.flags.insert("staff_manager".to_string());

    assert_eq!(
        calculate_staff_factor(&building, &staff),
        staff.security_happiness_bonus + staff.manager_happiness_bonus
    );
}

#[test]
fn included_utilities_have_a_persistent_happiness_benefit() {
    let mut building = Building::new("Utilities", 1, 1);
    let staff = StaffEffectsConfig::default();
    building.utilities_included = true;

    assert_eq!(calculate_staff_factor(&building, &staff), 5);
}

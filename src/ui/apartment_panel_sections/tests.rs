use super::*;

#[test]
fn apartment_upgrade_families_select_the_expected_cells() {
    let apply = |id: &str| UpgradeAction::Apply {
        upgrade_id: id.to_string(),
        target_id: Some(1),
    };
    assert_eq!(
        upgrade_icon_tile(&UpgradeAction::RepairApartment {
            apartment_id: 1,
            amount: 10,
        }),
        (0.0, 0.0)
    );
    assert_eq!(upgrade_icon_tile(&apply("soundproofing")), (1.0, 0.0));
    assert_eq!(upgrade_icon_tile(&apply("kitchen_renovation")), (2.0, 0.0));
    assert_eq!(upgrade_icon_tile(&apply("lighting_upgrade")), (0.0, 1.0));
    assert_eq!(upgrade_icon_tile(&apply("upgrade_to_cozy")), (2.0, 1.0));
}

use super::*;

use crate::data::config::GameConfig;

#[test]
fn test_upgrade_costs() {
    let building = Building::new("Test", 3, 2);
    let game_config = crate::data::config::load_config();
    let config = game_config.economy;
    let upgrades = game_config.upgrades;

    let repair = UpgradeAction::RepairApartment {
        apartment_id: 0,
        amount: 10,
    };
    assert_eq!(repair.cost(&building, &config, &upgrades), Some(100)); // 10 * $10

    let design = UpgradeAction::Apply {
        upgrade_id: "upgrade_to_practical".to_string(),
        target_id: Some(0),
    };
    assert_eq!(design.cost(&building, &config, &upgrades), Some(1200));
}

#[test]
fn test_apply_repair_upgrade() {
    let mut building = Building::new("Test", 3, 2);
    let _config = GameConfig::default().economy;
    let upgrades = GameConfig::default().upgrades;
    let initial_condition = building.apartments[0].condition;

    let action = UpgradeAction::RepairApartment {
        apartment_id: 0,
        amount: 20,
    };
    let cost = apply_upgrade(&mut building, &action, &upgrades);

    assert_eq!(cost, Some(())); // Returns Option<()>
    assert_eq!(building.apartments[0].condition, initial_condition + 20);
}

#[test]
fn configured_amenities_update_authoritative_fields() {
    let config = crate::data::config::load_config();
    let mut building = Building::new("Test", 1, 1);
    let base_quality = building.apartments[0].quality_score();

    assert!(apply_upgrade(
        &mut building,
        &UpgradeAction::Apply {
            upgrade_id: "lighting_upgrade".to_string(),
            target_id: Some(0),
        },
        &config.upgrades,
    )
    .is_some());
    assert!(building.apartments[0].quality_score() > base_quality);

    assert!(apply_upgrade(
        &mut building,
        &UpgradeAction::Apply {
            upgrade_id: "kitchen_renovation".to_string(),
            target_id: Some(0),
        },
        &config.upgrades,
    )
    .is_some());
    assert_eq!(building.apartments[0].kitchen_level, 2);

    let base_appeal = building.building_appeal();
    assert!(apply_upgrade(
        &mut building,
        &UpgradeAction::Apply {
            upgrade_id: "install_laundry".to_string(),
            target_id: None,
        },
        &config.upgrades,
    )
    .is_some());
    assert!(building.has_laundry);
    assert_eq!(building.building_appeal(), base_appeal + 10);
}

#[test]
fn configured_upgrade_actions_have_a_stable_order() {
    let config = crate::data::config::load_config();
    let building = Building::new("Test", 1, 1);
    let actions = available_building_upgrades(&building, &config.upgrades);
    let keys: Vec<_> = actions.iter().map(upgrade_sort_key).collect();

    assert!(keys.windows(2).all(|pair| pair[0] <= pair[1]));
}

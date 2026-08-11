use super::*;

#[test]
fn test_building_generation() {
    let building = Building::new("Test", 3, 2);
    assert_eq!(building.apartments.len(), 6);
}

#[test]
fn test_vacancy_tracking() {
    let mut building = Building::new("Test", 3, 2);
    assert_eq!(building.vacancy_count(), 6);
    assert_eq!(building.occupancy_count(), 0);

    let first_unit = building.get_apartment_mut(0);
    assert!(first_unit.is_some(), "expected apartment 0 to exist");
    if let Some(apartment) = first_unit {
        apartment.move_in(1);
    }

    let second_unit = building.get_apartment_mut(1);
    assert!(second_unit.is_some(), "expected apartment 1 to exist");
    if let Some(apartment) = second_unit {
        apartment.move_in(2);
    }

    assert_eq!(building.vacancy_count(), 4);
    assert_eq!(building.occupancy_count(), 2);
}

#[test]
fn sold_condos_leave_the_rental_roll() {
    let mut building = Building::new("Test", 1, 2);
    building.get_apartment_mut(0).unwrap().move_in(7);
    assert!(building.convert_unit_to_condo(0, "Owner", 20_000));

    assert_eq!(building.rental_unit_count(), 1);
    assert_eq!(building.occupancy_count(), 0);
    assert_eq!(building.vacancy_count(), 1);
    assert!(!building.has_full_rental_occupancy());
}

#[test]
fn buyback_quote_does_not_change_ownership() {
    let mut building = Building::new("Test", 1, 1);
    assert!(building.convert_unit_to_condo(0, "Owner", 20_000));
    assert!(matches!(
        building.ownership_model,
        OwnershipType::FullCondo(_)
    ));

    assert_eq!(building.condo_buyback_price(0), Some(22_000));
    assert!(building.is_unit_sold(0));
    assert!(building.complete_condo_buyback(0));
    assert!(matches!(
        building.ownership_model,
        OwnershipType::FullRental
    ));
    assert!(!building.is_unit_sold(0));
    assert_eq!(building.vacancy_count(), 1);
}

#[test]
fn test_building_appeal() {
    let building = Building::new("Test", 3, 2);
    // hallway_condition = 60, each apartment condition = 50
    // hallway_factor = 60 / 2 = 30
    // avg_factor = 50 / 2 = 25
    // total = 55
    assert_eq!(building.building_appeal(), 55);
}

#[test]
fn test_monthly_decay() {
    let mut building = Building::new("Test", 3, 2);
    let initial_hallway = building.hallway_condition;
    let initial_apt_condition = building.apartments[0].condition;

    building.apply_monthly_decay(3, 1);

    assert_eq!(building.hallway_condition, initial_hallway - 1);
    assert_eq!(building.apartments[0].condition, initial_apt_condition - 3);
}

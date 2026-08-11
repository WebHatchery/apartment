use super::*;

#[test]
fn rent_change_records_the_active_building() {
    let mut state = GameplayState::new();
    let apartment_id = state.building.apartments[0].id;
    state.city.buildings.push(state.building.clone());
    state.city.active_building_index = 1;
    state.sync_building();

    state.set_apartment_rent(apartment_id, 900).unwrap();

    assert!(state.gentrification.rent_history.contains_key(&1));
    assert!(!state.gentrification.rent_history.contains_key(&0));
}

#[test]
fn open_house_charges_once_and_sets_duration() {
    let mut state = GameplayState::new();
    let before = state.funds.balance;
    let cost = state.start_open_house().unwrap();

    assert_eq!(before - state.funds.balance, cost);
    assert_eq!(
        state.building.open_house_remaining,
        state.config.marketing.open_house_duration
    );
    assert!(state.start_open_house().is_err());
    assert_eq!(before - state.funds.balance, cost);
}

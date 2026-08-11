use super::*;

#[test]
fn test_city_creation() {
    let city = City::new("Test City");
    assert_eq!(city.neighborhoods.len(), 4);
    assert_eq!(city.buildings.len(), 0);
}

#[test]
fn test_starter_building() {
    let (city, _) = City::with_starter_building("Test City", 0);
    assert_eq!(city.buildings.len(), 1);
    assert!(city.neighborhoods[0].building_ids.contains(&0));
}

use super::*;

#[test]
fn test_apartment_quality_score() {
    let mut apt = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::Low);
    apt.condition = 50;
    apt.design = DesignType::Bare;
    assert_eq!(apt.quality_score(), 50); // 50 condition, no bonuses

    apt.design = DesignType::Cozy;
    assert_eq!(apt.quality_score(), 90); // 50 + 40 design
}

#[test]
fn test_soundproofing_effect() {
    let mut apt = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::High);
    assert_eq!(apt.effective_noise(), NoiseLevel::High);

    apt.has_soundproofing = true;
    assert_eq!(apt.effective_noise(), NoiseLevel::Low);
}

#[test]
fn test_condition_decay_and_repair() {
    let mut apt = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::Low);
    apt.condition = 50;

    apt.decay_condition(10);
    assert_eq!(apt.condition, 40);

    apt.repair(25);
    assert_eq!(apt.condition, 65);

    apt.repair(100); // Should clamp to 100
    assert_eq!(apt.condition, 100);

    apt.condition = 5;
    apt.decay_condition(10); // Should clamp to 0
    assert_eq!(apt.condition, 0);
}

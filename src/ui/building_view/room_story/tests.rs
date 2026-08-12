use super::*;
use crate::building::{ApartmentSize, NoiseLevel};

#[test]
fn every_player_controlled_room_upgrade_has_a_visual_layer() {
    let mut apartment = Apartment::new(1, "1A", 1, ApartmentSize::Small, NoiseLevel::Low);
    apartment.condition = 20;
    apartment.has_soundproofing = true;
    apartment.kitchen_level = 2;
    apartment.flags.insert("has_better_lighting".to_string());

    assert_eq!(
        room_features(&apartment),
        RoomFeatures {
            damaged: true,
            severely_damaged: true,
            cared_for: false,
            soundproofed: true,
            upgraded_lighting: true,
            renovated_kitchen: true,
        }
    );
}

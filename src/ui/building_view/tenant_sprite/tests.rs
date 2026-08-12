use super::*;
use crate::building::{ApartmentSize, NoiseLevel};

fn apartment(design: DesignType, kitchen_level: i32) -> Apartment {
    let mut apartment = Apartment::new(1, "1A", 1, ApartmentSize::Small, NoiseLevel::Low);
    apartment.design = design;
    apartment.kitchen_level = kitchen_level;
    apartment
}

#[test]
fn room_features_choose_visible_activities() {
    assert_eq!(
        pose_for(&apartment(DesignType::Bare, 0)),
        ResidentPose::Standing
    );
    assert_eq!(
        pose_for(&apartment(DesignType::Cozy, 0)),
        ResidentPose::Sitting
    );
    assert_eq!(
        pose_for(&apartment(DesignType::Bare, 1)),
        ResidentPose::Cooking
    );
}

#[test]
fn happiness_maps_across_all_five_face_columns() {
    assert_eq!(emotion_column(95), 0.0);
    assert_eq!(emotion_column(75), 1.0);
    assert_eq!(emotion_column(55), 2.0);
    assert_eq!(emotion_column(30), 3.0);
    assert_eq!(emotion_column(5), 4.0);
}

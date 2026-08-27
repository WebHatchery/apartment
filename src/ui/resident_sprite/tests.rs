use super::*;
use crate::building::{ApartmentSize, DesignType, NoiseLevel};

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
fn resident_animation_cycles_through_all_available_poses() {
    let apartment = apartment(DesignType::Practical, 1);
    assert_eq!(animated_pose(&apartment, 1, 0.0), ResidentPose::Standing);
    assert_eq!(animated_pose(&apartment, 1, 2.3), ResidentPose::Sitting);
    assert_eq!(animated_pose(&apartment, 1, 4.6), ResidentPose::Cooking);
}

#[test]
fn happiness_maps_across_all_five_face_columns() {
    assert_eq!(emotion_column(95), 0.0);
    assert_eq!(emotion_column(75), 1.0);
    assert_eq!(emotion_column(55), 2.0);
    assert_eq!(emotion_column(30), 3.0);
    assert_eq!(emotion_column(5), 4.0);
}

#[test]
fn head_anchor_accounts_for_each_pose() {
    let style = RESIDENT_STYLES[0];
    assert!(style.body_anchor_y[0] > 0.0);
    assert!(style.body_anchor_y[1] > style.body_anchor_y[2]);
    assert!(style.body_anchor_y[2] > style.body_anchor_y[0]);
}

#[test]
fn animation_anchors_follow_the_body_neck() {
    let style = RESIDENT_STYLES[0];
    let body_w = 80.0;
    let sprite_h = 100.0;
    let body_y = 36.0;
    let body_x = 20.0;
    let head_w = 34.0;
    let head_h = 48.0;
    let standing = resident_anchor(
        style,
        ResidentPose::Standing,
        55,
        body_x,
        body_y,
        body_w,
        sprite_h,
        head_w,
        head_h,
    );
    let sitting = resident_anchor(
        style,
        ResidentPose::Sitting,
        55,
        body_x,
        body_y,
        body_w,
        sprite_h,
        head_w,
        head_h,
    );
    assert_ne!(standing, sitting);
    assert!(
        (standing.0 - (body_x + style.body_anchor_x[0] * body_w - style.face_anchor_x[2] * head_w))
            .abs()
            < 0.01
    );
    assert!(sitting.1 > standing.1);
}

#[test]
fn open_collar_styles_anchor_to_the_lower_neck_seam() {
    for style in [RESIDENT_STYLES[1], RESIDENT_STYLES[3]] {
        assert!(style.body_anchor_y[0] > 0.10);
        assert!(style.body_anchor_y[1] >= 0.20);
        assert!(style.body_anchor_y[2] > 0.10);
        assert!((style.body_anchor_x[1] - style.body_anchor_x[2]).abs() < 0.001);
    }
}

#[test]
fn face_anchors_use_the_neck_not_the_visible_hair_bounds() {
    let rust = RESIDENT_STYLES[0];
    assert!(rust.face_anchor_x[0] > rust.face_anchor_x[4]);
    assert!(rust.face_anchor_x[0] > 0.55);
    let teal = RESIDENT_STYLES[1];
    assert!(teal.face_anchor_x[0] > 0.60);
    assert!(teal.face_anchor_y[0] > 0.94);
}

#[test]
fn heads_keep_a_bodily_scale_at_room_size() {
    for style in RESIDENT_STYLES {
        assert!(style.head_height_ratio <= 0.40);
        assert!(style.head_height_ratio >= 0.38);
    }
}

#[test]
fn reduced_heads_are_lifted_back_over_the_neck() {
    let style = RESIDENT_STYLES[1];
    let (_, head_y) = resident_anchor(
        style,
        ResidentPose::Sitting,
        55,
        20.0,
        36.0,
        80.0,
        100.0,
        28.0,
        39.0,
    );
    let neck_y = 36.0 + style.body_anchor_y[1] * 100.0;
    assert_eq!(head_y, neck_y - style.face_anchor_y[2] * 39.0);
}

#[test]
fn tenant_ids_cycle_through_four_matched_sprite_sets() {
    assert_eq!(style_for(0), RESIDENT_STYLES[0]);
    assert_eq!(style_for(1), RESIDENT_STYLES[1]);
    assert_eq!(style_for(2), RESIDENT_STYLES[2]);
    assert_eq!(style_for(3), RESIDENT_STYLES[3]);
    assert_eq!(style_for(4), RESIDENT_STYLES[0]);
    for style in RESIDENT_STYLES {
        assert!(style.body_id.starts_with("tenant_body_poses"));
        assert!(style.face_id.starts_with("tenant_face_emotions"));
    }
}

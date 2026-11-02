use apartment::screen_layout::{header_height, touch_rect, BuildingLayout, TOUCH_MIN};
use macroquad::prelude::Rect;

#[test]
fn unselected_building_recovers_the_full_workspace() {
    for (width, height) in [(1280.0, 720.0), (1920.0, 1080.0), (390.0, 844.0)] {
        let layout = BuildingLayout::new(width, height, false);
        assert_eq!(layout.world.w, width);
        assert_eq!(layout.world.bottom(), height - 44.0);
        assert_eq!(layout.world.y, header_height(width));
    }
}

#[test]
fn desktop_details_remain_narrow_without_overlapping_the_building() {
    for width in [760.0, 960.0, 1280.0, 1920.0, 2560.0] {
        let layout = BuildingLayout::new(width, 720.0, true);
        assert!(layout.panel.w >= 340.0 && layout.panel.w <= 400.0);
        assert!(layout.world.right() < layout.panel.x);
        assert!(layout.panel.right() <= width);
        assert!(layout.world.w >= 400.0);
    }
}

#[test]
fn portrait_details_keep_a_visible_building_above_the_sheet() {
    for (width, height) in [(360.0, 640.0), (390.0, 844.0), (600.0, 800.0)] {
        let layout = BuildingLayout::new(width, height, true);
        assert!(layout.world.bottom() < layout.panel.y);
        assert!(layout.world.h > 70.0);
        assert!(layout.panel.w >= 320.0);
        assert!(layout.panel.bottom() < height - 44.0);
    }
}

#[test]
fn wrapped_navigation_never_uses_the_world_area() {
    assert_eq!(header_height(1280.0), 56.0);
    assert_eq!(header_height(960.0), 104.0);
    assert_eq!(header_height(390.0), 148.0);
    for width in [390.0, 760.0, 1180.0] {
        let layout = BuildingLayout::new(width, 720.0, true);
        assert!(layout.world.y >= header_height(width));
        assert!(layout.panel.y >= header_height(width));
    }
}

#[test]
fn panel_clicks_cannot_be_building_clicks() {
    for width in [390.0, 760.0, 1280.0] {
        let layout = BuildingLayout::new(width, 844.0, true);
        for point in [
            layout.panel.point(),
            layout.panel.center(),
            macroquad::prelude::vec2(layout.panel.right(), layout.panel.bottom()),
        ] {
            assert!(!layout.world.contains(point));
        }
    }
}

#[test]
fn small_controls_grow_to_a_touch_target_about_their_centre() {
    for rect in [
        Rect::new(10.0, 20.0, 64.0, 34.0),
        Rect::new(0.0, 0.0, 30.0, 30.0),
        Rect::new(5.0, 5.0, 200.0, 40.0),
    ] {
        let grown = touch_rect(rect);
        assert!(grown.w >= TOUCH_MIN && grown.h >= TOUCH_MIN);
        assert!((grown.center() - rect.center()).length() < 1e-4);
    }
    let roomy = Rect::new(0.0, 0.0, 120.0, 48.0);
    assert_eq!(touch_rect(roomy), roomy);
}

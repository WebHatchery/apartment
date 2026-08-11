use super::*;

#[test]
fn ten_unit_cutaway_fits_short_workspace() {
    let view = Rect::new(0.0, 120.0, 416.0, 440.0);
    let metrics = cutaway_metrics(view, 5, 2);
    let top_floor_y = metrics.hallway_y - space::SM - metrics.unit_h - 4.0 * metrics.floor_step;

    assert!(top_floor_y >= view.y + 40.0);
    assert!(metrics.units_left >= view.x);
    assert!(metrics.units_left + metrics.units_width <= view.right());
    assert!(metrics.unit_w >= 68.0);
}

#[test]
fn four_across_manor_units_remain_clickable_at_narrow_breakpoint() {
    let view = Rect::new(0.0, 120.0, 416.0, 436.0);
    let metrics = cutaway_metrics(view, 4, 4);

    assert!(metrics.unit_w >= 80.0);
    assert!(metrics.unit_h >= 60.0);
    assert!(metrics.units_left + metrics.units_width <= view.right());
}

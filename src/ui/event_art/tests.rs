use super::*;

#[test]
fn every_shipped_event_scene_has_a_unique_atlas_cell() {
    let ids = [
        "event_rent_collected",
        "event_tenant_moved_in",
        "event_tenant_moved_out",
        "event_noise_complaint",
        "event_pipe_burst",
        "event_inspection",
        "event_heatwave",
        "event_new_business",
        "event_developer_offer",
    ];
    let mut cells = ids.map(event_tile).to_vec();
    cells.sort_by(|left, right| left.partial_cmp(right).unwrap());
    cells.dedup();
    assert_eq!(cells.len(), ids.len());
    assert!(!cells.contains(&None));
}

use super::*;

#[test]
fn every_workspace_owns_a_unique_atlas_cell() {
    let ids = [
        "icon_key",
        "icon_application",
        "icon_money",
        "icon_market",
        "icon_mail",
        "icon_inspection",
    ];
    let mut cells = ids.map(workspace_icon_tile).to_vec();
    cells.sort_by(|left, right| left.partial_cmp(right).unwrap());
    cells.dedup();
    assert_eq!(cells.len(), ids.len());
    assert!(!cells.contains(&None));
}

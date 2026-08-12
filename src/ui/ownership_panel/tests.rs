use super::*;

#[test]
fn paging_clamps_to_a_full_last_page() {
    assert_eq!(ownership_page(6, 3, 0.0, 58.0), (0, 3));
    assert_eq!(ownership_page(6, 3, 174.0, 58.0), (3, 3));
    assert_eq!(ownership_page(6, 3, 999.0, 58.0), (3, 3));
}

#[test]
fn paging_disables_when_every_unit_fits() {
    assert_eq!(ownership_page(3, 4, 116.0, 58.0), (0, 0));
    assert_eq!(ownership_page(0, 1, -58.0, 58.0), (0, 0));
}

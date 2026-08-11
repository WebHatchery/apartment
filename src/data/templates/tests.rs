use super::load_templates;

#[test]
fn campaign_roster_is_a_contiguous_unlock_chain() {
    let templates = load_templates().map(|t| t.templates).unwrap_or_default();
    assert!(templates.len() >= 6, "expected the full campaign roster");

    let mut orders: Vec<u32> = templates.iter().map(|t| t.unlock_order).collect();
    orders.sort_unstable();
    // Unlock chain must be 0..N with no gaps or duplicates (progression relies
    // on finding order == current+1).
    for (i, order) in orders.iter().enumerate() {
        assert_eq!(*order, i as u32, "unlock_order chain is not contiguous");
    }
    // Every building sits in a real neighborhood (0..=3).
    assert!(templates.iter().all(|t| t.neighborhood_id <= 3));
}

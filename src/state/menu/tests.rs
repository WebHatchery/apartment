use super::*;

#[test]
fn campaign_neighborhoods_map_to_distinct_facades() {
    let mut cells = [0, 1, 2, 3].map(campaign_facade_tile).to_vec();
    cells.sort_by(|left, right| left.partial_cmp(right).unwrap());
    cells.dedup();
    assert_eq!(cells.len(), 4);
}

#[test]
fn unknown_neighborhood_uses_the_suburban_facade() {
    assert_eq!(campaign_facade_tile(99), campaign_facade_tile(1));
}

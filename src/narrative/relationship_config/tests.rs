use super::load_relationship_config;

#[test]
fn relationship_events_load_and_parse_all_categories() {
    let cfg = load_relationship_config();
    // Every authored event (incl. the expanded banks) must deserialize.
    assert!(cfg.hostile.len() >= 8, "hostile: {}", cfg.hostile.len());
    assert!(cfg.friendly.len() >= 7, "friendly: {}", cfg.friendly.len());
    assert!(cfg.romance.len() >= 3, "romance: {}", cfg.romance.len());
    assert!(cfg.dilemma.len() >= 3, "dilemma: {}", cfg.dilemma.len());
}

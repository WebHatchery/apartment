use super::load_events_config;

#[test]
fn tenant_requests_load_for_every_archetype() {
    let cfg = load_events_config();
    for archetype in ["Student", "Professional", "Artist", "Family", "Elderly"] {
        let reqs = cfg
            .requests
            .get(archetype)
            .unwrap_or_else(|| panic!("no requests for {}", archetype));
        // Each archetype now offers several request kinds plus a "None" filler.
        assert!(
            reqs.len() >= 4,
            "{} has too few request templates: {}",
            archetype,
            reqs.len()
        );
    }
}

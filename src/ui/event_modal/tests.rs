use super::*;

#[test]
fn every_event_category_has_a_shipped_illustration_mapping() {
    let categories = [
        NarrativeEventType::NeighborhoodNews,
        NarrativeEventType::CityEvent,
        NarrativeEventType::TenantStory { tenant_id: 1 },
        NarrativeEventType::BuildingMilestone,
        NarrativeEventType::CharacterEncounter,
        NarrativeEventType::ExternalOffer,
        NarrativeEventType::SeasonalEvent,
        NarrativeEventType::RelationshipEvent,
    ];

    for category in categories {
        assert!(event_texture_id(&category).starts_with("event_"));
    }
}

use super::*;

#[test]
fn test_event_creation() {
    let event = NarrativeEvent::news(0, 1, "Test", "Description");
    assert!(!event.read);
    assert!(!event.requires_response);
}

#[test]
fn test_event_system() {
    let mut system = NarrativeEventSystem::new();
    let _id = system.add_event(NarrativeEvent::news(0, 1, "Test", "Desc"));
    assert_eq!(system.events.len(), 1);
}

#[test]
fn news_events_load_from_json() {
    let news = load_news_events();
    assert!(news.neighborhood.len() >= 8);
    assert!(news.city.len() >= 8);
    // Every season (0..=3) must have at least one seasonal template.
    for season in 0..4 {
        assert!(
            news.seasonal.iter().any(|t| t.season == season),
            "no seasonal template for season {}",
            season
        );
    }
}

#[test]
fn news_effect_spec_injects_runtime_neighborhood_id() {
    let spec = NewsEffectSpec {
        kind: "neighborhood_reputation".to_string(),
        amount: 5.0,
    };
    match spec.to_effect(3) {
        NarrativeEffect::NeighborhoodReputation {
            neighborhood_id,
            change,
        } => {
            assert_eq!(neighborhood_id, 3);
            assert_eq!(change, 5);
        }
        other => panic!("expected NeighborhoodReputation, got {:?}", other),
    }
}

#[test]
fn generated_neighborhood_event_targets_its_neighborhood() {
    use crate::city::{Neighborhood, NeighborhoodType};
    let news = load_news_events();
    let neighborhood = Neighborhood::new(7, NeighborhoodType::Downtown, "Test");
    let event = NarrativeEventSystem::neighborhood_event(&news, 1, &neighborhood);
    assert!(!event.headline.is_empty());
    assert_eq!(event.related_neighborhood_id, Some(7));
}

#[test]
fn expired_event_returns_default_effect() {
    let mut system = NarrativeEventSystem::new();
    let mut event = NarrativeEvent::with_choices(
        0,
        NarrativeEventType::CityEvent,
        1,
        "Tax Review",
        "The city needs a response.",
        vec![NarrativeChoice {
            label: "Object".to_string(),
            description: "Push back.".to_string(),
            effect: NarrativeEffect::Money { amount: 500 },
            reputation_change: 0,
        }],
    );
    event.default_effect = NarrativeEffect::Money { amount: -250 };
    event.response_deadline = Some(1);

    let event_id = system.add_event(event);
    let effects = system.expire_due_events(2);

    assert_eq!(effects.len(), 1);
    assert!(matches!(
        effects[0],
        NarrativeEffect::Money { amount: -250 }
    ));
    assert!(system.pending_events.is_empty());
    assert!(system.processed_events.contains(&event_id));
    assert!(system.events[0].read);
}

#[test]
fn no_choice_event_processes_default_effect() {
    let mut system = NarrativeEventSystem::new();
    let mut event = NarrativeEvent::news(0, 1, "Grant", "A grant arrived.");
    event.default_effect = NarrativeEffect::Money { amount: 750 };
    let event_id = system.add_event(event);

    let outcome = system.process_choice(event_id, 0);

    assert!(matches!(
        outcome.map(|o| o.effect),
        Some(NarrativeEffect::Money { amount: 750 })
    ));
    assert!(system.processed_events.contains(&event_id));
}

#[test]
fn immediate_news_is_processed_only_once() {
    let mut system = NarrativeEventSystem::new();
    let mut event = NarrativeEvent::news(0, 1, "Market shifts", "Demand rose.");
    event.default_effect = NarrativeEffect::EconomyChange {
        economy_health_change: 0.1,
    };
    system.add_event(event);

    let first = system.process_immediate_events();
    let second = system.process_immediate_events();

    assert_eq!(first.len(), 1);
    assert!(second.is_empty());
    assert!(system.events[0].read);
}

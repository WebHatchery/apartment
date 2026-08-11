use super::*;

#[test]
fn test_story_generation() {
    let story = TenantStory::generate(0, &TenantArchetype::Student);
    assert!(!story.job_title.is_empty());
    assert!(!story.hometown.is_empty());
}

#[test]
fn test_request_effects() {
    let request = TenantRequest::Pet {
        pet_type: "cat".to_string(),
    };
    let approval = request.approval_effect();
    matches!(approval, StoryImpact::Happiness(h) if h > 0);
}

#[test]
fn job_loss_is_a_disruptive_life_change() {
    let cfg = LifeEventsConfig::default();
    let (impact, description) = LifeChangeType::JobLoss.impact(&cfg);
    assert!(!description.is_empty());
    match impact {
        StoryImpact::Multiple(effects) => {
            assert!(effects
                .iter()
                .any(|e| matches!(e, StoryImpact::MoveOutRisk(_))));
            assert!(effects
                .iter()
                .any(|e| matches!(e, StoryImpact::Happiness(h) if *h < 0)));
        }
        other => panic!("expected multiple impacts, got {:?}", other),
    }
}

#[test]
fn new_job_is_a_positive_life_change() {
    let cfg = LifeEventsConfig::default();
    let (impact, _) = LifeChangeType::NewJob { better: true }.impact(&cfg);
    match impact {
        StoryImpact::Multiple(effects) => {
            assert!(effects
                .iter()
                .any(|e| matches!(e, StoryImpact::Happiness(h) if *h > 0)));
        }
        other => panic!("expected multiple impacts, got {:?}", other),
    }
}

#[test]
fn every_archetype_has_eligible_life_changes() {
    for archetype in [
        TenantArchetype::Student,
        TenantArchetype::Professional,
        TenantArchetype::Artist,
        TenantArchetype::Family,
        TenantArchetype::Elderly,
    ] {
        assert!(!LifeChangeType::eligible_for(&archetype).is_empty());
    }
}

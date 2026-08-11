use super::*;

#[test]
fn test_tutorial_progression() {
    let mut tutorial = TutorialManager::new();
    assert!(tutorial.active);
    assert_eq!(
        tutorial.current_milestone,
        Some(TutorialMilestone::InheritedMess)
    );

    tutorial.complete_milestone(TutorialMilestone::InheritedMess);
    assert_eq!(
        tutorial.current_milestone,
        Some(TutorialMilestone::FirstResident)
    );

    tutorial.complete_milestone(TutorialMilestone::FirstResident);
    assert_eq!(tutorial.current_milestone, Some(TutorialMilestone::TheLeak));

    tutorial.complete_milestone(TutorialMilestone::TheLeak);
    assert_eq!(
        tutorial.current_milestone,
        Some(TutorialMilestone::Complete)
    );

    tutorial.complete_milestone(TutorialMilestone::Complete);
    assert!(!tutorial.active);
    assert!(tutorial.is_complete());
}

#[test]
fn test_npc_relationship() {
    let mut tutorial = TutorialManager::new();
    let mentor_id = tutorial.mentor.id;

    assert_eq!(tutorial.mentor.relationship, 50);
    tutorial.modify_relationship(mentor_id, 20);
    assert_eq!(tutorial.mentor.relationship, 70);
    tutorial.modify_relationship(mentor_id, 50);
    assert_eq!(tutorial.mentor.relationship, 100); // Clamped
}

#[test]
fn starter_resident_does_not_complete_acquisition_lesson() {
    let mut tutorial = TutorialManager::new();
    tutorial.set_resident_baseline(1);
    assert!(!tutorial.has_new_resident(1));
    assert!(tutorial.has_new_resident(2));
}

#[test]
fn hint_only_emits_once_per_qualifying_month() {
    let mut tutorial = TutorialManager::new();
    assert!(!tutorial.should_emit_hint(4));
    assert!(tutorial.should_emit_hint(5));
    assert!(!tutorial.should_emit_hint(5));
    assert!(tutorial.should_emit_hint(10));
}

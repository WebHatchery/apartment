use super::*;

#[test]
fn final_milestone_closes_after_mentor_messages_are_read() {
    let mut state = GameplayState::new();
    state.tutorial.current_milestone = Some(TutorialMilestone::Complete);
    state.tutorial.pending_messages.clear();

    update_tutorial(&mut state);

    assert!(state.tutorial.is_complete());
    assert!(!state.tutorial.active);
    assert!(state.tutorial.current_milestone.is_none());
}

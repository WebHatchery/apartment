use super::*;

#[test]
fn overlays_reject_gameplay_actions_but_accept_event_responses() {
    let gameplay_action = UiAction::EndTurn;
    let pause_action = UiAction::OpenPauseMenu;
    let response = UiAction::ResolveEventChoice {
        event_id: 1,
        choice_index: 0,
    };

    assert!(!action_allowed_while_blocked(false, true, &gameplay_action));
    assert!(!action_allowed_while_blocked(false, true, &pause_action));
    assert!(action_allowed_while_blocked(false, true, &response));
    assert!(!action_allowed_while_blocked(true, false, &gameplay_action));
}

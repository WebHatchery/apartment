use crate::state::GameplayState;

#[test]
fn test_save_load_serialization() {
    // 1. Create a dummy state
    let mut state = GameplayState::new();
    state.funds.balance = 9999;
    state.current_tick = 5;
    state.next_tenant_id = 10;

    // 2. Serialize
    let serialized = serde_json::to_string(&state);
    assert!(
        serialized.is_ok(),
        "Failed to serialize: {:?}",
        serialized.as_ref().err()
    );
    let Ok(json) = serialized else {
        return;
    };

    // 3. Deserialize
    let deserialized: Result<GameplayState, _> = serde_json::from_str(&json);
    assert!(
        deserialized.is_ok(),
        "Failed to deserialize: {:?}",
        deserialized.as_ref().err()
    );
    let Ok(loaded) = deserialized else {
        return;
    };

    // 4. Verify fields
    assert_eq!(loaded.funds.balance, 9999);
    assert_eq!(loaded.current_tick, 5);
    assert_eq!(loaded.next_tenant_id, 10);
    // Default values for skipped fields
    assert_eq!(loaded.pending_actions.len(), 0);
}

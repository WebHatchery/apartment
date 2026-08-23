use super::*;
use crate::state::GameplayState;

fn pre_versioned_save_fixture() -> String {
    let mut state = GameplayState::new();
    state.current_building_id.clear();

    let tenant = state
        .tenants
        .first_mut()
        .expect("starter fixture needs its resident");
    tenant.building_id = u32::MAX;
    tenant.apartment_id = None;

    serde_json::to_string(&state).expect("legacy fixture should serialize")
}

#[test]
fn versioned_save_round_trips_gameplay_state() {
    let mut state = GameplayState::new();
    state.funds.balance = 9999;
    state.current_tick = 5;
    state.next_tenant_id = 10;

    let json =
        serde_json::to_string(&current_save(&state)).expect("versioned save should serialize");
    let value = serde_json::from_str(&json).expect("versioned save should deserialize");
    let loaded = decode_save(value).expect("current save should load");

    assert!(json.contains("\"save_format_version\":1"));
    assert_eq!(loaded.funds.balance, 9999);
    assert_eq!(loaded.current_tick, 5);
    assert_eq!(loaded.next_tenant_id, 10);
    assert_eq!(loaded.pending_actions.len(), 0);
}

#[test]
fn pre_versioned_fixture_restores_tenant_address_and_building_id() {
    let fixture = pre_versioned_save_fixture();
    let value = serde_json::from_str(&fixture).expect("legacy save should deserialize");
    let mut loaded = decode_save(value).expect("legacy save should be accepted");

    loaded.post_load();

    let tenant = loaded
        .tenants
        .first()
        .expect("fixture resident is retained");
    assert_eq!(loaded.current_building_id, "mvp_default");
    assert_eq!(
        loaded.config.regulations.random_inspection_chance_percent,
        loaded
            .config
            .difficulty
            .get("Easy")
            .expect("Easy template has difficulty rules")
            .random_inspection_chance_percent
    );
    assert_eq!(tenant.building_id, loaded.active_building_id());
    assert!(tenant.apartment_id.is_some());
    assert!(loaded
        .ever_occupied_buildings
        .contains(&loaded.active_building_id()));
}

#[test]
fn newer_versioned_save_is_rejected_without_a_migration_path() {
    let state = GameplayState::new();
    let json = serde_json::json!({
        "save_format_version": CURRENT_SAVE_FORMAT_VERSION + 1,
        "state": { "future_state": state },
    })
    .to_string();
    let value = serde_json::from_str(&json).expect("future envelope should deserialize");
    let result = decode_save(value);

    assert!(matches!(
        result,
        Err(error) if error.kind() == std::io::ErrorKind::Unsupported
    ));
}

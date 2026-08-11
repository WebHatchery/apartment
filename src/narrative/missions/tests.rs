use super::*;

#[test]
fn test_mission_lifecycle() {
    let mut manager = MissionManager::new();

    let mission = Mission::new(
        0,
        "Test Mission",
        "A test",
        0,
        MissionGoal::AcquireBuilding,
        MissionReward::Money(1000),
        Some(10),
    );

    let id = manager.add_mission(mission);
    assert_eq!(manager.available_missions().len(), 1);

    manager.accept_mission(id, 1);
    assert_eq!(manager.active_missions().len(), 1);
    assert_eq!(manager.available_missions().len(), 0);
}

#[test]
fn test_mission_expiration() {
    let mut manager = MissionManager::new();

    let mission = Mission::new(
        0,
        "Expiring Mission",
        "Will expire",
        0,
        MissionGoal::AcquireBuilding,
        MissionReward::Money(1000),
        Some(5),
    );

    let id = manager.add_mission(mission);
    manager.accept_mission(id, 1);

    manager.check_expirations(10);

    let mission = manager.missions.iter().find(|m| m.id == id);
    assert!(mission.is_some(), "expected expiring mission to exist");
    if let Some(mission) = mission {
        assert_eq!(mission.status, MissionStatus::Expired);
    }
}

#[test]
fn missions_load_from_json_and_gate_by_month() {
    let mut manager = MissionManager::new();
    manager.generate_available_missions(0);

    // The three starter missions are available from month 0.
    assert!(manager
        .missions
        .iter()
        .any(|m| m.title == "Student Housing Initiative"));
    assert!(manager.missions.iter().any(|m| m.title == "Full House"));
    assert!(manager.missions.len() >= 3);
    // A late-game mission is not available yet.
    assert!(!manager.missions.iter().any(|m| m.title == "Expansion Race"));

    // By month 6 it unlocks — and re-running doesn't duplicate anything.
    manager.generate_available_missions(6);
    assert!(manager.missions.iter().any(|m| m.title == "Expansion Race"));
    let full_house = manager
        .missions
        .iter()
        .filter(|m| m.title == "Full House")
        .count();
    assert_eq!(full_house, 1, "missions must not duplicate across calls");
}

#[test]
fn test_legacy_system() {
    let mut manager = MissionManager::new();

    manager.record_legacy_event(15, "The Great Fire", "A fire broke out in Building A.");
    manager.grant_award(2025, "Best Managed Property", "Sunset Apartments");

    assert_eq!(manager.legacy_events.len(), 1);
    assert_eq!(manager.awards.len(), 1);
    assert_eq!(manager.awards[0].year, 2025);
}

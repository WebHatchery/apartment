use super::*;
use crate::narrative::missions::Mission;
use crate::narrative::{MissionGoal, MissionStatus};
use crate::state::GameplayState;

#[test]
fn maintain_happiness_goal_accrues_a_month() {
    let mut state = GameplayState::new();
    if state.tenants.is_empty() {
        return;
    }
    for tenant in &mut state.tenants {
        tenant.happiness = 90;
    }
    let id = state.missions.add_mission(Mission::new(
        0,
        "Steady Ship",
        "Keep tenants content.",
        0,
        MissionGoal::MaintainHappiness {
            threshold: 50.0,
            months: 3,
            current_months: 0,
        },
        MissionReward::Money(100),
        None,
    ));
    state.missions.accept_mission(id, 1);

    update_missions(&mut state);

    let mission = state.missions.missions.iter().find(|m| m.id == id).unwrap();
    assert!(matches!(
        mission.goal,
        MissionGoal::MaintainHappiness {
            current_months: 1,
            ..
        }
    ));
    // One month of three: still in progress, no reward granted yet.
    assert_eq!(mission.status, MissionStatus::Active);
}

#[test]
fn full_repair_goal_stays_incomplete_for_a_neglected_building() {
    let mut state = GameplayState::new();
    // Drive the building below the repair bar so completion (and its UI
    // feedback, which needs a GL context) can't fire in the test.
    for apt in &mut state.building.apartments {
        apt.condition = 40;
    }
    state.building.hallway_condition = 40;
    let id = state.missions.add_mission(Mission::new(
        0,
        "Fix It Up",
        "Restore the building.",
        0,
        MissionGoal::FullRepair { building_id: 0 },
        MissionReward::Money(100),
        None,
    ));
    state.missions.accept_mission(id, 1);

    update_missions(&mut state);

    let mission = state.missions.missions.iter().find(|m| m.id == id).unwrap();
    assert_eq!(mission.status, MissionStatus::Active);
}

#[test]
fn full_repair_goal_uses_its_target_building() {
    let mut state = GameplayState::new();
    for apartment in &mut state.building.apartments {
        apartment.condition = 40;
    }
    state.building.hallway_condition = 40;

    let mut repaired = crate::building::Building::new("Repaired", 1, 1);
    repaired.apartments[0].condition = 95;
    repaired.hallway_condition = 95;
    state.city.add_building(repaired, 0).unwrap();

    let repaired = fully_repaired_buildings(&state);
    assert_eq!(repaired.get(&0), Some(&false));
    assert_eq!(repaired.get(&1), Some(&true));
}

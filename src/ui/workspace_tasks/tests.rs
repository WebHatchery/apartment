use super::*;

#[test]
fn every_mission_goal_owns_a_unique_atlas_cell() {
    let goals = [
        MissionGoal::HouseTenants {
            count: 1,
            archetype: None,
        },
        MissionGoal::ReachOccupancy { percentage: 1.0 },
        MissionGoal::MaintainHappiness {
            threshold: 70.0,
            months: 1,
            current_months: 0,
        },
        MissionGoal::PerfectCollection {
            months: 1,
            current_months: 0,
        },
        MissionGoal::FullRepair { building_id: 0 },
        MissionGoal::AcquireBuilding,
    ];
    let goal_count = goals.len();
    let mut cells = goals.map(|goal| mission_icon_tile(&goal)).to_vec();
    cells.sort_by(|left, right| left.partial_cmp(right).unwrap());
    cells.dedup();
    assert_eq!(cells.len(), goal_count);
}

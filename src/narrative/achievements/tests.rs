use super::AchievementSystem;

#[test]
fn achievements_load_from_json() {
    let system = AchievementSystem::new();
    // The full expanded set must deserialize (incl. HappinessAtLeast).
    assert!(system.list.len() >= 20, "loaded {}", system.list.len());
    // Ids are unique.
    let mut ids: Vec<&str> = system.list.iter().map(|a| a.id.as_str()).collect();
    ids.sort_unstable();
    let unique = {
        let mut u = ids.clone();
        u.dedup();
        u.len()
    };
    assert_eq!(ids.len(), unique, "duplicate achievement ids");
}

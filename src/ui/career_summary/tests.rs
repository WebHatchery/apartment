use super::*;

#[test]
fn career_rank_thresholds_are_exhaustive() {
    assert_eq!(career_rank(-1).0, "Building in distress");
    assert_eq!(career_rank(1).0, "Struggling owner");
    assert_eq!(career_rank(10_001).0, "Property manager");
    assert_eq!(career_rank(25_001).0, "Successful landlord");
    assert_eq!(career_rank(50_001).0, "Real estate tycoon");
}

#[test]
fn rank_threshold_values_stay_in_the_lower_band() {
    assert_eq!(career_rank(0).0, "Building in distress");
    assert_eq!(career_rank(10_000).0, "Struggling owner");
    assert_eq!(career_rank(25_000).0, "Property manager");
    assert_eq!(career_rank(50_000).0, "Successful landlord");
}

#[test]
fn achievement_conditions_select_their_visual_family() {
    assert_eq!(
        achievement_emblem_tile(&AchievementCondition::TotalTenants { min: 1 }),
        (0.0, 0.0)
    );
    assert_eq!(
        achievement_emblem_tile(&AchievementCondition::HappinessAtLeast { min: 70 }),
        (1.0, 0.0)
    );
    assert_eq!(
        achievement_emblem_tile(&AchievementCondition::Funds { min: 10_000 }),
        (0.0, 1.0)
    );
    assert_eq!(
        achievement_emblem_tile(&AchievementCondition::GameComplete),
        (1.0, 1.0)
    );
}

use super::*;

#[test]
fn test_gentrification_score() {
    let mut tracker = GentrificationTracker::new();
    let config = GentrificationConfig::default();
    assert_eq!(tracker.gentrification_score, 0);

    tracker.record_rent_change(0, 1, 500, 700, &config);
    assert!(tracker.gentrification_score > 0);
}

#[test]
fn test_demographic_diversity() {
    let snapshot = DemographicSnapshot {
        month: 0,
        student_count: 2,
        professional_count: 2,
        artist_count: 2,
        family_count: 0,
        elderly_count: 0,
        average_rent: 600,
    };

    assert!(snapshot.diversity_score() >= 40); // 3 different types
}

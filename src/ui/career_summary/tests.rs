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

use super::*;

#[test]
fn hard_difficulty_tightens_the_rules() {
    let mut config = GameConfig::default();
    let base_fine = config.regulations.fine_multiplier;
    let funds = config.apply_difficulty("Hard");
    assert_eq!(funds, 3500);
    assert!(config.regulations.fine_multiplier > base_fine);
    assert_eq!(config.regulations.random_inspection_chance_percent, 12);
    assert_eq!(config.tenant_risk.problem_applicant_chance_percent, 28);
}

#[test]
fn easy_difficulty_is_gentler_than_medium() {
    let mut easy = GameConfig::default();
    let easy_funds = easy.apply_difficulty("Easy");
    let mut medium = GameConfig::default();
    let medium_funds = medium.apply_difficulty("Medium");
    assert!(easy_funds > medium_funds);
    assert!(easy.regulations.fine_multiplier < medium.regulations.fine_multiplier);
}

#[test]
fn unknown_difficulty_defaults_to_5000_without_changes() {
    let mut config = GameConfig::default();
    let base_chance = config.regulations.random_inspection_chance_percent;
    let funds = config.apply_difficulty("Nonexistent");
    assert_eq!(funds, 5000);
    assert_eq!(
        config.regulations.random_inspection_chance_percent,
        base_chance
    );
}

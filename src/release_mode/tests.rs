use super::*;

#[test]
fn release_mode_uses_an_isolated_save_namespace() {
    assert_eq!(
        save_namespace(),
        if cfg!(feature = "demo") {
            "apartment_manager_demo"
        } else {
            "apartment_manager"
        }
    );
}

#[test]
fn demo_build_caps_a_run_at_about_ninety_days() {
    let mut config = GameConfig::default();
    apply_config(&mut config);

    let expected = if cfg!(feature = "demo") { 3 } else { 36 };
    assert_eq!(config.win_conditions.game_duration_ticks, Some(expected));
}

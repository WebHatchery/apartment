use super::*;

#[test]
fn critical_failure_config_escalates_with_age() {
    let cfg = CriticalFailureConfig::default();
    assert!(
        cfg.aging_probability_per_year > 0,
        "failures should age upward"
    );
    assert!(
        cfg.aging_cost_per_year > 0,
        "repairs should get costlier with age"
    );
    assert!(cfg.structural_repair_cost > cfg.boiler_repair_cost);
}

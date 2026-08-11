use super::*;

#[test]
fn test_regulation_fines() {
    let mut reg = Regulation::new(RegulationType::FireSafety);
    assert!(reg.compliant);
    assert_eq!(reg.current_fine(), 0);

    reg.add_violation();
    assert!(!reg.compliant);
    assert!(reg.current_fine() > 0);
}

#[test]
fn test_compliance_system() {
    let mut system = ComplianceSystem::new();
    system.init_building_regulations(0, false);

    assert!(system.get_regulations(0).is_some());
    assert!(!system.has_violations(0));
}

#[test]
fn first_failed_inspection_warns_before_fining() {
    let cfg = RegulationsConfig::default();
    let mut system = ComplianceSystem::new();
    system.init_building_regulations(0, false);

    // A condition well below the pass threshold cites every regulation.
    let inspection = system.run_inspection(0, 10, 6, InspectionTrigger::Random, &cfg);

    assert_eq!(inspection.total_fines, 0);
    assert!(inspection.results.iter().all(|r| !r.passed));
    assert_eq!(system.unpaid_fines, 0);
    assert!(system.compliance_reputation < 100);
    assert!(!system.pending_fixes.is_empty());
    assert!(system.has_violations(0));

    let repeat = system.run_inspection(0, 10, 7, InspectionTrigger::Random, &cfg);
    assert!(repeat.total_fines > 0);
    assert_eq!(system.unpaid_fines, repeat.total_fines);
}

#[test]
fn repairs_clear_pending_orders_before_the_deadline() {
    let cfg = RegulationsConfig::default();
    let mut system = ComplianceSystem::new();
    system.init_building_regulations(0, false);
    system.run_inspection(0, 10, 6, InspectionTrigger::Random, &cfg);

    assert!(system.resolve_fixes_if_compliant(0, 90, cfg.pass_condition_threshold) > 0);
    system.tick(20);
    assert_eq!(system.unpaid_fines, 0);
    assert!(system.pending_fixes.is_empty());
}

#[test]
fn clean_inspection_passes_a_maintained_building() {
    let cfg = RegulationsConfig::default();
    let mut system = ComplianceSystem::new();
    system.init_building_regulations(0, false);

    let inspection = system.run_inspection(0, 90, 6, InspectionTrigger::Random, &cfg);

    assert_eq!(inspection.total_fines, 0);
    assert!(inspection.results.iter().all(|r| r.passed));
    assert_eq!(system.unpaid_fines, 0);
    assert!(!system.has_violations(0));
}

#[test]
fn scheduled_inspection_only_grades_due_regulations() {
    let cfg = RegulationsConfig::default();
    let mut system = ComplianceSystem::new();
    system.init_building_regulations(0, false);

    // Nothing is due on a freshly initialised building, so a scheduled
    // inspection grades nothing and levies no fine.
    let inspection = system.run_inspection(0, 10, 1, InspectionTrigger::Scheduled, &cfg);

    assert!(inspection.results.is_empty());
    assert_eq!(system.unpaid_fines, 0);
}

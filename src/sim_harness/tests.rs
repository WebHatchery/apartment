use super::*;

/// Fast smoke test (always runs in CI): a single greedy playthrough must
/// complete 36 months without panicking and stay internally consistent.
#[test]
fn balance_harness_runs_without_panic() {
    rng::srand(1);
    let config = crate::data::config::load_config();
    let template = crate::data::templates::load_templates()
        .and_then(|templates| templates.templates.into_iter().next())
        .expect("starter template");
    let strat = strategies()[0];
    let duration = config.win_conditions.game_duration_ticks.unwrap_or(36);
    let result = Sim::new(&template, 1).run(&strat, duration);

    assert!(result.months.len() as u32 <= duration);
    assert!(!result.months.is_empty());
    assert!(result.end_occupancy >= 0.0 && result.end_occupancy <= 1.0);
}

#[test]
fn representative_balance_targets_hold() {
    let templates = crate::data::templates::load_templates()
        .expect("campaign templates")
        .templates;
    let policies = strategies();
    let mut bankruptcies_by_difficulty = std::collections::HashMap::new();

    for template in templates {
        let greedy = summarize(&template, &policies[0], 20);
        let investor = summarize(&template, &policies[1], 20);
        let neglect = summarize(&template, &policies[2], 20);
        let rent_max = summarize(&template, &policies[3], 20);
        let required_score = greedy.mean_score + greedy.mean_score.abs() / 10;

        assert!(
            investor.mean_score >= required_score,
            "{} Investor score {} did not beat Greedy {} by 10%",
            template.name,
            investor.mean_score,
            greedy.mean_score
        );
        assert!(
            investor.mean_final_balance >= greedy.mean_final_balance
                || investor.mean_score > greedy.mean_score,
            "{} Investor trailed Greedy in cash and score",
            template.name
        );
        assert!(
            (12.0..=18.0).contains(&investor.mean_payback_months),
            "{} investment payback was {:.1} months",
            template.name,
            investor.mean_payback_months
        );
        assert!(investor.mean_score > rent_max.mean_score);
        assert!(neglect.mean_end_condition < investor.mean_end_condition);
        assert!(neglect.mean_departures > investor.mean_departures);

        if template.difficulty == "Easy" {
            assert!(greedy.mean_final_balance < 20 * 10_000);
            assert!(investor.mean_final_balance < 20 * 10_000);
        }
        *bankruptcies_by_difficulty
            .entry(template.difficulty.clone())
            .or_insert(0usize) += investor.bankruptcy_count;
    }

    assert_eq!(bankruptcies_by_difficulty.get("Easy"), Some(&0));
    assert!(
        bankruptcies_by_difficulty
            .get("Medium")
            .copied()
            .unwrap_or(0)
            > 0
    );
    assert!(bankruptcies_by_difficulty.get("Hard").copied().unwrap_or(0) > 0);
}

/// Full balance report. Ignored by default (writes a file, runs many seeds).
/// Run with: `cargo test balance_report -- --ignored --nocapture`
#[test]
#[ignore]
fn balance_report() {
    let seeds = 60;
    let report = generate_report(seeds);
    println!("\n{}\n", report);

    std::fs::write("balance_report.md", &report).expect("write balance_report.md");
    println!("Wrote balance_report.md");
}

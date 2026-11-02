//! Built-in defaults for `GameConfig`, used when JSON is missing a block.

use super::config::*;
use std::collections::HashMap;

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
            starting_conditions: default_starting_conditions(),
            economy: default_economy(),
            decay: DecayConfig {
                apartment_per_tick: 3,
                hallway_per_tick: 1,
            },
            happiness: default_happiness(),
            win_conditions: WinConditions {
                full_occupancy_required: true,
                min_ticks_for_victory: 6,
                game_duration_ticks: Some(36),
            },
            applications: ApplicationConfig {
                expire_after_ticks: 3,
                base_per_vacancy: 0.5,
                appeal_bonus_divisor: 50,
                reputation_influence: 0.5,
            },
            ui: UiConfig {
                upgrade_labels: default_upgrade_labels(),
            },
            upgrades: HashMap::new(),
            matching: MatchingConfig::default(),
            thresholds: ThresholdsConfig::default(),
            operating_costs: OperatingCostsConfig::default(),
            staff_effects: StaffEffectsConfig::default(),
            tenant_risk: TenantRiskConfig::default(),
            vetting: VettingConfig::default(),
            marketing: MarketingConfig::default(),
            relationships: RelationshipsConfig::default(),
            cohesion: CohesionConfig::default(),
            gentrification: GentrificationConfig::default(),
            regulations: RegulationsConfig::default(),
            life_events: LifeEventsConfig::default(),
            critical_failures: CriticalFailureConfig::default(),
            portfolio: PortfolioConfig::default(),
            turn_effects: TurnEffectsConfig::default(),
            difficulty: default_difficulty_modifiers(),
            theme: ThemeConfig::default(),
            layout: LayoutConfig::default(),
            ui_thresholds: UiThresholdsConfig::default(),
            apartment: ApartmentPropertiesConfig::default(),
        }
    }
}

fn default_starting_conditions() -> StartingConditions {
    StartingConditions {
        player_money: 5000,
        starting_tenants: 1,
        building_floors: 3,
        units_per_floor: 2,
    }
}

fn default_economy() -> EconomyConfig {
    EconomyConfig {
        repair_cost_per_point: 10,
        hallway_repair_cost_per_point: 15,
        staff_costs: default_staff_costs(),
    }
}

fn default_difficulty_modifiers() -> HashMap<String, DifficultyModifiers> {
    let mut tiers = HashMap::new();
    tiers.insert(
        "Easy".to_string(),
        DifficultyModifiers {
            starting_funds: 7000,
            inspection_fine_multiplier: 0.75,
            random_inspection_chance_percent: 5,
            problem_applicant_chance_percent: 10,
            operating_cost_multiplier: 0.85,
        },
    );
    tiers.insert(
        "Medium".to_string(),
        DifficultyModifiers {
            starting_funds: 5000,
            inspection_fine_multiplier: 1.0,
            random_inspection_chance_percent: 8,
            problem_applicant_chance_percent: 18,
            operating_cost_multiplier: 1.0,
        },
    );
    tiers.insert(
        "Hard".to_string(),
        DifficultyModifiers {
            starting_funds: 3500,
            inspection_fine_multiplier: 1.5,
            random_inspection_chance_percent: 12,
            problem_applicant_chance_percent: 28,
            operating_cost_multiplier: 1.15,
        },
    );
    tiers
}

fn default_staff_costs() -> HashMap<String, i32> {
    let mut costs = HashMap::new();
    costs.insert("janitor".to_string(), 150);
    costs.insert("security".to_string(), 320);
    costs.insert("manager".to_string(), 480);
    costs
}

fn default_happiness() -> HappinessConfig {
    HappinessConfig {
        base: 50,
        min_for_victory: 60,
        leave_threshold: 15,
        leave_chance_percent: 35,
        unhappy_threshold: 30,
        tenure_bonus_max: 12,
        rent_bonus_multiplier: 0.02,
        rent_bonus_cap: 15,
        rent_penalty_multiplier: 0.05,
        rent_penalty_cap: -30,
        condition_bonus_multiplier: 0.3,
        condition_bonus_cap: 20,
        condition_penalty_multiplier: 1.0,
        condition_penalty_cap: 55,
        noise_quiet_bonus: 10.0,
        noise_high_penalty_base: -25,
        noise_tolerance_multiplier: 0.3,
        design_preferred_bonus: 20,
        design_hated_penalty: -25,
        design_style_modifiers: default_design_style_modifiers(),
        hallway_condition_base: 50,
        hallway_condition_multiplier: 0.2,
    }
}

fn default_design_style_modifiers() -> HashMap<String, i32> {
    let mut modifiers = HashMap::new();
    modifiers.insert("Bare".to_string(), -5);
    modifiers.insert("Practical".to_string(), 5);
    modifiers.insert("Cozy".to_string(), 10);
    modifiers.insert("Luxury".to_string(), 15);
    modifiers.insert("Opulent".to_string(), 20);
    modifiers
}

fn default_upgrade_labels() -> HashMap<String, String> {
    let mut labels = HashMap::new();
    labels.insert("repair_fmt".to_string(), "Repair +{}".to_string());
    labels.insert(
        "repair_hallway_fmt".to_string(),
        "Repair Hallway +{}".to_string(),
    );
    labels.insert("soundproofing".to_string(), "Add Soundproofing".to_string());
    labels.insert(
        "kitchen_renovation".to_string(),
        "Renovate Kitchen".to_string(),
    );
    labels.insert("install_laundry".to_string(), "Install Laundry".to_string());
    labels
}

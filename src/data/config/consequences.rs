//! Tuning for the systems that push back on the player: gentrification,
//! inspections, aging-building failures, and the passive portfolio.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GentrificationConfig {
    pub affordable_threshold: i32,
    pub rent_increase_threshold_percent: i32,
    pub rent_increase_score_divisor: i32,
    pub max_gentrification_score: i32,
    pub council_formation_threshold: f32,
    pub council_min_tenants: usize,
    /// Fraction the building's rent multiplier is rolled back when a tenant
    /// council forms (collective bargaining pushes back on rent hikes).
    #[serde(default = "default_council_rent_rollback")]
    pub council_rent_rollback: f32,
    /// Happiness the tenants gain from the solidarity of organizing.
    #[serde(default = "default_council_solidarity_happiness")]
    pub council_solidarity_happiness: i32,
    /// Max fractional bonus to a condo's sale price when the neighborhood is
    /// fully gentrified (scales with gentrification 0→100). Combined with the
    /// city's economy health, this makes *selling into a boom* a real timing
    /// decision rather than a flat, purposeless payout.
    #[serde(default = "default_condo_sale_boom_bonus")]
    pub condo_sale_boom_bonus: f32,
    /// Share of market value realized as cash after debt payoff, closing costs,
    /// and conversion fees. The market multiplier is applied on top of this.
    #[serde(default = "default_condo_sale_equity_share")]
    pub condo_sale_equity_share: f32,
}

fn default_council_rent_rollback() -> f32 {
    0.1
}

fn default_council_solidarity_happiness() -> i32 {
    5
}

fn default_condo_sale_boom_bonus() -> f32 {
    0.5
}

fn default_condo_sale_equity_share() -> f32 {
    0.25
}

impl Default for GentrificationConfig {
    fn default() -> Self {
        Self {
            affordable_threshold: 700,
            rent_increase_threshold_percent: 10,
            rent_increase_score_divisor: 5,
            max_gentrification_score: 100,
            council_formation_threshold: 0.4,
            council_min_tenants: 4,
            council_rent_rollback: default_council_rent_rollback(),
            council_solidarity_happiness: default_council_solidarity_happiness(),
            condo_sale_boom_bonus: default_condo_sale_boom_bonus(),
            condo_sale_equity_share: default_condo_sale_equity_share(),
        }
    }
}

/// Tuning for the building-inspection / code-compliance system.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegulationsConfig {
    /// Inspection score (min of average unit condition and hallway condition)
    /// at/above which a regulation passes. Below it, the regulation is cited.
    pub pass_condition_threshold: i32,
    /// Percent chance per month of an unscheduled spot-check inspection.
    pub random_inspection_chance_percent: i32,
    /// Multiplier applied to a regulation's base fine per citation.
    pub fine_multiplier: f32,
    /// Months granted to remedy a cited regulation before the fine escalates.
    pub fix_deadline_months: u32,
    /// Compliance reputation lost per citation.
    pub compliance_penalty_per_violation: i32,
    /// Compliance reputation regained after a fully clean inspection.
    pub compliance_gain_on_pass: i32,
    /// Visible neighborhood reputation lost when an inspection turns up citations.
    pub neighborhood_reputation_penalty: i32,
    /// Visible neighborhood reputation gained on a fully clean inspection.
    pub neighborhood_reputation_gain: i32,
}

impl Default for RegulationsConfig {
    fn default() -> Self {
        Self {
            pass_condition_threshold: 45,
            random_inspection_chance_percent: 8,
            fine_multiplier: 1.0,
            fix_deadline_months: 3,
            compliance_penalty_per_violation: 10,
            compliance_gain_on_pass: 5,
            neighborhood_reputation_penalty: 4,
            neighborhood_reputation_gain: 1,
        }
    }
}

/// Tuning for critical building failures (boiler, structural). Probability and
/// cost rise as the building ages, so the late game stops being a hands-off
/// victory lap and keeps demanding maintenance spend and reserves.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CriticalFailureConfig {
    /// Base monthly probability (out of 1000) of each failure type in year one.
    pub base_probability_per_1000: i32,
    /// Added to that probability for each full year the building has aged.
    pub aging_probability_per_year: i32,
    /// Base repair cost of a boiler failure.
    pub boiler_repair_cost: i32,
    /// Base repair cost of a structural failure.
    pub structural_repair_cost: i32,
    /// Extra repair cost added per full year of aging (applied to both types).
    pub aging_cost_per_year: i32,
    /// Percent of an emergency repair bill paid by insurance.
    #[serde(default = "default_insurance_cost_reduction_percent")]
    pub insurance_cost_reduction_percent: i32,
}

fn default_insurance_cost_reduction_percent() -> i32 {
    50
}

impl Default for CriticalFailureConfig {
    fn default() -> Self {
        Self {
            base_probability_per_1000: 5,
            aging_probability_per_year: 5,
            boiler_repair_cost: 1500,
            structural_repair_cost: 2500,
            aging_cost_per_year: 350,
            insurance_cost_reduction_percent: default_insurance_cost_reduction_percent(),
        }
    }
}

/// Portfolio-lite tuning: buildings you own but aren't actively managing run
/// themselves at a simplified steady state and contribute passive net income,
/// so a growing portfolio feels alive without fully simulating every building.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioConfig {
    /// Assumed stabilized occupancy of a hands-off (non-active) building.
    pub passive_occupancy: f32,
    /// Monthly overhead per unit charged against a non-active building (higher
    /// than the active building's — you're not there to run it tightly).
    pub passive_cost_per_unit: i32,
}

impl Default for PortfolioConfig {
    fn default() -> Self {
        Self {
            passive_occupancy: 0.8,
            passive_cost_per_unit: 190,
        }
    }
}

#[cfg(test)]
mod tests;

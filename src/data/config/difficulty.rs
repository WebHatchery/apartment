//! Per-difficulty rule modifiers and the one-time application of a tier's
//! rules when a game is created from a building template.

use serde::{Deserialize, Serialize};

use super::GameConfig;

/// Rule adjustments a difficulty tier applies at game start. These turn the
/// three property tiers from "same game, more units" into genuinely different
/// experiences — harder tiers start you with less cash and face stricter
/// inspections, more problem tenants, and higher overhead.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DifficultyModifiers {
    /// Starting cash for a game on this tier.
    pub starting_funds: i32,
    /// Multiplier applied to `regulations.fine_multiplier`.
    pub inspection_fine_multiplier: f32,
    /// Overrides `regulations.random_inspection_chance_percent`.
    pub random_inspection_chance_percent: i32,
    /// Overrides `tenant_risk.problem_applicant_chance_percent`.
    pub problem_applicant_chance_percent: i32,
    /// Multiplier applied to `operating_costs.base_monthly_cost_per_unit`.
    pub operating_cost_multiplier: f32,
}

impl GameConfig {
    /// Apply the modifiers for `difficulty` (case-insensitive) in place and
    /// return the tier's starting funds (falling back to 5000 if the tier is
    /// not configured). Called once when a game is created from a template.
    pub fn apply_difficulty(&mut self, difficulty: &str) -> i32 {
        let key = difficulty.to_lowercase();
        let Some(modifiers) = self
            .difficulty
            .iter()
            .find(|(name, _)| name.to_lowercase() == key)
            .map(|(_, m)| m.clone())
        else {
            return 5000;
        };

        self.regulations.fine_multiplier *= modifiers.inspection_fine_multiplier;
        self.regulations.random_inspection_chance_percent =
            modifiers.random_inspection_chance_percent;
        self.tenant_risk.problem_applicant_chance_percent =
            modifiers.problem_applicant_chance_percent;
        self.operating_costs.base_monthly_cost_per_unit =
            (self.operating_costs.base_monthly_cost_per_unit as f32
                * modifiers.operating_cost_multiplier) as i32;

        modifiers.starting_funds
    }
}

#[cfg(test)]
mod tests;

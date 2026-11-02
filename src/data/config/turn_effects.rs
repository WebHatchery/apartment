//! Tuning for the monthly turn's world events, emergency fallout, social
//! bonuses, complaint cutoffs, and the shared ranges for rent multipliers and
//! neighborhood rent demand.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TurnEffectsConfig {
    /// Monthly chance (percent) of a heatwave starting.
    pub heatwave_chance_percent: i32,
    pub heatwave_duration_months: u32,
    /// Happiness lost by each tenant of the active building per heatwave month.
    pub heatwave_happiness_penalty: i32,
    /// Monthly chance (percent) of a pipe bursting in one random unit.
    pub pipe_burst_chance_percent: i32,
    pub pipe_burst_damage: i32,
    /// Monthly chance (out of 1000) of a gentrification wave starting.
    pub gentrification_chance_per_1000: i32,
    pub gentrification_duration_months: u32,
    /// Added to the building's gentrification score per active month.
    pub gentrification_score_per_month: i32,
    /// Added to every neighborhood's rent demand per active month.
    pub gentrification_rent_demand_per_month: f32,
    /// Building appeal below which a random spot inspection fails.
    pub spot_inspection_appeal_threshold: i32,
    /// Monthly spot-inspection chance (percent) for a building below the threshold.
    pub spot_inspection_low_appeal_chance_percent: i32,
    /// Monthly spot-inspection chance (percent) otherwise.
    pub spot_inspection_chance_percent: i32,
    pub spot_inspection_fine: i32,
    /// Happiness each tenant loses when a boiler failure cannot be paid for.
    pub unpaid_boiler_happiness_penalty: i32,
    /// Hallway condition lost when structural repairs cannot be paid for.
    pub unpaid_structural_hallway_damage: i32,
    /// Building cohesion above which tenants gain the monthly social bonus.
    pub cohesion_bonus_threshold: i32,
    pub cohesion_happiness_bonus: i32,
    /// Noise happiness factor below which a noise complaint is raised.
    pub noise_complaint_factor: i32,
    /// Condition happiness factor below which a condition complaint is raised.
    pub condition_complaint_factor: i32,
    pub rent_multiplier_min: f32,
    pub rent_multiplier_max: f32,
    pub rent_demand_min: f32,
    pub rent_demand_max: f32,
}

impl Default for TurnEffectsConfig {
    fn default() -> Self {
        Self {
            heatwave_chance_percent: 2,
            heatwave_duration_months: 3,
            heatwave_happiness_penalty: 3,
            pipe_burst_chance_percent: 3,
            pipe_burst_damage: 30,
            gentrification_chance_per_1000: 5,
            gentrification_duration_months: 6,
            gentrification_score_per_month: 1,
            gentrification_rent_demand_per_month: 0.02,
            spot_inspection_appeal_threshold: 40,
            spot_inspection_low_appeal_chance_percent: 5,
            spot_inspection_chance_percent: 1,
            spot_inspection_fine: 500,
            unpaid_boiler_happiness_penalty: 30,
            unpaid_structural_hallway_damage: 20,
            cohesion_bonus_threshold: 20,
            cohesion_happiness_bonus: 1,
            noise_complaint_factor: -10,
            condition_complaint_factor: -15,
            rent_multiplier_min: 0.5,
            rent_multiplier_max: 2.0,
            rent_demand_min: 0.5,
            rent_demand_max: 2.0,
        }
    }
}

impl TurnEffectsConfig {
    /// Reject values that would make the turn's clamps or chances meaningless.
    pub fn validate(&self) -> Result<(), String> {
        let percents = [
            ("heatwave_chance_percent", self.heatwave_chance_percent),
            ("pipe_burst_chance_percent", self.pipe_burst_chance_percent),
            (
                "spot_inspection_low_appeal_chance_percent",
                self.spot_inspection_low_appeal_chance_percent,
            ),
            (
                "spot_inspection_chance_percent",
                self.spot_inspection_chance_percent,
            ),
        ];
        for (name, value) in percents {
            if !(0..=100).contains(&value) {
                return Err(format!("turn_effects.{name} must be 0-100, got {value}"));
            }
        }
        if !(0..=1000).contains(&self.gentrification_chance_per_1000) {
            return Err(format!(
                "turn_effects.gentrification_chance_per_1000 must be 0-1000, got {}",
                self.gentrification_chance_per_1000
            ));
        }
        if !(0.0 < self.rent_multiplier_min && self.rent_multiplier_min <= self.rent_multiplier_max)
        {
            return Err("turn_effects rent multiplier range must satisfy 0 < min <= max".into());
        }
        if !(0.0 < self.rent_demand_min && self.rent_demand_min <= self.rent_demand_max) {
            return Err("turn_effects rent demand range must satisfy 0 < min <= max".into());
        }
        Ok(())
    }

    pub fn clamp_rent_multiplier(&self, value: f32) -> f32 {
        value.clamp(self.rent_multiplier_min, self.rent_multiplier_max)
    }

    pub fn clamp_rent_demand(&self, value: f32) -> f32 {
        value.clamp(self.rent_demand_min, self.rent_demand_max)
    }
}

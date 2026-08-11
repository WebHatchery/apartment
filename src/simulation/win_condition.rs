use crate::building::Building;
use crate::data::config::{HappinessConfig, ThresholdsConfig, WinConditions};
use crate::economy::{PlayerFunds, TransactionType};
use crate::tenant::Tenant;
use serde::{Deserialize, Serialize};

/// Game outcome
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameOutcome {
    Victory {
        score: i32,
        months: u32,
        total_income: i32,
    },
    Bankruptcy {
        debt: i32,
    },
    AllTenantsLeft,
}

/// Check current game state for win/lose conditions
pub fn check_win_condition(
    building_id: u32,
    building: &Building,
    tenants: &[Tenant],
    funds: &PlayerFunds,
    current_tick: u32,
    has_ever_had_tenant: bool,
    win_conditions: &WinConditions,
    happiness_config: &HappinessConfig,
    thresholds: &ThresholdsConfig,
) -> Option<GameOutcome> {
    let active_tenants: Vec<&Tenant> = tenants
        .iter()
        .filter(|tenant| tenant.building_id == building_id)
        .collect();

    // Check for bankruptcy
    if funds.is_bankrupt() {
        return Some(GameOutcome::Bankruptcy {
            debt: funds.balance.abs(),
        });
    }

    // Check if all tenants left (only after the building was actually occupied at
    // some point — otherwise a brand-new empty building would instantly "lose").
    if has_ever_had_tenant
        && active_tenants.is_empty()
        && current_tick > thresholds.all_left_check_tick
    {
        return Some(GameOutcome::AllTenantsLeft);
    }

    // Check for game end (3 years = 36 months)
    let game_duration = win_conditions.game_duration_ticks.unwrap_or(36);
    if current_tick >= game_duration {
        // Calculate final score based on performance
        let avg_happiness: i32 = if active_tenants.is_empty() {
            0
        } else {
            active_tenants
                .iter()
                .map(|tenant| tenant.happiness)
                .sum::<i32>()
                / active_tenants.len() as i32
        };

        // Full occupancy is only required to earn the bonus when the config
        // says so; otherwise it's awarded unconditionally.
        let occupancy_bonus =
            if !win_conditions.full_occupancy_required || building.has_full_rental_occupancy() {
                100
            } else {
                0
            };
        // Reward clearing the configured happiness bar, on top of the
        // continuous happiness contribution below.
        let happiness_bonus = if avg_happiness >= happiness_config.min_for_victory {
            50
        } else {
            0
        };
        // A defensive floor: a run that (via a very low game_duration_ticks)
        // ends before min_ticks_for_victory still gets an outcome, just
        // without this small "played it out" bonus.
        let maturity_bonus = if current_tick >= win_conditions.min_ticks_for_victory {
            20
        } else {
            0
        };
        let tenant_count_bonus = (active_tenants.len() as i32) * 10;

        // Only earned operating income contributes to performance. Grants and
        // asset sales are useful liquidity, but counting them as income made a
        // condo conversion or mission cheque an immediate score exploit.
        let earned_rent: i32 = funds
            .transactions
            .iter()
            .filter(|transaction| transaction.transaction_type == TransactionType::RentIncome)
            .map(|transaction| transaction.amount.max(0))
            .sum();

        // Stewardship is the centre of the score: rent still matters, but it
        // cannot overwhelm tenant wellbeing and the state of the property.
        let score = (avg_happiness * 8)  // Tenant wellbeing contribution
            + (building.average_condition() * 5) // Building stewardship
            + (earned_rent / 400)         // Earned rent contribution
            + occupancy_bonus             // Full building bonus
            + happiness_bonus             // Cleared the victory happiness bar
            + maturity_bonus              // Played out at least the minimum duration
            + tenant_count_bonus; // Tenant retention bonus

        return Some(GameOutcome::Victory {
            total_income: funds.total_income,
            months: current_tick,
            score,
        });
    }

    None
}

#[cfg(test)]
mod tests;

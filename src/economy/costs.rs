use super::{PlayerFunds, Transaction, TransactionType};
use crate::building::{apply_upgrade, Building, UpgradeAction};
use crate::data::config::OperatingCostsConfig;

/// Calculate operating costs
pub struct OperatingCosts;

impl OperatingCosts {
    /// Calculate monthly property tax based on rent income.
    /// The effective rate escalates yearly as the property is reassessed.
    pub fn calculate_property_tax(
        _building: &Building,
        rent_income: i32,
        config: &OperatingCostsConfig,
        current_tick: u32,
    ) -> i32 {
        let years_owned = (current_tick / 12) as f32;
        let effective_rate =
            config.property_tax_rate + config.property_tax_annual_increase * years_owned;
        (rent_income as f32 * effective_rate) as i32
    }

    /// Fixed monthly overhead (mortgage/upkeep) charged for every unit
    /// regardless of occupancy — a structural drawdown that occupancy must beat.
    pub fn calculate_base_overhead(building: &Building, config: &OperatingCostsConfig) -> i32 {
        building.apartments.len() as i32 * config.base_monthly_cost_per_unit
    }

    /// Calculate monthly utilities
    pub fn calculate_utilities(building: &Building, config: &OperatingCostsConfig) -> i32 {
        if !building.utilities_included {
            return 0;
        }

        // Base cost per occupied unit
        let occupied = building.occupancy_count() as i32;
        occupied * config.utility_cost_per_unit
    }

    /// Calculate monthly insurance
    pub fn calculate_insurance(building: &Building, config: &OperatingCostsConfig) -> i32 {
        if !building.insurance_active {
            return 0;
        }

        // Discount for good condition
        let discount = if building.hallway_condition > config.insurance_good_condition_threshold {
            config.insurance_good_condition_discount
        } else {
            0
        };

        config.insurance_base_rate - discount
    }

    /// Calculate monthly staff salaries
    pub fn calculate_staff_salaries(
        building: &Building,
        config: &crate::data::config::EconomyConfig,
    ) -> i32 {
        let mut total = 0;

        for (staff_type, cost) in &config.staff_costs {
            let flag = format!("staff_{}", staff_type);
            if building.flags.contains(&flag) {
                total += cost;
            }
        }

        total
    }
}

/// Process an upgrade and deduct funds
/// Returns Ok(cost) if successful, Err(reason) if failed
pub fn process_upgrade(
    action: &UpgradeAction,
    building: &mut Building,
    funds: &mut PlayerFunds,
    config: &crate::data::config::GameConfig,
    current_tick: u32,
) -> Result<i32, String> {
    // Calculate cost using central logic
    let cost = action
        .cost(building, &config.economy, &config.upgrades)
        .ok_or("Invalid upgrade")?;

    // Additional Validation
    match action {
        UpgradeAction::RepairApartment { apartment_id, .. } => {
            let apt = building
                .get_apartment(*apartment_id)
                .ok_or("Apartment not found")?;
            if apt.condition >= 100 {
                return Err("Apartment already at max condition".to_string());
            }
        }
        UpgradeAction::RepairHallway { .. } => {
            if building.hallway_condition >= 100 {
                return Err("Hallway already at max condition".to_string());
            }
        }
        UpgradeAction::Apply {
            upgrade_id,
            target_id,
        } => {
            let def = config.upgrades.get(upgrade_id).ok_or("Unknown upgrade")?;
            match def.target {
                crate::data::config::UpgradeTarget::Apartment => {
                    let apt_id = target_id.ok_or("Missing apartment ID")?;
                    let apt = building
                        .get_apartment(apt_id)
                        .ok_or("Apartment not found")?;
                    if !crate::building::upgrades::check_requirements(
                        &def.requirements,
                        apt,
                        Some(building),
                    ) {
                        return Err("Upgrade requirements are no longer satisfied".to_string());
                    }
                }
                crate::data::config::UpgradeTarget::Building => {
                    if target_id.is_some() {
                        return Err("Building upgrades do not accept an apartment ID".to_string());
                    }
                    if !crate::building::upgrades::check_requirements_building(
                        &def.requirements,
                        building,
                    ) {
                        return Err("Upgrade requirements are no longer satisfied".to_string());
                    }
                }
            }
        }
    };

    // Check funds
    if !funds.can_afford(cost) {
        return Err(format!(
            "Insufficient funds (need ${}, have ${})",
            cost, funds.balance
        ));
    }

    // Create transaction description
    let description = match action {
        UpgradeAction::RepairApartment {
            apartment_id,
            amount,
        } => {
            let unit = building
                .get_apartment(*apartment_id)
                .map(|a| a.unit_number.clone())
                .unwrap_or_default();
            format!("Repair Unit {} (+{} condition)", unit, amount)
        }
        UpgradeAction::RepairHallway { amount } => {
            format!("Hallway repair (+{} condition)", amount)
        }
        UpgradeAction::Apply {
            upgrade_id,
            target_id,
        } => {
            let name = config
                .upgrades
                .get(upgrade_id)
                .map(|u| u.name.clone())
                .unwrap_or_else(|| "Upgrade".to_string());
            if let Some(apt_id) = target_id {
                let unit = building
                    .get_apartment(*apt_id)
                    .map(|a| a.unit_number.clone())
                    .unwrap_or_default();
                format!("{} (Unit {})", name, unit)
            } else {
                name
            }
        }
    };

    let transaction = Transaction::expense(
        match action {
            UpgradeAction::RepairApartment { .. } => TransactionType::RepairCost,
            UpgradeAction::RepairHallway { .. } => TransactionType::HallwayRepair,
            UpgradeAction::Apply { .. } => TransactionType::UpgradeCost,
        },
        cost,
        &description,
        current_tick,
    );

    // Prove the state transition succeeds before charging, keeping the action
    // atomic even if authored upgrade data is invalid.
    let mut updated_building = building.clone();
    apply_upgrade(&mut updated_building, action, &config.upgrades)
        .ok_or("Failed to apply upgrade")?;
    if !funds.deduct_expense(transaction) {
        return Err("Failed to deduct funds".to_string());
    }
    *building = updated_building;

    Ok(cost)
}

#[cfg(test)]
mod tests;

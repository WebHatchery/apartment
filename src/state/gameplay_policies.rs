//! Authoritative rent, marketing, and building-policy changes.

use crate::building::MarketingType;
use crate::economy::{Transaction, TransactionType};

use super::gameplay::GameplayState;

impl GameplayState {
    pub(super) fn set_apartment_rent(
        &mut self,
        apartment_id: u32,
        requested_rent: i32,
    ) -> Result<(i32, i32), String> {
        if self.building.is_unit_sold(apartment_id) {
            return Err("Privately owned condos cannot be repriced".to_string());
        }
        let building_id = self.active_building_id();
        let apartment = self
            .building
            .get_apartment_mut(apartment_id)
            .ok_or_else(|| "Apartment not found".to_string())?;
        let old_rent = apartment.rent_price;
        let new_rent = requested_rent.max(100);
        apartment.rent_price = new_rent;

        if old_rent != new_rent {
            self.gentrification.record_rent_change(
                building_id,
                self.current_tick,
                old_rent,
                new_rent,
                &self.config.gentrification,
            );
        }
        Ok((old_rent, new_rent))
    }

    pub(super) fn set_marketing_strategy(&mut self, strategy: MarketingType) {
        self.building.marketing_strategy = strategy;
    }

    pub(super) fn start_open_house(&mut self) -> Result<i32, String> {
        if self.building.open_house_remaining > 0 {
            return Err(format!(
                "Open house already has {} month(s) remaining",
                self.building.open_house_remaining
            ));
        }
        let cost = self.config.marketing.open_house_cost;
        let transaction = Transaction::expense(
            TransactionType::Marketing,
            cost,
            "Open House",
            self.current_tick,
        );
        if !self.funds.deduct_expense(transaction) {
            return Err(format!(
                "Open house costs ${}; only ${} is available",
                cost, self.funds.balance
            ));
        }
        self.building.open_house_remaining = self.config.marketing.open_house_duration;
        Ok(cost)
    }

    pub(super) fn set_utilities_policy(&mut self, included: bool) {
        self.building.utilities_included = included;
    }

    pub(super) fn set_insurance_policy(&mut self, active: bool) {
        self.building.insurance_active = active;
    }
}

#[cfg(test)]
mod tests;

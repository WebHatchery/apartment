//! Transaction-safe condo sale and buyback operations.

use super::gameplay_turn::feedback_point;
use crate::economy::{Transaction, TransactionType};
use crate::simulation::{GameEvent, NotificationLevel};
use crate::ui::colors;
use macroquad::prelude::*;

use super::gameplay::GameplayState;

impl GameplayState {
    pub(super) fn sell_unit_as_condo(&mut self, apartment_id: u32) {
        if self.building.is_unit_sold(apartment_id) {
            return;
        }

        let Some(apartment) = self.building.get_apartment(apartment_id) else {
            return;
        };
        let tenant_id = apartment.tenant_id;
        let rent = apartment.rent_price;
        let sale_price =
            (apartment.market_value() as f32 * self.condo_sale_market_multiplier()) as i32;

        if !self
            .building
            .convert_unit_to_condo(apartment_id, "New Owner", sale_price)
        {
            return;
        }

        if let Some(tenant_id) = tenant_id {
            if let Some(index) = self.tenants.iter().position(|tenant| {
                tenant.id == tenant_id && tenant.lives_in(self.active_building_id(), apartment_id)
            }) {
                let tenant = self.tenants.remove(index);
                let neighborhood_name = self
                    .city
                    .neighborhood_for_building(self.city.active_building_index)
                    .map(|neighborhood| neighborhood.name.as_str())
                    .unwrap_or("Unknown neighborhood");
                self.gentrification.record_unit_conversion(
                    &tenant,
                    rent,
                    self.current_tick,
                    &self.building.name,
                    neighborhood_name,
                    &self.config.gentrification,
                );
                self.tenant_stories.remove(&tenant_id);
            }
        }

        // A sold unit no longer contains one of the player's rental tenants and
        // must not retain a stale lease/listing in the apartment record.
        if let Some(apartment) = self.building.get_apartment_mut(apartment_id) {
            apartment.move_out();
            apartment.is_listed_for_lease = false;
            apartment.preferred_archetype = None;
        }
        let building_id = self.active_building_id();
        self.applications.retain(|application| {
            application.building_id != building_id || application.apartment_id != apartment_id
        });

        self.funds.add_income(Transaction::income(
            TransactionType::AssetSale,
            sale_price,
            "Condo Sale",
            self.current_tick,
        ));
        self.floating_texts.spawn(
            format!("+${}", sale_price),
            feedback_point(),
            colors::POSITIVE(),
        );
        self.save_building_to_city();
    }

    pub(super) fn buy_back_condo(&mut self, apartment_id: u32) {
        let Some(buyback_cost) = self.building.condo_buyback_price(apartment_id) else {
            return;
        };
        if !self.funds.can_afford(buyback_cost) {
            self.event_log.log(
                GameEvent::Notification {
                    message: format!(
                        "Condo buyback requires ${}; only ${} is available.",
                        buyback_cost, self.funds.balance
                    ),
                    level: NotificationLevel::Warning,
                },
                self.current_tick,
            );
            self.floating_texts.spawn(
                "Cannot Afford Buyback",
                feedback_point(),
                colors::NEGATIVE(),
            );
            return;
        }

        if !self.building.complete_condo_buyback(apartment_id) {
            return;
        }
        let transaction = Transaction::expense(
            TransactionType::BuildingPurchase,
            buyback_cost,
            "Condo Buyback",
            self.current_tick,
        );
        if !self.funds.deduct_expense(transaction) {
            // The affordability check above makes this unreachable without an
            // intervening mutation. Keep the operation atomic if that changes.
            return;
        }

        if let Some(apartment) = self.building.get_apartment_mut(apartment_id) {
            apartment.move_out();
            apartment.is_listed_for_lease = false;
            apartment.preferred_archetype = None;
        }
        self.floating_texts.spawn(
            format!("-${}", buyback_cost),
            feedback_point(),
            colors::NEGATIVE(),
        );
        self.floating_texts.spawn(
            "Unit Repurchased!",
            vec2(screen_width() / 2.0, screen_height() / 2.0 + 30.0),
            colors::POSITIVE(),
        );
        self.save_building_to_city();
    }
}

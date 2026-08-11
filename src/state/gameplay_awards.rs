// Periodic payoffs and reckonings: mission tax breaks each month, and the
// annual awards / tenant-council check.

use crate::economy::{Transaction, TransactionType};
use crate::simulation::GameEvent;
use crate::ui::colors;

use super::gameplay::GameplayState;

impl GameplayState {
    pub(super) fn apply_active_tax_breaks(&mut self) {
        let refund = self.process_active_tax_breaks();
        if refund > 0 {
            self.spawn_center_text(
                &format!("Tax Break +${}", refund),
                0.0,
                60.0,
                colors::POSITIVE(),
            );
        }
    }

    fn process_active_tax_breaks(&mut self) -> i32 {
        if self.active_tax_breaks.is_empty() {
            return 0;
        }

        let percentage = self
            .active_tax_breaks
            .iter()
            .map(|tax_break| tax_break.percentage)
            .sum::<f32>()
            .clamp(0.0, 0.75);
        let tax_paid = self.current_tick_property_tax();
        let refund = (tax_paid as f32 * percentage).round() as i32;

        if refund > 0 {
            self.funds.add_income(Transaction::income(
                TransactionType::Grant,
                refund,
                "Mission Tax Break Refund",
                self.current_tick,
            ));
        }

        for tax_break in &mut self.active_tax_breaks {
            tax_break.remaining_months = tax_break.remaining_months.saturating_sub(1);
        }
        self.active_tax_breaks
            .retain(|tax_break| tax_break.remaining_months > 0 && tax_break.percentage > 0.0);

        refund
    }

    fn current_tick_property_tax(&self) -> i32 {
        self.funds
            .transactions_for_tick(self.current_tick)
            .iter()
            .filter(|transaction| {
                transaction.transaction_type == TransactionType::PropertyTax
                    && transaction.amount < 0
            })
            .map(|transaction| transaction.amount.abs())
            .sum()
    }

    pub(super) fn check_annual_awards(&mut self) {
        let active_tenants = self.active_tenants_cloned();
        let avg_happiness = if active_tenants.is_empty() {
            0.0
        } else {
            active_tenants
                .iter()
                .map(|tenant| tenant.happiness as f32)
                .sum::<f32>()
                / active_tenants.len() as f32
        };
        let total = self.building.rental_unit_count();
        let occupied = self.building.occupancy_count();
        let occupancy_rate = if total > 0 {
            occupied as f32 / total as f32
        } else {
            0.0
        };

        self.missions.check_for_awards(
            self.current_tick,
            &self.building.name,
            avg_happiness,
            occupancy_rate,
            active_tenants.len() as u32,
        );

        let forming = self.tenant_network.should_form_council(
            &active_tenants,
            &self.config.gentrification,
            self.config.happiness.unhappy_threshold,
        );

        if forming && !self.council_formed {
            self.council_formed = true;
            self.apply_council_collective_action();
        } else if !forming {
            // Conditions improved; the council disbands and could re-form later.
            self.council_formed = false;
        }
    }

    /// A newly-formed tenant council's one-time collective action: it bargains
    /// the building's rent multiplier down and the solidarity of organizing
    /// lifts tenant morale. This is the mechanical consequence of pushing rent
    /// too hard on an unhappy building — previously the council was cosmetic.
    fn apply_council_collective_action(&mut self) {
        let rollback = self.config.gentrification.council_rent_rollback;
        self.building.rent_multiplier =
            (self.building.rent_multiplier * (1.0 - rollback)).clamp(0.5, 2.0);

        let bump = self.config.gentrification.council_solidarity_happiness;
        let building_id = self.active_building_id();
        for tenant in &mut self.tenants {
            if tenant.building_id == building_id {
                tenant.happiness = (tenant.happiness + bump).clamp(0, 100);
            }
        }

        self.spawn_center_text(
            "Tenants formed a council — rent rolled back!",
            0.0,
            30.0,
            colors::ACCENT(),
        );
        self.event_log.log(
            GameEvent::Notification {
                message: "A tenant council organized and bargained rent down.".to_string(),
                level: crate::simulation::NotificationLevel::Warning,
            },
            self.current_tick,
        );
        self.missions.record_legacy_event(
            self.current_tick,
            "Tenant Council Formed",
            "Tenants organized a council and won a rent rollback.",
        );
    }
}

#[cfg(test)]
mod tests;

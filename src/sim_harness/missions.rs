//! Mission and tax-break integration for representative headless runs.

use super::Sim;
use crate::economy::{Transaction, TransactionType};
use crate::narrative::{ActiveTaxBreak, MissionReward};
use crate::simulation::{GameEvent, TickResult};
use std::collections::HashMap;

impl Sim {
    pub(super) fn prepare_missions(&mut self) {
        self.missions.generate_available_missions(self.current_tick);
        let available: Vec<u32> = self
            .missions
            .available_missions()
            .iter()
            .map(|mission| mission.id)
            .collect();
        for mission_id in available {
            self.missions.accept_mission(mission_id, self.current_tick);
        }
    }

    pub(super) fn application_reputation_multiplier(&self) -> f32 {
        let influence = self.config.applications.reputation_influence;
        (1.0 + (self.reputation - 50) as f32 / 50.0 * influence).clamp(0.25, 2.0)
    }

    pub(super) fn apply_active_tax_breaks(&mut self) {
        if self.active_tax_breaks.is_empty() {
            return;
        }
        let percentage = self
            .active_tax_breaks
            .iter()
            .map(|tax_break| tax_break.percentage)
            .sum::<f32>()
            .clamp(0.0, 0.75);
        let tax_paid: i32 = self
            .funds
            .transactions_for_tick(self.current_tick)
            .iter()
            .filter(|transaction| {
                transaction.transaction_type == TransactionType::PropertyTax
                    && transaction.amount < 0
            })
            .map(|transaction| transaction.amount.abs())
            .sum();
        let refund = (tax_paid as f32 * percentage).round() as i32;
        if refund > 0 {
            self.funds.add_income(Transaction::income(
                TransactionType::Grant,
                refund,
                "Mission Tax Break Refund",
                self.current_tick,
            ));
            self.mission_cash += refund;
        }
        for tax_break in &mut self.active_tax_breaks {
            tax_break.remaining_months = tax_break.remaining_months.saturating_sub(1);
        }
        self.active_tax_breaks
            .retain(|tax_break| tax_break.remaining_months > 0 && tax_break.percentage > 0.0);
    }

    pub(super) fn update_missions(&mut self, result: &TickResult) {
        let mut repaired = HashMap::new();
        repaired.insert(
            0,
            !self.building.apartments.is_empty()
                && self
                    .building
                    .apartments
                    .iter()
                    .all(|apartment| apartment.condition >= 90)
                && self.building.hallway_condition >= 90,
        );
        let perfect_collection = !self.tenants.is_empty()
            && !result
                .events
                .iter()
                .any(|event| matches!(event, GameEvent::RentMissed { .. }));
        let occupancy = self.occupancy();
        let avg_happiness = self.avg_happiness() as f32;
        let building_count = self.city.buildings.len();
        let completed = self
            .missions
            .evaluate_active(&crate::narrative::MissionSnapshot {
                current_month: self.current_tick,
                active_building_id: 0,
                active_tenants: &self.tenants,
                occupancy,
                avg_happiness,
                perfect_collection,
                fully_repaired_by_building: &repaired,
                building_count,
            });
        for mission in completed {
            self.missions_completed += 1;
            match mission.reward {
                MissionReward::Money(amount) => {
                    self.funds.add_income(Transaction::income(
                        TransactionType::Grant,
                        amount,
                        "Mission Reward",
                        self.current_tick,
                    ));
                    self.mission_cash += amount;
                }
                MissionReward::TaxBreak { months, percentage } => self
                    .active_tax_breaks
                    .push(ActiveTaxBreak::new(months, percentage)),
                MissionReward::Reputation(amount) => {
                    self.reputation = (self.reputation + amount).clamp(0, 100);
                }
                MissionReward::UnlockBuilding(_) => {}
            }
        }
    }
}

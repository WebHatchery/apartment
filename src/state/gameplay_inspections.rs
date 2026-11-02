//! Building inspections and the regulatory fines they produce.

use crate::economy::{Transaction, TransactionType};
use crate::simulation::GameEvent;
use crate::ui::colors;
use macroquad::prelude::*;

use super::gameplay::GameplayState;

impl GameplayState {
    /// Run any scheduled or random building inspections for the active building,
    /// bill the resulting fines, and reflect the outcome in reputation. A
    /// well-maintained building (condition at/above the configured threshold)
    /// passes cleanly; a neglected one gets cited and fined — the economic teeth
    /// that make repairs and upgrades matter.
    pub(super) fn run_due_inspections(&mut self) {
        let building_id = self.city.active_building_index as u32;
        let due = self.compliance.has_due_inspection(building_id);
        let random_check = macroquad_toolkit::rng::gen_range(0, 100)
            < self.config.regulations.random_inspection_chance_percent;

        if due || random_check {
            let trigger = if due {
                crate::consequences::InspectionTrigger::Scheduled
            } else {
                crate::consequences::InspectionTrigger::Random
            };
            self.execute_inspection(trigger);
        }

        self.bill_outstanding_fines();
    }

    /// Run a single inspection of the active building with the given trigger,
    /// grade it against current condition, and surface the outcome (reputation
    /// move, event-log entry, floating text). Fines accrue to
    /// `compliance.unpaid_fines`; call `bill_outstanding_fines` to charge them.
    pub(super) fn execute_inspection(&mut self, trigger: crate::consequences::InspectionTrigger) {
        let building_id = self.city.active_building_index as u32;
        self.execute_inspection_for(building_id, trigger);
    }

    pub(super) fn execute_inspection_for(
        &mut self,
        building_id: u32,
        trigger: crate::consequences::InspectionTrigger,
    ) {
        let building = if building_id == self.active_building_id() {
            Some(&self.building)
        } else {
            self.city.buildings.get(building_id as usize)
        };
        let Some(building) = building else {
            return;
        };
        let inspection_score = building.average_condition().min(building.hallway_condition);
        let config = self.config.regulations.clone();

        let inspection = self.compliance.run_inspection(
            building_id,
            inspection_score,
            self.current_tick,
            trigger,
            &config,
        );

        let citations = inspection.results.iter().filter(|r| !r.passed).count();
        if citations > 0 {
            self.adjust_neighborhood_reputation_for_building(
                building_id,
                -config.neighborhood_reputation_penalty,
            );
            self.event_log.log(
                GameEvent::Notification {
                    message: format!(
                        "Inspection failed: {} citation(s), {} in fines.",
                        citations, inspection.total_fines
                    ),
                    level: crate::simulation::NotificationLevel::Warning,
                },
                self.current_tick,
            );
            self.floating_texts.spawn(
                format!("Inspection: {} cited!", citations),
                vec2(screen_width() / 2.0, screen_height() / 2.0),
                colors::NEGATIVE(),
            );
        } else if !inspection.results.is_empty() {
            self.adjust_neighborhood_reputation_for_building(
                building_id,
                config.neighborhood_reputation_gain,
            );
            self.floating_texts.spawn(
                "Inspection passed",
                vec2(screen_width() / 2.0, screen_height() / 2.0),
                colors::POSITIVE(),
            );
        }
    }

    /// Charge any outstanding regulatory fines (from inspections or missed fix
    /// deadlines escalated in `ComplianceSystem::tick`) as a required expense, so
    /// persistent neglect can genuinely push a landlord toward bankruptcy.
    pub(super) fn bill_outstanding_fines(&mut self) {
        if self.compliance.unpaid_fines > 0 {
            let amount = self.compliance.unpaid_fines;
            self.funds.apply_required_expense(Transaction::expense(
                TransactionType::InspectionFine,
                amount,
                "Regulatory fines",
                self.current_tick,
            ));
            self.compliance.unpaid_fines = 0;
        }
    }
}

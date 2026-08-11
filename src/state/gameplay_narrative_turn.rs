// The narrative half of the monthly turn: events, mail, dialogue, missions,
// and tenant requests.

use macroquad_toolkit::rng;

use crate::simulation::{GameEvent, NotificationLevel, TickResult};

use super::gameplay::GameplayState;

impl GameplayState {
    pub(super) fn generate_monthly_narrative(&mut self, result: &TickResult) {
        self.narrative_events.generate_events(
            self.current_tick,
            &self.city.neighborhoods,
            &self.city.buildings,
            &self.tenants,
        );
        for (headline, effect) in self.narrative_events.process_immediate_events() {
            self.apply_narrative_effect(&effect);
            self.event_log.log(
                GameEvent::Notification {
                    message: headline,
                    level: NotificationLevel::Info,
                },
                self.current_tick,
            );
        }

        let expenses = self
            .funds
            .transactions_for_tick(self.current_tick)
            .iter()
            .filter(|transaction| transaction.amount < 0)
            .map(|transaction| transaction.amount.abs())
            .sum();
        self.mailbox.generate_mail(
            self.current_tick,
            result.rent_collected,
            expenses,
            &self.tenants,
            &self.city.buildings,
        );
        self.mailbox.cleanup(self.current_tick, 12);

        self.generate_dialogues();
        self.generate_tenant_requests();
    }

    fn generate_dialogues(&mut self) {
        let tenants = self.active_tenants_cloned();
        let building = self.building.clone();
        let funds = self.funds.clone();
        self.dialogue_system.generate_dialogues(
            self.current_tick,
            &tenants,
            &building,
            &funds,
            &self.tenant_network,
        );
    }

    /// With a manager employed, routine tenant requests are handled for you
    /// (approved) instead of piling up as manual to-dos — the manager's job.
    pub(super) fn auto_approve_manager_requests(&mut self) {
        if !self.config.staff_effects.manager_auto_approve_requests
            || !self.building.flags.contains("staff_manager")
        {
            return;
        }

        let active_ids: std::collections::HashSet<u32> = self
            .active_tenants_cloned()
            .into_iter()
            .map(|tenant| tenant.id)
            .collect();
        let tenant_ids: Vec<u32> = self
            .tenant_stories
            .iter()
            .filter(|(tenant_id, story)| {
                active_ids.contains(tenant_id) && story.pending_request.is_some()
            })
            .map(|(id, _)| *id)
            .collect();

        for tenant_id in tenant_ids {
            let effect = self.tenant_stories.get_mut(&tenant_id).and_then(|story| {
                story.pending_request.take().map(|request| {
                    let effect = request.approval_effect();
                    story.add_event(
                        self.current_tick,
                        "Request approved by property manager",
                        effect.clone(),
                    );
                    effect
                })
            });

            if let Some(effect) = effect {
                self.apply_story_impact(tenant_id, effect);
            }
        }
    }

    fn generate_tenant_requests(&mut self) {
        let building_id = self.active_building_id();
        for tenant in &self.tenants {
            if tenant.building_id != building_id {
                continue;
            }
            if let Some(story) = self.tenant_stories.get_mut(&tenant.id) {
                if rng::gen_range(0, 100) < 10 {
                    story.make_request(&tenant.archetype, &self.tenant_events_config);
                }
            }
        }
    }

    pub(super) fn expire_narrative_events(&mut self) {
        let expired_effects = self.narrative_events.expire_due_events(self.current_tick);
        for effect in expired_effects {
            self.apply_narrative_effect(&effect);
        }
    }
}

#[cfg(test)]
mod tests;

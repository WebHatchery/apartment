//! Monthly turn advancement for gameplay state. The narrative, inspection,
//! neighborhood, and awards halves of the turn live in sibling modules.

use crate::economy::{Transaction, TransactionType};
use crate::simulation::{advance_tick, ActiveWorldEvent, ActiveWorldEventKind, GameEvent};
use crate::ui::colors;
use macroquad::prelude::*;

use super::gameplay::GameplayState;

impl GameplayState {
    /// End the current turn and advance time.
    pub fn end_turn(&mut self) {
        let building_id = self.active_building_id();
        let occupied_before: Vec<u32> = self
            .building
            .apartments
            .iter()
            .filter(|unit| !unit.is_vacant())
            .map(|unit| unit.id)
            .collect();
        // Latch once the building has ever been occupied, so the "all tenants left"
        // loss can distinguish real mass-departure from a not-yet-filled building.
        if self.active_tenant_count() > 0 {
            self.has_ever_had_tenant = true;
            self.ever_occupied_buildings.insert(building_id);
        }

        let reputation_multiplier = self.application_reputation_multiplier();

        let result = advance_tick(
            crate::simulation::TickWorld {
                building: &mut self.building,
                tenants: &mut self.tenants,
                applications: &mut self.applications,
                funds: &mut self.funds,
                event_log: &mut self.event_log,
                next_tenant_id: &mut self.next_tenant_id,
            },
            &mut self.current_tick,
            &crate::simulation::TickContext {
                building_id,
                has_ever_had_tenant: self.ever_occupied_buildings.contains(&building_id),
                reputation_multiplier,
                config: &self.config,
            },
        );

        self.game_outcome = result.outcome.clone();
        self.register_active_world_events(&result.events);
        self.apply_active_world_events();
        self.apply_active_tax_breaks();
        self.update_city_systems();
        self.collect_portfolio_passive_income();
        self.generate_monthly_narrative(&result);
        self.generate_tenant_life_events();
        self.auto_approve_manager_requests();
        self.expire_narrative_events();
        self.sync_building();
        self.missions.generate_available_missions(self.current_tick);

        if self.current_tick.is_multiple_of(12) && self.current_tick > 0 {
            self.check_annual_awards();
        }

        self.apply_monthly_social_happiness();
        self.update_context_hints();
        self.complete_building_on_victory(&result);
        // Record the tick result before evaluating missions so goals like
        // PerfectCollection can inspect this month's rent outcome.
        self.last_tick_result = Some(result);
        self.update_missions();
        self.recently_vacated = occupied_before
            .into_iter()
            .filter(|id| {
                self.building
                    .get_apartment(*id)
                    .is_some_and(|unit| unit.is_vacant())
            })
            .collect();
        self.close_month();
        self.autosave_current_game();
    }

    /// Close the ledger month after every turn step has posted its money, so
    /// the report matches the balance change across the month.
    fn close_month(&mut self) {
        let report = self.ledger.close_month(self.current_tick, &self.funds);
        self.event_log.log(
            GameEvent::MonthEnd {
                tick: report.tick,
                income: report.income(),
                expenses: report.costs(),
                balance: report.ending_balance,
            },
            self.current_tick,
        );
        self.open_month_summary(report);
    }

    pub(super) fn spawn_center_text(
        &mut self,
        text: &str,
        offset_x: f32,
        offset_y: f32,
        color: Color,
    ) {
        self.floating_texts.spawn(
            text,
            vec2(
                screen_width() / 2.0 + offset_x,
                screen_height() / 2.0 + offset_y,
            ),
            color,
        );
    }

    fn register_active_world_events(&mut self, events: &[GameEvent]) {
        for event in events {
            match event {
                GameEvent::Heatwave { tick_duration } => {
                    self.add_active_world_event(ActiveWorldEventKind::Heatwave, *tick_duration);
                }
                GameEvent::Gentrification { tick_duration, .. } => {
                    self.add_active_world_event(
                        ActiveWorldEventKind::Gentrification,
                        *tick_duration,
                    );
                }
                _ => {}
            }
        }
    }

    fn add_active_world_event(&mut self, kind: ActiveWorldEventKind, duration: u32) {
        if duration == 0 {
            return;
        }

        if let Some(existing) = self
            .active_world_events
            .iter_mut()
            .find(|event| event.kind == kind)
        {
            existing.remaining_ticks = existing.remaining_ticks.max(duration);
            return;
        }

        self.active_world_events
            .push(ActiveWorldEvent::new(kind, duration));
    }

    fn apply_active_world_events(&mut self) {
        let mut heatwave_active = false;
        let mut gentrification_active = false;

        for event in &mut self.active_world_events {
            match &event.kind {
                ActiveWorldEventKind::Heatwave => {
                    heatwave_active = true;
                }
                ActiveWorldEventKind::Gentrification => {
                    gentrification_active = true;
                }
            }
            event.tick();
        }

        let effects = &self.config.turn_effects;
        if heatwave_active {
            let building_id = self.active_building_id();
            for tenant in &mut self.tenants {
                if tenant.building_id == building_id {
                    tenant.happiness =
                        (tenant.happiness - effects.heatwave_happiness_penalty).max(0);
                }
            }
        }

        if gentrification_active {
            self.gentrification.gentrification_score = (self.gentrification.gentrification_score
                + effects.gentrification_score_per_month)
                .min(self.config.gentrification.max_gentrification_score);

            for neighborhood in &mut self.city.neighborhoods {
                neighborhood.stats.gentrification =
                    (neighborhood.stats.gentrification + 1).min(100);
                neighborhood.stats.rent_demand = effects.clamp_rent_demand(
                    neighborhood.stats.rent_demand + effects.gentrification_rent_demand_per_month,
                );
            }
        }

        self.active_world_events
            .retain(|event| event.remaining_ticks > 0);
    }

    fn update_city_systems(&mut self) {
        self.save_building_to_city();
        self.city.tick(&self.config.turn_effects);

        let active_tenants = self.active_tenants_cloned();
        let (rel_changes, rel_events) = self.tenant_network.tick(
            &active_tenants,
            &self.building,
            &self.config.relationships,
            &self.relationship_events_config,
            self.current_tick,
        );
        self.notifications.add_relationship_changes(rel_changes);
        for mut event in rel_events {
            event.month = self.current_tick;
            if event.requires_response {
                event.response_deadline = Some(self.current_tick + 2);
            }
            let requires_response = event.requires_response;
            let event_id = self.narrative_events.add_event(event);
            if !requires_response {
                let effect = self
                    .narrative_events
                    .process_default_now(event_id)
                    .unwrap_or(crate::narrative::events::NarrativeEffect::None);
                self.apply_narrative_effect(&effect);
            }
        }

        let inspection_score = self
            .building
            .average_condition()
            .min(self.building.hallway_condition);
        let active_building_id = self.active_building_id();
        self.compliance.resolve_fixes_if_compliant(
            active_building_id,
            inspection_score,
            self.config.regulations.pass_condition_threshold,
        );
        self.compliance.tick(self.current_tick);
        self.run_due_inspections();
        self.gentrification
            .update_affordable_units(&self.building, &self.config.gentrification);
    }

    /// Portfolio-lite: buildings you own but aren't actively managing run
    /// themselves at a simplified steady state and contribute passive net income
    /// each month. The active building is fully simulated by `advance_tick` and
    /// excluded here.
    pub(super) fn collect_portfolio_passive_income(&mut self) {
        let active = self.city.active_building_index;
        let cfg = &self.config.portfolio;
        let mut net = 0i32;
        let mut earning = 0u32;
        for (i, building) in self.city.buildings.iter().enumerate() {
            if i == active || building.apartments.is_empty() {
                continue;
            }
            let potential: i32 = building
                .apartments
                .iter()
                .filter(|apartment| !building.is_unit_sold(apartment.id))
                .map(|apartment| apartment.rent_price)
                .sum();
            let income = (potential as f32 * cfg.passive_occupancy) as i32;
            let cost = building.apartments.len() as i32 * cfg.passive_cost_per_unit;
            net += income - cost;
            earning += 1;
        }

        if earning == 0 || net == 0 {
            return;
        }

        if net > 0 {
            self.funds.add_income(Transaction::income(
                TransactionType::RentIncome,
                net,
                "Portfolio passive income",
                self.current_tick,
            ));
        } else {
            self.funds.apply_required_expense(Transaction::expense(
                TransactionType::Mortgage,
                net.abs(),
                "Portfolio upkeep",
                self.current_tick,
            ));
        }

        self.event_log.log(
            GameEvent::Notification {
                message: format!(
                    "Your other {} propert{} netted {:+} this month.",
                    earning,
                    if earning == 1 { "y" } else { "ies" },
                    net
                ),
                level: crate::simulation::NotificationLevel::Info,
            },
            self.current_tick,
        );
    }

    fn autosave_current_game(&mut self) {
        if !self.autosave_enabled {
            return;
        }
        if let Err(error) = crate::save::save_game(self) {
            eprintln!("Failed to save game: {}", error);
            self.spawn_center_text("Save Failed!", 0.0, 0.0, colors::NEGATIVE());
        }
    }

    fn apply_monthly_social_happiness(&mut self) {
        let building_id = self.active_building_id();
        let active_tenants = self.active_tenants_cloned();
        let cohesion = self
            .tenant_network
            .calculate_cohesion(&active_tenants, &self.config.cohesion);
        for tenant in &mut self.tenants {
            if tenant.building_id != building_id {
                continue;
            }
            let relationship_bonus = crate::tenant::happiness::calculate_relationship_happiness(
                tenant.id,
                &self.tenant_network,
                &self.config.relationships,
            );
            let effects = &self.config.turn_effects;
            let cohesion_bonus = if cohesion > effects.cohesion_bonus_threshold {
                effects.cohesion_happiness_bonus
            } else {
                0
            };
            tenant.happiness =
                (tenant.happiness + relationship_bonus + cohesion_bonus).clamp(0, 100);
        }
    }

    fn update_context_hints(&mut self) {
        let building_id = self.active_building_id();
        let total_units = self.building.rental_unit_count();
        let vacancy_count = self.building.vacancy_count();
        let avg_condition = self.building.average_condition();
        let any_unhappy = self
            .tenants
            .iter()
            .filter(|tenant| tenant.building_id == building_id)
            .any(|tenant| tenant.is_unhappy(self.config.happiness.unhappy_threshold));

        self.notifications.check_context_hints(
            self.current_tick,
            vacancy_count,
            total_units,
            avg_condition,
            self.funds.balance,
            any_unhappy,
        );
    }

    /// A completed campaign (the tick's own `Victory` outcome) records the
    /// building as finished and unlocks the next property. Outcomes reached
    /// outside the tick, such as selling every building, do not unlock. The
    /// career-summary transition and its achievement check happen in `update`.
    fn complete_building_on_victory(&mut self, result: &crate::simulation::TickResult) {
        if !matches!(
            result.outcome,
            Some(crate::simulation::GameOutcome::Victory { .. })
        ) {
            return;
        }
        if let Err(error) = self.unlock_next_building() {
            self.report_action_failure(&error);
        }
    }
}

/// Where to acknowledge a player's action: just above the point they
/// tapped, kept on screen, instead of the middle of the screen.
pub(super) fn feedback_point() -> Vec2 {
    let (x, y) = mouse_position();
    vec2(
        x.clamp(60.0, screen_width() - 60.0),
        (y - 24.0).clamp(40.0, screen_height() - 40.0),
    )
}

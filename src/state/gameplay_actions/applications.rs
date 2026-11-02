//! Application decisions: accepting offers, rejecting applicants and vetting.

use macroquad::prelude::*;
use macroquad_toolkit::rng;

use crate::narrative::TenantStory;
use crate::simulation::GameEvent;
use crate::state::GameplayState;
use crate::ui::colors;

/// Which paid screening an applicant receives.
pub(in crate::state) enum VettingCheck {
    Credit,
    Background,
}

impl GameplayState {
    /// Applications are listed per building; a stale index from another
    /// building's list must never act on this one.
    pub(in crate::state) fn application_in_active_building(&self, index: usize) -> bool {
        self.applications
            .get(index)
            .is_some_and(|application| application.building_id == self.active_building_id())
    }

    pub(in crate::state) fn accept_application(&mut self, application_index: usize) {
        if let Some(app) = self
            .applications
            .get(application_index)
            .filter(|application| application.building_id == self.active_building_id())
            .cloned()
        {
            let Some(apt) = self.building.get_apartment(app.apartment_id) else {
                return;
            };

            if !apt.is_vacant() || self.building.is_unit_sold(app.apartment_id) {
                self.event_log.log(
                    GameEvent::Notification {
                        message:
                            "Application could not be accepted because the unit is unavailable."
                                .to_string(),
                        level: crate::simulation::NotificationLevel::Warning,
                    },
                    self.current_tick,
                );
                return;
            }

            let apartment_unit = apt.unit_number.clone();
            let apartment_rent = apt.rent_price;
            // The unit is valid. From here the application is consumed
            // whether the applicant accepts or declines the offer.
            self.applications.remove(application_index);
            let mut tenant = app.tenant;
            let offer = crate::tenant::matching::LeaseOffer::from_config(
                apartment_rent,
                &self.config.matching.lease_defaults,
            );
            let accept_probability = crate::tenant::matching::evaluate_lease_offer(
                &tenant,
                &offer,
                &self.config.matching.lease_acceptance,
            );
            let leverage_penalty = tenant.negotiation_leverage() as f32 * 0.002;
            let adjusted_accept_probability =
                (accept_probability - leverage_penalty).clamp(0.0, 1.0);

            if rng::gen_range(0.0, 1.0) > adjusted_accept_probability {
                self.event_log.log(
                    GameEvent::Notification {
                        message: format!(
                            "{} declined the lease offer for Unit {}.",
                            tenant.name, apartment_unit
                        ),
                        level: crate::simulation::NotificationLevel::Info,
                    },
                    self.current_tick,
                );

                let mouse = mouse_position();
                self.floating_texts.spawn(
                    "Offer Declined",
                    vec2(mouse.0, mouse.1 - 20.0),
                    colors::WARNING(),
                );
                return;
            }

            tenant.move_into_building(app.building_id, app.apartment_id);

            if let Some(apt) = self.building.get_apartment_mut(app.apartment_id) {
                apt.move_in(tenant.id);
            }
            // The unit is let; competing applicants must not stay
            // actionable (or keep charging vetting fees).
            self.applications.retain(|other| {
                other.building_id != app.building_id || other.apartment_id != app.apartment_id
            });

            self.event_log.log(
                GameEvent::TenantMovedIn {
                    tenant_name: tenant.name.clone(),
                    apartment_unit,
                },
                self.current_tick,
            );

            let mouse = mouse_position();
            self.floating_texts.spawn(
                "Welcome!",
                vec2(mouse.0, mouse.1 - 20.0),
                colors::POSITIVE(),
            );

            let story = TenantStory::generate(tenant.id, &tenant.archetype);
            self.tenant_stories.insert(tenant.id, story);

            self.tenants.push(tenant);
        }
    }

    pub(in crate::state) fn run_vetting_check(
        &mut self,
        application_index: usize,
        check: VettingCheck,
    ) {
        if !self.application_in_active_building(application_index) {
            return;
        }
        let app = &mut self.applications[application_index];
        let vetting = &self.config.vetting;
        let outcome = match check {
            VettingCheck::Credit => crate::tenant::vetting::perform_credit_check(
                app,
                &mut self.funds,
                vetting,
                self.current_tick,
            )
            .map(|result| {
                (
                    result.reliability_score,
                    format!(
                        "Credit: {} - {}",
                        result.reliability_score, result.recommendation
                    ),
                )
            }),
            VettingCheck::Background => crate::tenant::vetting::perform_background_check(
                app,
                &mut self.funds,
                vetting,
                self.current_tick,
            )
            .map(|result| {
                (
                    result.behavior_score,
                    format!(
                        "Background: {} - {}",
                        result.behavior_score, result.history_notes
                    ),
                )
            }),
        };
        let center = vec2(screen_width() / 2.0, screen_height() / 2.0);
        match outcome {
            Some((score, text)) => {
                let tone = if score >= 75 {
                    colors::POSITIVE()
                } else if score >= 50 {
                    colors::WARNING()
                } else {
                    colors::NEGATIVE()
                };
                self.floating_texts.spawn(text, center, tone);
            }
            None => {
                let label = match check {
                    VettingCheck::Credit => "Cannot perform credit check",
                    VettingCheck::Background => "Cannot perform background check",
                };
                self.floating_texts.spawn(label, center, colors::NEGATIVE());
            }
        }
    }
}

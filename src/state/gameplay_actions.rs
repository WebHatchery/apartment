//! UI intent dispatch: routes each `UiAction` to the state change it requests.

use crate::economy::process_upgrade;
use crate::simulation::GameEvent;
use crate::ui::{colors, UiAction};
use macroquad::prelude::*;

use super::gameplay::GameplayState;
use super::mission_system;
use super::tutorial_system;

mod applications;
mod city;
mod navigation;
mod story;

use applications::VettingCheck;

impl GameplayState {
    /// Process a UI action
    pub(super) fn process_action(&mut self, action: UiAction) {
        let Some(action) = self.navigate(action) else {
            return;
        };
        match action {
            UiAction::ListApartment {
                apartment_id,
                preference,
            } => {
                if self.building.is_unit_sold(apartment_id) {
                    return;
                }
                if let Some(apt) = self.building.get_apartment_mut(apartment_id) {
                    apt.is_listed_for_lease = true;
                    apt.preferred_archetype = preference;

                    self.floating_texts.spawn(
                        "Listed for Lease",
                        vec2(screen_width() / 2.0, screen_height() / 2.0),
                        colors::POSITIVE(),
                    );
                }
            }

            UiAction::UnlistApartment { apartment_id } => {
                if let Some(apt) = self.building.get_apartment_mut(apartment_id) {
                    apt.is_listed_for_lease = false;
                    apt.preferred_archetype = None;

                    self.floating_texts.spawn(
                        "Property Unlisted",
                        vec2(screen_width() / 2.0, screen_height() / 2.0),
                        colors::TEXT(),
                    );
                }
            }

            UiAction::UpgradeAction(upgrade) => {
                let description = upgrade.label(&self.config.ui, &self.config.upgrades);
                match process_upgrade(
                    &upgrade,
                    &mut self.building,
                    &mut self.funds,
                    &self.config,
                    self.current_tick,
                ) {
                    Ok(cost) => {
                        self.event_log.log(
                            GameEvent::UpgradeCompleted { description, cost },
                            self.current_tick,
                        );

                        let mouse = mouse_position();
                        self.floating_texts.spawn(
                            format!("-${}", cost),
                            vec2(mouse.0, mouse.1 - 20.0),
                            colors::NEGATIVE(),
                        );
                    }
                    Err(reason) => self.report_action_failure(&reason),
                }
            }
            UiAction::SetRent {
                apartment_id,
                new_rent,
            } => match self.set_apartment_rent(apartment_id, new_rent) {
                Ok((old, new)) if old != new => self.floating_texts.spawn(
                    format!("Rent ${} → ${}", old, new),
                    vec2(screen_width() / 2.0, screen_height() / 2.0),
                    if new > old {
                        colors::WARNING()
                    } else {
                        colors::POSITIVE()
                    },
                ),
                Ok(_) => {}
                Err(reason) => self.report_action_failure(&reason),
            },
            UiAction::AcceptApplication { application_index } => {
                self.accept_application(application_index);
            }
            UiAction::RejectApplication { application_index } => {
                if self.application_in_active_building(application_index) {
                    self.applications.remove(application_index);
                }
            }
            UiAction::CreditCheck { application_index } => {
                self.run_vetting_check(application_index, VettingCheck::Credit);
            }
            UiAction::BackgroundCheck { application_index } => {
                self.run_vetting_check(application_index, VettingCheck::Background);
            }
            // One month per decision: further End Month taps while the
            // closed month is still being reviewed are ignored.
            UiAction::EndTurn if !self.month_summary_open => self.end_turn(),
            UiAction::EndTurn => {}
            UiAction::AcceptMission { mission_id } => {
                if !self.missions.accept_mission(mission_id, self.current_tick) {
                    self.report_action_failure("That task is no longer available");
                }
            }
            UiAction::PurchaseBuilding { listing_id } => self.purchase_listing(listing_id),

            UiAction::ApproveRequest { tenant_id } => self.resolve_request(tenant_id, true),
            UiAction::DenyRequest { tenant_id } => self.resolve_request(tenant_id, false),
            UiAction::SetMarketing { strategy } => {
                self.set_marketing_strategy(strategy);
            }
            UiAction::StartOpenHouse => match self.start_open_house() {
                Ok(cost) => self.floating_texts.spawn(
                    format!("Open House -${}", cost),
                    vec2(screen_width() / 2.0, screen_height() / 2.0),
                    colors::ACCENT(),
                ),
                Err(reason) => self.report_action_failure(&reason),
            },
            UiAction::SetUtilitiesIncluded { included } => {
                self.set_utilities_policy(included);
            }
            UiAction::SetInsuranceActive { active } => {
                self.set_insurance_policy(active);
            }
            UiAction::SellUnitAsCondo { apartment_id } => {
                self.sell_unit_as_condo(apartment_id);
            }
            UiAction::BuybackCondo { apartment_id } => {
                self.buy_back_condo(apartment_id);
            }
            UiAction::ResolveDialogue {
                dialogue_id,
                choice_index,
            } => {
                if let Some(effects) = self
                    .dialogue_system
                    .resolve_dialogue(dialogue_id, choice_index)
                {
                    for effect in effects {
                        self.apply_dialogue_effect(effect);
                    }

                    self.floating_texts.spawn(
                        "Dialogue Resolved",
                        vec2(screen_width() / 2.0, screen_height() / 2.0),
                        colors::ACCENT(),
                    );
                }
            }
            UiAction::ResolveEventChoice {
                event_id,
                choice_index,
            } => {
                if let Some(outcome) = self.narrative_events.process_choice(event_id, choice_index)
                {
                    self.apply_narrative_effect(&outcome.effect);
                    self.apply_reputation_change(
                        outcome.reputation_change,
                        outcome.neighborhood_id,
                    );
                }
            }
            // Navigation intents were consumed by `navigate` above.
            _ => {}
        }
    }

    pub(super) fn report_action_failure(&mut self, reason: &str) {
        self.event_log.log(
            GameEvent::Notification {
                message: reason.to_string(),
                level: crate::simulation::NotificationLevel::Warning,
            },
            self.current_tick,
        );
        self.floating_texts.spawn(
            reason,
            vec2(screen_width() / 2.0, screen_height() / 2.0),
            colors::NEGATIVE(),
        );
    }

    /// Update tutorial state based on game conditions (called every frame)
    pub fn update_tutorial(&mut self) {
        tutorial_system::update_tutorial(self);
    }

    /// Update missions states (called on turn end)
    pub fn update_missions(&mut self) {
        mission_system::update_missions(self);
    }
}

#[cfg(test)]
mod tests;

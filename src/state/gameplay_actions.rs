//! UI intent dispatch: routes each `UiAction` to the state change it requests.

use crate::economy::process_upgrade;
use crate::simulation::GameEvent;
use crate::ui::{colors, Selection, UiAction};
use macroquad::prelude::*;

use super::gameplay::{GameplayState, ViewMode};
use super::mission_system;
use super::tutorial_system;

mod applications;
mod city;
mod story;

use applications::VettingCheck;

impl GameplayState {
    /// Process a UI action
    pub(super) fn process_action(&mut self, action: UiAction) {
        match action {
            UiAction::SelectApartment(id) => {
                self.selection = Selection::Apartment(id);
                self.panel_scroll_offset = 0.0;
            }
            UiAction::OpenTenantUnit(id) => {
                self.view_mode = ViewMode::Building;
                self.selection = Selection::Apartment(id);
                self.panel_scroll_offset = 0.0;
            }
            UiAction::SelectTenant(id) => {
                self.selected_tenant_id = Some(id);
            }
            UiAction::ClearTenantSelection => {
                self.selected_tenant_id = None;
            }
            UiAction::SelectApplications(filter) => {
                self.view_mode = ViewMode::Building;
                self.selection = Selection::Applications(filter);
                self.panel_scroll_offset = 0.0;
                self.application_page = 0;
            }
            UiAction::SelectBuilding => {
                self.view_mode = ViewMode::Building;
                self.selection = Selection::Building;
                self.panel_scroll_offset = 0.0;
            }
            UiAction::SelectHallway => {
                self.selection = Selection::Hallway;
                self.panel_scroll_offset = 0.0;
            }
            UiAction::ClearSelection => {
                self.selection = Selection::None;
            }
            UiAction::ToggleActivityDrawer => {
                self.activity_drawer_open = !self.activity_drawer_open;
            }

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
                let description =
                    upgrade.label(&self.building, &self.config.ui, &self.config.upgrades);
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
            UiAction::EndTurn => {
                self.end_turn();
            }
            UiAction::OpenPauseMenu => {
                self.show_pause_menu = true;
            }
            UiAction::ReturnToMenu => {
                self.pending_quit_to_menu = true;
            }

            // Workspace navigation
            UiAction::OpenBuilding => {
                self.view_mode = ViewMode::Building;
                self.selection = Selection::None;
            }
            UiAction::OpenTenants => {
                self.view_mode = ViewMode::Tenants;
                self.selection = Selection::None;
            }
            UiAction::OpenFinances => {
                self.view_mode = ViewMode::Finances;
                self.selection = Selection::None;
            }
            UiAction::OpenCityMap => {
                self.save_building_to_city();
                self.view_mode = ViewMode::CityMap;
                self.selection = Selection::None;
            }
            UiAction::OpenMail => {
                self.view_mode = ViewMode::Mail;
            }
            UiAction::OpenTasks => {
                self.view_mode = ViewMode::Tasks;
            }
            UiAction::OpenMailItem { mail_id } => {
                if self.mailbox.mark_read(mail_id) {
                    self.selected_mail_id = Some(mail_id);
                }
            }
            UiAction::ClearSelectedMail => {
                self.selected_mail_id = None;
            }
            UiAction::SetInboxPage { page } => {
                self.inbox_page = page;
            }
            UiAction::SetTasksPage { page } => {
                self.tasks_page = page;
            }
            UiAction::SetApplicationPage { page } => {
                self.application_page = page;
            }
            UiAction::SetTenantsPage { page } => {
                self.tenants_page = page;
            }
            UiAction::SetRequestsPage { page } => {
                self.requests_page = page;
            }
            UiAction::SetCareerBadgesPage { page } => {
                self.career_badges_page = page;
            }
            UiAction::AcceptMission { mission_id } => {
                if !self.missions.accept_mission(mission_id, self.current_tick) {
                    self.report_action_failure("That task is no longer available");
                }
            }
            UiAction::PurchaseBuilding { listing_id } => self.purchase_listing(listing_id),

            // Phase 3: Tenant requests
            UiAction::ApproveRequest { tenant_id } => {
                let effect = self.tenant_stories.get_mut(&tenant_id).and_then(|story| {
                    story.pending_request.take().map(|request| {
                        let effect = request.approval_effect();
                        story.add_event(
                            self.current_tick,
                            "Request approved by landlord",
                            effect.clone(),
                        );
                        effect
                    })
                });

                if let Some(effect) = effect {
                    self.apply_story_impact(tenant_id, effect);
                }
            }
            UiAction::DenyRequest { tenant_id } => {
                let effect = self.tenant_stories.get_mut(&tenant_id).and_then(|story| {
                    story.pending_request.take().map(|request| {
                        let effect = request.denial_effect();
                        story.add_event(
                            self.current_tick,
                            "Request denied by landlord",
                            effect.clone(),
                        );
                        effect
                    })
                });

                if let Some(effect) = effect {
                    self.apply_story_impact(tenant_id, effect);
                }
            }

            // Phase 3: Ownership
            UiAction::SelectOwnership => {
                self.selection = Selection::Ownership;
            }
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

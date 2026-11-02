//! Game view rendering - split from gameplay.rs for maintainability

use crate::assets::AssetManager;
use crate::narrative::NotificationCategory;
use crate::ui::workspace_nav::{draw_workspace_nav, WorkspaceTab};
use crate::ui::{
    draw_apartment_panel, draw_application_panel, draw_building_summary, draw_building_view,
    draw_hallway_panel, draw_header, draw_notifications, draw_ownership_panel, Selection,
};
use macroquad::prelude::*;

use super::gameplay::{GameplayState, ViewMode};

impl GameplayState {
    /// Main draw function - dispatches to appropriate view
    pub fn draw(&mut self, assets: &AssetManager) {
        let pause_was_showing = self.show_pause_menu;
        match self.view_mode {
            ViewMode::Building => {
                self.draw_building_mode(assets);
            }
            ViewMode::Tenants => {
                if let Some(action) = crate::ui::workspace_views::draw_tenants_view(self, assets) {
                    self.pending_actions.push(action);
                }
            }
            ViewMode::Finances => {
                if let Some(action) = crate::ui::workspace_finances::draw_finances_view(self) {
                    self.pending_actions.push(action);
                }
            }
            ViewMode::CityMap => {
                let (action, scroll) = crate::ui::city_view::draw_city_view(
                    &self.city,
                    self.selected_neighborhood_id,
                    self.city_scroll,
                    assets,
                    &self.narrative_events,
                );
                self.city_scroll = scroll;
                if let Some(action) = action {
                    self.handle_city_action(action);
                }
            }
            ViewMode::Market => {
                let listings: Vec<&crate::city::PropertyListing> =
                    self.city.market.listings.iter().collect();
                if let Some(action) = crate::ui::city_view::draw_market_panel(
                    &listings,
                    &self.city.neighborhoods,
                    self.funds.balance,
                    self.market_page,
                    self.selected_neighborhood_id,
                    self.pending_property_purchase,
                    assets,
                ) {
                    self.handle_city_action(action);
                }
            }
            ViewMode::Mail => {
                if let Some(action) = crate::ui::workspace_inbox::draw_inbox_view(self, assets) {
                    self.pending_actions.push(action);
                }
            }
            ViewMode::Tasks => {
                if let Some(action) = crate::ui::workspace_tasks::draw_tasks_view(self, assets) {
                    self.pending_actions.push(action);
                }
            }
            ViewMode::CareerSummary => {
                if let Some(action) = crate::ui::career_summary::draw_career_summary(self, assets) {
                    self.pending_actions.push(action);
                }
            }
        }

        if self.view_mode != ViewMode::CareerSummary {
            if let Some(action) = draw_header(
                self.funds.balance,
                self.current_tick,
                self.building.occupancy_count(),
                self.building.rental_unit_count(),
            ) {
                self.pending_actions.push(action);
            }
            let active_tab = match self.view_mode {
                ViewMode::Building => WorkspaceTab::Building,
                ViewMode::Tenants => WorkspaceTab::Tenants,
                ViewMode::Finances => WorkspaceTab::Finances,
                ViewMode::CityMap | ViewMode::Market => WorkspaceTab::City,
                ViewMode::Mail => WorkspaceTab::Inbox,
                ViewMode::Tasks => WorkspaceTab::Tasks,
                ViewMode::CareerSummary => unreachable!(),
            };
            let pending_tasks = self.missions.available_missions().len()
                + self.missions.active_missions().len()
                + self
                    .tenant_stories
                    .values()
                    .filter(|story| story.pending_request.is_some())
                    .count();
            if let Some(action) = draw_workspace_nav(
                active_tab,
                self.mailbox.unread_count(),
                pending_tasks,
                assets,
            ) {
                self.pending_actions.push(action);
            }
        }

        // The history drawer owns clicks in its covered area.
        if self.activity_drawer_open {
            let footer_y = screen_height() - crate::ui::layout::FOOTER_HEIGHT();
            if mouse_position().1 >= footer_y - crate::ui::notifications::drawer_height() {
                self.pending_actions.clear();
            }
        }

        let blocking_event_open = self
            .narrative_events
            .events
            .iter()
            .any(|e| !e.read && e.requires_response);

        // Draw blocking narrative event modal (Phase 4)
        // Find first unread event that requires response
        let blocking_event = self
            .narrative_events
            .events
            .iter()
            .find(|e| !e.read && e.requires_response);

        if self.month_summary_open {
            // The closed month comes first; choices and coaching follow.
            if let Some(summary) = &self.latest_month {
                let action = crate::ui::month_summary::draw_month_summary(summary);
                self.pending_actions
                    .retain(|action| matches!(action, crate::ui::UiAction::OpenPauseMenu));
                self.pending_actions.extend(action);
            }
        } else if let Some(event) = blocking_event {
            if self.event_scroll_event_id != Some(event.id) {
                self.event_scroll_event_id = Some(event.id);
                self.event_scroll_offset = 0.0;
            }
            let (action, scroll_offset) =
                crate::ui::event_modal::draw_event_modal(event, assets, self.event_scroll_offset);
            self.event_scroll_offset = scroll_offset;
            if let Some(action) = action {
                self.pending_actions.push(action);
            }
            self.pending_actions.retain(|action| {
                matches!(
                    action,
                    crate::ui::UiAction::ResolveEventChoice { .. }
                        | crate::ui::UiAction::OpenPauseMenu
                )
            });
        } else {
            self.event_scroll_event_id = None;
            self.event_scroll_offset = 0.0;
        }

        // Compact activity handle and optional history drawer.
        if self.view_mode != ViewMode::CareerSummary {
            let issue = self.current_issue();
            if let Some(action) = draw_notifications(
                &self.event_log,
                issue.as_ref(),
                self.latest_month.as_ref(),
                assets,
                self.activity_drawer_open,
            ) {
                self.pending_actions.push(action);
            }
        }

        // Floating text
        self.floating_texts.draw();

        if self.coach_visible(blocking_event_open) {
            self.draw_tutorial_coach(assets);
        }
        if self.help_open && !self.month_summary_open && !blocking_event_open {
            self.draw_help_card();
        }

        // Toasts dismiss on their own buttons, so they stay hidden while the
        // pause menu owns input. Opening the pause menu is always allowed.
        if !self.show_pause_menu {
            // Tutorial overlay (takes precedence)
            if self.month_summary_open || blocking_event_open {
                // Coaching and toasts wait for the summary and any choice.
            } else if self.tutorial.active && !self.tutorial.pending_messages.is_empty() {
                self.draw_tutorial_overlay(assets);
                self.pending_actions
                    .retain(|action| matches!(action, crate::ui::UiAction::OpenPauseMenu));
            }
            // Notification overlay (shows when tutorial is done/empty); the
            // open history drawer owns that area, so toasts wait for it.
            else if self.notifications.has_pending() && !self.activity_drawer_open {
                self.draw_notification_overlay();
            }
        }

        // Draw pause menu on top of everything if active
        if self.show_pause_menu {
            self.draw_pause_menu_overlay();
        }
        if pause_was_showing {
            self.pending_actions.clear();
        }
    }

    pub(super) fn draw_building_mode(&mut self, assets: &AssetManager) {
        let building_id = self.active_building_id();
        let active_tenants: Vec<_> = self
            .tenants
            .iter()
            .filter(|tenant| tenant.building_id == building_id)
            .cloned()
            .collect();

        // Reframe on selection or canvas resize so the place being acted on
        // stays in view above (or beside) its inspector.
        let canvas = (screen_width(), screen_height());
        if canvas != self.framed_canvas {
            self.framed_canvas = canvas;
            self.tower_focus_pending = true;
        }
        if std::mem::take(&mut self.tower_focus_pending) {
            self.tower_scroll_offset = crate::ui::building_view::reveal_selection(
                &self.building,
                &self.selection,
                self.tower_scroll_offset,
            );
        }

        // Draw Building View
        let (tower_action, tower_scroll) = draw_building_view(
            &self.building,
            &active_tenants,
            &self.selection,
            assets,
            self.tower_scroll_offset,
        );
        self.tower_scroll_offset = tower_scroll;
        if let Some(action) = tower_action {
            self.pending_actions.push(action);
        }

        match self.selection {
            Selection::Apartment(id) => {
                if let Some(apt) = self.building.get_apartment(id) {
                    let (action, new_scroll) = draw_apartment_panel(
                        apt,
                        &crate::ui::UnitPanelContext {
                            building: &self.building,
                            tenants: &active_tenants,
                            money: self.funds.balance,
                            assets,
                            config: &self.config,
                            network: &self.tenant_network,
                            stories: &self.tenant_stories,
                            disclosure: self.panel_disclosure,
                        },
                        self.panel_scroll_offset,
                    );
                    self.panel_scroll_offset = new_scroll;
                    if let Some(action) = action {
                        self.pending_actions.push(action);
                    }
                }
            }
            Selection::Hallway => {
                let (action, new_scroll) = draw_hallway_panel(
                    &self.building,
                    self.funds.balance,
                    self.panel_scroll_offset,
                    assets,
                    &self.config,
                    self.panel_disclosure,
                );
                self.panel_scroll_offset = new_scroll;
                if let Some(action) = action {
                    self.pending_actions.push(action);
                }
            }
            Selection::Applications(filter) => {
                if let Some(action) = draw_application_panel(
                    &self.applications,
                    self.active_building_id(),
                    &self.building,
                    filter,
                    self.application_page,
                    0.0,
                    assets,
                ) {
                    self.pending_actions.push(action);
                }
            }
            Selection::Ownership => {
                let (action, new_scroll) = draw_ownership_panel(
                    &self.building,
                    self.condo_sale_market_multiplier(),
                    self.panel_scroll_offset,
                );
                self.panel_scroll_offset = new_scroll;
                if let Some(action) = action {
                    self.pending_actions.push(action);
                }
            }
            Selection::None => {}
            Selection::Building => {
                let monthly_net = self.ledger.reports.last().map_or(0, |report| report.net);
                if let Some(action) = draw_building_summary(
                    &self.building,
                    &active_tenants,
                    self.applications
                        .iter()
                        .filter(|application| application.building_id == building_id)
                        .count(),
                    monthly_net,
                ) {
                    self.pending_actions.push(action);
                }
            }
        }
        if self.selection != Selection::None {
            let panel = crate::ui::layout::context_panel();
            if crate::ui::widgets::button_at(
                Rect::new(panel.right() - 68.0, panel.y + 2.0, 64.0, 34.0),
                "Close",
                true,
                crate::ui::theme::Tone::Secondary,
            ) {
                self.pending_actions.clear();
                self.pending_actions
                    .push(crate::ui::UiAction::ClearSelection);
            }
        }
    }

    /// Draw the tutorial overlay as a bottom toast. Dismisses on "Next".
    pub(super) fn draw_tutorial_overlay(&mut self, assets: &AssetManager) {
        if self.tutorial.pending_messages.is_empty() {
            return;
        }
        let message = self.tutorial.pending_messages[0].clone();
        let clicked = if let Some(portrait) = assets.get_texture("npc_uncle_artie") {
            crate::ui::widgets::draw_portrait_toast(
                portrait,
                "Uncle Artie",
                &message,
                crate::ui::widgets::ToastKind::Info,
                "Continue",
            )
        } else {
            crate::ui::widgets::draw_toast(
                "",
                "Uncle Artie",
                &message,
                crate::ui::widgets::ToastKind::Info,
                "Continue",
            )
        };
        if clicked {
            self.tutorial.pending_messages.remove(0);
        }
    }

    /// Non-blocking, target-anchored coaching for the current tutorial step.
    /// The introductory dialogue still uses a modal toast; once dismissed,
    /// this marker stays beside the control the player actually needs.
    /// Draw the hint/relationship notification as a bottom toast. Dismisses on
    /// "OK".
    pub(super) fn draw_notification_overlay(&mut self) {
        let Some(notification) = self.notifications.pending.first() else {
            return;
        };
        let kind = match notification.category {
            NotificationCategory::Positive => crate::ui::widgets::ToastKind::Positive,
            NotificationCategory::Warning => crate::ui::widgets::ToastKind::Warning,
            NotificationCategory::Info => crate::ui::widgets::ToastKind::Info,
            NotificationCategory::Hint => crate::ui::widgets::ToastKind::Hint,
        };
        let mut body = notification.message.clone();
        if let Some(desc) = &notification.description {
            body.push('\n');
            body.push_str(desc);
        }
        if crate::ui::widgets::draw_toast("", "", &body, kind, "OK") {
            self.notifications.pop();
        }
    }
}

//! Game view rendering - split from gameplay.rs for maintainability

use crate::assets::AssetManager;
use crate::narrative::NotificationCategory;
use crate::narrative::TutorialMilestone;
use crate::ui::workspace_nav::{draw_workspace_nav, WorkspaceTab};
use crate::ui::{
    colors, draw_apartment_panel, draw_application_panel, draw_building_summary,
    draw_building_view, draw_hallway_panel, draw_header, draw_notifications, draw_ownership_panel,
    Selection,
};
use macroquad::prelude::*;

use super::gameplay::{GameplayState, ViewMode};
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

impl GameplayState {
    /// Main draw function - dispatches to appropriate view
    pub fn draw(&mut self, assets: &AssetManager) {
        let pause_was_showing = self.show_pause_menu;
        if self.view_mode != ViewMode::CareerSummary {
            let monthly_net = self.ledger.reports.last().map_or(0, |report| report.net);
            if let Some(action) = draw_header(
                self.funds.balance,
                monthly_net,
                self.current_tick,
                &self.building.name,
                self.building.occupancy_count(),
                self.building.rental_unit_count(),
                assets,
            ) {
                self.pending_actions.push(action);
            }
        }
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
                if let Some(action) = crate::ui::workspace_views::draw_finances_view(self) {
                    self.pending_actions.push(action);
                }
            }
            ViewMode::CityMap => {
                if let Some(action) =
                    crate::ui::city_view::draw_city_map(&self.city, assets, &self.narrative_events)
                {
                    self.handle_city_action(action);
                }

                if let Some(action) = crate::ui::city_view::draw_portfolio_panel(
                    &self.city,
                    self.city.active_building_index,
                    assets,
                ) {
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
                    assets,
                ) {
                    self.handle_city_action(action);
                }
            }
            ViewMode::Mail => {
                if let Some(action) = crate::ui::workspace_inbox::draw_inbox_view(self) {
                    self.pending_actions.push(action);
                }
            }
            ViewMode::Tasks => {
                if let Some(action) = crate::ui::workspace_tasks::draw_tasks_view(self, assets) {
                    self.pending_actions.push(action);
                }
            }
            ViewMode::CareerSummary => {
                if let Some(action) = crate::ui::career_summary::draw_career_summary(self) {
                    self.pending_actions.push(action);
                }
            }
        }

        if self.view_mode != ViewMode::CareerSummary {
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

        // Draw blocking narrative event modal (Phase 4)
        // Find first unread event that requires response
        let blocking_event = self
            .narrative_events
            .events
            .iter()
            .find(|e| !e.read && e.requires_response);

        if let Some(event) = blocking_event {
            if let Some(action) = crate::ui::event_modal::draw_event_modal(event) {
                self.pending_actions.push(action);
            }
            self.pending_actions
                .retain(|action| matches!(action, crate::ui::UiAction::ResolveEventChoice { .. }));
        }

        // Compact activity handle and optional history drawer.
        if self.view_mode != ViewMode::CareerSummary {
            if let Some(action) = draw_notifications(
                &self.event_log,
                self.current_tick,
                assets,
                self.activity_drawer_open,
            ) {
                self.pending_actions.push(action);
            }
        }

        // Floating text
        self.floating_texts.draw();

        if self.view_mode == ViewMode::Building
            && self.tutorial.active
            && self.tutorial.pending_messages.is_empty()
            && !self.activity_drawer_open
        {
            self.draw_tutorial_coach();
        }

        // Tutorial overlay (takes precedence)
        if self.tutorial.active && !self.tutorial.pending_messages.is_empty() {
            self.draw_tutorial_overlay(assets);
            self.pending_actions.clear();
        }
        // Notification overlay (shows when tutorial is done/empty)
        else if self.notifications.has_pending() {
            self.draw_notification_overlay();
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

        // Draw Building View
        if let Some(action) =
            draw_building_view(&self.building, &active_tenants, &self.selection, assets)
        {
            self.pending_actions.push(action);
        }

        // Slide the detail panel in from the right as the selection tween eases
        // to 1.0 (0 offset = settled in place).
        let panel_offset = (1.0 - self.panel_tween.current()) * 60.0;

        match self.selection {
            Selection::Apartment(id) => {
                if let Some(apt) = self.building.get_apartment(id) {
                    let (action, new_scroll) = draw_apartment_panel(
                        apt,
                        &self.building,
                        &active_tenants,
                        self.funds.balance,
                        panel_offset,
                        self.panel_scroll_offset,
                        assets,
                        &self.config,
                        &self.tenant_network,
                        &self.tenant_stories,
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
                    panel_offset,
                    self.panel_scroll_offset,
                    assets,
                    &self.config,
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
            Selection::None => {
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
    }

    /// Draw the pause menu overlay (called from draw())
    pub(super) fn draw_pause_menu_overlay(&mut self) {
        // Semi-transparent overlay
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.7),
        );

        // Menu panel
        let panel_w = 300.0;
        let panel_h = 366.0;
        let panel_x = (screen_width() - panel_w) / 2.0;
        let panel_y = (screen_height() - panel_h) / 2.0;

        draw_rectangle(panel_x, panel_y, panel_w, panel_h, colors::SURFACE());
        draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, colors::ACCENT());

        // Title
        let title = "PAUSED";
        let title_width = measure_ui_text(title, None, 32, 1.0).width;
        draw_ui_text(
            title,
            panel_x + (panel_w - title_width) / 2.0,
            panel_y + 40.0,
            32.0,
            colors::TEXT_BRIGHT(),
        );

        let btn_w = 200.0;
        let btn_h = 40.0;
        let btn_x = panel_x + (panel_w - btn_w) / 2.0;
        let mut btn_y = panel_y + 70.0;

        // Resume button
        if self.menu_button(btn_x, btn_y, btn_w, btn_h, "Resume") {
            self.show_pause_menu = false;
        }
        btn_y += 50.0;

        // Fullscreen toggle
        let fs_label = if self.is_fullscreen {
            "Windowed Mode"
        } else {
            "Fullscreen"
        };
        if self.menu_button(btn_x, btn_y, btn_w, btn_h, fs_label) {
            self.is_fullscreen = !self.is_fullscreen;
            set_fullscreen(self.is_fullscreen);
        }
        btn_y += 50.0;

        // Save button
        if self.menu_button(btn_x, btn_y, btn_w, btn_h, "Save Game") {
            if crate::save::save_game(self).is_ok() {
                self.floating_texts.spawn(
                    "Game Saved!",
                    vec2(screen_width() / 2.0, screen_height() / 2.0),
                    colors::POSITIVE(),
                );
            }
            self.show_pause_menu = false;
        }
        btn_y += 50.0;

        // Quit to Menu button
        if self.menu_button(btn_x, btn_y, btn_w, btn_h, "Quit to Menu") {
            self.pending_quit_to_menu = true;
        }

        // Quit Game button (exits completely) — native only; a browser tab has
        // nothing to exit and std::process::exit is unsupported on wasm.
        #[cfg(not(target_arch = "wasm32"))]
        {
            btn_y += 50.0;
            if self.menu_button(btn_x, btn_y, btn_w, btn_h, "Quit Game") {
                std::process::exit(0);
            }
        }

        // Touch-first continuation hint; keyboard shortcuts remain supplemental.
        draw_ui_text(
            "Tap RESUME to continue",
            panel_x + (panel_w - 164.0) / 2.0,
            panel_y + panel_h - 20.0,
            14.0,
            colors::TEXT_DIM(),
        );
    }

    /// Helper for drawing menu buttons
    pub(super) fn menu_button(&self, x: f32, y: f32, w: f32, h: f32, text: &str) -> bool {
        let mouse = mouse_position();
        let hovered = mouse.0 >= x && mouse.0 <= x + w && mouse.1 >= y && mouse.1 <= y + h;
        let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

        let bg_color = if hovered {
            colors::HOVERED()
        } else {
            colors::SURFACE_ALT()
        };

        draw_rectangle(x, y, w, h, bg_color);
        draw_rectangle_lines(
            x,
            y,
            w,
            h,
            1.0,
            if hovered {
                colors::PRIMARY()
            } else {
                colors::BORDER_STRONG()
            },
        );

        let text_width = measure_ui_text(text, None, 20, 1.0).width;
        draw_ui_text(
            text,
            x + (w - text_width) / 2.0,
            y + h / 2.0 + 6.0,
            20.0,
            colors::TEXT(),
        );

        clicked
    }

    /// Draw the tutorial overlay as a bottom toast. Dismisses on "Next".
    pub(super) fn draw_tutorial_overlay(&mut self, _assets: &AssetManager) {
        if self.tutorial.pending_messages.is_empty() {
            return;
        }
        let message = format!("{} Tap CONTINUE.", self.tutorial.pending_messages[0]);
        if crate::ui::widgets::draw_toast(
            "",
            "Uncle Artie",
            &message,
            crate::ui::widgets::ToastKind::Info,
            "Continue",
        ) {
            self.tutorial.pending_messages.remove(0);
        }
    }

    /// Non-blocking, target-anchored coaching for the current tutorial step.
    /// The introductory dialogue still uses a modal toast; once dismissed,
    /// this marker stays beside the control the player actually needs.
    fn draw_tutorial_coach(&self) {
        use crate::ui::theme::{color, scale, space};
        use crate::ui::widgets::{draw_card, line_height, wrap};

        let Some(milestone) = self.tutorial.current_milestone.as_ref() else {
            return;
        };
        let view_w = screen_width() * crate::ui::layout::PANEL_SPLIT();
        let footer_y = screen_height() - crate::ui::layout::FOOTER_HEIGHT();
        let (anchor, message, place_above) = match milestone {
            TutorialMilestone::InheritedMess => (
                vec2(view_w / 2.0, footer_y - 30.0),
                "Start here: select the hallway, then choose a repair until it reaches 80% condition.",
                true,
            ),
            TutorialMilestone::FirstResident => {
                let listed = self
                    .building
                    .apartments
                    .iter()
                    .any(|unit| unit.is_vacant() && unit.is_listed_for_lease);
                let active_apps = self
                    .applications
                    .iter()
                    .any(|application| application.building_id == self.active_building_id());
                if active_apps {
                    (
                        vec2(view_w / 2.0 - 72.0, crate::ui::layout::HEADER_HEIGHT() + 28.0),
                        "Applicants are ready. Open Applications and choose a resident.",
                        false,
                    )
                } else if listed {
                    (
                        vec2(screen_width() - 68.0, 32.0),
                        "The unit is listed. End the month to bring in applicants.",
                        false,
                    )
                } else {
                    (
                        vec2(view_w / 2.0, crate::ui::layout::HEADER_HEIGHT() + 150.0),
                        "Select a vacant unit, set a fair rent, and list it for lease.",
                        false,
                    )
                }
            }
            TutorialMilestone::TheLeak => (
                vec2(view_w / 2.0, crate::ui::layout::HEADER_HEIGHT() + 190.0),
                "The damaged unit is marked in red. Select it and make a repair.",
                false,
            ),
            TutorialMilestone::Complete => return,
        };

        let card_w = view_w.min(360.0) - space::LG * 2.0;
        let lines = wrap(message, card_w - space::LG * 2.0, scale::BODY);
        let card_h = 42.0 + lines.len() as f32 * line_height(scale::BODY);
        let card_x = (anchor.x - card_w / 2.0).clamp(space::SM, view_w - card_w - space::SM);
        let preferred_y = if place_above {
            anchor.y - card_h - 18.0
        } else {
            anchor.y + 18.0
        };
        let card_y = preferred_y.clamp(
            crate::ui::layout::HEADER_HEIGHT() + space::SM,
            footer_y - card_h - space::SM,
        );

        draw_line(
            anchor.x,
            anchor.y,
            anchor.x,
            if place_above { card_y + card_h } else { card_y },
            3.0,
            color::PRIMARY(),
        );
        draw_circle(anchor.x, anchor.y, 7.0, color::PRIMARY());
        draw_card(Rect::new(card_x, card_y, card_w, card_h), true);
        draw_ui_text(
            "UNCLE ARTIE'S NEXT STEP",
            card_x + space::LG,
            card_y + 21.0,
            scale::LABEL,
            color::PRIMARY(),
        );
        let mut y = card_y + 36.0;
        for line in lines {
            draw_ui_text(
                &line,
                card_x + space::LG,
                y + scale::BODY,
                scale::BODY,
                color::TEXT(),
            );
            y += line_height(scale::BODY);
        }
    }

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

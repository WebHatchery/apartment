//! Screenshot-harness scene seeding: each named scene resolves to a base
//! screen plus an optional deterministic adjustment of the gameplay state.

use super::capture_seeds::*;
use super::Game;
use crate::state::{GameState, GameplayState, MenuState, ViewMode};
use crate::ui::Selection;

/// The screen a capture scene starts from before scene-specific tweaks.
enum CaptureBase {
    ResidentGallery,
    Menu,
    MenuScreen,
    Gameplay { showcase: bool },
    Workspace(ViewMode),
}

fn capture_base(scene: &str) -> CaptureBase {
    use CaptureBase::*;
    match scene {
        "resident_sprites" => ResidentGallery,
        "menu" | "menu_compact" | "menu_wide" | "menu_tiny" => Menu,
        "menu_new_game" | "menu_new_game_tiny" | "menu_settings" | "menu_settings_tiny" => {
            MenuScreen
        }
        "tower_scrolled" | "gameplay" => Gameplay { showcase: false },
        _ => match scene.split('_').next().unwrap_or_default() {
            "tenants" => Workspace(ViewMode::Tenants),
            "finances" => Workspace(ViewMode::Finances),
            "city" => Workspace(ViewMode::CityMap),
            "market" | "purchase" => Workspace(ViewMode::Market),
            "inbox" => Workspace(ViewMode::Mail),
            "tasks" | "requests" => Workspace(ViewMode::Tasks),
            "career" => Workspace(ViewMode::CareerSummary),
            "showcase" | "condition" | "building" | "unit" | "hallway" | "applications"
            | "ownership" | "pause" | "history" | "event" | "notification" => {
                Gameplay { showcase: true }
            }
            _ => Gameplay { showcase: false },
        },
    }
}

/// Scene-specific state on top of the base screen; scenes without an entry
/// show the base screen unchanged.
fn adjust_scene(scene: &str, state: &mut GameplayState) {
    match scene {
        "condition_worn" | "condition_repaired" => {
            let condition = if scene == "condition_worn" { 18 } else { 90 };
            state.building.hallway_condition = condition;
            for unit in &mut state.building.apartments {
                unit.condition = condition;
            }
        }
        "building_details" => state.selection = Selection::Building,
        "unit_showcase" | "unit_compact" | "unit_tiny" => {
            state.selection = Selection::Apartment(3);
        }
        "unit_scrolled" | "unit_scrolled_tiny" => {
            state.selection = Selection::Apartment(3);
            state.panel_scroll_offset = if scene == "unit_scrolled_tiny" {
                480.0
            } else {
                240.0
            };
        }
        "hallway_showcase" | "hallway_compact" | "hallway_tiny" => {
            state.selection = Selection::Hallway;
        }
        "hallway_scrolled" | "hallway_scrolled_tiny" => {
            state.selection = Selection::Hallway;
            state.panel_scroll_offset = 240.0;
        }
        "applications_showcase" | "applications_compact" | "applications_tiny" => {
            state.selection = Selection::Applications(None);
        }
        "applications_more_showcase" | "applications_more_tiny" => {
            state.selection = Selection::Applications(None);
            state.application_page = 1;
        }
        "ownership_showcase" | "ownership_compact" | "ownership_wide" | "ownership_tiny" => {
            state.selection = Selection::Ownership;
        }
        "ownership_scrolled" => {
            state.selection = Selection::Ownership;
            state.panel_scroll_offset = 174.0;
        }
        "tenants_detail_tiny" => {
            state.selected_tenant_id = state.tenants.first().map(|tenant| tenant.id);
        }
        "tenants_more_showcase" | "tenants_more_tiny" => state.tenants_page = 1,
        "finances_scrolled_tiny" => state.finance_scroll_offset = 120.0,
        "city_filtered_showcase" | "city_filtered_tiny" => {
            state.selected_neighborhood_id = Some(1);
        }
        "market_showcase" | "market_compact" | "market_wide" | "market_tiny" => {
            seed_showcase_market(state);
        }
        "market_filtered_showcase" | "market_filtered_tiny" => {
            seed_showcase_market(state);
            state.selected_neighborhood_id = Some(1);
        }
        "purchase_review_showcase" | "purchase_review_tiny" => {
            seed_showcase_market(state);
            state.pending_property_purchase = Some(0);
        }
        "inbox_list_tiny" => state.selected_mail_id = None,
        "tasks_scrolled_tiny" => state.tasks_scroll_offset = 360.0,
        "requests_more_showcase" | "requests_more_tiny" => {
            seed_showcase_requests(state);
            state.requests_page = 1;
        }
        "pause_showcase" | "pause_compact" | "pause_tiny" => state.show_pause_menu = true,
        "pause_settings" | "pause_settings_tiny" => {
            state.show_pause_menu = true;
            state.pause_settings = Some(crate::state::SettingsScreen::new(
                crate::state::centered_settings_top,
            ));
        }
        "history_showcase" | "history_compact" | "history_tiny" => {
            seed_showcase_activity(state);
            state.activity_drawer_open = true;
        }
        "career_showcase" | "career_compact" | "career_wide" | "career_tiny" => {
            seed_showcase_career(state);
        }
        "career_more_showcase" | "career_more_tiny" => {
            seed_showcase_career(state);
            state.career_badges_page = 1;
        }
        "tutorial_coach_showcase" | "tutorial_coach_compact" | "tutorial_coach_tiny" => {
            state.tutorial.pending_messages.clear();
        }
        "event_showcase" | "event_compact" | "event_tiny" => seed_showcase_event(state),
        "event_notice_showcase" | "event_notice_compact" | "event_notice_tiny" => {
            seed_showcase_notice(state);
        }
        "notification_showcase" | "notification_compact" | "notification_tiny" => {
            let mut notification = crate::narrative::notifications::GameNotification::positive(
                "+",
                "Maya R. and Jon Bell have become friends!",
            );
            notification.description =
                Some("Friendly neighbors boost each other's happiness.".to_string());
            state.notifications.pending.push(notification);
        }
        "tower_scrolled" => {
            state.tutorial.pending_messages.clear();
            state.tower_scroll_offset = f32::MAX;
        }
        _ => {}
    }
}

impl Game {
    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.capture_resident_gallery = false;
        match capture_base(scene) {
            CaptureBase::ResidentGallery => {
                self.capture_resident_gallery = true;
                return;
            }
            CaptureBase::Menu => self.state = GameState::Menu(MenuState::new()),
            CaptureBase::MenuScreen => self.state = GameState::Menu(MenuState::for_capture(scene)),
            CaptureBase::Gameplay { showcase } => self.seed_gameplay_capture(showcase),
            CaptureBase::Workspace(view_mode) => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.view_mode = view_mode;
                }
            }
        }
        if let GameState::Gameplay(state) = &mut self.state {
            adjust_scene(scene, state);
        }
    }

    fn seed_gameplay_capture(&mut self, showcase: bool) {
        let template = crate::data::templates::load_templates()
            .and_then(|templates| templates.templates.into_iter().next());
        let Some(template) = template else {
            self.state = GameState::Menu(MenuState::new());
            return;
        };
        // Capture scenes are regression evidence, so their residents, events,
        // applications, and decorative activity must not drift between runs.
        let mut state =
            GameplayState::new_with_template_seed(self.config.clone(), template, 0x5EC0_0D57);
        if showcase {
            seed_showcase_residents(&mut state);
        }
        self.state = GameState::Gameplay(state);
    }
}

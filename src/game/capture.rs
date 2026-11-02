//! Screenshot-harness scene seeding. A scene name is a state key plus an
//! optional review-size suffix (`_showcase`, `_compact`, `_wide`, `_tiny`,
//! `_portrait`); the suffix only names the PNG, the window size comes from the
//! capture wrapper. Each key resolves to a base screen plus an optional
//! deterministic adjustment of the gameplay state.

use super::capture_seeds::*;
use super::Game;
use crate::state::{GameState, GameplayState, MenuState, ViewMode};
use crate::ui::{Disclosure, Selection};

const SIZE_SUFFIXES: [&str; 5] = ["_showcase", "_compact", "_wide", "_tiny", "_portrait"];

/// The state key of a scene name, without its review-size suffix.
fn scene_key(scene: &str) -> &str {
    SIZE_SUFFIXES
        .iter()
        .find_map(|suffix| scene.strip_suffix(suffix))
        .filter(|key| !key.is_empty())
        .unwrap_or(scene)
}

/// The screen a capture scene starts from before scene-specific tweaks.
enum CaptureBase {
    ResidentGallery,
    Menu,
    MenuScreen,
    Gameplay { showcase: bool, template: usize },
    Workspace(ViewMode),
}

fn capture_base(key: &str) -> CaptureBase {
    use CaptureBase::*;
    let showcase = Gameplay {
        showcase: true,
        template: 0,
    };
    match key {
        "resident_sprites" => ResidentGallery,
        "menu" => Menu,
        "menu_new_game" | "menu_settings" => MenuScreen,
        "tower_scrolled" | "gameplay" | "tutorial" | "tutorial_coach" => Gameplay {
            showcase: false,
            template: 0,
        },
        // Blackwood Manor: four floors of four units.
        "tower_tall" | "tower_tall_unit" => Gameplay {
            showcase: true,
            template: 2,
        },
        _ => match key.split('_').next().unwrap_or_default() {
            "tenants" => Workspace(ViewMode::Tenants),
            "finances" => Workspace(ViewMode::Finances),
            "city" => Workspace(ViewMode::CityMap),
            "market" | "purchase" => Workspace(ViewMode::Market),
            "inbox" => Workspace(ViewMode::Mail),
            "tasks" | "requests" => Workspace(ViewMode::Tasks),
            "career" => Workspace(ViewMode::CareerSummary),
            _ => showcase,
        },
    }
}

fn open_all_sections(state: &mut GameplayState) {
    for section in [
        Disclosure::ResidentDetails,
        Disclosure::UnitDetails,
        Disclosure::Improvements,
        Disclosure::Staff,
    ] {
        state.panel_disclosure.toggle(section);
    }
}

/// The id of the showcase unit at `index` in building order.
fn unit_id(state: &GameplayState, index: usize) -> u32 {
    state.building.apartments[index].id
}

fn select_unit(state: &mut GameplayState, index: usize) {
    state.selection = Selection::Apartment(unit_id(state, index));
}

/// Empty a unit so its vacancy controls show.
fn vacate(state: &mut GameplayState, apartment_id: u32, listed: bool) {
    if let Some(apartment) = state.building.get_apartment_mut(apartment_id) {
        if let Some(tenant_id) = apartment.tenant_id {
            state.tenants.retain(|tenant| tenant.id != tenant_id);
        }
        apartment.move_out();
        apartment.is_listed_for_lease = listed;
    }
}

/// Scene-specific state on top of the base screen; keys without an entry
/// show the base screen unchanged.
fn adjust_scene(key: &str, state: &mut GameplayState) {
    match key {
        "condition_worn" | "condition_repaired" => {
            let condition = if key == "condition_worn" { 18 } else { 90 };
            state.building.hallway_condition = condition;
            for unit in &mut state.building.apartments {
                unit.condition = condition;
            }
        }
        "building_details" => state.selection = Selection::Building,
        "unit" => select_unit(state, 3),
        "unit_open" => {
            select_unit(state, 3);
            open_all_sections(state);
        }
        "unit_scrolled" => {
            select_unit(state, 3);
            open_all_sections(state);
            state.panel_scroll_offset = 240.0;
        }
        "unit_top" => select_unit(state, 5),
        // The showcase seeds its first unit at 18% with a pending request.
        "unit_damaged" => select_unit(state, 0),
        "unit_unaffordable" => {
            select_unit(state, 0);
            state.funds.balance = 40;
        }
        "unit_vacant" | "unit_listed" => {
            let id = unit_id(state, 1);
            vacate(state, id, key == "unit_listed");
            state.selection = Selection::Apartment(id);
        }
        "unit_sold" => {
            let id = unit_id(state, 2);
            vacate(state, id, false);
            state.building.convert_unit_to_condo(id, "Priya N.", 68_000);
            state.selection = Selection::Apartment(id);
        }
        "tower_tall_unit" => {
            let top = state
                .building
                .apartments
                .iter()
                .max_by_key(|unit| unit.floor);
            if let Some(top) = top {
                state.selection = Selection::Apartment(top.id);
            }
        }
        "hallway" => state.selection = Selection::Hallway,
        "hallway_open" => {
            state.selection = Selection::Hallway;
            open_all_sections(state);
        }
        "hallway_scrolled" => {
            state.selection = Selection::Hallway;
            open_all_sections(state);
            state.panel_scroll_offset = 240.0;
        }
        "applications" => state.selection = Selection::Applications(None),
        "applications_more" => {
            state.selection = Selection::Applications(None);
            state.application_page = 1;
        }
        "ownership" => state.selection = Selection::Ownership,
        "ownership_scrolled" => {
            state.selection = Selection::Ownership;
            state.panel_scroll_offset = 174.0;
        }
        "tenants_detail" => {
            state.selected_tenant_id = state.tenants.first().map(|tenant| tenant.id);
        }
        "tenants_more" => state.tenants_page = 1,
        "finances_scrolled" => state.finance_scroll_offset = 120.0,
        "city_filtered" => state.selected_neighborhood_id = Some(1),
        "market" => seed_showcase_market(state),
        "market_filtered" => {
            seed_showcase_market(state);
            state.selected_neighborhood_id = Some(1);
        }
        "purchase_review" => {
            seed_showcase_market(state);
            state.pending_property_purchase = Some(0);
        }
        "inbox_list" => state.selected_mail_id = None,
        "tasks_scrolled" => state.tasks_scroll_offset = 360.0,
        "requests_more" => {
            seed_showcase_requests(state);
            state.requests_page = 1;
        }
        "pause" => state.show_pause_menu = true,
        "pause_settings" => {
            state.show_pause_menu = true;
            state.pause_settings = Some(crate::state::SettingsScreen::new(
                crate::state::centered_settings_top,
            ));
        }
        "history" => {
            seed_showcase_activity(state);
            state.activity_drawer_open = true;
        }
        "career" => seed_showcase_career(state),
        "career_more" => {
            seed_showcase_career(state);
            state.career_badges_page = 1;
        }
        "tutorial_coach" => state.tutorial.pending_messages.clear(),
        "event" => seed_showcase_event(state),
        "event_notice" => seed_showcase_notice(state),
        "notification" => {
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
        let key = scene_key(scene);
        match capture_base(key) {
            CaptureBase::ResidentGallery => {
                self.capture_resident_gallery = true;
                return;
            }
            CaptureBase::Menu => self.state = GameState::Menu(Box::new(MenuState::new())),
            CaptureBase::MenuScreen => {
                self.state = GameState::Menu(Box::new(MenuState::for_capture(key)))
            }
            CaptureBase::Gameplay { showcase, template } => {
                self.seed_gameplay_capture(showcase, template)
            }
            CaptureBase::Workspace(view_mode) => {
                self.seed_gameplay_capture(true, 0);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.view_mode = view_mode;
                }
            }
        }
        if let GameState::Gameplay(state) = &mut self.state {
            adjust_scene(key, state);
        }
    }

    fn seed_gameplay_capture(&mut self, showcase: bool, template: usize) {
        let template = crate::data::templates::load_templates()
            .and_then(|templates| templates.templates.into_iter().nth(template));
        let Some(template) = template else {
            self.state = GameState::Menu(Box::new(MenuState::new()));
            return;
        };
        // Capture scenes are regression evidence, so their residents, events,
        // applications, and decorative activity must not drift between runs.
        let mut state =
            GameplayState::new_with_template_seed(self.config.clone(), template, 0x5EC0_0D57);
        if showcase {
            seed_showcase_residents(&mut state);
        }
        self.state = GameState::Gameplay(Box::new(state));
    }
}

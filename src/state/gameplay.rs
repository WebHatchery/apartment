//! The gameplay save-state: everything one campaign session owns.

use crate::building::Building;
use crate::data::config::GameConfig;
use crate::economy::{FinancialLedger, PlayerFunds};
use crate::simulation::{ActiveWorldEvent, EventLog, GameOutcome, TickResult};
use crate::tenant::{Tenant, TenantApplication};
use crate::ui::{FloatingTextLayer, Selection, UiAction};
use std::collections::{HashMap, HashSet};

// Phase 3 imports
use crate::city::City;
use crate::consequences::{ComplianceSystem, GentrificationTracker, TenantNetwork};
use crate::narrative::{
    load_events_config, load_relationship_config, Mailbox, MissionManager, NarrativeEventSystem,
    NotificationManager, RelationshipEventsConfig, TenantEventsConfig, TenantStory,
    TutorialManager,
};

use serde::{Deserialize, Serialize};

/// Floating text layer defaults, tuned to match the game's previous
/// hand-rolled `FloatingText` (1.5s life, 30px/s rise, quick velocity decay).
fn default_autosave_enabled() -> bool {
    true
}

pub(super) fn default_floating_text_layer() -> FloatingTextLayer {
    let mut layer = FloatingTextLayer::new();
    layer.default_lifetime = 1.5;
    layer.default_rise_speed = 30.0;
    layer.drag = 0.05;
    layer
}

/// View mode for the gameplay screen
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub(crate) enum ViewMode {
    #[default]
    Building, // Current single-building view
    Tenants,
    Finances,
    CityMap, // City overview with all neighborhoods
    Market,  // Property acquisition screen
    Mail,    // Mailbox view
    Tasks,
    CareerSummary, // Phase 5: Endgame result
}

#[derive(Serialize, Deserialize)]
pub struct GameplayState {
    // Phase 3: City replaces single building
    pub city: City,

    // Legacy field for backwards compatibility - now derived from city
    #[serde(skip)]
    pub building: Building,

    #[serde(skip)]
    pub config: GameConfig,

    // Tenants
    pub tenants: Vec<Tenant>,
    pub applications: Vec<TenantApplication>,
    pub next_tenant_id: u32,

    // Economy
    pub funds: PlayerFunds,
    pub ledger: FinancialLedger,

    // Simulation
    pub event_log: EventLog,
    pub current_tick: u32,
    pub game_outcome: Option<GameOutcome>,
    pub last_tick_result: Option<TickResult>,
    #[serde(default)]
    pub active_world_events: Vec<ActiveWorldEvent>,

    // Phase 3: Consequence systems
    pub tenant_network: TenantNetwork,
    pub compliance: ComplianceSystem,
    pub gentrification: GentrificationTracker,

    // Phase 3: Narrative systems
    pub narrative_events: NarrativeEventSystem,
    pub mailbox: Mailbox,
    pub tenant_stories: HashMap<u32, TenantStory>,
    pub dialogue_system: crate::narrative::DialogueSystem,
    #[serde(skip)]
    pub tenant_events_config: TenantEventsConfig,
    #[serde(skip)]
    pub relationship_events_config: RelationshipEventsConfig,

    // Phase 4: Tutorial & Missions
    pub tutorial: TutorialManager,
    pub missions: MissionManager,
    #[serde(default)]
    pub active_tax_breaks: Vec<crate::narrative::ActiveTaxBreak>,

    // Phase 5: Notifications (relationship changes, hints)
    pub notifications: NotificationManager,

    // Phase 5: Achievements
    pub achievements: crate::narrative::AchievementSystem,

    // UI state - skipped from serialization
    #[serde(skip)]
    pub view_mode: ViewMode,
    #[serde(skip)]
    pub selection: Selection,
    #[serde(skip)]
    pub pending_actions: Vec<UiAction>,
    #[serde(skip, default = "default_floating_text_layer")]
    pub floating_texts: FloatingTextLayer,
    #[serde(skip)]
    pub panel_scroll_offset: f32,
    #[serde(skip)]
    pub tower_scroll_offset: f32,
    /// Set when a place is selected so the tower scrolls it into view.
    #[serde(skip)]
    pub tower_focus_pending: bool,
    /// Canvas size the tower was last framed for; a resize reframes it.
    #[serde(skip)]
    pub framed_canvas: (f32, f32),
    #[serde(skip)]
    pub panel_disclosure: crate::ui::PanelDisclosure,
    #[serde(skip)]
    pub finance_scroll_offset: f32,
    #[serde(skip)]
    pub tasks_scroll_offset: f32,
    #[serde(skip)]
    pub event_scroll_offset: f32,
    #[serde(skip)]
    pub event_scroll_event_id: Option<u32>,
    #[serde(skip)]
    pub selected_mail_id: Option<u32>,
    #[serde(skip)]
    pub selected_tenant_id: Option<u32>,
    #[serde(skip)]
    pub inbox_page: usize,
    #[serde(skip)]
    pub tasks_page: usize,
    #[serde(skip)]
    pub city_scroll: f32,
    #[serde(skip)]
    pub market_page: usize,
    #[serde(skip)]
    pub selected_neighborhood_id: Option<u32>,
    #[serde(skip)]
    pub pending_property_purchase: Option<u32>,
    #[serde(skip)]
    pub application_page: usize,
    #[serde(skip)]
    pub tenants_page: usize,
    #[serde(skip)]
    pub requests_page: usize,
    #[serde(skip)]
    pub career_badges_page: usize,
    #[serde(skip)]
    pub show_pause_menu: bool,
    #[serde(skip)]
    pub activity_drawer_open: bool,
    /// Player preference from settings; applied on entering gameplay.
    #[serde(skip, default = "default_autosave_enabled")]
    pub autosave_enabled: bool,
    #[serde(skip)]
    pub pending_quit_to_menu: bool,
    #[serde(skip)]
    pub pending_load_game: bool,
    /// Settings page opened from the pause menu.
    #[serde(skip)]
    pub(crate) pause_settings: Option<super::settings_screen::SettingsScreen>,

    /// Current building template ID (for unlock tracking)
    #[serde(default)]
    pub current_building_id: String,

    /// Latches true once the building has ever housed a tenant. The "all tenants
    /// left" loss condition keys off this so it can't fire on a building that was
    /// simply never occupied yet.
    #[serde(default)]
    pub has_ever_had_tenant: bool,

    /// Building IDs that have genuinely been occupied. This replaces the
    /// legacy run-wide latch for per-building loss checks.
    #[serde(default)]
    pub ever_occupied_buildings: HashSet<u32>,

    /// True while a tenant council is organized. Latches when one forms (so its
    /// collective action applies once) and clears when conditions improve, so a
    /// council can re-form if the landlord backslides.
    #[serde(default)]
    pub council_formed: bool,

    /// The run's RNG seed, recorded so a run can be reproduced (bug reports,
    /// daily challenges) and re-applied on load so reloading doesn't reroll
    /// outcomes.
    #[serde(default)]
    pub seed: u64,
}

/// Pick a fresh run seed from wall-clock time. Uses macroquad's date source so
/// it works on both native and wasm (unlike `std::time`, which panics on wasm).
fn generate_run_seed() -> u64 {
    let now = macroquad::miniquad::date::now();
    ((now * 1_000_000.0) as u64) ^ 0x9E37_79B9_7F4A_7C15
}

impl GameplayState {
    /// Create a new game using the first configured building template.
    #[cfg(test)]
    pub fn new() -> Self {
        let config = crate::data::config::load_config();
        let template = crate::data::templates::load_templates()
            .and_then(|templates| templates.templates.into_iter().next())
            .unwrap_or_else(default_starter_template);

        Self::new_with_template(config, template)
    }

    /// Create a new game with a specific building template, choosing a fresh
    /// run seed from wall-clock entropy. Every game therefore differs (the RNG
    /// was previously never seeded, so all playthroughs were identical), and the
    /// chosen seed is recorded for reproducibility / bug reports.
    pub fn new_with_template(
        config: GameConfig,
        template: crate::data::templates::BuildingTemplate,
    ) -> Self {
        Self::new_with_template_seed(config, template, generate_run_seed())
    }

    /// Create a new game with a specific building template and an explicit run
    /// seed. Two games created from the same (config, template, seed) produce
    /// the same randomness — the basis for reproducible runs and daily
    /// challenges.
    pub fn new_with_template_seed(
        mut config: GameConfig,
        template: crate::data::templates::BuildingTemplate,
        seed: u64,
    ) -> Self {
        use crate::building::Building;

        crate::release_mode::apply_config(&mut config);

        // Seed the shared RNG before any generation so the run is reproducible
        // from `seed`.
        macroquad_toolkit::rng::srand(seed);

        // Apply the tier's rule modifiers (fines, inspections, problem tenants,
        // overhead) and derive its starting funds — this is what makes the three
        // property tiers genuinely different games, not just different sizes.
        let starting_funds = config.apply_difficulty(&template.difficulty);

        // Create building from template
        let building = Building::from_template(&template);
        let building_id = template.id.clone();

        // Place the building in its campaign neighborhood (falls back to a bare
        // slot if that neighborhood is full/missing).
        let mut city = City::new("Metropolis");
        let neighborhood_id = template.neighborhood_id;
        let starter_building_index = city
            .add_building(building.clone(), neighborhood_id)
            .unwrap_or_else(|_| {
                let index = city.buildings.len() as u32;
                city.buildings.push(building.clone());
                city.total_buildings_managed += 1;
                index
            });
        city.active_building_index = starter_building_index as usize;

        // Historic-quarter buildings carry preservation regulations.
        let is_historic = city
            .neighborhoods
            .iter()
            .find(|n| n.id == neighborhood_id)
            .map(|n| n.is_historic())
            .unwrap_or(false);

        // Initialize compliance
        let mut compliance = ComplianceSystem::new();
        compliance.init_building_regulations(starter_building_index, is_historic);

        let mut state = Self {
            city,
            building,
            config,
            tenants: Vec::new(),
            applications: Vec::new(),
            next_tenant_id: 1,
            funds: PlayerFunds::new(starting_funds),
            ledger: FinancialLedger::default(),
            event_log: EventLog::new(),
            current_tick: 0,
            game_outcome: None,
            last_tick_result: None,
            active_world_events: Vec::new(),

            tenant_network: TenantNetwork::new(),
            compliance,
            gentrification: GentrificationTracker::new(),
            narrative_events: NarrativeEventSystem::new(),
            mailbox: Mailbox::new(),
            tenant_stories: HashMap::new(),
            dialogue_system: crate::narrative::DialogueSystem::new(),
            tenant_events_config: load_events_config(),
            relationship_events_config: load_relationship_config(),

            tutorial: TutorialManager::new(),
            missions: MissionManager::new(),
            active_tax_breaks: Vec::new(),
            notifications: NotificationManager::new(),
            achievements: crate::narrative::AchievementSystem::new(),

            view_mode: ViewMode::Building,
            selection: Selection::None,
            pending_actions: Vec::new(),
            floating_texts: default_floating_text_layer(),
            panel_scroll_offset: 0.0,
            tower_scroll_offset: 0.0,
            tower_focus_pending: false,
            framed_canvas: (0.0, 0.0),
            panel_disclosure: crate::ui::PanelDisclosure::default(),
            finance_scroll_offset: 0.0,
            tasks_scroll_offset: 0.0,
            event_scroll_offset: 0.0,
            event_scroll_event_id: None,
            selected_mail_id: None,
            selected_tenant_id: None,
            inbox_page: 0,
            tasks_page: 0,
            city_scroll: 0.0,
            market_page: 0,
            selected_neighborhood_id: None,
            pending_property_purchase: None,
            application_page: 0,
            tenants_page: 0,
            requests_page: 0,
            career_badges_page: 0,
            show_pause_menu: false,
            activity_drawer_open: false,
            autosave_enabled: true,
            pending_quit_to_menu: false,
            pending_load_game: false,
            pause_settings: None,
            current_building_id: building_id,
            has_ever_had_tenant: false,
            ever_occupied_buildings: HashSet::new(),
            council_formed: false,
            seed,
        };

        // Handle initial tenant if present in template
        if let Some(data) = &template.initial_tenant {
            if let Some(archetype) = crate::tenant::TenantArchetype::from_id(&data.archetype) {
                if let Some(apt) = state
                    .building
                    .apartments
                    .iter_mut()
                    .find(|a| a.unit_number == data.apartment_unit)
                {
                    let tenant_id = state.next_tenant_id;
                    state.next_tenant_id += 1;

                    let mut tenant = Tenant::new(tenant_id, &data.name, archetype);
                    tenant.move_into_building(starter_building_index, apt.id);
                    apt.move_in(tenant_id);

                    let story = TenantStory::generate(tenant_id, &tenant.archetype);
                    state.tenant_stories.insert(tenant_id, story);
                    state.tenants.push(tenant);

                    if let Some(city_building) = state.city.active_building_mut() {
                        if let Some(city_apt) =
                            city_building.apartments.iter_mut().find(|a| a.id == apt.id)
                        {
                            city_apt.move_in(tenant_id);
                        }
                    }
                }
            }
        }

        // Generate initial applications (neutral reputation at game start).
        state.applications = crate::tenant::generate_applications(
            starter_building_index,
            &state.building,
            &[],
            0,
            &mut state.next_tenant_id,
            1.0,
            &state.config,
        );

        state.missions.generate_available_missions(0);
        state
            .tutorial
            .set_resident_baseline(state.active_tenant_count());

        if state.active_tenant_count() > 0 {
            state.has_ever_had_tenant = true;
            state.ever_occupied_buildings.insert(starter_building_index);
        }

        state
    }

    /// Adopt the player's saved gameplay preferences (fullscreen is applied
    /// to the window directly by the settings page).
    pub fn apply_settings(&mut self, settings: &macroquad_toolkit::settings::GameSettings) {
        self.autosave_enabled = settings.autosave_enabled;
    }

    pub fn active_building_id(&self) -> u32 {
        self.city.active_building_index as u32
    }

    pub fn active_tenant_count(&self) -> usize {
        let building_id = self.active_building_id();
        self.tenants
            .iter()
            .filter(|tenant| tenant.building_id == building_id)
            .count()
    }

    pub(super) fn active_tenants_cloned(&self) -> Vec<Tenant> {
        let building_id = self.active_building_id();
        self.tenants
            .iter()
            .filter(|tenant| tenant.building_id == building_id)
            .cloned()
            .collect()
    }

    /// Save the current `building` state back to the city
    pub fn save_building_to_city(&mut self) {
        if let Some(city_building) = self.city.active_building_mut() {
            *city_building = self.building.clone();
        }
    }

    /// Sync the `building` field with the active city building
    pub fn sync_building(&mut self) {
        if let Some(b) = self.city.active_building() {
            self.building = b.clone();
        }
    }

    /// Unlock a specific building (by its template `unlock_order`) in the
    /// persistent player progress — used by `MissionReward::UnlockBuilding`.
    pub(super) fn unlock_building_by_order(&self, unlock_order: u32) -> Result<(), String> {
        let templates = crate::data::templates::load_templates();
        crate::save::update_player_progress(|progress| {
            if let Some(template) = templates
                .iter()
                .flat_map(|templates| &templates.templates)
                .find(|t| t.unlock_order == unlock_order)
            {
                progress.unlock_building(&template.id);
            }
        })
    }

    /// Mark the current building completed and unlock the next one in
    /// `unlock_order`.
    pub fn unlock_next_building(&self) -> Result<(), String> {
        let templates = crate::data::templates::load_templates();
        crate::save::update_player_progress(|progress| {
            progress.mark_completed(&self.current_building_id);
            let Some(templates) = &templates else {
                return;
            };
            let current_order = templates
                .templates
                .iter()
                .find(|t| t.id == self.current_building_id)
                .map(|t| t.unlock_order)
                .unwrap_or(0);
            if let Some(next_template) = templates
                .templates
                .iter()
                .find(|t| t.unlock_order == current_order + 1)
            {
                progress.unlock_building(&next_template.id);
            }
        })
    }
}

#[cfg(test)]
fn default_starter_template() -> crate::data::templates::BuildingTemplate {
    use crate::data::templates::{ApartmentTemplate, BuildingTemplate};

    BuildingTemplate {
        id: "starter".to_string(),
        name: "Starter Building".to_string(),
        unlock_order: 0,
        difficulty: "easy".to_string(),
        neighborhood_id: 1,
        description: "A small starter property.".to_string(),
        floors: 2,
        units_per_floor: 2,
        hallway_condition: 60,
        apartments: vec![
            ApartmentTemplate {
                unit_number: "1A".to_string(),
                floor: 1,
                size_str: "small".to_string(),
                base_noise_str: "high".to_string(),
                initial_condition: 55,
                initial_design: "bare".to_string(),
                initial_rent: 600,
            },
            ApartmentTemplate {
                unit_number: "1B".to_string(),
                floor: 1,
                size_str: "medium".to_string(),
                base_noise_str: "low".to_string(),
                initial_condition: 60,
                initial_design: "bare".to_string(),
                initial_rent: 800,
            },
            ApartmentTemplate {
                unit_number: "2A".to_string(),
                floor: 2,
                size_str: "small".to_string(),
                base_noise_str: "low".to_string(),
                initial_condition: 65,
                initial_design: "practical".to_string(),
                initial_rent: 650,
            },
            ApartmentTemplate {
                unit_number: "2B".to_string(),
                floor: 2,
                size_str: "medium".to_string(),
                base_noise_str: "low".to_string(),
                initial_condition: 65,
                initial_design: "practical".to_string(),
                initial_rent: 850,
            },
        ],
        initial_tenant: None,
    }
}

use crate::assets::AssetManager;
use crate::building::DesignType;
use crate::data::config::{load_config, GameConfig};
use crate::state::{GameState, MenuState, StateTransition};
use crate::tenant::{Tenant, TenantArchetype};

pub struct Game {
    pub state: GameState,
    pub config: GameConfig,
    pub assets: AssetManager,
}

impl Game {
    pub async fn new() -> Self {
        let mut assets = AssetManager::new();
        assets.load_assets().await;

        let config = load_config();

        Self {
            state: GameState::Menu(MenuState::new()),
            config,
            assets,
        }
    }

    pub fn update(&mut self) {
        let transition = match &mut self.state {
            GameState::Menu(s) => s.update(&self.assets, &self.config),
            GameState::Gameplay(s) => s.update(&self.assets),
        };

        if let Some(t) = transition {
            self.transition(t);
        }
    }

    pub fn draw(&mut self) {
        match &mut self.state {
            GameState::Menu(s) => s.draw(&self.assets),
            GameState::Gameplay(s) => s.draw(&self.assets),
        }
    }

    fn transition(&mut self, transition: StateTransition) {
        self.state = match transition {
            StateTransition::ToMenu => GameState::Menu(MenuState::new()),
            StateTransition::ToGameplay(s) => GameState::Gameplay(s),
        };
    }

    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "menu" => self.state = GameState::Menu(MenuState::new()),
            "showcase" => self.seed_gameplay_capture(true),
            "gameplay" => self.seed_gameplay_capture(false),
            _ => {
                self.seed_gameplay_capture(false);
            }
        }
    }

    fn seed_gameplay_capture(&mut self, showcase: bool) {
        let template = crate::data::templates::load_templates()
            .and_then(|templates| templates.templates.into_iter().next());
        let Some(template) = template else {
            self.state = GameState::Menu(MenuState::new());
            return;
        };
        let mut state =
            crate::state::GameplayState::new_with_template(self.config.clone(), template);
        if showcase {
            seed_showcase_residents(&mut state);
        }
        self.state = GameState::Gameplay(state);
    }
}

fn seed_showcase_residents(state: &mut crate::state::GameplayState) {
    let identities = [
        ("Maya R.", TenantArchetype::Artist, 8),
        ("Jon Bell", TenantArchetype::Student, 28),
        ("The Garcias", TenantArchetype::Family, 52),
        ("Sarah M.", TenantArchetype::Professional, 74),
        ("Walter H.", TenantArchetype::Elderly, 94),
        ("Alex Chen", TenantArchetype::Student, 68),
    ];
    let designs = [
        DesignType::Bare,
        DesignType::Practical,
        DesignType::Cozy,
        DesignType::Practical,
        DesignType::Cozy,
        DesignType::Bare,
    ];
    let conditions = [18, 46, 78, 90, 96, 62];
    let building_id = state.active_building_id();
    state.tenants.clear();

    for (index, apartment) in state.building.apartments.iter_mut().enumerate() {
        let (name, archetype, happiness) = &identities[index % identities.len()];
        let tenant_id = index as u32 + 1;
        let mut tenant = Tenant::new(tenant_id, name, archetype.clone());
        tenant.happiness = *happiness;
        tenant.move_into_building(building_id, apartment.id);
        apartment.move_in(tenant_id);
        apartment.design = designs[index % designs.len()].clone();
        apartment.condition = conditions[index % conditions.len()];
        apartment.kitchen_level = if index == 1 || index == 3 { 2 } else { 0 };
        apartment.has_soundproofing = index == 4;
        if index == 3 || index == 4 {
            apartment.flags.insert("has_better_lighting".to_string());
        }
        state.tenants.push(tenant);
    }
    state.next_tenant_id = state.tenants.len() as u32 + 1;
    state.current_tick = 10;
    state.building.hallway_condition = 92;
    state.building.has_laundry = true;
    state.building.flags.insert("has_laundry".to_string());
    state
        .building
        .flags
        .insert("staff_receptionist".to_string());
    state.building.flags.insert("staff_security".to_string());
    state.tutorial.active = false;
    state.tutorial.current_milestone = None;
    state.tutorial.pending_messages.clear();
    state.has_ever_had_tenant = true;
    state.ever_occupied_buildings.insert(building_id);
    if let Some(city_building) = state.city.active_building_mut() {
        *city_building = state.building.clone();
    }
}

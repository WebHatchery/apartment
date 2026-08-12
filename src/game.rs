use crate::assets::AssetManager;
use crate::building::DesignType;
use crate::data::config::{load_config, GameConfig};
use crate::economy::{Transaction, TransactionType};
use crate::narrative::{MailItem, TenantRequest, TenantStory};
use crate::state::{GameState, MenuState, StateTransition, ViewMode};
use crate::tenant::{Tenant, TenantApplication, TenantArchetype};

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
            "showcase" | "showcase_compact" => self.seed_gameplay_capture(true),
            "unit_showcase" | "unit_compact" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Apartment(3);
                }
            }
            "unit_scrolled" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Apartment(3);
                    state.panel_scroll_offset = 240.0;
                }
            }
            "hallway_showcase" | "hallway_compact" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Hallway;
                }
            }
            "hallway_scrolled" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Hallway;
                    state.panel_scroll_offset = 240.0;
                }
            }
            "applications_showcase" | "applications_compact" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Applications(None);
                }
            }
            "tenants_showcase" | "tenants_compact" => {
                self.seed_workspace_capture(ViewMode::Tenants);
            }
            "finances_showcase" | "finances_compact" => {
                self.seed_workspace_capture(ViewMode::Finances);
            }
            "city_showcase" | "city_compact" => {
                self.seed_workspace_capture(ViewMode::CityMap);
            }
            "inbox_showcase" | "inbox_compact" => {
                self.seed_workspace_capture(ViewMode::Mail);
            }
            "tasks_showcase" | "tasks_compact" => {
                self.seed_workspace_capture(ViewMode::Tasks);
            }
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

    fn seed_workspace_capture(&mut self, view_mode: ViewMode) {
        self.seed_gameplay_capture(true);
        if let GameState::Gameplay(state) = &mut self.state {
            state.view_mode = view_mode;
        }
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
    state.tenant_stories.clear();

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
        state
            .tenant_stories
            .insert(tenant_id, TenantStory::generate(tenant_id, archetype));
    }
    state.next_tenant_id = state.tenants.len() as u32 + 1;
    if let Some(story) = state.tenant_stories.get_mut(&1) {
        story.pending_request = Some(TenantRequest::Pet {
            pet_type: "small rescue dog".to_string(),
        });
    }
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
    state.applications = [
        (20, "Nina Brooks", TenantArchetype::Artist, 0, true, false),
        (
            21,
            "David Okafor",
            TenantArchetype::Professional,
            1,
            false,
            false,
        ),
    ]
    .into_iter()
    .filter_map(
        |(id, name, archetype, apartment_index, credit, background)| {
            let apartment = state.building.apartments.get(apartment_index)?;
            let mut tenant = Tenant::new(id, name, archetype);
            tenant.happiness = if id % 2 == 0 { 82 } else { 34 };
            let result = crate::tenant::matching::calculate_match_score(
                &tenant,
                apartment,
                &state.config.matching,
            );
            let mut application =
                TenantApplication::new_for_building(tenant, building_id, apartment.id, result, 10);
            application.revealed_reliability = credit;
            application.revealed_behavior = background;
            Some(application)
        },
    )
    .collect();
    seed_showcase_finances(state);
    seed_showcase_mail(state);
}

fn seed_showcase_finances(state: &mut crate::state::GameplayState) {
    let months = [
        (7, 3_900, 800, 0, 6_100),
        (8, 4_200, 1_200, 800, 8_300),
        (9, 4_500, 900, 2_200, 9_700),
        (10, 4_750, 1_365, 1_000, 3_065),
    ];
    state.ledger.reports.clear();
    let mut transactions = Vec::new();
    for (month, income, operating, upgrades, ending_balance) in months {
        let mut monthly = vec![
            Transaction::income(TransactionType::RentIncome, income, "Rent collected", month),
            Transaction::expense(
                TransactionType::PropertyTax,
                operating,
                "Tax and operations",
                month,
            ),
        ];
        if upgrades > 0 {
            monthly.push(Transaction::expense(
                TransactionType::UpgradeCost,
                upgrades,
                "Property improvements",
                month,
            ));
        }
        let refs: Vec<_> = monthly.iter().collect();
        state.ledger.generate_report(month, &refs, ending_balance);
        transactions.extend(monthly);
    }
    state.funds.balance = 3_065;
    state.funds.total_income = transactions
        .iter()
        .filter(|transaction| transaction.amount > 0)
        .map(|transaction| transaction.amount)
        .sum();
    state.funds.total_expenses = transactions
        .iter()
        .filter(|transaction| transaction.amount < 0)
        .map(|transaction| transaction.amount.abs())
        .sum();
    state.funds.transactions = transactions;
}

fn seed_showcase_mail(state: &mut crate::state::GameplayState) {
    state.mailbox = crate::narrative::Mailbox::new();
    state.mailbox.receive(MailItem::tenant_letter(
        0,
        1,
        "Maya R.",
        10,
        "A little help with the kitchen",
        "Hi! The new lighting is wonderful. Could we talk about making room for a small rescue dog? I promise to keep the shared spaces tidy.",
    ));
    state
        .mailbox
        .receive(MailItem::financial_statement(0, 10, 4_750, 2_365, 2_385));
    state.mailbox.receive(MailItem::news_clipping(
        0,
        9,
        "Sunset block draws new neighbours",
        "Fresh paint, brighter windows and a busy lobby are changing how the neighbourhood sees Sunset Apartments.",
    ));
    state.selected_mail_id = Some(0);
}

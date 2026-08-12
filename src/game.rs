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
            "menu" | "menu_compact" | "menu_wide" | "menu_tiny" => {
                self.state = GameState::Menu(MenuState::new())
            }
            "showcase" | "showcase_compact" | "showcase_wide" | "showcase_tiny" => {
                self.seed_gameplay_capture(true)
            }
            "unit_showcase" | "unit_compact" | "unit_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Apartment(3);
                }
            }
            "unit_scrolled" | "unit_scrolled_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Apartment(3);
                    state.panel_scroll_offset = 240.0;
                }
            }
            "hallway_showcase" | "hallway_compact" | "hallway_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Hallway;
                }
            }
            "hallway_scrolled" | "hallway_scrolled_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Hallway;
                    state.panel_scroll_offset = 240.0;
                }
            }
            "applications_showcase" | "applications_compact" | "applications_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Applications(None);
                }
            }
            "applications_more_showcase" | "applications_more_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Applications(None);
                    state.application_page = 1;
                }
            }
            "ownership_showcase" | "ownership_compact" | "ownership_wide" | "ownership_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Ownership;
                }
            }
            "ownership_scrolled" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selection = crate::ui::Selection::Ownership;
                    state.panel_scroll_offset = 174.0;
                }
            }
            "tenants_showcase" | "tenants_compact" | "tenants_tiny" => {
                self.seed_workspace_capture(ViewMode::Tenants);
            }
            "tenants_more_showcase" | "tenants_more_tiny" => {
                self.seed_workspace_capture(ViewMode::Tenants);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.tenants_page = 1;
                }
            }
            "finances_showcase" | "finances_compact" | "finances_wide" | "finances_tiny" => {
                self.seed_workspace_capture(ViewMode::Finances);
            }
            "city_showcase" | "city_compact" | "city_wide" | "city_tiny" => {
                self.seed_workspace_capture(ViewMode::CityMap);
            }
            "city_filtered_showcase" | "city_filtered_tiny" => {
                self.seed_workspace_capture(ViewMode::CityMap);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.selected_neighborhood_id = Some(1);
                }
            }
            "market_showcase" | "market_compact" | "market_wide" | "market_tiny" => {
                self.seed_workspace_capture(ViewMode::Market);
                if let GameState::Gameplay(state) = &mut self.state {
                    seed_showcase_market(state);
                }
            }
            "market_filtered_showcase" | "market_filtered_tiny" => {
                self.seed_workspace_capture(ViewMode::Market);
                if let GameState::Gameplay(state) = &mut self.state {
                    seed_showcase_market(state);
                    state.selected_neighborhood_id = Some(1);
                }
            }
            "purchase_review_showcase" | "purchase_review_tiny" => {
                self.seed_workspace_capture(ViewMode::Market);
                if let GameState::Gameplay(state) = &mut self.state {
                    seed_showcase_market(state);
                    state.pending_property_purchase = Some(0);
                }
            }
            "inbox_showcase" | "inbox_compact" | "inbox_tiny" => {
                self.seed_workspace_capture(ViewMode::Mail);
            }
            "tasks_showcase" | "tasks_compact" | "tasks_tiny" => {
                self.seed_workspace_capture(ViewMode::Tasks);
            }
            "requests_more_showcase" | "requests_more_tiny" => {
                self.seed_workspace_capture(ViewMode::Tasks);
                if let GameState::Gameplay(state) = &mut self.state {
                    seed_showcase_requests(state);
                    state.requests_page = 1;
                }
            }
            "pause_showcase" | "pause_compact" | "pause_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.show_pause_menu = true;
                }
            }
            "history_showcase" | "history_compact" | "history_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    seed_showcase_activity(state);
                    state.activity_drawer_open = true;
                }
            }
            "career_showcase" | "career_compact" | "career_wide" | "career_tiny" => {
                self.seed_workspace_capture(ViewMode::CareerSummary);
                if let GameState::Gameplay(state) = &mut self.state {
                    seed_showcase_career(state);
                }
            }
            "career_more_showcase" | "career_more_tiny" => {
                self.seed_workspace_capture(ViewMode::CareerSummary);
                if let GameState::Gameplay(state) = &mut self.state {
                    seed_showcase_career(state);
                    state.career_badges_page = 1;
                }
            }
            "tutorial_showcase" | "tutorial_compact" | "tutorial_wide" | "tutorial_tiny" => {
                self.seed_gameplay_capture(false)
            }
            "tutorial_coach_showcase" | "tutorial_coach_compact" | "tutorial_coach_tiny" => {
                self.seed_gameplay_capture(false);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.tutorial.pending_messages.clear();
                }
            }
            "event_showcase" | "event_compact" | "event_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    seed_showcase_event(state);
                }
            }
            "event_notice_showcase" | "event_notice_compact" | "event_notice_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    seed_showcase_notice(state);
                }
            }
            "notification_showcase" | "notification_compact" | "notification_tiny" => {
                self.seed_gameplay_capture(true);
                if let GameState::Gameplay(state) = &mut self.state {
                    state.notifications.pending.push(
                        crate::narrative::notifications::GameNotification::positive(
                            "+",
                            "Maya R. and Jon Bell have become friends!",
                        ),
                    );
                    state.notifications.pending[0].description =
                        Some("Friendly neighbors boost each other's happiness.".to_string());
                }
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

fn seed_showcase_activity(state: &mut crate::state::GameplayState) {
    use crate::simulation::{GameEvent, NotificationLevel};

    state.event_log.log(
        GameEvent::Notification {
            message: "The Greenfield Heights block welcomed two new families.".to_string(),
            level: NotificationLevel::Info,
        },
        10,
    );
    state.event_log.log(
        GameEvent::UpgradeCompleted {
            description: "Warm hallway lighting".to_string(),
            cost: 1_000,
        },
        10,
    );
    state.event_log.log(
        GameEvent::RentPaid {
            tenant_name: "Maya R.".to_string(),
            amount: 600,
        },
        10,
    );
    state.event_log.log(
        GameEvent::NoiseComplaint {
            tenant_name: "Jon Bell".to_string(),
        },
        10,
    );
}

fn seed_showcase_career(state: &mut crate::state::GameplayState) {
    state.current_tick = 36;
    state.funds.balance = 32_750;
    for id in [
        "first_tenant",
        "full_house",
        "landlord",
        "well_regarded",
        "happy_home",
        "survivor",
    ] {
        state.achievements.unlock(id);
    }
}

fn seed_showcase_event(state: &mut crate::state::GameplayState) {
    use crate::narrative::events::{
        NarrativeChoice, NarrativeEffect, NarrativeEvent, NarrativeEventType,
    };

    let event = NarrativeEvent::with_choices(
        0,
        NarrativeEventType::NeighborhoodNews,
        state.current_tick,
        "A cafe for the empty storefront",
        "A local baker wants to turn the unused ground-floor storefront into a neighborhood cafe. A little help with the first fit-out could bring new life to the whole block.",
        vec![
            NarrativeChoice {
                label: "Offer a starter grant · $750".to_string(),
                description: "Help pay for counters, seating, and the first oven.".to_string(),
                effect: NarrativeEffect::Money { amount: -750 },
                reputation_change: 8,
            },
            NarrativeChoice {
                label: "Offer the space only".to_string(),
                description: "Lease the storefront without contributing to the fit-out."
                    .to_string(),
                effect: NarrativeEffect::None,
                reputation_change: 2,
            },
            NarrativeChoice {
                label: "Keep the storefront vacant".to_string(),
                description: "Decline the proposal and hold out for a different tenant."
                    .to_string(),
                effect: NarrativeEffect::None,
                reputation_change: -2,
            },
        ],
    );
    state.narrative_events.add_event(event);
}

fn seed_showcase_market(state: &mut crate::state::GameplayState) {
    use crate::city::{BuildingCondition, FinancingOption, PropertyListing};

    state.funds.balance = 420_000;
    let specs = [
        (
            "Juniper Court",
            0,
            BuildingCondition::Fair,
            3,
            2,
            285_000,
            3,
        ),
        (
            "Foundry House",
            2,
            BuildingCondition::Poor,
            4,
            2,
            218_000,
            2,
        ),
        ("Rose Terrace", 3, BuildingCondition::Good, 3, 3, 398_000, 5),
        (
            "Garden Mews",
            1,
            BuildingCondition::Excellent,
            2,
            3,
            465_000,
            4,
        ),
    ];
    state.city.market.listings = specs
        .into_iter()
        .enumerate()
        .map(
            |(index, (name, neighborhood, condition, floors, units, price, tenants))| {
                PropertyListing {
                    id: index as u32,
                    name: name.to_string(),
                    neighborhood_id: state.city.neighborhoods[neighborhood].id,
                    condition,
                    num_floors: floors,
                    units_per_floor: units,
                    asking_price: price,
                    existing_tenants: tenants,
                    months_on_market: index as u32 + 1,
                    available_financing: vec![
                        FinancingOption::Cash,
                        FinancingOption::Mortgage {
                            down_payment_percent: 0.2,
                            interest_rate: 0.06,
                            term_months: 120,
                        },
                    ],
                    notes: Vec::new(),
                }
            },
        )
        .collect();
}

fn seed_showcase_notice(state: &mut crate::state::GameplayState) {
    use crate::narrative::events::{NarrativeEvent, NarrativeEventType};

    let event = NarrativeEvent::with_choices(
        0,
        NarrativeEventType::SeasonalEvent,
        state.current_tick,
        "Heatwave watch issued",
        "The city expects several unusually hot days. Check vulnerable residents, keep shared water available, and plan for higher utility use this month.",
        Vec::new(),
    );
    state.narrative_events.add_event(event);
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
        (22, "Priya Shah", TenantArchetype::Student, 2, true, true),
        (23, "Elena Ruiz", TenantArchetype::Family, 3, false, true),
        (24, "George Wu", TenantArchetype::Elderly, 4, true, false),
        (
            25,
            "Amara Mensah",
            TenantArchetype::Professional,
            5,
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

fn seed_showcase_requests(state: &mut crate::state::GameplayState) {
    let requests = [
        TenantRequest::Pet {
            pet_type: "small rescue dog".to_string(),
        },
        TenantRequest::TemporaryGuest {
            guest_name: "aunt Lila".to_string(),
            duration_months: 2,
        },
        TenantRequest::HomeBusiness {
            business_type: "ceramics studio".to_string(),
        },
        TenantRequest::Modification {
            description: "install accessible handrails".to_string(),
        },
        TenantRequest::Sublease,
    ];
    for (tenant, request) in state.tenants.iter().zip(requests) {
        if let Some(story) = state.tenant_stories.get_mut(&tenant.id) {
            story.pending_request = Some(request);
        }
    }
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
    use crate::narrative::dialogue::{DialogueChoice, DialogueEffect, DialogueType};

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
    state.dialogue_system.add_dialogue(
        DialogueType::FaceToFaceRequest,
        1,
        None,
        "Quiet for late shifts",
        "Could we keep the laundry room quiet after ten? My early shifts start before sunrise.",
        vec![
            DialogueChoice {
                text: "Post quiet hours".to_string(),
                effects: vec![DialogueEffect::HappinessChange {
                    tenant_id: 1,
                    amount: 6,
                }],
            },
            DialogueChoice {
                text: "Leave it informal".to_string(),
                effects: vec![DialogueEffect::HappinessChange {
                    tenant_id: 1,
                    amount: -3,
                }],
            },
        ],
        None,
    );
}

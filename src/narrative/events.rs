use macroquad_toolkit::rng;
use serde::{Deserialize, Serialize};

/// Types of narrative events
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum NarrativeEventType {
    /// News about the neighborhood
    NeighborhoodNews,
    /// City-wide event affecting economy
    CityEvent,
    /// Tenant-specific story beat
    TenantStory { tenant_id: u32 },
    /// Building milestone
    BuildingMilestone,
    /// Random character encounter
    CharacterEncounter,
    /// External offer (developer wants to buy, investor interest)
    ExternalOffer,
    /// Seasonal event
    SeasonalEvent,
    /// Relationship event (hostile/friendly interaction)
    RelationshipEvent,
}

/// A narrative event with context and choices
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NarrativeEvent {
    pub id: u32,
    pub event_type: NarrativeEventType,
    pub month: u32,
    pub headline: String,
    pub description: String,
    /// Optional choices the player can make
    pub choices: Vec<NarrativeChoice>,
    /// Effect if no choice is made (or no choices available)
    pub default_effect: NarrativeEffect,
    /// Has this been seen by the player?
    pub read: bool,
    /// Does this require a response?
    pub requires_response: bool,
    /// Deadline month for response (if applicable)
    pub response_deadline: Option<u32>,
    /// Optional related neighborhood ID
    pub related_neighborhood_id: Option<u32>,
}

/// A choice for a narrative event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NarrativeChoice {
    pub label: String,
    pub description: String,
    pub effect: NarrativeEffect,
    pub reputation_change: i32,
}

/// The full outcome of resolving an event choice: the gameplay effect plus the
/// reputation change and which neighborhood it applies to.
#[derive(Clone, Debug)]
pub struct ChoiceOutcome {
    pub effect: NarrativeEffect,
    pub reputation_change: i32,
    pub neighborhood_id: Option<u32>,
}

/// Effects of narrative events/choices
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NarrativeEffect {
    /// No gameplay effect
    None,
    /// Money gained or lost
    Money { amount: i32 },
    /// Reputation in neighborhood
    NeighborhoodReputation { neighborhood_id: u32, change: i32 },
    /// Building-wide happiness change
    BuildingHappiness { building_id: u32, change: i32 },
    /// Specific tenant happiness
    TenantHappiness { tenant_id: u32, change: i32 },
    /// Economic change
    EconomyChange { economy_health_change: f32 },
    /// Rent demand change
    RentDemand { neighborhood_id: u32, change: f32 },
    /// Trigger an inspection
    TriggerInspection { building_id: u32 },
    /// Property value change
    PropertyValue {
        building_id: u32,
        change_percent: f32,
    },
    /// Change relationship strength between tenants
    RelationshipStrength {
        tenant_a_id: u32,
        tenant_b_id: u32,
        change: i32,
    },
    /// Change landlord opinion (how much tenant likes player)
    OpinionChange { tenant_id: u32, amount: i32 },
    /// Tenant moves out
    MoveOut { tenant_id: u32 },
    /// Sell the building (Game Over / Victory)
    SellBuilding { building_id: u32 },
    /// Multiple effects
    Multiple { effects: Vec<NarrativeEffect> },
}

impl NarrativeEvent {
    /// Create a simple news event
    pub fn news(id: u32, month: u32, headline: &str, description: &str) -> Self {
        Self {
            id,
            event_type: NarrativeEventType::NeighborhoodNews,
            month,
            headline: headline.to_string(),
            description: description.to_string(),
            choices: Vec::new(),
            default_effect: NarrativeEffect::None,
            read: false,
            requires_response: false,
            response_deadline: None,
            related_neighborhood_id: None,
        }
    }

    /// Create an event with choices
    pub fn with_choices(
        id: u32,
        event_type: NarrativeEventType,
        month: u32,
        headline: &str,
        description: &str,
        choices: Vec<NarrativeChoice>,
    ) -> Self {
        Self {
            id,
            event_type,
            month,
            headline: headline.to_string(),
            description: description.to_string(),
            choices,
            default_effect: NarrativeEffect::None,
            read: false,
            requires_response: true,
            response_deadline: Some(month + 2), // 2 months to respond
            related_neighborhood_id: None,
        }
    }

    /// Check if deadline has passed
    pub fn is_expired(&self, current_month: u32) -> bool {
        self.response_deadline
            .map(|d| current_month > d)
            .unwrap_or(false)
    }
}

/// Manages narrative events
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NarrativeEventSystem {
    pub events: Vec<NarrativeEvent>,
    pub next_event_id: u32,
    /// Events awaiting player response
    pub pending_events: Vec<u32>,
    /// Processed event IDs
    pub processed_events: Vec<u32>,
}

impl NarrativeEventSystem {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            next_event_id: 0,
            pending_events: Vec::new(),
            processed_events: Vec::new(),
        }
    }

    /// Add a new event
    pub fn add_event(&mut self, mut event: NarrativeEvent) -> u32 {
        let id = self.next_event_id;
        event.id = id;
        self.next_event_id += 1;

        if event.requires_response {
            self.pending_events.push(id);
        }

        self.events.push(event);
        id
    }

    /// Process a choice for an event
    pub fn process_choice(&mut self, event_id: u32, choice_index: usize) -> Option<ChoiceOutcome> {
        let event = self.events.iter_mut().find(|e| e.id == event_id)?;
        let neighborhood_id = event.related_neighborhood_id;
        let (effect, reputation_change) = if event.choices.is_empty() {
            (event.default_effect.clone(), 0)
        } else {
            let choice = event.choices.get(choice_index)?;
            (choice.effect.clone(), choice.reputation_change)
        };

        event.read = true;
        self.pending_events.retain(|&id| id != event_id);
        self.processed_events.push(event_id);

        Some(ChoiceOutcome {
            effect,
            reputation_change,
            neighborhood_id,
        })
    }

    /// Consume non-interactive news exactly once. These events describe world
    /// changes rather than player decisions, so their default effect applies
    /// immediately and the headline is returned for the visible event log.
    pub fn process_immediate_events(&mut self) -> Vec<(String, NarrativeEffect)> {
        let ids: Vec<u32> = self
            .events
            .iter()
            .filter(|event| {
                !event.requires_response
                    && matches!(
                        event.event_type,
                        NarrativeEventType::NeighborhoodNews
                            | NarrativeEventType::CityEvent
                            | NarrativeEventType::SeasonalEvent
                            | NarrativeEventType::BuildingMilestone
                    )
                    && !self.processed_events.contains(&event.id)
            })
            .map(|event| event.id)
            .collect();

        let mut outcomes = Vec::new();
        for id in ids {
            if let Some(event) = self.events.iter_mut().find(|event| event.id == id) {
                event.read = true;
                self.processed_events.push(id);
                outcomes.push((event.headline.clone(), event.default_effect.clone()));
            }
        }
        outcomes
    }

    /// Mark one authored non-response event handled and return its default
    /// effect. Callers that generate special event categories can use this to
    /// apply the effect immediately without leaving stale processed state.
    pub fn process_default_now(&mut self, event_id: u32) -> Option<NarrativeEffect> {
        let event = self.events.iter_mut().find(|event| event.id == event_id)?;
        if event.requires_response || self.processed_events.contains(&event_id) {
            return None;
        }
        event.read = true;
        self.processed_events.push(event_id);
        Some(event.default_effect.clone())
    }

    /// Expire an event and return the default consequence for no response.
    pub fn expire_event(&mut self, event_id: u32) -> Option<NarrativeEffect> {
        let event = self.events.iter_mut().find(|e| e.id == event_id)?;

        event.read = true;
        self.pending_events.retain(|&id| id != event_id);
        self.processed_events.push(event_id);

        Some(event.default_effect.clone())
    }

    /// Expire all overdue response events and return their consequences.
    pub fn expire_due_events(&mut self, current_month: u32) -> Vec<NarrativeEffect> {
        let expired: Vec<u32> = self
            .events
            .iter()
            .filter(|e| e.is_expired(current_month) && !self.processed_events.contains(&e.id))
            .map(|e| e.id)
            .collect();

        expired
            .into_iter()
            .filter_map(|id| self.expire_event(id))
            .collect()
    }

    /// Generate random events based on game state
    pub fn generate_events(
        &mut self,
        month: u32,
        neighborhoods: &[crate::city::Neighborhood],
        buildings: &[crate::building::Building],
        _tenants: &[crate::tenant::Tenant],
    ) {
        // News event copy + effects are data-driven (assets/news_events.json).
        let news = load_news_events();

        // Chance for neighborhood news
        if rng::gen_range(0, 100) < 20 {
            if let Some(neighborhood) = rng::choose(neighborhoods) {
                let event = Self::neighborhood_event(&news, month, neighborhood);
                self.add_event(event);
            }
        }

        // Chance for city-wide event
        if rng::gen_range(0, 100) < 10 {
            let event = Self::city_event(&news, month);
            self.add_event(event);
        }

        // Seasonal events
        let season = (month % 12) / 3; // 0=spring, 1=summer, 2=fall, 3=winter
        if rng::gen_range(0, 100) < 15 {
            let event = Self::seasonal_event(&news, month, season);
            self.add_event(event);
        }

        // Developer/investor offers (rare)
        if rng::gen_range(0, 100) < 5 && !buildings.is_empty() {
            if let Some(building) = rng::choose(buildings) {
                let building_id = buildings
                    .iter()
                    .position(|b| std::ptr::eq(b, building))
                    .unwrap_or(0) as u32;
                let event = self.generate_offer_event(month, building_id, building);
                self.add_event(event);
            }
        }

        // Building milestones
        for building in buildings.iter() {
            if building.has_full_rental_occupancy() && rng::gen_range(0, 100) < 30 {
                let event = NarrativeEvent::news(
                    0,
                    month,
                    &format!("{} Achieves Full Occupancy!", building.name),
                    "All units are now occupied. Your reputation is growing.",
                );
                self.add_event(event);
            }
        }

        // Expiration effects are applied by gameplay state after generation.
    }

    fn neighborhood_event(
        news: &NewsEventsConfig,
        month: u32,
        neighborhood: &crate::city::Neighborhood,
    ) -> NarrativeEvent {
        if let Some(template) = rng::choose(&news.neighborhood) {
            let mut event =
                NarrativeEvent::news(0, month, &template.headline, &template.description);
            event.default_effect = template.effect.to_effect(neighborhood.id);
            event.related_neighborhood_id = Some(neighborhood.id);
            event
        } else {
            NarrativeEvent::news(0, month, "Neighborhood Update", "No local news this month.")
        }
    }

    fn city_event(news: &NewsEventsConfig, month: u32) -> NarrativeEvent {
        let mut event = if let Some(template) = rng::choose(&news.city) {
            let mut event =
                NarrativeEvent::news(0, month, &template.headline, &template.description);
            // City effects are neighborhood-independent, so the id is unused.
            event.default_effect = template.effect.to_effect(0);
            event
        } else {
            NarrativeEvent::news(0, month, "City Update", "No major city news this month.")
        };
        event.event_type = NarrativeEventType::CityEvent;
        event
    }

    fn seasonal_event(news: &NewsEventsConfig, month: u32, season: u32) -> NarrativeEvent {
        // Pick at random among the templates tagged for the current season, so
        // the same seasonal beat doesn't recur every single year.
        let candidates: Vec<&NewsTemplate> = news
            .seasonal
            .iter()
            .filter(|t| t.season == season)
            .collect();
        let mut event = match rng::choose(&candidates) {
            Some(template) => {
                let mut event =
                    NarrativeEvent::news(0, month, &template.headline, &template.description);
                event.default_effect = template.effect.to_effect(0);
                event
            }
            None => NarrativeEvent::news(0, month, "Seasonal Update", "The seasons turn."),
        };
        event.event_type = NarrativeEventType::SeasonalEvent;
        event
    }

    fn generate_offer_event(
        &self,
        month: u32,
        building_id: u32,
        building: &crate::building::Building,
    ) -> NarrativeEvent {
        let base_value = 50000 * building.apartments.len() as i32;
        // Increased offer multiplier to 2.5x - 4.0x base value to be "worth it"
        let offer = (base_value as f32 * rng::gen_range(2.5, 4.0)) as i32;

        // Countering is a gamble decided now: usually the developer sweetens the
        // deal ~25%, but sometimes they walk and the sale is off (you keep the
        // building). Baked at generation since a choice resolves to one effect.
        let counter_succeeds = rng::gen_range(0.0, 1.0) < 0.6;
        let counter_amount = (offer as f32 * 1.25) as i32;
        let counter_effect = if counter_succeeds {
            NarrativeEffect::Multiple {
                effects: vec![
                    NarrativeEffect::Money {
                        amount: counter_amount,
                    },
                    NarrativeEffect::SellBuilding { building_id },
                ],
            }
        } else {
            // Developer walks away — no sale this round.
            NarrativeEffect::None
        };

        NarrativeEvent::with_choices(
            0,
            NarrativeEventType::ExternalOffer,
            month,
            "Developer Makes Offer",
            &format!(
                "A developer has expressed interest in purchasing {} for ${}.",
                building.name, offer
            ),
            vec![
                NarrativeChoice {
                    label: "Accept Offer".to_string(),
                    description: format!("Sell the building for ${}", offer),
                    effect: NarrativeEffect::Multiple {
                        effects: vec![
                            NarrativeEffect::Money { amount: offer },
                            NarrativeEffect::SellBuilding { building_id },
                        ],
                    },
                    reputation_change: -20,
                },
                NarrativeChoice {
                    label: "Counter Offer".to_string(),
                    description: "Hold out for ~25% more (they may walk away)".to_string(),
                    effect: counter_effect,
                    reputation_change: 0,
                },
                NarrativeChoice {
                    label: "Decline".to_string(),
                    description: "This building is not for sale".to_string(),
                    effect: NarrativeEffect::None,
                    reputation_change: 5,
                },
            ],
        )
    }
}

impl Default for NarrativeEventSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// A single news-event template as authored in `assets/news_events.json`.
#[derive(Clone, Debug, Deserialize)]
struct NewsTemplate {
    headline: String,
    description: String,
    effect: NewsEffectSpec,
    /// Only meaningful for seasonal templates: which season (0=spring, 1=summer,
    /// 2=fall, 3=winter) this belongs to. Ignored for neighborhood/city banks.
    #[serde(default)]
    season: u32,
}

/// A data-driven effect spec. The concrete `NarrativeEffect` is built at
/// generation time so runtime ids (e.g. the neighborhood the news is about) can
/// be injected — they can't be baked into static content.
#[derive(Clone, Debug, Deserialize)]
struct NewsEffectSpec {
    kind: String,
    #[serde(default)]
    amount: f32,
}

impl NewsEffectSpec {
    fn to_effect(&self, neighborhood_id: u32) -> NarrativeEffect {
        match self.kind.as_str() {
            "neighborhood_reputation" => NarrativeEffect::NeighborhoodReputation {
                neighborhood_id,
                change: self.amount as i32,
            },
            "rent_demand" => NarrativeEffect::RentDemand {
                neighborhood_id,
                change: self.amount,
            },
            "economy_change" => NarrativeEffect::EconomyChange {
                economy_health_change: self.amount,
            },
            _ => NarrativeEffect::None,
        }
    }
}

/// The full set of news-event template banks.
#[derive(Clone, Debug, Deserialize, Default)]
struct NewsEventsConfig {
    #[serde(default)]
    neighborhood: Vec<NewsTemplate>,
    #[serde(default)]
    city: Vec<NewsTemplate>,
    /// Indexed by season (0=spring, 1=summer, 2=fall, 3=winter).
    #[serde(default)]
    seasonal: Vec<NewsTemplate>,
}

fn load_news_events() -> NewsEventsConfig {
    #[cfg(target_arch = "wasm32")]
    let json = macroquad_toolkit::include_json_str!("../../assets/news_events.json").to_string();

    #[cfg(not(target_arch = "wasm32"))]
    let json = std::fs::read_to_string("assets/news_events.json").unwrap_or_else(|_| {
        macroquad_toolkit::include_json_str!("../../assets/news_events.json").to_string()
    });

    serde_json::from_str(&json).unwrap_or_else(|e| {
        eprintln!("Failed to parse news_events.json: {}", e);
        NewsEventsConfig::default()
    })
}

#[cfg(test)]
mod tests;

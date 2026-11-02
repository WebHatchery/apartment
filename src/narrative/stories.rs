//! Tenant life stories: backgrounds, life changes and landlord requests.

use crate::data::config::LifeEventsConfig;
use crate::narrative::events_config::{RequestTemplate, TenantEventsConfig};
use crate::tenant::TenantArchetype;
use macroquad_toolkit::rng;
use serde::{Deserialize, Serialize};

/// A story event in a tenant's life
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoryEvent {
    pub month: u32,
    pub description: String,
    pub impact: StoryImpact,
}

/// How a story event affects gameplay
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StoryImpact {
    /// No gameplay effect, just flavor
    None,
    /// Affects happiness
    Happiness(i32),
    /// Affects rent tolerance
    RentTolerance(i32),
    /// Tenant may need to move
    MoveOutRisk(i32), // 0-100 probability
    /// Tenant requests something
    Request(TenantRequest),
    /// Tenant gets a roommate
    Roommate,
    /// Tenant has life event
    LifeChange(LifeChangeType),
    SetApartmentFlag(String),
    Multiple(Vec<StoryImpact>),
}

/// Types of requests tenants can make
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TenantRequest {
    /// Can I have a pet?
    Pet { pet_type: String },
    /// Can a family member stay temporarily?
    TemporaryGuest {
        guest_name: String,
        duration_months: u32,
    },
    /// Can I run a small business from home?
    HomeBusiness { business_type: String },
    /// Can I modify the apartment?
    Modification { description: String },
    /// Can I sublease part of the unit?
    Sublease,
}

impl TenantRequest {
    /// What happens if the landlord denies
    pub fn denial_effect(&self) -> StoryImpact {
        match self {
            TenantRequest::Pet { .. } => StoryImpact::Happiness(-10),
            TenantRequest::TemporaryGuest { .. } => StoryImpact::Happiness(-5),
            TenantRequest::HomeBusiness { .. } => StoryImpact::Happiness(-8),
            TenantRequest::Modification { .. } => StoryImpact::Happiness(-5),
            TenantRequest::Sublease => StoryImpact::MoveOutRisk(30),
        }
    }

    pub fn approval_effect(&self) -> StoryImpact {
        match self {
            TenantRequest::Pet { .. } => StoryImpact::Happiness(15),
            TenantRequest::TemporaryGuest { .. } => StoryImpact::Happiness(10),
            TenantRequest::HomeBusiness { business_type } => {
                let impacts = if business_type.to_lowercase().contains("music")
                    || business_type.to_lowercase().contains("drum")
                {
                    vec![
                        StoryImpact::Happiness(15),
                        StoryImpact::SetApartmentFlag("high_noise".to_string()),
                    ]
                } else {
                    vec![StoryImpact::Happiness(15)]
                };
                StoryImpact::Multiple(impacts)
            }
            TenantRequest::Modification { .. } => StoryImpact::Happiness(10),
            TenantRequest::Sublease => StoryImpact::Happiness(5),
        }
    }
}

/// Types of life changes
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LifeChangeType {
    /// Got a new job
    NewJob { better: bool },
    /// Lost their job
    JobLoss,
    /// Got married/partnered
    Partnered,
    /// Broke up/divorced
    Separated,
    /// Had a baby
    NewBaby,
    /// Child moved out
    ChildLeftHome,
    /// Health issue
    HealthIssue,
    /// Retirement
    Retired,
    /// Starting school
    StartedSchool,
    /// Graduated
    Graduated,
}

impl LifeChangeType {
    /// The concrete gameplay consequence of this life change, plus a short
    /// third-person description used to surface the emergent story. Magnitudes
    /// come from `LifeEventsConfig` so balance stays data-driven.
    pub fn impact(&self, cfg: &LifeEventsConfig) -> (StoryImpact, String) {
        use StoryImpact::{Happiness, MoveOutRisk, Multiple, RentTolerance};
        match self {
            LifeChangeType::NewJob { better: true } => (
                Multiple(vec![
                    Happiness(cfg.positive_happiness),
                    RentTolerance(cfg.rent_tolerance_boost),
                ]),
                "landed a better-paying job".to_string(),
            ),
            LifeChangeType::NewJob { better: false } => (
                Happiness(cfg.positive_happiness / 2),
                "started a new job".to_string(),
            ),
            LifeChangeType::JobLoss => (
                Multiple(vec![
                    Happiness(-cfg.negative_happiness),
                    RentTolerance(-cfg.rent_tolerance_drop),
                    MoveOutRisk(cfg.major_move_out_risk),
                ]),
                "lost their job".to_string(),
            ),
            LifeChangeType::Partnered => (
                Multiple(vec![
                    Happiness(cfg.positive_happiness),
                    RentTolerance(cfg.rent_tolerance_boost / 2),
                ]),
                "moved in with a partner".to_string(),
            ),
            LifeChangeType::Separated => (
                Multiple(vec![
                    Happiness(-cfg.negative_happiness),
                    RentTolerance(-cfg.rent_tolerance_drop / 2),
                    MoveOutRisk(cfg.minor_move_out_risk),
                ]),
                "went through a separation".to_string(),
            ),
            LifeChangeType::NewBaby => (
                Multiple(vec![
                    Happiness(cfg.positive_happiness / 2),
                    MoveOutRisk(cfg.major_move_out_risk),
                ]),
                "had a baby and needs more space".to_string(),
            ),
            LifeChangeType::ChildLeftHome => (
                MoveOutRisk(cfg.minor_move_out_risk),
                "became an empty-nester considering downsizing".to_string(),
            ),
            LifeChangeType::HealthIssue => (
                Multiple(vec![
                    Happiness(-cfg.negative_happiness),
                    MoveOutRisk(cfg.minor_move_out_risk),
                ]),
                "is dealing with a health issue".to_string(),
            ),
            LifeChangeType::Retired => (
                RentTolerance(-cfg.rent_tolerance_drop / 2),
                "retired onto a fixed income".to_string(),
            ),
            LifeChangeType::StartedSchool => (
                Happiness(cfg.positive_happiness / 2),
                "started school".to_string(),
            ),
            LifeChangeType::Graduated => (
                Multiple(vec![
                    Happiness(cfg.positive_happiness),
                    MoveOutRisk(cfg.major_move_out_risk),
                ]),
                "graduated and may move for work".to_string(),
            ),
        }
    }

    /// Life changes plausible for a given archetype.
    pub fn eligible_for(archetype: &TenantArchetype) -> Vec<LifeChangeType> {
        match archetype {
            TenantArchetype::Student => vec![
                LifeChangeType::Graduated,
                LifeChangeType::StartedSchool,
                LifeChangeType::NewJob { better: true },
                LifeChangeType::Partnered,
                LifeChangeType::HealthIssue,
            ],
            TenantArchetype::Professional => vec![
                LifeChangeType::NewJob { better: true },
                LifeChangeType::JobLoss,
                LifeChangeType::Partnered,
                LifeChangeType::Separated,
                LifeChangeType::NewBaby,
                LifeChangeType::HealthIssue,
            ],
            TenantArchetype::Artist => vec![
                LifeChangeType::NewJob { better: false },
                LifeChangeType::JobLoss,
                LifeChangeType::Partnered,
                LifeChangeType::Separated,
                LifeChangeType::HealthIssue,
            ],
            TenantArchetype::Family => vec![
                LifeChangeType::NewBaby,
                LifeChangeType::ChildLeftHome,
                LifeChangeType::JobLoss,
                LifeChangeType::NewJob { better: true },
                LifeChangeType::Separated,
                LifeChangeType::HealthIssue,
            ],
            TenantArchetype::Elderly => vec![
                LifeChangeType::Retired,
                LifeChangeType::HealthIssue,
                LifeChangeType::ChildLeftHome,
                LifeChangeType::Separated,
            ],
        }
    }
}

/// Complete story/background for a tenant
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantStory {
    pub tenant_id: u32,

    // Background
    pub job_title: String,
    pub hometown: String,
    pub move_reason: String,
    pub hobbies: Vec<String>,
    pub personality_traits: Vec<String>,

    // Family
    pub has_partner: bool,
    pub has_children: bool,
    pub num_children: u32,

    // History
    pub story_events: Vec<StoryEvent>,

    // Active requests
    pub pending_request: Option<TenantRequest>,
}

impl TenantStory {
    /// Generate a story for a new tenant
    pub fn generate(tenant_id: u32, archetype: &TenantArchetype) -> Self {
        background::generate(tenant_id, archetype)
    }

    /// Add a story event
    pub fn add_event(&mut self, month: u32, description: &str, impact: StoryImpact) {
        self.story_events.push(StoryEvent {
            month,
            description: description.to_string(),
            impact,
        });
    }

    /// Make a random request based on archetype using loaded config
    pub fn make_request(&mut self, archetype: &TenantArchetype, config: &TenantEventsConfig) {
        if self.pending_request.is_some() {
            return;
        }

        let archetype_key = archetype.name();

        if let Some(templates) = config.requests.get(archetype_key) {
            let total_weight: u32 = templates
                .iter()
                .map(|t| match t {
                    RequestTemplate::Pet { weight, .. } => *weight,
                    RequestTemplate::Sublease { weight } => *weight,
                    RequestTemplate::HomeBusiness { weight, .. } => *weight,
                    RequestTemplate::Modification { weight, .. } => *weight,
                    RequestTemplate::TemporaryGuest { weight, .. } => *weight,
                    RequestTemplate::None { weight } => *weight,
                })
                .sum();

            if total_weight == 0 {
                return;
            }

            let mut roll = rng::gen_range(0, total_weight);

            for template in templates {
                let weight = match template {
                    RequestTemplate::Pet { weight, .. } => *weight,
                    RequestTemplate::Sublease { weight } => *weight,
                    RequestTemplate::HomeBusiness { weight, .. } => *weight,
                    RequestTemplate::Modification { weight, .. } => *weight,
                    RequestTemplate::TemporaryGuest { weight, .. } => *weight,
                    RequestTemplate::None { weight } => *weight,
                };

                if roll < weight {
                    // Selected!
                    self.pending_request = match template {
                        RequestTemplate::Pet { options, .. } => Some(TenantRequest::Pet {
                            pet_type: rng::choose(options)
                                .cloned()
                                .unwrap_or_else(|| "cat".to_string()),
                        }),
                        RequestTemplate::Sublease { .. } => Some(TenantRequest::Sublease),
                        RequestTemplate::HomeBusiness { options, .. } => {
                            Some(TenantRequest::HomeBusiness {
                                business_type: rng::choose(options)
                                    .cloned()
                                    .unwrap_or_else(|| "consulting".to_string()),
                            })
                        }
                        RequestTemplate::Modification { options, .. } => {
                            Some(TenantRequest::Modification {
                                description: rng::choose(options)
                                    .cloned()
                                    .unwrap_or_else(|| "improvement".to_string()),
                            })
                        }
                        RequestTemplate::TemporaryGuest {
                            options,
                            duration_min,
                            duration_max,
                            ..
                        } => Some(TenantRequest::TemporaryGuest {
                            guest_name: rng::choose(options)
                                .cloned()
                                .unwrap_or_else(|| "Guest".to_string()),
                            duration_months: rng::gen_range(*duration_min, *duration_max + 1),
                        }),
                        RequestTemplate::None { .. } => None,
                    };
                    return;
                }
                roll -= weight;
            }
        }
    }
}

mod background;

#[cfg(test)]
mod tests;

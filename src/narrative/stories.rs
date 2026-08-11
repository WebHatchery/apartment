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
        let generator = BackgroundGenerator::default();
        generator.generate(tenant_id, archetype)
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

/// Generates tenant backgrounds
pub struct BackgroundGenerator {
    job_titles: std::collections::HashMap<TenantArchetype, Vec<&'static str>>,
    hometowns: Vec<&'static str>,
    move_reasons: std::collections::HashMap<TenantArchetype, Vec<&'static str>>,
    hobbies: std::collections::HashMap<TenantArchetype, Vec<&'static str>>,
    traits: Vec<&'static str>,
}

impl BackgroundGenerator {
    pub fn generate(&self, tenant_id: u32, archetype: &TenantArchetype) -> TenantStory {
        let job_title = self
            .job_titles
            .get(archetype)
            .and_then(|jobs| rng::choose(jobs).copied())
            .unwrap_or("Worker")
            .to_string();

        let hometown = rng::choose(&self.hometowns)
            .copied()
            .unwrap_or("Unknown")
            .to_string();

        let move_reason = self
            .move_reasons
            .get(archetype)
            .and_then(|reasons| rng::choose(reasons).copied())
            .unwrap_or("Needed a change")
            .to_string();

        let hobby_pool = self.hobbies.get(archetype).cloned().unwrap_or_default();
        let num_hobbies = rng::gen_range(1, 3);
        let hobbies: Vec<String> = rng::choose_multiple(&hobby_pool, num_hobbies)
            .into_iter()
            .map(|s| (*s).to_string())
            .collect();

        let num_traits = rng::gen_range(1, 3);
        let personality_traits: Vec<String> = rng::choose_multiple(&self.traits, num_traits)
            .into_iter()
            .map(|s| (*s).to_string())
            .collect();

        let (has_partner, has_children, num_children) = match archetype {
            TenantArchetype::Student => (false, false, 0),
            TenantArchetype::Professional => (rng::gen_range(0, 100) < 30, false, 0),
            TenantArchetype::Artist => (rng::gen_range(0, 100) < 20, false, 0),
            TenantArchetype::Family => (true, true, rng::gen_range(1, 4)),
            TenantArchetype::Elderly => {
                (rng::gen_range(0, 100) < 50, rng::gen_range(0, 100) < 70, 0)
            }
        };

        TenantStory {
            tenant_id,
            job_title,
            hometown,
            move_reason,
            hobbies,
            personality_traits,
            has_partner,
            has_children,
            num_children,
            story_events: Vec::new(),
            pending_request: None,
        }
    }
}

impl Default for BackgroundGenerator {
    fn default() -> Self {
        use std::collections::HashMap;

        let mut job_titles = HashMap::new();
        job_titles.insert(
            TenantArchetype::Student,
            vec![
                "University Student",
                "Graduate Student",
                "Community College Student",
                "Trade School Student",
                "Exchange Student",
                "Medical Student",
            ],
        );
        job_titles.insert(
            TenantArchetype::Professional,
            vec![
                "Software Developer",
                "Accountant",
                "Marketing Manager",
                "Lawyer",
                "Project Manager",
                "Financial Analyst",
                "Consultant",
                "Doctor",
                "Engineer",
                "Architect",
            ],
        );
        job_titles.insert(
            TenantArchetype::Artist,
            vec![
                "Painter",
                "Musician",
                "Writer",
                "Photographer",
                "Graphic Designer",
                "Sculptor",
                "Filmmaker",
                "Dancer",
                "Potter",
                "Illustrator",
            ],
        );
        job_titles.insert(
            TenantArchetype::Family,
            vec![
                "Teacher",
                "Nurse",
                "Small Business Owner",
                "Sales Representative",
                "Office Manager",
                "Electrician",
                "Chef",
                "Social Worker",
            ],
        );
        job_titles.insert(
            TenantArchetype::Elderly,
            vec![
                "Retired Teacher",
                "Retired Accountant",
                "Retired Nurse",
                "Retired Factory Worker",
                "Retired Business Owner",
                "Widower",
            ],
        );

        let hometowns = vec![
            "the suburbs",
            "a small town",
            "across the country",
            "overseas",
            "downtown",
            "the countryside",
            "another city",
            "up north",
            "the coast",
            "the midwest",
        ];

        let mut move_reasons = HashMap::new();
        move_reasons.insert(
            TenantArchetype::Student,
            vec![
                "Started at the local university.",
                "Needed to be closer to campus.",
                "Looking for affordable housing near school.",
                "Moving for an internship.",
            ],
        );
        move_reasons.insert(
            TenantArchetype::Professional,
            vec![
                "Got a new job in the area.",
                "Wanted a shorter commute.",
                "Looking for a quieter neighborhood.",
                "Relocated for work.",
            ],
        );
        move_reasons.insert(
            TenantArchetype::Artist,
            vec![
                "Looking for an inspiring space.",
                "Needed a studio with good light.",
                "Drawn to the creative community here.",
                "Escaping the high rents elsewhere.",
            ],
        );
        move_reasons.insert(
            TenantArchetype::Family,
            vec![
                "Needed more space for the kids.",
                "Moving for the school district.",
                "Wanted a safer neighborhood.",
                "Growing family needs.",
            ],
        );
        move_reasons.insert(
            TenantArchetype::Elderly,
            vec![
                "Downsizing after retirement.",
                "Wanted to be closer to family.",
                "Looking for a quieter place.",
                "Needed a ground floor unit.",
            ],
        );

        let mut hobbies = HashMap::new();
        hobbies.insert(
            TenantArchetype::Student,
            vec![
                "gaming",
                "studying",
                "partying",
                "jogging",
                "reading",
                "cooking on a budget",
                "streaming",
                "yoga",
            ],
        );
        hobbies.insert(
            TenantArchetype::Professional,
            vec![
                "wine tasting",
                "golf",
                "reading",
                "fitness",
                "travel",
                "cooking",
                "podcasts",
                "networking events",
            ],
        );
        hobbies.insert(
            TenantArchetype::Artist,
            vec![
                "painting",
                "music",
                "writing",
                "photography",
                "sculpting",
                "gallery hopping",
                "poetry readings",
                "experimental cooking",
            ],
        );
        hobbies.insert(
            TenantArchetype::Family,
            vec![
                "family outings",
                "cooking",
                "gardening",
                "board games",
                "soccer practice",
                "movie nights",
                "camping",
            ],
        );
        hobbies.insert(
            TenantArchetype::Elderly,
            vec![
                "gardening",
                "crossword puzzles",
                "watching TV",
                "knitting",
                "reading",
                "bird watching",
                "walking",
                "bingo",
            ],
        );

        let traits = vec![
            "quiet",
            "friendly",
            "private",
            "social",
            "neat",
            "messy",
            "punctual",
            "easygoing",
            "strict",
            "flexible",
            "chatty",
            "reserved",
        ];

        Self {
            job_titles,
            hometowns,
            move_reasons,
            hobbies,
            traits,
        }
    }
}

#[cfg(test)]
mod tests;

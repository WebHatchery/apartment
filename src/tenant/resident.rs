use super::TenantArchetype;
use macroquad_toolkit::rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: u32,
    pub name: String,
    pub archetype: TenantArchetype,

    // Current state
    pub happiness: i32,       // 0-100
    pub months_residing: u32, // How long they've lived here
    /// Index of the city building that owns `apartment_id`. Apartment IDs are
    /// only unique inside a building, so both fields are required to identify
    /// a unit in a portfolio.
    #[serde(default)]
    pub building_id: u32,
    pub apartment_id: Option<u32>,

    // Tolerances (derived from archetype but can vary slightly)
    pub rent_tolerance: i32,  // Max rent they'll accept
    pub noise_tolerance: i32, // 0-100, higher = more tolerant

    // Hidden stats (revealed via vetting)
    pub rent_reliability: i32, // 0-100, higher = more reliable rent payment
    pub behavior_score: i32,   // 0-100, higher = better neighbor behavior

    // Relationship with landlord
    pub landlord_opinion: i32, // -100 to 100, affects negotiations
}

impl Tenant {
    pub fn new(id: u32, name: &str, archetype: TenantArchetype) -> Self {
        let prefs = archetype.preferences();

        // Base reliability/behavior by archetype
        let (base_reliability, base_behavior) = match archetype {
            TenantArchetype::Student => (55, 50), // Lower reliability, average behavior
            TenantArchetype::Professional => (90, 85), // Very reliable, good behavior
            TenantArchetype::Artist => (60, 70),  // Moderate reliability, decent behavior
            TenantArchetype::Family => (80, 75),  // Reliable, good behavior
            TenantArchetype::Elderly => (95, 90), // Very reliable, excellent behavior
        };

        Self {
            id,
            name: name.to_string(),
            archetype,
            happiness: 70, // Start reasonably happy
            months_residing: 0,
            building_id: 0,
            apartment_id: None,
            rent_tolerance: prefs.ideal_rent_max,
            noise_tolerance: if prefs.prefers_quiet { 30 } else { 70 },
            landlord_opinion: 0,
            rent_reliability: base_reliability,
            behavior_score: base_behavior,
        }
    }

    /// Create a tenant with some randomization
    pub fn generate(id: u32, archetype: TenantArchetype) -> Self {
        let name = generate_random_name(&archetype);
        let mut tenant = Self::new(id, &name, archetype);

        // Add some variance to tolerances (±15%)
        let variance = 0.15;
        let rent_var = (tenant.rent_tolerance as f32 * variance) as i32;
        tenant.rent_tolerance += rng::gen_range(-rent_var, rent_var);

        let noise_var = (tenant.noise_tolerance as f32 * variance) as i32;
        tenant.noise_tolerance =
            (tenant.noise_tolerance + rng::gen_range(-noise_var, noise_var)).clamp(0, 100);

        // Random initial opinion slight variance
        tenant.landlord_opinion = rng::gen_range(-5, 6);

        // Add variance to hidden stats (±20%)
        let reliability_var = (tenant.rent_reliability as f32 * 0.2) as i32;
        tenant.rent_reliability = (tenant.rent_reliability
            + rng::gen_range(-reliability_var, reliability_var))
        .clamp(0, 100);

        let behavior_var = (tenant.behavior_score as f32 * 0.2) as i32;
        tenant.behavior_score =
            (tenant.behavior_score + rng::gen_range(-behavior_var, behavior_var)).clamp(0, 100);

        tenant
    }

    /// Check if tenant is at risk of leaving (early warning threshold)
    pub fn is_unhappy(&self, unhappy_threshold: i32) -> bool {
        self.happiness < unhappy_threshold
    }

    /// Roll whether an unhappy-enough tenant actually leaves this tick.
    /// Once at/below `leave_threshold`, there's a monthly chance they move out
    /// rather than an instant, deterministic exit.
    pub fn will_leave(&self, leave_threshold: i32, leave_chance_percent: i32) -> bool {
        self.happiness <= leave_threshold && rng::gen_range(0, 100) < leave_chance_percent
    }

    /// Update happiness (called each tick)
    pub fn set_happiness(&mut self, new_happiness: i32) {
        self.happiness = new_happiness.clamp(0, 100);
    }

    /// Increment months residing
    pub fn add_month(&mut self) {
        self.months_residing += 1;

        // Long-term tenants slowly trust landlord more (if not hated)
        if self.months_residing > 12 && self.landlord_opinion > -50 {
            // Slowly drift towards 0 or positive from neutral
            // (logic handled in systems, just basic state update here)
        }
    }

    /// Move into a unit in a specific city building.
    pub fn move_into_building(&mut self, building_id: u32, apartment_id: u32) {
        self.building_id = building_id;
        self.apartment_id = Some(apartment_id);
        self.months_residing = 0;
    }

    /// Whether this tenant currently lives in the addressed unit.
    pub fn lives_in(&self, building_id: u32, apartment_id: u32) -> bool {
        self.building_id == building_id && self.apartment_id == Some(apartment_id)
    }

    /// Move out of current apartment
    pub fn move_out(&mut self) {
        self.apartment_id = None;
    }

    /// Calculate negotiation leverage (0-100)
    pub fn negotiation_leverage(&self) -> i32 {
        // Loyalty bonus: up to 24 points for 2 years
        let loyalty_bonus = (self.months_residing as i32).min(24);

        // Opinion factor: High opinion reduces leverage (they trust you),
        // low opinion increases it (they are defensive)
        // Map -100..100 -> 20..-20
        let opinion_factor = -self.landlord_opinion / 5;

        (loyalty_bonus + opinion_factor).clamp(0, 100)
    }
}

/// Generate a random name appropriate for the archetype
fn generate_random_name(archetype: &TenantArchetype) -> String {
    let registry = crate::data::archetypes::archetypes();
    if let Some(definition) = registry.get(archetype.id()) {
        let first = rng::choose(&definition.name_pool.first_names)
            .map(String::as_str)
            .unwrap_or("Pat");
        let last = rng::choose(&definition.name_pool.last_initials)
            .map(String::as_str)
            .unwrap_or("X");

        if last == "Family" {
            return format!("{} {}", first, last);
        }

        return format!("{} {}.", first, last);
    }

    let first_names = match archetype {
        TenantArchetype::Student => vec![
            "Alex", "Jordan", "Casey", "Riley", "Morgan", "Sam", "Taylor", "Jamie", "Quinn",
            "Avery",
        ],
        TenantArchetype::Professional => vec![
            "Michael",
            "Sarah",
            "David",
            "Jennifer",
            "Robert",
            "Lisa",
            "James",
            "Amanda",
            "William",
            "Elizabeth",
        ],
        TenantArchetype::Artist => vec![
            "Luna", "River", "Sky", "Echo", "Rain", "Ash", "Sage", "Raven", "Nova", "Orion",
            "Willow", "Jasper",
        ],
        TenantArchetype::Family => vec![
            "The Smiths",
            "The Garcias",
            "The Kims",
            "The Patels",
            "The Joneses",
            "The Wangs",
            "The Johnsons",
            "The Millers",
        ],
        TenantArchetype::Elderly => vec![
            "Mrs. Higgins",
            "Mr. Abernathy",
            "Betty",
            "Harold",
            "Martha",
            "Walter",
            "Ethel",
            "Arthur",
        ],
    };

    let last_initials = vec![
        "A", "B", "C", "D", "E", "F", "G", "H", "J", "K", "L", "M", "N", "P", "R", "S", "T", "W",
    ];

    let first = *rng::choose(&first_names).unwrap_or(&"Pat");
    let last = *rng::choose(&last_initials).unwrap_or(&"X");

    format!("{} {}.", first, last)
}

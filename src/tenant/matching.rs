//! How well an applicant fits a unit, and whether they accept an offer.

use super::{happiness, Tenant};
use crate::building::Apartment;
use crate::data::config::MatchingConfig;

/// Result of matching a tenant to an apartment
use serde::{Deserialize, Serialize};

/// Result of matching a tenant to an apartment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchResult {
    pub score: i32,           // 0-100, higher = better match
    pub meets_minimum: bool,  // Would tenant even consider this?
    pub reasons: Vec<String>, // Why this score
}

/// Calculate how well a tenant matches an apartment
pub fn calculate_match_score(
    tenant: &Tenant,
    apartment: &Apartment,
    config: &MatchingConfig,
) -> MatchResult {
    let mut score = config.base_score;
    let mut reasons = Vec::new();

    let prefs = tenant.archetype.preferences();

    // Check minimum requirements
    let meets_minimum = happiness::apartment_meets_minimum(tenant, apartment);

    // Penalize but don't strictly forbid (allows "desperate" or "unqualified" applicants)
    if !meets_minimum {
        score += config.desperate_penalty;
        reasons.push("Does not meet requirements (Desperate/Unqualified)".to_string());
    }

    // Rent scoring
    let rent_diff = prefs.ideal_rent_max - apartment.rent_price;
    if rent_diff > config.rent_great_threshold {
        score += config.rent_great_bonus;
        reasons.push("Great price".to_string());
    } else if rent_diff > 0 {
        score += config.rent_fair_bonus;
        reasons.push("Fair price".to_string());
    } else if rent_diff > -100 {
        score += config.rent_slight_penalty;
        reasons.push("Slightly expensive".to_string());
    } else {
        score += config.rent_unaffordable_penalty;
        reasons.push("Cannot afford established budget".to_string());
    }

    // Condition scoring
    if apartment.condition >= config.condition_excellent_threshold {
        let bonus = (config.condition_excellent_bonus as f32 * prefs.condition_sensitivity) as i32;
        score += bonus;
        reasons.push("Excellent condition".to_string());
    } else if apartment.condition >= config.condition_good_threshold {
        let bonus = (config.condition_good_bonus as f32 * prefs.condition_sensitivity) as i32;
        score += bonus;
        reasons.push("Good condition".to_string());
    } else if apartment.condition < config.condition_poor_threshold {
        let penalty = (config.condition_poor_penalty as f32 * prefs.condition_sensitivity) as i32;
        score -= penalty;
        reasons.push("Poor condition".to_string());
    }

    // Noise scoring
    match apartment.effective_noise() {
        crate::building::NoiseLevel::Low => {
            if prefs.prefers_quiet {
                let bonus = (config.noise_quiet_bonus as f32 * prefs.noise_sensitivity) as i32;
                score += bonus;
                reasons.push("Nice and quiet".to_string());
            }
        }
        crate::building::NoiseLevel::High => {
            let penalty = (config.noise_loud_penalty as f32 * prefs.noise_sensitivity) as i32;
            score -= penalty;
            reasons.push("Too noisy".to_string());
        }
    }

    // Design scoring
    if let Some(ref preferred) = prefs.preferred_design {
        if &apartment.design == preferred {
            let bonus = (config.design_preferred_bonus as f32 * prefs.design_sensitivity) as i32;
            score += bonus;
            reasons.push(format!("Loves the {:?} style", apartment.design));
        }
    }

    // Size bonus (everyone likes more space)
    match apartment.size {
        crate::building::ApartmentSize::Small => {}
        crate::building::ApartmentSize::Medium => {
            score += config.size_medium_bonus;
            reasons.push("Good space".to_string());
        }
        crate::building::ApartmentSize::Large => {
            score += config.size_medium_bonus + 5;
            reasons.push("Great space".to_string());
        }
        crate::building::ApartmentSize::Penthouse => {
            score += config.size_medium_bonus + 15;
            reasons.push("Luxurious space".to_string());
        }
    }

    MatchResult {
        score: score.clamp(0, 100),
        meets_minimum,
        reasons,
    }
}

/// Find the best apartment match for a tenant from available options
pub fn find_best_match<'a>(
    tenant: &Tenant,
    apartments: &'a [&'a Apartment],
    config: &MatchingConfig,
) -> Option<(&'a Apartment, MatchResult)> {
    apartments
        .iter()
        .filter(|apt| apt.is_vacant())
        .map(|apt| (*apt, calculate_match_score(tenant, apt, config)))
        // No longer filtering by meets_minimum - allow all applicants
        .max_by_key(|(_, result)| result.score)
}

/// Parameters for a lease offer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaseOffer {
    pub rent_price: i32,
    pub security_deposit_months: u32, // 1, 2, or 3
    pub lease_duration_months: u32,   // Usually 6 or 12
    pub cleaning_fee: i32,            // 0 or more
}

use crate::data::config::LeaseDefaultsConfig;

impl LeaseOffer {
    /// Create a lease offer using config values
    pub fn from_config(rent: i32, config: &LeaseDefaultsConfig) -> Self {
        Self {
            rent_price: rent,
            security_deposit_months: config.security_deposit_months,
            lease_duration_months: config.lease_duration_months,
            cleaning_fee: config.cleaning_fee,
        }
    }
}

use crate::data::config::LeaseAcceptanceConfig;

/// Calculate probability of tenant accepting a lease offer (0.0 to 1.0)
pub fn evaluate_lease_offer(
    tenant: &Tenant,
    offer: &LeaseOffer,
    config: &LeaseAcceptanceConfig,
) -> f32 {
    let prefs = tenant.archetype.preferences();
    let mut probability = 1.0;

    // 1. Rent affordability (Hard limit)
    if offer.rent_price > tenant.rent_tolerance {
        return 0.0;
    }

    // 2. Security Deposit Impact
    // High deposit hurts budget-conscious tenants (Students, Artists)
    // Rent sensitivity serves as a proxy for "budget consciousness"
    if offer.security_deposit_months > 1 {
        let deposit_penalty = match offer.security_deposit_months {
            2 => config.deposit_2_month_penalty * prefs.rent_sensitivity,
            _ => config.deposit_3_month_penalty * prefs.rent_sensitivity, // 3+ months
        };
        probability -= deposit_penalty;
    }

    // 3. Lease Duration
    // Students/Artists might prefer shorter leases (6 months)
    // Families/Elderly/Professionals prefer stability (12 months)
    match tenant.archetype {
        crate::tenant::TenantArchetype::Student | crate::tenant::TenantArchetype::Artist => {
            if offer.lease_duration_months == 6 {
                probability += config.short_lease_bonus; // Bonus for flexibility
            } else if offer.lease_duration_months > 12 {
                probability -= config.short_lease_bonus; // Too long
            }
        }
        _ => {
            // Stability seekers
            if offer.lease_duration_months < 12 {
                probability -= config.long_lease_penalty; // Too unstable
            }
        }
    }

    // 4. Cleaning Fee
    if offer.cleaning_fee > 0 {
        // Flat penalty relative to rent
        let fee_ratio = offer.cleaning_fee as f32 / offer.rent_price as f32;
        probability -= fee_ratio * prefs.rent_sensitivity;
    }

    // 5. Rent Value (Deal vs Rip-off)
    let rent_diff = prefs.ideal_rent_max - offer.rent_price;
    if rent_diff < 0 {
        // Slightly above comfortable max (but below absolute tolerance)
        probability -= config.expensive_penalty;
    } else if rent_diff > 100 {
        // Good deal
        probability += config.good_deal_bonus;
    }

    probability.clamp(0.0, 1.0)
}

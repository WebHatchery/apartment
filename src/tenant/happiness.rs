use super::{ArchetypePreferences, Tenant};
use crate::building::{Apartment, Building, DesignType, NoiseLevel};

/// All factors that influence happiness
#[derive(Clone, Debug)]
pub struct HappinessFactors {
    pub base_happiness: i32,
    pub rent_factor: i32,      // Negative if too expensive
    pub condition_factor: i32, // Based on apartment condition
    pub noise_factor: i32,     // Negative if too noisy
    pub design_factor: i32,    // Based on design preference
    pub hallway_factor: i32,   // Building shared space condition
    pub tenure_bonus: i32,     // Small bonus for long-term residents
    pub staff_factor: i32,     // Security/manager presence
}

impl HappinessFactors {
    pub fn total(&self) -> i32 {
        (self.base_happiness
            + self.rent_factor
            + self.condition_factor
            + self.noise_factor
            + self.design_factor
            + self.hallway_factor
            + self.tenure_bonus
            + self.staff_factor)
            .clamp(0, 100)
    }
}

use crate::data::config::{HappinessConfig, StaffEffectsConfig};

/// Calculate happiness factors for a tenant in their apartment
pub fn calculate_happiness(
    tenant: &Tenant,
    apartment: &Apartment,
    building: &Building,
    config: &HappinessConfig,
    staff: &StaffEffectsConfig,
) -> HappinessFactors {
    let prefs = tenant.archetype.preferences();

    HappinessFactors {
        base_happiness: config.base,
        rent_factor: calculate_rent_factor(apartment.rent_price, &prefs, config),
        condition_factor: calculate_condition_factor(apartment.condition, &prefs, config),
        noise_factor: calculate_noise_factor(
            &apartment.effective_noise(),
            tenant.noise_tolerance,
            &prefs,
            config,
        ),
        design_factor: calculate_design_factor(&apartment.design, &prefs, config),
        hallway_factor: calculate_hallway_factor(building.hallway_condition, config),
        tenure_bonus: calculate_tenure_bonus(tenant.months_residing, config),
        staff_factor: calculate_staff_factor(building, staff),
    }
}

/// Happiness contribution from on-site staff. Persisted through the happiness
/// recompute (unlike a one-off nudge), so hiring security/a manager is felt.
fn calculate_staff_factor(building: &Building, staff: &StaffEffectsConfig) -> i32 {
    let mut factor = 0;
    if building.flags.contains("staff_security") {
        factor += staff.security_happiness_bonus;
    }
    if building.flags.contains("staff_manager") {
        factor += staff.manager_happiness_bonus;
    }
    if building.utilities_included {
        factor += staff.utilities_happiness_bonus;
    }
    factor
}

fn calculate_rent_factor(rent: i32, prefs: &ArchetypePreferences, config: &HappinessConfig) -> i32 {
    let diff = prefs.ideal_rent_max - rent;
    let sensitivity = prefs.rent_sensitivity;

    if diff >= 0 {
        // Under budget - small bonus
        ((diff as f32 * config.rent_bonus_multiplier * sensitivity) as i32)
            .min(config.rent_bonus_cap)
    } else {
        // Over budget - penalty
        ((diff as f32 * config.rent_penalty_multiplier * sensitivity) as i32)
            .max(config.rent_penalty_cap)
    }
}

fn calculate_condition_factor(
    condition: i32,
    prefs: &ArchetypePreferences,
    config: &HappinessConfig,
) -> i32 {
    let sensitivity = prefs.condition_sensitivity;
    let min_acceptable = prefs.min_acceptable_condition;

    if condition >= min_acceptable {
        // Good condition - bonus based on how much above minimum
        let excess = condition - min_acceptable;
        ((excess as f32 * config.condition_bonus_multiplier * sensitivity) as i32)
            .min(config.condition_bonus_cap)
    } else {
        // Below minimum - significant penalty
        let deficit = min_acceptable - condition;
        -((deficit as f32 * config.condition_penalty_multiplier * sensitivity) as i32)
            .min(config.condition_penalty_cap)
    }
}

fn calculate_noise_factor(
    noise: &NoiseLevel,
    tolerance: i32,
    prefs: &ArchetypePreferences,
    config: &HappinessConfig,
) -> i32 {
    let sensitivity = prefs.noise_sensitivity;

    match noise {
        NoiseLevel::Low => {
            // Quiet - small bonus for those who prefer it
            if prefs.prefers_quiet {
                (config.noise_quiet_bonus * sensitivity) as i32
            } else {
                0
            }
        }
        NoiseLevel::High => {
            // Noisy - penalty based on sensitivity and tolerance
            let base_penalty = config.noise_high_penalty_base;
            let tolerance_mod = (tolerance as f32 * config.noise_tolerance_multiplier) as i32;
            ((base_penalty + tolerance_mod) as f32 * sensitivity) as i32
        }
    }
}

fn calculate_design_factor(
    design: &DesignType,
    prefs: &ArchetypePreferences,
    config: &HappinessConfig,
) -> i32 {
    let sensitivity = prefs.design_sensitivity;
    let mut factor = 0;

    // Check if this is their preferred design
    if let Some(ref preferred) = prefs.preferred_design {
        if design == preferred {
            factor += config.design_preferred_bonus;
        }
    }

    // Check if this is their hated design
    if let Some(ref hated) = prefs.hates_design {
        if design == hated {
            factor += config.design_hated_penalty;
        }
    }

    // General design quality bonus
    if let Some(val) = config.design_style_modifiers.get(&format!("{:?}", design)) {
        factor += *val;
    }

    (factor as f32 * sensitivity) as i32
}

fn calculate_hallway_factor(hallway_condition: i32, config: &HappinessConfig) -> i32 {
    // Shared space affects everyone equally but mildly
    ((hallway_condition - config.hallway_condition_base) as f32
        * config.hallway_condition_multiplier) as i32
}

fn calculate_tenure_bonus(months: u32, config: &HappinessConfig) -> i32 {
    // Long-term residents get a small stability bonus
    (months as i32).min(config.tenure_bonus_max)
}

/// Check if apartment meets minimum requirements for tenant
pub fn apartment_meets_minimum(tenant: &Tenant, apartment: &Apartment) -> bool {
    let prefs = tenant.archetype.preferences();

    // Check condition minimum
    if apartment.condition < prefs.min_acceptable_condition {
        return false;
    }

    // Check rent tolerance
    if apartment.rent_price > tenant.rent_tolerance {
        return false;
    }

    // Check design compatibility (artists won't accept bare)
    if let Some(ref hated) = prefs.hates_design {
        if &apartment.design == hated {
            return false;
        }
    }

    // Check noise for noise-sensitive tenants
    if prefs.prefers_quiet
        && matches!(apartment.effective_noise(), NoiseLevel::High)
        && tenant.noise_tolerance < 40
    {
        return false;
    }

    true
}

/// Calculate happiness modifier from tenant relationships
pub fn calculate_relationship_happiness(
    tenant_id: u32,
    network: &crate::consequences::TenantNetwork,
    config: &crate::data::config::RelationshipsConfig,
) -> i32 {
    let mut bonus = 0;

    for relationship in &network.relationships {
        if relationship.tenant_a_id == tenant_id || relationship.tenant_b_id == tenant_id {
            bonus += relationship.relationship_type.happiness_modifier(config);
        }
    }

    // Cap the relationship bonus
    bonus.clamp(-20, 20)
}

#[cfg(test)]
mod tests;

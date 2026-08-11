use super::{matching::MatchResult, Tenant, TenantArchetype};
use crate::building::Building;
use crate::data::config::{GameConfig, TenantRiskConfig};
use macroquad_toolkit::rng;
use serde::{Deserialize, Serialize};

/// Shape a fresh applicant's risk: a data-driven minority are forced into the
/// "problem tenant" range (so screening has something to catch), and any risky
/// applicant then gets a rent-tolerance premium (so a bad tenant is *tempting*).
/// Together these are the reward/risk structure that makes tenant selection a
/// real decision rather than a rubber stamp.
pub fn apply_applicant_risk_profile(tenant: &mut Tenant, config: &TenantRiskConfig) {
    // A minority of applicants are problem tenants regardless of archetype.
    if rng::gen_range(0, 100) < config.problem_applicant_chance_percent {
        let reliability_floor = rng::gen_range(20, config.unreliable_threshold.max(21));
        let behavior_floor = rng::gen_range(20, config.low_behavior_threshold.max(21));
        tenant.rent_reliability = tenant.rent_reliability.min(reliability_floor);
        tenant.behavior_score = tenant.behavior_score.min(behavior_floor);
    }
    apply_risk_rent_premium(tenant, config);
}

/// Give risky applicants (unreliable or poorly-behaved) a rent-tolerance
/// premium: they're more desperate, so they'll accept and pay higher rent. The
/// premium scales from full (at the worst applicants) to zero as an applicant
/// reaches the unreliable threshold.
pub fn apply_risk_rent_premium(tenant: &mut Tenant, config: &TenantRiskConfig) {
    let safety = tenant.rent_reliability.min(tenant.behavior_score);
    let threshold = config.unreliable_threshold.max(1);
    if safety >= threshold {
        return;
    }
    let risk_fraction = (threshold - safety) as f32 / threshold as f32;
    let premium = (tenant.rent_tolerance as f32 * config.risky_rent_premium_percent as f32 / 100.0
        * risk_fraction) as i32;
    tenant.rent_tolerance += premium;
}

/// A tenant application for a specific apartment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantApplication {
    pub tenant: Tenant,
    /// City building index. Apartment IDs repeat between buildings.
    #[serde(default)]
    pub building_id: u32,
    pub apartment_id: u32,
    pub match_result: MatchResult,
    pub tick_created: u32, // When this application was generated

    // Vetting state (hidden stats revealed after checks)
    pub revealed_reliability: bool, // Credit check done?
    pub revealed_behavior: bool,    // Background check done?
}

impl TenantApplication {
    pub fn new_for_building(
        tenant: Tenant,
        building_id: u32,
        apartment_id: u32,
        match_result: MatchResult,
        tick: u32,
    ) -> Self {
        Self {
            tenant,
            building_id,
            apartment_id,
            match_result,
            tick_created: tick,
            revealed_reliability: false,
            revealed_behavior: false,
        }
    }

    pub fn is_expired_after(&self, current_tick: u32, expire_after_ticks: u32) -> bool {
        current_tick > self.tick_created + expire_after_ticks
    }
}

/// Generate new tenant applications for listed apartments
pub fn generate_applications(
    building_id: u32,
    building: &Building,
    existing_applications: &[TenantApplication],
    current_tick: u32,
    next_tenant_id: &mut u32,
    reputation_multiplier: f32,
    config: &GameConfig,
) -> Vec<TenantApplication> {
    let mut new_applications = Vec::new();

    // 1. Identify listed vacancies
    let listed_apartments: Vec<&_> = building
        .vacant_apartments()
        .into_iter()
        .filter(|a| a.is_listed_for_lease)
        .collect();

    if listed_apartments.is_empty() {
        return new_applications;
    }

    let building_appeal = building.building_appeal();

    // Marketing and front-desk staffing both make listed units easier for
    // applicants to discover.
    let marketing_multiplier = match building.marketing_strategy {
        crate::building::MarketingType::None => 1.0,
        crate::building::MarketingType::SocialMedia => 2.0,
        crate::building::MarketingType::LocalNewspaper => 1.5,
        crate::building::MarketingType::PremiumAgency => 1.5,
    };

    let open_house_multiplier = if building.open_house_remaining > 0 {
        2.0
    } else {
        1.0
    };
    let staff_multiplier = application_staff_multiplier(building, &config.staff_effects);

    // 2. Generate applications for EACH listed apartment
    for apt in listed_apartments {
        // Base probability per apartment
        let appeal_divisor = config.applications.appeal_bonus_divisor.max(1) as f32;
        let appeal_factor = (building_appeal as f32 / appeal_divisor).max(0.5);
        let chance = config.applications.base_per_vacancy
            * appeal_factor
            * marketing_multiplier
            * open_house_multiplier
            * staff_multiplier
            * reputation_multiplier;

        // Random check to see if we generate an applicant this tick
        if rng::gen_range(0.0, 1.0) < chance {
            // Pick archetype based on preference + marketing
            let archetype = pick_archetype_with_preference(
                &building.marketing_strategy,
                apt.preferred_archetype.as_ref(),
            );

            // Generate tenant
            let mut tenant = Tenant::generate(*next_tenant_id, archetype);
            apply_applicant_risk_profile(&mut tenant, &config.tenant_risk);
            *next_tenant_id += 1;

            // Check match
            let apt_slice = [apt];
            if let Some((_, match_result)) =
                super::matching::find_best_match(&tenant, &apt_slice, &config.matching)
            {
                // Check dupes
                let already_applied =
                    existing_applications.iter().any(|app| {
                        app.building_id == building_id
                            && app.apartment_id == apt.id
                            && app.tenant.archetype == tenant.archetype
                    }) || new_applications.iter().any(|app: &TenantApplication| {
                        app.building_id == building_id
                            && app.apartment_id == apt.id
                            && app.tenant.archetype == tenant.archetype
                    });

                if !already_applied {
                    new_applications.push(TenantApplication::new_for_building(
                        tenant,
                        building_id,
                        apt.id,
                        match_result,
                        current_tick,
                    ));
                }
            }
        }
    }

    new_applications
}

pub fn application_staff_multiplier(
    building: &Building,
    config: &crate::data::config::StaffEffectsConfig,
) -> f32 {
    if building.flags.contains("staff_receptionist") {
        config.receptionist_application_multiplier
    } else {
        1.0
    }
}

fn pick_archetype_with_preference(
    marketing: &crate::building::MarketingType,
    preference: Option<&TenantArchetype>,
) -> TenantArchetype {
    // If preference exists, 80% chance to pick it
    if let Some(pref) = preference {
        if rng::gen_range(0, 100) < 80 {
            return pref.clone();
        }
    }

    let mut weighted_archetypes = base_weighted_archetypes();

    for (archetype, weight) in &mut weighted_archetypes {
        let multiplier = match marketing {
            crate::building::MarketingType::SocialMedia => match archetype {
                TenantArchetype::Student | TenantArchetype::Artist => 2,
                _ => 1,
            },
            crate::building::MarketingType::LocalNewspaper => match archetype {
                TenantArchetype::Family | TenantArchetype::Elderly => 2,
                _ => 1,
            },
            crate::building::MarketingType::PremiumAgency => match archetype {
                TenantArchetype::Professional => 3,
                TenantArchetype::Family => 2,
                _ => 1,
            },
            crate::building::MarketingType::None => 1,
        };
        *weight *= multiplier;
    }

    let total_weight: u32 = weighted_archetypes.iter().map(|(_, weight)| *weight).sum();
    let mut roll = rng::gen_range(0, total_weight.max(1));
    for (archetype, weight) in weighted_archetypes {
        if roll < weight {
            return archetype;
        }
        roll -= weight;
    }

    TenantArchetype::Student
}

fn base_weighted_archetypes() -> Vec<(TenantArchetype, u32)> {
    let registry = crate::data::archetypes::archetypes();
    let mut weighted: Vec<(TenantArchetype, u32)> = registry
        .definitions
        .values()
        .filter_map(|definition| {
            TenantArchetype::from_id(&definition.id)
                .map(|archetype| (archetype, definition.spawn_weight.max(1)))
        })
        .collect();

    if weighted.is_empty() {
        weighted = vec![
            (TenantArchetype::Student, 35),
            (TenantArchetype::Professional, 25),
            (TenantArchetype::Family, 15),
            (TenantArchetype::Elderly, 10),
            (TenantArchetype::Artist, 15),
        ];
    }

    // A seeded roll must map to the same archetype after every process launch.
    // HashMap iteration order is randomized, so establish the authored ID order
    // before applying weights and walking the cumulative distribution.
    weighted.sort_by_key(|(archetype, _)| archetype.id());
    weighted
}

/// Process tenant decisions to leave
pub fn process_departures(
    building_id: u32,
    tenants: &mut Vec<Tenant>,
    building: &mut Building,
    config: &crate::data::config::HappinessConfig,
) -> Vec<String> {
    let mut notifications = Vec::new();
    let mut departing_ids = Vec::new();

    for tenant in tenants.iter_mut() {
        if tenant.building_id != building_id {
            continue;
        }
        // Roll once — will_leave is probabilistic, so reuse the result rather
        // than re-rolling it for the early-warning check below.
        let leaving = tenant.will_leave(config.leave_threshold, config.leave_chance_percent);

        if tenant.is_unhappy(config.unhappy_threshold) && !leaving {
            notifications.push(format!("{} is unhappy and may leave soon!", tenant.name));
        }

        if leaving {
            notifications.push(format!("{} has moved out!", tenant.name));
            departing_ids.push(tenant.id);

            // Clear apartment
            if let Some(apt_id) = tenant.apartment_id {
                if let Some(apt) = building.get_apartment_mut(apt_id) {
                    apt.move_out();
                }
            }

            // Clear tenant's apartment reference
            tenant.move_out();
        }
    }

    tenants.retain(|t| !departing_ids.contains(&t.id));
    notifications
}

#[cfg(test)]
mod tests;

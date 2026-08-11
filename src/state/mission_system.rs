use super::GameplayState;
use crate::narrative::{ActiveTaxBreak, MissionReward};
use crate::simulation::GameEvent;
use crate::ui::colors;
use macroquad::prelude::*;

/// System for handling mission updates and rewards
pub fn update_missions(state: &mut GameplayState) {
    let current_month = state.current_tick;
    let active_building_id = state.active_building_id();
    let active_tenants = state.active_tenants_cloned();

    // Snapshot this month's building-wide signals up front so per-mission
    // evaluation can read them without borrowing conflicts.
    let avg_happiness = if active_tenants.is_empty() {
        0.0
    } else {
        active_tenants
            .iter()
            .map(|t| t.happiness as f32)
            .sum::<f32>()
            / active_tenants.len() as f32
    };
    // "Perfect collection" = at least one tenant and no missed-rent event this
    // month.
    let perfect_collection = !active_tenants.is_empty()
        && state.last_tick_result.as_ref().is_some_and(|r| {
            !r.events
                .iter()
                .any(|e| matches!(e, GameEvent::RentMissed { .. }))
        });
    let fully_repaired_by_building = fully_repaired_buildings(state);

    let rental_units = state.building.rental_unit_count();
    let occupancy = if rental_units > 0 {
        state.building.occupancy_count() as f32 / rental_units as f32
    } else {
        0.0
    };
    let completed_missions = state.missions.evaluate_active(
        current_month,
        active_building_id,
        &active_tenants,
        occupancy,
        avg_happiness,
        perfect_collection,
        &fully_repaired_by_building,
        state.city.buildings.len(),
    );

    for mission in completed_missions {
        state.missions.record_legacy_event(
            current_month,
            &format!("Mission Complete: {}", mission.title),
            &format!("Completed objective: {}", mission.description),
        );

        match mission.reward {
            MissionReward::Money(amount) => {
                let t = crate::economy::Transaction::income(
                    crate::economy::TransactionType::Grant,
                    amount,
                    "Mission Reward",
                    current_month,
                );
                state.funds.add_income(t);

                state.floating_texts.spawn(
                    format!("+${}", amount),
                    vec2(screen_width() / 2.0, screen_height() / 2.0 + 30.0),
                    colors::POSITIVE(),
                );
            }
            MissionReward::UnlockBuilding(unlock_order) => {
                state.unlock_building_by_order(unlock_order);
                state.floating_texts.spawn(
                    "New property unlocked!",
                    vec2(screen_width() / 2.0, screen_height() / 2.0 + 30.0),
                    colors::ACCENT(),
                );
            }
            MissionReward::Reputation(amount) => {
                // Reward reputation in the active building's neighborhood.
                state.apply_reputation_change(amount, None);
            }
            MissionReward::TaxBreak { months, percentage } => {
                state
                    .active_tax_breaks
                    .push(ActiveTaxBreak::new(months, percentage));
                state.floating_texts.spawn(
                    format!(
                        "Tax Break! {}% for {} months",
                        (percentage * 100.0) as i32,
                        months
                    ),
                    vec2(screen_width() / 2.0, screen_height() / 2.0 + 30.0),
                    colors::POSITIVE(),
                );
            }
        }
    }
}

fn fully_repaired_buildings(state: &GameplayState) -> std::collections::HashMap<u32, bool> {
    let mut repaired: std::collections::HashMap<u32, bool> = state
        .city
        .buildings
        .iter()
        .enumerate()
        .map(|(building_id, building)| {
            (
                building_id as u32,
                !building.apartments.is_empty()
                    && building
                        .apartments
                        .iter()
                        .all(|apartment| apartment.condition >= 90)
                    && building.hallway_condition >= 90,
            )
        })
        .collect();
    repaired.insert(
        state.active_building_id(),
        !state.building.apartments.is_empty()
            && state
                .building
                .apartments
                .iter()
                .all(|apartment| apartment.condition >= 90)
            && state.building.hallway_condition >= 90,
    );
    repaired
}

#[cfg(test)]
mod tests;

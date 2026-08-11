//! Narrative effect application for gameplay state.

use crate::narrative::events::NarrativeEffect;
use crate::narrative::{MissionGoal, MissionStatus};
use crate::ui::colors;
use macroquad::prelude::*;

use super::gameplay::{GameplayState, ViewMode};

impl GameplayState {
    /// Apply a narrative effect to the current gameplay state.
    pub(super) fn apply_narrative_effect(&mut self, effect: &NarrativeEffect) {
        match effect {
            NarrativeEffect::None => {}
            NarrativeEffect::Money { amount } => {
                if *amount < 0 {
                    self.funds
                        .apply_required_expense(crate::economy::Transaction::expense(
                            crate::economy::TransactionType::CriticalFailure,
                            amount.abs(),
                            "Event Consequence",
                            self.current_tick,
                        ));
                } else {
                    self.funds.add_income(crate::economy::Transaction::income(
                        crate::economy::TransactionType::Grant,
                        *amount,
                        "Event Reward",
                        self.current_tick,
                    ));
                }
            }
            NarrativeEffect::TenantHappiness { tenant_id, change } => {
                if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == *tenant_id) {
                    tenant.happiness = (tenant.happiness + change).clamp(0, 100);
                }
            }
            NarrativeEffect::OpinionChange { tenant_id, amount } => {
                if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == *tenant_id) {
                    tenant.landlord_opinion = (tenant.landlord_opinion + amount).clamp(-100, 100);
                }
            }
            NarrativeEffect::RelationshipStrength {
                tenant_a_id,
                tenant_b_id,
                change,
            } => {
                self.tenant_network
                    .apply_relationship_change(*tenant_a_id, *tenant_b_id, *change);
            }
            NarrativeEffect::MoveOut { tenant_id } => {
                if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == *tenant_id) {
                    tenant.happiness = 0;
                }
            }
            NarrativeEffect::SellBuilding { building_id } => {
                self.sell_building_from_event(*building_id);
            }
            NarrativeEffect::Multiple { effects } => {
                for effect in effects {
                    self.apply_narrative_effect(effect);
                }
            }
            NarrativeEffect::NeighborhoodReputation {
                neighborhood_id,
                change,
            } => {
                if let Some(neighborhood) = self
                    .city
                    .neighborhoods
                    .iter_mut()
                    .find(|n| n.id == *neighborhood_id)
                {
                    neighborhood.reputation = (neighborhood.reputation + change).clamp(0, 100);
                }
            }
            NarrativeEffect::BuildingHappiness {
                building_id,
                change,
            } => {
                for tenant in &mut self.tenants {
                    if tenant.building_id == *building_id {
                        tenant.happiness = (tenant.happiness + change).clamp(0, 100);
                    }
                }
            }
            NarrativeEffect::EconomyChange {
                economy_health_change,
            } => {
                self.city.economy_health =
                    (self.city.economy_health + economy_health_change).clamp(0.5, 1.5);
            }
            NarrativeEffect::RentDemand {
                neighborhood_id,
                change,
            } => {
                if let Some(neighborhood) = self
                    .city
                    .neighborhoods
                    .iter_mut()
                    .find(|n| n.id == *neighborhood_id)
                {
                    neighborhood.stats.rent_demand =
                        (neighborhood.stats.rent_demand + change).clamp(0.5, 2.0);
                }
            }
            NarrativeEffect::TriggerInspection { building_id } => {
                self.execute_inspection_for(
                    *building_id,
                    crate::consequences::InspectionTrigger::TenantComplaint,
                );
                self.bill_outstanding_fines();
            }
            NarrativeEffect::PropertyValue {
                building_id,
                change_percent,
            } => {
                // Property value is expressed through the building's rent ceiling:
                // a value change lets the landlord command proportionally more (or
                // less) rent.
                let factor = 1.0 + change_percent / 100.0;
                if *building_id == self.active_building_id() {
                    self.building.rent_multiplier =
                        (self.building.rent_multiplier * factor).clamp(0.5, 2.0);
                    self.save_building_to_city();
                } else if let Some(building) = self.city.buildings.get_mut(*building_id as usize) {
                    building.rent_multiplier = (building.rent_multiplier * factor).clamp(0.5, 2.0);
                }
            }
        }
    }

    fn sell_building_from_event(&mut self, building_id: u32) {
        let index = building_id as usize;
        if index >= self.city.buildings.len() {
            return;
        }

        self.save_building_to_city();
        let sold_building = self.city.buildings[index].clone();
        let neighborhood_name = self
            .city
            .neighborhood_for_building(index)
            .map(|neighborhood| neighborhood.name.clone())
            .unwrap_or_else(|| "Unknown neighborhood".to_string());
        let removed_tenant_ids: std::collections::HashSet<u32> = self
            .tenants
            .iter()
            .filter(|tenant| tenant.building_id == building_id)
            .map(|tenant| tenant.id)
            .collect();

        for tenant in self
            .tenants
            .iter()
            .filter(|tenant| tenant.building_id == building_id)
        {
            let rent = tenant
                .apartment_id
                .and_then(|apartment_id| sold_building.get_apartment(apartment_id))
                .map(|apartment| apartment.rent_price)
                .unwrap_or(0);
            self.gentrification
                .displacements
                .push(crate::consequences::DisplacementEvent {
                    tenant_name: tenant.name.clone(),
                    archetype: tenant.archetype.clone(),
                    original_rent: rent,
                    final_rent: rent,
                    months_resided: tenant.months_residing,
                    reason: crate::consequences::DisplacementReason::BuildingSold,
                    month: self.current_tick,
                    building_name: sold_building.name.clone(),
                    neighborhood_name: neighborhood_name.clone(),
                });
        }

        self.tenants
            .retain(|tenant| tenant.building_id != building_id);
        self.applications
            .retain(|application| application.building_id != building_id);
        self.tenant_stories
            .retain(|tenant_id, _| !removed_tenant_ids.contains(tenant_id));
        self.tenant_network.relationships.retain(|relationship| {
            !removed_tenant_ids.contains(&relationship.tenant_a_id)
                && !removed_tenant_ids.contains(&relationship.tenant_b_id)
        });
        self.tenant_network
            .long_term_tenants
            .retain(|record| !removed_tenant_ids.contains(&record.tenant_id));
        self.tenant_network
            .dilemma_history
            .retain(|tenant_id, _| !removed_tenant_ids.contains(tenant_id));
        self.tenant_network
            .tensions
            .retain(|tension| tension.building_id != building_id);

        self.city.buildings.remove(index);
        reindex_building_references(self, building_id);

        if self.city.buildings.is_empty() {
            self.game_outcome = Some(crate::simulation::GameOutcome::Victory {
                score: self.funds.balance,
                months: self.current_tick,
                total_income: self.funds.total_income,
            });
            self.view_mode = ViewMode::CareerSummary;
        } else {
            let previous_active = self.city.active_building_index;
            self.city.active_building_index = if previous_active > index {
                previous_active - 1
            } else if previous_active == index {
                index.min(self.city.buildings.len() - 1)
            } else {
                previous_active
            };
            self.sync_building();
            self.selection = crate::ui::Selection::None;
            self.council_formed = false;
            self.floating_texts.spawn(
                "Building Sold!",
                vec2(screen_width() / 2.0, screen_height() / 2.0),
                colors::POSITIVE(),
            );
        }
    }
}

fn reindex_building_references(state: &mut GameplayState, removed_id: u32) {
    for neighborhood in &mut state.city.neighborhoods {
        neighborhood.building_ids.retain(|id| *id != removed_id);
        for id in &mut neighborhood.building_ids {
            if *id > removed_id {
                *id -= 1;
            }
        }
    }
    for tenant in &mut state.tenants {
        if tenant.building_id > removed_id {
            tenant.building_id -= 1;
        }
    }
    for application in &mut state.applications {
        if application.building_id > removed_id {
            application.building_id -= 1;
        }
    }
    for tension in &mut state.tenant_network.tensions {
        if tension.building_id > removed_id {
            tension.building_id -= 1;
        }
    }

    state.ever_occupied_buildings = state
        .ever_occupied_buildings
        .iter()
        .filter_map(|id| reindexed_id(*id, removed_id))
        .collect();
    state.compliance.building_regulations = reindex_map(
        std::mem::take(&mut state.compliance.building_regulations),
        removed_id,
    );
    state
        .compliance
        .inspection_history
        .retain_mut(|inspection| reindex_id_mut(&mut inspection.building_id, removed_id));
    state
        .compliance
        .pending_fixes
        .retain_mut(|(id, _, _)| reindex_id_mut(id, removed_id));
    state.gentrification.rent_history = reindex_map(
        std::mem::take(&mut state.gentrification.rent_history),
        removed_id,
    );
    state.gentrification.demographic_shifts = reindex_map(
        std::mem::take(&mut state.gentrification.demographic_shifts),
        removed_id,
    );

    for mission in &mut state.missions.missions {
        if let MissionGoal::FullRepair { building_id } = &mut mission.goal {
            if *building_id == removed_id {
                if matches!(
                    mission.status,
                    MissionStatus::Available | MissionStatus::Active
                ) {
                    mission.fail();
                }
            } else if *building_id > removed_id {
                *building_id -= 1;
            }
        }
    }
    for event in &mut state.narrative_events.events {
        adjust_effect_building_id(&mut event.default_effect, removed_id);
        for choice in &mut event.choices {
            adjust_effect_building_id(&mut choice.effect, removed_id);
        }
    }
}

fn reindexed_id(id: u32, removed_id: u32) -> Option<u32> {
    match id.cmp(&removed_id) {
        std::cmp::Ordering::Less => Some(id),
        std::cmp::Ordering::Equal => None,
        std::cmp::Ordering::Greater => Some(id - 1),
    }
}

fn reindex_id_mut(id: &mut u32, removed_id: u32) -> bool {
    if let Some(new_id) = reindexed_id(*id, removed_id) {
        *id = new_id;
        true
    } else {
        false
    }
}

fn reindex_map<T>(
    values: std::collections::HashMap<u32, T>,
    removed_id: u32,
) -> std::collections::HashMap<u32, T> {
    values
        .into_iter()
        .filter_map(|(id, value)| reindexed_id(id, removed_id).map(|id| (id, value)))
        .collect()
}

fn adjust_effect_building_id(effect: &mut NarrativeEffect, removed_id: u32) {
    let id = match effect {
        NarrativeEffect::BuildingHappiness { building_id, .. }
        | NarrativeEffect::TriggerInspection { building_id }
        | NarrativeEffect::PropertyValue { building_id, .. }
        | NarrativeEffect::SellBuilding { building_id } => Some(building_id),
        NarrativeEffect::Multiple { effects } => {
            for effect in effects {
                adjust_effect_building_id(effect, removed_id);
            }
            None
        }
        _ => None,
    };
    if let Some(id) = id {
        if *id == removed_id {
            *effect = NarrativeEffect::None;
        } else if *id > removed_id {
            *id -= 1;
        }
    }
}

#[cfg(test)]
mod tests;

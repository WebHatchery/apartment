use super::*;
use crate::state::GameplayState;

#[test]
fn neighborhood_reputation_effect_moves_reputation() {
    let mut state = GameplayState::new();
    let nid = state.city.neighborhoods[0].id;
    let before = state.city.neighborhoods[0].reputation;
    state.apply_narrative_effect(&NarrativeEffect::NeighborhoodReputation {
        neighborhood_id: nid,
        change: 10,
    });
    assert_eq!(
        state.city.neighborhoods[0].reputation,
        (before + 10).clamp(0, 100)
    );
}

#[test]
fn economy_change_effect_clamps_to_boom_ceiling() {
    let mut state = GameplayState::new();
    state.city.economy_health = 1.4;
    state.apply_narrative_effect(&NarrativeEffect::EconomyChange {
        economy_health_change: 0.5,
    });
    assert!((state.city.economy_health - 1.5).abs() < f32::EPSILON);
}

#[test]
fn rent_demand_effect_moves_neighborhood_demand() {
    let mut state = GameplayState::new();
    let nid = state.city.neighborhoods[0].id;
    let before = state.city.neighborhoods[0].stats.rent_demand;
    state.apply_narrative_effect(&NarrativeEffect::RentDemand {
        neighborhood_id: nid,
        change: 0.1,
    });
    assert!(state.city.neighborhoods[0].stats.rent_demand > before);
}

#[test]
fn property_value_effect_scales_rent_ceiling() {
    let mut state = GameplayState::new();
    state.building.rent_multiplier = 1.0;
    state.apply_narrative_effect(&NarrativeEffect::PropertyValue {
        building_id: 0,
        change_percent: 10.0,
    });
    assert!((state.building.rent_multiplier - 1.1).abs() < 0.001);
}

#[test]
fn property_value_effect_updates_the_named_non_active_building() {
    let mut state = GameplayState::new();
    let second = crate::building::Building::new("Second", 1, 1);
    state.city.add_building(second, 0).unwrap();

    state.apply_narrative_effect(&NarrativeEffect::PropertyValue {
        building_id: 1,
        change_percent: 20.0,
    });

    assert!((state.city.buildings[1].rent_multiplier - 1.2).abs() < 0.001);
    assert!((state.building.rent_multiplier - 1.0).abs() < 0.001);
}

#[test]
fn inspection_effect_records_the_named_building() {
    let mut state = GameplayState::new();
    state
        .city
        .add_building(crate::building::Building::new("Second", 1, 1), 0)
        .unwrap();

    state.apply_narrative_effect(&NarrativeEffect::TriggerInspection { building_id: 1 });

    assert_eq!(
        state
            .compliance
            .inspection_history
            .last()
            .map(|inspection| inspection.building_id),
        Some(1)
    );
}

#[test]
fn portfolio_reindex_keeps_cross_system_building_ids_aligned() {
    let mut state = GameplayState::new();
    state
        .city
        .add_building(crate::building::Building::new("Second", 1, 1), 0)
        .unwrap();
    let mut tenant = crate::tenant::Tenant::new(
        99,
        "Second Resident",
        crate::tenant::TenantArchetype::Student,
    );
    tenant.building_id = 1;
    state.tenants.push(tenant);
    state.compliance.init_building_regulations(1, false);
    state.ever_occupied_buildings.insert(1);
    state.gentrification.rent_history.insert(1, Vec::new());
    state
        .tenant_network
        .apply_tension_change(1, 0, 1, 20, "test");

    state.city.buildings.remove(0);
    reindex_building_references(&mut state, 0);

    assert_eq!(
        state.tenants.last().map(|tenant| tenant.building_id),
        Some(0)
    );
    assert!(state.compliance.building_regulations.contains_key(&0));
    assert_eq!(
        state.ever_occupied_buildings,
        std::collections::HashSet::from([0])
    );
    assert!(state.gentrification.rent_history.contains_key(&0));
    assert_eq!(state.tenant_network.tensions[0].building_id, 0);
}

#[test]
fn building_happiness_effect_shifts_all_tenants() {
    let mut state = GameplayState::new();
    if state.tenants.is_empty() {
        return; // No tenants to shift; the empty case simply must not panic.
    }
    for tenant in &mut state.tenants {
        tenant.happiness = 50;
    }
    state.apply_narrative_effect(&NarrativeEffect::BuildingHappiness {
        building_id: 0,
        change: -5,
    });
    assert!(state.tenants.iter().all(|t| t.happiness == 45));
}

#[test]
fn reputation_change_moves_active_neighborhood() {
    // Use the non-UI mutation helper: apply_reputation_change also pushes
    // floating text, which needs a macroquad GL context unit tests lack.
    let mut state = GameplayState::new();
    let before = state.active_neighborhood_reputation();
    state.adjust_active_neighborhood_reputation(10);
    assert_eq!(
        state.active_neighborhood_reputation(),
        (before + 10).clamp(0, 100)
    );
}

#[test]
fn same_seed_reproduces_initial_state() {
    use crate::data::config::load_config;
    use crate::data::templates::load_templates;

    let Some(template) = load_templates().and_then(|t| t.templates.into_iter().next()) else {
        return;
    };
    let a = GameplayState::new_with_template_seed(load_config(), template.clone(), 777);
    let b = GameplayState::new_with_template_seed(load_config(), template, 777);

    assert_eq!(a.seed, 777);
    assert_eq!(a.next_tenant_id, b.next_tenant_id);
    let archetypes = |s: &GameplayState| {
        s.applications
            .iter()
            .map(|app| format!("{:?}", app.tenant.archetype))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        archetypes(&a),
        archetypes(&b),
        "same seed must reproduce the same initial applicants"
    );
}

#[test]
fn non_active_buildings_contribute_passive_income() {
    use crate::building::Building;

    let mut state = GameplayState::new();
    // Add a second building (index 1, non-active) with known rents.
    let mut passive = Building::new("Passive Block", 2, 2); // 4 units
    for apt in &mut passive.apartments {
        apt.rent_price = 1000;
    }
    let _ = state.city.add_building(passive, 0);

    let before = state.funds.balance;
    state.collect_portfolio_passive_income();
    // 4 units × $1000 × 0.8 − 4 × $190 = 3200 − 760 = 2440 (positive).
    assert!(
        state.funds.balance > before,
        "an owned non-active building should earn passive income"
    );
}

#[test]
fn historic_building_carries_extra_preservation_regulation() {
    use crate::data::config::load_config;
    use crate::data::templates::load_templates;

    let templates = load_templates().map(|t| t.templates).unwrap_or_default();
    let historic = templates.iter().find(|t| t.neighborhood_id == 3).cloned();
    let plain = templates.iter().find(|t| t.neighborhood_id != 3).cloned();
    let (Some(historic), Some(plain)) = (historic, plain) else {
        return;
    };

    let hstate = GameplayState::new_with_template(load_config(), historic);
    let pstate = GameplayState::new_with_template(load_config(), plain);
    let reg_count = |s: &GameplayState| {
        s.compliance
            .building_regulations
            .values()
            .map(|v| v.len())
            .max()
            .unwrap_or(0)
    };
    assert!(
        reg_count(&hstate) > reg_count(&pstate),
        "historic building should carry the extra preservation regulation ({} vs {})",
        reg_count(&hstate),
        reg_count(&pstate)
    );
}

#[test]
fn condo_sale_multiplier_tracks_the_market() {
    let mut state = GameplayState::new();
    state.city.economy_health = 1.5; // boom
    let boom = state.condo_sale_market_multiplier();
    state.city.economy_health = 0.5; // recession
    let bust = state.condo_sale_market_multiplier();
    assert!(boom > state.config.gentrification.condo_sale_equity_share);
    assert!(bust < state.config.gentrification.condo_sale_equity_share);
    assert!(boom > bust);
}

#[test]
fn application_multiplier_scales_with_reputation() {
    let mut state = GameplayState::new();
    state.adjust_active_neighborhood_reputation(-50); // drive toward 0
    let low = state.application_reputation_multiplier();
    state.adjust_active_neighborhood_reputation(100); // drive toward 100
    let high = state.application_reputation_multiplier();
    assert!(low < 1.0, "poor reputation should suppress applicants");
    assert!(high > 1.0, "strong reputation should draw applicants");
}

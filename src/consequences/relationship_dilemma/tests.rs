use super::*;
use crate::building::Building;
use crate::tenant::TenantArchetype;

fn make_building() -> Building {
    Building::new("Test", 2, 2)
}

fn place_tenant(id: u32, name: &str, building: &mut Building, apt_index: usize) -> Tenant {
    let mut tenant = Tenant::new(id, name, TenantArchetype::Professional);
    let apt = &mut building.apartments[apt_index];
    apt.tenant_id = Some(id);
    tenant.apartment_id = Some(apt.id);
    tenant
}

fn setup() -> (TenantNetwork, Vec<Tenant>, Building, DilemmaConfig) {
    let mut building = make_building();
    for apt in &mut building.apartments {
        apt.rent_price = 500;
    }
    let sarah = place_tenant(1, "Sarah", &mut building, 0);
    let alex = place_tenant(2, "Alex", &mut building, 1);
    let kim = place_tenant(3, "Kim", &mut building, 2);
    building.apartments[0].rent_price = 900; // Sarah pays a premium

    let mut network = TenantNetwork::new();
    network.apply_relationship_change(1, 2, -40); // Sarah-Alex hostile
    network.apply_relationship_change(1, 3, -40); // Sarah-Kim hostile

    (
        network,
        vec![sarah, alex, kim],
        building,
        DilemmaConfig::default(),
    )
}

#[test]
fn detects_high_rent_disruptor() {
    let (network, tenants, building, config) = setup();
    let info = network
        .find_disruptor(&tenants, &building, &config, 10)
        .expect("Sarah should qualify");
    assert_eq!(info.tenant_id, 1);
    assert_eq!(info.rent, 900);
    let mut victims = info.victim_ids.clone();
    victims.sort();
    assert_eq!(victims, vec![2, 3]);
}

#[test]
fn average_rent_troublemaker_is_not_a_dilemma() {
    let (network, tenants, mut building, config) = setup();
    building.apartments[0].rent_price = 500; // No premium, no dilemma
    assert!(network
        .find_disruptor(&tenants, &building, &config, 10)
        .is_none());
}

#[test]
fn cooldown_suppresses_repeat_dilemmas() {
    let (mut network, tenants, building, config) = setup();
    network.dilemma_history.insert(1, 8);
    assert!(network
        .find_disruptor(&tenants, &building, &config, 10)
        .is_none());
    assert!(network
        .find_disruptor(&tenants, &building, &config, 8 + config.cooldown_months)
        .is_some());
}

#[test]
fn placeholders_substitute_and_effects_resolve() {
    let (network, tenants, building, config) = setup();
    let info = network
        .find_disruptor(&tenants, &building, &config, 10)
        .unwrap();

    let template = RelationshipEventTemplate {
        id: "test".to_string(),
        trigger_strength_min: None,
        trigger_strength_max: None,
        probability: 100,
        headline: "The {tenant} Problem".to_string(),
        description: "{tenant} in {apt} pays ${rent}, {victim_count} suffer: {victims}".to_string(),
        choices: vec![
            crate::narrative::relationship_config::RelationshipChoiceTemplate {
                label: "Evict {tenant}".to_string(),
                description: String::new(),
                effect: NarrativeEffect::Multiple {
                    effects: vec![
                        NarrativeEffect::MoveOut { tenant_id: 0 },
                        NarrativeEffect::TenantHappiness {
                            tenant_id: 1,
                            change: 20,
                        },
                    ],
                },
                reputation_change: 0,
            },
        ],
        default_effect: None,
    };

    let event = generate_dilemma_event(&template, &info, &tenants, &building).unwrap();
    assert_eq!(event.headline, "The Sarah Problem");
    assert!(event.description.contains("$900"));
    assert!(event.description.contains("2 suffer"));
    assert!(event.description.contains("Alex"));
    assert!(event.requires_response);
    assert_eq!(event.choices[0].label, "Evict Sarah");

    match &event.choices[0].effect {
        NarrativeEffect::Multiple { effects } => {
            assert!(matches!(
                effects[0],
                NarrativeEffect::MoveOut { tenant_id: 1 }
            ));
            // Victim happiness fans out to both hostile neighbors
            match &effects[1] {
                NarrativeEffect::Multiple { effects: fanned } => {
                    assert_eq!(fanned.len(), 2);
                }
                other => panic!("expected fan-out, got {:?}", other),
            }
        }
        other => panic!("expected Multiple, got {:?}", other),
    }
}

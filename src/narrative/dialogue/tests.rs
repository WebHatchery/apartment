use super::*;

#[test]
fn test_dialogue_creation() {
    let mut system = DialogueSystem::new();
    let choices = vec![DialogueChoice {
        text: "Yes".to_string(),
        effects: vec![DialogueEffect::MoneyChange(100)],
    }];

    let id = system.add_dialogue(
        DialogueType::FaceToFaceRequest,
        1,
        None,
        "Test",
        "Test Desc",
        choices,
        None,
    );

    assert_eq!(system.pending_dialogues().len(), 1);
    assert_eq!(system.pending_dialogues()[0].id, id);
}

#[test]
fn rent_negotiation_uses_the_tenants_actual_rent_ceiling() {
    use crate::tenant::{Tenant, TenantArchetype};

    let mut tenant = Tenant::new(1, "Renter", TenantArchetype::Student);
    tenant.rent_tolerance = 800;
    tenant.happiness = 40;

    assert!(!rent_negotiation_eligible(&tenant, 800));
    assert!(rent_negotiation_eligible(&tenant, 801));
}

#[test]
fn conflict_mediation_generated_from_hostile_pair() {
    use crate::consequences::TenantNetwork;
    use crate::tenant::{Tenant, TenantArchetype};

    let tenants = vec![
        Tenant::generate(1, TenantArchetype::Professional),
        Tenant::generate(2, TenantArchetype::Artist),
    ];
    let mut network = TenantNetwork::new();
    // A strong negative change with no prior relationship creates a Hostile one.
    network.apply_relationship_change(1, 2, -60);

    let mut system = DialogueSystem::new();
    let bodies = load_dialogue_bodies();
    system.generate_conflict_mediation(&tenants, &network, &bodies);

    let dialogue = system
        .active_dialogues
        .iter()
        .find(|d| d.dialogue_type == DialogueType::ConflictMediation);
    let dialogue = dialogue.expect("a conflict dialogue should be generated");
    assert_eq!(dialogue.target_id, Some(2));
    // {initiator}/{target} placeholders are substituted with tenant names.
    assert!(!dialogue.description.contains('{'));
    assert!(!dialogue.choices.is_empty());
}

#[test]
fn dialogue_bodies_load_from_json() {
    let bodies = load_dialogue_bodies();
    // Every archetype has at least one face-to-face body, plus a default.
    for key in [
        "Student",
        "Professional",
        "Artist",
        "Family",
        "Elderly",
        "default",
    ] {
        let list = bodies
            .face_to_face
            .get(key)
            .unwrap_or_else(|| panic!("no face_to_face bodies for {}", key));
        assert!(!list.is_empty(), "{} has an empty body list", key);
    }
    // At least one archetype offers multiple request variants.
    assert!(
        bodies.face_to_face.values().any(|v| v.len() >= 2),
        "expected some archetypes to have multiple request bodies"
    );
    assert!(bodies.conflict_mediation.is_some());
    assert!(bodies.rent_negotiation.is_some());
}

#[test]
fn test_dialogue_resolution() {
    let mut system = DialogueSystem::new();
    let choices = vec![DialogueChoice {
        text: "Yes".to_string(),
        effects: vec![DialogueEffect::MoneyChange(100)],
    }];

    let id = system.add_dialogue(
        DialogueType::FaceToFaceRequest,
        1,
        None,
        "Test",
        "Test Desc",
        choices,
        None,
    );

    let effects = system.resolve_dialogue(id, 0);
    assert!(effects.is_some(), "expected dialogue effects");
    if let Some(effects) = effects {
        assert_eq!(effects.len(), 1);
        assert!(
            matches!(effects[0], DialogueEffect::MoneyChange(100)),
            "expected money change effect"
        );
    }

    assert_eq!(system.pending_dialogues().len(), 0);
}

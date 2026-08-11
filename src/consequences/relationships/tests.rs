use super::*;

#[test]
fn test_relationship_modifiers() {
    let config = RelationshipsConfig::default();
    assert!(RelationshipType::Friendly.happiness_modifier(&config) > 0);
    assert!(RelationshipType::Hostile.happiness_modifier(&config) < 0);
}

#[test]
fn test_network_basics() {
    let mut network = TenantNetwork::new();
    network.add_relationship(1, 2, RelationshipType::Friendly);

    assert!(network.relationship_between(1, 2).is_some());
    assert!(network.relationship_between(2, 1).is_some());
    assert!(network.relationship_between(1, 3).is_none());
}

#[test]
fn tenants_in_different_buildings_cannot_form_a_relationship() {
    let mut tenant_a = crate::tenant::Tenant::new(1, "A", crate::tenant::TenantArchetype::Student);
    let mut tenant_b = crate::tenant::Tenant::new(2, "B", crate::tenant::TenantArchetype::Student);
    tenant_a.move_into_building(0, 0);
    tenant_b.move_into_building(1, 1);

    assert!(!TenantRelationship::can_form(&tenant_a, &tenant_b));
}

#[test]
fn apartment_tensions_are_scoped_to_a_building() {
    let mut network = TenantNetwork::new();
    network.apply_tension_change(0, 0, 1, 20, "Noise");
    network.apply_tension_change(1, 0, 1, 30, "Music");

    assert_eq!(network.tensions.len(), 2);
    assert_eq!(network.tensions[0].building_id, 0);
    assert_eq!(network.tensions[1].building_id, 1);
}

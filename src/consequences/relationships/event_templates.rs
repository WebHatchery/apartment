//! Turns relationship event templates into concrete narrative events.

use super::{TenantNetwork, TenantRelationship};
use crate::narrative::events::{NarrativeChoice, NarrativeEffect, NarrativeEventType};
use crate::narrative::relationship_config::RelationshipEventTemplate;
use crate::narrative::NarrativeEvent;

impl TenantNetwork {
    pub(super) fn generate_event_from_template(
        &self,
        template: &RelationshipEventTemplate,
        rel: &TenantRelationship,
        tenants: &[crate::tenant::Tenant],
        building: &crate::building::Building,
    ) -> Option<NarrativeEvent> {
        let tenant_a = tenants.iter().find(|t| t.id == rel.tenant_a_id)?;
        let tenant_b = tenants.iter().find(|t| t.id == rel.tenant_b_id)?;

        let apt_a = tenant_a
            .apartment_id
            .and_then(|id| building.get_apartment(id))
            .map(|a| format!("Apt {}", a.unit_number))
            .unwrap_or("Unknown".to_string());
        let apt_b = tenant_b
            .apartment_id
            .and_then(|id| building.get_apartment(id))
            .map(|a| format!("Apt {}", a.unit_number))
            .unwrap_or("Unknown".to_string());

        let description = template
            .description
            .replace("{tenant_a}", &tenant_a.name)
            .replace("{tenant_b}", &tenant_b.name)
            .replace("{apt_a}", &apt_a)
            .replace("{apt_b}", &apt_b);

        let choices: Vec<NarrativeChoice> = template
            .choices
            .iter()
            .map(|c| NarrativeChoice {
                label: c
                    .label
                    .replace("{tenant_a}", &tenant_a.name)
                    .replace("{tenant_b}", &tenant_b.name),
                description: c
                    .description
                    .replace("{tenant_a}", &tenant_a.name)
                    .replace("{tenant_b}", &tenant_b.name),
                effect: self.resolve_effect(&c.effect, rel.tenant_a_id, rel.tenant_b_id),
                reputation_change: c.reputation_change,
            })
            .collect();

        let default_effect = template
            .default_effect
            .as_ref()
            .map(|e| self.resolve_effect(e, rel.tenant_a_id, rel.tenant_b_id))
            .unwrap_or(NarrativeEffect::None);

        Some(NarrativeEvent {
            id: 0, // Will be set by system
            event_type: NarrativeEventType::RelationshipEvent,
            month: 0, // Will be set by caller
            headline: template.headline.clone(),
            description,
            choices,
            default_effect,
            read: false,
            requires_response: !template.choices.is_empty(),
            response_deadline: if !template.choices.is_empty() {
                Some(2)
            } else {
                None
            },
            related_neighborhood_id: None, // Could link to neighborhood?
        })
    }

    fn resolve_effect(&self, effect: &NarrativeEffect, a: u32, b: u32) -> NarrativeEffect {
        match effect {
            NarrativeEffect::RelationshipStrength {
                tenant_a_id,
                tenant_b_id,
                change,
            } => {
                // If IDs are 0, replace with actual
                let ta = if *tenant_a_id == 0 { a } else { *tenant_a_id };
                let tb = if *tenant_b_id == 0 { b } else { *tenant_b_id };
                NarrativeEffect::RelationshipStrength {
                    tenant_a_id: ta,
                    tenant_b_id: tb,
                    change: *change,
                }
            }
            NarrativeEffect::TenantHappiness { tenant_id, change } => {
                let t_resolved = if *tenant_id == 0 {
                    a
                } else if *tenant_id == 1 {
                    b
                } else {
                    *tenant_id
                };
                NarrativeEffect::TenantHappiness {
                    tenant_id: t_resolved,
                    change: *change,
                }
            }
            NarrativeEffect::OpinionChange { tenant_id, amount } => {
                let t_resolved = if *tenant_id == 0 {
                    a
                } else if *tenant_id == 1 {
                    b
                } else {
                    *tenant_id
                };
                NarrativeEffect::OpinionChange {
                    tenant_id: t_resolved,
                    amount: *amount,
                }
            }
            NarrativeEffect::MoveOut { tenant_id } => {
                let t_resolved = if *tenant_id == 0 {
                    a
                } else if *tenant_id == 1 {
                    b
                } else {
                    *tenant_id
                };
                NarrativeEffect::MoveOut {
                    tenant_id: t_resolved,
                }
            }
            NarrativeEffect::Multiple { effects } => NarrativeEffect::Multiple {
                effects: effects
                    .iter()
                    .map(|e| self.resolve_effect(e, a, b))
                    .collect(),
            },
            _ => effect.clone(),
        }
    }
}

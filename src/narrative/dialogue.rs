use macroquad_toolkit::rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DialogueType {
    /// High-priority tenant issue (broken heater, pest infestation)
    FaceToFaceRequest,
    /// Tenant A complaining about Tenant B
    ConflictMediation,
    /// Rent change conversations
    RentNegotiation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DialogueEffect {
    /// Change tenant happiness
    HappinessChange { tenant_id: u32, amount: i32 },
    /// Gain or lose money
    MoneyChange(i32),
    /// Change tension between apartments
    TensionChange { apt_a: u32, apt_b: u32, amount: i32 },
    /// Change relationship between tenants
    RelationshipChange {
        tenant_a: u32,
        tenant_b: u32,
        change: i32,
    },
    /// Change landlord opinion
    OpinionChange { tenant_id: u32, amount: i32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub text: String,
    pub effects: Vec<DialogueEffect>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveDialogue {
    pub id: u32,
    pub dialogue_type: DialogueType,
    pub initiator_id: u32,
    /// Other tenant involved (if conflict)
    pub target_id: Option<u32>,
    pub headline: String,
    pub description: String,
    pub choices: Vec<DialogueChoice>,
    /// When auto-resolves (if ignored)
    pub deadline_month: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueSystem {
    pub active_dialogues: Vec<ActiveDialogue>,
    next_id: u32,
}

impl DialogueSystem {
    pub fn new() -> Self {
        Self {
            active_dialogues: Vec::new(),
            next_id: 1,
        }
    }

    /// Queue a new dialogue
    pub fn add_dialogue(
        &mut self,
        dialogue_type: DialogueType,
        initiator: u32,
        target: Option<u32>,
        headline: &str,
        description: &str,
        choices: Vec<DialogueChoice>,
        deadline: Option<u32>,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.active_dialogues.push(ActiveDialogue {
            id,
            dialogue_type,
            initiator_id: initiator,
            target_id: target,
            headline: headline.to_string(),
            description: description.to_string(),
            choices,
            deadline_month: deadline,
        });

        id
    }

    /// Get dialogues needing response
    pub fn pending_dialogues(&self) -> Vec<&ActiveDialogue> {
        self.active_dialogues.iter().collect()
    }

    /// Apply selected choice and return effects
    pub fn resolve_dialogue(
        &mut self,
        dialogue_id: u32,
        choice_index: usize,
    ) -> Option<Vec<DialogueEffect>> {
        if let Some(index) = self
            .active_dialogues
            .iter()
            .position(|d| d.id == dialogue_id)
        {
            let dialogue = self.active_dialogues.remove(index);

            if let Some(choice) = dialogue.choices.get(choice_index) {
                return Some(choice.effects.clone());
            }
        }
        None
    }

    /// Generate dialogues based on game state
    pub fn generate_dialogues(
        &mut self,
        month: u32,
        tenants: &[crate::tenant::Tenant],
        building: &crate::building::Building,
        funds: &crate::economy::PlayerFunds,
        network: &crate::consequences::TenantNetwork,
    ) {
        // Dialogue copy, choices, and effects are data-driven
        // (assets/dialogue_bodies.json).
        let bodies = load_dialogue_bodies();
        self.generate_conflict_mediation(tenants, network, &bodies);
        self.generate_rent_negotiations(building, tenants, &bodies);

        // Low funds shave the repair cost the tenant is quoted.
        let is_low_on_funds = funds.balance < 500;
        let building_quality = building.building_appeal();

        // Small chance for random face-to-face request from unhappy tenants
        for tenant in tenants {
            // More likely to complain if building quality is low or it's been many months
            let complaint_chance = if building_quality < 50 { 10 } else { 5 };
            let months_factor = if month > 12 { 2 } else { 0 };

            if tenant.happiness < 40 && rng::gen_range(0, 100) < (complaint_chance + months_factor)
            {
                // Pick a random request body from the archetype's list, falling
                // back to the "default" list.
                let key = tenant.archetype.name();
                let list = bodies
                    .face_to_face
                    .get(key)
                    .filter(|v| !v.is_empty())
                    .or_else(|| bodies.face_to_face.get("default"));
                let Some(template) = list.and_then(|v| rng::choose(v)) else {
                    continue;
                };

                // Avoid duplicates
                if self
                    .active_dialogues
                    .iter()
                    .any(|d| d.initiator_id == tenant.id)
                {
                    continue;
                }

                let repair_cost = if is_low_on_funds { 30 } else { 50 };
                let ctx = DialogueContext {
                    initiator_id: tenant.id,
                    target_id: None,
                    initiator_name: tenant.name.clone(),
                    target_name: String::new(),
                    repair_cost,
                };
                self.add_dialogue(
                    DialogueType::FaceToFaceRequest,
                    tenant.id,
                    None,
                    &substitute(&template.headline, &ctx),
                    &substitute(&template.description, &ctx),
                    build_choices(template, &ctx),
                    None,
                );
            }
        }
    }

    /// Tenant A complaining about Tenant B — sourced from a real hostile
    /// relationship in the tenant network. Generates one conflict at a time.
    fn generate_conflict_mediation(
        &mut self,
        tenants: &[crate::tenant::Tenant],
        network: &crate::consequences::TenantNetwork,
        bodies: &DialogueBodies,
    ) {
        use crate::consequences::RelationshipType;

        let Some(template) = &bodies.conflict_mediation else {
            return;
        };

        let housed = |id: u32| tenants.iter().any(|t| t.id == id);
        let name_of = |id: u32| {
            tenants
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "a neighbor".to_string())
        };

        let pair = network.relationships.iter().find(|r| {
            matches!(r.relationship_type, RelationshipType::Hostile)
                && housed(r.tenant_a_id)
                && housed(r.tenant_b_id)
                && !self
                    .active_dialogues
                    .iter()
                    .any(|d| d.initiator_id == r.tenant_a_id)
        });

        let Some(relationship) = pair else {
            return;
        };
        let (a, b) = (relationship.tenant_a_id, relationship.tenant_b_id);
        let ctx = DialogueContext {
            initiator_id: a,
            target_id: Some(b),
            initiator_name: name_of(a),
            target_name: name_of(b),
            repair_cost: 0,
        };

        self.add_dialogue(
            DialogueType::ConflictMediation,
            a,
            Some(b),
            &substitute(&template.headline, &ctx),
            &substitute(&template.description, &ctx),
            build_choices(template, &ctx),
            None,
        );
    }

    /// Rent-change conversations: price-sensitive tenants push back when the
    /// building charges above baseline rent.
    fn generate_rent_negotiations(
        &mut self,
        building: &crate::building::Building,
        tenants: &[crate::tenant::Tenant],
        bodies: &DialogueBodies,
    ) {
        let Some(template) = &bodies.rent_negotiation else {
            return;
        };
        for tenant in tenants {
            let Some(apartment) = tenant
                .apartment_id
                .and_then(|apartment_id| building.get_apartment(apartment_id))
            else {
                continue;
            };
            if !rent_negotiation_eligible(tenant, apartment.rent_price) {
                continue;
            }
            if self
                .active_dialogues
                .iter()
                .any(|d| d.initiator_id == tenant.id)
            {
                continue;
            }
            if rng::gen_range(0, 100) >= 6 {
                continue;
            }

            let ctx = DialogueContext {
                initiator_id: tenant.id,
                target_id: None,
                initiator_name: tenant.name.clone(),
                target_name: String::new(),
                repair_cost: 0,
            };
            self.add_dialogue(
                DialogueType::RentNegotiation,
                tenant.id,
                None,
                &substitute(&template.headline, &ctx),
                &substitute(&template.description, &ctx),
                build_choices(template, &ctx),
                None,
            );
        }
    }

    /// Handle expiring dialogues
    pub fn tick(&mut self, current_month: u32) {
        // Remove expired dialogues
        self.active_dialogues.retain(|d| {
            if let Some(deadline) = d.deadline_month {
                deadline > current_month
            } else {
                true
            }
        });
    }
}

fn rent_negotiation_eligible(tenant: &crate::tenant::Tenant, rent: i32) -> bool {
    use crate::tenant::TenantArchetype;

    rent > tenant.rent_tolerance
        && tenant.happiness < 55
        && matches!(
            tenant.archetype,
            TenantArchetype::Elderly | TenantArchetype::Family | TenantArchetype::Student
        )
}

impl Default for DialogueSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// A data-driven dialogue effect. The concrete `DialogueEffect` is built at
/// generation time so runtime tenant ids can be injected — static content can't
/// know which tenants are involved.
#[derive(Clone, Debug, Deserialize)]
struct DialogueEffectSpec {
    kind: String,
    /// "initiator" (the tenant who raised the dialogue, the default) or "target"
    /// (the other tenant, e.g. in a conflict).
    #[serde(default)]
    target: String,
    #[serde(default)]
    amount: i32,
}

#[derive(Clone, Debug, Deserialize)]
struct DialogueChoiceTemplate {
    text: String,
    effects: Vec<DialogueEffectSpec>,
}

#[derive(Clone, Debug, Deserialize)]
struct DialogueBodyTemplate {
    headline: String,
    description: String,
    choices: Vec<DialogueChoiceTemplate>,
}

/// All authored dialogue bodies (`assets/dialogue_bodies.json`).
#[derive(Clone, Debug, Deserialize, Default)]
struct DialogueBodies {
    /// Face-to-face requests keyed by archetype name (each a list of possible
    /// bodies, picked at random), with a `"default"` fallback list.
    #[serde(default)]
    face_to_face: std::collections::HashMap<String, Vec<DialogueBodyTemplate>>,
    #[serde(default)]
    conflict_mediation: Option<DialogueBodyTemplate>,
    #[serde(default)]
    rent_negotiation: Option<DialogueBodyTemplate>,
}

/// Runtime values substituted into a dialogue template at generation time.
struct DialogueContext {
    initiator_id: u32,
    target_id: Option<u32>,
    initiator_name: String,
    target_name: String,
    repair_cost: i32,
}

fn substitute(text: &str, ctx: &DialogueContext) -> String {
    text.replace("{initiator}", &ctx.initiator_name)
        .replace("{target}", &ctx.target_name)
        .replace("{cost}", &ctx.repair_cost.to_string())
}

fn resolve_effect_spec(spec: &DialogueEffectSpec, ctx: &DialogueContext) -> Option<DialogueEffect> {
    let target_id = if spec.target == "target" {
        ctx.target_id
    } else {
        Some(ctx.initiator_id)
    };
    match spec.kind.as_str() {
        "happiness" => target_id.map(|id| DialogueEffect::HappinessChange {
            tenant_id: id,
            amount: spec.amount,
        }),
        "opinion" => target_id.map(|id| DialogueEffect::OpinionChange {
            tenant_id: id,
            amount: spec.amount,
        }),
        "money" => Some(DialogueEffect::MoneyChange(spec.amount)),
        // The repair quote varies with the landlord's funds, resolved at runtime.
        "repair_money" => Some(DialogueEffect::MoneyChange(-ctx.repair_cost)),
        "relationship" => ctx.target_id.map(|t| DialogueEffect::RelationshipChange {
            tenant_a: ctx.initiator_id,
            tenant_b: t,
            change: spec.amount,
        }),
        _ => None,
    }
}

fn build_choices(template: &DialogueBodyTemplate, ctx: &DialogueContext) -> Vec<DialogueChoice> {
    template
        .choices
        .iter()
        .map(|choice| DialogueChoice {
            text: substitute(&choice.text, ctx),
            effects: choice
                .effects
                .iter()
                .filter_map(|spec| resolve_effect_spec(spec, ctx))
                .collect(),
        })
        .collect()
}

fn load_dialogue_bodies() -> DialogueBodies {
    #[cfg(target_arch = "wasm32")]
    let json =
        macroquad_toolkit::include_json_str!("../../assets/dialogue_bodies.json").to_string();

    #[cfg(not(target_arch = "wasm32"))]
    let json = std::fs::read_to_string("assets/dialogue_bodies.json").unwrap_or_else(|_| {
        macroquad_toolkit::include_json_str!("../../assets/dialogue_bodies.json").to_string()
    });

    serde_json::from_str(&json).unwrap_or_else(|e| {
        eprintln!("Failed to parse dialogue_bodies.json: {}", e);
        DialogueBodies::default()
    })
}

#[cfg(test)]
mod tests;

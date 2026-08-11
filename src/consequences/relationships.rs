use crate::data::config::RelationshipsConfig;
use crate::narrative::events::{NarrativeChoice, NarrativeEffect, NarrativeEventType};
use crate::narrative::relationship_config::RelationshipEventTemplate;
use crate::narrative::{NarrativeEvent, RelationshipChange, RelationshipEventsConfig};
use macroquad_toolkit::rng;
use serde::{Deserialize, Serialize};

/// Type of relationship between tenants
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    /// Friendly neighbors who help each other
    Friendly,
    /// Neutral - no strong feelings either way
    Neutral,
    /// Conflict - noise complaints, disputes
    Hostile,
    /// Romantic relationship (may combine units)
    Romantic,
    /// Family connection
    Family,
}

impl RelationshipType {
    pub fn happiness_modifier(&self, config: &RelationshipsConfig) -> i32 {
        let key = match self {
            RelationshipType::Friendly => "friendly",
            RelationshipType::Neutral => "neutral",
            RelationshipType::Hostile => "hostile",
            RelationshipType::Romantic => "romantic",
            RelationshipType::Family => "family",
        };
        *config.happiness_modifiers.get(key).unwrap_or(&0)
    }
}

/// A relationship between two tenants
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantRelationship {
    pub tenant_a_id: u32,
    pub tenant_b_id: u32,
    pub relationship_type: RelationshipType,
    /// How strong the relationship is (0-100)
    pub strength: i32,
    /// Months this relationship has existed
    pub duration_months: u32,
    /// Recent interactions that affected the relationship
    pub recent_events: Vec<String>,

    // Phase 4C: Landlord opinions
    pub landlord_opinion_a: i32, // How tenant A views landlord (-100 to 100)
    pub landlord_opinion_b: i32, // How tenant B views landlord
}

/// Dynamic tension between apartments (e.g., noise complaints)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SocialTension {
    #[serde(default)]
    pub building_id: u32,
    pub apartment_a: u32,
    pub apartment_b: u32,
    pub tension_level: i32, // 0-100
    pub cause: String,
}

impl TenantRelationship {
    pub fn new(tenant_a: u32, tenant_b: u32, initial_type: RelationshipType) -> Self {
        Self {
            tenant_a_id: tenant_a,
            tenant_b_id: tenant_b,
            relationship_type: initial_type,
            strength: 50,
            duration_months: 0,
            recent_events: Vec::new(),
            landlord_opinion_a: 0,
            landlord_opinion_b: 0,
        }
    }

    /// Apply monthly relationship dynamics
    pub fn tick(&mut self, config: &RelationshipsConfig) {
        self.duration_months += 1;

        // Long-term relationships tend to strengthen
        if self.duration_months > 6 && !matches!(self.relationship_type, RelationshipType::Hostile)
        {
            self.strength = (self.strength + 1).min(100);
        }

        // Hostile relationships can cool down over time
        if matches!(self.relationship_type, RelationshipType::Hostile)
            && rng::gen_range(0, 100) < config.hostile_cooldown_chance
        {
            self.strength = (self.strength - config.hostile_strength_decay).max(0);
            if self.strength < config.hostile_transition_threshold {
                self.relationship_type = RelationshipType::Neutral;
                self.recent_events.push("Conflict cooled down".to_string());
            }
        }

        // Clean up old events
        while self.recent_events.len() > 5 {
            self.recent_events.remove(0);
        }
    }

    /// Can these tenants potentially form this relationship?
    pub fn can_form(tenant_a: &crate::tenant::Tenant, tenant_b: &crate::tenant::Tenant) -> bool {
        // Relationships are local to one building, between different units.
        tenant_a.building_id == tenant_b.building_id
            && tenant_a.apartment_id != tenant_b.apartment_id
    }
}

/// Manages all tenant relationships in a building
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantNetwork {
    pub relationships: Vec<TenantRelationship>,
    /// Track tenant history for displacement detection
    pub long_term_tenants: Vec<LongTermTenantRecord>,

    // Phase 4C: Social Tension
    pub tensions: Vec<SocialTension>,

    /// Month each tenant last starred in a dilemma event (cooldown tracking)
    #[serde(default)]
    pub dilemma_history: std::collections::HashMap<u32, u32>,
}

/// Record of a long-term tenant's history
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LongTermTenantRecord {
    pub tenant_id: u32,
    pub tenant_name: String,
    pub archetype: crate::tenant::TenantArchetype,
    pub months_at_move_in: u32,
    pub original_rent: i32,
    pub current_rent: i32,
    pub is_displaced: bool,
    pub displacement_reason: Option<String>,
}

impl TenantNetwork {
    pub fn new() -> Self {
        Self {
            relationships: Vec::new(),
            long_term_tenants: Vec::new(),
            tensions: Vec::new(),
            dilemma_history: std::collections::HashMap::new(),
        }
    }

    /// Get relationship between two specific tenants
    fn relationship_between(&self, tenant_a: u32, tenant_b: u32) -> Option<&TenantRelationship> {
        self.relationships.iter().find(|r| {
            (r.tenant_a_id == tenant_a && r.tenant_b_id == tenant_b)
                || (r.tenant_a_id == tenant_b && r.tenant_b_id == tenant_a)
        })
    }

    fn relationship_between_mut(
        &mut self,
        tenant_a: u32,
        tenant_b: u32,
    ) -> Option<&mut TenantRelationship> {
        self.relationships.iter_mut().find(|r| {
            (r.tenant_a_id == tenant_a && r.tenant_b_id == tenant_b)
                || (r.tenant_a_id == tenant_b && r.tenant_b_id == tenant_a)
        })
    }

    /// Create a new relationship
    fn add_relationship(
        &mut self,
        tenant_a: u32,
        tenant_b: u32,
        rel_type: RelationshipType,
    ) -> Option<RelationshipType> {
        if self.relationship_between(tenant_a, tenant_b).is_none() {
            let rel = TenantRelationship::new(tenant_a, tenant_b, rel_type.clone());
            self.relationships.push(rel);
            Some(rel_type)
        } else {
            None
        }
    }

    /// Apply a direct change to social tension between apartments.
    pub fn apply_tension_change(
        &mut self,
        building_id: u32,
        apt_a: u32,
        apt_b: u32,
        amount: i32,
        cause: &str,
    ) {
        if apt_a == apt_b || amount == 0 {
            return;
        }

        let existing = self.tensions.iter_mut().find(|tension| {
            tension.building_id == building_id
                && ((tension.apartment_a == apt_a && tension.apartment_b == apt_b)
                    || (tension.apartment_a == apt_b && tension.apartment_b == apt_a))
        });

        if let Some(tension) = existing {
            tension.tension_level = (tension.tension_level + amount).clamp(0, 100);
            if !cause.is_empty() {
                tension.cause = cause.to_string();
            }
        } else if amount > 0 {
            self.tensions.push(SocialTension {
                building_id,
                apartment_a: apt_a,
                apartment_b: apt_b,
                tension_level: amount.clamp(0, 100),
                cause: cause.to_string(),
            });
        }

        self.tensions.retain(|tension| tension.tension_level > 0);
    }

    /// Apply a direct strength change between two tenants, creating a relationship if needed.
    pub fn apply_relationship_change(&mut self, tenant_a: u32, tenant_b: u32, change: i32) {
        if tenant_a == tenant_b || change == 0 {
            return;
        }

        if let Some(relationship) = self.relationship_between_mut(tenant_a, tenant_b) {
            relationship.strength = (relationship.strength + change).clamp(0, 100);
            relationship
                .recent_events
                .push(format!("Dialogue changed relationship by {:+}", change));
            update_relationship_type_from_strength(relationship, change);
            return;
        }

        let rel_type = if change > 0 {
            RelationshipType::Friendly
        } else {
            RelationshipType::Hostile
        };
        let mut relationship = TenantRelationship::new(tenant_a, tenant_b, rel_type);
        relationship.strength = (50 + change).clamp(0, 100);
        relationship
            .recent_events
            .push(format!("Dialogue created relationship at {:+}", change));
        self.relationships.push(relationship);
    }

    /// Process monthly relationship dynamics
    pub fn tick(
        &mut self,
        tenants: &[crate::tenant::Tenant],
        building: &crate::building::Building,
        config: &RelationshipsConfig,
        events_config: &RelationshipEventsConfig,
        current_month: u32,
    ) -> (Vec<RelationshipChange>, Vec<NarrativeEvent>) {
        let mut changes = Vec::new();
        let mut events = Vec::new(); // Phase 4
        let active_tenant_ids: std::collections::HashSet<u32> =
            tenants.iter().map(|tenant| tenant.id).collect();

        // Update existing relationships
        for relationship in self.relationships.iter_mut().filter(|relationship| {
            active_tenant_ids.contains(&relationship.tenant_a_id)
                && active_tenant_ids.contains(&relationship.tenant_b_id)
        }) {
            relationship.tick(config);

            // Phase 4D: Detect relationship changes (e.g. Hostile -> Neutral)
            // This would require tracking old state, which we can add later
        }

        // Chance to form new relationships between neighbors
        for tenant_a in tenants {
            for tenant_b in tenants {
                if tenant_a.id >= tenant_b.id {
                    continue; // Avoid duplicates
                }

                // Check if they're already related
                if self
                    .relationship_between(tenant_a.id, tenant_b.id)
                    .is_some()
                {
                    continue;
                }

                // Check if they can form a relationship
                if !TenantRelationship::can_form(tenant_a, tenant_b) {
                    continue;
                }

                // Chance per month for new relationship
                if rng::gen_range(0, 100) < config.formation_chance {
                    let rel_type =
                        self.determine_initial_relationship(tenant_a, tenant_b, building, config);
                    if let Some(actual_type) =
                        self.add_relationship(tenant_a.id, tenant_b.id, rel_type.clone())
                    {
                        let is_positive = !matches!(actual_type, RelationshipType::Hostile);

                        changes.push(RelationshipChange::NewRelationship {
                            tenant_a_name: tenant_a.name.clone(),
                            tenant_b_name: tenant_b.name.clone(),
                            relationship_type: format!("{:?}", actual_type),
                            is_positive,
                        });
                    }
                }
            }
        }

        // Phase 4: Generate relationship events
        for rel in &self.relationships {
            let possible_events = match rel.relationship_type {
                RelationshipType::Hostile => &events_config.hostile,
                RelationshipType::Friendly => &events_config.friendly,
                RelationshipType::Romantic | RelationshipType::Family => &events_config.romance,
                _ => continue,
            };

            for template in possible_events {
                // Check probability
                if rng::gen_range(0, 100) >= template.probability {
                    continue;
                }

                // Check triggers
                if let Some(min) = template.trigger_strength_min {
                    if rel.strength < min {
                        continue;
                    }
                }
                if let Some(max) = template.trigger_strength_max {
                    if rel.strength > max {
                        continue;
                    }
                }

                // Generate event
                if let Some(event) =
                    self.generate_event_from_template(template, rel, tenants, building)
                {
                    events.push(event);
                }
            }
        }

        // Emergent dilemma: a premium-rent tenant whose hostile relationships
        // are draining the building forces a keep-or-evict choice.
        if let Some(event) = self.maybe_generate_dilemma(
            tenants,
            building,
            &config.dilemma,
            events_config,
            current_month,
        ) {
            events.push(event);
        }

        (changes, events)
    }

    fn generate_event_from_template(
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

    /// Determine what kind of relationship forms between two tenants
    fn determine_initial_relationship(
        &self,
        tenant_a: &crate::tenant::Tenant,
        tenant_b: &crate::tenant::Tenant,
        building: &crate::building::Building,
        config: &RelationshipsConfig,
    ) -> RelationshipType {
        use crate::tenant::TenantArchetype;

        // Get their apartments
        let apt_a = tenant_a
            .apartment_id
            .and_then(|id| building.get_apartment(id));
        let apt_b = tenant_b
            .apartment_id
            .and_then(|id| building.get_apartment(id));

        // Noise conflicts: Artists/Students in noisy units + Professionals/Elderly nearby
        let noisy_type_a = matches!(
            tenant_a.archetype,
            TenantArchetype::Artist | TenantArchetype::Student
        );
        let quiet_type_b = matches!(
            tenant_b.archetype,
            TenantArchetype::Professional | TenantArchetype::Elderly | TenantArchetype::Family
        );

        if (noisy_type_a && quiet_type_b)
            || (!noisy_type_a && !quiet_type_b && rng::gen_range(0, 100) < 20)
        {
            // Check if apartments are adjacent (same floor or floor ±1)
            if let (Some(a), Some(b)) = (apt_a, apt_b) {
                if (a.floor as i32 - b.floor as i32).abs() <= 1
                    && rng::gen_range(0, 100) < config.adjacent_hostile_chance
                {
                    return RelationshipType::Hostile;
                }
            }
        }

        // Same archetype tends to be friendly
        if tenant_a.archetype == tenant_b.archetype
            && rng::gen_range(0, 100) < config.same_archetype_friendly_chance
        {
            return RelationshipType::Friendly;
        }

        // Families tend to connect
        if matches!(tenant_a.archetype, TenantArchetype::Family)
            && matches!(tenant_b.archetype, TenantArchetype::Family)
        {
            return RelationshipType::Friendly;
        }

        // Default to neutral
        RelationshipType::Neutral
    }

    /// Calculate community cohesion bonus based on matching archetypes
    pub fn calculate_cohesion(
        &self,
        tenants: &[crate::tenant::Tenant],
        config: &crate::data::config::CohesionConfig,
    ) -> i32 {
        if tenants.is_empty() {
            return 0;
        }

        let mut archetype_counts = std::collections::HashMap::new();
        for tenant in tenants {
            *archetype_counts
                .entry(tenant.archetype.clone())
                .or_insert(0) += 1;
        }

        let mut bonus = 0;

        // Bonus for having significant groups of same archetype
        for (_, count) in archetype_counts {
            if count >= config.archetype_group_threshold {
                bonus += config.archetype_group_base_bonus
                    + (count - config.archetype_group_threshold) * config.archetype_group_per_extra;
            }
        }

        // Bonus for friendly relationships
        let tenant_ids: std::collections::HashSet<u32> =
            tenants.iter().map(|tenant| tenant.id).collect();
        let building_id = tenants[0].building_id;
        let friendly_count = self
            .relationships
            .iter()
            .filter(|r| {
                tenant_ids.contains(&r.tenant_a_id)
                    && tenant_ids.contains(&r.tenant_b_id)
                    && matches!(
                        r.relationship_type,
                        RelationshipType::Friendly | RelationshipType::Family
                    )
            })
            .count() as i32;

        bonus += friendly_count * config.friendly_relationship_bonus;

        // Penalty for tensions/hostility
        let hostile_count = self
            .relationships
            .iter()
            .filter(|r| {
                tenant_ids.contains(&r.tenant_a_id)
                    && tenant_ids.contains(&r.tenant_b_id)
                    && matches!(r.relationship_type, RelationshipType::Hostile)
            })
            .count() as i32;

        bonus -= hostile_count * config.hostile_relationship_penalty;
        let tension_count = self
            .tensions
            .iter()
            .filter(|tension| tension.building_id == building_id)
            .count() as i32;
        bonus -= tension_count * config.tension_penalty;

        bonus.clamp(config.cohesion_min, config.cohesion_max)
    }

    /// Check if tenants are unhappy enough to form a council
    pub fn should_form_council(
        &self,
        tenants: &[crate::tenant::Tenant],
        config: &crate::data::config::GentrificationConfig,
        unhappy_threshold: i32,
    ) -> bool {
        if tenants.len() < config.council_min_tenants {
            return false;
        }

        let unhappy_count = tenants
            .iter()
            .filter(|t| t.is_unhappy(unhappy_threshold))
            .count();
        let relative_unhappiness = unhappy_count as f32 / tenants.len() as f32;

        // Formation threshold from config
        relative_unhappiness >= config.council_formation_threshold
    }
}

fn update_relationship_type_from_strength(relationship: &mut TenantRelationship, change: i32) {
    if matches!(
        relationship.relationship_type,
        RelationshipType::Family | RelationshipType::Romantic
    ) {
        return;
    }

    if relationship.strength >= 70 && change > 0 {
        relationship.relationship_type = RelationshipType::Friendly;
    } else if relationship.strength <= 30 && change < 0 {
        relationship.relationship_type = RelationshipType::Hostile;
    } else if relationship.strength > 30 && relationship.strength < 70 {
        relationship.relationship_type = RelationshipType::Neutral;
    }
}

impl Default for TenantNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;

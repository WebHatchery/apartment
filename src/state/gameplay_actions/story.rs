//! Story, request and dialogue consequences applied to residents and funds.

use macroquad_toolkit::rng;

use crate::narrative::StoryImpact;
use crate::simulation::GameEvent;
use crate::state::GameplayState;

impl GameplayState {
    pub(in crate::state) fn apply_story_impact(&mut self, tenant_id: u32, impact: StoryImpact) {
        let mut stack = vec![impact];
        while let Some(effect) = stack.pop() {
            match effect {
                StoryImpact::None | StoryImpact::Request(_) | StoryImpact::Roommate => {}
                StoryImpact::LifeChange(life_change) => {
                    // Expand a life change into its concrete consequences and
                    // process them through the same pipeline.
                    let (impact, _description) = life_change.impact(&self.config.life_events);
                    stack.push(impact);
                }
                StoryImpact::Happiness(amount) => {
                    if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == tenant_id) {
                        tenant.happiness = (tenant.happiness + amount).clamp(0, 100);
                    }
                }
                StoryImpact::RentTolerance(amount) => {
                    if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == tenant_id) {
                        tenant.rent_tolerance = (tenant.rent_tolerance + amount).max(100);
                    }
                }
                StoryImpact::MoveOutRisk(chance) => {
                    if rng::gen_range(0, 100) < chance {
                        let tenant_name = if let Some(tenant) =
                            self.tenants.iter_mut().find(|t| t.id == tenant_id)
                        {
                            tenant.happiness = 0;
                            Some(tenant.name.clone())
                        } else {
                            None
                        };

                        if let Some(tenant_name) = tenant_name {
                            self.event_log.log(
                                GameEvent::TenantUnhappy {
                                    tenant_name,
                                    happiness: 0,
                                },
                                self.current_tick,
                            );
                        }
                    }
                }
                StoryImpact::SetApartmentFlag(flag) => {
                    if let Some(apt) = self
                        .building
                        .apartments
                        .iter_mut()
                        .find(|apartment| apartment.tenant_id == Some(tenant_id))
                    {
                        apt.flags.insert(flag);
                    }
                }
                StoryImpact::Multiple(sub_effects) => {
                    stack.extend(sub_effects);
                }
            }
        }
    }

    pub(in crate::state) fn apply_dialogue_effect(
        &mut self,
        effect: crate::narrative::dialogue::DialogueEffect,
    ) {
        match effect {
            crate::narrative::dialogue::DialogueEffect::HappinessChange { tenant_id, amount } => {
                if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == tenant_id) {
                    tenant.happiness = (tenant.happiness + amount).clamp(0, 100);
                }
            }
            crate::narrative::dialogue::DialogueEffect::MoneyChange(amount) => {
                self.apply_dialogue_money_change(amount);
            }
            crate::narrative::dialogue::DialogueEffect::TensionChange {
                apt_a,
                apt_b,
                amount,
            } => {
                self.tenant_network.apply_tension_change(
                    self.active_building_id(),
                    apt_a,
                    apt_b,
                    amount,
                    "Dialogue choice",
                );
            }
            crate::narrative::dialogue::DialogueEffect::RelationshipChange {
                tenant_a,
                tenant_b,
                change,
            } => {
                self.tenant_network
                    .apply_relationship_change(tenant_a, tenant_b, change);
            }
            crate::narrative::dialogue::DialogueEffect::OpinionChange { tenant_id, amount } => {
                if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == tenant_id) {
                    tenant.landlord_opinion = (tenant.landlord_opinion + amount).clamp(-100, 100);
                }
            }
        }
    }

    pub(in crate::state) fn apply_dialogue_money_change(&mut self, amount: i32) {
        if amount > 0 {
            self.funds.add_income(crate::economy::Transaction::income(
                crate::economy::TransactionType::Grant,
                amount,
                "Dialogue Reward",
                self.current_tick,
            ));
        } else {
            self.funds
                .apply_required_expense(crate::economy::Transaction::expense(
                    crate::economy::TransactionType::CriticalFailure,
                    amount.abs(),
                    "Dialogue Cost",
                    self.current_tick,
                ));
        }
    }
}

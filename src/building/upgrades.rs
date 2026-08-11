use super::{Apartment, ApartmentSize, Building, DesignType};
use crate::data::config::{
    EconomyConfig, UiConfig, UpgradeDefinition, UpgradeRequirement, UpgradeTarget,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UpgradeAction {
    RepairApartment {
        apartment_id: u32,
        amount: i32,
    },
    RepairHallway {
        amount: i32,
    },
    // Generic upgrade identified by ID (from config.json)
    Apply {
        upgrade_id: String,
        target_id: Option<u32>,
    },
}

impl UpgradeAction {
    /// Get a user-friendly label for this action
    pub fn label(
        &self,
        _building: &Building,
        config: &UiConfig,
        upgrades: &HashMap<String, UpgradeDefinition>,
    ) -> String {
        match self {
            UpgradeAction::RepairApartment { amount, .. } => {
                let fmt = config
                    .upgrade_labels
                    .get("repair_fmt")
                    .map(|s| s.as_str())
                    .unwrap_or("Repair +{}");
                fmt.replace("{}", &amount.to_string())
            }
            UpgradeAction::RepairHallway { amount } => {
                let fmt = config
                    .upgrade_labels
                    .get("repair_hallway_fmt")
                    .map(|s| s.as_str())
                    .unwrap_or("Repair Hallway +{}");
                fmt.replace("{}", &amount.to_string())
            }
            UpgradeAction::Apply { upgrade_id, .. } => upgrades
                .get(upgrade_id)
                .map(|u| u.name.clone())
                .unwrap_or_else(|| upgrade_id.clone()),
        }
    }

    /// Calculate the cost of this action
    pub fn cost(
        &self,
        _building: &Building,
        config: &EconomyConfig,
        upgrades: &HashMap<String, UpgradeDefinition>,
    ) -> Option<i32> {
        match self {
            UpgradeAction::RepairApartment { amount, .. } => {
                Some(amount * config.repair_cost_per_point)
            }
            UpgradeAction::RepairHallway { amount } => {
                Some(amount * config.hallway_repair_cost_per_point)
            }
            UpgradeAction::Apply {
                upgrade_id,
                target_id: _,
            } => {
                let base_cost = upgrades.get(upgrade_id).map(|u| u.cost)?;

                Some(base_cost)
            }
        }
    }
}

/// Apply an upgrade action to the building
/// Returns the cost if successful, None if failed
pub fn apply_upgrade(
    building: &mut Building,
    action: &UpgradeAction,
    upgrades: &HashMap<String, UpgradeDefinition>,
) -> Option<()> {
    match action {
        UpgradeAction::RepairApartment {
            apartment_id,
            amount,
        } => {
            let apt = building.get_apartment_mut(*apartment_id)?;
            apt.repair(*amount);
            Some(())
        }
        UpgradeAction::RepairHallway { amount } => {
            building.repair_hallway(*amount);
            Some(())
        }
        UpgradeAction::Apply {
            upgrade_id,
            target_id,
        } => {
            let def = upgrades.get(upgrade_id)?;
            match def.target {
                UpgradeTarget::Apartment => {
                    let apt_id = (*target_id)?;
                    let apt = building.get_apartment_mut(apt_id)?;

                    // Apply generic effects
                    for effect in &def.effects {
                        match effect {
                            crate::data::config::UpgradeEffect::SetFlag(flag) => {
                                apt.flags.insert(flag.clone());
                                if flag == "has_soundproofing" {
                                    apt.has_soundproofing = true;
                                }
                                if flag == "has_renovated_kitchen" && apt.kitchen_level < 1 {
                                    apt.kitchen_level = 2;
                                }
                            }
                            crate::data::config::UpgradeEffect::RemoveFlag(flag) => {
                                apt.flags.remove(flag);
                                if flag == "has_soundproofing" {
                                    apt.has_soundproofing = false;
                                }
                                if flag == "has_renovated_kitchen" {
                                    apt.kitchen_level = 0;
                                }
                            }
                            crate::data::config::UpgradeEffect::SetDesign(design_str) => {
                                match design_str.as_str() {
                                    "Bare" => apt.design = DesignType::Bare,
                                    "Practical" => apt.design = DesignType::Practical,
                                    "Cozy" => apt.design = DesignType::Cozy,
                                    "Luxury" => apt.design = DesignType::Luxury,
                                    "Opulent" => apt.design = DesignType::Opulent,
                                    _ => {}
                                }
                            }
                        }
                    }
                    Some(())
                }
                UpgradeTarget::Building => {
                    for effect in &def.effects {
                        match effect {
                            crate::data::config::UpgradeEffect::SetFlag(flag) => {
                                building.flags.insert(flag.clone());
                                if flag == "has_laundry" {
                                    building.has_laundry = true;
                                }
                            }
                            crate::data::config::UpgradeEffect::RemoveFlag(flag) => {
                                building.flags.remove(flag);
                                if flag == "has_laundry" {
                                    building.has_laundry = false;
                                }
                            }
                            crate::data::config::UpgradeEffect::SetDesign(_) => return None,
                        }
                    }
                    Some(())
                }
            }
        }
    }
}

pub fn available_apartment_upgrades(
    apt: &Apartment,
    upgrades: &HashMap<String, UpgradeDefinition>,
) -> Vec<UpgradeAction> {
    let mut actions = Vec::new();

    // 1. Repair (hardcoded logic for now as it's variable amount)
    if apt.condition < 100 {
        let amount = (100 - apt.condition).min(10);
        actions.push(UpgradeAction::RepairApartment {
            apartment_id: apt.id,
            amount,
        });
    }

    // 2. Generic Upgrades (includes Design upgrades now)
    for (id, def) in upgrades {
        if def.target == UpgradeTarget::Apartment
            && check_requirements(&def.requirements, apt, None)
        {
            actions.push(UpgradeAction::Apply {
                upgrade_id: id.clone(),
                target_id: Some(apt.id),
            });
        }
    }

    actions.sort_by(|a, b| upgrade_sort_key(a).cmp(&upgrade_sort_key(b)));

    actions
}

pub fn available_building_upgrades(
    building: &Building,
    upgrades: &HashMap<String, UpgradeDefinition>,
) -> Vec<UpgradeAction> {
    let mut actions = Vec::new();

    // 1. Repair Hallway
    if building.hallway_condition < 100 {
        let amount = (100 - building.hallway_condition).min(10);
        actions.push(UpgradeAction::RepairHallway { amount });
    }

    // 2. Generic Upgrades
    for (id, def) in upgrades {
        if def.target == UpgradeTarget::Building
            && check_requirements_building(&def.requirements, building)
        {
            actions.push(UpgradeAction::Apply {
                upgrade_id: id.clone(),
                target_id: None,
            });
        }
    }

    actions.sort_by(|a, b| upgrade_sort_key(a).cmp(&upgrade_sort_key(b)));

    actions
}

fn upgrade_sort_key(action: &UpgradeAction) -> (u8, &str) {
    match action {
        UpgradeAction::RepairApartment { .. } | UpgradeAction::RepairHallway { .. } => (0, ""),
        UpgradeAction::Apply { upgrade_id, .. } => (1, upgrade_id),
    }
}

pub(crate) fn check_requirements(
    reqs: &[UpgradeRequirement],
    apt: &Apartment,
    _building: Option<&Building>,
) -> bool {
    for req in reqs {
        match req {
            UpgradeRequirement::MissingFlag(flag) => {
                // Check new generic flags AND backwards compatibility hardcoded fields
                if apt.flags.contains(flag) {
                    return false;
                }
                if flag == "has_soundproofing" && apt.has_soundproofing {
                    return false;
                }
                if flag == "has_renovated_kitchen" && apt.kitchen_level >= 2 {
                    return false;
                }
            }
            UpgradeRequirement::HasFlag(flag) => {
                let has = apt.flags.contains(flag)
                    || (flag == "has_soundproofing" && apt.has_soundproofing)
                    || (flag == "has_renovated_kitchen" && apt.kitchen_level >= 2);
                if !has {
                    return false;
                }
            }
            UpgradeRequirement::HasDesign(design_str) => {
                let current = match apt.design {
                    DesignType::Bare => "Bare",
                    DesignType::Practical => "Practical",
                    DesignType::Cozy => "Cozy",
                    DesignType::Luxury => "Luxury",
                    DesignType::Opulent => "Opulent",
                };
                if current != design_str {
                    return false;
                }
            }
            UpgradeRequirement::MissingDesign(design_str) => {
                let current = match apt.design {
                    DesignType::Bare => "Bare",
                    DesignType::Practical => "Practical",
                    DesignType::Cozy => "Cozy",
                    DesignType::Luxury => "Luxury",
                    DesignType::Opulent => "Opulent",
                };
                if current == design_str {
                    return false;
                }
            }
            UpgradeRequirement::MinSize(size_str) => {
                let current_val = match apt.size {
                    ApartmentSize::Small => 0,
                    ApartmentSize::Medium => 1,
                    ApartmentSize::Large => 2,
                    ApartmentSize::Penthouse => 3,
                };

                let req_val = match size_str.to_lowercase().as_str() {
                    "small" => 0,
                    "medium" => 1,
                    "large" => 2,
                    "penthouse" => 3,
                    _ => 0, // Fallback
                };

                if current_val < req_val {
                    return false;
                }
            }
        }
    }
    true
}

pub(crate) fn check_requirements_building(
    reqs: &[UpgradeRequirement],
    building: &Building,
) -> bool {
    for req in reqs {
        match req {
            UpgradeRequirement::MissingFlag(flag) => {
                if building.flags.contains(flag) {
                    return false;
                }
                if flag == "has_laundry" && building.has_laundry {
                    return false;
                }
            }
            UpgradeRequirement::HasFlag(flag) => {
                let has = building.flags.contains(flag)
                    || (flag == "has_laundry" && building.has_laundry);
                if !has {
                    return false;
                }
            }
            UpgradeRequirement::HasDesign(_)
            | UpgradeRequirement::MissingDesign(_)
            | UpgradeRequirement::MinSize(_) => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests;

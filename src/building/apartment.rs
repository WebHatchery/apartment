use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DesignType {
    Bare,
    Practical,
    Cozy,
    Luxury,
    Opulent,
}

impl DesignType {
    /// Design appeal score (affects tenant happiness)
    pub fn appeal_score(&self) -> i32 {
        let config = crate::data::config::active().apartment;
        match self {
            DesignType::Bare => config.design_appeal_bare,
            DesignType::Practical => config.design_appeal_practical,
            DesignType::Cozy => config.design_appeal_cozy,
            DesignType::Luxury => config.design_appeal_luxury,
            DesignType::Opulent => config.design_appeal_opulent,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ApartmentSize {
    Small,
    Medium,
    Large,
    Penthouse,
}

impl ApartmentSize {
    pub fn base_rent(&self) -> i32 {
        let config = crate::data::config::active().apartment;
        match self {
            ApartmentSize::Small => config.base_rent_small,
            ApartmentSize::Medium => config.base_rent_medium,
            ApartmentSize::Large => config.base_rent_large,
            ApartmentSize::Penthouse => config.base_rent_penthouse,
        }
    }

    pub fn space_score(&self) -> i32 {
        let config = crate::data::config::active().apartment;
        match self {
            ApartmentSize::Small => config.space_score_small,
            ApartmentSize::Medium => config.space_score_medium,
            ApartmentSize::Large => config.space_score_large,
            ApartmentSize::Penthouse => config.space_score_penthouse,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NoiseLevel {
    Low,
    High,
}

impl NoiseLevel {
    pub fn noise_penalty(&self) -> i32 {
        match self {
            NoiseLevel::Low => 0,
            NoiseLevel::High => crate::data::config::active().apartment.noise_penalty_high,
        }
    }
}

use crate::tenant::TenantArchetype;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Apartment {
    pub id: u32,
    pub unit_number: String, // e.g., "1A", "2B"
    pub floor: u32,

    // Core stats
    pub condition: i32, // 0-100
    pub design: DesignType,
    pub size: ApartmentSize,
    pub base_noise: NoiseLevel, // Inherent noise (street-facing, etc.)
    pub has_soundproofing: bool,
    pub kitchen_level: i32, // 0=Basic, 1=Renovated, 2=Luxury
    pub rent_price: i32,

    // Occupancy
    pub tenant_id: Option<u32>,
    pub flags: HashSet<String>,

    // Leasing
    pub is_listed_for_lease: bool,
    pub preferred_archetype: Option<TenantArchetype>,
}

impl Apartment {
    pub fn new(
        id: u32,
        unit_number: &str,
        floor: u32,
        size: ApartmentSize,
        base_noise: NoiseLevel,
    ) -> Self {
        let rent_price = size.base_rent();
        Self {
            id,
            unit_number: unit_number.to_string(),
            floor,
            condition: 50, // Start at half condition
            design: DesignType::Bare,
            size,
            base_noise,
            has_soundproofing: false,
            kitchen_level: 0,
            rent_price,
            tenant_id: None,
            flags: HashSet::new(),
            is_listed_for_lease: false,
            preferred_archetype: None,
        }
    }

    /// Current effective noise level (considers soundproofing)
    pub fn effective_noise(&self) -> NoiseLevel {
        if self.has_soundproofing {
            NoiseLevel::Low
        } else if self.flags.contains("high_noise") {
            NoiseLevel::High
        } else {
            self.base_noise.clone()
        }
    }

    /// Is the apartment currently vacant?
    pub fn is_vacant(&self) -> bool {
        self.tenant_id.is_none()
    }

    /// Calculate overall apartment quality score (0-100)
    pub fn quality_score(&self) -> i32 {
        let base = self.condition;
        let design_bonus = self.design.appeal_score();
        let noise_mod = self.effective_noise().noise_penalty();
        let space_bonus = self.size.space_score();
        let kitchen_bonus = self.kitchen_level * 15;
        let lighting_bonus = if self.flags.contains("has_better_lighting") {
            crate::data::config::active()
                .apartment
                .lighting_quality_bonus
        } else {
            0
        };

        (base + design_bonus + noise_mod + space_bonus + kitchen_bonus + lighting_bonus)
            .clamp(0, 100)
    }

    /// Apply condition decay (called each tick)
    pub fn decay_condition(&mut self, amount: i32) {
        self.condition = (self.condition - amount).max(0);
    }

    /// Repair the apartment
    pub fn repair(&mut self, amount: i32) {
        self.condition = (self.condition + amount).min(100);
    }

    /// Move a tenant in
    pub fn move_in(&mut self, tenant_id: u32) {
        self.tenant_id = Some(tenant_id);
        self.is_listed_for_lease = false;
        self.preferred_archetype = None;
    }

    /// Move tenant out
    pub fn move_out(&mut self) {
        self.tenant_id = None;
    }

    /// Calculate market value for selling the unit
    /// Takes into account: size, condition, design, kitchen, floor, soundproofing
    pub fn market_value(&self) -> i32 {
        let config = crate::data::config::active().apartment;

        // Base price by size
        let base_price = match self.size {
            ApartmentSize::Small => config.market_base_small,
            ApartmentSize::Medium => config.market_base_medium,
            ApartmentSize::Large => config.market_base_large,
            ApartmentSize::Penthouse => config.market_base_penthouse,
        };

        // Condition bonus: positive per point above 50, negative per point below 50
        let condition_bonus = if self.condition > 50 {
            (self.condition - 50) * config.market_condition_bonus_above_50_per_point
        } else {
            (self.condition - 50) * config.market_condition_bonus_below_50_per_point
        };

        // Design bonus
        let design_bonus = match self.design {
            DesignType::Bare => config.market_design_bonus_bare,
            DesignType::Practical => config.market_design_bonus_practical,
            DesignType::Cozy => config.market_design_bonus_cozy,
            DesignType::Luxury => config.market_design_bonus_luxury,
            DesignType::Opulent => config.market_design_bonus_opulent,
        };

        // Kitchen bonus
        let kitchen_bonus = match self.kitchen_level {
            0 => 0,
            1 => config.market_kitchen_bonus_level1,
            _ => config.market_kitchen_bonus_level2_plus, // Level 2+
        };

        // Floor bonus: higher floors worth more
        let floor_bonus = (self.floor as i32) * config.market_floor_bonus_per_floor;

        // Soundproofing bonus
        let soundproofing_bonus = if self.has_soundproofing {
            config.market_soundproofing_bonus
        } else {
            0
        };
        let lighting_bonus = if self.flags.contains("has_better_lighting") {
            config.market_lighting_bonus
        } else {
            0
        };

        // Noise penalty for noisy units without soundproofing
        let noise_penalty = match self.base_noise {
            NoiseLevel::High if !self.has_soundproofing => config.market_high_noise_penalty,
            _ => 0,
        };

        (base_price
            + condition_bonus
            + design_bonus
            + kitchen_bonus
            + floor_bonus
            + soundproofing_bonus
            + lighting_bonus
            + noise_penalty)
            .max(config.market_value_floor)
    }
}

#[cfg(test)]
mod tests;

use super::ownership::OwnershipType;
use super::{Apartment, ApartmentSize, NoiseLevel};
use crate::data::config::MarketingConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Marketing campaign types with different costs and target demographics
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum MarketingType {
    #[default]
    None, // No active marketing
    SocialMedia,    // Attracts Students/Artists
    LocalNewspaper, // Attracts Elderly/Families
    PremiumAgency,  // Attracts Professionals
}

impl MarketingType {
    pub fn monthly_cost(&self, config: &MarketingConfig) -> i32 {
        match self {
            MarketingType::None => config.none_cost,
            MarketingType::SocialMedia => config.social_media_cost,
            MarketingType::LocalNewspaper => config.local_newspaper_cost,
            MarketingType::PremiumAgency => config.premium_agency_cost,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MarketingType::None => "None",
            MarketingType::SocialMedia => "Social Media",
            MarketingType::LocalNewspaper => "Local Newspaper",
            MarketingType::PremiumAgency => "Premium Agency",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Building {
    pub name: String,
    pub apartments: Vec<Apartment>,
    pub hallway_condition: i32, // 0-100, affects building appeal
    pub rent_multiplier: f32,   // 0.5 - 2.0 default 1.0
    pub has_laundry: bool,      // Amenity
    pub ownership_model: OwnershipType,

    // Operating flags
    pub utilities_included: bool,
    pub insurance_active: bool,

    // Marketing & Tenant Acquisition
    pub marketing_strategy: MarketingType, // Current marketing approach
    pub open_house_remaining: u32,         // Months of open house bonus remaining
    pub flags: HashSet<String>,
}

impl Building {
    /// Create a new building with generated apartments
    pub fn new(name: &str, num_floors: u32, units_per_floor: u32) -> Self {
        let mut apartments = Vec::new();
        let mut id = 0;

        for floor in 1..=num_floors {
            for unit in 0..units_per_floor {
                let unit_letter = (b'A' + unit as u8) as char;
                let unit_number = format!("{}{}", floor, unit_letter);

                // Alternate sizes and noise levels for variety
                let size = if (floor + unit) % 2 == 0 {
                    ApartmentSize::Small
                } else {
                    ApartmentSize::Medium
                };

                // Ground floor and street-facing (A) units are noisier
                let noise = if floor == 1 || unit == 0 {
                    NoiseLevel::High
                } else {
                    NoiseLevel::Low
                };

                apartments.push(Apartment::new(id, &unit_number, floor, size, noise));
                id += 1;
            }
        }

        Self {
            name: name.to_string(),
            apartments,
            hallway_condition: 60, // Start slightly worn
            rent_multiplier: 1.0,
            has_laundry: false,
            ownership_model: OwnershipType::FullRental,

            // Defaults
            utilities_included: false,
            insurance_active: false,
            marketing_strategy: MarketingType::None,
            open_house_remaining: 0,
            flags: HashSet::new(),
        }
    }

    /// Create a building from a template
    pub fn from_template(template: &crate::data::templates::BuildingTemplate) -> Self {
        let mut apartments = Vec::new();
        for (id, apt_template) in template.apartments.iter().enumerate() {
            let mut apt = Apartment::new(
                id as u32,
                &apt_template.unit_number,
                apt_template.floor,
                apt_template.size(),
                apt_template.base_noise(),
            );

            // Apply template specifics
            apt.condition = apt_template.initial_condition;
            apt.rent_price = apt_template.initial_rent;
            apt.design = apt_template.initial_design();

            apartments.push(apt);
        }

        Self {
            name: template.name.clone(),
            apartments,
            hallway_condition: template.hallway_condition,
            rent_multiplier: 1.0,
            has_laundry: false, // Could be in template?
            ownership_model: OwnershipType::FullRental,
            utilities_included: false,
            insurance_active: false,
            marketing_strategy: MarketingType::None,
            open_house_remaining: 0,
            flags: HashSet::new(),
        }
    }

    /// Get apartment by ID
    pub fn get_apartment(&self, id: u32) -> Option<&Apartment> {
        self.apartments.iter().find(|a| a.id == id)
    }

    /// Get mutable apartment by ID
    pub fn get_apartment_mut(&mut self, id: u32) -> Option<&mut Apartment> {
        self.apartments.iter_mut().find(|a| a.id == id)
    }

    /// Get all vacant apartments
    pub fn vacant_apartments(&self) -> Vec<&Apartment> {
        self.apartments
            .iter()
            .filter(|a| a.is_vacant() && !self.is_unit_sold(a.id))
            .collect()
    }

    /// Count vacant units still owned and available to rent.
    pub fn vacancy_count(&self) -> usize {
        self.vacant_apartments().len()
    }

    /// Count occupied rental units. Sold condos are outside the rental roll.
    pub fn occupancy_count(&self) -> usize {
        self.apartments
            .iter()
            .filter(|a| !self.is_unit_sold(a.id) && !a.is_vacant())
            .count()
    }

    /// Number of units still owned by the player as rental inventory.
    pub fn rental_unit_count(&self) -> usize {
        self.apartments.len().saturating_sub(self.sold_unit_count())
    }

    pub fn has_full_rental_occupancy(&self) -> bool {
        let rental_units = self.rental_unit_count();
        rental_units > 0 && self.occupancy_count() == rental_units
    }

    /// Calculate overall building appeal (affects tenant applications)
    pub fn building_appeal(&self) -> i32 {
        let hallway_factor = self.hallway_condition / 2; // 0-50
        let avg_condition: i32 = if self.apartments.is_empty() {
            0
        } else {
            self.apartments.iter().map(|a| a.condition).sum::<i32>() / self.apartments.len() as i32
        };
        let avg_factor = avg_condition / 2; // 0-50

        let mut score = hallway_factor + avg_factor;

        if self.has_laundry {
            score += 10;
        }

        score.min(100)
    }

    /// Repair hallway
    pub fn repair_hallway(&mut self, amount: i32) {
        self.hallway_condition = (self.hallway_condition + amount).min(100);
    }

    /// Decay hallway condition
    pub fn decay_hallway(&mut self, amount: i32) {
        self.hallway_condition = (self.hallway_condition - amount).max(0);
    }

    /// Apply decay to all apartments and hallway using configured rates.
    pub fn apply_monthly_decay(&mut self, apartment_decay: i32, hallway_decay: i32) {
        for apt in &mut self.apartments {
            apt.decay_condition(apartment_decay);
        }
        self.decay_hallway(hallway_decay);
    }

    /// Calculate average condition of all apartments
    pub fn average_condition(&self) -> i32 {
        if self.apartments.is_empty() {
            return 0;
        }
        let total: i32 = self.apartments.iter().map(|a| a.condition).sum();
        total / self.apartments.len() as i32
    }

    /// Convert a rental unit to a condo (sell it)
    pub fn convert_unit_to_condo(
        &mut self,
        apartment_id: u32,
        owner_name: &str,
        sale_price: i32,
    ) -> bool {
        use super::ownership::CondoBoard;

        // Check if apartment exists
        if !self.apartments.iter().any(|a| a.id == apartment_id) {
            return false;
        }

        // Initialize board if rental
        let converted = match &mut self.ownership_model {
            OwnershipType::FullRental => {
                let mut board = CondoBoard::new();
                board.add_unit(apartment_id, owner_name, 200, sale_price); // $200 HOA default
                self.ownership_model = OwnershipType::MixedOwnership(board);
                true
            }
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
                // Check if already in board
                if board.units.iter().any(|u| u.apartment_id == apartment_id) {
                    return false; // Already owned
                }
                board.add_unit(apartment_id, owner_name, 200, sale_price);
                true
            }
            _ => false, // Can't convert from Coop/Social easily yet
        };
        if converted {
            self.normalize_condo_ownership();
        }
        converted
    }
    pub fn update_ownership(&mut self, current_month: u32) -> bool {
        match &mut self.ownership_model {
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
                board.collect_fees();
                board.resolve_votes(current_month);
                true
            }
            _ => false,
        }
    }

    /// Check if a specific apartment has been sold as a condo
    pub fn is_unit_sold(&self, apartment_id: u32) -> bool {
        match &self.ownership_model {
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
                board.units.iter().any(|u| u.apartment_id == apartment_id)
            }
            _ => false,
        }
    }

    pub fn sold_unit_count(&self) -> usize {
        match &self.ownership_model {
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
                board.units.len()
            }
            _ => 0,
        }
    }

    /// Get the condo info for a sold unit (owner name, HOA, purchase price)
    pub fn get_condo_info(&self, apartment_id: u32) -> Option<(String, i32)> {
        match &self.ownership_model {
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => board
                .units
                .iter()
                .find(|u| u.apartment_id == apartment_id)
                .map(|u| (u.owner_name.clone(), u.purchase_price)),
            _ => None,
        }
    }

    /// Quote a condo buyback without changing ownership.
    pub fn condo_buyback_price(&self, apartment_id: u32) -> Option<i32> {
        match &self.ownership_model {
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => board
                .units
                .iter()
                .find(|unit| unit.apartment_id == apartment_id)
                .map(|unit| (unit.purchase_price as f32 * 1.1) as i32),
            _ => None,
        }
    }

    /// Complete a previously validated condo buyback.
    pub fn complete_condo_buyback(&mut self, apartment_id: u32) -> bool {
        match &mut self.ownership_model {
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
                let Some(index) = board
                    .units
                    .iter()
                    .position(|unit| unit.apartment_id == apartment_id)
                else {
                    return false;
                };
                board.units.remove(index);
            }
            _ => return false,
        }

        self.normalize_condo_ownership();
        true
    }

    fn normalize_condo_ownership(&mut self) {
        let ownership = std::mem::take(&mut self.ownership_model);
        self.ownership_model = match ownership {
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
                if board.units.is_empty() {
                    OwnershipType::FullRental
                } else if board.units.len() == self.apartments.len() {
                    OwnershipType::FullCondo(board)
                } else {
                    OwnershipType::MixedOwnership(board)
                }
            }
            other => other,
        };
    }
}

#[cfg(test)]
mod tests;

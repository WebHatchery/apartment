use super::{Neighborhood, NeighborhoodType, PropertyMarket};
use crate::building::Building;
use macroquad_toolkit::rng;
use serde::{Deserialize, Serialize};

/// The city contains all neighborhoods and provides the top-level game world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct City {
    pub name: String,
    pub neighborhoods: Vec<Neighborhood>,
    pub buildings: Vec<Building>,
    pub market: PropertyMarket,

    /// Currently selected building index
    pub active_building_index: usize,

    /// Global economic factors
    pub economy_health: f32, // 0.5 = recession, 1.0 = normal, 1.5 = boom
    pub interest_rate: f32,  // Affects loan costs
    pub inflation_rate: f32, // Affects rent expectations

    /// City statistics
    pub total_months: u32,
    pub total_buildings_managed: u32,
}

impl City {
    /// Create a new city with default neighborhoods
    pub fn new(name: &str) -> Self {
        let neighborhoods = vec![
            Neighborhood::new(0, NeighborhoodType::Downtown, "Central District"),
            Neighborhood::new(1, NeighborhoodType::Suburbs, "Greenfield Heights"),
            Neighborhood::new(2, NeighborhoodType::Industrial, "Old Docks"),
            Neighborhood::new(3, NeighborhoodType::Historic, "Heritage Row"),
        ];

        Self {
            name: name.to_string(),
            neighborhoods,
            buildings: Vec::new(),
            market: PropertyMarket::new(),
            active_building_index: 0,
            economy_health: 1.0,
            interest_rate: 0.05,
            inflation_rate: 0.02,
            total_months: 0,
            total_buildings_managed: 0,
        }
    }

    /// Create a city with a starter building already assigned to a neighborhood.
    #[cfg(test)]
    pub fn with_starter_building(name: &str, neighborhood_id: u32) -> (Self, u32) {
        let mut city = Self::new(name);
        let building = Building::new("Starter Building", 2, 2);
        let building_id = city
            .add_building(building, neighborhood_id)
            .unwrap_or_else(|_| {
                let building_id = city.buildings.len() as u32;
                city.buildings.push(Building::new("Starter Building", 2, 2));
                city.total_buildings_managed += 1;
                building_id
            });
        city.active_building_index = building_id as usize;
        (city, building_id)
    }

    /// Get the currently active building
    pub fn active_building(&self) -> Option<&Building> {
        self.buildings.get(self.active_building_index)
    }

    /// Get mutable reference to the currently active building
    pub fn active_building_mut(&mut self) -> Option<&mut Building> {
        self.buildings.get_mut(self.active_building_index)
    }

    /// Switch to a different building
    pub fn switch_building(&mut self, index: usize) {
        if index < self.buildings.len() {
            self.active_building_index = index;
        }
    }

    /// Get neighborhood for a building
    pub fn neighborhood_for_building(&self, building_index: usize) -> Option<&Neighborhood> {
        let building_id = building_index as u32;
        self.neighborhoods
            .iter()
            .find(|n| n.building_ids.contains(&building_id))
    }

    /// Add a new building to a neighborhood
    pub fn add_building(
        &mut self,
        building: Building,
        neighborhood_id: u32,
    ) -> Result<u32, String> {
        // Check if neighborhood can accept more buildings
        let neighborhood = self
            .neighborhoods
            .iter_mut()
            .find(|n| n.id == neighborhood_id)
            .ok_or("Neighborhood not found")?;

        if !neighborhood.can_add_building() {
            return Err("Neighborhood is at capacity".to_string());
        }

        let building_id = self.buildings.len() as u32;
        self.buildings.push(building);
        neighborhood.add_building(building_id);
        self.total_buildings_managed += 1;

        Ok(building_id)
    }

    /// Get all buildings as a vector of (index, building, neighborhood_name)
    pub fn buildings_with_info(&self) -> Vec<(usize, &Building, String)> {
        self.buildings
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let neighborhood_name = self
                    .neighborhood_for_building(i)
                    .map(|n| n.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                (i, b, neighborhood_name)
            })
            .collect()
    }

    /// Monthly tick for all city systems
    pub fn tick(&mut self) {
        self.total_months += 1;

        // Update neighborhoods
        for neighborhood in &mut self.neighborhoods {
            neighborhood.tick();
        }

        // Refresh market listings periodically
        if self.total_months.is_multiple_of(3) {
            self.market.refresh_listings(&self.neighborhoods);
        }

        // Random economic events
        self.update_economy();
    }

    /// Update economic conditions
    fn update_economy(&mut self) {
        // Small random fluctuations
        let change = rng::gen_range(-5, 6) as f32 / 100.0;
        self.economy_health = (self.economy_health + change).clamp(0.5, 1.5);

        // Interest rates inversely track economy health
        let target_rate = 0.08 - (self.economy_health - 1.0) * 0.05;
        self.interest_rate =
            (self.interest_rate + (target_rate - self.interest_rate) * 0.1).clamp(0.02, 0.15);

        // Inflation tracks economy health
        self.inflation_rate = (self.economy_health - 0.7) * 0.05;
    }
}

impl Default for City {
    fn default() -> Self {
        Self::new("Metropolis")
    }
}

#[cfg(test)]
mod tests;

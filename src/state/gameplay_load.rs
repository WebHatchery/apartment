//! Post-load repair: rebuild skipped fields and restore save invariants.

use super::gameplay::{default_floating_text_layer, GameplayState, ViewMode};
use crate::narrative::{load_events_config, load_relationship_config, TenantStory};
use crate::ui::Selection;

impl GameplayState {
    /// Restore fields that are intentionally skipped from save data.
    pub fn post_load(&mut self) {
        if self.current_building_id.is_empty() {
            self.current_building_id = crate::data::templates::load_templates()
                .and_then(|templates| templates.templates.into_iter().next())
                .map(|template| template.id)
                .unwrap_or_else(|| "mvp_default".to_string());
        }
        self.ledger.align_unreported(&self.funds);
        self.config = crate::data::config::load_config();
        crate::release_mode::apply_config(&mut self.config);
        // config isn't serialized, so re-apply the building's difficulty
        // modifiers that were baked in at new-game time.
        if let Some(templates) = crate::data::templates::load_templates() {
            if let Some(template) = templates
                .templates
                .iter()
                .find(|t| t.id == self.current_building_id)
            {
                self.config.apply_difficulty(&template.difficulty);
            }
        }
        // Re-seed the shared RNG from the saved run seed so reloading a save
        // doesn't let the player reroll future random outcomes.
        macroquad_toolkit::rng::srand(self.seed);
        self.tenant_events_config = load_events_config();
        self.relationship_events_config = load_relationship_config();
        self.view_mode = ViewMode::Building;
        self.selection = Selection::None;
        self.pending_actions.clear();
        self.floating_texts = default_floating_text_layer();
        self.panel_scroll_offset = 0.0;
        self.selected_mail_id = None;
        self.inbox_page = 0;
        self.tasks_page = 0;
        self.city_page = 0;
        self.market_page = 0;
        self.selected_neighborhood_id = None;
        self.pending_property_purchase = None;
        self.application_page = 0;
        self.tenants_page = 0;
        self.requests_page = 0;
        self.career_badges_page = 0;
        self.show_pause_menu = false;
        self.activity_drawer_open = false;
        self.pending_quit_to_menu = false;
        self.active_world_events
            .retain(|event| event.remaining_ticks > 0);

        self.ensure_city_integrity();
        self.restore_tenant_building_ids();
        self.ever_occupied_buildings.extend(
            self.tenants
                .iter()
                .filter(|tenant| tenant.apartment_id.is_some())
                .map(|tenant| tenant.building_id),
        );
        self.sync_building();
        self.ensure_compliance_for_buildings();
        self.ensure_tenant_stories();

        if self.ever_occupied_buildings.is_empty() && self.has_ever_had_tenant {
            self.ever_occupied_buildings
                .insert(self.city.active_building_index as u32);
        }
    }

    fn ensure_city_integrity(&mut self) {
        if self.city.buildings.is_empty() {
            self.city.buildings.push(self.building.clone());
            self.city.active_building_index = 0;
        }

        if self.city.active_building_index >= self.city.buildings.len() {
            self.city.active_building_index = 0;
        }

        for building_id in 0..self.city.buildings.len() as u32 {
            let already_linked = self
                .city
                .neighborhoods
                .iter()
                .any(|neighborhood| neighborhood.building_ids.contains(&building_id));

            if already_linked {
                continue;
            }

            if let Some(neighborhood) = self
                .city
                .neighborhoods
                .iter_mut()
                .find(|neighborhood| neighborhood.can_add_building())
            {
                neighborhood.add_building(building_id);
            }
        }

        self.city.total_buildings_managed = self
            .city
            .total_buildings_managed
            .max(self.city.buildings.len() as u32);
    }

    fn ensure_compliance_for_buildings(&mut self) {
        let missing: Vec<(u32, bool)> = (0..self.city.buildings.len() as u32)
            .filter(|building_id| {
                !self
                    .compliance
                    .building_regulations
                    .contains_key(building_id)
            })
            .map(|building_id| {
                let is_historic = self
                    .city
                    .neighborhood_for_building(building_id as usize)
                    .is_some_and(|neighborhood| {
                        matches!(
                            neighborhood.neighborhood_type,
                            crate::city::NeighborhoodType::Historic
                        )
                    });
                (building_id, is_historic)
            })
            .collect();

        for (building_id, is_historic) in missing {
            self.compliance
                .init_building_regulations(building_id, is_historic);
        }
    }

    fn ensure_tenant_stories(&mut self) {
        for tenant in &self.tenants {
            self.tenant_stories
                .entry(tenant.id)
                .or_insert_with(|| TenantStory::generate(tenant.id, &tenant.archetype));
        }
    }

    /// Old saves predate portfolio-scoped tenant addresses. Apartment
    /// occupancy stores globally unique tenant IDs, so use it to recover the
    /// correct building whenever possible.
    fn restore_tenant_building_ids(&mut self) {
        for tenant in &mut self.tenants {
            if let Some((building_id, apartment_id)) = self
                .city
                .buildings
                .iter()
                .enumerate()
                .find_map(|(building_id, building)| {
                    building
                        .apartments
                        .iter()
                        .find(|apartment| apartment.tenant_id == Some(tenant.id))
                        .map(|apartment| (building_id as u32, apartment.id))
                })
            {
                tenant.building_id = building_id;
                tenant.apartment_id = Some(apartment_id);
            }
        }
    }
}

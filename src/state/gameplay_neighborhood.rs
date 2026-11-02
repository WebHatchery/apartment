//! Reads and writes of the active building's neighborhood standing, and the
//! market conditions derived from it.

use crate::ui::colors;
use macroquad::prelude::*;

use super::gameplay::GameplayState;

impl GameplayState {
    /// Nudge the visible reputation of the neighborhood the active building sits
    /// in, clamped to [0, 100].
    pub(super) fn adjust_active_neighborhood_reputation(&mut self, delta: i32) {
        let building_id = self.city.active_building_index as u32;
        self.adjust_neighborhood_reputation_for_building(building_id, delta);
    }

    pub(super) fn adjust_neighborhood_reputation_for_building(
        &mut self,
        building_id: u32,
        delta: i32,
    ) {
        if let Some(neighborhood) = self
            .city
            .neighborhoods
            .iter_mut()
            .find(|n| n.building_ids.contains(&building_id))
        {
            neighborhood.reputation = (neighborhood.reputation + delta).clamp(0, 100);
        }
    }

    /// Gentrification pressure (0–100) of the neighborhood the active building
    /// sits in, defaulting to 0 when the building isn't placed yet.
    fn active_neighborhood_gentrification(&self) -> i32 {
        let building_id = self.city.active_building_index as u32;
        self.city
            .neighborhoods
            .iter()
            .find(|n| n.building_ids.contains(&building_id))
            .map(|n| n.stats.gentrification)
            .unwrap_or(0)
    }

    /// Realizable equity multiplier applied to a condo's market value. Debt,
    /// conversion costs, and closing costs keep the base proceeds below full
    /// market value; a boom and gentrification still make timing matter.
    pub(super) fn condo_sale_market_multiplier(&self) -> f32 {
        let economy = self.city.economy_health; // 0.5..1.5
        let gentrification = self.active_neighborhood_gentrification() as f32 / 100.0;
        let boom_bonus = self.config.gentrification.condo_sale_boom_bonus;
        let equity_share = self.config.gentrification.condo_sale_equity_share;
        (equity_share * economy * (1.0 + gentrification * boom_bonus)).clamp(0.1, 0.75)
    }

    /// Reputation of the neighborhood the active building sits in (0–100),
    /// defaulting to the neutral 50 when the building isn't placed yet.
    pub(super) fn active_neighborhood_reputation(&self) -> i32 {
        let building_id = self.city.active_building_index as u32;
        self.city
            .neighborhoods
            .iter()
            .find(|n| n.building_ids.contains(&building_id))
            .map(|n| n.reputation)
            .unwrap_or(50)
    }

    /// Applicant-volume multiplier derived from the active neighborhood's
    /// reputation. Neutral reputation (50) yields 1.0; a strong reputation draws
    /// proportionally more applicants and a poor one drives them away — the
    /// consequence that makes reputation worth cultivating.
    pub(super) fn application_reputation_multiplier(&self) -> f32 {
        let reputation = self.active_neighborhood_reputation();
        let influence = self.config.applications.reputation_influence;
        (1.0 + (reputation - 50) as f32 / 50.0 * influence).clamp(0.25, 2.0)
    }

    /// Apply a reputation change to a specific neighborhood (or the active
    /// building's neighborhood when `neighborhood_id` is `None`) with feedback.
    /// This is the write path that makes reputation a currency the player moves
    /// through event choices and mission rewards.
    pub(super) fn apply_reputation_change(&mut self, delta: i32, neighborhood_id: Option<u32>) {
        if delta == 0 {
            return;
        }
        match neighborhood_id {
            Some(id) => {
                if let Some(neighborhood) = self.city.neighborhoods.iter_mut().find(|n| n.id == id)
                {
                    neighborhood.reputation = (neighborhood.reputation + delta).clamp(0, 100);
                }
            }
            None => self.adjust_active_neighborhood_reputation(delta),
        }

        let color = if delta >= 0 {
            colors::POSITIVE()
        } else {
            colors::NEGATIVE()
        };
        self.floating_texts.spawn(
            format!("Rep {:+}", delta),
            vec2(screen_width() / 2.0, screen_height() / 2.0 + 60.0),
            color,
        );
    }
}

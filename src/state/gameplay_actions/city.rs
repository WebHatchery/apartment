//! City-map navigation and property purchases.

use crate::state::gameplay_turn::feedback_point;
use macroquad::prelude::*;

use crate::city::NeighborhoodType;
use crate::simulation::GameEvent;
use crate::state::{GameplayState, ViewMode};
use crate::ui::{colors, UiAction};

impl GameplayState {
    pub(in crate::state) fn purchase_listing(&mut self, listing_id: u32) {
        if let Some(listing) = self
            .city
            .market
            .listings
            .iter()
            .find(|l| l.id == listing_id)
            .cloned()
        {
            if self.funds.balance >= listing.asking_price {
                let building = listing.to_building();
                let neighborhood_id = listing.neighborhood_id;

                if let Ok(building_id) = self.city.add_building(building, neighborhood_id) {
                    let transaction = crate::economy::Transaction::expense(
                        crate::economy::TransactionType::BuildingPurchase,
                        listing.asking_price,
                        "Building Purchase",
                        self.current_tick,
                    );
                    self.funds.deduct_expense(transaction);

                    let is_historic = self.city.neighborhoods.iter().any(|n| {
                        n.id == neighborhood_id
                            && matches!(n.neighborhood_type, NeighborhoodType::Historic)
                    });
                    self.compliance
                        .init_building_regulations(building_id, is_historic);

                    self.city.market.listings.retain(|l| l.id != listing_id);

                    self.floating_texts.spawn(
                        "Building Purchased!",
                        feedback_point(),
                        colors::POSITIVE(),
                    );

                    self.event_log.log(
                        GameEvent::UpgradeCompleted {
                            description: "Purchased new building".to_string(),
                            cost: listing.asking_price,
                        },
                        self.current_tick,
                    );
                }
            } else {
                self.report_action_failure(&format!(
                    "Purchase requires ${}; only ${} is available",
                    listing.asking_price, self.funds.balance
                ));
            }
        }
    }

    pub(in crate::state) fn handle_city_action(
        &mut self,
        action: crate::ui::city_view::CityMapAction,
    ) {
        use crate::ui::city_view::CityMapAction;

        match action {
            CityMapAction::SelectNeighborhood(id) => {
                self.selected_neighborhood_id = if self.selected_neighborhood_id == Some(id) {
                    None
                } else {
                    Some(id)
                };
                self.city_scroll = 0.0;
                self.market_page = 0;
            }
            CityMapAction::ClearNeighborhood => {
                self.selected_neighborhood_id = None;
                self.city_scroll = 0.0;
                self.market_page = 0;
            }
            CityMapAction::EnterBuilding(index) => {
                self.save_building_to_city();
                self.city.switch_building(index);
                self.sync_building();
                self.view_mode = ViewMode::Building;
            }
            CityMapAction::OpenMarket => {
                self.market_page = 0;
                self.view_mode = ViewMode::Market;
            }
            CityMapAction::CloseMarket => {
                self.pending_property_purchase = None;
                self.view_mode = ViewMode::CityMap;
            }
            CityMapAction::ReviewPurchase(listing_id) => {
                self.pending_property_purchase = Some(listing_id);
            }
            CityMapAction::CancelPurchase => {
                self.pending_property_purchase = None;
            }
            CityMapAction::ConfirmPurchase(listing_id) => {
                self.pending_property_purchase = None;
                self.pending_actions
                    .push(UiAction::PurchaseBuilding { listing_id });
            }
            CityMapAction::SetMarketPage(page) => {
                self.market_page = page;
            }
        }
    }
}

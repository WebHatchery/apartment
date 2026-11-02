//! Scripted landlord behaviour: maintenance, staffing, policy, condo sales
//! and portfolio growth for headless runs.

use super::*;

impl Sim {
    /// Repairs, optional design upgrades, and optional staff hires.
    pub(super) fn maintain(&mut self, strat: &Strategy) {
        let tick = self.current_tick;
        let mut repair_budget = if strat.upgrade_designs { 2_000 } else { 750 };

        // Compliance grades the lower of hallway and average unit condition,
        // so a careful player fixes the shared escape route before cosmetics.
        if !self.tenants.is_empty() && self.building.hallway_condition < strat.repair_threshold {
            let cost_per = self.config.economy.hallway_repair_cost_per_point;
            let amount = (strat.repair_threshold - self.building.hallway_condition)
                .min(repair_budget / cost_per);
            let cost = amount * cost_per;
            if amount > 0 && self.affordable(cost, strat.cash_reserve) {
                let _ = process_upgrade(
                    &UpgradeAction::RepairHallway { amount },
                    &mut self.building,
                    &mut self.funds,
                    &self.config,
                    tick,
                );
                repair_budget -= cost;
            }
        }

        // Repair worn units (cheapest-first is irrelevant; just cap by reserve).
        let ids: Vec<u32> = self
            .building
            .apartments
            .iter()
            .filter(|apartment| !self.building.is_unit_sold(apartment.id))
            .map(|apartment| apartment.id)
            .collect();
        for id in ids {
            let (cond, cost_per) = {
                let apt = self.building.get_apartment(id).unwrap();
                (apt.condition, self.config.economy.repair_cost_per_point)
            };
            if cond < strat.repair_threshold {
                // Scripted players repair to their policy threshold, not to
                // pristine condition. The old harness forced enormous month-0
                // restoration bills that no cash-conscious player would make.
                let amount = (strat.repair_threshold - cond).min(repair_budget / cost_per);
                let cost = amount * cost_per;
                if amount > 0 && self.affordable(cost, strat.cash_reserve) {
                    let _ = process_upgrade(
                        &UpgradeAction::RepairApartment {
                            apartment_id: id,
                            amount,
                        },
                        &mut self.building,
                        &mut self.funds,
                        &self.config,
                        tick,
                    );
                    repair_budget -= cost;
                }
            }
        }

        if strat.hire_staff {
            self.hire_staff();
        }

        if strat.upgrade_designs
            && self.occupancy() >= 0.75
            && self.affordable(5_000, strat.cash_reserve)
        {
            // Push one occupied unit up a design tier per month while flush.
            let target = self.building.apartments.iter().find_map(|a| {
                if a.tenant_id.is_some()
                    && matches!(a.design, DesignType::Bare | DesignType::Practical)
                {
                    Some(a.id)
                } else {
                    None
                }
            });
            if let Some(id) = target {
                let upgrade_id = self
                    .building
                    .get_apartment(id)
                    .map(|apartment| match apartment.design {
                        DesignType::Bare => "upgrade_to_practical",
                        DesignType::Practical => "upgrade_to_cozy",
                        _ => "",
                    })
                    .unwrap_or("");
                let balance_before = self.funds.balance;
                let rent_before = self
                    .building
                    .get_apartment(id)
                    .map(|apartment| apartment.rent_price)
                    .unwrap_or_default();
                if process_upgrade(
                    &UpgradeAction::Apply {
                        upgrade_id: upgrade_id.to_string(),
                        target_id: Some(id),
                    },
                    &mut self.building,
                    &mut self.funds,
                    &self.config,
                    tick,
                )
                .is_ok()
                {
                    self.investment_spend += balance_before - self.funds.balance;
                    // A real landlord reprices after investing in the unit —
                    // capture some of the upgrade's value as higher rent
                    // instead of leaving it purely as a sunk cosmetic cost.
                    if let Some(apt) = self.building.get_apartment_mut(id) {
                        // A conservative post-renovation reprice. Flat deltas
                        // keep a design tier's payback consistent across cheap
                        // and premium campaign apartments.
                        apt.rent_price += match apt.design {
                            DesignType::Practical => 100,
                            DesignType::Cozy => 150,
                            _ => 0,
                        };
                        self.monthly_rent_uplift += apt.rent_price - rent_before;
                    }
                }
            }
        }
    }

    /// Hire staff through the same data-driven upgrade actions used by the UI.
    pub(super) fn hire_staff(&mut self) {
        for (role, upgrade_id, minimum_balance) in [("staff_janitor", "hire_janitor", 10_000)] {
            if self.funds.balance >= minimum_balance && !self.building.flags.contains(role) {
                let _ = process_upgrade(
                    &UpgradeAction::Apply {
                        upgrade_id: upgrade_id.to_string(),
                        target_id: None,
                    },
                    &mut self.building,
                    &mut self.funds,
                    &self.config,
                    self.current_tick,
                );
                break;
            }
        }
    }

    pub(super) fn manage_policy(&mut self, strat: &Strategy) {
        if strat.tenant_services {
            self.building.utilities_included = true;
            self.building.insurance_active = self.current_tick >= 12;
            self.building.marketing_strategy = if self.occupancy() >= 1.0 {
                MarketingType::None
            } else if self
                .building
                .apartments
                .iter()
                .map(|apartment| apartment.rent_price)
                .max()
                .unwrap_or_default()
                > 1_200
            {
                MarketingType::PremiumAgency
            } else {
                MarketingType::SocialMedia
            };
            if self.occupancy() < 0.75
                && self.building.open_house_remaining == 0
                && self.affordable(self.config.marketing.open_house_cost, strat.cash_reserve)
                && self.funds.deduct_expense(Transaction::expense(
                    TransactionType::Marketing,
                    self.config.marketing.open_house_cost,
                    "Open House",
                    self.current_tick,
                ))
            {
                self.building.open_house_remaining = self.config.marketing.open_house_duration;
            }
        }

        match strat.special {
            SpecialPolicy::AggressiveRent
                if self.current_tick > 0 && self.current_tick.is_multiple_of(6) =>
            {
                let rental_ids: Vec<u32> = self
                    .building
                    .apartments
                    .iter()
                    .filter(|apartment| !self.building.is_unit_sold(apartment.id))
                    .map(|apartment| apartment.id)
                    .collect();
                for apartment_id in rental_ids {
                    let apartment = self.building.get_apartment_mut(apartment_id).unwrap();
                    apartment.rent_price = (apartment.rent_price as f32 * 1.15).round() as i32;
                }
            }
            SpecialPolicy::CondoSale if self.current_tick >= 12 && self.condos_sold == 0 => {
                self.sell_one_condo();
            }
            SpecialPolicy::PortfolioExpansion if self.current_tick >= 12 => {
                self.try_expand_portfolio(strat.cash_reserve);
            }
            _ => {}
        }
    }

    pub(super) fn sell_one_condo(&mut self) {
        let Some(apartment_id) = self
            .building
            .apartments
            .iter()
            .filter(|apartment| !self.building.is_unit_sold(apartment.id))
            .min_by_key(|apartment| (apartment.tenant_id.is_some(), apartment.rent_price))
            .map(|apartment| apartment.id)
        else {
            return;
        };
        let (tenant_id, market_value) = self
            .building
            .get_apartment(apartment_id)
            .map(|apartment| (apartment.tenant_id, apartment.market_value()))
            .unwrap_or_default();
        let neighborhood_pressure = self
            .city
            .neighborhood_for_building(self.city.active_building_index)
            .map(|neighborhood| neighborhood.stats.gentrification as f32 / 100.0)
            .unwrap_or_default();
        let sale_multiplier = (self.config.gentrification.condo_sale_equity_share
            * self.city.economy_health
            * (1.0 + neighborhood_pressure * self.config.gentrification.condo_sale_boom_bonus))
            .clamp(0.1, 0.75);
        let sale_price = (market_value as f32 * sale_multiplier) as i32;
        if !self
            .building
            .convert_unit_to_condo(apartment_id, "New Owner", sale_price)
        {
            return;
        }
        if let Some(tenant_id) = tenant_id {
            self.tenants.retain(|tenant| tenant.id != tenant_id);
            self.departures += 1;
        }
        if let Some(apartment) = self.building.get_apartment_mut(apartment_id) {
            apartment.move_out();
            apartment.is_listed_for_lease = false;
        }
        self.applications
            .retain(|application| application.apartment_id != apartment_id);
        self.funds.add_income(Transaction::income(
            TransactionType::AssetSale,
            sale_price,
            "Condo Sale",
            self.current_tick,
        ));
        self.condos_sold += 1;
    }

    pub(super) fn try_expand_portfolio(&mut self, reserve: i32) {
        let listing = self
            .city
            .market
            .listings
            .iter()
            .filter(|listing| self.affordable(listing.asking_price, reserve))
            .min_by_key(|listing| listing.asking_price)
            .cloned();
        let Some(listing) = listing else { return };
        if self
            .city
            .add_building(listing.to_building(), listing.neighborhood_id)
            .is_err()
        {
            return;
        }
        if self.funds.deduct_expense(Transaction::expense(
            TransactionType::BuildingPurchase,
            listing.asking_price,
            "Building Purchase",
            self.current_tick,
        )) {
            self.city
                .market
                .listings
                .retain(|candidate| candidate.id != listing.id);
            self.buildings_bought += 1;
        }
    }

    pub(super) fn collect_portfolio_income(&mut self) {
        let active = self.city.active_building_index;
        let net: i32 = self
            .city
            .buildings
            .iter()
            .enumerate()
            .filter(|(index, building)| *index != active && !building.apartments.is_empty())
            .map(|(_, building)| {
                let potential: i32 = building
                    .apartments
                    .iter()
                    .filter(|apartment| !building.is_unit_sold(apartment.id))
                    .map(|apartment| apartment.rent_price)
                    .sum();
                (potential as f32 * self.config.portfolio.passive_occupancy) as i32
                    - building.apartments.len() as i32 * self.config.portfolio.passive_cost_per_unit
            })
            .sum();
        if net > 0 {
            self.funds.add_income(Transaction::income(
                TransactionType::RentIncome,
                net,
                "Portfolio passive income",
                self.current_tick,
            ));
        } else if net < 0 {
            self.funds.apply_required_expense(Transaction::expense(
                TransactionType::Mortgage,
                net.abs(),
                "Portfolio upkeep",
                self.current_tick,
            ));
        }
    }
}

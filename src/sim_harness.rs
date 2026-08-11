//! Headless balance-simulation harness.
//!
//! Drives the game's *pure* simulation logic (no macroquad/rendering) under a
//! set of scripted "landlord" strategies over the full 36-month game, records
//! per-month metrics, and writes a markdown balance report. This lets us
//! measure the economy empirically ("is it too easy to make money?") instead of
//! guessing, and re-run it after every balance tweak.
//!
//! Run it with:
//!   cargo test --lib balance -- --ignored --nocapture      (bin crate: `--bin` not needed)
//!   cargo test balance_report -- --ignored --nocapture
//!
//! The report is written to `balance_report.md` at the repo root.

use crate::building::{Building, DesignType, MarketingType, UpgradeAction};
use crate::city::City;
use crate::consequences::{ComplianceSystem, InspectionTrigger};
use crate::data::config::GameConfig;
use crate::data::templates::BuildingTemplate;
use crate::economy::{process_upgrade, FinancialLedger, PlayerFunds, Transaction, TransactionType};
use crate::simulation::{advance_tick, EventLog, GameOutcome};
use crate::tenant::matching::{evaluate_lease_offer, LeaseOffer};
use crate::tenant::vetting::{perform_background_check, perform_credit_check};
use crate::tenant::{Tenant, TenantApplication};
use macroquad_toolkit::rng;

mod finance;
mod missions;
mod report;
mod strategy;

use report::generate_report;
use strategy::{strategies, SpecialPolicy, Strategy};

/// Metrics captured at the end of each simulated month.
#[derive(Clone, Copy)]
struct MonthMetrics {
    rent: i32,
    expenses: i32,
}

/// Aggregated outcome of a single full playthrough.
struct RunResult {
    months: Vec<MonthMetrics>,
    final_balance: i32,
    end_occupancy: f32,
    score: i32,
    departures: u32,
    applications_generated: u32,
    end_happiness: i32,
    end_condition: i32,
    investment_spend: i32,
    investment_payback_months: Option<f32>,
    condos_sold: u32,
    buildings_bought: u32,
    missions_completed: u32,
    mission_cash: i32,
    outcome: Option<GameOutcome>,
}

struct Sim {
    building: Building,
    tenants: Vec<Tenant>,
    applications: Vec<TenantApplication>,
    funds: PlayerFunds,
    ledger: FinancialLedger,
    event_log: EventLog,
    compliance: ComplianceSystem,
    current_tick: u32,
    next_tenant_id: u32,
    config: GameConfig,
    city: City,
    missions: crate::narrative::MissionManager,
    active_tax_breaks: Vec<crate::narrative::ActiveTaxBreak>,
    reputation: i32,
    investment_spend: i32,
    monthly_rent_uplift: i32,
    condos_sold: u32,
    buildings_bought: u32,
    missions_completed: u32,
    mission_cash: i32,
    departures: u32,
    applications_generated: u32,
}

impl Sim {
    /// Start through the exact live new-game constructor. This applies the
    /// template's difficulty, historic regulations, inherited tenant, initial
    /// applications, and starting cash before the headless driver takes over.
    fn new(template: &BuildingTemplate, seed: u64) -> Self {
        let state = crate::state::GameplayState::new_with_template_seed(
            crate::data::config::load_config(),
            template.clone(),
            seed,
        );

        Self {
            building: state.building.clone(),
            tenants: state.tenants.clone(),
            applications: state.applications.clone(),
            funds: state.funds.clone(),
            ledger: state.ledger.clone(),
            event_log: state.event_log.clone(),
            compliance: state.compliance.clone(),
            current_tick: state.current_tick,
            next_tenant_id: state.next_tenant_id,
            config: state.config.clone(),
            city: state.city.clone(),
            missions: state.missions.clone(),
            active_tax_breaks: state.active_tax_breaks.clone(),
            reputation: 50,
            investment_spend: 0,
            monthly_rent_uplift: 0,
            condos_sold: 0,
            buildings_bought: 0,
            missions_completed: 0,
            mission_cash: 0,
            departures: 0,
            applications_generated: 0,
        }
    }

    /// Mirror the real game's monthly inspection + fine billing so the harness
    /// measures the regulatory teeth that punish neglect (the game runs these in
    /// `end_turn`, outside `advance_tick`).
    fn run_inspections_and_fines(&mut self) {
        let score = self
            .building
            .average_condition()
            .min(self.building.hallway_condition);
        self.compliance.resolve_fixes_if_compliant(
            0,
            score,
            self.config.regulations.pass_condition_threshold,
        );
        self.compliance.tick(self.current_tick);

        let cfg = self.config.regulations.clone();
        let due = self.compliance.has_due_inspection(0);
        let random_check = rng::gen_range(0, 100) < cfg.random_inspection_chance_percent;
        if due || random_check {
            let trigger = if due {
                InspectionTrigger::Scheduled
            } else {
                InspectionTrigger::Random
            };
            self.compliance
                .run_inspection(0, score, self.current_tick, trigger, &cfg);
        }

        if self.compliance.unpaid_fines > 0 {
            let amount = self.compliance.unpaid_fines;
            self.funds.apply_required_expense(Transaction::expense(
                TransactionType::InspectionFine,
                amount,
                "Regulatory fines",
                self.current_tick,
            ));
            self.compliance.unpaid_fines = 0;
        }
    }

    fn total_units(&self) -> usize {
        self.building.apartments.len()
    }

    fn occupancy(&self) -> f32 {
        let total = self.total_units();
        if total == 0 {
            return 0.0;
        }
        self.building.occupancy_count() as f32 / total as f32
    }

    fn avg_happiness(&self) -> i32 {
        if self.tenants.is_empty() {
            return 0;
        }
        self.tenants.iter().map(|t| t.happiness).sum::<i32>() / self.tenants.len() as i32
    }

    /// List every vacant unit so it can draw applicants next tick.
    fn list_vacancies(&mut self) {
        for apt in &mut self.building.apartments {
            if apt.is_vacant() {
                apt.is_listed_for_lease = true;
            }
        }
    }

    /// Decide on each pending application. Accepting rolls the same lease-decline
    /// dice the real UI does, and a decline consumes the applicant (as in-game).
    fn handle_applications(&mut self, strat: &Strategy) {
        // The bot acts on every pending application each month, so we drain the
        // whole queue; a declined offer simply consumes the applicant.
        let applications = std::mem::take(&mut self.applications);
        for mut app in applications {
            if strat.vet_applicants {
                let credit = perform_credit_check(
                    &mut app,
                    &mut self.funds,
                    &self.config.vetting,
                    self.current_tick,
                );
                let background = perform_background_check(
                    &mut app,
                    &mut self.funds,
                    &self.config.vetting,
                    self.current_tick,
                );
                if credit.is_some()
                    && background.is_some()
                    && (app.tenant.rent_reliability < self.config.tenant_risk.unreliable_threshold
                        || app.tenant.behavior_score
                            < self.config.tenant_risk.low_behavior_threshold)
                {
                    continue;
                }
            }

            let Some(apt) = self.building.get_apartment(app.apartment_id) else {
                continue;
            };
            if !apt.is_vacant() {
                continue;
            }

            let offer =
                LeaseOffer::from_config(apt.rent_price, &self.config.matching.lease_defaults);
            let accept_prob =
                evaluate_lease_offer(&app.tenant, &offer, &self.config.matching.lease_acceptance);
            let leverage_penalty = app.tenant.negotiation_leverage() as f32 * 0.002;
            let adjusted = (accept_prob - leverage_penalty).clamp(0.0, 1.0);

            if rng::gen_range(0.0, 1.0) > adjusted {
                // Tenant declined the offer — applicant is gone.
                continue;
            }

            let apartment_id = app.apartment_id;
            let mut tenant = app.tenant;
            tenant.move_into_building(0, apartment_id);
            if let Some(apt) = self.building.get_apartment_mut(apartment_id) {
                apt.move_in(tenant.id);
            }
            self.tenants.push(tenant);
        }
    }

    fn affordable(&self, cost: i32, reserve: i32) -> bool {
        self.funds.balance - cost >= reserve
    }

    /// Repairs, optional design upgrades, and optional staff hires.
    fn maintain(&mut self, strat: &Strategy) {
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
    fn hire_staff(&mut self) {
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

    fn manage_policy(&mut self, strat: &Strategy) {
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

    fn sell_one_condo(&mut self) {
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

    fn try_expand_portfolio(&mut self, reserve: i32) {
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

    fn collect_portfolio_income(&mut self) {
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

    /// Play the full game under `strat` and return the aggregated result.
    fn run(mut self, strat: &Strategy, duration: u32) -> RunResult {
        let mut months = Vec::with_capacity(duration as usize);
        let mut outcome = None;
        let mut has_ever_had_tenant = false;

        for _ in 0..duration {
            self.prepare_missions();
            self.list_vacancies();
            self.handle_applications(strat);
            self.manage_policy(strat);
            self.maintain(strat);

            has_ever_had_tenant |= !self.tenants.is_empty();
            let reputation_multiplier = self.application_reputation_multiplier();

            let result = advance_tick(
                0,
                &mut self.building,
                &mut self.tenants,
                &mut self.applications,
                &mut self.funds,
                &mut self.ledger,
                &mut self.event_log,
                &mut self.current_tick,
                &mut self.next_tenant_id,
                has_ever_had_tenant,
                reputation_multiplier,
                &self.config,
            );

            self.apply_active_tax_breaks();
            // Apply the regulatory teeth that live outside advance_tick so the
            // report reflects the real cost of neglect.
            self.run_inspections_and_fines();
            self.city.tick();
            self.collect_portfolio_income();
            self.update_missions(&result);

            let expenses = self.tick_expenses();
            let earned_income = self.tick_earned_income();
            self.departures += result.tenants_moved_out.len() as u32;
            self.applications_generated += result.new_applications as u32;
            if outcome.is_none() {
                outcome = result.outcome.clone();
            }

            months.push(MonthMetrics {
                rent: earned_income,
                expenses,
            });

            if outcome.is_some() {
                break;
            }
        }

        let score = match &outcome {
            Some(GameOutcome::Victory { score, .. }) => *score,
            Some(GameOutcome::Bankruptcy { debt }) => -*debt,
            Some(GameOutcome::AllTenantsLeft) | None => 0,
        };

        RunResult {
            months,
            final_balance: self.funds.balance,
            end_occupancy: self.occupancy(),
            score,
            departures: self.departures,
            applications_generated: self.applications_generated,
            end_happiness: self.avg_happiness(),
            end_condition: self.building.average_condition(),
            investment_spend: self.investment_spend,
            investment_payback_months: (self.monthly_rent_uplift > 0)
                .then_some(self.investment_spend as f32 / self.monthly_rent_uplift as f32),
            condos_sold: self.condos_sold,
            buildings_bought: self.buildings_bought,
            missions_completed: self.missions_completed,
            mission_cash: self.mission_cash,
            outcome,
        }
    }
}

/// Mean of the (rent - expenses) net over the final `window` months — the
/// "steady-state" monthly profit once the building is established.
fn steady_state_net(months: &[MonthMetrics], window: usize) -> i32 {
    if months.is_empty() {
        return 0;
    }
    let slice = &months[months.len().saturating_sub(window)..];
    let sum: i64 = slice.iter().map(|m| (m.rent - m.expenses) as i64).sum();
    (sum / slice.len() as i64) as i32
}

/// Averaged summary of many seeded runs of one strategy.
struct StrategySummary {
    name: &'static str,
    mean_final_balance: i64,
    mean_end_occupancy: f32,
    bankruptcy_count: usize,
    mean_steady_net: i64,
    mean_score: i64,
    mean_departures: f32,
    mean_applications: f32,
    mean_end_happiness: i32,
    mean_end_condition: i32,
    mean_investment_spend: i64,
    mean_payback_months: f32,
    mean_condos_sold: f32,
    mean_buildings_bought: f32,
    mean_missions_completed: f32,
    mean_mission_cash: i64,
    victory_count: usize,
    all_left_count: usize,
}

fn summarize(template: &BuildingTemplate, strat: &Strategy, seeds: u64) -> StrategySummary {
    let duration = crate::data::config::load_config()
        .win_conditions
        .game_duration_ticks
        .unwrap_or(36);

    let mut sum_final = 0i64;
    let mut sum_end_occ = 0f32;
    let mut bankruptcies = 0usize;
    let mut sum_steady = 0i64;
    let mut sum_score = 0i64;
    let mut sum_departures = 0u32;
    let mut sum_applications = 0u32;
    let mut sum_happiness = 0i64;
    let mut sum_condition = 0i64;
    let mut sum_investment_spend = 0i64;
    let mut sum_payback = 0.0f32;
    let mut payback_runs = 0usize;
    let mut sum_condos = 0u32;
    let mut sum_buildings = 0u32;
    let mut sum_missions = 0u32;
    let mut sum_mission_cash = 0i64;
    let mut victories = 0usize;
    let mut all_left = 0usize;

    for seed in 0..seeds {
        let sim = Sim::new(template, 0xA11CE ^ seed);
        let result = sim.run(strat, duration);

        sum_final += result.final_balance as i64;
        sum_end_occ += result.end_occupancy;
        sum_score += result.score as i64;
        sum_departures += result.departures;
        sum_applications += result.applications_generated;
        sum_happiness += result.end_happiness as i64;
        sum_condition += result.end_condition as i64;
        sum_investment_spend += result.investment_spend as i64;
        if let Some(payback) = result.investment_payback_months {
            sum_payback += payback;
            payback_runs += 1;
        }
        sum_condos += result.condos_sold;
        sum_buildings += result.buildings_bought;
        sum_missions += result.missions_completed;
        sum_mission_cash += result.mission_cash as i64;
        sum_steady += steady_state_net(&result.months, 12) as i64;

        if matches!(result.outcome, Some(GameOutcome::Bankruptcy { .. })) {
            bankruptcies += 1;
        }
        if matches!(result.outcome, Some(GameOutcome::Victory { .. })) {
            victories += 1;
        }
        if matches!(result.outcome, Some(GameOutcome::AllTenantsLeft)) {
            all_left += 1;
        }
    }

    let runs = seeds as i64;
    StrategySummary {
        name: strat.name,
        mean_final_balance: sum_final / runs,
        mean_end_occupancy: sum_end_occ / seeds as f32,
        bankruptcy_count: bankruptcies,
        mean_steady_net: sum_steady / runs,
        mean_score: sum_score / runs,
        mean_departures: sum_departures as f32 / seeds as f32,
        mean_applications: sum_applications as f32 / seeds as f32,
        mean_end_happiness: (sum_happiness / runs) as i32,
        mean_end_condition: (sum_condition / runs) as i32,
        mean_investment_spend: sum_investment_spend / runs,
        mean_payback_months: if payback_runs > 0 {
            sum_payback / payback_runs as f32
        } else {
            f32::NAN
        },
        mean_condos_sold: sum_condos as f32 / seeds as f32,
        mean_buildings_bought: sum_buildings as f32 / seeds as f32,
        mean_missions_completed: sum_missions as f32 / seeds as f32,
        mean_mission_cash: sum_mission_cash / runs,
        victory_count: victories,
        all_left_count: all_left,
    }
}

struct TierSummary {
    template: BuildingTemplate,
    starting_cash: i32,
    strategies: Vec<StrategySummary>,
}

#[cfg(test)]
mod tests;

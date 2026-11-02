use super::{PlayerFunds, Transaction, TransactionType};
use serde::{Deserialize, Serialize};

/// Monthly financial summary
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonthlyReport {
    pub tick: u32,
    pub rent_income: i32,
    pub repair_costs: i32,
    pub upgrade_costs: i32,
    pub net: i32,
    pub ending_balance: i32,
}

/// Financial tracking across the game
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinancialLedger {
    pub reports: Vec<MonthlyReport>,
    /// Count of `PlayerFunds::transactions` already covered by a closed month.
    /// Everything after it belongs to the month in progress, whatever tick the
    /// transaction was tagged with. `None` only for saves from before this
    /// field existed; `post_load` aligns it with the current transactions.
    #[serde(default)]
    reported_transactions: Option<usize>,
}

impl FinancialLedger {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
            reported_transactions: Some(0),
        }
    }

    /// Start the month in progress at the current end of the transaction list
    /// when a save predates month tracking.
    pub fn align_unreported(&mut self, funds: &PlayerFunds) {
        if self.reported_transactions.is_none() {
            self.mark_reported(funds);
        }
    }

    /// Treat every existing transaction as belonging to a closed month.
    pub fn mark_reported(&mut self, funds: &PlayerFunds) {
        self.reported_transactions = Some(funds.transactions.len());
    }

    /// Transactions recorded since the last closed month.
    pub fn unreported<'a>(&self, funds: &'a PlayerFunds) -> &'a [Transaction] {
        let start = self
            .reported_transactions
            .unwrap_or(0)
            .min(funds.transactions.len());
        &funds.transactions[start..]
    }

    /// Close the month in progress: report every transaction since the last
    /// close, including spending made before End Month and anything the
    /// turn added after the simulation tick.
    pub fn close_month(&mut self, tick: u32, funds: &PlayerFunds) -> MonthlyReport {
        let month: Vec<&Transaction> = self.unreported(funds).iter().collect();
        let report = self.generate_report(tick, &month, funds.balance);
        self.mark_reported(funds);
        report
    }

    /// Generate a monthly report from transactions
    pub fn generate_report(
        &mut self,
        tick: u32,
        transactions: &[&Transaction],
        ending_balance: i32,
    ) -> MonthlyReport {
        let mut rent_income = 0;
        let mut repair_costs = 0;
        let mut upgrade_costs = 0;

        for t in transactions {
            match t.transaction_type {
                TransactionType::RentIncome | TransactionType::Grant => {
                    rent_income += t.amount.abs()
                }
                TransactionType::RepairCost | TransactionType::HallwayRepair => {
                    repair_costs += t.amount.abs();
                }
                TransactionType::UpgradeCost => upgrade_costs += t.amount.abs(),
                TransactionType::BuildingPurchase => upgrade_costs += t.amount.abs(), // Count as capital upgrade for now
                TransactionType::AssetSale => rent_income += t.amount.abs(), // Count condo sales as income
                TransactionType::PropertyTax
                | TransactionType::Mortgage
                | TransactionType::Utilities
                | TransactionType::Insurance
                | TransactionType::StaffSalary
                | TransactionType::CriticalFailure
                | TransactionType::Marketing
                | TransactionType::Vetting
                | TransactionType::InspectionFine => {
                    // These are all operating expenses, count them in repair_costs for simplicity
                    repair_costs += t.amount.abs();
                }
            }
        }

        let report = MonthlyReport {
            tick,
            rent_income,
            repair_costs,
            upgrade_costs,
            net: rent_income - repair_costs - upgrade_costs,
            ending_balance,
        };

        self.reports.push(report.clone());
        report
    }
}

impl Default for FinancialLedger {
    fn default() -> Self {
        Self::new()
    }
}

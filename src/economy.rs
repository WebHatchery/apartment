//! # Economy Module
//!
//! Manages all financial flows in the game:
//! - `Money`: Player funds, transaction history.
//! - `Rent`: Collection logic and rent setting.
//! - `Costs`: Operating expenses, taxes, utilities.
//! - `Ledger`: Monthly financial reporting.

mod costs;
mod ledger;
mod money;
mod rent;

pub use costs::{process_upgrade, OperatingCosts};
pub use ledger::{FinancialLedger, MonthlyReport};
pub use money::{PlayerFunds, Transaction, TransactionType};
pub use rent::collect_rent;

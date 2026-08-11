use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    RentIncome,
    RepairCost,
    UpgradeCost,
    HallwayRepair,
    BuildingPurchase,
    AssetSale,
    PropertyTax,
    Mortgage,
    Utilities,
    Insurance,
    StaffSalary,
    CriticalFailure,
    Marketing,
    Vetting,
    InspectionFine,
    Grant, // Mission rewards, grants, bonuses
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    pub amount: i32, // Positive = income, negative = expense
    pub description: String,
    pub tick: u32,
}

impl Transaction {
    pub fn income(
        transaction_type: TransactionType,
        amount: i32,
        description: &str,
        tick: u32,
    ) -> Self {
        Self {
            transaction_type,
            amount: amount.abs(), // Ensure positive
            description: description.to_string(),
            tick,
        }
    }

    pub fn expense(
        transaction_type: TransactionType,
        amount: i32,
        description: &str,
        tick: u32,
    ) -> Self {
        Self {
            transaction_type,
            amount: -amount.abs(), // Ensure negative
            description: description.to_string(),
            tick,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerFunds {
    pub balance: i32,
    pub total_income: i32,
    pub total_expenses: i32,
    pub transactions: Vec<Transaction>,
}

impl PlayerFunds {
    /// Short operating-credit buffer before the run is irrecoverably bankrupt.
    /// A single mandatory bill may take cash negative; the player gets a brief
    /// chance to lease a unit or sell an asset before the bank closes the run.
    pub const BANKRUPTCY_DEBT_LIMIT: i32 = 2_500;
    pub fn new(starting_balance: i32) -> Self {
        Self {
            balance: starting_balance,
            total_income: 0,
            total_expenses: 0,
            transactions: Vec::new(),
        }
    }

    /// Check if player can afford an expense
    pub fn can_afford(&self, cost: i32) -> bool {
        self.balance >= cost
    }

    /// Add income to balance
    pub fn add_income(&mut self, transaction: Transaction) {
        let amount = transaction.amount.abs();
        self.balance += amount;
        self.total_income += amount;
        self.transactions.push(transaction);
    }

    /// Deduct expense from balance (returns false if insufficient funds)
    pub fn deduct_expense(&mut self, transaction: Transaction) -> bool {
        let cost = transaction.amount.abs();
        if self.balance < cost {
            return false;
        }

        self.balance -= cost;
        self.total_expenses += cost;
        self.transactions.push(transaction);
        true
    }

    /// Record a mandatory expense even if it pushes the player into debt.
    pub fn apply_required_expense(&mut self, transaction: Transaction) {
        let cost = transaction.amount.abs();
        self.balance -= cost;
        self.total_expenses += cost;
        self.transactions.push(transaction);
    }

    /// Check if player is bankrupt
    pub fn is_bankrupt(&self) -> bool {
        self.balance < -Self::BANKRUPTCY_DEBT_LIMIT
    }

    /// Get transactions for a specific tick
    pub fn transactions_for_tick(&self, tick: u32) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|t| t.tick == tick)
            .collect()
    }
}

impl Default for PlayerFunds {
    fn default() -> Self {
        Self::new(5000) // Default starting funds
    }
}

#[cfg(test)]
mod tests;

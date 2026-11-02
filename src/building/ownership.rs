//! Ownership models (rental, condo board, co-op) and condo-board votes.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum OwnershipType {
    #[default]
    FullRental, // Player owns all, rents all (Default)
    MixedOwnership(CondoBoard), // Some units sold
    FullCondo(CondoBoard),      // All units sold, player manages
    CooperativeHousing,         // Tenant-owned (Future)
    SocialHousing,              // Government subsidized (Future)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CondoUnit {
    pub apartment_id: u32,
    pub owner_name: String,
    pub monthly_hoa: i32,
    pub owner_satisfaction: i32, // 0-100
    pub voting_power: i32,       // Usually based on SqFt or equal
    pub purchase_price: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BoardVote {
    pub proposal: String,
    pub cost: i32,
    pub votes_for: u32,
    pub votes_against: u32,
    pub deadline_month: u32,
    pub is_resolved: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct CondoBoard {
    pub units: Vec<CondoUnit>,
    pub reserve_fund: i32,
    pub pending_votes: Vec<BoardVote>,
}

impl CondoBoard {
    pub fn new() -> Self {
        Self {
            units: Vec::new(),
            reserve_fund: 0,
            pending_votes: Vec::new(),
        }
    }

    /// Add a unit to the condo association
    pub fn add_unit(
        &mut self,
        apartment_id: u32,
        owner_name: &str,
        monthly_hoa: i32,
        purchase_price: i32,
    ) {
        self.units.push(CondoUnit {
            apartment_id,
            owner_name: owner_name.to_string(),
            monthly_hoa,
            owner_satisfaction: 50, // Start neutral
            voting_power: 1,        // Default 1 vote per unit
            purchase_price,
        });
    }

    /// Collect HOA fees from all units
    pub fn collect_fees(&mut self) -> i32 {
        let total: i32 = self.units.iter().map(|u| u.monthly_hoa).sum();
        self.reserve_fund += total;
        total
    }

    /// Resolve votes that have reached deadline
    pub fn resolve_votes(&mut self, current_month: u32) -> Vec<String> {
        let mut results = Vec::new();

        for vote in &mut self.pending_votes {
            if !vote.is_resolved && current_month >= vote.deadline_month {
                // Simulate voting if not all votes cast (simple logic for now)
                // In a real system, we'd cast votes based on owner satisfaction

                // If more than 50% of voting power approves
                let total_power: i32 = self.units.iter().map(|u| u.voting_power).sum();
                let threshold = total_power / 2;

                // Auto-cast remaining votes based on satisfaction
                // High satisfaction -> Vote FOR upgrades/repairs
                // Low satisfaction -> Vote AGAINST spending

                for unit in &self.units {
                    // This is where individual AI voting happening would go
                    // For now, let's just say satisfaction > 50 = YES
                    if unit.owner_satisfaction > 50 {
                        vote.votes_for += unit.voting_power as u32;
                    } else {
                        vote.votes_against += unit.voting_power as u32;
                    }
                }

                vote.passed = vote.votes_for > threshold as u32;
                vote.is_resolved = true;

                let result_str = format!(
                    "Vote '{}': {} (For: {}, Against: {})",
                    vote.proposal,
                    if vote.passed { "PASSED" } else { "FAILED" },
                    vote.votes_for,
                    vote.votes_against
                );
                results.push(result_str);

                // If passed and involves cost, deduct from reserves
                if vote.passed && vote.cost > 0 {
                    self.reserve_fund -= vote.cost;
                }
            }
        }

        // Cleanup resolved votes
        self.pending_votes.retain(|v| !v.is_resolved);

        results
    }
}

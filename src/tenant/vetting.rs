//! Paid applicant screening: credit and background checks.

use super::application::TenantApplication;
use crate::data::config::VettingConfig;
use crate::economy::{PlayerFunds, Transaction, TransactionType};

/// Results of a credit check
pub struct CreditCheckResult {
    pub reliability_score: i32,
    pub recommendation: String,
}

/// Results of a background check
pub struct BackgroundCheckResult {
    pub behavior_score: i32,
    pub history_notes: String,
}

/// Perform a credit check on a tenant applicant
pub fn perform_credit_check(
    application: &mut TenantApplication,
    funds: &mut PlayerFunds,
    config: &VettingConfig,
    current_tick: u32,
) -> Option<CreditCheckResult> {
    if application.revealed_reliability {
        return None; // Already checked
    }

    if !funds.deduct_expense(Transaction::expense(
        TransactionType::Vetting,
        config.credit_check_cost,
        "Credit Check",
        current_tick,
    )) {
        return None; // Cannot afford
    }

    // Reveal stats
    application.revealed_reliability = true;
    let score = application.tenant.rent_reliability;

    // Generate recommendation based on thresholds
    let thresholds = &config.credit_thresholds;
    let recommendation = if score >= thresholds.excellent {
        "Excellent credit history. Highly recommended.".to_string()
    } else if score >= thresholds.good {
        "Good credit standing. No major concerns.".to_string()
    } else if score >= thresholds.average {
        "Average credit. Has some missed payments.".to_string()
    } else if score >= thresholds.below_average {
        "Below average. High risk of late rent.".to_string()
    } else {
        "Poor credit history. Default risk high.".to_string()
    };

    Some(CreditCheckResult {
        reliability_score: score,
        recommendation,
    })
}

/// Perform a background check (previous landlord reference)
pub fn perform_background_check(
    application: &mut TenantApplication,
    funds: &mut PlayerFunds,
    config: &VettingConfig,
    current_tick: u32,
) -> Option<BackgroundCheckResult> {
    if application.revealed_behavior {
        return None; // Already checked
    }

    if !funds.deduct_expense(Transaction::expense(
        TransactionType::Vetting,
        config.background_check_cost,
        "Background Check",
        current_tick,
    )) {
        return None; // Cannot afford
    }

    // Reveal stats
    application.revealed_behavior = true;
    let score = application.tenant.behavior_score;

    // Generate notes based on thresholds
    let thresholds = &config.behavior_thresholds;
    let history_notes = if score >= thresholds.excellent {
        "Quiet, respectful, keeps unit in perfect condition.".to_string()
    } else if score >= thresholds.good {
        "Generally good tenant. No noise complaints.".to_string()
    } else if score >= thresholds.average {
        "Occasional minor complaints but pays for damages.".to_string()
    } else if score >= thresholds.below_average {
        "History of noise complaints and minor damage.".to_string()
    } else {
        "Evicted from previous apartment for disturbance.".to_string()
    };

    Some(BackgroundCheckResult {
        behavior_score: score,
        history_notes,
    })
}

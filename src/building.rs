//! # Building Module
//!
//! Handles the physical structure of the game world:
//! - `Apartment`: Individual units, their condition, and properties.
//! - `Building`: The container for apartments and shared spaces (hallways).
//! - `Upgrades`: Systems for improving building and apartment quality.
//! - `Ownership`: Logic for selling units as condos.

mod apartment;
pub mod ownership;
mod structure;
pub mod upgrades;

pub use apartment::{Apartment, ApartmentSize, DesignType, NoiseLevel};
pub use structure::{Building, MarketingType};
pub use upgrades::{apply_upgrade, UpgradeAction};

//! # Tenant Module
//!
//! Everything related to the people living in the building:
//! - `Tenant`: Individual stats, name, and state.
//! - `Archetypes`: Defined behaviors and preferences (e.g., Student, Retiree).
//! - `Happiness`: Calculations for tenant satisfaction.
//! - `Applications`: New potential tenants and vetting.

mod application;
mod archetype;
pub mod happiness;
pub mod matching;
mod resident;
pub mod vetting;

pub use archetype::{ArchetypePreferences, TenantArchetype};
pub use happiness::calculate_happiness;
pub use resident::Tenant;
// pub use matching::MatchResult;
pub use application::{generate_applications, process_departures, TenantApplication};

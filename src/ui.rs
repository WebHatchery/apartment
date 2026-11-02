//! # UI Module
//!
//! Pure view layer using Macroquad immediate mode UI:
//! - Uses `UiAction` pattern to send intents back to the game logic.
//! - Contains sub-panels for different views (Building, Apartment, City).
//! - Floating text comes from the toolkit `fx::FloatingTextLayer`.
//! - Strictly separation of concerns: No game state mutation happens here.

pub mod theme;
pub mod widgets;

mod apartment_panel;
mod apartment_panel_sections;
mod application_panel;
pub mod building_view;
pub mod career_summary;
pub mod city_view; // Phase 3 city map
mod city_view_widgets;
mod common;
mod event_art;
pub mod event_modal; // Phase 4 event modal
mod hallway_panel;
pub mod header;
mod inspector_summary;
pub mod month_summary;
pub mod notifications;
pub mod ownership_panel; // Phase 3 ownership
mod panel_flow;
mod resident_sprite;
mod tenant_panel;
mod tenants_view;
pub mod workspace_finances;
pub mod workspace_inbox;
pub mod workspace_nav;
pub mod workspace_tasks;
pub mod workspace_views;

pub use apartment_panel::{draw_apartment_panel, UnitPanelContext};
pub use building_view::draw_building_view;
pub use common::*;
pub use hallway_panel::draw_hallway_panel;
pub use ownership_panel::draw_ownership_panel;
pub(crate) use resident_sprite::draw_resident_sprite_gallery;

pub use application_panel::draw_application_panel;
pub use header::draw_header;
pub use inspector_summary::draw_building_summary;
pub use macroquad_toolkit::fx::FloatingTextLayer;
pub use notifications::draw_notifications;

use serde::{Deserialize, Serialize};

/// What's currently selected for the detail panel
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum Selection {
    #[default]
    None,
    Apartment(u32),            // Apartment ID
    Applications(Option<u32>), // Show pending applications (Optionally filtered by apartment)
    Building,                  // Overall building details
    Hallway,                   // Hallway details
    Ownership,                 // Ownership View
}

/// Collapsible inspector sections; closed by default so each panel leads
/// with its immediate decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disclosure {
    ResidentDetails,
    UnitDetails,
    Improvements,
    Staff,
}

/// Which inspector sections the player has opened this session.
#[derive(Clone, Copy, Debug, Default)]
pub struct PanelDisclosure {
    open: [bool; 4],
}

impl PanelDisclosure {
    fn index(section: Disclosure) -> usize {
        match section {
            Disclosure::ResidentDetails => 0,
            Disclosure::UnitDetails => 1,
            Disclosure::Improvements => 2,
            Disclosure::Staff => 3,
        }
    }

    pub fn is_open(&self, section: Disclosure) -> bool {
        self.open[Self::index(section)]
    }

    pub fn toggle(&mut self, section: Disclosure) {
        let index = Self::index(section);
        self.open[index] = !self.open[index];
    }
}

use crate::building::UpgradeAction;

/// UI action intents (returned to game logic)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UiAction {
    SelectApartment(u32),
    OpenTenantUnit(u32),
    SelectTenant(u32),
    ClearTenantSelection,
    SelectApplications(Option<u32>),
    SelectHallway,
    SelectBuilding,
    SelectOwnership,
    ClearSelection,
    ToggleActivityDrawer,
    DismissMonthSummary,
    ToggleCoach,
    CloseHelp,
    ToggleDisclosure(Disclosure),

    // Generic Upgrade Action
    UpgradeAction(UpgradeAction),

    SetRent {
        apartment_id: u32,
        new_rent: i32,
    },

    // Tenant actions
    AcceptApplication {
        application_index: usize,
    },
    RejectApplication {
        application_index: usize,
    },

    // Game flow
    EndTurn,
    OpenPauseMenu,
    ReturnToMenu, // Used by Career Summary

    // Phase 3: City navigation
    OpenBuilding,
    OpenTenants,
    OpenFinances,
    OpenCityMap,
    OpenMail,
    OpenTasks,
    OpenMailItem {
        mail_id: u32,
    },
    ClearSelectedMail,
    SetInboxPage {
        page: usize,
    },
    SetTasksPage {
        page: usize,
    },
    SetApplicationPage {
        page: usize,
    },
    SetTenantsPage {
        page: usize,
    },
    SetRequestsPage {
        page: usize,
    },
    SetCareerBadgesPage {
        page: usize,
    },
    AcceptMission {
        mission_id: u32,
    },
    PurchaseBuilding {
        listing_id: u32,
    },

    // Phase 3: Tenant requests
    ApproveRequest {
        tenant_id: u32,
    },
    DenyRequest {
        tenant_id: u32,
    },

    // Phase 3: Ownership
    SellUnitAsCondo {
        apartment_id: u32,
    },
    BuybackCondo {
        apartment_id: u32,
    },
    SetMarketing {
        strategy: crate::building::MarketingType,
    },
    StartOpenHouse,
    SetUtilitiesIncluded {
        included: bool,
    },
    SetInsuranceActive {
        active: bool,
    },

    // Phase 4: Dialogue system
    ResolveDialogue {
        dialogue_id: u32,
        choice_index: usize,
    },
    ResolveEventChoice {
        event_id: u32,
        choice_index: usize,
    },

    // Phase 4: Tenant vetting
    CreditCheck {
        application_index: usize,
    },
    BackgroundCheck {
        application_index: usize,
    },

    // Leasing
    ListApartment {
        apartment_id: u32,
        preference: Option<crate::tenant::TenantArchetype>,
    },
    UnlistApartment {
        apartment_id: u32,
    },
}

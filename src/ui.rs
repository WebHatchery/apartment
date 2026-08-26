//! # UI Module
//!
//! Pure view layer using Macroquad immediate mode UI:
//! - Uses `UiAction` pattern to send intents back to the game logic.
//! - Contains sub-panels for different views (Building, Apartment, City).
//! - Floating text and panel-slide animations come from `macroquad-toolkit`
//!   (`fx::FloatingTextLayer`, `math::Tween`), re-exported below.
//! - Strictly separation of concerns: No game state mutation happens here.

pub mod theme;
pub mod widgets;

mod apartment_panel;
mod apartment_panel_sections;
mod application_panel;
mod building_view;
pub mod career_summary;
pub mod city_view; // Phase 3 city map
mod city_view_widgets;
mod common;
mod event_art;
pub mod event_modal; // Phase 4 event modal
mod hallway_panel;
mod header;
mod inspector_summary;
mod notifications;
pub mod ownership_panel; // Phase 3 ownership
mod resident_sprite;
mod tenant_panel;
pub mod workspace_inbox;
pub mod workspace_nav;
pub mod workspace_tasks;
pub mod workspace_views;

pub use apartment_panel::draw_apartment_panel;
pub use building_view::draw_building_view;
pub use common::*;
pub use hallway_panel::draw_hallway_panel;
pub use ownership_panel::draw_ownership_panel;
pub(crate) use resident_sprite::draw_resident_sprite_gallery;

pub use application_panel::draw_application_panel;
pub use header::draw_header;
pub use inspector_summary::draw_building_summary;
pub use macroquad_toolkit::fx::FloatingTextLayer;
pub use macroquad_toolkit::math::Tween;
pub use notifications::draw_notifications;

use serde::{Deserialize, Serialize};

/// What's currently selected for the detail panel
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum Selection {
    #[default]
    None,
    Apartment(u32),            // Apartment ID
    Applications(Option<u32>), // Show pending applications (Optionally filtered by apartment)
    Hallway,                   // Hallway details
    Ownership,                 // Ownership View
}

use crate::building::UpgradeAction;

/// UI action intents (returned to game logic)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UiAction {
    SelectApartment(u32),
    SelectApplications(Option<u32>),
    SelectHallway,
    SelectOwnership,
    ClearSelection,
    ToggleActivityDrawer,

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

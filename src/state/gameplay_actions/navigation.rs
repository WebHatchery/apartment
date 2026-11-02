//! Navigation intents: selection, workspace switching, paging and overlays.
//! None of these change the simulation.

use crate::state::{GameplayState, ViewMode};
use crate::ui::{Selection, UiAction};

impl GameplayState {
    /// Apply `action` if it only moves the player's view; otherwise hand it
    /// back for the simulation dispatcher.
    pub(in crate::state) fn navigate(&mut self, action: UiAction) -> Option<UiAction> {
        match action {
            UiAction::SelectApartment(id) => {
                self.selection = Selection::Apartment(id);
                self.panel_scroll_offset = 0.0;
                self.tower_focus_pending = true;
            }
            UiAction::OpenTenantUnit(id) => {
                self.view_mode = ViewMode::Building;
                self.selection = Selection::Apartment(id);
                self.panel_scroll_offset = 0.0;
                self.tower_focus_pending = true;
            }
            UiAction::SelectTenant(id) => {
                self.selected_tenant_id = Some(id);
            }
            UiAction::ClearTenantSelection => {
                self.selected_tenant_id = None;
            }
            UiAction::SelectApplications(filter) => {
                self.view_mode = ViewMode::Building;
                self.selection = Selection::Applications(filter);
                self.panel_scroll_offset = 0.0;
                self.application_page = 0;
            }
            UiAction::SelectBuilding => {
                self.view_mode = ViewMode::Building;
                self.selection = Selection::Building;
                self.panel_scroll_offset = 0.0;
            }
            UiAction::SelectHallway => {
                self.selection = Selection::Hallway;
                self.panel_scroll_offset = 0.0;
                self.tower_focus_pending = true;
            }
            UiAction::ToggleDisclosure(section) => self.panel_disclosure.toggle(section),
            UiAction::DismissMonthSummary => self.month_summary_open = false,
            UiAction::ToggleCoach => self.coach_hidden = !self.coach_hidden,
            UiAction::CloseHelp => self.help_open = false,
            UiAction::ClearSelection => {
                self.selection = Selection::None;
            }
            UiAction::ToggleActivityDrawer => {
                self.activity_drawer_open = !self.activity_drawer_open;
            }

            UiAction::OpenPauseMenu => {
                self.show_pause_menu = true;
            }
            UiAction::ReturnToMenu => {
                self.pending_quit_to_menu = true;
            }

            UiAction::OpenBuilding => {
                self.view_mode = ViewMode::Building;
                self.selection = Selection::None;
            }
            UiAction::OpenTenants => {
                self.view_mode = ViewMode::Tenants;
                self.selection = Selection::None;
            }
            UiAction::OpenFinances => {
                self.view_mode = ViewMode::Finances;
                self.selection = Selection::None;
            }
            UiAction::OpenCityMap => {
                self.save_building_to_city();
                self.view_mode = ViewMode::CityMap;
                self.selection = Selection::None;
            }
            UiAction::OpenMail => {
                self.view_mode = ViewMode::Mail;
            }
            UiAction::OpenTasks => {
                self.view_mode = ViewMode::Tasks;
            }
            UiAction::OpenMailItem { mail_id } => {
                if self.mailbox.mark_read(mail_id) {
                    self.selected_mail_id = Some(mail_id);
                }
            }
            UiAction::ClearSelectedMail => {
                self.selected_mail_id = None;
            }
            UiAction::SetInboxPage { page } => {
                self.inbox_page = page;
            }
            UiAction::SetTasksPage { page } => {
                self.tasks_page = page;
            }
            UiAction::SetApplicationPage { page } => {
                self.application_page = page;
            }
            UiAction::SetTenantsPage { page } => {
                self.tenants_page = page;
            }
            UiAction::SetRequestsPage { page } => {
                self.requests_page = page;
            }
            UiAction::SetCareerBadgesPage { page } => {
                self.career_badges_page = page;
            }
            UiAction::SelectOwnership => {
                self.selection = Selection::Ownership;
            }
            other => return Some(other),
        }
        None
    }
}

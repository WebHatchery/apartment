use serde::{Deserialize, Serialize};

/// Status of a mission
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MissionStatus {
    Available,
    Active,
    Completed,
    Failed,
    Expired,
}

/// Type of mission reward
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MissionReward {
    Money(i32),
    TaxBreak { months: u32, percentage: f32 },
    Reputation(i32),
    UnlockBuilding(u32),
}

/// A mission-earned property tax reduction that remains active for future months.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ActiveTaxBreak {
    pub remaining_months: u32,
    pub percentage: f32,
}

impl ActiveTaxBreak {
    pub fn new(months: u32, percentage: f32) -> Self {
        Self {
            remaining_months: months,
            percentage: percentage.clamp(0.0, 1.0),
        }
    }
}

/// A mission/quest in the game
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mission {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub giver_npc_id: u32,
    pub goal: MissionGoal,
    pub reward: MissionReward,
    pub status: MissionStatus,
    pub deadline: Option<u32>, // Month deadline
    pub started_month: Option<u32>,
}

/// Goals for missions
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MissionGoal {
    /// House a specific number of tenants of a type
    HouseTenants {
        count: u32,
        archetype: Option<String>,
    },
    /// Reach a certain occupancy rate
    ReachOccupancy { percentage: f32 },
    /// Maintain happiness above a threshold for X months
    MaintainHappiness {
        threshold: f32,
        months: u32,
        current_months: u32,
    },
    /// Collect rent without issues for X months
    PerfectCollection { months: u32, current_months: u32 },
    /// Repair all issues in a building
    FullRepair { building_id: u32 },
    /// Acquire a new building
    AcquireBuilding,
}

impl Mission {
    pub fn new(
        id: u32,
        title: &str,
        description: &str,
        giver_npc_id: u32,
        goal: MissionGoal,
        reward: MissionReward,
        deadline: Option<u32>,
    ) -> Self {
        Self {
            id,
            title: title.to_string(),
            description: description.to_string(),
            giver_npc_id,
            goal,
            reward,
            status: MissionStatus::Available,
            deadline,
            started_month: None,
        }
    }

    /// Start the mission
    pub fn start(&mut self, current_month: u32) {
        self.status = MissionStatus::Active;
        self.started_month = Some(current_month);
    }

    /// Complete the mission
    pub fn complete(&mut self) {
        self.status = MissionStatus::Completed;
    }

    /// Fail the mission
    pub fn fail(&mut self) {
        self.status = MissionStatus::Failed;
    }

    /// Check if mission has expired
    pub fn check_expired(&mut self, current_month: u32) -> bool {
        if let Some(deadline) = self.deadline {
            if current_month > deadline && self.status == MissionStatus::Active {
                self.status = MissionStatus::Expired;
                return true;
            }
        }
        false
    }
}

/// Manages all missions in the game
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionManager {
    pub missions: Vec<Mission>,
    pub next_mission_id: u32,
    /// Legacy tracking: major events that happened
    pub legacy_events: Vec<LegacyEvent>,
    /// Awards earned
    pub awards: Vec<BuildingAward>,
}

/// A major event recorded in the player's legacy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegacyEvent {
    pub month: u32,
    pub year: u32,
    pub title: String,
    pub description: String,
}

/// Building awards for the legacy system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingAward {
    pub year: u32,
    pub title: String,
    pub building_name: String,
}

/// A completed mission and the authored reward to apply. Goal evaluation is
/// shared by live gameplay and the balance harness; presentation and reward
/// side effects remain with their respective caller.
pub struct CompletedMission {
    pub title: String,
    pub description: String,
    pub reward: MissionReward,
}

impl MissionManager {
    pub fn new() -> Self {
        Self {
            missions: Vec::new(),
            next_mission_id: 0,
            legacy_events: Vec::new(),
            awards: Vec::new(),
        }
    }

    /// Add a new mission
    pub fn add_mission(&mut self, mut mission: Mission) -> u32 {
        let id = self.next_mission_id;
        mission.id = id;
        self.next_mission_id += 1;
        self.missions.push(mission);
        id
    }

    /// Get available missions
    pub fn available_missions(&self) -> Vec<&Mission> {
        self.missions
            .iter()
            .filter(|m| m.status == MissionStatus::Available)
            .collect()
    }

    /// Get active missions
    pub fn active_missions(&self) -> Vec<&Mission> {
        self.missions
            .iter()
            .filter(|m| m.status == MissionStatus::Active)
            .collect()
    }

    /// Get completed missions
    pub fn completed_missions(&self) -> Vec<&Mission> {
        self.missions
            .iter()
            .filter(|m| m.status == MissionStatus::Completed)
            .collect()
    }

    /// Accept a mission by ID
    pub fn accept_mission(&mut self, mission_id: u32, current_month: u32) -> bool {
        if let Some(mission) = self.missions.iter_mut().find(|m| m.id == mission_id) {
            if mission.status == MissionStatus::Available {
                mission.start(current_month);
                return true;
            }
        }
        false
    }

    /// Check all active missions for expiration
    pub fn check_expirations(&mut self, current_month: u32) {
        for mission in &mut self.missions {
            mission.check_expired(current_month);
        }
    }

    /// Record a legacy event
    pub fn record_legacy_event(&mut self, month: u32, title: &str, description: &str) {
        let year = 2024 + (month / 12);
        self.legacy_events.push(LegacyEvent {
            month,
            year,
            title: title.to_string(),
            description: description.to_string(),
        });
    }

    /// Award a building award
    pub fn grant_award(&mut self, year: u32, title: &str, building_name: &str) {
        self.awards.push(BuildingAward {
            year,
            title: title.to_string(),
            building_name: building_name.to_string(),
        });
    }

    /// Check for annual awards (call at end of each year / month 12, 24, etc.)
    pub fn check_for_awards(
        &mut self,
        current_month: u32,
        building_name: &str,
        avg_happiness: f32,
        occupancy_rate: f32,
        total_tenants_housed: u32,
    ) {
        // Only check at year boundaries
        if !current_month.is_multiple_of(12) || current_month == 0 {
            return;
        }

        let year = 2024 + (current_month / 12);

        // Check if we already have an award for this year
        if self.awards.iter().any(|a| a.year == year) {
            return;
        }

        // Best Managed Property - high happiness
        if avg_happiness >= 80.0 {
            self.grant_award(year, "Best Managed Property", building_name);
            self.record_legacy_event(
                current_month,
                "Award Won!",
                &format!("Won 'Best Managed Property {}' for {}", year, building_name),
            );
        }
        // Full Occupancy Achievement
        else if occupancy_rate >= 1.0 {
            self.grant_award(year, "Perfect Occupancy", building_name);
            self.record_legacy_event(
                current_month,
                "Award Won!",
                &format!("Achieved 100% occupancy at {} in {}", building_name, year),
            );
        }
        // Community Builder - housed many tenants
        else if total_tenants_housed >= 20 {
            self.grant_award(year, "Community Builder", building_name);
            self.record_legacy_event(
                current_month,
                "Award Won!",
                &format!(
                    "Housed {} tenants at {} by {}",
                    total_tenants_housed, building_name, year
                ),
            );
        }
    }

    /// Add every mission from `assets/missions.json` whose `min_month` has
    /// arrived and that isn't already present. Called at game start (month 0)
    /// and each month, this replaces the old hardcoded starter/late-game
    /// generators — mission content now lives in data, not Rust.
    pub fn generate_available_missions(&mut self, current_month: u32) {
        for template in load_mission_templates() {
            if template.min_month > current_month {
                continue;
            }
            if self.missions.iter().any(|m| m.title == template.title) {
                continue;
            }
            // A relative deadline is measured from when the mission unlocks.
            let deadline = template
                .deadline_months
                .map(|months| template.min_month + months);
            let mission = Mission::new(
                0,
                &template.title,
                &template.description,
                template.giver_npc_id,
                template.goal,
                template.reward,
                deadline,
            );
            self.add_mission(mission);
        }
    }

    /// Evaluate active missions from an authoritative monthly snapshot.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_active(
        &mut self,
        current_month: u32,
        active_building_id: u32,
        active_tenants: &[crate::tenant::Tenant],
        occupancy: f32,
        avg_happiness: f32,
        perfect_collection: bool,
        fully_repaired_by_building: &std::collections::HashMap<u32, bool>,
        building_count: usize,
    ) -> Vec<CompletedMission> {
        self.check_expirations(current_month);
        if building_count == 0 {
            for mission in &mut self.missions {
                if mission.status == MissionStatus::Active
                    && matches!(mission.goal, MissionGoal::AcquireBuilding)
                {
                    mission.fail();
                }
            }
        }

        let mut completed = Vec::new();
        for mission in &mut self.missions {
            if mission.status != MissionStatus::Active {
                continue;
            }
            let goal_met = match &mut mission.goal {
                MissionGoal::HouseTenants { count, archetype } => {
                    active_tenants
                        .iter()
                        .filter(|tenant| {
                            tenant.building_id == active_building_id
                                && archetype
                                    .as_ref()
                                    .is_none_or(|name| tenant.archetype.name() == name)
                        })
                        .count() as u32
                        >= *count
                }
                MissionGoal::ReachOccupancy { percentage } => occupancy >= *percentage,
                MissionGoal::AcquireBuilding => building_count > 1,
                MissionGoal::MaintainHappiness {
                    threshold,
                    months,
                    current_months,
                } => {
                    if !active_tenants.is_empty() && avg_happiness >= *threshold {
                        *current_months += 1;
                    } else {
                        *current_months = 0;
                    }
                    *current_months >= *months
                }
                MissionGoal::PerfectCollection {
                    months,
                    current_months,
                } => {
                    if perfect_collection {
                        *current_months += 1;
                    } else {
                        *current_months = 0;
                    }
                    *current_months >= *months
                }
                MissionGoal::FullRepair { building_id } => fully_repaired_by_building
                    .get(building_id)
                    .copied()
                    .unwrap_or(false),
            };
            if goal_met {
                mission.complete();
                completed.push(CompletedMission {
                    title: mission.title.clone(),
                    description: mission.description.clone(),
                    reward: mission.reward.clone(),
                });
            }
        }
        completed
    }
}

/// A mission as authored in `assets/missions.json`, before runtime fields
/// (id/status/started_month) are assigned.
#[derive(Clone, Debug, Deserialize)]
struct MissionTemplate {
    title: String,
    description: String,
    giver_npc_id: u32,
    #[serde(default)]
    min_month: u32,
    #[serde(default)]
    deadline_months: Option<u32>,
    goal: MissionGoal,
    reward: MissionReward,
}

fn load_mission_templates() -> Vec<MissionTemplate> {
    macroquad_toolkit::data_loader::load_json_file_with_fallback_sync(
        "assets/missions.json",
        macroquad_toolkit::include_json_str!("../../assets/missions.json"),
        macroquad_toolkit::data_loader::JsonFallbackPolicy::ReadError,
    )
    .unwrap_or_else(|e| {
        eprintln!("Failed to parse missions.json: {}", e);
        Vec::new()
    })
}

impl Default for MissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;

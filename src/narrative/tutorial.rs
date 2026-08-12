use serde::{Deserialize, Serialize};

/// An NPC character in the narrative system
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NarrativeNpc {
    pub id: u32,
    pub name: String,
    pub role: NpcRole,
    pub relationship: i32, // -100 to 100
}

/// Role of the NPC in the game
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NpcRole {
    Mentor,
    Rival,
    Ally,
    Neutral,
}

impl NarrativeNpc {
    pub fn new(id: u32, name: &str, role: NpcRole) -> Self {
        let relationship = match &role {
            NpcRole::Mentor => 50,
            NpcRole::Ally => 30,
            NpcRole::Rival => -30,
            NpcRole::Neutral => 0,
        };
        Self {
            id,
            name: name.to_string(),
            role,
            relationship,
        }
    }
}

/// Tutorial milestones for the guided experience
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TutorialMilestone {
    /// Clear the first building of trash/issues
    InheritedMess,
    /// Complete the first tenant acquisition flow
    FirstResident,
    /// Handle a repair decision
    TheLeak,
    /// Completed the tutorial
    Complete,
}

/// Manages tutorial state and progression
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TutorialManager {
    pub active: bool,
    pub current_milestone: Option<TutorialMilestone>,
    pub completed_milestones: Vec<TutorialMilestone>,
    pub mentor: NarrativeNpc,
    pub rivals: Vec<NarrativeNpc>,
    pub allies: Vec<NarrativeNpc>,
    /// Messages queued from the mentor
    pub pending_messages: Vec<String>,
    /// Whether the rival has been introduced
    pub rival_introduced: bool,
    /// Number of residents present when the guided lease milestone begins.
    /// Starter tenants therefore do not skip the acquisition lesson.
    #[serde(default)]
    pub resident_baseline: usize,
    /// Last month in which the contextual hint was emitted. `None` also lets
    /// old saves enter the new one-hint-per-month behavior cleanly.
    #[serde(default)]
    pub last_hint_month: Option<u32>,
}

impl TutorialManager {
    pub fn new() -> Self {
        // Create Uncle Artie as the mentor
        let mentor = NarrativeNpc::new(0, "Uncle Artie", NpcRole::Mentor);

        // Create initial rivals
        let rivals = vec![NarrativeNpc::new(1, "Magnuson Corp", NpcRole::Rival)];

        // Create initial allies
        let allies = vec![NarrativeNpc::new(2, "Councilwoman Reyes", NpcRole::Ally)];

        Self {
            active: true,
            current_milestone: Some(TutorialMilestone::InheritedMess),
            completed_milestones: Vec::new(),
            mentor,
            rivals,
            allies,
            pending_messages: vec![
                "Welcome! I'm your Uncle Artie. I've left you this building in my will."
                    .to_string(),
                "It's a bit of a mess, but nothing we can't fix together.".to_string(),
                "First, select the Hallway and repair it to fix up the place.".to_string(),
            ],
            rival_introduced: false,
            resident_baseline: 0,
            last_hint_month: None,
        }
    }

    pub fn set_resident_baseline(&mut self, residents: usize) {
        self.resident_baseline = residents;
    }

    pub fn has_new_resident(&self, residents: usize) -> bool {
        residents > self.resident_baseline
    }

    /// Hints are evaluated every frame, but should only be emitted once for a
    /// qualifying month rather than dozens of times during that frame loop.
    pub fn should_emit_hint(&mut self, month: u32) -> bool {
        if month == 0 || !month.is_multiple_of(5) || self.last_hint_month == Some(month) {
            return false;
        }
        self.last_hint_month = Some(month);
        true
    }

    /// Check if a milestone is completed
    pub fn is_milestone_complete(&self, milestone: &TutorialMilestone) -> bool {
        self.completed_milestones.contains(milestone)
    }

    /// Complete a milestone and advance to the next
    pub fn complete_milestone(&mut self, milestone: TutorialMilestone) {
        if !self.is_milestone_complete(&milestone) {
            self.completed_milestones.push(milestone.clone());

            // Advance to next milestone
            self.current_milestone = match milestone {
                TutorialMilestone::InheritedMess => {
                    self.pending_messages.push(
                        "Great job cleaning up! Now let's find your first tenant.".to_string(),
                    );
                    self.pending_messages.push(
                        "Tap an apartment, adjust RENT if needed, then tap LIST FOR LEASE."
                            .to_string(),
                    );
                    self.pending_messages.push(
                        "Then tap END MONTH to let time pass. Applicants will arrive!".to_string(),
                    );
                    Some(TutorialMilestone::FirstResident)
                }
                TutorialMilestone::FirstResident => {
                    self.pending_messages
                        .push("Excellent! You've got your first resident.".to_string());
                    self.pending_messages.push(
                        "But uh-oh, looks like there's a leak in one of the units...".to_string(),
                    );
                    Some(TutorialMilestone::TheLeak)
                }
                TutorialMilestone::TheLeak => {
                    self.pending_messages
                        .push("You handled that like a pro!".to_string());
                    self.pending_messages
                        .push("I think you're ready to manage on your own now.".to_string());
                    self.pending_messages
                        .push("Good luck, and remember - treat your tenants well!".to_string());
                    Some(TutorialMilestone::Complete)
                }
                TutorialMilestone::Complete => {
                    self.active = false;
                    None
                }
            };
        }
    }

    /// Check if tutorial is complete
    pub fn is_complete(&self) -> bool {
        self.is_milestone_complete(&TutorialMilestone::Complete)
    }

    /// Get NPC by ID
    pub fn get_npc(&self, id: u32) -> Option<&NarrativeNpc> {
        if self.mentor.id == id {
            return Some(&self.mentor);
        }
        if let Some(npc) = self.rivals.iter().find(|n| n.id == id) {
            return Some(npc);
        }
        if let Some(npc) = self.allies.iter().find(|n| n.id == id) {
            return Some(npc);
        }
        None
    }

    /// Modify relationship with an NPC
    pub fn modify_relationship(&mut self, npc_id: u32, change: i32) {
        if self.mentor.id == npc_id {
            self.mentor.relationship = (self.mentor.relationship + change).clamp(-100, 100);
            return;
        }
        if let Some(npc) = self.rivals.iter_mut().find(|n| n.id == npc_id) {
            npc.relationship = (npc.relationship + change).clamp(-100, 100);
            return;
        }
        if let Some(npc) = self.allies.iter_mut().find(|n| n.id == npc_id) {
            npc.relationship = (npc.relationship + change).clamp(-100, 100);
        }
    }

    /// Get a contextual hint for the current milestone
    pub fn get_hint(&self) -> Option<&'static str> {
        match &self.current_milestone {
            Some(TutorialMilestone::InheritedMess) => {
                Some("Hint: Click the Hallway and repair it to 80+ condition.")
            }
            Some(TutorialMilestone::FirstResident) => {
                Some("Hint: List an apartment for lease, End Month, then check Applications.")
            }
            Some(TutorialMilestone::TheLeak) => {
                Some("Hint: A unit has low condition. Repair it to proceed!")
            }
            Some(TutorialMilestone::Complete) | None => None,
        }
    }

    /// Check if the player should see the rival introduction
    pub fn should_introduce_rival(&self, month: u32) -> bool {
        // Introduce Magnuson Corp after first 6 months
        month >= 6 && !self.is_complete()
    }
}

impl Default for TutorialManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;

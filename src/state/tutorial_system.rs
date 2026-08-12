use super::GameplayState;
use crate::narrative::TutorialMilestone;
use crate::ui::colors;
use macroquad::prelude::*;

/// System for handling tutorial updates and milestones
pub fn update_tutorial(state: &mut GameplayState) {
    // Skip if tutorial is complete
    if state.tutorial.is_complete() {
        return;
    }

    if !state.tutorial.active {
        return;
    }

    // Check if we should introduce the rival (Magnuson Corp)
    if state.tutorial.should_introduce_rival(state.current_tick) && !state.tutorial.rival_introduced
    {
        // Get rival NPC info and add an introduction message
        if let Some(rival) = state.tutorial.get_npc(1) {
            // Magnuson Corp ID = 1
            let rival_name = rival.name.clone();
            state.tutorial.pending_messages.push(format!(
                "I hear {} has been buying up properties nearby. Watch out for them!",
                rival_name
            ));
            state.tutorial.rival_introduced = true;
        }
    }

    // Display hint for current milestone if stuck for a while
    if state.tutorial.should_emit_hint(state.current_tick) {
        if let Some(hint) = state.tutorial.get_hint() {
            state.floating_texts.spawn(
                hint,
                vec2(screen_width() / 2.0, screen_height() - 100.0),
                colors::TEXT_DIM(),
            );
        }
    }

    if let Some(milestone) = &state.tutorial.current_milestone {
        match milestone {
            TutorialMilestone::InheritedMess => {
                // Completes when building is sufficiently clean/repaired
                // For MVP: Hallway condition > 80
                if state.building.hallway_condition >= 80 {
                    // Improve relationship with mentor for completing milestone
                    state.tutorial.modify_relationship(0, 10); // Uncle Artie ID = 0

                    state
                        .tutorial
                        .complete_milestone(TutorialMilestone::InheritedMess);
                    state.floating_texts.spawn(
                        "Tutorial: Cleaned Up!",
                        vec2(screen_width() / 2.0, screen_height() / 2.0),
                        colors::POSITIVE(),
                    );
                }
            }
            TutorialMilestone::FirstResident => {
                // Completes only after acquiring somebody beyond any starter
                // resident supplied by the scenario template.
                if state.tutorial.has_new_resident(state.active_tenant_count()) {
                    state.tutorial.modify_relationship(0, 15); // Mentor happy

                    state
                        .tutorial
                        .complete_milestone(TutorialMilestone::FirstResident);
                    state.floating_texts.spawn(
                        "Tutorial: First Resident!",
                        vec2(screen_width() / 2.0, screen_height() / 2.0 + 30.0),
                        colors::POSITIVE(),
                    );

                    // Trigger The Leak event immediately implies a problem
                    // We can sabotage a unit or just let narrative flow
                    if let Some(apt) = state.building.apartments.first_mut() {
                        apt.condition = 20; // Critical condition for visual urgency
                    }

                    // Visual cue
                    state.floating_texts.spawn(
                        "LEAK DETECTED!",
                        vec2(screen_width() / 2.0, screen_height() / 2.0 + 60.0),
                        colors::NEGATIVE(),
                    );
                }
            }
            TutorialMilestone::TheLeak => {
                // Check if repairs are done (no units in critical condition)
                // We check < 30 because the leak sets it to 20. Repair (+25) brings it to 45.
                // If they just did a small repair (+10), it would be 30.
                let all_good = !state.building.apartments.iter().any(|a| a.condition < 30);
                if all_good {
                    state.tutorial.modify_relationship(0, 20); // Mentor very happy

                    state
                        .tutorial
                        .complete_milestone(TutorialMilestone::TheLeak);
                    state.floating_texts.spawn(
                        "Tutorial Complete!",
                        vec2(screen_width() / 2.0, screen_height() / 2.0),
                        colors::POSITIVE(),
                    );

                    // Messages are already in pending_messages and will be shown by the tutorial overlay
                }
            }
            TutorialMilestone::Complete => {
                if state.tutorial.pending_messages.is_empty() {
                    state
                        .tutorial
                        .complete_milestone(TutorialMilestone::Complete);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;

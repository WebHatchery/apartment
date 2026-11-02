//! Monthly wear on units and the lobby, with milestone events.

use super::GameEvent;
use crate::building::Building;
use crate::data::config::{DecayConfig, ThresholdsConfig};

/// Apply monthly decay to all building elements
/// Returns events for significant decay milestones
pub fn apply_decay(
    building: &mut Building,
    decay: &DecayConfig,
    thresholds: &ThresholdsConfig,
) -> Vec<GameEvent> {
    let mut events = Vec::new();

    // Track conditions before decay for event generation
    let conditions_before: Vec<_> = building
        .apartments
        .iter()
        .map(|a| (a.id, a.unit_number.clone(), a.condition))
        .collect();

    let hallway_before = building.hallway_condition;

    // Apply decay
    building.apply_monthly_decay(decay.apartment_per_tick, decay.hallway_per_tick);

    // Check for significant condition changes in apartments
    for (id, unit, old_condition) in conditions_before {
        if let Some(apt) = building.get_apartment(id) {
            let new_condition = apt.condition;

            // Check for crossing thresholds
            if old_condition >= thresholds.critical_condition
                && new_condition < thresholds.critical_condition
            {
                events.push(GameEvent::CriticalCondition {
                    apartment_unit: unit,
                    condition: new_condition,
                });
            } else if old_condition >= thresholds.poor_condition
                && new_condition < thresholds.poor_condition
            {
                events.push(GameEvent::PoorCondition {
                    apartment_unit: unit,
                    condition: new_condition,
                });
            }
        }
    }

    // Check hallway
    let hallway_after = building.hallway_condition;
    if hallway_before >= thresholds.poor_condition && hallway_after < thresholds.poor_condition {
        events.push(GameEvent::HallwayDeteriorating {
            condition: hallway_after,
        });
    }

    events
}

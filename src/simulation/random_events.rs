//! Random monthly building events such as emergencies and heatwaves.

use crate::building::Building;
use crate::data::config::TurnEffectsConfig;
use crate::economy::{PlayerFunds, Transaction, TransactionType};
use crate::simulation::events::GameEvent;
use macroquad_toolkit::rng;

/// Roll this month's random building events. Duration events (heatwave,
/// gentrification) are only emitted here; `GameplayState` tracks and applies
/// them as active world events.
pub fn roll_random_events(
    building: &mut Building,
    funds: &mut PlayerFunds,
    current_tick: u32,
    effects: &TurnEffectsConfig,
) -> Vec<GameEvent> {
    let mut events = Vec::new();

    // 1. Heatwave
    if rng::gen_range(0, 100) < effects.heatwave_chance_percent {
        events.push(GameEvent::Heatwave {
            tick_duration: effects.heatwave_duration_months,
        });
    }

    // 2. Pipe burst in ONE apartment
    if rng::gen_range(0, 100) < effects.pipe_burst_chance_percent {
        let num_apts = building.apartments.len();
        if num_apts > 0 {
            let idx = rng::gen_range(0, num_apts);
            if let Some(apt) = building.apartments.get_mut(idx) {
                let damage = effects.pipe_burst_damage;
                let old_condition = apt.condition;
                apt.condition = (apt.condition - damage).max(0);

                // Only report if it actually did damage or if it's a significant event
                if old_condition > 0 {
                    events.push(GameEvent::PipeBurst {
                        apartment_unit: apt.unit_number.clone(),
                        damage,
                    });
                }
            }
        }
    }

    // 3. Gentrification (very rare)
    if rng::gen_range(0, 1000) < effects.gentrification_chance_per_1000 {
        events.push(GameEvent::Gentrification {
            tick_duration: effects.gentrification_duration_months,
            effect_desc: "Rent tolerance +20%".to_string(),
        });
    }

    // 4. Spot inspection, likelier (and failed) when building appeal is low.
    let appeal = building.building_appeal();
    let threshold = effects.spot_inspection_appeal_threshold;
    let inspection_chance = if appeal < threshold {
        effects.spot_inspection_low_appeal_chance_percent
    } else {
        effects.spot_inspection_chance_percent
    };

    if rng::gen_range(0, 100) < inspection_chance {
        let passed = appeal >= threshold;
        let fine = if passed {
            0
        } else {
            effects.spot_inspection_fine
        };

        if fine > 0 {
            // Deduct fine
            funds.apply_required_expense(Transaction::expense(
                TransactionType::InspectionFine,
                fine,
                "Failed Inspection Fine",
                current_tick,
            ));
        }

        events.push(GameEvent::Inspection {
            result: if passed {
                "Passed".to_string()
            } else {
                format!("Failed (Building Appeal < {threshold})")
            },
            fine,
        });
    }

    events
}

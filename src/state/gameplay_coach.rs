//! Tutorial coaching and the Help card. The coach points at the place it
//! describes using the same geometry the building and header draw with, can
//! be hidden and reopened (pause menu Tips) without touching tutorial
//! progress, and gives a one-line panel instruction while a room is open.

use std::collections::HashMap;
use std::sync::OnceLock;

use macroquad::prelude::*;
use macroquad_toolkit::data_loader::{load_json_file_with_fallback_sync, JsonFallbackPolicy};
use macroquad_toolkit::ui::draw_ui_text;
use serde::Deserialize;

use super::gameplay::{GameplayState, ViewMode};
use crate::assets::AssetManager;
use crate::narrative::TutorialMilestone;
use crate::ui::building_view::{place_rect, reveal_place, Place};
use crate::ui::theme::{color, scale, space, Tone};
use crate::ui::widgets::{button_at, draw_card, draw_panel, line_height, wrap};
use crate::ui::{Selection, UiAction};

#[derive(Debug, Default, Deserialize)]
struct StepText {
    world: String,
    panel: String,
}

#[derive(Debug, Default, Deserialize)]
struct CoachText {
    title: String,
    hide_label: String,
    steps: HashMap<String, StepText>,
    help_title: String,
    help: Vec<String>,
}

/// One tutorial step: what it is called in `tutorial_coach.json`, where the
/// player should tap, and which open panel it continues in.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Step {
    RepairLobby,
    ListUnit(u32),
    EndMonth,
    ChooseApplicant,
    RepairDamage(u32),
}

const STEP_KEYS: [&str; 5] = [
    "repair_lobby",
    "list_unit",
    "end_month",
    "choose_applicant",
    "repair_damage",
];

impl Step {
    fn key(self) -> &'static str {
        match self {
            Step::RepairLobby => STEP_KEYS[0],
            Step::ListUnit(_) => STEP_KEYS[1],
            Step::EndMonth => STEP_KEYS[2],
            Step::ChooseApplicant => STEP_KEYS[3],
            Step::RepairDamage(_) => STEP_KEYS[4],
        }
    }

    /// The building place the step points at (End Month lives in the header).
    fn place(self) -> Option<Place> {
        match self {
            Step::RepairLobby => Some(Place::Lobby),
            Step::ListUnit(id) | Step::RepairDamage(id) => Some(Place::Unit(id)),
            Step::ChooseApplicant => Some(Place::DetailsButton),
            Step::EndMonth => None,
        }
    }

    /// Whether `selection` is the panel this step continues in.
    fn continues_in(self, selection: &Selection) -> bool {
        match (self, selection) {
            (Step::RepairLobby, Selection::Hallway) => true,
            (Step::ListUnit(id) | Step::RepairDamage(id), Selection::Apartment(open)) => {
                id == *open
            }
            (Step::ChooseApplicant, Selection::Building | Selection::Applications(_)) => true,
            (Step::EndMonth, selection) => *selection != Selection::None,
            _ => false,
        }
    }
}

fn coach_text() -> &'static CoachText {
    static TEXT: OnceLock<CoachText> = OnceLock::new();
    TEXT.get_or_init(|| {
        let loaded = load_json_file_with_fallback_sync::<CoachText>(
            "assets/tutorial_coach.json",
            macroquad_toolkit::include_json_str!("../../assets/tutorial_coach.json"),
            JsonFallbackPolicy::ReadError,
        )
        .map_err(|error| error.to_string())
        .and_then(|text| {
            match STEP_KEYS.iter().find(|key| !text.steps.contains_key(**key)) {
                Some(missing) => Err(format!("missing step '{missing}'")),
                None if text.help.is_empty() => Err("help must not be empty".to_string()),
                None => Ok(text),
            }
        });
        loaded.unwrap_or_else(|error| {
            eprintln!("Invalid tutorial_coach.json: {error}");
            CoachText::default()
        })
    })
}

impl GameplayState {
    fn coach_step(&self) -> Option<Step> {
        let building_id = self.active_building_id();
        match self.tutorial.current_milestone.as_ref()? {
            TutorialMilestone::InheritedMess => Some(Step::RepairLobby),
            TutorialMilestone::FirstResident => {
                if self
                    .applications
                    .iter()
                    .any(|application| application.building_id == building_id)
                {
                    return Some(Step::ChooseApplicant);
                }
                let vacant = self
                    .building
                    .apartments
                    .iter()
                    .filter(|unit| unit.is_vacant() && !self.building.is_unit_sold(unit.id));
                if vacant.clone().any(|unit| unit.is_listed_for_lease) {
                    return Some(Step::EndMonth);
                }
                vacant.map(|unit| Step::ListUnit(unit.id)).next()
            }
            TutorialMilestone::TheLeak => self
                .building
                .apartments
                .iter()
                .filter(|unit| unit.condition < 30)
                .min_by_key(|unit| unit.condition)
                .map(|unit| Step::RepairDamage(unit.id)),
            TutorialMilestone::Complete => None,
        }
    }

    /// Whether the coach may draw now: building view, tutorial running, no
    /// message, summary, choice or drawer owning the screen.
    pub(super) fn coach_visible(&self, blocking_event_open: bool) -> bool {
        self.view_mode == ViewMode::Building
            && self.tutorial.active
            && !self.coach_hidden
            && self.tutorial.pending_messages.is_empty()
            && !self.activity_drawer_open
            && !self.month_summary_open
            && !blocking_event_open
    }

    /// Screen rectangle of the thing the step asks the player to tap.
    fn coach_target(&self, step: Step) -> Option<Rect> {
        let Some(place) = step.place() else {
            return Some(crate::ui::header::end_month_rect());
        };
        place_rect(
            &self.building,
            &self.selection,
            self.tower_scroll_offset,
            place,
        )
    }

    pub(super) fn draw_tutorial_coach(&mut self, assets: &AssetManager) {
        let Some(step) = self.coach_step() else {
            return;
        };
        let text = coach_text();
        let Some(step_text) = text.steps.get(step.key()) else {
            return;
        };
        let in_panel = self.selection != Selection::None;
        // Once per step, scroll the tower so the place it names is in view;
        // afterwards the player scrolls freely.
        let step_id = format!("{step:?}");
        if !in_panel && self.coach_framed_step.as_deref() != Some(step_id.as_str()) {
            if let Some(place) = step.place() {
                self.tower_scroll_offset = reveal_place(
                    &self.building,
                    &self.selection,
                    place,
                    self.tower_scroll_offset,
                );
            }
            self.coach_framed_step = Some(step_id);
        }
        // In the step's own panel, say what to tap there; elsewhere point
        // back at the building, which stays visible beside any panel.
        let message = if in_panel && step.continues_in(&self.selection) {
            &step_text.panel
        } else {
            &step_text.world
        };
        let world = crate::ui::layout::building(in_panel).world;
        let target = (!in_panel || step == Step::EndMonth)
            .then(|| self.coach_target(step))
            .flatten();

        // Beside an open panel the coach shrinks to one banner at the top of
        // the tower, clear of its scroll controls and of the panel itself.
        let compact = in_panel;
        let hide_w = 72.0;
        let card_w = (world.w - space::LG * 2.0).min(if compact { 520.0 } else { 380.0 });
        let text_w = if compact {
            card_w - space::LG - hide_w - space::SM * 2.0
        } else {
            card_w - space::LG * 2.0
        };
        let lines = wrap(message, text_w, scale::BODY);
        let header_h = if compact { space::SM } else { 52.0 };
        let card_h = (header_h + lines.len() as f32 * line_height(scale::BODY) + space::SM)
            .max(if compact { 52.0 } else { 0.0 });
        // Sit at the end of the workspace away from the target.
        let target_low = target.is_some_and(|rect| rect.center().y > world.center().y);
        // Keep clear of the tower's Up/Down strip at the bottom.
        let card_y = if compact || target_low {
            world.y + space::SM
        } else {
            world.bottom() - 58.0 - card_h
        };
        let card = Rect::new(world.x + (world.w - card_w) * 0.5, card_y, card_w, card_h);

        if let Some(target) = target {
            let anchor = vec2(
                target.center().x,
                if target_low {
                    target.y
                } else {
                    target.bottom()
                },
            );
            let from_y = if target_low { card.bottom() } else { card.y };
            draw_rectangle_lines(
                target.x,
                target.y,
                target.w,
                target.h,
                3.0,
                color::PRIMARY(),
            );
            draw_line(
                anchor.x.clamp(card.x, card.right()),
                from_y,
                anchor.x,
                anchor.y,
                3.0,
                color::PRIMARY(),
            );
            draw_circle(anchor.x, anchor.y, 6.0, color::PRIMARY());
        }

        // The card owns its area: nothing beneath it may react this frame.
        if card.contains(mouse_position().into()) {
            self.pending_actions.clear();
        }
        draw_card(card, true);
        if !compact {
            let title_x = if let Some(portrait) = assets.get_texture("npc_uncle_artie") {
                let size = 36.0;
                draw_texture_ex(
                    portrait,
                    card.x + space::SM,
                    card.y + space::SM,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(size, size)),
                        ..Default::default()
                    },
                );
                card.x + space::SM * 2.0 + size
            } else {
                card.x + space::LG
            };
            draw_ui_text(
                &text.title,
                title_x,
                card.y + 30.0,
                scale::LABEL,
                color::PRIMARY(),
            );
        }
        if button_at(
            Rect::new(card.right() - hide_w - 4.0, card.y + 4.0, hide_w, 44.0),
            &text.hide_label,
            true,
            Tone::Secondary,
        ) {
            self.pending_actions.push(UiAction::ToggleCoach);
        }
        let mut y = card.y + header_h;
        for line in lines {
            draw_ui_text(
                &line,
                card.x + space::LG,
                y + scale::BODY,
                scale::BODY,
                color::TEXT(),
            );
            y += line_height(scale::BODY);
        }
    }

    /// The general Help card (pause menu, after the tutorial). Owns input.
    pub(super) fn draw_help_card(&mut self) {
        let text = coach_text();
        let top = crate::ui::layout::HEADER_HEIGHT();
        let available = screen_height() - top - crate::ui::layout::FOOTER_HEIGHT() - space::LG;
        draw_rectangle(
            0.0,
            top,
            screen_width(),
            screen_height() - top,
            Color::new(0.0, 0.0, 0.0, 0.55),
        );
        let w = (screen_width() - space::LG * 2.0).min(520.0);
        let text_w = w - space::PAD * 2.0 - space::LG;
        let lines: Vec<Vec<String>> = text
            .help
            .iter()
            .map(|tip| wrap(tip, text_w, scale::BODY))
            .collect();
        let body: f32 = lines
            .iter()
            .map(|tip| tip.len() as f32 * line_height(scale::BODY) + space::XS)
            .sum();
        let h = (38.0 + space::SM + space::PAD * 2.0 + body + 44.0 + space::SM).min(available);
        let card = Rect::new(
            (screen_width() - w) * 0.5,
            top + (available - h) * 0.5 + space::SM,
            w,
            h,
        );
        let inner = draw_panel(card, &text.help_title);
        let close = Rect::new(inner.x, inner.bottom() - 44.0, inner.w, 44.0);
        let mut y = inner.y;
        'tips: for tip in lines {
            for (index, line) in tip.iter().enumerate() {
                if y + line_height(scale::BODY) > close.y {
                    break 'tips;
                }
                if index == 0 {
                    draw_circle(inner.x + 4.0, y + scale::BODY * 0.6, 3.0, color::PRIMARY());
                }
                draw_ui_text(
                    line,
                    inner.x + space::LG,
                    y + scale::BODY,
                    scale::BODY,
                    color::TEXT(),
                );
                y += line_height(scale::BODY);
            }
            y += space::XS;
        }
        self.pending_actions
            .retain(|action| matches!(action, UiAction::OpenPauseMenu));
        if button_at(close, "Close", true, Tone::Primary) {
            self.pending_actions.push(UiAction::CloseHelp);
        }
    }
}

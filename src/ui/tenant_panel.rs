//! Resident and vacancy sections of the unit inspector.

use crate::assets::AssetManager;
use crate::building::Apartment;
use crate::consequences::TenantNetwork;
use crate::narrative::{TenantRequest, TenantStory};
use crate::tenant::{Tenant, TenantArchetype};
use macroquad::prelude::*;
use std::collections::HashMap;

use super::panel_flow::PanelFlow;
use super::theme::{scale, Tone};
use super::{common::*, UiAction};
use macroquad_toolkit::ui::draw_ui_text;

const PORTRAIT: f32 = 52.0;

/// Portrait, name, archetype and relationship chips in one fixed row.
pub(super) fn draw_resident_identity(
    flow: &mut PanelFlow,
    tenant: &Tenant,
    network: &TenantNetwork,
    assets: &AssetManager,
) {
    let row_h = PORTRAIT + 4.0;
    if flow.fits(row_h) {
        let (x, y) = (flow.x, flow.y);
        let has_portrait = super::resident_sprite::draw_face_portrait(
            tenant,
            Rect::new(x, y, PORTRAIT, PORTRAIT),
            assets,
        );
        if !has_portrait {
            draw_rectangle(x, y, 4.0, PORTRAIT, archetype_color(&tenant.archetype));
        }
        let text_x = if has_portrait {
            x + PORTRAIT + 10.0
        } else {
            x + 12.0
        };
        let name = macroquad_toolkit::ui::truncate_text_to_width(
            &tenant.name,
            flow.x + flow.w - text_x,
            scale::HEADING,
        );
        draw_ui_text(&name, text_x, y + 18.0, scale::HEADING, colors::TEXT());
        draw_ui_text(
            &format!(
                "{} · {} months",
                tenant.archetype.name(),
                tenant.months_residing
            ),
            text_x,
            y + 36.0,
            scale::LABEL,
            colors::TEXT_DIM(),
        );
        draw_relationship_icons(tenant.id, network, text_x, y + 42.0);
    }
    flow.advance(row_h);
}

/// Who they are beyond the lease: background from their story.
pub(super) fn draw_resident_details(
    flow: &mut PanelFlow,
    tenant: &Tenant,
    stories: &HashMap<u32, TenantStory>,
) {
    if let Some(story) = stories.get(&tenant.id) {
        flow.kv("Work", &story.job_title, colors::TEXT());
        flow.kv("From", &story.hometown, colors::TEXT());
        flow.paragraph(
            &format!("Moved in: {}", story.move_reason),
            scale::LABEL,
            colors::TEXT_DIM(),
        );
    }
    flow.kv(
        "Tenure",
        &format!("{} months", tenant.months_residing),
        colors::TEXT_DIM(),
    );
}

fn draw_relationship_icons(tenant_id: u32, network: &TenantNetwork, text_x: f32, icon_y: f32) {
    use crate::consequences::RelationshipType;
    let relationships: Vec<_> = network
        .relationships
        .iter()
        .filter(|rel| rel.tenant_a_id == tenant_id || rel.tenant_b_id == tenant_id)
        .collect();

    if relationships.is_empty() {
        return;
    }

    // Small colored chips (the bundled UI font can't render emoji glyphs).
    let mut icon_x = text_x;
    let chip_h = 16.0;
    for rel in relationships.iter().take(4) {
        let (label, fill) = match rel.relationship_type {
            RelationshipType::Friendly => ("Friend", colors::POSITIVE()),
            RelationshipType::Hostile => ("Feud", colors::NEGATIVE()),
            RelationshipType::Romantic => ("Romance", colors::ARTIST()),
            RelationshipType::Family => ("Family", colors::FAMILY()),
            RelationshipType::Neutral => ("Neutral", colors::TEXT_DIM()),
        };
        let w = crate::ui::widgets::draw_badge(
            icon_x,
            icon_y,
            chip_h,
            label,
            fill,
            colors::TEXT_BRIGHT(),
        );
        icon_x += w + 4.0;
    }

    if relationships.len() > 4 {
        draw_ui_text(
            "+",
            icon_x + 2.0,
            icon_y + chip_h - 3.0,
            13.0,
            colors::TEXT_DIM(),
        );
    }
}

/// A resident's open request is a decision, so it follows the repair.
pub(super) fn draw_pending_request(
    flow: &mut PanelFlow,
    tenant: &Tenant,
    stories: &HashMap<u32, TenantStory>,
) -> Option<UiAction> {
    let request = stories.get(&tenant.id)?.pending_request.as_ref()?;
    flow.divider();
    flow.section_label("PENDING REQUEST");
    flow.paragraph(&request_text(request), scale::BODY, colors::TEXT());
    flow.paragraph(
        &request_effect_text(request),
        scale::LABEL,
        colors::ACCENT(),
    );
    match flow.button_row(&[
        ("Approve", true, Tone::Positive),
        ("Deny", true, Tone::Danger),
    ])? {
        0 => Some(UiAction::ApproveRequest {
            tenant_id: tenant.id,
        }),
        _ => Some(UiAction::DenyRequest {
            tenant_id: tenant.id,
        }),
    }
}

fn request_text(request: &TenantRequest) -> String {
    match request {
        TenantRequest::Pet { pet_type } => format!("Can I keep a {}?", pet_type),
        TenantRequest::TemporaryGuest {
            guest_name,
            duration_months,
        } => format!("Can {} stay for {} months?", guest_name, duration_months),
        TenantRequest::HomeBusiness { business_type } => {
            format!("Can I start a {} business?", business_type)
        }
        TenantRequest::Modification { description } => format!("Can I {}?", description),
        TenantRequest::Sublease => "Can I sublease a room?".to_string(),
    }
}

fn request_effect_text(request: &TenantRequest) -> String {
    let effect = request.approval_effect();
    let mut effect_text = String::new();
    let mut stack = vec![effect];

    while let Some(effect) = stack.pop() {
        match effect {
            crate::narrative::StoryImpact::Happiness(amount) => {
                append_effect_text(&mut effect_text, &format!("Happiness {:+}", amount));
            }
            crate::narrative::StoryImpact::SetApartmentFlag(flag) => {
                if flag == "high_noise" {
                    append_effect_text(&mut effect_text, "Noise Increases");
                } else {
                    append_effect_text(&mut effect_text, &format!("Flag: {}", flag));
                }
            }
            crate::narrative::StoryImpact::Multiple(list) => {
                for item in list.iter().rev() {
                    stack.push(item.clone());
                }
            }
            _ => {}
        }
    }

    let denial = match request {
        TenantRequest::Pet { .. } => "Deny: Happiness -10",
        TenantRequest::TemporaryGuest { .. } => "Deny: Happiness -5",
        TenantRequest::HomeBusiness { .. } => "Deny: Happiness -8",
        TenantRequest::Modification { .. } => "Deny: Happiness -5",
        TenantRequest::Sublease => "Deny: 30% move-out risk",
    };
    format!("Approve: {} · {}", effect_text, denial)
}

fn append_effect_text(effect_text: &mut String, value: &str) {
    if !effect_text.is_empty() {
        effect_text.push_str(", ");
    }
    effect_text.push_str(value);
}

/// Listing, pricing and applications for an empty unit.
pub(super) fn draw_vacancy_actions(flow: &mut PanelFlow, apt: &Apartment) -> Option<UiAction> {
    if apt.is_listed_for_lease {
        let target = apt
            .preferred_archetype
            .as_ref()
            .map_or("any tenant", TenantArchetype::name);
        flow.kv("Looking for", target, colors::TEXT());
        if flow.button("View applications", true, Tone::Primary) {
            return Some(UiAction::SelectApplications(Some(apt.id)));
        }
        if flow.button("Take off the market", true, Tone::Secondary) {
            return Some(UiAction::UnlistApartment {
                apartment_id: apt.id,
            });
        }
        return None;
    }

    if flow.button("List for any tenant", true, Tone::Primary) {
        return Some(UiAction::ListApartment {
            apartment_id: apt.id,
            preference: None,
        });
    }
    match flow.button_row(&[
        ("Rent -$50", true, Tone::Secondary),
        ("Rent +$50", true, Tone::Secondary),
    ]) {
        Some(0) => {
            return Some(UiAction::SetRent {
                apartment_id: apt.id,
                new_rent: apt.rent_price - 50,
            })
        }
        Some(_) => {
            return Some(UiAction::SetRent {
                apartment_id: apt.id,
                new_rent: apt.rent_price + 50,
            })
        }
        None => {}
    }
    flow.section_label("OR LIST FOR ONE KIND OF TENANT");
    let rows: [&[(TenantArchetype, &str)]; 2] = [
        &[
            (TenantArchetype::Student, "Student"),
            (TenantArchetype::Professional, "Pro"),
            (TenantArchetype::Artist, "Artist"),
        ],
        &[
            (TenantArchetype::Family, "Family"),
            (TenantArchetype::Elderly, "Elderly"),
        ],
    ];
    for row in rows {
        let labels: Vec<_> = row
            .iter()
            .map(|(_, label)| (*label, true, Tone::Secondary))
            .collect();
        if let Some(index) = flow.button_row(&labels) {
            return Some(UiAction::ListApartment {
                apartment_id: apt.id,
                preference: Some(row[index].0.clone()),
            });
        }
    }
    None
}

use crate::assets::AssetManager;
use crate::building::Apartment;
use crate::consequences::TenantNetwork;
use crate::narrative::{TenantRequest, TenantStory};
use crate::tenant::Tenant;
use macroquad::prelude::*;
use std::collections::HashMap;

use super::theme::scale;
use super::{common::*, UiAction};
use macroquad_toolkit::ui::{draw_ui_text, wrap_text_ex};

pub(super) fn draw_tenant_info(
    apt: &Apartment,
    tenants: &[Tenant],
    assets: &AssetManager,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
    network: &TenantNetwork,
    stories: &HashMap<u32, TenantStory>,
) -> Option<UiAction> {
    if *y >= content_top && *y <= content_bottom {
        draw_line(
            content_x,
            *y,
            content_x + panel_w - 30.0,
            *y,
            1.0,
            colors::TEXT_DIM(),
        );
    }
    *y += 15.0;

    if let Some(tenant_id) = apt.tenant_id {
        return draw_occupied_tenant_info(
            tenant_id,
            tenants,
            assets,
            content_x,
            y,
            panel_w,
            content_top,
            content_bottom,
            network,
            stories,
        );
    }

    draw_vacant_unit_actions(apt, content_x, y, panel_w, content_top, content_bottom)
}

fn draw_occupied_tenant_info(
    tenant_id: u32,
    tenants: &[Tenant],
    assets: &AssetManager,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
    network: &TenantNetwork,
    stories: &HashMap<u32, TenantStory>,
) -> Option<UiAction> {
    let tenant = tenants.iter().find(|t| t.id == tenant_id)?;
    let w = panel_w - 30.0;

    if *y >= content_top && *y + 22.0 <= content_bottom {
        crate::ui::widgets::section_label(content_x, *y, "TENANT");
    }
    *y += 22.0;

    // Portrait + name/archetype/relationship block — a fixed 84px-tall row so
    // nothing that follows can draw on top of it.
    let row_top = *y;
    if row_top >= content_top && row_top + 84.0 <= content_bottom {
        let has_portrait = super::resident_sprite::draw_face_portrait(
            tenant,
            Rect::new(content_x, row_top, 80.0, 80.0),
            assets,
        );
        if !has_portrait {
            draw_rectangle(
                content_x,
                row_top,
                4.0,
                60.0,
                archetype_color(&tenant.archetype),
            );
        }
        let text_x = if has_portrait {
            content_x + 92.0
        } else {
            content_x + 12.0
        };
        draw_ui_text(
            &tenant.name,
            text_x,
            row_top + 20.0,
            scale::TITLE,
            colors::TEXT(),
        );
        draw_ui_text(
            tenant.archetype.name(),
            text_x,
            row_top + 40.0,
            scale::LABEL,
            colors::TEXT_DIM(),
        );
        draw_relationship_icons(tenant.id, network, text_x, row_top + 50.0);
    }
    *y += 88.0;

    draw_tenant_happiness(
        tenant,
        assets,
        content_x,
        y,
        panel_w,
        content_top,
        content_bottom,
    );

    if *y >= content_top && *y + 20.0 <= content_bottom {
        crate::ui::widgets::kv_row(
            content_x,
            *y,
            w,
            "Tenure",
            &format!("{} months", tenant.months_residing),
            colors::TEXT_DIM(),
        );
    }
    *y += 26.0;

    // Pending request as its own section, below the tenant info.
    draw_pending_request(
        tenant,
        stories,
        content_x,
        y,
        panel_w,
        content_top,
        content_bottom,
    )
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

fn draw_pending_request(
    tenant: &Tenant,
    stories: &HashMap<u32, TenantStory>,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
) -> Option<UiAction> {
    let story = stories.get(&tenant.id)?;
    let request = story.pending_request.as_ref()?;
    let w = panel_w - 30.0;

    // Divider + section header.
    if *y >= content_top && *y <= content_bottom {
        draw_line(content_x, *y, content_x + w, *y, 1.0, colors::BORDER());
    }
    *y += 14.0;
    if *y >= content_top && *y + 18.0 <= content_bottom {
        crate::ui::widgets::section_label(content_x, *y, "PENDING REQUEST");
    }
    *y += 22.0;

    // Wrapped request text.
    let req_text = request_text(request);
    for line in wrap_text_ex(&req_text, w, None, scale::BODY) {
        if *y >= content_top && *y + scale::BODY <= content_bottom {
            draw_ui_text(
                &line,
                content_x,
                *y + scale::BODY,
                scale::BODY,
                colors::TEXT(),
            );
        }
        *y += scale::BODY * 1.35;
    }
    *y += 4.0;

    let effect_text = approval_effect_text(request);
    if !effect_text.is_empty() {
        if *y >= content_top && *y + 16.0 <= content_bottom {
            draw_ui_text(
                &format!("Effect: {}", effect_text),
                content_x,
                *y + scale::LABEL,
                scale::LABEL,
                colors::ACCENT(),
            );
        }
        *y += 24.0;
    }

    *y += 6.0;
    let btn_w = ((w - 10.0) / 2.0).min(140.0);
    if crate::ui::widgets::button_at(
        Rect::new(content_x, *y, btn_w, 40.0),
        "Approve",
        true,
        crate::ui::theme::Tone::Positive,
    ) {
        return Some(UiAction::ApproveRequest {
            tenant_id: tenant.id,
        });
    }
    if crate::ui::widgets::button_at(
        Rect::new(content_x + btn_w + 10.0, *y, btn_w, 40.0),
        "Deny",
        true,
        crate::ui::theme::Tone::Danger,
    ) {
        return Some(UiAction::DenyRequest {
            tenant_id: tenant.id,
        });
    }

    *y += 38.0;
    None
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

fn approval_effect_text(request: &TenantRequest) -> String {
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

    effect_text
}

fn append_effect_text(effect_text: &mut String, value: &str) {
    if !effect_text.is_empty() {
        effect_text.push_str(", ");
    }
    effect_text.push_str(value);
}

fn draw_tenant_happiness(
    tenant: &Tenant,
    _assets: &AssetManager,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
) {
    let w = panel_w - 30.0;
    if *y >= content_top && *y + 22.0 <= content_bottom {
        crate::ui::widgets::section_label(content_x, *y, "HAPPINESS");
    }
    *y += 22.0;
    if *y >= content_top && *y + 20.0 <= content_bottom {
        crate::ui::widgets::stat_meter(
            content_x,
            *y,
            w,
            tenant.happiness,
            100,
            happiness_color(tenant.happiness),
        );
    }
    *y += 28.0;
}

fn draw_vacant_unit_actions(
    apt: &Apartment,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
) -> Option<UiAction> {
    if *y >= content_top && *y + 18.0 <= content_bottom {
        draw_ui_text("VACANT", content_x, *y, 18.0, colors::WARNING());
    }
    *y += 25.0;

    let btn_w = panel_w - 30.0;

    if apt.is_listed_for_lease {
        return draw_listed_vacancy_actions(apt, content_x, y, btn_w, content_top, content_bottom);
    }

    draw_unlisted_vacancy_actions(apt, content_x, y, btn_w, content_top, content_bottom)
}

fn draw_listed_vacancy_actions(
    apt: &Apartment,
    content_x: f32,
    y: &mut f32,
    btn_w: f32,
    content_top: f32,
    content_bottom: f32,
) -> Option<UiAction> {
    if *y >= content_top && *y + 16.0 <= content_bottom {
        draw_ui_text("Status: LISTED", content_x, *y, 16.0, colors::POSITIVE());
    }
    *y += 20.0;

    if *y >= content_top && *y + 14.0 <= content_bottom {
        let target_text = if let Some(pref) = &apt.preferred_archetype {
            format!("Target: {}", pref.name())
        } else {
            "Target: Open (Any)".to_string()
        };
        draw_ui_text(&target_text, content_x, *y, 14.0, colors::TEXT());
    }
    *y += 30.0;

    if *y >= content_top
        && *y + 40.0 <= content_bottom
        && button(content_x, *y, btn_w, 40.0, "View Applications", true)
    {
        return Some(UiAction::SelectApplications(Some(apt.id)));
    }
    *y += 46.0;

    if *y >= content_top
        && *y + 40.0 <= content_bottom
        && button(content_x, *y, btn_w, 40.0, "Unlist Property", true)
    {
        return Some(UiAction::UnlistApartment {
            apartment_id: apt.id,
        });
    }
    *y += 46.0;

    None
}

fn draw_unlisted_vacancy_actions(
    apt: &Apartment,
    content_x: f32,
    y: &mut f32,
    btn_w: f32,
    content_top: f32,
    content_bottom: f32,
) -> Option<UiAction> {
    if *y >= content_top && *y + 14.0 <= content_bottom {
        draw_ui_text(
            "Status: OFF MARKET",
            content_x,
            *y,
            14.0,
            colors::TEXT_DIM(),
        );
    }
    *y += 30.0;

    if *y >= content_top && *y + 20.0 <= content_bottom {
        draw_ui_text(
            &format!("Rent: ${}", apt.rent_price),
            content_x,
            *y,
            20.0,
            colors::TEXT(),
        );

        let btn_size = 40.0;
        let controls_x = content_x + btn_w - btn_size * 2.0 - 8.0;
        if button(controls_x, *y - 25.0, btn_size, btn_size, "-", true) {
            return Some(UiAction::SetRent {
                apartment_id: apt.id,
                new_rent: apt.rent_price - 50,
            });
        }
        if button(
            controls_x + btn_size + 8.0,
            *y - 25.0,
            btn_size,
            btn_size,
            "+",
            true,
        ) {
            return Some(UiAction::SetRent {
                apartment_id: apt.id,
                new_rent: apt.rent_price + 50,
            });
        }
    }
    *y += 48.0;

    if *y >= content_top && *y + 14.0 <= content_bottom {
        draw_ui_text("List for Lease:", content_x, *y, 14.0, colors::ACCENT());
    }
    *y += 20.0;

    if *y >= content_top
        && *y + 40.0 <= content_bottom
        && button(content_x, *y, btn_w, 40.0, "Any Tenant", true)
    {
        return Some(UiAction::ListApartment {
            apartment_id: apt.id,
            preference: None,
        });
    }
    *y += 46.0;

    let tenant_types = [
        (crate::tenant::TenantArchetype::Student, "Student"),
        (crate::tenant::TenantArchetype::Professional, "Pro"),
        (crate::tenant::TenantArchetype::Artist, "Artist"),
        (crate::tenant::TenantArchetype::Family, "Family"),
        (crate::tenant::TenantArchetype::Elderly, "Elderly"),
    ];
    let small_btn_w = (btn_w - 10.0) / 2.0;

    for (index, (archetype, label)) in tenant_types.iter().enumerate() {
        let col = index % 2;
        let x = content_x + col as f32 * (small_btn_w + 10.0);

        if *y >= content_top
            && *y + 40.0 <= content_bottom
            && button(x, *y, small_btn_w, 40.0, label, true)
        {
            return Some(UiAction::ListApartment {
                apartment_id: apt.id,
                preference: Some(archetype.clone()),
            });
        }

        if col == 1 || index == tenant_types.len() - 1 {
            *y += 46.0;
        }
    }
    *y += 10.0;

    None
}

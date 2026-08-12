//! Mail and conversation workspace.

use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

use crate::narrative::MailType;
use crate::state::GameplayState;

use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, draw_card, draw_panel, line_height, wrap};
use super::UiAction;

fn content_rect() -> Rect {
    Rect::new(
        space::LG,
        crate::ui::layout::HEADER_HEIGHT() + space::LG,
        screen_width() - space::LG * 2.0,
        screen_height()
            - crate::ui::layout::HEADER_HEIGHT()
            - crate::ui::layout::FOOTER_HEIGHT()
            - space::LG * 2.0,
    )
}

pub fn draw_inbox_view(state: &GameplayState) -> Option<UiAction> {
    let rect = content_rect();
    let pending_dialogues = state.dialogue_system.pending_dialogues();
    let body_y = draw_title(
        rect,
        &format!(
            "{} unread letters · {} conversations waiting",
            state.mailbox.unread_count(),
            pending_dialogues.len()
        ),
    );
    let gap = space::LG;
    let list_w = (rect.w * 0.34).clamp(260.0, 400.0);
    let list_rect = Rect::new(rect.x, body_y, list_w, rect.bottom() - body_y);
    let detail_rect = Rect::new(
        list_rect.right() + gap,
        body_y,
        rect.right() - list_rect.right() - gap,
        list_rect.h,
    );
    let list = draw_panel(list_rect, "Letters");
    let selected_id = state
        .selected_mail_id
        .or_else(|| state.mailbox.items.last().map(|item| item.id));
    if let Some(action) = draw_mail_list(state, list, selected_id) {
        return Some(action);
    }
    let detail = draw_panel(detail_rect, "Reading room");
    let dialogue_h = if pending_dialogues.is_empty() {
        0.0
    } else {
        (detail.h * 0.44).max(150.0)
    };
    let mail_h = detail.h - dialogue_h - if dialogue_h > 0.0 { space::LG } else { 0.0 };
    draw_selected_mail(state, detail, selected_id, mail_h);
    if let Some(dialogue) = pending_dialogues.first() {
        return draw_dialogue(detail, mail_h, dialogue);
    }
    None
}

fn draw_title(rect: Rect, subtitle: &str) -> f32 {
    draw_ui_text(
        "Inbox",
        rect.x,
        rect.y + scale::TITLE,
        scale::TITLE,
        color::TEXT_BRIGHT(),
    );
    draw_ui_text(
        subtitle,
        rect.x,
        rect.y + scale::TITLE + line_height(scale::BODY),
        scale::BODY,
        color::TEXT_DIM(),
    );
    rect.y + scale::TITLE + line_height(scale::BODY) + space::LG
}

fn draw_mail_list(state: &GameplayState, list: Rect, selected_id: Option<u32>) -> Option<UiAction> {
    let mut y = list.y;
    let page_size = (((list.h - 38.0) / 53.0).floor() as usize).max(1);
    let page_count = state.mailbox.items.len().max(1).div_ceil(page_size);
    let page = state.inbox_page.min(page_count - 1);
    for item in state
        .mailbox
        .recent(state.mailbox.items.len())
        .into_iter()
        .skip(page * page_size)
        .take(page_size)
    {
        let row = Rect::new(list.x, y, list.w, 48.0);
        draw_card(row, selected_id == Some(item.id));
        let marker = if item.read { "" } else { "• " };
        let subject = truncate_text_to_width(
            &format!(
                "{}{} · {}",
                marker,
                mail_type_label(&item.mail_type),
                item.subject
            ),
            row.w - space::MD * 2.0,
            scale::LABEL,
        );
        draw_ui_text(
            &subject,
            row.x + space::MD,
            row.y + 19.0,
            scale::LABEL,
            if item.read {
                color::TEXT_DIM()
            } else {
                color::TEXT_BRIGHT()
            },
        );
        draw_ui_text(
            &truncate_text_to_width(
                &format!("{} · Month {}", item.sender, item.month_received),
                row.w - space::MD * 2.0,
                scale::CAPTION,
            ),
            row.x + space::MD,
            row.y + 38.0,
            scale::CAPTION,
            color::TEXT_DIM(),
        );
        if row.contains(Vec2::from(mouse_position())) && is_mouse_button_released(MouseButton::Left)
        {
            return Some(UiAction::OpenMailItem { mail_id: item.id });
        }
        y += 53.0;
    }
    draw_pager(list, page, page_count)
}

fn draw_pager(list: Rect, page: usize, page_count: usize) -> Option<UiAction> {
    let pager_y = list.bottom() - 42.0;
    let pager_w = (list.w - 92.0 - space::SM * 2.0) / 2.0;
    if button_at(
        Rect::new(list.x, pager_y, pager_w, 40.0),
        "Prev",
        page > 0,
        Tone::Secondary,
    ) {
        return Some(UiAction::SetInboxPage { page: page - 1 });
    }
    draw_ui_text(
        &format!("{}/{}", page + 1, page_count),
        list.x + pager_w + space::SM + 28.0,
        pager_y + 20.0,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    if button_at(
        Rect::new(list.right() - pager_w, pager_y, pager_w, 40.0),
        "Next",
        page + 1 < page_count,
        Tone::Secondary,
    ) {
        return Some(UiAction::SetInboxPage { page: page + 1 });
    }
    None
}

fn draw_selected_mail(state: &GameplayState, detail: Rect, selected_id: Option<u32>, mail_h: f32) {
    let Some(item) =
        selected_id.and_then(|id| state.mailbox.items.iter().find(|item| item.id == id))
    else {
        draw_ui_text(
            "No mail yet.",
            detail.x,
            detail.y + scale::BODY,
            scale::BODY,
            color::TEXT_DIM(),
        );
        return;
    };
    draw_ui_text(
        &truncate_text_to_width(&item.subject, detail.w, scale::HEADING),
        detail.x,
        detail.y + scale::HEADING,
        scale::HEADING,
        color::TEXT_BRIGHT(),
    );
    draw_ui_text(
        &format!("From {} · Month {}", item.sender, item.month_received),
        detail.x,
        detail.y + scale::HEADING + line_height(scale::LABEL),
        scale::LABEL,
        color::TEXT_DIM(),
    );
    let mut y = detail.y + line_height(scale::HEADING) + line_height(scale::LABEL) + space::SM;
    for line in wrap(&item.body, detail.w, scale::BODY)
        .iter()
        .take(((mail_h - 58.0) / line_height(scale::BODY)).max(1.0) as usize)
    {
        draw_ui_text(line, detail.x, y + scale::BODY, scale::BODY, color::TEXT());
        y += line_height(scale::BODY);
    }
}

fn draw_dialogue(
    detail: Rect,
    mail_h: f32,
    dialogue: &crate::narrative::ActiveDialogue,
) -> Option<UiAction> {
    let y = detail.y + mail_h + space::LG;
    draw_ui_text(
        "CONVERSATION NEEDS A RESPONSE",
        detail.x,
        y + scale::LABEL,
        scale::LABEL,
        color::WARNING(),
    );
    draw_ui_text(
        &truncate_text_to_width(&dialogue.headline, detail.w, scale::BODY),
        detail.x,
        y + line_height(scale::LABEL) + scale::BODY,
        scale::BODY,
        color::TEXT_BRIGHT(),
    );
    let desc_y = y + line_height(scale::LABEL) + line_height(scale::BODY);
    draw_ui_text(
        &truncate_text_to_width(&dialogue.description, detail.w, scale::LABEL),
        detail.x,
        desc_y + scale::LABEL,
        scale::LABEL,
        color::TEXT_DIM(),
    );
    let button_y = desc_y + line_height(scale::LABEL) + space::SM;
    let count = dialogue.choices.len().max(1);
    let button_w = (detail.w - space::SM * (count.saturating_sub(1)) as f32) / count as f32;
    for (index, choice) in dialogue.choices.iter().enumerate() {
        if button_at(
            Rect::new(
                detail.x + index as f32 * (button_w + space::SM),
                button_y,
                button_w,
                40.0,
            ),
            &truncate_text_to_width(&choice.text, button_w - space::MD, scale::LABEL),
            true,
            if index == 0 {
                Tone::Primary
            } else {
                Tone::Secondary
            },
        ) {
            return Some(UiAction::ResolveDialogue {
                dialogue_id: dialogue.id,
                choice_index: index,
            });
        }
    }
    None
}

fn mail_type_label(mail_type: &MailType) -> &'static str {
    match mail_type {
        MailType::TenantLetter { .. } => "TENANT",
        MailType::CityNotice => "CITY",
        MailType::Financial => "FINANCE",
        MailType::Advertisement => "OFFER",
        MailType::News => "NEWS",
        MailType::Personal => "PERSONAL",
        MailType::Official => "OFFICIAL",
    }
}

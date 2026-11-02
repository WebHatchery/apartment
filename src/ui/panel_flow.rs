//! Scrolling layout cursor for inspector panels. Every row is drawn and made
//! clickable only when it fits wholly inside the visible content band, so a
//! row clipped by the header or scroll strip can never receive input.

use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width};

use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, kv_row, line_height, section_label, stat_meter, wrap};

/// Height of a full-width action button and the gap after it.
pub(crate) const BUTTON_H: f32 = 44.0;
const BUTTON_GAP: f32 = 6.0;
/// Space under the content band for the fixed scroll controls.
const SCROLL_STRIP: f32 = 54.0;

pub(crate) struct PanelFlow {
    /// Left edge of the content column.
    pub x: f32,
    /// Top of the next row in screen space.
    pub y: f32,
    /// Content column width.
    pub w: f32,
    top: f32,
    bottom: f32,
    origin: f32,
}

impl PanelFlow {
    pub fn new(panel: Rect, scroll: f32) -> Self {
        let origin = panel.y + 50.0;
        Self {
            x: panel.x + 15.0,
            y: origin - scroll,
            w: panel.w - 30.0,
            top: panel.y + 40.0,
            bottom: panel.bottom() - SCROLL_STRIP,
            origin,
        }
    }

    /// Whether the visible content band is tall enough for decorative rows
    /// (meters that repeat a value already shown as text) without pushing the
    /// current decision below the fold.
    pub fn roomy(&self) -> bool {
        self.bottom - self.top >= 300.0
    }

    /// Whether a row of height `h` at the cursor is wholly visible.
    pub fn fits(&self, h: f32) -> bool {
        self.y >= self.top && self.y + h <= self.bottom
    }

    pub fn advance(&mut self, h: f32) {
        self.y += h;
    }

    pub fn gap(&mut self) {
        self.y += space::SM;
    }

    pub fn section_label(&mut self, text: &str) {
        let h = line_height(scale::LABEL) + space::XS;
        if self.fits(h) {
            section_label(self.x, self.y, text);
        }
        self.y += h + space::XS;
    }

    pub fn divider(&mut self) {
        if self.fits(1.0) {
            draw_line(
                self.x,
                self.y,
                self.x + self.w,
                self.y,
                1.0,
                color::BORDER(),
            );
        }
        self.y += space::MD;
    }

    pub fn kv(&mut self, key: &str, value: &str, value_color: Color) {
        let h = line_height(scale::BODY) + 3.0;
        if self.fits(h) {
            kv_row(self.x, self.y, self.w, key, value, value_color);
        }
        self.y += h + space::XS;
    }

    pub fn meter(&mut self, value: i32, max: i32, fill: Color) {
        let h = 16.0 + space::SM;
        if self.fits(h) {
            stat_meter(self.x, self.y, self.w, value, max, fill);
        }
        self.y += h + space::XS;
    }

    /// One line of text, truncated to the column.
    pub fn line(&mut self, text: &str, size: f32, text_color: Color) {
        let h = line_height(size);
        if self.fits(h) {
            let text = truncate_text_to_width(text, self.w, size);
            draw_ui_text(&text, self.x, self.y + size, size, text_color);
        }
        self.y += h;
    }

    /// Text wrapped to the column so no decision text is cut off.
    pub fn paragraph(&mut self, text: &str, size: f32, text_color: Color) {
        for line in wrap(text, self.w, size) {
            self.line(&line, size, text_color);
        }
        self.y += space::XS;
    }

    /// A full-width action. Returns true on release inside the button.
    pub fn button(&mut self, label: &str, enabled: bool, tone: Tone) -> bool {
        let clicked = self.fits(BUTTON_H)
            && button_at(
                Rect::new(self.x, self.y, self.w, BUTTON_H),
                label,
                enabled,
                tone,
            );
        self.y += BUTTON_H + BUTTON_GAP;
        clicked
    }

    /// Equal-width buttons in one row; returns the index released.
    pub fn button_row(&mut self, labels: &[(&str, bool, Tone)]) -> Option<usize> {
        let count = labels.len().max(1) as f32;
        let w = (self.w - space::SM * (count - 1.0)) / count;
        let mut clicked = None;
        if self.fits(BUTTON_H) {
            for (index, (label, enabled, tone)) in labels.iter().enumerate() {
                let rect = Rect::new(self.x + index as f32 * (w + space::SM), self.y, w, BUTTON_H);
                if button_at(rect, label, *enabled, *tone) {
                    clicked = Some(index);
                }
            }
        }
        self.y += BUTTON_H + BUTTON_GAP;
        clicked
    }

    /// A section header that opens or closes the rows below it. Returns
    /// true when the player toggled it.
    pub fn disclosure(&mut self, title: &str, summary: &str, open: bool) -> bool {
        let state = if open { "Hide" } else { "Show" };
        let rect = Rect::new(self.x, self.y, self.w, BUTTON_H);
        let clicked = self.fits(BUTTON_H) && button_at(rect, "", true, Tone::Secondary);
        if self.fits(BUTTON_H) {
            let baseline = rect.y + rect.h * 0.5 + scale::LABEL * 0.5 - 1.0;
            let state_w = measure_ui_text(state, None, scale::LABEL as u16, 1.0).width;
            let label = if summary.is_empty() {
                title.to_string()
            } else {
                format!("{title} · {summary}")
            };
            let label =
                truncate_text_to_width(&label, rect.w - state_w - space::MD * 3.0, scale::LABEL);
            draw_ui_text(
                &label,
                rect.x + space::MD,
                baseline,
                scale::LABEL,
                color::TEXT(),
            );
            draw_ui_text(
                state,
                rect.right() - state_w - space::MD,
                baseline,
                scale::LABEL,
                color::TEXT_DIM(),
            );
        }
        self.y += BUTTON_H + BUTTON_GAP;
        clicked
    }

    /// Largest useful scroll for the rows laid out so far.
    pub fn max_scroll(&self, scroll: f32) -> f32 {
        let content_height = self.y + scroll - self.origin;
        (content_height - (self.bottom - self.origin)).max(0.0)
    }
}

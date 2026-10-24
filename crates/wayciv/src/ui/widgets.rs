//! Labelled buttons for the HUD: action pills (icon, name, hotkey) that can
//! be primary or disabled with a reason, and small round tool buttons.

use super::glyph::{self, Glyph};
use super::theme;
use crate::palette::{BAD, GOLD, GOLD_DIM, NAVY_HOVER, NAVY_RAISED, TEAL_DIM, TEXT, TEXT_DIM};
use egui::{Color32, FontId, Painter, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, pos2, vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The most useful thing to do right now: green, gold-trimmed, larger.
    Primary,
    Normal,
    /// Destructive (Disband): red trim.
    Danger,
}

pub struct Btn<'a> {
    pub glyph: Option<Glyph>,
    pub label: &'a str,
    pub key: Option<&'a str>,
    pub color: Color32,
    pub kind: Kind,
    /// What it does; the hotkey is appended automatically.
    pub tip: String,
    /// Why it can't be used right now.
    pub disabled: Option<String>,
}

impl<'a> Btn<'a> {
    pub fn new(label: &'a str) -> Self {
        Btn {
            glyph: None,
            label,
            key: None,
            color: TEXT,
            kind: Kind::Normal,
            tip: String::new(),
            disabled: None,
        }
    }
    pub fn glyph(mut self, g: Glyph, color: Color32) -> Self {
        self.glyph = Some(g);
        self.color = color;
        self
    }
    pub fn key(mut self, key: Option<&'a str>) -> Self {
        self.key = key;
        self
    }
    pub fn kind(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self
    }
    pub fn tip(mut self, tip: impl Into<String>) -> Self {
        self.tip = tip.into();
        self
    }
    pub fn disabled(mut self, reason: Option<String>) -> Self {
        self.disabled = reason;
        self
    }
}

/// Tooltip text: what it does, the hotkey, and why it's unavailable.
pub fn tooltip(name: &str, tip: &str, key: Option<&str>, disabled: Option<&str>) -> String {
    let mut out = match key {
        Some(k) => format!("{name} ({k})"),
        None => name.to_string(),
    };
    if !tip.is_empty() {
        out.push('\n');
        out.push_str(tip);
    }
    if let Some(why) = disabled {
        out.push_str("\n\nNot available: ");
        out.push_str(why);
    }
    out
}

/// Width a pill needs.
pub fn pill_width(painter: &Painter, b: &Btn) -> f32 {
    let size = if b.kind == Kind::Primary { 17.0 } else { 15.0 };
    let galley = painter.layout_no_wrap(b.label.to_string(), FontId::proportional(size), TEXT);
    let mut w = galley.size().x + 26.0;
    if b.glyph.is_some() {
        w += 28.0;
    }
    if let Some(k) = b.key {
        w += keycap_width(painter, k) + 8.0;
    }
    w
}

fn keycap_width(painter: &Painter, k: &str) -> f32 {
    let g = painter.layout_no_wrap(k.to_string(), FontId::proportional(11.0), TEXT);
    (g.size().x + 8.0).max(16.0)
}

/// Draws a labelled pill button; true when clicked (never when disabled).
pub fn pill(ui: &mut Ui, b: Btn) -> bool {
    let primary = b.kind == Kind::Primary;
    // Every pill takes a 42 px tall cell so mixed rows line up.
    let h = if primary { 42.0 } else { 36.0 };
    let w = pill_width(ui.painter(), &b);
    let enabled = b.disabled.is_none();
    let (cell, response) = ui.allocate_exact_size(vec2(w, 42.0), Sense::click());
    let rect = Rect::from_center_size(cell.center(), vec2(w, h));
    paint_pill(ui.painter(), rect, &response, &b);
    let tip = tooltip(b.label, &b.tip, b.key, b.disabled.as_deref());
    let response = response.on_hover_text(tip);
    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
    } else {
        false
    }
}

pub fn paint_pill(painter: &Painter, rect: Rect, response: &egui::Response, b: &Btn) {
    let primary = b.kind == Kind::Primary;
    let enabled = b.disabled.is_none();
    let hovered = response.hovered() && enabled;
    let fade = |c: Color32| if enabled { c } else { c.gamma_multiply(0.4) };
    let (fill, rim, width) = match b.kind {
        Kind::Primary => (
            if hovered {
                Color32::from_rgb(44, 150, 102)
            } else {
                Color32::from_rgb(30, 116, 80)
            },
            GOLD,
            2.5,
        ),
        Kind::Danger => (
            if hovered { NAVY_HOVER } else { NAVY_RAISED },
            if hovered { BAD } else { BAD.gamma_multiply(0.6) },
            1.5,
        ),
        Kind::Normal => (
            if hovered { NAVY_HOVER } else { NAVY_RAISED },
            if hovered { GOLD } else { GOLD_DIM },
            1.5,
        ),
    };
    let r = rect.height() / 2.0;
    painter.rect_filled(rect, r, fade(fill));
    painter.rect_stroke(rect, r, Stroke::new(width, fade(rim)), StrokeKind::Inside);
    let mut x = rect.left() + 10.0;
    if let Some(g) = b.glyph {
        let c = pos2(x + 12.0, rect.center().y);
        painter.circle_filled(c, 13.0, Color32::from_black_alpha(if primary { 70 } else { 90 }));
        glyph::paint(painter, g, c, 8.5, fade(b.color));
        x += 28.0;
    }
    let size = if primary { 17.0 } else { 15.0 };
    let label_color = fade(if primary { Color32::WHITE } else { TEXT });
    let rect_text = theme::text_bold(
        painter,
        pos2(x + 2.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        b.label,
        size,
        label_color,
    );
    if let Some(k) = b.key {
        let kw = keycap_width(painter, k);
        theme::keycap(
            painter,
            pos2(rect_text.right() + 8.0 + kw / 2.0, rect.center().y),
            k,
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Plus,
    Minus,
    Globe,
    Target,
    Prev,
    Next,
    Close,
}

/// A small round button with a drawn symbol; true when clicked.
pub fn tool_button(
    ui: &mut Ui,
    d: f32,
    tool: Tool,
    active: bool,
    name: &str,
    tip: &str,
    key: Option<&str>,
) -> bool {
    let (rect, _) = ui.allocate_exact_size(vec2(d, d), Sense::hover());
    tool_button_at(ui, rect, tool, active, name, tip, key)
}

/// Like `tool_button`, at a fixed place.
pub fn tool_button_at(
    ui: &mut Ui,
    rect: Rect,
    tool: Tool,
    active: bool,
    name: &str,
    tip: &str,
    key: Option<&str>,
) -> bool {
    let response = ui.interact(rect, ui.scope_id().with(("tool", name)), Sense::click());
    paint_tool(ui.painter(), rect, &response, tool, active);
    response
        .on_hover_text(tooltip(name, tip, key, None))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

pub fn paint_tool(painter: &Painter, rect: Rect, response: &egui::Response, tool: Tool, active: bool) {
    let c = rect.center();
    let r = rect.width() / 2.0;
    let hovered = response.hovered();
    let fill = if active {
        TEAL_DIM
    } else if hovered {
        NAVY_HOVER
    } else {
        NAVY_RAISED
    };
    theme::disc(painter, c, r - 1.0, fill, if hovered || active { GOLD } else { GOLD_DIM });
    tool_icon(painter, tool, c, r * 0.52, if active { GOLD } else { TEXT });
}

fn tool_icon(painter: &Painter, tool: Tool, c: Pos2, r: f32, color: Color32) {
    let w = (r * 0.28).max(1.6);
    let stroke = Stroke::new(w, color);
    let line = |a: Pos2, b: Pos2| {
        painter.line_segment([a, b], stroke);
    };
    match tool {
        Tool::Close => {
            let d = r * 0.8;
            line(c + vec2(-d, -d), c + vec2(d, d));
            line(c + vec2(-d, d), c + vec2(d, -d));
        }
        Tool::Plus | Tool::Minus => {
            line(c - vec2(r, 0.0), c + vec2(r, 0.0));
            if tool == Tool::Plus {
                line(c - vec2(0.0, r), c + vec2(0.0, r));
            }
        }
        Tool::Globe => {
            painter.circle_stroke(c, r, stroke);
            line(c - vec2(r, 0.0), c + vec2(r, 0.0));
            let pts: Vec<Pos2> = (0..=24)
                .map(|i| {
                    let a = i as f32 / 24.0 * std::f32::consts::TAU;
                    c + vec2(a.cos() * r * 0.45, a.sin() * r)
                })
                .collect();
            painter.add(Shape::line(pts, Stroke::new(w * 0.8, color)));
        }
        Tool::Target => {
            painter.circle_stroke(c, r * 0.62, stroke);
            for d in [vec2(1.0, 0.0), vec2(-1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, -1.0)] {
                line(c + d * r * 0.62, c + d * r * 1.15);
            }
            painter.circle_filled(c, r * 0.18, color);
        }
        Tool::Prev | Tool::Next => {
            let s = if tool == Tool::Prev { -1.0 } else { 1.0 };
            let tip = c + vec2(s * r * 0.55, 0.0);
            line(tip, c + vec2(-s * r * 0.5, -r * 0.9));
            line(tip, c + vec2(-s * r * 0.5, r * 0.9));
        }
    }
}

/// A one-line dim hint, wrapping to `width`.
pub fn hint_label(ui: &mut Ui, text: &str, width: f32) {
    ui.scope(|ui| {
        ui.set_max_width(width);
        ui.label(
            egui::RichText::new(text)
                .italics()
                .size(13.5)
                .color(TEXT_DIM),
        );
    });
}

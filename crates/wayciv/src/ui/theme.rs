//! Shared HUD drawing: gold-trimmed plates, progress rings, round buttons,
//! circular portraits.

use super::glyph::{self, Glyph};
use crate::palette::{self, GOLD, GOLD_DIM, NAVY, NAVY_HOVER, NAVY_RAISED, TEXT};
use egui::{
    Align2, Color32, CornerRadius, FontId, Mesh, Painter, Pos2, Rect, Response, Sense, Shape,
    Stroke, StrokeKind, Ui, epaint::Vertex, pos2, vec2,
};
use std::f32::consts::{PI, TAU};

/// The frame of every floating HUD panel and window: navy with a gold trim.
pub fn plate_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(palette::with_alpha(NAVY, 240))
        .stroke(Stroke::new(2.0, GOLD_DIM))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(12)
        .shadow(egui::Shadow {
            offset: [0, 4],
            blur: 14,
            spread: 0,
            color: Color32::from_black_alpha(120),
        })
}

/// A plate painted into a rect: fill, gold outer trim, thin teal inner line.
pub fn plate(painter: &Painter, rect: Rect, radius: f32) {
    painter.rect_filled(rect, radius, palette::with_alpha(NAVY, 238));
    painter.rect_stroke(rect, radius, Stroke::new(2.0, GOLD_DIM), StrokeKind::Inside);
    painter.rect_stroke(
        rect.shrink(4.0),
        (radius - 3.0).max(0.0),
        Stroke::new(1.0, palette::with_alpha(palette::TEAL, 70)),
        StrokeKind::Inside,
    );
}

/// An arc starting at 12 o'clock, clockwise; `frac` of a full turn.
pub fn arc(painter: &Painter, c: Pos2, r: f32, frac: f32, width: f32, color: Color32) {
    let frac = frac.clamp(0.0, 1.0);
    if frac <= 0.001 {
        return;
    }
    let steps = ((frac * 64.0) as usize).max(3);
    let pts: Vec<Pos2> = (0..=steps)
        .map(|i| {
            let a = -PI / 2.0 + TAU * frac * i as f32 / steps as f32;
            c + vec2(a.cos(), a.sin()) * r
        })
        .collect();
    painter.add(Shape::line(pts, Stroke::new(width, color)));
}

/// A progress ring: a dim track plus the colored arc.
pub fn ring(painter: &Painter, c: Pos2, r: f32, frac: f32, width: f32, color: Color32) {
    painter.circle_stroke(c, r, Stroke::new(width, Color32::from_white_alpha(28)));
    arc(painter, c, r, frac, width, color);
}

/// A navy disc with a gold rim, the base of every round element.
pub fn disc(painter: &Painter, c: Pos2, r: f32, fill: Color32, rim: Color32) {
    painter.circle_filled(c, r, fill);
    painter.circle_stroke(c, r, Stroke::new(2.0, rim));
}

/// Text with a position and alignment; returns its rect.
pub fn text(
    painter: &Painter,
    pos: Pos2,
    align: Align2,
    s: impl ToString,
    size: f32,
    color: Color32,
) -> Rect {
    painter.text(pos, align, s.to_string(), FontId::proportional(size), color)
}

/// Bold-looking text: the default font has no bold face, so draw twice.
pub fn text_bold(
    painter: &Painter,
    pos: Pos2,
    align: Align2,
    s: impl ToString,
    size: f32,
    color: Color32,
) -> Rect {
    let s = s.to_string();
    painter.text(
        pos + vec2(0.6, 0.0),
        align,
        s.clone(),
        FontId::proportional(size),
        color,
    );
    painter.text(pos, align, s, FontId::proportional(size), color)
}

/// A small filled badge with a number, e.g. the unread count.
pub fn badge(painter: &Painter, c: Pos2, label: &str, fill: Color32) {
    let galley = painter.layout_no_wrap(label.to_string(), FontId::proportional(12.0), Color32::WHITE);
    let w = (galley.size().x + 8.0).max(18.0);
    let rect = Rect::from_center_size(c, vec2(w, 18.0));
    painter.rect_filled(rect, 9.0, fill);
    painter.rect_stroke(rect, 9.0, Stroke::new(1.5, NAVY), StrokeKind::Outside);
    painter.galley(c - galley.size() / 2.0, galley, Color32::WHITE);
}

/// A key-cap hint such as `N` or `Space`, drawn centered on `c`.
pub fn keycap(painter: &Painter, c: Pos2, label: &str) {
    let galley = painter.layout_no_wrap(label.to_string(), FontId::proportional(11.0), TEXT);
    let w = (galley.size().x + 8.0).max(16.0);
    let rect = Rect::from_center_size(c, vec2(w, 16.0));
    painter.rect_filled(rect, 4.0, Color32::from_rgb(8, 14, 24));
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0, GOLD_DIM), StrokeKind::Inside);
    painter.galley(c - galley.size() / 2.0, galley, TEXT);
}

/// A circular button holding a glyph. Returns the click response.
pub fn round_button(
    ui: &mut Ui,
    diameter: f32,
    g: Glyph,
    glyph_color: Color32,
    hotkey: Option<&str>,
    enabled: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        vec2(diameter, diameter),
        if enabled { Sense::click() } else { Sense::hover() },
    );
    paint_round_button(ui.painter(), rect, &response, g, glyph_color, hotkey, enabled, false);
    response
}

#[allow(clippy::too_many_arguments)]
pub fn paint_round_button(
    painter: &Painter,
    rect: Rect,
    response: &Response,
    g: Glyph,
    glyph_color: Color32,
    hotkey: Option<&str>,
    enabled: bool,
    active: bool,
) {
    let c = rect.center();
    let r = rect.width() / 2.0;
    let fill = if active {
        palette::TEAL_DIM
    } else if response.hovered() && enabled {
        NAVY_HOVER
    } else {
        NAVY_RAISED
    };
    let rim = if response.hovered() && enabled || active {
        GOLD
    } else {
        GOLD_DIM
    };
    disc(painter, c, r - 1.0, fill, rim);
    let color = if enabled {
        glyph_color
    } else {
        glyph_color.gamma_multiply(0.35)
    };
    glyph::paint(painter, g, c, r * 0.5, color);
    if let Some(k) = hotkey {
        keycap(painter, c + vec2(r * 0.55, r * 0.8), k);
    }
}

/// A pill-shaped button with optional glyph; returns the response.
pub fn pill_button(ui: &mut Ui, g: Option<Glyph>, label: &str, color: Color32, min_width: f32) -> Response {
    let font = FontId::proportional(15.0);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, TEXT);
    let icon = if g.is_some() { 24.0 } else { 0.0 };
    let w = (galley.size().x + icon + 24.0).max(min_width);
    let (rect, response) = ui.allocate_exact_size(vec2(w, 32.0), Sense::click());
    let painter = ui.painter();
    let fill = if response.hovered() { NAVY_HOVER } else { NAVY_RAISED };
    painter.rect_filled(rect, 16.0, fill);
    painter.rect_stroke(
        rect,
        16.0,
        Stroke::new(1.5, if response.hovered() { GOLD } else { GOLD_DIM }),
        StrokeKind::Inside,
    );
    let mut x = rect.left() + 12.0;
    if let Some(g) = g {
        glyph::paint(painter, g, pos2(x + 8.0, rect.center().y), 8.0, color);
        x += 24.0;
    }
    painter.galley(pos2(x, rect.center().y - galley.size().y / 2.0), galley, TEXT);
    response
}

/// Paints a texture clipped to a circle, `uv` selecting the part to show.
pub fn image_circle(painter: &Painter, tex: egui::TextureId, c: Pos2, r: f32, uv: Rect) {
    let mut mesh = Mesh::with_texture(tex);
    let n = 48;
    mesh.vertices.push(Vertex {
        pos: c,
        uv: uv.center(),
        color: Color32::WHITE,
    });
    for i in 0..=n {
        let a = i as f32 / n as f32 * TAU;
        let d = vec2(a.cos(), a.sin());
        mesh.vertices.push(Vertex {
            pos: c + d * r,
            uv: uv.center() + vec2(d.x * uv.width(), d.y * uv.height()) * 0.5,
            color: Color32::WHITE,
        });
    }
    for i in 1..=n as u32 {
        mesh.indices.extend([0, i, i + 1]);
    }
    painter.add(Shape::mesh(mesh));
}

/// A leader's portrait in a circle, falling back to a monogram in the civ's colors.
pub fn portrait_disc(
    painter: &Painter,
    portraits: &mut crate::portraits::Portraits,
    c: Pos2,
    r: f32,
    leader_id: &str,
    name: &str,
    colors: ([u8; 3], [u8; 3]),
) {
    let (primary, secondary) = (palette::rgb(colors.0), palette::rgb(colors.1));
    match portraits.get(painter.ctx(), leader_id) {
        Some(tex) => {
            painter.circle_filled(c, r, NAVY);
            image_circle(
                painter,
                tex.id(),
                c,
                r,
                Rect::from_min_max(pos2(0.08, 0.0), pos2(0.92, 0.84)),
            );
        }
        None => {
            painter.circle_filled(c, r, primary);
            let initials: String = name
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .take(2)
                .collect();
            text_bold(painter, c, Align2::CENTER_CENTER, initials, r * 0.9, secondary);
        }
    }
}

/// A horizontal bar: dim track and a colored fill.
pub fn bar(painter: &Painter, rect: Rect, frac: f32, color: Color32) {
    painter.rect_filled(rect, rect.height() / 2.0, Color32::from_white_alpha(24));
    let fill = Rect::from_min_size(
        rect.min,
        vec2((rect.width() * frac.clamp(0.0, 1.0)).max(rect.height()), rect.height()),
    );
    if frac > 0.0 {
        painter.rect_filled(fill, rect.height() / 2.0, color);
    }
}

/// A gold section heading with a thin rule.
pub fn heading(ui: &mut Ui, s: &str) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new(s).size(16.0).strong().color(GOLD));
    let y = ui.cursor().top();
    ui.painter().line_segment(
        [pos2(ui.min_rect().left(), y), pos2(ui.max_rect().right(), y)],
        Stroke::new(1.0, palette::with_alpha(GOLD_DIM, 160)),
    );
    ui.add_space(4.0);
}

/// The yield glyph for a yield index (food, production, gold, science,
/// culture, faith).
pub fn yield_glyph(i: usize) -> Glyph {
    [
        Glyph::Food,
        Glyph::Production,
        Glyph::Gold,
        Glyph::Science,
        Glyph::Culture,
        Glyph::Faith,
    ][i]
}

/// A small colored label such as "At war".
pub fn tag(ui: &mut Ui, label: &str, color: Color32) {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), FontId::proportional(15.0), TEXT);
    let (rect, _) = ui.allocate_exact_size(vec2(galley.size().x + 24.0, 26.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 13.0, color.gamma_multiply(0.3));
    painter.rect_stroke(rect, 13.0, Stroke::new(1.5, color), StrokeKind::Inside);
    painter.galley(rect.center() - galley.size() / 2.0, galley, TEXT);
}

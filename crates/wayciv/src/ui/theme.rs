//! Shared HUD drawing: gold-trimmed plates, progress rings, round buttons,
//! circular portraits, ornamental rules.
//!
//! The look: deep navy panels lit from the upper left (a vertical gradient
//! and a soft sheen), a beveled gold trim with studs on the corners, and a
//! soft drop shadow. Everything is built from vector shapes so it stays
//! crisp at any window size.

use super::anim;
use super::glyph::{self, Glyph};
use crate::palette::{
    self, GOLD, GOLD_DIM, GOLD_HI, GOLD_LO, NAVY, NAVY_HOVER, NAVY_RAISED, PLATE_BOTTOM,
    PLATE_TOP, TEXT,
};
use egui::epaint::{Vertex, WHITE_UV};
use egui::{
    Align2, Color32, CornerRadius, FontId, Id, InnerResponse, Mesh, Painter, Pos2, Rect, Response,
    RichText, Sense, Shadow, Shape, Stroke, StrokeKind, Ui, pos2, vec2,
};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// A filled rounded rectangle whose color runs from `top` to `bottom`.
pub fn gradient_rect(rect: Rect, radius: f32, top: Color32, bottom: Color32) -> Shape {
    let r = radius.clamp(0.0, rect.width().min(rect.height()) / 2.0);
    let steps = if r > 1.0 { 6 } else { 0 };
    // Rows from the top edge down: (y, inset from the sides).
    let mut rows: Vec<(f32, f32)> = (0..=steps)
        .map(|i| {
            let a = FRAC_PI_2 * i as f32 / steps.max(1) as f32;
            (rect.top() + r * (1.0 - a.cos()), r * (1.0 - a.sin()))
        })
        .collect();
    let bottom_rows: Vec<(f32, f32)> = rows
        .iter()
        .rev()
        .map(|&(y, inset)| (rect.bottom() - (y - rect.top()), inset))
        .collect();
    rows.extend(bottom_rows);
    let mut mesh = Mesh::default();
    for &(y, inset) in &rows {
        let t = ((y - rect.top()) / rect.height().max(1.0)).clamp(0.0, 1.0);
        let color = top.lerp_to_gamma(bottom, t);
        for x in [rect.left() + inset, rect.right() - inset] {
            mesh.vertices.push(Vertex {
                pos: pos2(x, y),
                uv: WHITE_UV,
                color,
            });
        }
    }
    for i in 0..rows.len() as u32 - 1 {
        let a = i * 2;
        mesh.indices.extend([a, a + 1, a + 2, a + 1, a + 3, a + 2]);
    }
    Shape::mesh(mesh)
}

/// A diamond stud centered on `c`.
fn diamond(c: Pos2, r: f32, fill: Color32) -> Shape {
    Shape::convex_polygon(
        vec![
            c + vec2(0.0, -r),
            c + vec2(r, 0.0),
            c + vec2(0.0, r),
            c + vec2(-r, 0.0),
        ],
        fill,
        Stroke::NONE,
    )
}

/// The beveled gold trim around `rect`: a dark keyline, the gold band, a lit
/// top edge and shaded bottom edge, a fine inner line and corner studs.
fn trim_shapes(rect: Rect, radius: f32, out: &mut Vec<Shape>) {
    out.push(Shape::rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, Color32::from_black_alpha(230)),
        StrokeKind::Outside,
    ));
    out.push(Shape::rect_stroke(
        rect,
        radius,
        Stroke::new(2.0, GOLD_DIM),
        StrokeKind::Inside,
    ));
    // Light from the upper left: bright top edge, shaded bottom edge.
    let r = radius;
    out.push(Shape::line_segment(
        [
            pos2(rect.left() + r, rect.top() + 1.0),
            pos2(rect.right() - r, rect.top() + 1.0),
        ],
        Stroke::new(2.0, GOLD),
    ));
    out.push(Shape::line_segment(
        [
            pos2(rect.left() + r, rect.bottom() - 1.0),
            pos2(rect.right() - r, rect.bottom() - 1.0),
        ],
        Stroke::new(2.0, GOLD_LO),
    ));
    out.push(Shape::line_segment(
        [
            pos2(rect.left() + r + 6.0, rect.top() + 3.0),
            pos2(rect.right() - r - 6.0, rect.top() + 3.0),
        ],
        Stroke::new(1.0, palette::with_alpha(GOLD_HI, 70)),
    ));
    out.push(Shape::rect_stroke(
        rect.shrink(5.0),
        (radius - 4.0).max(0.0),
        Stroke::new(1.0, palette::with_alpha(palette::TEAL, 42)),
        StrokeKind::Inside,
    ));
    let k = radius * 0.2929;
    for (x, y) in [
        (rect.left() + k, rect.top() + k),
        (rect.right() - k, rect.top() + k),
        (rect.left() + k, rect.bottom() - k),
        (rect.right() - k, rect.bottom() - k),
    ] {
        out.push(diamond(pos2(x, y), 3.6, GOLD_LO));
        out.push(diamond(pos2(x, y), 2.6, GOLD_HI));
    }
}

/// All the shapes of a plate: shadow, gradient, sheen and trim.
fn plate_shapes(rect: Rect, radius: f32) -> Vec<Shape> {
    let mut out = vec![
        Shadow {
            offset: [0, 6],
            blur: 20,
            spread: 1,
            color: Color32::from_black_alpha(150),
        }
        .as_shape(rect, CornerRadius::same(radius as u8))
        .into(),
        gradient_rect(rect, radius, PLATE_TOP, PLATE_BOTTOM),
    ];
    // A faint glow from the top of the panel.
    let sheen = Rect::from_min_size(rect.min, vec2(rect.width(), (rect.height() * 0.5).min(90.0)));
    out.push(gradient_rect(
        sheen.shrink2(vec2(2.0, 2.0)),
        radius - 2.0,
        Color32::from_white_alpha(14),
        Color32::TRANSPARENT,
    ));
    trim_shapes(rect, radius, &mut out);
    out
}

/// A plate painted into a rect.
pub fn plate(painter: &Painter, rect: Rect, radius: f32) {
    painter.add(Shape::Vec(plate_shapes(rect, radius)));
}

/// A plate that wraps whatever `add` draws: the frame of every floating HUD
/// panel. The plate is drawn behind the contents once their size is known.
pub fn plate_ui<R>(
    ui: &mut Ui,
    margin: i8,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let back = ui.painter().add(Shape::Noop);
    let out = egui::Frame::new().inner_margin(margin).show(ui, add);
    ui.painter()
        .set(back, Shape::Vec(plate_shapes(out.response.rect, 14.0)));
    out
}

/// A raised inset surface (rows, cards, wells) in the plate's navy.
fn well_shapes(rect: Rect, radius: f32, fill: Color32, edge: Color32) -> Vec<Shape> {
    vec![
        gradient_rect(
            rect,
            radius,
            fill.lerp_to_gamma(Color32::WHITE, 0.07),
            fill.lerp_to_gamma(Color32::BLACK, 0.25),
        ),
        Shape::rect_stroke(rect, radius, Stroke::new(1.0, edge), StrokeKind::Inside),
    ]
}

pub fn well(painter: &Painter, rect: Rect, radius: f32, fill: Color32, edge: Color32) {
    painter.add(Shape::Vec(well_shapes(rect, radius, fill, edge)));
}

/// A well that wraps whatever `add` draws.
pub fn well_ui<R>(
    ui: &mut Ui,
    margin: i8,
    fill: Color32,
    edge: Color32,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let back = ui.painter().add(Shape::Noop);
    let out = egui::Frame::new().inner_margin(margin).show(ui, add);
    ui.painter().set(
        back,
        Shape::Vec(well_shapes(out.response.rect, 10.0, fill, edge)),
    );
    out
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

/// A progress ring: a dim track plus the colored arc, with a lit edge.
pub fn ring(painter: &Painter, c: Pos2, r: f32, frac: f32, width: f32, color: Color32) {
    painter.circle_stroke(c, r, Stroke::new(width, Color32::from_black_alpha(110)));
    painter.circle_stroke(c, r, Stroke::new(width, Color32::from_white_alpha(22)));
    arc(painter, c, r, frac, width, color);
    arc(
        painter,
        c,
        r + width * 0.18,
        frac,
        width * 0.4,
        color.lerp_to_gamma(Color32::WHITE, 0.45),
    );
}

/// A navy disc with a beveled gold rim, the base of every round element.
pub fn disc(painter: &Painter, c: Pos2, r: f32, fill: Color32, rim: Color32) {
    painter.circle_filled(c + vec2(0.0, 2.0), r + 1.5, Color32::from_black_alpha(90));
    painter.circle_filled(c, r, fill);
    // Inner shade at the bottom, sheen at the top.
    painter.circle_stroke(
        c,
        r - 2.5,
        Stroke::new(3.0, Color32::from_black_alpha(46)),
    );
    painter.circle_stroke(c, r, Stroke::new(2.0, rim));
    let lit: Vec<Pos2> = (0..=16)
        .map(|i| {
            let a = PI * (1.05 + 0.5 * i as f32 / 16.0);
            c + vec2(a.cos(), a.sin()) * (r - 1.2)
        })
        .collect();
    painter.add(Shape::line(
        lit,
        Stroke::new(1.5, palette::with_alpha(GOLD_HI, 150)),
    ));
    painter.circle_stroke(
        c,
        r + 1.0,
        Stroke::new(1.0, Color32::from_black_alpha(190)),
    );
}

/// A thin ornamental rule: a diamond at `from` and a line fading out to
/// `to_x`.
pub fn fade_rule(painter: &Painter, from: Pos2, to_x: f32, color: Color32) {
    let mut mesh = Mesh::default();
    let (a, b) = (from.x, to_x);
    let h = 0.75;
    for (x, alpha) in [(a, 1.0), (a + (b - a) * 0.5, 0.55), (b, 0.0)] {
        let c = palette::with_alpha(color, (color.a() as f32 * alpha) as u8);
        for dy in [-h, h] {
            mesh.vertices.push(Vertex {
                pos: pos2(x, from.y + dy),
                uv: WHITE_UV,
                color: c,
            });
        }
    }
    mesh.indices.extend([0, 1, 2, 1, 3, 2, 2, 3, 4, 3, 5, 4]);
    painter.add(Shape::mesh(mesh));
    painter.add(diamond(from, 3.0, color));
}

/// A rule with a diamond in the middle that fades out to both ends.
pub fn divider(painter: &Painter, left: f32, right: f32, y: f32, color: Color32) {
    let mid = (left + right) / 2.0;
    for (from, to) in [(mid - 8.0, left), (mid + 8.0, right)] {
        let mut mesh = Mesh::default();
        for (x, alpha) in [(from, 1.0), (to, 0.0)] {
            let c = palette::with_alpha(color, (color.a() as f32 * alpha) as u8);
            for dy in [-0.75, 0.75] {
                mesh.vertices.push(Vertex {
                    pos: pos2(x, y + dy),
                    uv: WHITE_UV,
                    color: c,
                });
            }
        }
        mesh.indices.extend([0, 1, 2, 1, 3, 2]);
        painter.add(Shape::mesh(mesh));
    }
    painter.add(diamond(pos2(mid, y), 4.0, color));
    painter.add(diamond(pos2(mid, y), 2.2, GOLD_LO));
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

/// Bold text (a real bold face).
pub fn text_bold(
    painter: &Painter,
    pos: Pos2,
    align: Align2,
    s: impl ToString,
    size: f32,
    color: Color32,
) -> Rect {
    painter.text(pos, align, s.to_string(), palette::bold(size), color)
}

/// Serif display text for titles.
pub fn text_display(
    painter: &Painter,
    pos: Pos2,
    align: Align2,
    s: impl ToString,
    size: f32,
    color: Color32,
) -> Rect {
    painter.text(pos, align, s.to_string(), palette::display(size), color)
}

/// Title text for use in labels.
pub fn title(s: impl Into<String>, size: f32) -> RichText {
    RichText::new(s).font(palette::display(size)).color(GOLD)
}

/// Bold text for use in labels.
pub fn bold(s: impl Into<String>, size: f32) -> RichText {
    RichText::new(s).font(palette::bold(size))
}

/// A small filled badge with a number, e.g. the unread count. It pops
/// (scales up with a bounce and a ripple) when its number changes or when
/// it reappears.
pub fn badge(painter: &Painter, c: Pos2, label: &str, fill: Color32) {
    let ctx = painter.ctx();
    let id = Id::unique(("badge", fill.to_array()));
    let (frame, key) = (ctx.cumulative_frame_nr(), anim::hash(&label));
    let prev = ctx.data(|d| d.get_temp::<(u64, u64)>(id));
    ctx.data_mut(|d| d.insert_temp(id, (frame, key)));
    if prev.is_some_and(|(f, k)| f + 1 < frame || k != key) {
        anim::trigger(ctx, id.with("pop"));
    }
    let pop = anim::progress(ctx, id.with("pop"), 0.45);
    let scale = pop.map_or(1.0, |p| 0.55 + 0.45 * anim::ease_out_back(p));
    let galley = painter.layout_no_wrap(
        label.to_string(),
        palette::bold(12.0 * scale),
        Color32::WHITE,
    );
    let w = (galley.size().x + 8.0).max(18.0 * scale);
    let rect = Rect::from_center_size(c, vec2(w, 18.0 * scale));
    if let Some(p) = pop {
        let e = anim::ease_out(p);
        painter.rect_stroke(
            rect.expand(2.0 + 9.0 * e),
            9.0 + 9.0 * e,
            Stroke::new(2.0, fill.gamma_multiply((1.0 - e) * 0.9)),
            StrokeKind::Outside,
        );
    }
    let fill = match pop {
        Some(p) => anim::mix(Color32::WHITE, fill, anim::ease_out((p * 2.5).min(1.0))),
        None => fill,
    };
    painter.add(Shape::Vec(vec![
        Shadow {
            offset: [0, 1],
            blur: 4,
            spread: 0,
            color: Color32::from_black_alpha(120),
        }
        .as_shape(rect, CornerRadius::same((9.0 * scale) as u8))
        .into(),
        gradient_rect(
            rect,
            9.0 * scale,
            fill.lerp_to_gamma(Color32::WHITE, 0.25),
            fill.lerp_to_gamma(Color32::BLACK, 0.2),
        ),
    ]));
    painter.rect_stroke(rect, 9.0 * scale, Stroke::new(1.5, NAVY), StrokeKind::Outside);
    painter.galley(c - galley.size() / 2.0, galley, Color32::WHITE);
}

/// A key-cap hint such as `N` or `Space`, drawn centered on `c`.
pub fn keycap(painter: &Painter, c: Pos2, label: &str) {
    let galley = painter.layout_no_wrap(label.to_string(), palette::bold(10.5), TEXT);
    let w = (galley.size().x + 8.0).max(16.0);
    let rect = Rect::from_center_size(c, vec2(w, 16.0));
    painter.add(gradient_rect(
        rect,
        4.0,
        Color32::from_rgb(20, 30, 46),
        Color32::from_rgb(6, 11, 20),
    ));
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
    let ctx = painter.ctx();
    let id = response.id;
    let hot = anim::hover(ctx, id.with("hot"), response.hovered() && enabled, 0.12);
    let down = anim::hover(
        ctx,
        id.with("down"),
        enabled && response.is_pointer_button_down_on(),
        0.06,
    );
    if response.clicked() && enabled {
        anim::trigger(ctx, id.with("pop"));
    }
    let c = rect.center();
    let r = rect.width() / 2.0 * (1.0 + 0.05 * hot - 0.08 * down);
    let lit = if active { 1.0 } else { hot };
    let fill = anim::mix(
        if active { palette::TEAL_DIM } else { NAVY_RAISED },
        NAVY_HOVER,
        if active { hot * 0.5 } else { hot },
    );
    let rim = anim::mix(GOLD_DIM, GOLD, lit);
    glow_circle(painter, c, r - 1.0, GOLD, hot * 0.8);
    if active {
        glow_circle(painter, c, r - 1.0, palette::TEAL, 0.6);
    }
    if let Some(p) = anim::progress(ctx, id.with("pop"), 0.3) {
        let e = anim::ease_out(p);
        painter.circle_stroke(
            c,
            r + 2.0 + 10.0 * e,
            Stroke::new(2.0, GOLD.gamma_multiply((1.0 - e) * 0.8)),
        );
    }
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

/// A soft halo around a rounded rect: a few widening, fading strokes.
pub fn glow(painter: &Painter, rect: Rect, radius: f32, color: Color32, strength: f32) {
    if strength <= 0.01 {
        return;
    }
    for (i, a) in [0.32, 0.2, 0.11, 0.05].into_iter().enumerate() {
        let grow = 1.5 + i as f32 * 2.2;
        painter.rect_stroke(
            rect.expand(grow),
            radius + grow,
            Stroke::new(2.4, color.gamma_multiply(a * strength)),
            StrokeKind::Middle,
        );
    }
}

/// A soft halo around a circle.
pub fn glow_circle(painter: &Painter, c: Pos2, r: f32, color: Color32, strength: f32) {
    if strength <= 0.01 {
        return;
    }
    for (i, a) in [0.32, 0.2, 0.11, 0.05].into_iter().enumerate() {
        let grow = 2.0 + i as f32 * 2.6;
        painter.circle_stroke(c, r + grow, Stroke::new(2.6, color.gamma_multiply(a * strength)));
    }
}

/// A pill-shaped button with optional glyph; returns the response.
pub fn pill_button(ui: &mut Ui, g: Option<Glyph>, label: &str, color: Color32, min_width: f32) -> Response {
    pill_button_if(ui, g, label, color, min_width, true)
}

/// A pill-shaped button that can be dimmed and made unclickable.
pub fn pill_button_if(
    ui: &mut Ui,
    g: Option<Glyph>,
    label: &str,
    color: Color32,
    min_width: f32,
    enabled: bool,
) -> Response {
    let font = palette::bold(14.5);
    let text_color = if enabled { TEXT } else { palette::TEXT_DIM.gamma_multiply(0.6) };
    let color = if enabled { color } else { color.gamma_multiply(0.4) };
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_color);
    let icon = if g.is_some() { 24.0 } else { 0.0 };
    let w = (galley.size().x + icon + 28.0).max(min_width);
    let (base, response) = ui.allocate_exact_size(
        vec2(w, 34.0),
        if enabled { Sense::click() } else { Sense::hover() },
    );
    let ctx = ui.ctx().clone();
    let id = response.id;
    let hot = anim::hover(&ctx, id.with("hot"), response.hovered() && enabled, 0.12);
    let down = anim::hover(
        &ctx,
        id.with("down"),
        enabled && response.is_pointer_button_down_on(),
        0.06,
    );
    if response.clicked() && enabled {
        anim::trigger(&ctx, id.with("pop"));
    }
    let rect = base
        .translate(vec2(0.0, -1.5 * hot * (1.0 - down)))
        .shrink(1.2 * down);
    let painter = ui.painter();
    glow(painter, rect, 17.0, GOLD, hot * 0.7);
    if let Some(p) = anim::progress(&ctx, id.with("pop"), 0.3) {
        let e = anim::ease_out(p);
        painter.rect_stroke(
            rect.expand(1.0 + 7.0 * e),
            17.0 + 7.0 * e,
            Stroke::new(2.0, GOLD.gamma_multiply((1.0 - e) * 0.8)),
            StrokeKind::Outside,
        );
    }
    painter.add(Shape::from(
        Shadow {
            offset: [0, 2],
            blur: 6,
            spread: 0,
            color: Color32::from_black_alpha(100),
        }
        .as_shape(rect, CornerRadius::same(17)),
    ));
    well(
        painter,
        rect,
        17.0,
        anim::mix(NAVY_RAISED, NAVY_HOVER, hot),
        anim::mix(GOLD_DIM, GOLD, hot),
    );
    painter.rect_stroke(
        rect.shrink(2.0),
        15.0,
        Stroke::new(1.0, palette::with_alpha(GOLD_HI, (26.0 + 34.0 * hot) as u8)),
        StrokeKind::Inside,
    );
    let mut x = rect.center().x - (galley.size().x + icon) / 2.0;
    if let Some(g) = g {
        glyph::paint(painter, g, pos2(x + 8.0, rect.center().y), 8.0, color);
        x += 24.0;
    }
    painter.galley(pos2(x, rect.center().y - galley.size().y / 2.0), galley, text_color);
    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

/// Paints a texture clipped to a circle, `uv` selecting the part to show.
pub fn image_circle(painter: &Painter, tex: egui::TextureId, c: Pos2, r: f32, uv: Rect) {
    let mut mesh = Mesh::with_texture(tex);
    let n = 64;
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

/// A leader's portrait in a circle, falling back to a monogram in the civ's
/// colors. The medallion has a soft vignette and a double gold rim.
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
    painter.circle_filled(c + vec2(0.0, 2.0), r + 2.0, Color32::from_black_alpha(100));
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
            // Vignette: darken toward the rim, lit from the upper left.
            for (k, a) in [(0.0, 70u8), (1.5, 44), (3.0, 22)] {
                painter.circle_stroke(
                    c,
                    r - 2.0 - k * r / 30.0,
                    Stroke::new(r / 12.0, Color32::from_black_alpha(a)),
                );
            }
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

/// A leader's portrait medallion: the painting in a disc, a colored status
/// ring and a double gold rim. `r` is the outer radius.
#[allow(clippy::too_many_arguments)]
pub fn medallion(
    painter: &Painter,
    portraits: &mut crate::portraits::Portraits,
    c: Pos2,
    r: f32,
    leader_id: &str,
    name: &str,
    colors: ([u8; 3], [u8; 3]),
    ring: Color32,
) {
    portrait_disc(painter, portraits, c, r - 4.0, leader_id, name, colors);
    painter.circle_stroke(c, r - 2.0, Stroke::new(3.5, ring));
    painter.circle_stroke(c, r - 3.8, Stroke::new(1.0, Color32::from_black_alpha(150)));
    painter.circle_stroke(c, r + 0.5, Stroke::new(1.5, GOLD_DIM));
    painter.circle_stroke(c, r + 2.0, Stroke::new(1.0, Color32::from_black_alpha(190)));
}

/// A horizontal bar: dark inset track and a glossy colored fill.
pub fn bar(painter: &Painter, rect: Rect, frac: f32, color: Color32) {
    let radius = rect.height() / 2.0;
    painter.rect_filled(rect, radius, Color32::from_black_alpha(120));
    painter.rect_filled(rect.shrink(0.5), radius, Color32::from_white_alpha(14));
    if frac > 0.0 {
        let fill = Rect::from_min_size(
            rect.min,
            vec2((rect.width() * frac.clamp(0.0, 1.0)).max(rect.height()), rect.height()),
        )
        .shrink(0.75);
        painter.add(gradient_rect(
            fill,
            fill.height() / 2.0,
            color.lerp_to_gamma(Color32::WHITE, 0.35),
            color.lerp_to_gamma(Color32::BLACK, 0.25),
        ));
    }
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, Color32::from_black_alpha(170)),
        StrokeKind::Outside,
    );
}

/// Like `bar`, but the fill eases to new values (`id` must be stable).
pub fn bar_eased(painter: &Painter, id: Id, rect: Rect, frac: f32, color: Color32) {
    let f = anim::smooth(painter.ctx(), id, frac.clamp(0.0, 1.0), 9.0);
    bar(painter, rect, f, color);
}

/// A gold section heading with an ornamental rule.
pub fn heading(ui: &mut Ui, s: &str) {
    ui.add_space(4.0);
    ui.label(title(s, 17.0));
    let y = ui.cursor().top() + 1.0;
    fade_rule(
        ui.painter(),
        pos2(ui.min_rect().left() + 3.0, y),
        ui.max_rect().right(),
        palette::with_alpha(GOLD, 200),
    );
    ui.add_space(6.0);
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
        .layout_no_wrap(label.to_string(), palette::bold(14.0), TEXT);
    let (rect, _) = ui.allocate_exact_size(vec2(galley.size().x + 26.0, 26.0), Sense::hover());
    let painter = ui.painter();
    painter.add(gradient_rect(
        rect,
        13.0,
        color.gamma_multiply(0.45),
        color.gamma_multiply(0.2),
    ));
    painter.rect_stroke(rect, 13.0, Stroke::new(1.5, color), StrokeKind::Inside);
    painter.galley(rect.center() - galley.size() / 2.0, galley, TEXT);
}

/// The backdrop of the title screens: deep navy with a soft glow high up
/// and a faint lattice of hexagons.
pub fn backdrop(painter: &Painter, rect: Rect) {
    painter.add(gradient_rect(
        rect,
        0.0,
        Color32::from_rgb(14, 24, 42),
        Color32::from_rgb(4, 8, 16),
    ));
    // Hex lattice, fading out toward the edges by drawing it in the glow.
    let r = 52.0;
    let (dx, dy) = (r * 3.0_f32.sqrt(), r * 1.5);
    let stroke = Stroke::new(1.0, palette::with_alpha(GOLD_DIM, 20));
    let mut lines: Vec<Shape> = Vec::new();
    let rows = (rect.height() / dy) as i32 + 2;
    let cols = (rect.width() / dx) as i32 + 2;
    for row in -1..rows {
        for col in -1..cols {
            let c = pos2(
                rect.left() + col as f32 * dx + if row % 2 == 0 { 0.0 } else { dx / 2.0 },
                rect.top() + row as f32 * dy,
            );
            // Three edges per hexagon; the neighbors supply the rest.
            let pts: Vec<Pos2> = (2..=5)
                .map(|i| {
                    let a = PI / 6.0 + i as f32 * PI / 3.0;
                    c + vec2(a.cos(), a.sin()) * r
                })
                .collect();
            lines.push(Shape::line(pts, stroke));
        }
    }
    painter.add(Shape::Vec(lines));
    // The glow: a radial fan from the upper middle.
    let center = pos2(rect.center().x, rect.top() + rect.height() * 0.28);
    let radius = rect.width().max(rect.height()) * 0.75;
    let mut mesh = Mesh::default();
    mesh.vertices.push(Vertex {
        pos: center,
        uv: WHITE_UV,
        color: Color32::from_rgba_premultiplied(26, 48, 80, 90),
    });
    let n = 48;
    for i in 0..=n {
        let a = i as f32 / n as f32 * TAU;
        mesh.vertices.push(Vertex {
            pos: center + vec2(a.cos(), a.sin()) * radius,
            uv: WHITE_UV,
            color: Color32::TRANSPARENT,
        });
    }
    for i in 1..=n as u32 {
        mesh.indices.extend([0, i, i + 1]);
    }
    painter.with_clip_rect(rect).add(Shape::mesh(mesh));
}

/// A segmented-control button: a pill that is gold-lit when selected.
pub fn toggle_pill(ui: &mut Ui, label: &str, selected: bool) -> Response {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), palette::bold(14.0), TEXT);
    let (rect, response) =
        ui.allocate_exact_size(vec2(galley.size().x + 28.0, 30.0), Sense::click());
    let painter = ui.painter();
    let hovered = response.hovered();
    well(
        painter,
        rect,
        15.0,
        if selected {
            Color32::from_rgb(70, 56, 24)
        } else if hovered {
            NAVY_HOVER
        } else {
            NAVY_RAISED
        },
        Color32::TRANSPARENT,
    );
    painter.rect_stroke(
        rect,
        15.0,
        Stroke::new(
            if selected { 2.0 } else { 1.5 },
            if selected || hovered { GOLD } else { GOLD_DIM },
        ),
        StrokeKind::Inside,
    );
    painter.galley(
        rect.center() - galley.size() / 2.0,
        galley,
        if selected { GOLD_HI } else { TEXT },
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

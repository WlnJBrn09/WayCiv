//! Small builders for the soft, gradient overlays on the map. Everything
//! goes into one `Mesh` per layer, so a layer costs one draw however many
//! tiles it covers. Colors are egui's premultiplied `Color32`, so fading to
//! `with_alpha(c, 0)` blends cleanly.

use egui::{Color32, Mesh, Pos2, Vec2, vec2};
use std::f32::consts::TAU;

/// A point with its color.
pub type Vert = (Pos2, Color32);

pub fn tri(m: &mut Mesh, a: Vert, b: Vert, c: Vert) {
    let i = m.vertices.len() as u32;
    for (p, col) in [a, b, c] {
        m.colored_vertex(p, col);
    }
    m.add_triangle(i, i + 1, i + 2);
}

/// A convex quad given in order around its outline.
pub fn quad(m: &mut Mesh, a: Vert, b: Vert, c: Vert, d: Vert) {
    let i = m.vertices.len() as u32;
    for (p, col) in [a, b, c, d] {
        m.colored_vertex(p, col);
    }
    m.add_triangle(i, i + 1, i + 2);
    m.add_triangle(i, i + 2, i + 3);
}

/// A convex polygon filled with one color.
pub fn fill(m: &mut Mesh, pts: &[Pos2], color: Color32) {
    let i = m.vertices.len() as u32;
    for &p in pts {
        m.colored_vertex(p, color);
    }
    for k in 1..pts.len() as u32 - 1 {
        m.add_triangle(i, i + k, i + k + 1);
    }
}

/// A straight line with a one-pixel soft edge (the same look as egui's own
/// strokes) and square ends that overshoot by half the width, so lines
/// meeting at a hex corner close up without a notch.
pub fn line(m: &mut Mesh, a: Pos2, b: Pos2, width: f32, color: Color32) {
    let along = (b - a).normalized();
    if !along.x.is_finite() {
        return;
    }
    let side = vec2(-along.y, along.x);
    let half = width * 0.5;
    let (outer, inner) = (half + 0.5, (half - 0.5).max(0.0));
    // Lines thinner than a pixel fade instead of vanishing.
    let color = if width < 1.0 { color.gamma_multiply(width) } else { color };
    let clear = Color32::TRANSPARENT;
    let (a, b) = (a - along * half, b + along * half);
    let rows = [(-outer, clear), (-inner, color), (inner, color), (outer, clear)];
    for w in rows.windows(2) {
        quad(
            m,
            (a + side * w[0].0, w[0].1),
            (b + side * w[0].0, w[0].1),
            (b + side * w[1].0, w[1].1),
            (a + side * w[1].0, w[1].1),
        );
    }
}

/// A closed polygon outline.
pub fn outline(m: &mut Mesh, pts: &[Pos2], width: f32, color: Color32) {
    for i in 0..pts.len() {
        line(m, pts[i], pts[(i + 1) % pts.len()], width, color);
    }
}

/// An ellipse that fades from `color` at the center to nothing at the rim.
pub fn radial(m: &mut Mesh, c: Pos2, radius: Vec2, color: Color32) {
    const STEPS: usize = 20;
    let i = m.vertices.len() as u32;
    m.colored_vertex(c, color);
    for k in 0..STEPS {
        let a = TAU * k as f32 / STEPS as f32;
        m.colored_vertex(c + vec2(a.cos() * radius.x, a.sin() * radius.y), Color32::TRANSPARENT);
    }
    for k in 0..STEPS as u32 {
        m.add_triangle(i, i + 1 + k, i + 1 + (k + 1) % STEPS as u32);
    }
}

/// A soft round blob: opaque in the middle third, easing out to the rim.
pub fn puff(m: &mut Mesh, c: Pos2, radius: f32, color: Color32) {
    const STEPS: usize = 16;
    let rows = [(0.0, 1.0), (0.45, 0.8), (0.75, 0.35), (1.0, 0.0)];
    let i = m.vertices.len() as u32;
    m.colored_vertex(c, color);
    for &(r, a) in &rows[1..] {
        for k in 0..STEPS {
            let ang = TAU * k as f32 / STEPS as f32;
            m.colored_vertex(c + vec2(ang.cos(), ang.sin()) * radius * r, color.gamma_multiply(a));
        }
    }
    let ring = |row: u32, k: u32| i + 1 + row * STEPS as u32 + k % STEPS as u32;
    for k in 0..STEPS as u32 {
        m.add_triangle(i, ring(0, k), ring(0, k + 1));
        for row in 0..2 {
            m.add_triangle(ring(row, k), ring(row + 1, k), ring(row + 1, k + 1));
            m.add_triangle(ring(row, k), ring(row + 1, k + 1), ring(row, k + 1));
        }
    }
}

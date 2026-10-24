//! Painting the one-off map effects scheduled in `anim`: arrows, stones and
//! shells in flight, impact bursts, rising damage numbers, dust and shock
//! rings, and sparkles. Each is drawn from its progress `u` (0 to 1) around
//! the center `c` of the tile it belongs to; `z` is the hex radius in pixels.

use super::anim::{Fx, FxKind, Shot, ease_out, smooth};
use egui::{Align2, Color32, FontId, Pos2, Shape, Stroke, vec2};
use std::f32::consts::{PI, TAU};

/// Paints `fx` at progress `u` around tile center `c`.
pub fn draw(painter: &egui::Painter, fx: &Fx, u: f32, c: Pos2, z: f32) {
    match &fx.kind {
        FxKind::Projectile { to, shot } => projectile(painter, *shot, u, c, *to * z, z),
        FxKind::Impact { big } => impact(painter, u, c, z, *big),
        FxKind::Float { text, color, big } => float(painter, text, *color, *big, u, c, z),
        FxKind::Ring { color, size } => ring(painter, *color, *size, u, c, z),
        FxKind::Puff => puff(painter, u, c, z),
        FxKind::Sparkle { seed } => sparkle(painter, *seed, u, c, z),
    }
}

fn hash(a: u32, b: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 13;
    (h & 0xffff) as f32 / 65535.0
}

fn alpha(c: [u8; 3], a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c[0], c[1], c[2], (a.clamp(0.0, 1.0) * 255.0) as u8)
}

const DUST: [u8; 3] = [214, 192, 150];

fn projectile(painter: &egui::Painter, shot: Shot, u: f32, c: Pos2, to: egui::Vec2, z: f32) {
    let len = to.length();
    let arc = match shot {
        Shot::Arrow => 0.20,
        Shot::Stone => 0.34,
        Shot::Shell => 0.12,
    } * len.max(z);
    let chest = vec2(0.0, -0.25 * z);
    let at = |u: f32| c + chest + to * u + vec2(0.0, -4.0 * arc * u * (1.0 - u));
    let p = at(u);
    // The tangent, for pointing the arrow and trailing smoke.
    let ahead = at((u + 0.02).min(1.0));
    let dir = (ahead - at((u - 0.02).max(0.0))).normalized();
    match shot {
        Shot::Arrow => {
            let tail = p - dir * (0.38 * z);
            painter.line_segment([tail, p], Stroke::new((z * 0.05).max(1.5), Color32::from_rgb(70, 50, 30)));
            painter.line_segment(
                [tail - dir * (0.06 * z), tail + dir * (0.05 * z)],
                Stroke::new((z * 0.1).max(2.0), Color32::from_rgb(230, 230, 220)),
            );
            let head = vec2(-dir.y, dir.x) * (0.05 * z);
            painter.add(Shape::convex_polygon(
                vec![p + dir * (0.07 * z), p + head, p - head],
                Color32::from_rgb(210, 214, 220),
                Stroke::NONE,
            ));
        }
        Shot::Stone => {
            // A shadow slides along the ground under it.
            let ground = c + to * u + vec2(0.0, 0.1 * z);
            painter.add(Shape::ellipse_filled(
                ground,
                vec2(0.14 * z, 0.06 * z),
                Color32::from_black_alpha(80),
            ));
            let r = (z * 0.11).max(2.5);
            painter.circle_filled(p, r, Color32::from_rgb(92, 86, 80));
            painter.circle_filled(p + vec2(-r * 0.3, -r * 0.3), r * 0.4, Color32::from_rgb(150, 142, 132));
        }
        Shot::Shell => {
            for k in 1..=5 {
                let q = at((u - 0.025 * k as f32).max(0.0));
                painter.circle_filled(
                    q,
                    (z * (0.10 - 0.012 * k as f32)).max(1.0),
                    alpha([255, 170, 80], 0.5 - 0.09 * k as f32),
                );
            }
            let r = (z * 0.1).max(2.5);
            painter.circle_filled(p, r, Color32::from_rgb(34, 34, 38));
            painter.circle_filled(p + vec2(-r * 0.3, -r * 0.3), r * 0.35, Color32::from_rgb(110, 110, 120));
        }
    }
}

fn impact(painter: &egui::Painter, u: f32, c: Pos2, z: f32, big: bool) {
    let k = if big { 1.45 } else { 1.0 };
    let at = c + vec2(0.0, -0.1 * z);
    let e = ease_out(u);
    // A quick white flash, then sparks flying outward.
    let flash = (1.0 - u * 3.0).max(0.0);
    if flash > 0.0 {
        painter.circle_filled(at, z * 0.26 * k * (0.6 + e), alpha([255, 245, 210], flash * 0.7));
    }
    let a = (1.0 - u).powf(1.5);
    for i in 0..8 {
        let ang = TAU * i as f32 / 8.0 + 0.4 * (i % 3) as f32;
        let dir = vec2(ang.cos(), ang.sin() * 0.7);
        let (r0, r1) = (0.1 * z * k + 0.35 * z * k * e, 0.1 * z * k + 0.55 * z * k * e);
        painter.line_segment(
            [at + dir * r0, at + dir * r1],
            Stroke::new((z * 0.04 * k).max(1.2), alpha([255, 214, 120], a)),
        );
    }
    // Smoke.
    painter.circle_filled(
        at + vec2(0.0, -0.2 * z * e),
        z * 0.2 * k * (0.5 + e),
        alpha([120, 116, 112], 0.35 * (1.0 - u)),
    );
}

fn float(painter: &egui::Painter, text: &str, color: [u8; 3], big: bool, u: f32, c: Pos2, z: f32) {
    let rise = 0.95 * z * ease_out(u);
    let size = (z * if big { 0.62 } else { 0.48 }).clamp(13.0, 34.0);
    // Pops a little larger as it appears.
    let pop = 1.0 + 0.45 * (1.0 - ease_out(u * 5.0));
    let a = 1.0 - smooth((u - 0.6) / 0.4);
    let font = FontId::proportional(size * pop);
    let at = c + vec2(0.0, -0.45 * z - rise);
    let shade = alpha([10, 10, 14], a * 0.9);
    for (dx, dy) in [(-1.5, 0.0), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5), (1.5, 1.5)] {
        painter.text(at + vec2(dx, dy), Align2::CENTER_CENTER, text, font.clone(), shade);
    }
    painter.text(at, Align2::CENTER_CENTER, text, font, alpha(color, a));
}

fn ring(painter: &egui::Painter, color: Option<[u8; 3]>, size: f32, u: f32, c: Pos2, z: f32) {
    let e = ease_out(u);
    let rx = z * size * (0.15 + 0.85 * e);
    let col = color.unwrap_or(DUST);
    let ground = c + vec2(0.0, 0.2 * z);
    painter.add(Shape::ellipse_stroke(
        ground,
        vec2(rx, rx * 0.5),
        Stroke::new((z * 0.1 * (1.0 - u)).max(1.0), alpha(col, 0.7 * (1.0 - u))),
    ));
    if u < 0.5 {
        painter.add(Shape::ellipse_filled(
            ground,
            vec2(rx * 0.8, rx * 0.4),
            alpha(col, 0.18 * (1.0 - u * 2.0)),
        ));
    }
}

fn puff(painter: &egui::Painter, u: f32, c: Pos2, z: f32) {
    let e = ease_out(u);
    for i in 0..7 {
        let ang = TAU * i as f32 / 7.0 + 0.5;
        let reach = 0.2 + 0.18 * hash(i, 3);
        let at = c + vec2(ang.cos() * reach * z * e, 0.28 * z + ang.sin() * reach * 0.45 * z * e - 0.2 * z * u);
        painter.circle_filled(
            at,
            z * (0.05 + 0.06 * u) * (0.7 + 0.6 * hash(i, 9)),
            alpha(DUST, 0.55 * (1.0 - u)),
        );
    }
}

fn sparkle(painter: &egui::Painter, seed: u32, u: f32, c: Pos2, z: f32) {
    for i in 0..8u32 {
        // Each star has its own start and short life.
        let start = 0.55 * hash(seed, i * 3 + 1);
        let v = (u - start) / 0.45;
        if !(0.0..1.0).contains(&v) {
            continue;
        }
        let ang = TAU * hash(seed, i * 3 + 2);
        let reach = 0.2 + 0.6 * hash(seed, i * 3 + 3);
        let at = c + vec2(ang.cos() * reach * z, ang.sin() * reach * 0.6 * z - 0.35 * z - 0.35 * z * v);
        let s = z * 0.2 * (v * PI).sin() * (0.7 + 0.5 * hash(seed, i + 40));
        let col = alpha([255, 238, 170], 0.95 * (v * PI).sin());
        let w = s * 0.22;
        painter.add(Shape::convex_polygon(
            vec![at + vec2(0.0, -s), at + vec2(w, 0.0), at + vec2(0.0, s), at + vec2(-w, 0.0)],
            col,
            Stroke::NONE,
        ));
        let (s2, w2) = (s * 0.65, w);
        painter.add(Shape::convex_polygon(
            vec![at + vec2(-s2, 0.0), at + vec2(0.0, w2), at + vec2(s2, 0.0), at + vec2(0.0, -w2)],
            col,
            Stroke::NONE,
        ));
    }
}

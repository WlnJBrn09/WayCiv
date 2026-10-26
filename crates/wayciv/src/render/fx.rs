//! Painting the one-off map effects scheduled in `anim`: arrows, stones and
//! shells in flight, impact bursts, rising damage numbers, dust and shock
//! rings, and sparkles. Each is drawn from its progress `u` (0 to 1) around
//! the center `c` of the tile it belongs to; `z` is the hex radius in pixels.
//! Glows and smoke are soft gradient meshes, so they blend rather than blot.

use super::anim::{Fx, FxKind, Shot, ease_out, smooth};
use super::mesh;
use egui::{Align2, Color32, FontId, Mesh, Pos2, Shape, Stroke, vec2};
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
const SPARK: [u8; 3] = [255, 214, 120];

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
    // Everything flying slides a shadow along the ground beneath it.
    let ground = c + to * u + vec2(0.0, 0.1 * z);
    let lift = (p.y - ground.y).abs() / z.max(1.0);
    let squash = 1.0 / (1.0 + lift * 1.4);
    let mut shade = Mesh::default();
    mesh::radial(&mut shade, ground, vec2(0.17 * z * squash, 0.07 * z * squash), Color32::from_black_alpha((90.0 * squash) as u8));
    painter.add(Shape::mesh(shade));
    match shot {
        Shot::Arrow => {
            // A faint streak of air behind the shaft.
            let tail = p - dir * (0.38 * z);
            let mut streak = Mesh::default();
            let wake = p - dir * (0.9 * z);
            let side = vec2(-dir.y, dir.x) * (0.025 * z);
            let clear = Color32::TRANSPARENT;
            let white = Color32::from_white_alpha(46);
            mesh::tri(&mut streak, (wake, clear), (tail - side, white), (tail + side, white));
            painter.add(Shape::mesh(streak));
            painter.line_segment([tail, p], Stroke::new((z * 0.05).max(1.5), Color32::from_rgb(86, 60, 34)));
            painter.line_segment(
                [tail - dir * (0.06 * z), tail + dir * (0.05 * z)],
                Stroke::new((z * 0.1).max(2.0), Color32::from_rgb(236, 232, 220)),
            );
            let head = vec2(-dir.y, dir.x) * (0.05 * z);
            painter.add(Shape::convex_polygon(
                vec![p + dir * (0.08 * z), p + head, p - head],
                Color32::from_rgb(214, 218, 226),
                Stroke::NONE,
            ));
        }
        Shot::Stone => {
            // A short trail of dust the stone kicks up.
            let mut trail = Mesh::default();
            for k in 1..=4 {
                let q = at((u - 0.03 * k as f32).max(0.0));
                mesh::puff(&mut trail, q, z * (0.08 + 0.015 * k as f32), alpha([150, 140, 126], 0.22 - 0.04 * k as f32));
            }
            painter.add(Shape::mesh(trail));
            let r = (z * 0.11).max(2.5);
            painter.circle_filled(p, r, Color32::from_rgb(88, 82, 76));
            painter.circle_filled(p + vec2(r * 0.18, r * 0.2), r * 0.7, Color32::from_black_alpha(60));
            painter.circle_filled(p + vec2(-r * 0.3, -r * 0.3), r * 0.4, Color32::from_rgb(158, 150, 140));
        }
        Shot::Shell => {
            let mut trail = Mesh::default();
            for k in 1..=6 {
                let q = at((u - 0.025 * k as f32).max(0.0));
                let a = 0.5 - 0.07 * k as f32;
                mesh::puff(&mut trail, q, z * (0.14 - 0.012 * k as f32).max(0.04), alpha([255, 160, 70], a));
            }
            mesh::radial(&mut trail, p, vec2(z * 0.3, z * 0.3), alpha([255, 190, 100], 0.4));
            painter.add(Shape::mesh(trail));
            let r = (z * 0.1).max(2.5);
            painter.circle_filled(p, r, Color32::from_rgb(30, 30, 34));
            painter.circle_filled(p + vec2(-r * 0.3, -r * 0.3), r * 0.35, Color32::from_rgb(116, 116, 128));
        }
    }
}

fn impact(painter: &egui::Painter, u: f32, c: Pos2, z: f32, big: bool) {
    let k = if big { 1.45 } else { 1.0 };
    let at = c + vec2(0.0, -0.1 * z);
    let e = ease_out(u);
    let mut m = Mesh::default();
    // Smoke first, rising and spreading, then the fire in front of it.
    for i in 0..4u32 {
        let drift = vec2((hash(i, 5) - 0.5) * 0.5 * z * k * e, -0.18 * z * k * e * (0.6 + 0.5 * hash(i, 7)));
        mesh::puff(
            &mut m,
            at + drift,
            z * 0.2 * k * (0.5 + e) * (0.7 + 0.5 * hash(i, 11)),
            alpha([104, 98, 94], 0.42 * (1.0 - u)),
        );
    }
    // A hot flash that bleeds out into a warm bloom.
    let flash = (1.0 - u * 2.6).max(0.0);
    if flash > 0.0 {
        let r = z * 0.34 * k * (0.6 + e);
        mesh::radial(&mut m, at, vec2(r * 1.5, r * 1.5), alpha([255, 168, 70], flash * 0.55));
        mesh::radial(&mut m, at, vec2(r, r), alpha([255, 246, 214], flash * 0.9));
    }
    painter.add(Shape::mesh(m));
    // Sparks fly outward, thin at the tip.
    let a = (1.0 - u).powf(1.5);
    for i in 0..10u32 {
        let ang = TAU * i as f32 / 10.0 + 0.5 * hash(i, 2);
        let dir = vec2(ang.cos(), ang.sin() * 0.7);
        let reach = 0.7 + 0.5 * hash(i, 4);
        let (r0, r1) = (0.08 * z * k + 0.35 * z * k * e * reach, 0.1 * z * k + 0.6 * z * k * e * reach);
        painter.line_segment(
            [at + dir * r0, at + dir * r1],
            Stroke::new((z * 0.045 * k).max(1.2), alpha(SPARK, a)),
        );
        painter.circle_filled(at + dir * r1, (z * 0.02 * k).max(0.9), alpha([255, 250, 220], a));
    }
    // A shock ring on the ground.
    if u < 0.55 {
        let rx = z * 0.9 * k * ease_out(u / 0.55);
        painter.add(Shape::ellipse_stroke(
            c + vec2(0.0, 0.2 * z),
            vec2(rx, rx * 0.45),
            Stroke::new((z * 0.05).max(1.0), alpha([255, 236, 190], 0.5 * (1.0 - u / 0.55))),
        ));
    }
}

fn float(painter: &egui::Painter, text: &str, color: [u8; 3], big: bool, u: f32, c: Pos2, z: f32) {
    let rise = 0.95 * z * ease_out(u);
    let size = (z * if big { 0.62 } else { 0.48 }).clamp(13.0, 34.0);
    // Pops a little larger as it appears.
    let pop = 1.0 + 0.45 * (1.0 - ease_out(u * 5.0));
    let a = 1.0 - smooth((u - 0.6) / 0.4);
    let font = FontId::proportional(size * pop);
    let at = c + vec2(0.0, -0.45 * z - rise);
    // A soft halo of the number's own color, then an ink outline all round.
    let mut glow = Mesh::default();
    mesh::radial(&mut glow, at, vec2(size * 1.1, size * 0.75), alpha(color, 0.22 * a));
    painter.add(Shape::mesh(glow));
    let shade = alpha([8, 8, 12], a * 0.92);
    let reach = (size * 0.06).max(1.4);
    for i in 0..8 {
        let ang = TAU * i as f32 / 8.0;
        painter.text(at + vec2(ang.cos(), ang.sin()) * reach, Align2::CENTER_CENTER, text, font.clone(), shade);
    }
    painter.text(at + vec2(0.0, reach * 1.6), Align2::CENTER_CENTER, text, font.clone(), shade);
    painter.text(at, Align2::CENTER_CENTER, text, font, alpha(color, a));
}

fn ring(painter: &egui::Painter, color: Option<[u8; 3]>, size: f32, u: f32, c: Pos2, z: f32) {
    let e = ease_out(u);
    let col = color.unwrap_or(DUST);
    let ground = c + vec2(0.0, 0.2 * z);
    // Two rings, the second trailing the first, over a soft fill.
    for (lag, weight) in [(0.0, 1.0), (0.18, 0.55)] {
        let e = ease_out((u - lag).max(0.0) / (1.0 - lag));
        let rx = z * size * (0.15 + 0.85 * e);
        painter.add(Shape::ellipse_stroke(
            ground,
            vec2(rx, rx * 0.5),
            Stroke::new((z * 0.09 * (1.0 - u)).max(1.0), alpha(col, 0.7 * weight * (1.0 - u))),
        ));
    }
    let rx = z * size * (0.15 + 0.85 * e);
    let mut m = Mesh::default();
    mesh::radial(&mut m, ground, vec2(rx, rx * 0.5), alpha(col, 0.3 * (1.0 - u)));
    painter.add(Shape::mesh(m));
}

fn puff(painter: &egui::Painter, u: f32, c: Pos2, z: f32) {
    let e = ease_out(u);
    let mut m = Mesh::default();
    for i in 0..7 {
        let ang = TAU * i as f32 / 7.0 + 0.5;
        let reach = 0.2 + 0.18 * hash(i, 3);
        let at = c + vec2(ang.cos() * reach * z * e, 0.28 * z + ang.sin() * reach * 0.45 * z * e - 0.2 * z * u);
        mesh::puff(
            &mut m,
            at,
            z * (0.09 + 0.08 * u) * (0.7 + 0.6 * hash(i, 9)),
            alpha(DUST, 0.5 * (1.0 - u)),
        );
    }
    painter.add(Shape::mesh(m));
}

fn sparkle(painter: &egui::Painter, seed: u32, u: f32, c: Pos2, z: f32) {
    let mut m = Mesh::default();
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
        let glint = (v * PI).sin();
        let s = z * 0.2 * glint * (0.7 + 0.5 * hash(seed, i + 40));
        let col = alpha([255, 238, 170], 0.95 * glint);
        let w = s * 0.22;
        mesh::radial(&mut m, at, vec2(s * 1.6, s * 1.6), alpha([255, 224, 140], 0.28 * glint));
        let (hot, mid) = (col, alpha([255, 250, 224], glint));
        mesh::quad(&mut m, (at + vec2(0.0, -s), hot), (at + vec2(w, 0.0), mid), (at + vec2(0.0, s), hot), (at + vec2(-w, 0.0), mid));
        let s2 = s * 0.65;
        mesh::quad(&mut m, (at + vec2(-s2, 0.0), hot), (at + vec2(0.0, w), mid), (at + vec2(s2, 0.0), hot), (at + vec2(0.0, -w), mid));
    }
    painter.add(Shape::mesh(m));
}

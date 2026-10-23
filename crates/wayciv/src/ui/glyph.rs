//! Small vector symbols for the HUD: yields, menu buttons, unit actions and
//! fallbacks for icons that haven't been generated yet. Each glyph is drawn
//! inside a circle of radius `r` around `c`.

use egui::{Color32, Painter, Pos2, Shape, Stroke, Vec2, pos2, vec2};
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Glyph {
    Science,
    Culture,
    Faith,
    Gold,
    Tourism,
    Trade,
    Population,
    Food,
    Production,
    Housing,
    Amenity,
    Loyalty,
    Shield,
    Sleep,
    Skip,
    Cross,
    City,
    Build,
    Boot,
    Swords,
    Heart,
    Bell,
    Crown,
    Temple,
    Star,
    Cup,
    Hex,
    Chip,
    Hourglass,
    Chevrons,
    Arrow,
    Bow,
}

/// Fills a star-shaped outline as a fan of triangles from `c`.
fn fan(painter: &Painter, c: Pos2, pts: &[Pos2], fill: Color32) {
    for i in 0..pts.len() {
        let a = pts[i];
        let b = pts[(i + 1) % pts.len()];
        painter.add(Shape::convex_polygon(vec![c, a, b], fill, Stroke::NONE));
    }
}

fn star(c: Pos2, r_out: f32, r_in: f32, points: usize, rot: f32) -> Vec<Pos2> {
    (0..points * 2)
        .map(|i| {
            let r = if i % 2 == 0 { r_out } else { r_in };
            let a = rot + i as f32 * PI / points as f32;
            c + vec2(a.cos(), a.sin()) * r
        })
        .collect()
}

fn rot_rect(c: Pos2, half: Vec2, angle: f32) -> Vec<Pos2> {
    let (s, co) = angle.sin_cos();
    [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
        .iter()
        .map(|(x, y)| {
            let v = vec2(x * half.x, y * half.y);
            c + vec2(v.x * co - v.y * s, v.x * s + v.y * co)
        })
        .collect()
}

pub fn paint(painter: &Painter, g: Glyph, c: Pos2, r: f32, color: Color32) {
    let p = |x: f32, y: f32| c + vec2(x, y) * r;
    let w = (r * 0.2).max(1.4);
    let stroke = Stroke::new(w, color);
    let dark = Color32::from_black_alpha(110);
    let poly = |pts: Vec<Pos2>| {
        painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
    };
    match g {
        Glyph::Science => {
            poly(vec![
                p(-0.28, -0.85),
                p(0.28, -0.85),
                p(0.28, -0.2),
                p(0.85, 0.8),
                p(-0.85, 0.8),
                p(-0.28, -0.2),
            ]);
            painter.line_segment([p(-0.42, -0.85), p(0.42, -0.85)], stroke);
            painter.circle_filled(p(-0.1, 0.45), r * 0.14, dark);
            painter.circle_filled(p(0.22, 0.28), r * 0.09, dark);
        }
        Glyph::Culture => {
            painter.circle_filled(p(-0.3, 0.55), r * 0.34, color);
            painter.line_segment([p(0.0, 0.5), p(0.0, -0.8)], stroke);
            painter.add(Shape::line(
                vec![p(0.0, -0.8), p(0.35, -0.55), p(0.62, -0.2)],
                stroke,
            ));
        }
        Glyph::Faith => {
            fan(painter, c, &star(c, r * 0.95, r * 0.28, 4, -PI / 2.0), color);
            painter.circle_filled(c, r * 0.14, dark);
        }
        Glyph::Gold => {
            painter.circle_filled(c, r * 0.9, color);
            painter.circle_stroke(c, r * 0.62, Stroke::new(w * 0.7, dark));
            painter.line_segment([p(0.0, -0.3), p(0.0, 0.3)], Stroke::new(w * 0.8, dark));
        }
        Glyph::Tourism => {
            painter.rect_filled(
                egui::Rect::from_center_size(p(0.0, 0.1), vec2(1.7, 1.2) * r),
                r * 0.2,
                color,
            );
            painter.rect_filled(
                egui::Rect::from_center_size(p(-0.3, -0.6), vec2(0.6, 0.3) * r),
                r * 0.08,
                color,
            );
            painter.circle_filled(p(0.0, 0.12), r * 0.38, dark);
            painter.circle_stroke(p(0.0, 0.12), r * 0.38, Stroke::new(w * 0.6, Color32::WHITE));
        }
        Glyph::Trade => {
            painter.line_segment([p(-0.85, -0.35), p(0.8, -0.35)], stroke);
            poly(vec![p(0.9, -0.35), p(0.4, -0.75), p(0.4, 0.05)]);
            painter.line_segment([p(0.85, 0.35), p(-0.8, 0.35)], stroke);
            poly(vec![p(-0.9, 0.35), p(-0.4, -0.05), p(-0.4, 0.75)]);
        }
        Glyph::Population => {
            painter.circle_filled(p(0.0, -0.42), r * 0.34, color);
            let mut pts: Vec<Pos2> = (0..=12)
                .map(|i| {
                    let a = PI + PI * i as f32 / 12.0;
                    p(0.82 * a.cos(), 0.9 + 0.7 * a.sin())
                })
                .collect();
            pts.dedup();
            poly(pts);
        }
        Glyph::Food => {
            painter.line_segment([p(0.0, 0.9), p(0.0, -0.7)], stroke);
            for k in 0..3 {
                let y = -0.45 + 0.4 * k as f32;
                painter.line_segment([p(0.0, y + 0.25), p(-0.5, y - 0.1)], stroke);
                painter.line_segment([p(0.0, y + 0.25), p(0.5, y - 0.1)], stroke);
            }
        }
        Glyph::Production => {
            let pts = star(c, r * 0.95, r * 0.72, 8, 0.0);
            fan(painter, c, &pts, color);
            painter.circle_filled(c, r * 0.34, dark);
        }
        Glyph::Housing => {
            poly(vec![
                p(-0.78, 0.8),
                p(0.78, 0.8),
                p(0.78, -0.05),
                p(0.0, -0.85),
                p(-0.78, -0.05),
            ]);
            painter.rect_filled(
                egui::Rect::from_center_size(p(0.0, 0.5), vec2(0.4, 0.55) * r),
                0.0,
                dark,
            );
        }
        Glyph::Amenity => {
            painter.circle_stroke(c, r * 0.85, stroke);
            painter.circle_filled(p(-0.3, -0.25), r * 0.11, color);
            painter.circle_filled(p(0.3, -0.25), r * 0.11, color);
            let mouth: Vec<Pos2> = (0..=8)
                .map(|i| {
                    let a = 0.2 * PI + 0.6 * PI * i as f32 / 8.0;
                    p(0.5 * a.cos(), 0.5 * a.sin() + 0.05)
                })
                .collect();
            painter.add(Shape::line(mouth, stroke));
        }
        Glyph::Loyalty => {
            painter.line_segment([p(-0.5, 0.9), p(-0.5, -0.85)], stroke);
            poly(vec![p(-0.5, -0.85), p(0.85, -0.4), p(-0.5, 0.1)]);
        }
        Glyph::Shield => {
            poly(vec![
                p(-0.75, -0.8),
                p(0.75, -0.8),
                p(0.75, 0.1),
                p(0.0, 0.92),
                p(-0.75, 0.1),
            ]);
            painter.line_segment([p(0.0, -0.55), p(0.0, 0.5)], Stroke::new(w * 0.6, dark));
        }
        Glyph::Sleep => {
            let z = |x: f32, y: f32, s: f32, wd: f32| {
                painter.add(Shape::line(
                    vec![
                        p(x - s, y - s),
                        p(x + s, y - s),
                        p(x - s, y + s),
                        p(x + s, y + s),
                    ],
                    Stroke::new(wd, color),
                ));
            };
            z(-0.2, 0.3, 0.45, w);
            z(0.45, -0.5, 0.25, w * 0.8);
        }
        Glyph::Skip => {
            for dx in [-0.45, 0.2] {
                painter.add(Shape::line(
                    vec![p(dx, -0.6), p(dx + 0.5, 0.0), p(dx, 0.6)],
                    stroke,
                ));
            }
        }
        Glyph::Cross => {
            painter.line_segment([p(-0.6, -0.6), p(0.6, 0.6)], stroke);
            painter.line_segment([p(-0.6, 0.6), p(0.6, -0.6)], stroke);
        }
        Glyph::City => {
            let base = egui::Rect::from_min_max(p(-0.8, -0.2), p(0.8, 0.85));
            painter.rect_filled(base, 0.0, color);
            for x in [-0.8, -0.27, 0.27] {
                painter.rect_filled(
                    egui::Rect::from_min_size(p(x, -0.7), vec2(0.53, 0.5) * r),
                    0.0,
                    color,
                );
            }
            painter.rect_filled(
                egui::Rect::from_center_size(p(0.0, 0.58), vec2(0.4, 0.55) * r),
                r * 0.2,
                dark,
            );
        }
        Glyph::Build => {
            painter.line_segment([p(-0.7, 0.8), p(0.3, -0.2)], Stroke::new(w * 1.2, color));
            poly(rot_rect(p(0.35, -0.4), vec2(0.62, 0.26) * r, -PI / 4.0));
        }
        Glyph::Boot => {
            painter.line_segment([p(-0.8, 0.0), p(0.5, 0.0)], stroke);
            painter.add(Shape::line(
                vec![p(0.1, -0.5), p(0.7, 0.0), p(0.1, 0.5)],
                stroke,
            ));
        }
        Glyph::Swords => {
            painter.line_segment([p(-0.7, 0.7), p(0.65, -0.65)], stroke);
            painter.line_segment([p(0.7, 0.7), p(-0.65, -0.65)], stroke);
            painter.line_segment([p(-0.75, 0.2), p(-0.2, 0.75)], Stroke::new(w * 0.8, color));
            painter.line_segment([p(0.75, 0.2), p(0.2, 0.75)], Stroke::new(w * 0.8, color));
        }
        Glyph::Heart => {
            painter.circle_filled(p(-0.4, -0.3), r * 0.46, color);
            painter.circle_filled(p(0.4, -0.3), r * 0.46, color);
            poly(vec![p(-0.84, -0.12), p(0.84, -0.12), p(0.0, 0.88)]);
        }
        Glyph::Bell => {
            let mut pts: Vec<Pos2> = (0..=10)
                .map(|i| {
                    let a = PI + PI * i as f32 / 10.0;
                    p(0.52 * a.cos(), -0.15 + 0.62 * a.sin())
                })
                .collect();
            pts.push(p(0.85, 0.6));
            pts.push(p(-0.85, 0.6));
            poly(pts);
            painter.circle_filled(p(0.0, 0.78), r * 0.18, color);
        }
        Glyph::Crown => {
            fan(
                painter,
                p(0.0, 0.6),
                &[
                    p(-0.85, 0.6),
                    p(-0.85, -0.5),
                    p(-0.4, 0.0),
                    p(0.0, -0.75),
                    p(0.4, 0.0),
                    p(0.85, -0.5),
                    p(0.85, 0.6),
                ],
                color,
            );
        }
        Glyph::Temple => {
            poly(vec![p(-0.9, -0.2), p(0.0, -0.85), p(0.9, -0.2)]);
            for x in [-0.6, -0.1, 0.4] {
                painter.rect_filled(
                    egui::Rect::from_min_size(p(x, -0.1), vec2(0.28, 0.75) * r),
                    0.0,
                    color,
                );
            }
            painter.rect_filled(
                egui::Rect::from_min_size(p(-0.9, 0.72), vec2(1.8, 0.2) * r),
                0.0,
                color,
            );
        }
        Glyph::Star => {
            fan(
                painter,
                c,
                &star(c, r * 0.95, r * 0.4, 5, -PI / 2.0),
                color,
            );
        }
        Glyph::Cup => {
            poly(vec![p(-0.55, -0.7), p(0.55, -0.7), p(0.35, 0.2), p(-0.35, 0.2)]);
            painter.rect_filled(
                egui::Rect::from_center_size(p(0.0, 0.45), vec2(0.2, 0.5) * r),
                0.0,
                color,
            );
            painter.rect_filled(
                egui::Rect::from_center_size(p(0.0, 0.8), vec2(0.8, 0.18) * r),
                0.0,
                color,
            );
            for s in [-1.0, 1.0] {
                painter.add(Shape::line(
                    vec![p(0.55 * s, -0.5), p(0.85 * s, -0.4), p(0.35 * s, 0.0)],
                    Stroke::new(w * 0.7, color),
                ));
            }
        }
        Glyph::Hex => {
            let pts: Vec<Pos2> = (0..6)
                .map(|i| {
                    let a = PI / 6.0 + i as f32 * TAU / 6.0;
                    p(0.88 * a.cos(), 0.88 * a.sin())
                })
                .collect();
            painter.add(Shape::closed_line(pts, stroke));
            painter.circle_filled(c, r * 0.22, color);
        }
        Glyph::Chip => {
            painter.rect_stroke(
                egui::Rect::from_center_size(c, vec2(1.2, 1.2) * r),
                r * 0.12,
                stroke,
                egui::StrokeKind::Middle,
            );
            painter.rect_filled(
                egui::Rect::from_center_size(c, vec2(0.5, 0.5) * r),
                0.0,
                color,
            );
            for k in [-0.35, 0.35] {
                for (a, b) in [
                    (p(k, -0.6), p(k, -0.92)),
                    (p(k, 0.6), p(k, 0.92)),
                    (p(-0.6, k), p(-0.92, k)),
                    (p(0.6, k), p(0.92, k)),
                ] {
                    painter.line_segment([a, b], Stroke::new(w * 0.7, color));
                }
            }
        }
        Glyph::Hourglass => {
            poly(vec![p(-0.6, -0.85), p(0.6, -0.85), p(0.0, 0.0)]);
            poly(vec![p(0.0, 0.0), p(0.6, 0.85), p(-0.6, 0.85)]);
        }
        Glyph::Chevrons => {
            for dx in [-0.55, 0.0] {
                painter.add(Shape::line(
                    vec![p(dx, -0.55), p(dx + 0.5, 0.0), p(dx, 0.55)],
                    stroke,
                ));
            }
        }
        Glyph::Arrow => {
            painter.line_segment([p(-0.8, 0.0), p(0.5, 0.0)], stroke);
            poly(vec![p(0.9, 0.0), p(0.35, -0.5), p(0.35, 0.5)]);
        }
        Glyph::Bow => {
            let arc: Vec<Pos2> = (0..=10)
                .map(|i| {
                    let a = -PI / 2.0 + PI * i as f32 / 10.0;
                    p(0.5 * a.cos() - 0.3, 0.9 * a.sin())
                })
                .collect();
            painter.add(Shape::line(arc, stroke));
            painter.line_segment([p(-0.3, -0.9), p(-0.3, 0.9)], Stroke::new(w * 0.4, color));
            painter.line_segment([p(-0.5, 0.0), p(0.8, 0.0)], Stroke::new(w * 0.6, color));
            poly(vec![p(0.95, 0.0), p(0.55, -0.22), p(0.55, 0.22)]);
        }
    }
    let _ = pos2(0.0, 0.0);
}

//! The original flat-colour terrain drawing, kept as the fallback for when
//! the texture assets in `assets/terrain` are missing.

use super::super::TileCtx;
use super::super::hex::{EDGES, corners, hex, world_dir};
use crate::palette;
use egui::{Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, vec2};

/// The flat-colour tile (used when the texture assets are missing).
pub fn draw_tile(painter: &egui::Painter, t: &TileCtx) {
    let (game, c, z) = (t.game, t.c, t.z);
    let rules = &game.rules;
    let content = &rules.content;
    let tile = t.tile();
    let outline = Stroke::new(
        if z > 14.0 { 1.0 } else { 0.5 },
        Color32::from_rgba_unmultiplied(0, 0, 0, 90),
    );
    let terrain = &content.terrains[tile.terrain];
    let mut color = palette::biome(&terrain.id);
    if terrain.hills {
        color = color.gamma_multiply(1.08);
    }
    painter.add(hex(c, z, color, outline));
    if terrain.hills && !terrain.mountain {
        // Two rounded humps, darker than the biome color.
        for (dx, dy, w) in [(-0.3, 0.32, 0.3), (0.28, 0.38, 0.24)] {
            let base = c + vec2(dx * z, dy * z);
            let hump: Vec<Pos2> = (0..=8)
                .map(|i| {
                    let a = std::f32::consts::PI * (1.0 + i as f32 / 8.0);
                    base + vec2(a.cos() * w * z, a.sin() * w * z * 0.8)
                })
                .collect();
            painter.add(Shape::convex_polygon(
                hump,
                color.gamma_multiply(0.72),
                Stroke::new(1.0, color.gamma_multiply(0.5)),
            ));
        }
    }
    if terrain.mountain {
        let peak = vec![
            c + vec2(-0.6 * z, 0.45 * z),
            c + vec2(0.0, -0.55 * z),
            c + vec2(0.6 * z, 0.45 * z),
        ];
        painter.add(Shape::convex_polygon(
            peak,
            Color32::from_rgb(110, 105, 100),
            Stroke::new(1.0, Color32::from_rgb(60, 55, 50)),
        ));
        let cap = vec![
            c + vec2(-0.2 * z, -0.18 * z),
            c + vec2(0.0, -0.55 * z),
            c + vec2(0.2 * z, -0.18 * z),
        ];
        painter.add(Shape::convex_polygon(
            cap,
            Color32::from_rgb(245, 245, 250),
            Stroke::NONE,
        ));
    }
    if let Some(f) = tile.feature {
        draw_feature(
            painter,
            c,
            z,
            &content.features[f].id,
            content.features[f].natural_wonder,
            t.detail().then_some(content.features[f].name.as_str()),
        );
    }
    if tile.cliffs {
        // Cliffs: rocky edges toward the water.
        let pts = corners(c, z * 0.96);
        for dir in 0..6 {
            if t.neighbor(dir).is_some_and(|n| game.map.is_water(rules, n)) {
                let (a, b) = EDGES[dir];
                painter.line_segment(
                    [pts[a], pts[b]],
                    Stroke::new((z * 0.16).max(2.0), Color32::from_rgb(92, 78, 66)),
                );
                let inner = corners(c, z * 0.84);
                painter.line_segment(
                    [inner[a], inner[b]],
                    Stroke::new((z * 0.05).max(1.0), Color32::from_rgb(150, 135, 115)),
                );
            }
        }
    }
}

/// Flat rivers (used when the texture assets are missing).
pub fn draw_river(painter: &egui::Painter, t: &TileCtx) {
    let (game, c, z) = (t.game, t.c, t.z);
    let rules = &game.rules;
    let tile = t.tile();
    if !tile.river {
        return;
    }
    let river = Stroke::new((z * 0.12).max(1.5), Color32::from_rgb(60, 130, 220));
    // Known flow (Earth map): one segment to the downstream neighbor, so
    // rivers draw as clean lines; into the sea they stop at the shore.
    if let Some(dir) = tile.river_dir {
        let dir = dir as usize;
        let reach = if t.neighbor(dir).is_some_and(|n| game.map.is_water(rules, n)) {
            0.5
        } else {
            1.0
        };
        painter.line_segment([c, c + world_dir(dir) * z * reach], river);
        return;
    }
    let mut drew = false;
    for dir in 0..6 {
        if let Some(n) = t.neighbor(dir)
            && (game.map.tile(n).river || game.map.is_water(rules, n))
        {
            let off = world_dir(dir) * z * 0.5;
            painter.line_segment([c, c + off], river);
            drew = true;
        }
    }
    if !drew {
        painter.circle_filled(c, z * 0.1, Color32::from_rgb(60, 130, 220));
    }
}

/// A flat sample of a biome (and feature) for the map key.
pub fn legend_swatch(painter: &egui::Painter, terrain_id: &str, feature_id: Option<&str>, rect: Rect) {
    painter.rect_filled(rect, 3.0, palette::biome(terrain_id));
    if let Some(feature) = feature_id {
        draw_feature(painter, rect.center(), rect.height() * 0.55, feature, false, None);
    }
}

pub fn draw_feature(
    painter: &egui::Painter,
    c: Pos2,
    z: f32,
    id: &str,
    natural_wonder: bool,
    label: Option<&str>,
) {
    if natural_wonder {
        let star: Vec<Pos2> = (0..10)
            .map(|i| {
                let r = if i % 2 == 0 { 0.55 } else { 0.24 } * z;
                let a = (-90.0 + 36.0 * i as f32).to_radians();
                c + vec2(a.cos(), a.sin()) * r
            })
            .collect();
        painter.add(Shape::closed_line(
            star.clone(),
            Stroke::new(2.0, Color32::from_rgb(120, 80, 0)),
        ));
        painter.add(Shape::convex_polygon(
            vec![
                c + vec2(0.0, -0.5 * z),
                c + vec2(0.45 * z, 0.35 * z),
                c + vec2(-0.45 * z, 0.35 * z),
            ],
            Color32::from_rgba_unmultiplied(255, 210, 60, 140),
            Stroke::NONE,
        ));
        if let Some(label) = label {
            painter.text(
                c + vec2(0.0, 0.7 * z),
                Align2::CENTER_TOP,
                label,
                FontId::proportional(z * 0.3),
                Color32::from_rgb(255, 230, 140),
            );
        }
        return;
    }
    match id {
        "FEATURE_FOREST" => {
            for (dx, dy) in [(-0.35, 0.05), (0.0, -0.3), (0.35, 0.05), (0.0, 0.35)] {
                let base = c + vec2(dx * z, dy * z);
                let tree = vec![
                    base + vec2(0.0, -0.22 * z),
                    base + vec2(0.16 * z, 0.12 * z),
                    base + vec2(-0.16 * z, 0.12 * z),
                ];
                painter.add(Shape::convex_polygon(
                    tree,
                    Color32::from_rgb(30, 90, 35),
                    Stroke::new(1.0, Color32::from_rgb(15, 50, 20)),
                ));
            }
        }
        "FEATURE_JUNGLE" => {
            for (dx, dy) in [
                (-0.35, -0.1),
                (0.05, -0.35),
                (0.35, 0.0),
                (-0.1, 0.3),
                (0.3, 0.35),
            ] {
                painter.circle(
                    c + vec2(dx * z, dy * z),
                    z * 0.17,
                    Color32::from_rgb(25, 110, 70),
                    Stroke::new(1.0, Color32::from_rgb(10, 60, 40)),
                );
            }
        }
        "FEATURE_MARSH" => {
            for dy in [-0.25, 0.05, 0.35] {
                painter.line_segment(
                    [c + vec2(-0.4 * z, dy * z), c + vec2(0.4 * z, dy * z)],
                    Stroke::new((z * 0.07).max(1.0), Color32::from_rgb(60, 120, 170)),
                );
            }
        }
        "FEATURE_FLOODPLAINS" => {
            // Desert floodplains: a green, fertile strip on the sand.
            painter.add(Shape::convex_polygon(
                vec![
                    c + vec2(-0.6 * z, -0.12 * z),
                    c + vec2(0.6 * z, -0.28 * z),
                    c + vec2(0.6 * z, 0.08 * z),
                    c + vec2(-0.6 * z, 0.24 * z),
                ],
                Color32::from_rgb(120, 165, 70),
                Stroke::NONE,
            ));
            for dy in [-0.32, 0.36] {
                painter.line_segment(
                    [c + vec2(-0.45 * z, dy * z), c + vec2(0.45 * z, dy * z)],
                    Stroke::new((z * 0.05).max(1.0), Color32::from_rgb(140, 120, 70)),
                );
            }
        }
        "FEATURE_FLOODPLAINS_GRASSLAND" | "FEATURE_FLOODPLAINS_PLAINS" => {
            for dy in [-0.25, 0.0, 0.25] {
                painter.line_segment(
                    [
                        c + vec2(-0.45 * z, dy * z),
                        c + vec2(0.45 * z, (dy + 0.08) * z),
                    ],
                    Stroke::new((z * 0.06).max(1.0), Color32::from_rgb(120, 105, 60)),
                );
            }
        }
        "FEATURE_TAIGA" => {
            // Narrow, dark conifers.
            for (dx, dy) in [
                (-0.38, 0.1),
                (-0.12, -0.25),
                (0.14, 0.12),
                (0.38, -0.2),
                (0.0, 0.42),
            ] {
                let base = c + vec2(dx * z, dy * z);
                let tree = vec![
                    base + vec2(0.0, -0.28 * z),
                    base + vec2(0.1 * z, 0.12 * z),
                    base + vec2(-0.1 * z, 0.12 * z),
                ];
                painter.add(Shape::convex_polygon(
                    tree,
                    Color32::from_rgb(28, 78, 66),
                    Stroke::new(1.0, Color32::from_rgb(12, 40, 34)),
                ));
            }
        }
        "FEATURE_DUNES" => {
            // Wind-shaped sand ridges.
            for (dx, dy) in [(-0.2, -0.25), (0.15, 0.05), (-0.15, 0.35)] {
                let arc: Vec<Pos2> = (0..=8)
                    .map(|i| {
                        let t = i as f32 / 8.0;
                        c + vec2(
                            (dx - 0.3 + 0.6 * t) * z,
                            (dy - 0.12 * (std::f32::consts::PI * t).sin()) * z,
                        )
                    })
                    .collect();
                painter.add(Shape::line(
                    arc,
                    Stroke::new((z * 0.07).max(1.0), Color32::from_rgb(190, 145, 70)),
                ));
            }
        }
        "FEATURE_GLACIER" => {
            painter.add(hex(
                c,
                z * 0.9,
                Color32::from_rgb(232, 242, 252),
                Stroke::new(1.0, Color32::from_rgb(180, 205, 235)),
            ));
            for (a, b) in [
                ((-0.4, -0.1), (0.1, 0.05)),
                ((0.0, 0.3), (0.35, 0.15)),
                ((-0.1, -0.4), (0.2, -0.3)),
            ] {
                painter.line_segment(
                    [c + vec2(a.0 * z, a.1 * z), c + vec2(b.0 * z, b.1 * z)],
                    Stroke::new((z * 0.04).max(1.0), Color32::from_rgb(140, 175, 215)),
                );
            }
        }
        "FEATURE_OASIS" => {
            painter.circle_filled(
                c + vec2(0.0, 0.1 * z),
                z * 0.25,
                Color32::from_rgb(60, 150, 210),
            );
            painter.circle_filled(
                c + vec2(0.18 * z, -0.15 * z),
                z * 0.12,
                Color32::from_rgb(40, 130, 50),
            );
        }
        "FEATURE_ICE" => {
            painter.add(hex(
                c,
                z * 0.85,
                Color32::from_rgba_unmultiplied(235, 245, 255, 200),
                Stroke::new(1.0, Color32::from_rgb(170, 200, 230)),
            ));
        }
        _ => {}
    }
}

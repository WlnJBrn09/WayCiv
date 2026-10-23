//! Resources, tile improvements, roads, and barbarian camps.

use super::TileCtx;
use super::hex::{abbreviation, world_dir};
use crate::icons::Icons;
use egui::{Align2, Color32, FontId, Rect, Shape, Stroke, vec2};
use std::path::PathBuf;

pub struct ResourceArt {
    #[allow(dead_code)]
    assets: PathBuf,
}

impl ResourceArt {
    pub fn new(assets: PathBuf) -> Self {
        Self { assets }
    }
}

/// Layer 2b: roads, toward neighboring roads and cities.
pub fn draw_road(_art: &mut ResourceArt, painter: &egui::Painter, t: &TileCtx) {
    let (game, c, z) = (t.game, t.c, t.z);
    if !t.tile().road {
        return;
    }
    let road = |p: wayciv_core::Pos| game.map.tile(p).road || game.city_at(p).is_some();
    for dir in 0..6 {
        if t.neighbor(dir).is_some_and(road) {
            let off = world_dir(dir) * z * 0.5;
            painter.line_segment(
                [c, c + off],
                Stroke::new((z * 0.1).max(1.5), Color32::from_rgb(150, 105, 55)),
            );
        }
    }
}

/// Layer 4: a tile improvement (farm, mine, ...) or a barbarian camp.
pub fn draw_improvement(_art: &mut ResourceArt, painter: &egui::Painter, t: &TileCtx) {
    let (game, pos, c, z) = (t.game, t.pos, t.c, t.z);
    let content = &game.rules.content;
    let tile = t.tile();
    let detail = t.detail();
    if game.camps.contains_key(&pos) {
        // Barbarian camp: a red tent.
        let tent = vec![
            c + vec2(0.0, -0.45 * z),
            c + vec2(0.45 * z, 0.35 * z),
            c + vec2(-0.45 * z, 0.35 * z),
        ];
        painter.add(Shape::convex_polygon(
            tent,
            Color32::from_rgb(190, 45, 35),
            Stroke::new(2.0, Color32::BLACK),
        ));
        painter.line_segment(
            [c + vec2(0.0, -0.1 * z), c + vec2(0.0, 0.35 * z)],
            Stroke::new(2.0, Color32::BLACK),
        );
    } else if let Some(i) = tile.improvement
        && detail
    {
        let name = &content.improvements[i].name;
        let label = abbreviation(name);
        let at = c + vec2(0.0, 0.55 * z);
        painter.rect_filled(
            Rect::from_center_size(at, vec2(z * 0.7, z * 0.32)),
            3.0,
            Color32::from_rgba_unmultiplied(70, 45, 20, 220),
        );
        painter.text(
            at,
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(z * 0.24),
            Color32::from_rgb(240, 220, 180),
        );
    }
}

/// Layer 4, after improvements: a resource the viewer can see.
pub fn draw_resource(
    _art: &mut ResourceArt,
    icons: &mut Icons,
    painter: &egui::Painter,
    t: &TileCtx,
) {
    let (game, c, z) = (t.game, t.c, t.z);
    let rules = &game.rules;
    let content = &rules.content;
    let player = &game.players[t.viewer];
    let tile = t.tile();
    let detail = t.detail();
    if let Some(r) = tile.resource {
        let res = &content.resources[r];
        if player.meets(rules, res.prereq_tech.as_deref(), None) && z > 10.0 {
            let color = match res.class.as_str() {
                "RESOURCECLASS_LUXURY" => Color32::from_rgb(190, 90, 220),
                "RESOURCECLASS_STRATEGIC" => Color32::from_rgb(220, 70, 60),
                _ => Color32::from_rgb(80, 190, 80),
            };
            let at = c + vec2(0.42 * z, -0.42 * z);
            let radius = if z > 16.0 { z * 0.27 } else { z * 0.2 };
            let drew = z > 16.0
                && icons.paint(
                    painter,
                    "resource",
                    &res.id,
                    Rect::from_center_size(at, vec2(radius * 2.0, radius * 2.0)),
                );
            if drew {
                painter.circle_stroke(at, radius, Stroke::new((z * 0.06).max(1.5), color));
            } else {
                painter.circle(at, z * 0.2, color, Stroke::new(1.0, Color32::BLACK));
                if detail {
                    painter.text(
                        at,
                        Align2::CENTER_CENTER,
                        abbreviation(&res.name),
                        FontId::proportional(z * 0.2),
                        Color32::WHITE,
                    );
                }
            }
        }
    }
}

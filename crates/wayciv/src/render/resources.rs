//! Resources, tile improvements, roads, and barbarian camps.
//!
//! Sprites are rendered offline (tools/resources) into
//! `assets/resources/{sprites,icons,improvements}/`. Textures load on first
//! use; anything missing falls back to simple painted shapes.

use super::TileCtx;
use super::hex::{abbreviation, hex, world_dir};
use crate::icons::Icons;
use egui::epaint::{Mesh, Vertex};
use egui::{
    Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, TextureHandle, TextureOptions, pos2, vec2,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Width of a 3/4 sprite in hex radii (the Blender ortho scale).
const SPRITE: f32 = 1.7;
/// Width of a top-down field decal in hex radii.
const DECAL: f32 = 2.0;

struct Tex {
    handle: TextureHandle,
    /// Average colour of the opaque pixels, for tiny-zoom markers.
    avg: Color32,
}

pub struct ResourceArt {
    assets: PathBuf,
    cache: HashMap<String, Option<Tex>>,
}

impl ResourceArt {
    pub fn new(assets: PathBuf) -> Self {
        Self {
            assets,
            cache: HashMap::new(),
        }
    }

    /// `kind` is the folder under assets/resources, `name` the file stem.
    fn tex(&mut self, ctx: &egui::Context, kind: &str, name: &str) -> Option<&Tex> {
        let key = format!("{kind}/{name}");
        if !self.cache.contains_key(&key) {
            let path = self
                .assets
                .join("resources")
                .join(kind)
                .join(format!("{name}.png"));
            let tex = image::open(path).ok().map(|img| {
                let img = img.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let (mut r, mut g, mut b, mut n) = (0u64, 0u64, 0u64, 0u64);
                for p in img.pixels() {
                    if p[3] > 200 {
                        r += p[0] as u64;
                        g += p[1] as u64;
                        b += p[2] as u64;
                        n += 1;
                    }
                }
                let n = n.max(1);
                Tex {
                    handle: ctx.load_texture(
                        format!("resources/{key}"),
                        egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()),
                        TextureOptions::LINEAR,
                    ),
                    avg: Color32::from_rgb((r / n) as u8, (g / n) as u8, (b / n) as u8),
                }
            });
            self.cache.insert(key.clone(), tex);
        }
        self.cache.get(&key).and_then(|t| t.as_ref())
    }
}

fn hash(t: &TileCtx) -> u32 {
    (t.pos.x as u32)
        .wrapping_mul(73_856_093)
        .wrapping_add((t.pos.y as u32).wrapping_mul(19_349_663))
        .rotate_left(7)
        .wrapping_mul(2_654_435_761)
}

/// An image centered on `at`, `size` px wide, rotated by `turns` * 60 degrees.
fn blit(painter: &egui::Painter, tex: &Tex, at: Pos2, size: f32, tint: Color32, turns: u32) {
    if turns == 0 {
        painter.image(
            tex.handle.id(),
            Rect::from_center_size(at, vec2(size, size)),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            tint,
        );
        return;
    }
    let ang = (turns as f32) * std::f32::consts::FRAC_PI_3;
    let (s, c) = ang.sin_cos();
    let h = size / 2.0;
    let mut mesh = Mesh::with_texture(tex.handle.id());
    for (dx, dy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        mesh.vertices.push(Vertex {
            pos: at + vec2(dx * h * c - dy * h * s, dx * h * s + dy * h * c),
            uv: pos2((dx + 1.0) / 2.0, (dy + 1.0) / 2.0),
            color: tint,
        });
    }
    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    painter.add(Shape::mesh(mesh));
}

// ---------------------------------------------------------------- roads

#[derive(Clone, Copy, PartialEq)]
enum RoadKind {
    Dirt,
    Paved,
    Rail,
}

/// Layer 2b: roads, through the tile center toward neighboring roads and
/// cities. Style follows the viewer's technology.
pub fn draw_road(_art: &mut ResourceArt, painter: &egui::Painter, t: &TileCtx) {
    let (game, c, z) = (t.game, t.c, t.z);
    if !t.tile().road {
        return;
    }
    let rules = &game.rules;
    let player = &game.players[t.viewer];
    let kind = if player.has_tech(rules, "TECH_INDUSTRIALIZATION") {
        RoadKind::Rail
    } else if player.has_tech(rules, "TECH_ENGINEERING")
        || player.has_tech(rules, "TECH_CONSTRUCTION")
    {
        RoadKind::Paved
    } else {
        RoadKind::Dirt
    };
    let road = |p: wayciv_core::Pos| game.map.tile(p).road || game.city_at(p).is_some();
    let ends: Vec<Pos2> = (0..6)
        .filter(|&dir| t.neighbor(dir).is_some_and(road))
        .map(|dir| c + world_dir(dir) * z * 0.5)
        .collect();
    let w = (z * 0.14).max(1.8);

    // Polylines through the center; two ends bend smoothly through it.
    let mut paths: Vec<Vec<Pos2>> = Vec::new();
    match ends.len() {
        0 => {}
        2 => {
            let (a, b) = (ends[0], ends[1]);
            let n = 8;
            paths.push(
                (0..=n)
                    .map(|i| {
                        let t = i as f32 / n as f32;
                        let u = 1.0 - t;
                        // Quadratic bezier with the control point at the center.
                        (a.to_vec2() * (u * u)
                            + c.to_vec2() * (2.0 * u * t)
                            + b.to_vec2() * (t * t))
                            .to_pos2()
                    })
                    .collect(),
            );
        }
        _ => {
            for e in &ends {
                paths.push(vec![c, *e]);
            }
        }
    }
    let hub = ends.len() != 2;
    let (edge, body, mid) = match kind {
        RoadKind::Dirt => (
            Color32::from_rgba_unmultiplied(86, 58, 32, 210),
            Color32::from_rgb(178, 138, 86),
            Color32::from_rgba_unmultiplied(206, 170, 118, 150),
        ),
        RoadKind::Paved => (
            Color32::from_rgba_unmultiplied(70, 66, 62, 220),
            Color32::from_rgb(168, 160, 146),
            Color32::from_rgba_unmultiplied(214, 206, 188, 170),
        ),
        RoadKind::Rail => (
            Color32::from_rgba_unmultiplied(60, 56, 52, 220),
            Color32::from_rgb(120, 112, 102),
            Color32::from_rgb(70, 50, 34),
        ),
    };
    let (ew, bw) = (w * 1.45, w);
    for p in &paths {
        painter.add(Shape::line(p.clone(), Stroke::new(ew, edge)));
    }
    if hub {
        painter.circle_filled(c, ew / 2.0, edge);
    }
    for p in &paths {
        painter.add(Shape::line(p.clone(), Stroke::new(bw, body)));
    }
    if hub {
        painter.circle_filled(c, bw / 2.0, body);
    }
    if z < 18.0 {
        return;
    }
    match kind {
        RoadKind::Dirt | RoadKind::Paved => {
            for p in &paths {
                painter.add(Shape::line(p.clone(), Stroke::new(bw * 0.3, mid)));
            }
        }
        RoadKind::Rail => {
            for p in &paths {
                // Sleepers every few pixels, then two steel rails.
                let pts = resample(p, (z * 0.12).max(3.0));
                for pair in pts.windows(2) {
                    let d = (pair[1] - pair[0]).normalized();
                    let n = vec2(-d.y, d.x) * bw * 0.62;
                    painter.line_segment(
                        [pair[0] - n, pair[0] + n],
                        Stroke::new((z * 0.04).max(1.0), mid),
                    );
                }
                for side in [-1.0, 1.0] {
                    let off: Vec<Pos2> = pts
                        .windows(2)
                        .flat_map(|pair| {
                            let d = (pair[1] - pair[0]).normalized();
                            let n = vec2(-d.y, d.x) * bw * 0.26 * side;
                            [pair[0] + n, pair[1] + n]
                        })
                        .collect();
                    painter.add(Shape::line(
                        off,
                        Stroke::new((z * 0.025).max(0.8), Color32::from_rgb(214, 216, 222)),
                    ));
                }
            }
        }
    }
}

/// Points along a polyline roughly `step` apart.
fn resample(p: &[Pos2], step: f32) -> Vec<Pos2> {
    let mut out = vec![p[0]];
    let mut next = step;
    let mut walked = 0.0;
    for pair in p.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let len = a.distance(b);
        while next <= walked + len {
            out.push(a + (b - a) * ((next - walked) / len.max(1e-3)));
            next += step;
        }
        walked += len;
    }
    if let Some(&last) = p.last()
        && out.last() != Some(&last)
    {
        out.push(last);
    }
    out
}

// --------------------------------------------------------- improvements

#[derive(Clone, Copy, PartialEq)]
enum Layout {
    /// Covers the whole tile (farm, plantation): a decal.
    Field,
    /// A ring around the middle (pasture): the resource stays centered.
    Ring,
    /// A structure: the resource moves to the side.
    Side,
}

fn sprite_name(id: &str) -> String {
    let lower = id.to_lowercase();
    if lower == "improvement_barbarian_camp" {
        return "barbarian_camp".into();
    }
    lower.replacen("improvement_", "imp_", 1)
}

fn layout_of(name: &str) -> Layout {
    match name {
        "imp_farm" | "imp_plantation" => Layout::Field,
        "imp_pasture" | "imp_pairidaeza" => Layout::Ring,
        _ => Layout::Side,
    }
}

/// The improvement (or camp) on this tile and its sprite name.
fn improvement_of(t: &TileCtx) -> Option<String> {
    if t.game.camps.contains_key(&t.pos) {
        return Some("barbarian_camp".into());
    }
    t.tile()
        .improvement
        .map(|i| sprite_name(&t.game.rules.content.improvements[i].id))
}

fn smoke(painter: &egui::Painter, at: Pos2, z: f32, seed: u32) {
    for k in 0..4u32 {
        let wob = ((seed.wrapping_add(k * 97) % 100) as f32 / 100.0 - 0.5) * 0.12;
        let p = at + vec2(wob * z + k as f32 * 0.04 * z, -(0.12 + 0.16 * k as f32) * z);
        let a = (110 - 22 * k) as u8;
        painter.circle_filled(
            p,
            z * (0.07 + 0.035 * k as f32),
            Color32::from_rgba_unmultiplied(70, 66, 64, a),
        );
    }
}

/// Layer 4: a tile improvement (farm, mine, ...) or a barbarian camp.
pub fn draw_improvement(art: &mut ResourceArt, painter: &egui::Painter, t: &TileCtx) {
    let (c, z) = (t.c, t.z);
    let Some(name) = improvement_of(t) else {
        return;
    };
    let tile = t.tile();
    let camp = name == "barbarian_camp";
    let pillaged = tile.pillaged && !camp;
    let layout = layout_of(&name);
    let has_resource = tile.resource.is_some();
    let ctx = painter.ctx().clone();
    let h = hash(t);

    // Tiny zoom: a flat patch or dot of the sprite's average color.
    if z < 14.0 {
        let avg = art
            .tex(&ctx, "improvements", &name)
            .map(|x| x.avg)
            .unwrap_or(Color32::from_rgb(140, 100, 60));
        if layout == Layout::Field {
            painter.add(hex(
                c,
                z * 0.85,
                Color32::from_rgba_unmultiplied(avg.r(), avg.g(), avg.b(), 190),
                Stroke::NONE,
            ));
        } else {
            let col = if camp {
                Color32::from_rgb(190, 40, 34)
            } else {
                avg
            };
            painter.circle(
                c + vec2(0.0, 0.2 * z),
                z * 0.22,
                col,
                Stroke::new(1.0, Color32::BLACK),
            );
        }
        return;
    }

    if camp {
        // A faint red warning ring so camps read at a glance.
        painter.add(hex(
            c,
            z * 0.9,
            Color32::from_rgba_unmultiplied(170, 30, 28, 36),
            Stroke::new((z * 0.04).max(1.0), Color32::from_rgba_unmultiplied(200, 40, 34, 150)),
        ));
    }
    let tint = if pillaged {
        Color32::from_rgb(150, 138, 130)
    } else {
        Color32::WHITE
    };
    // Where the structure sits when a resource shares the tile.
    let side = layout == Layout::Side && has_resource;
    let at = if side { c + vec2(-0.2 * z, 0.08 * z) } else { c };
    let scale = if side { 0.86 } else { 1.0 };
    let drew = if let Some(tex) = art.tex(&ctx, "improvements", &name) {
        match layout {
            Layout::Field => blit(painter, tex, c, DECAL * z, tint, h % 6),
            _ => blit(painter, tex, at, SPRITE * z * scale, tint, 0),
        }
        true
    } else if let Some(tex) = art.tex(&ctx, "improvements", "imp_generic") {
        blit(painter, tex, at, SPRITE * z * scale, tint, 0);
        true
    } else {
        false
    };
    if !drew {
        fallback_improvement(painter, t, &name, camp);
    }
    if pillaged {
        smoke(painter, at, z, h);
        // A few cracks across the site.
        let dark = Stroke::new(
            (z * 0.04).max(1.0),
            Color32::from_rgba_unmultiplied(30, 20, 14, 200),
        );
        painter.line_segment(
            [at + vec2(-0.3 * z, 0.1 * z), at + vec2(0.0, 0.25 * z)],
            dark,
        );
        painter.line_segment(
            [at + vec2(0.0, 0.25 * z), at + vec2(0.28 * z, 0.05 * z)],
            dark,
        );
    }
}

/// Missing art: a plain label (or a red tent for a camp).
fn fallback_improvement(painter: &egui::Painter, t: &TileCtx, name: &str, camp: bool) {
    let (c, z) = (t.c, t.z);
    if camp {
        painter.add(Shape::convex_polygon(
            vec![
                c + vec2(0.0, -0.45 * z),
                c + vec2(0.45 * z, 0.35 * z),
                c + vec2(-0.45 * z, 0.35 * z),
            ],
            Color32::from_rgb(190, 45, 35),
            Stroke::new(2.0, Color32::BLACK),
        ));
        return;
    }
    let at = c + vec2(0.0, 0.55 * z);
    painter.rect_filled(
        Rect::from_center_size(at, vec2(z * 0.7, z * 0.32)),
        3.0,
        Color32::from_rgba_unmultiplied(70, 45, 20, 220),
    );
    if t.detail() {
        painter.text(
            at,
            Align2::CENTER_CENTER,
            abbreviation(name.trim_start_matches("imp_")),
            FontId::proportional(z * 0.24),
            Color32::from_rgb(240, 220, 180),
        );
    }
}

// ------------------------------------------------------------ resources

fn class_color(class: &str) -> Color32 {
    match class {
        "RESOURCECLASS_LUXURY" => Color32::from_rgb(178, 98, 214),
        "RESOURCECLASS_STRATEGIC" => Color32::from_rgb(226, 80, 68),
        "RESOURCECLASS_ARTIFACT" => Color32::from_rgb(226, 184, 84),
        _ => Color32::from_rgb(90, 190, 84),
    }
}

/// Layer 4, after improvements: a resource the viewer can see.
pub fn draw_resource(
    art: &mut ResourceArt,
    icons: &mut Icons,
    painter: &egui::Painter,
    t: &TileCtx,
) {
    let (game, c, z) = (t.game, t.c, t.z);
    let rules = &game.rules;
    let content = &rules.content;
    let player = &game.players[t.viewer];
    let tile = t.tile();
    let Some(r) = tile.resource else {
        return;
    };
    let res = &content.resources[r];
    if !player.meets(rules, res.prereq_tech.as_deref(), None) || z <= 8.0 {
        return;
    }
    let color = class_color(&res.class);
    // Kept small and low-key: a little model on the tile and a small,
    // slightly see-through badge tucked into the corner.
    let corner = c + vec2(0.5 * z, -0.42 * z);

    // Tiny zoom: just a small class dot.
    if z < 14.0 {
        painter.circle(corner, z * 0.12, color.gamma_multiply(0.85), Stroke::new(0.8, Color32::from_black_alpha(160)));
        return;
    }

    let ctx = painter.ctx().clone();
    let name = res.id.to_lowercase();
    // The model, unless a city or wonder fills the tile.
    let crowded =
        tile.district.is_some() || tile.wonder.is_some() || game.city_at(t.pos).is_some();
    if !crowded {
        let (at, scale) = match improvement_of(t).map(|n| layout_of(&n)) {
            None => (c + vec2(0.0, 0.12 * z), 0.6),
            Some(Layout::Field) => (c + vec2(0.0, 0.08 * z), 0.5),
            Some(Layout::Ring) => (c + vec2(0.0, 0.1 * z), 0.45),
            Some(Layout::Side) => (c + vec2(0.24 * z, -0.04 * z), 0.42),
        };
        if let Some(tex) = art.tex(&ctx, "sprites", &name) {
            blit(painter, tex, at, SPRITE * z * scale, Color32::WHITE, 0);
        }
    }

    // The class badge in the corner.
    let radius = (z * 0.15).max(5.0);
    let badge = Rect::from_center_size(corner, vec2(radius * 2.0, radius * 2.0));
    let tint = Color32::from_white_alpha(215);
    if let Some(tex) = art.tex(&ctx, "icons", &name) {
        painter.image(
            tex.handle.id(),
            badge,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            tint,
        );
        return;
    }
    if z > 16.0 && icons.paint(painter, "resource", &res.id, badge) {
        painter.circle_stroke(corner, radius, Stroke::new((z * 0.035).max(1.0), color.gamma_multiply(0.8)));
    } else {
        painter.circle(corner, z * 0.12, color.gamma_multiply(0.85), Stroke::new(0.8, Color32::from_black_alpha(160)));
        if t.detail() {
            painter.text(
                corner,
                Align2::CENTER_CENTER,
                abbreviation(&res.name),
                FontId::proportional(z * 0.12),
                Color32::WHITE,
            );
        }
    }
}

//! Resources, tile improvements, roads, and barbarian camps.
//!
//! Sprites are rendered offline (tools/resources) into
//! `assets/resources/{sprites,icons,improvements}/`. Textures load on first
//! use; anything missing falls back to simple painted shapes.

use super::TileCtx;
use super::hex::{abbreviation, world_dir};
use super::mesh;
use crate::icons::Icons;
use egui::epaint::{Mesh, Vertex};
use egui::{
    Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, TextureHandle, TextureOptions, Vec2, pos2, vec2,
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

/// Colors of one road surface.
struct Surface {
    /// Dark verge, soft at the outside.
    verge: Color32,
    /// Worn shoulder between verge and body.
    shoulder: Color32,
    body: Color32,
    /// Ruts, joints and other marks along the body.
    mark: Color32,
}

fn surface(kind: RoadKind) -> Surface {
    match kind {
        RoadKind::Dirt => Surface {
            verge: Color32::from_rgb(96, 70, 40),
            shoulder: Color32::from_rgb(150, 118, 74),
            body: Color32::from_rgb(188, 154, 104),
            mark: Color32::from_rgb(132, 98, 58),
        },
        RoadKind::Paved => Surface {
            verge: Color32::from_rgb(78, 66, 54),
            shoulder: Color32::from_rgb(146, 130, 108),
            body: Color32::from_rgb(190, 174, 148),
            mark: Color32::from_rgb(104, 90, 74),
        },
        RoadKind::Rail => Surface {
            verge: Color32::from_rgb(56, 52, 48),
            shoulder: Color32::from_rgb(104, 98, 90),
            body: Color32::from_rgb(132, 124, 112),
            mark: Color32::from_rgb(66, 46, 30),
        },
    }
}

fn road_kind(t: &TileCtx) -> RoadKind {
    let rules = &t.game.rules;
    let player = &t.game.players[t.viewer];
    if player.has_tech(rules, "TECH_INDUSTRIALIZATION") {
        RoadKind::Rail
    } else if player.has_tech(rules, "TECH_ENGINEERING")
        || player.has_tech(rules, "TECH_CONSTRUCTION")
    {
        RoadKind::Paved
    } else {
        RoadKind::Dirt
    }
}

/// Smooth value noise in 0..1 along `u`, keyed by `seed`.
fn wobble(seed: u32, u: f32) -> f32 {
    let h = |k: u32| {
        (seed
            .wrapping_add(k.wrapping_mul(2_246_822_519))
            .wrapping_mul(2_654_435_761)
            >> 8) as f32
            / 16_777_216.0
    };
    let (i, f) = (u.floor(), u.fract());
    let f = f * f * (3.0 - 2.0 * f);
    h(i as u32) * (1.0 - f) + h(i as u32 + 1) * f
}

/// Unit normals along a polyline.
fn normals(pts: &[Pos2]) -> Vec<Vec2> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let d = (pts[(i + 1).min(n - 1)] - pts[i.saturating_sub(1)]).normalized();
            vec2(-d.y, d.x)
        })
        .collect()
}

/// A ribbon along `pts` with a color for each lateral offset (in px) of
/// `profile`; transparent rim colors give soft edges. `tone` scales the
/// brightness along the path (0 to 1).
fn ribbon(mesh: &mut Mesh, pts: &[Pos2], profile: &[(f32, Color32)], tone: &dyn Fn(f32) -> f32) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    let norms = normals(pts);
    let base = mesh.vertices.len() as u32;
    let across = profile.len() as u32;
    for (i, &p) in pts.iter().enumerate() {
        let k = tone(i as f32 / (n - 1) as f32);
        for &(off, col) in profile {
            let lit = |v: u8| (v as f32 * k).min(255.0) as u8;
            let col = Color32::from_rgba_premultiplied(lit(col.r()), lit(col.g()), lit(col.b()), col.a());
            mesh.colored_vertex(p + norms[i] * off, col);
        }
    }
    for i in 0..(n as u32 - 1) {
        for j in 0..across - 1 {
            let a = base + i * across + j;
            let b = base + (i + 1) * across + j;
            mesh.add_triangle(a, b, b + 1);
            mesh.add_triangle(a, b + 1, a + 1);
        }
    }
}

/// `pts` pushed sideways by `off(i)` px.
fn offset(pts: &[Pos2], off: impl Fn(usize) -> f32) -> Vec<Pos2> {
    let norms = normals(pts);
    pts.iter()
        .enumerate()
        .map(|(i, &p)| p + norms[i] * off(i))
        .collect()
}

/// Layer 2b: roads, through the tile center toward neighboring roads and
/// cities. Style follows the viewer's technology. Tiles with a field draw
/// theirs after the field instead (see `draw_improvement`).
pub fn draw_road(_art: &mut ResourceArt, painter: &egui::Painter, t: &TileCtx) {
    if improvement_of(t).is_some_and(|n| layout_of(&n) == Layout::Field) {
        return;
    }
    paint_road(painter, t);
}

fn paint_road(painter: &egui::Painter, t: &TileCtx) {
    let (game, c, z) = (t.game, t.c, t.z);
    if !t.tile().road {
        return;
    }
    let kind = road_kind(t);
    let road = |p: wayciv_core::Pos| game.map.tile(p).road || game.city_at(p).is_some();
    let dirs: Vec<usize> = (0..6)
        .filter(|&dir| t.neighbor(dir).is_some_and(road))
        .collect();
    if dirs.is_empty() {
        return;
    }
    let seed = hash(t);
    let w = (z * 0.105).max(1.6);
    let sf = surface(kind);
    let fade = |c: Color32, k: f32| c.gamma_multiply(k);
    let clear = Color32::TRANSPARENT;

    // Each spoke leaves the center and reaches the edge midpoint, bending a
    // little in between; the bend vanishes at the edge, so neighbors join up.
    let spoke = |dir: usize| -> Vec<Pos2> {
        let end = c + world_dir(dir) * z * 0.5;
        let bend = (wobble(seed.wrapping_add(dir as u32 * 31), 0.5) - 0.5) * 0.34 * z;
        let n = 10;
        let line: Vec<Pos2> = (0..=n).map(|i| c + (end - c) * (i as f32 / n as f32)).collect();
        let norms = normals(&line);
        line.iter()
            .enumerate()
            .map(|(i, &p)| p + norms[i] * bend * (std::f32::consts::PI * i as f32 / n as f32).sin())
            .collect()
    };
    let paths: Vec<Vec<Pos2>> = if dirs.len() == 2 {
        // Two ends: one smooth curve through the center.
        let (a, b) = (
            c + world_dir(dirs[0]) * z * 0.5,
            c + world_dir(dirs[1]) * z * 0.5,
        );
        let n = 16;
        vec![
            (0..=n)
                .map(|i| {
                    let t = i as f32 / n as f32;
                    let u = 1.0 - t;
                    (a.to_vec2() * (u * u) + c.to_vec2() * (2.0 * u * t) + b.to_vec2() * (t * t))
                        .to_pos2()
                })
                .collect(),
        ]
    } else {
        dirs.iter().map(|&d| spoke(d)).collect()
    };

    // Wear varies slowly along the road.
    let tone = move |u: f32| 0.93 + 0.14 * wobble(seed, u * 3.0);
    let profile: Vec<(f32, Color32)> = match kind {
        RoadKind::Dirt => vec![
            (-2.0 * w, clear),
            (-1.35 * w, fade(sf.verge, 0.45)),
            (-0.85 * w, fade(sf.shoulder, 0.92)),
            (-0.45 * w, sf.body),
            (0.45 * w, sf.body),
            (0.85 * w, fade(sf.shoulder, 0.92)),
            (1.35 * w, fade(sf.verge, 0.45)),
            (2.0 * w, clear),
        ],
        RoadKind::Paved | RoadKind::Rail => {
            let rim = if kind == RoadKind::Paved { 0.95 } else { 0.9 };
            vec![
                (-1.7 * w, clear),
                (-1.25 * w, fade(sf.verge, 0.5)),
                (-rim * w, sf.verge),
                (-0.7 * w, sf.shoulder),
                (-0.5 * w, sf.body),
                (0.5 * w, sf.body),
                (0.7 * w, sf.shoulder),
                (rim * w, sf.verge),
                (1.25 * w, fade(sf.verge, 0.5)),
                (1.7 * w, clear),
            ]
        }
    };
    let mut mesh = Mesh::default();
    for p in &paths {
        ribbon(&mut mesh, p, &profile, &tone);
    }
    // Junction: a soft round patch so spokes blend at the hub.
    if dirs.len() != 2 {
        mesh::puff(&mut mesh, c, w * 1.5, sf.body);
    }
    painter.add(Shape::mesh(mesh));
    if z < 18.0 {
        return;
    }

    // Surface detail.
    let mut mesh = Mesh::default();
    let mut pebbles: Vec<(Pos2, f32)> = Vec::new();
    for (pi, p) in paths.iter().enumerate() {
        let salt = seed.wrapping_add(pi as u32 * 9);
        match kind {
            RoadKind::Dirt => {
                // Two wheel ruts and a few pebbles.
                let rut = [(-0.4 * w, clear), (0.0, fade(sf.mark, 0.55)), (0.4 * w, clear)];
                for side in [-1.0, 1.0] {
                    let off = offset(p, |i| side * 0.3 * w + (wobble(salt, i as f32 * 0.5) - 0.5) * 0.12 * w);
                    ribbon(&mut mesh, &off, &rut, &|_| 1.0);
                }
                for (i, q) in resample(p, (z * 0.2).max(5.0)).into_iter().enumerate() {
                    let h = salt.wrapping_add(i as u32).wrapping_mul(2_654_435_761);
                    let dx = ((h >> 8) % 100) as f32 / 100.0 - 0.5;
                    let dy = ((h >> 16) % 100) as f32 / 100.0 - 0.5;
                    pebbles.push((q + vec2(dx, dy) * 1.8 * w, (z * 0.02).max(0.9)));
                }
            }
            RoadKind::Paved => {
                // Cobbles: staggered short joints across the road.
                for (i, q) in resample(p, (z * 0.07).max(3.0)).windows(2).enumerate() {
                    let d = (q[1] - q[0]).normalized();
                    let n = vec2(-d.y, d.x);
                    let stagger = if i % 2 == 0 { 0.2 } else { -0.2 };
                    for k in -1..=1 {
                        let a = q[0] + n * (k as f32 * 0.4 + stagger) * w;
                        mesh::line(&mut mesh, a, a + d * z * 0.022, (z * 0.016).max(0.7), fade(sf.mark, 0.5));
                    }
                }
            }
            RoadKind::Rail => {
                for pair in resample(p, (z * 0.075).max(3.0)).windows(2) {
                    let d = (pair[1] - pair[0]).normalized();
                    let n = vec2(-d.y, d.x) * w * 0.72;
                    mesh::line(&mut mesh, pair[0] - n, pair[0] + n, (z * 0.032).max(1.0), sf.mark);
                }
                let steel = [
                    (-0.1 * w, clear),
                    (0.0, Color32::from_rgb(214, 218, 226)),
                    (0.1 * w, clear),
                ];
                for side in [-1.0, 1.0] {
                    ribbon(&mut mesh, &offset(p, |_| side * 0.3 * w), &steel, &|_| 1.0);
                }
            }
        }
    }
    painter.add(Shape::mesh(mesh));
    for (q, r) in pebbles {
        painter.circle_filled(q, r, fade(sf.mark, 0.7));
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

/// Where the farmhouse stands in a field decal, in hex radii from the
/// center (screen axes), before the decal's rotation.
const YARD: Vec2 = vec2(-0.42, -0.34);

/// Draws a field decal (rotated to the tile's hash), the road across it, and
/// the farmhouse on its yard. The art follows the viewer's era; some tiles
/// keep an older look.
fn draw_field(
    art: &mut ResourceArt,
    painter: &egui::Painter,
    t: &TileCtx,
    name: &str,
    tint: Color32,
    h: u32,
) {
    let (c, z) = (t.c, t.z);
    let ctx = painter.ctx().clone();
    let mut era: usize = match t.game.era_of(t.viewer) {
        0 | 1 => 0,
        2 | 3 => 1,
        4 | 5 => 2,
        _ => 3,
    };
    if h % 3 == 0 {
        era = era.saturating_sub(1);
    }
    let look = (h >> 9) & 1;
    let decal = if name == "imp_farm" {
        format!("imp_farm_{era}_{look}")
    } else {
        format!("imp_plantation_{look}")
    };
    let turns = h % 6;
    if let Some(tex) = art.tex(&ctx, "improvements", &decal) {
        blit(painter, tex, c, DECAL * z, tint, turns);
    }
    paint_road(painter, t);
    let (s, co) = (turns as f32 * std::f32::consts::FRAC_PI_3).sin_cos();
    let at = c + vec2(YARD.x * co - YARD.y * s, YARD.x * s + YARD.y * co) * z;
    let scale = if t.tile().resource.is_some() { 0.46 } else { 0.68 };
    if let Some(tex) = art.tex(&ctx, "improvements", &format!("imp_farmhouse_{era}")) {
        blit(painter, tex, at, SPRITE * z * scale, tint, 0);
    }
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
        let avg = match name.as_str() {
            "imp_farm" => Color32::from_rgb(156, 146, 72),
            "imp_plantation" => Color32::from_rgb(88, 142, 62),
            _ => art
                .tex(&ctx, "improvements", &name)
                .map(|x| x.avg)
                .unwrap_or(Color32::from_rgb(140, 100, 60)),
        };
        if layout == Layout::Field {
            let mut blob = Mesh::default();
            mesh::puff(&mut blob, c, z * 1.05, avg);
            painter.add(Shape::mesh(blob));
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
        // A soft red glow so camps read at a glance.
        let mut glow = Mesh::default();
        mesh::puff(&mut glow, c, z * 1.0, Color32::from_rgba_unmultiplied(190, 34, 30, 70));
        painter.add(Shape::mesh(glow));
    } else if name == "imp_goody_hut" {
        // Tribal villages glow gold, slowly breathing, to invite explorers.
        let t = painter.ctx().input(|i| i.time) as f32;
        let pulse = 0.5 + 0.5 * (t * 1.6 + (h % 1000) as f32).sin();
        let mut glow = Mesh::default();
        mesh::puff(
            &mut glow,
            c,
            z * 1.3,
            Color32::from_rgba_unmultiplied(255, 236, 170, (150.0 + 60.0 * pulse) as u8),
        );
        painter.add(Shape::mesh(glow));
        painter.circle_stroke(
            c + vec2(0.0, 0.05 * z),
            z * (0.66 + 0.04 * pulse),
            Stroke::new(
                (z * 0.035).max(1.5),
                Color32::from_rgba_unmultiplied(255, 205, 80, (140.0 + 90.0 * pulse) as u8),
            ),
        );
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
    let drew = if layout == Layout::Field {
        draw_field(art, painter, t, &name, tint, h);
        true
    } else if let Some(tex) = art.tex(&ctx, "improvements", &name) {
        blit(painter, tex, at, SPRITE * z * scale, tint, 0);
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
    if !player.meets(rules, res.prereq_tech.as_deref()) || z <= 8.0 {
        return;
    }
    let color = class_color(&res.class);
    // The model on the tile, large enough to read at a glance, and the
    // class badge in the corner.
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
            None => (c + vec2(0.0, 0.1 * z), 0.86),
            Some(Layout::Field) => (c + vec2(0.0, 0.08 * z), 0.62),
            Some(Layout::Ring) => (c + vec2(0.0, 0.1 * z), 0.58),
            Some(Layout::Side) => (c + vec2(0.26 * z, -0.04 * z), 0.52),
        };
        if let Some(tex) = art.tex(&ctx, "sprites", &name) {
            blit(painter, tex, at, SPRITE * z * scale, Color32::WHITE, 0);
        }
    }

    // The class badge in the corner.
    let radius = (z * 0.19).max(6.0);
    let badge = Rect::from_center_size(corner, vec2(radius * 2.0, radius * 2.0));
    let tint = Color32::WHITE;
    painter.circle_filled(corner + vec2(0.0, radius * 0.12), radius * 1.04, Color32::from_black_alpha(90));
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

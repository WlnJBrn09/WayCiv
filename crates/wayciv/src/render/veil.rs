//! What hides, dims and lights the map: the unexplored world (a dark
//! parchment chart with drifting mist), the soft edge where explored land
//! meets it, the sepia fog over tiles you have seen but cannot see now, and
//! the lighting laid over everything (cloud shadows, sun, vignette).

use super::TileCtx;
use super::anim::smooth;
use super::hex::{EDGES, SQRT3, corners};
use super::mesh;
use crate::palette::with_alpha;
use egui::epaint::Vertex;
use egui::{Color32, ColorImage, Mesh, Painter, Pos2, Rect, Shape, TextureHandle, TextureId, TextureOptions, pos2, vec2};

/// The color of unexplored ground on average; the frontier fades into it.
pub const UNKNOWN: Color32 = Color32::from_rgb(34, 29, 25);
/// Tint of tiles you have seen but cannot see now.
const FOG: [u8; 3] = [78, 66, 50];

/// Textures for the unexplored world, made on first use.
#[derive(Default)]
pub struct VeilArt {
    parchment: Option<TextureHandle>,
    mist: Option<TextureHandle>,
}

fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ seed.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h & 0xffff) as f32 / 65535.0
}

/// Smooth value noise over a lattice that wraps every `px` x `py` cells.
fn noise(x: f32, y: f32, px: i32, py: i32, seed: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (smooth(x - x0), smooth(y - y0));
    let at = |dx: i32, dy: i32| hash((x0 as i32 + dx).rem_euclid(px), (y0 as i32 + dy).rem_euclid(py), seed);
    let top = at(0, 0) + (at(1, 0) - at(0, 0)) * fx;
    let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * fx;
    top + (bottom - top) * fy
}

/// Layered noise that tiles over the unit square; `cells` is the lattice
/// size of the first layer.
fn fbm(u: f32, v: f32, cells: i32, layers: u32, seed: u32) -> f32 {
    let (mut sum, mut amp, mut norm) = (0.0, 1.0, 0.0);
    for i in 0..layers {
        let n = cells << i;
        sum += amp * noise(u * n as f32, v * n as f32, n, n, seed + i);
        norm += amp;
        amp *= 0.5;
    }
    sum / norm
}

const SIZE: usize = 512;

/// Aged parchment in the dark: mottled stains, fine fibres, a few creases.
fn parchment() -> ColorImage {
    let mut image = ColorImage::filled([SIZE, SIZE], UNKNOWN);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (u, v) = (x as f32 / SIZE as f32, y as f32 / SIZE as f32);
            let blotch = fbm(u, v, 3, 5, 11);
            let stain = smooth((fbm(u, v, 2, 3, 71) - 0.5) * 3.0 + 0.5);
            // Fibres run in both directions at a fine scale.
            let fibre = noise(u * 220.0, v * 7.0, 220, 7, 5) * 0.5 + noise(u * 6.0, v * 190.0, 6, 190, 9) * 0.5;
            let grain = hash(x as i32, y as i32, 3);
            let crease = 1.0 - (noise(u * 5.0, v * 5.0, 5, 5, 41) - 0.5).abs() * 14.0;
            let lum = (blotch - 0.5) * 26.0 + (fibre - 0.5) * 9.0 + (grain - 0.5) * 3.0 - stain * 7.0 - crease.max(0.0) * 6.0;
            let c = |base: f32, warm: f32| (base + lum * warm).clamp(0.0, 255.0) as u8;
            image.pixels[y * SIZE + x] = Color32::from_rgb(c(40.0, 1.0), c(34.0, 0.9), c(28.0, 0.75));
        }
    }
    image
}

/// Soft cloud banks, white with a noisy alpha.
fn mist() -> ColorImage {
    const S: usize = 256;
    let mut image = ColorImage::filled([S, S], Color32::TRANSPARENT);
    for y in 0..S {
        for x in 0..S {
            let (u, v) = (x as f32 / S as f32, y as f32 / S as f32);
            let n = fbm(u, v, 4, 5, 23);
            let a = smooth(((n - 0.42) / 0.3).clamp(0.0, 1.0));
            image.pixels[y * S + x] = Color32::from_rgba_unmultiplied(214, 196, 164, (a * 255.0) as u8);
        }
    }
    image
}

/// World units across one repeat of each texture.
const PARCHMENT_SPAN: f32 = 22.0;
const MIST_SPAN: f32 = 34.0;
/// The chart's graticule, in world units.
const GRATICULE: f32 = 7.5;

impl VeilArt {
    fn textures(&mut self, painter: &Painter) -> (TextureId, TextureId) {
        let options = TextureOptions::LINEAR_REPEAT.with_mipmap_mode(Some(egui::TextureFilter::Linear));
        let ctx = painter.ctx();
        let p = self.parchment.get_or_insert_with(|| ctx.load_texture("veil-parchment", parchment(), options));
        let a = p.id();
        let m = self.mist.get_or_insert_with(|| ctx.load_texture("veil-mist", mist(), options));
        (a, m.id())
    }
}

/// A textured quad over `rect` whose texture is pinned to the world: `span`
/// world units per repeat, drifting by `drift` units.
fn pinned(painter: &Painter, tex: TextureId, rect: Rect, center: egui::Vec2, z: f32, span: f32, drift: egui::Vec2, tint: Color32) {
    let uv = |p: Pos2| {
        let w = (p - rect.center()) / z + center + drift;
        pos2(w.x / span, w.y / span)
    };
    let mut m = Mesh::with_texture(tex);
    for p in [rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()] {
        m.vertices.push(Vertex { pos: p, uv: uv(p), color: tint });
    }
    m.indices.extend([0, 1, 2, 0, 2, 3]);
    painter.add(Shape::mesh(m));
}

/// The unexplored world, drawn under the terrain: parchment, a faint
/// chart graticule and slow banks of mist.
pub fn unknown(art: &mut VeilArt, painter: &Painter, rect: Rect, center: egui::Vec2, z: f32, now: f32) {
    let (parchment, mist) = art.textures(painter);
    pinned(painter, parchment, rect, center, z, PARCHMENT_SPAN, vec2(0.0, 0.0), Color32::WHITE);

    let to_screen = |w: egui::Vec2| rect.center() + (w - center) * z;
    let mut lines = Mesh::default();
    let ink = Color32::from_rgba_unmultiplied(206, 176, 120, 20);
    let (lo, hi) = (center - rect.size() / (2.0 * z), center + rect.size() / (2.0 * z));
    let width = if z >= 20.0 { 1.4 } else { 1.0 };
    let mut x = (lo.x / GRATICULE).floor() * GRATICULE;
    while x <= hi.x {
        let (a, b) = (to_screen(vec2(x, lo.y)), to_screen(vec2(x, hi.y)));
        mesh::line(&mut lines, a, b, width, ink);
        x += GRATICULE;
    }
    let mut y = (lo.y / GRATICULE).floor() * GRATICULE;
    while y <= hi.y {
        let (a, b) = (to_screen(vec2(lo.x, y)), to_screen(vec2(hi.x, y)));
        mesh::line(&mut lines, a, b, width, ink);
        y += GRATICULE;
    }
    painter.add(Shape::mesh(lines));

    // Two banks drifting different ways, so the mist never repeats visibly.
    let tint = |a: f32| Color32::from_white_alpha((a * 255.0) as u8);
    pinned(painter, mist, rect, center, z, MIST_SPAN, vec2(now * 0.12, now * 0.04), tint(0.16));
    pinned(painter, mist, rect, center, z, MIST_SPAN * 0.63, vec2(-now * 0.07 + 9.0, now * 0.05 + 4.0), tint(0.1));
}

/// Which of a hex's corners touch a neighbor satisfying `flag` (indexed by
/// direction): corner i lies on the edges of directions 6-i and 7-i.
fn corner_flags(flag: [bool; 6]) -> [bool; 6] {
    std::array::from_fn(|i| flag[(6 - i) % 6] || flag[(7 - i) % 6])
}

/// The soft edge of explored land: the parchment mist creeps in over the
/// tiles beside unexplored ones. Added once per tile.
pub fn frontier(m: &mut Mesh, t: &TileCtx) {
    let player = &t.game.players[t.viewer];
    let grid = t.game.map.grid;
    let hidden: [bool; 6] = std::array::from_fn(|d| t.neighbor(d).is_none_or(|n| !player.revealed[grid.index(n)]));
    if !hidden.contains(&true) {
        return;
    }
    let touches = corner_flags(hidden);
    // Rows from the rim inward: (radius, strength).
    let rows = [(1.02, 1.0), (0.84, 0.62), (0.68, 0.28), (0.5, 0.0)];
    let rings: Vec<[Pos2; 6]> = rows.iter().map(|&(r, _)| corners(t.c, t.z * r)).collect();
    for &(a, b) in &EDGES {
        for i in 0..rows.len() - 1 {
            let tone = |row: usize, corner: usize| {
                let k = if touches[corner] { rows[row].1 } else { 0.0 };
                with_alpha(UNKNOWN, (k * 252.0) as u8)
            };
            mesh::quad(
                m,
                (rings[i][a], tone(i, a)),
                (rings[i][b], tone(i, b)),
                (rings[i + 1][b], tone(i + 1, b)),
                (rings[i + 1][a], tone(i + 1, a)),
            );
        }
    }
}

/// The fog over a tile you have seen but cannot see now (`strength` 0 to
/// 1): a desaturating sepia wash that leaves the grid line clear and eases
/// out toward tiles that are in sight.
pub fn fog(t: &TileCtx, strength: f32) -> Shape {
    let player = &t.game.players[t.viewer];
    let grid = t.game.map.grid;
    let lit: [bool; 6] = std::array::from_fn(|d| t.neighbor(d).is_some_and(|n| player.visible[grid.index(n)]));
    let wash = |k: f32| Color32::from_rgba_unmultiplied(FOG[0], FOG[1], FOG[2], (k * strength * 160.0) as u8);
    let clear = (t.z * 0.05).clamp(1.2, 2.8) * 0.5 - 0.2;
    let r = t.z - clear / (SQRT3 / 2.0);
    let mut m = Mesh::default();
    if !lit.contains(&true) {
        mesh::fill(&mut m, &corners(t.c, r), wash(1.0));
        return Shape::mesh(m);
    }
    let touches = corner_flags(lit);
    let inner = corners(t.c, r * 0.55);
    let outer = corners(t.c, r);
    mesh::fill(&mut m, &inner, wash(1.0));
    for &(a, b) in &EDGES {
        let rim = |c: usize| wash(if touches[c] { 0.3 } else { 1.0 });
        mesh::quad(&mut m, (inner[a], wash(1.0)), (inner[b], wash(1.0)), (outer[b], rim(b)), (outer[a], rim(a)));
    }
    Shape::mesh(m)
}

/// Soft shadows of clouds drifting over the land: one coarse mesh over the
/// view, so the cost doesn't depend on how many tiles are showing.
pub fn cloud_shadows(painter: &Painter, rect: Rect, center: egui::Vec2, z: f32, now: f32) {
    const CELL: f32 = 48.0;
    let (cols, rows) = ((rect.width() / CELL).ceil() as usize + 1, (rect.height() / CELL).ceil() as usize + 1);
    let mut m = Mesh::default();
    for r in 0..=rows {
        for c in 0..=cols {
            let p = rect.min + vec2(c as f32, r as f32) * CELL;
            let w = (p - rect.center()) / z + center;
            // Crossing swells make blobs; the drift is slow.
            let n = ((w.x * 0.17 - now * 0.09).sin() + (w.y * 0.23 + now * 0.05).sin()) * 0.5
                + 0.45 * (w.x * 0.07 + w.y * 0.11 - now * 0.04).sin()
                + 0.2 * (w.x * 0.41 + w.y * 0.33 + now * 0.07).sin();
            let a = smooth((n - 0.35) / 0.95);
            m.colored_vertex(p, Color32::from_rgba_unmultiplied(8, 14, 30, (a * 54.0) as u8));
        }
    }
    let stride = (cols + 1) as u32;
    for r in 0..rows as u32 {
        for c in 0..cols as u32 {
            let i = r * stride + c;
            m.add_triangle(i, i + 1, i + stride);
            m.add_triangle(i + 1, i + stride + 1, i + stride);
        }
    }
    painter.add(Shape::mesh(m));
}

/// Sunlight from the upper left and a vignette that darkens the corners:
/// two coarse meshes over the whole view.
pub fn lighting(painter: &Painter, rect: Rect) {
    const N: usize = 14;
    let mut sun = Mesh::default();
    let mut dark = Mesh::default();
    for r in 0..=N {
        for c in 0..=N {
            let (u, v) = (c as f32 / N as f32, r as f32 / N as f32);
            let p = pos2(rect.left() + u * rect.width(), rect.top() + v * rect.height());
            // Distance from the middle, 0 at the center, about 1 at the corners.
            let d = (((u - 0.5) * 2.0).powi(2) + ((v - 0.5) * 2.0).powi(2)).sqrt() / 1.41;
            let edge = smooth(((d - 0.5) / 0.5).clamp(0.0, 1.0));
            dark.colored_vertex(p, Color32::from_rgba_unmultiplied(4, 6, 14, (edge * 92.0) as u8));
            let warm = (1.0 - ((u * u + v * v).sqrt() / 1.3)).clamp(0.0, 1.0);
            sun.colored_vertex(p, Color32::from_rgba_unmultiplied(255, 226, 150, (warm * warm * 22.0) as u8));
        }
    }
    let stride = (N + 1) as u32;
    for m in [&mut sun, &mut dark] {
        for r in 0..N as u32 {
            for c in 0..N as u32 {
                let i = r * stride + c;
                m.add_triangle(i, i + 1, i + stride);
                m.add_triangle(i + 1, i + stride + 1, i + stride);
            }
        }
    }
    painter.add(Shape::mesh(sun));
    painter.add(Shape::mesh(dark));
}

//! What is laid over the terrain: the hex grid, territory (tint, border
//! ribbons and inner glow) and the tile highlights (reachable, attackable,
//! building sites, the selected and hovered tile, the walking path).
//! Everything is built into meshes by the caller's loop; nothing here keeps
//! state between frames.

use super::TileCtx;
use super::hex::{EDGES, SQRT3, corners, world_dir};
use super::mesh::{self, Vert};
use crate::palette::{rgb, with_alpha};
use egui::{Color32, Mesh, Pos2, Vec2};
use wayciv_core::Pos;

/// Light comes from the upper left: edges facing NW and W catch it, the
/// ones facing E and SE are in shade. Indexed by direction (E, NE, NW, W,
/// SW, SE): positive is a highlight, negative a shadow.
const LIGHT: [f32; 6] = [-0.7, 0.25, 1.0, 1.0, 0.25, -0.7];

/// Distance of a hexagon's inset outline from its edge, as a radius change.
fn inset(d: f32) -> f32 {
    d / (SQRT3 / 2.0)
}

/// The hex grid for one tile: a dark ink line on every edge, with a light
/// bevel inside it that makes tiles read as raised. Each shared edge's line
/// is drawn once, by the tile east, north-east or north-west of it.
/// `strength` fades the grid out when zoomed far out.
pub fn grid(m: &mut Mesh, t: &TileCtx, strength: f32) {
    let (c, z) = (t.c, t.z);
    let player = &t.game.players[t.viewer];
    let grid = t.game.map.grid;
    let ink_w = (z * 0.05).clamp(1.2, 2.8);
    let ink = Color32::from_rgba_unmultiplied(18, 14, 10, (165.0 * strength) as u8);
    let outer = corners(c, z);
    let bevel = z >= 18.0;
    let bevel_w = (z * 0.022).clamp(0.8, 1.6);
    let ring = corners(c, z - inset(ink_w * 0.5 + bevel_w * 0.5));
    let soft = z >= 24.0;
    let depth = z * 0.2;
    let deep = corners(c, z - inset(depth));
    for (dir, &(a, b)) in EDGES.iter().enumerate() {
        let shared = t
            .neighbor(dir)
            .is_some_and(|n| player.revealed[grid.index(n)]);
        if dir < 3 || !shared {
            mesh::line(m, outer[a], outer[b], ink_w, ink);
        }
        let light = LIGHT[dir];
        if bevel {
            let (col, a8) = if light > 0.0 { ((255, 244, 210), 92.0) } else { ((10, 8, 6), 70.0) };
            let alpha = (a8 * light.abs() * strength) as u8;
            mesh::line(
                m,
                ring[a],
                ring[b],
                bevel_w,
                Color32::from_rgba_unmultiplied(col.0, col.1, col.2, alpha),
            );
        }
        if soft {
            let (col, a8) = if light > 0.0 { ((255, 240, 200), 36.0) } else { ((8, 6, 4), 44.0) };
            let alpha = (a8 * light.abs() * strength) as u8;
            let edge = Color32::from_rgba_unmultiplied(col.0, col.1, col.2, alpha);
            let clear = with_alpha(edge, 0);
            mesh::quad(m, (outer[a], edge), (outer[b], edge), (deep[b], clear), (deep[a], clear));
        }
    }
}

/// How the territory ribbon is built, from the hex edge inward: a dark key
/// line, the civilization's color, a thin line in its second color, then a
/// soft glow fading into the land. Each row is (distance from the edge, color).
fn ribbon(z: f32, primary: Color32, secondary: Color32, k: f32) -> [(f32, Color32); 9] {
    let key = Color32::from_black_alpha((170.0 * k) as u8);
    let width = (z * 0.17).clamp(4.0, 9.5);
    let thin = (z * 0.045).clamp(1.3, 2.6);
    let rim = 1.4;
    let glow = z * 0.5;
    let edge = rim + width + thin;
    let (p, s) = (primary.gamma_multiply(k), secondary.gamma_multiply(k));
    // The glow is a lighter wash of the main color, so it shows on dark ones too.
    let haze = primary.lerp_to_gamma(Color32::WHITE, 0.3).gamma_multiply(k);
    [
        (-0.6, with_alpha(key, 0)),
        (0.3, key),
        (rim, key),
        (rim, p),
        (rim + width, p),
        (rim + width, s),
        (edge, s),
        (edge, haze.gamma_multiply(0.6)),
        (edge + glow, with_alpha(haze, 0)),
    ]
}

/// The inward unit vector of the edge of a tile facing `dir`.
fn inward(dir: usize) -> Vec2 {
    -world_dir(dir) / SQRT3
}

/// The territory of one tile: a faint tint over the whole hex and a ribbon
/// along every edge that faces another owner. Ribbons join at the hex
/// corners (mitered on convex turns, with a wedge fill on concave ones), so
/// a border reads as one continuous line. `claim` fades fresh conquests in.
pub fn territory(m: &mut Mesh, t: &TileCtx, claim: f32) {
    let game = t.game;
    let Some(owner) = t.tile().owner else { return };
    let o = &game.players[owner];
    let (primary, secondary) = (rgb(o.primary), rgb(o.secondary));
    let (c, z) = (t.c, t.z);
    mesh::fill(m, &corners(c, z), with_alpha(primary, (50.0 * claim) as u8));

    let owner_at = |p: Option<Pos>| p.map(|p| game.map.tile(p).owner);
    let rows = ribbon(z, primary, secondary, claim);
    let pts = corners(c, z);
    for (dir, &(a, b)) in EDGES.iter().enumerate() {
        let Some(n) = t.neighbor(dir) else { continue };
        if game.map.tile(n).owner == Some(owner) {
            continue;
        }
        let normal = inward(dir);
        // At each end, the offset of the ribbon's inner rows per unit of
        // depth, and the wedge to fill on a concave turn.
        let mut ends = [(pts[a], normal, None), (pts[b], normal, None)];
        for (end, other) in [(0, (dir + 1) % 6), (1, (dir + 5) % 6)] {
            let across = t.neighbor(other);
            if owner_at(across) == Some(Some(owner)) {
                // The border turns away along the neighbor's edge.
                let turn = (world_dir(other) - world_dir(dir)) / SQRT3;
                let lower = across.is_some_and(|y| game.map.grid.index(y) > game.map.grid.index(t.pos));
                ends[end].2 = lower.then_some(turn);
            } else {
                let next = inward(other);
                ends[end].1 = (normal + next) / (1.0 + normal.dot(next));
            }
        }
        for w in rows.windows(2) {
            let at = |end: usize, row: (f32, Color32)| -> Vert { (ends[end].0 + ends[end].1 * row.0, row.1) };
            mesh::quad(m, at(0, w[0]), at(1, w[0]), at(1, w[1]), at(0, w[1]));
            for end in 0..2 {
                if let Some(turn) = ends[end].2 {
                    let there = |row: (f32, Color32)| -> Vert { (ends[end].0 + turn * row.0, row.1) };
                    mesh::quad(m, at(end, w[0]), there(w[0]), there(w[1]), at(end, w[1]));
                }
            }
        }
    }
}

/// A ring of color fading from the rim of the hex inward: the glow of a
/// highlighted tile.
pub fn glow_hex(m: &mut Mesh, c: Pos2, r: f32, depth: f32, color: Color32) {
    let outer = corners(c, r);
    let deep = corners(c, (r - inset(depth)).max(0.0));
    let clear = with_alpha(color, 0);
    for &(a, b) in &EDGES {
        mesh::quad(m, (outer[a], color), (outer[b], color), (deep[b], clear), (deep[a], clear));
    }
}

/// A hex outline.
pub fn hex_line(m: &mut Mesh, c: Pos2, r: f32, width: f32, color: Color32) {
    mesh::outline(m, &corners(c, r), width, color);
}

/// A tile marked by a highlight: a soft inner glow and a clean outline with
/// a dark keyline outside it.
pub fn mark(m: &mut Mesh, c: Pos2, z: f32, glow: Color32, line: Color32, width: f32) {
    glow_hex(m, c, z * 0.97, z * 0.4, glow);
    hex_line(m, c, z * 0.97, width + 2.0, Color32::from_black_alpha(70));
    hex_line(m, c, z * 0.97, width, line);
}

/// Points spaced `step` apart along a polyline, starting `phase` in.
pub fn along(path: &[Pos2], step: f32, phase: f32) -> Vec<(Pos2, Vec2)> {
    let mut out = Vec::new();
    let mut carry = phase.rem_euclid(step);
    for w in path.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = (b - a).length();
        if len < 0.01 {
            continue;
        }
        let dir = (b - a) / len;
        let mut at = carry;
        while at < len {
            out.push((a + dir * at, dir));
            at += step;
        }
        carry = at - len;
    }
    out
}

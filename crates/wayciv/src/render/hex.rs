//! Hex geometry shared by the map renderers: pointy-top hexes, odd-r rows,
//! world units where the hex radius is 1.

use egui::{Color32, Pos2, Shape, Stroke, Vec2, vec2};
use wayciv_core::Pos;

pub const SQRT3: f32 = 1.732_050_8;

/// Axial neighbor offsets, matching `wayciv_core::hex`: E, NE, NW, W, SW, SE.
pub const DIRS: [(i32, i32); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];
/// Corner pairs of the edge facing each direction (corner i at 60*i-30 degrees).
pub const EDGES: [(usize, usize); 6] = [(0, 1), (5, 0), (4, 5), (3, 4), (2, 3), (1, 2)];

/// A tile's center in world units.
pub fn world(pos: Pos) -> Vec2 {
    vec2(
        SQRT3 * (pos.x as f32 + 0.5 * (pos.y & 1) as f32),
        1.5 * pos.y as f32,
    )
}

/// Corner i sits at 60*i-30 degrees (0 = lower right of the east edge).
pub fn corners(center: Pos2, r: f32) -> [Pos2; 6] {
    std::array::from_fn(|i| {
        let angle = (60.0 * i as f32 - 30.0).to_radians();
        center + vec2(angle.cos(), angle.sin()) * r
    })
}

pub fn hex(center: Pos2, r: f32, fill: Color32, stroke: Stroke) -> Shape {
    Shape::convex_polygon(corners(center, r).to_vec(), fill, stroke)
}

/// The world-space offset toward neighbor `dir` (length sqrt 3).
pub fn world_dir(dir: usize) -> Vec2 {
    let (q, r) = DIRS[dir];
    vec2(SQRT3 * (q as f32 + r as f32 / 2.0), 1.5 * r as f32)
}

pub fn neighbor(grid: &wayciv_core::Grid, pos: Pos, dir: usize) -> Option<Pos> {
    let (q, r) = pos.axial();
    grid.normalize(Pos::from_axial(q + DIRS[dir].0, r + DIRS[dir].1))
}

/// A short label from a name: initials of the first two words.
pub fn abbreviation(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.len() >= 2 {
        words
            .iter()
            .take(2)
            .filter_map(|w| w.chars().next())
            .collect()
    } else {
        name.chars().take(2).collect()
    }
}

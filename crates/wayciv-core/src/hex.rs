//! Pointy-top hex coordinates. Tiles are stored in "odd-r" offset rows
//! (odd rows shifted half a hex right, as in Civ VI); rules use axial
//! coordinates for distance and neighbors. The map wraps east-west.

use std::fmt;

/// Offset coordinates of a tile: column `x`, row `y`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Axial `(q, r)`.
    pub fn axial(self) -> (i32, i32) {
        (self.x - (self.y - (self.y & 1)) / 2, self.y)
    }

    pub fn from_axial(q: i32, r: i32) -> Self {
        Self { x: q + (r - (r & 1)) / 2, y: r }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Axial neighbor offsets, clockwise from east.
const AXIAL_DIRECTIONS: [(i32, i32); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];

/// Map dimensions with east-west wrapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    pub width: i32,
    pub height: i32,
}

impl Grid {
    pub fn len(self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub fn index(self, pos: Pos) -> usize {
        (pos.y * self.width + pos.x) as usize
    }

    pub fn pos(self, index: usize) -> Pos {
        Pos::new(index as i32 % self.width, index as i32 / self.width)
    }

    /// Wraps x; `None` above or below the map.
    pub fn normalize(self, pos: Pos) -> Option<Pos> {
        (0..self.height).contains(&pos.y).then(|| Pos::new(pos.x.rem_euclid(self.width), pos.y))
    }

    pub fn neighbors(self, pos: Pos) -> impl Iterator<Item = Pos> {
        let (q, r) = pos.axial();
        AXIAL_DIRECTIONS.into_iter().filter_map(move |(dq, dr)| self.normalize(Pos::from_axial(q + dq, r + dr)))
    }

    /// Hex distance, taking the shorter way around the east-west wrap.
    pub fn distance(self, a: Pos, b: Pos) -> i32 {
        let mut best = i32::MAX;
        for shift in [-self.width, 0, self.width] {
            let (aq, ar) = a.axial();
            let (bq, br) = Pos::new(b.x + shift, b.y).axial();
            let (dq, dr) = (aq - bq, ar - br);
            best = best.min((dq.abs() + dr.abs() + (dq + dr).abs()) / 2);
        }
        best
    }

    /// Every tile within `radius` of `center` (including it), nearest first.
    pub fn within(self, center: Pos, radius: i32) -> Vec<Pos> {
        let (cq, cr) = center.axial();
        let mut tiles = Vec::new();
        for dq in -radius..=radius {
            for dr in (-radius).max(-dq - radius)..=radius.min(-dq + radius) {
                if let Some(pos) = self.normalize(Pos::from_axial(cq + dq, cr + dr))
                    && !tiles.contains(&pos) {
                        tiles.push(pos);
                    }
            }
        }
        tiles.sort_by_key(|p| self.distance(center, *p));
        tiles
    }

    pub fn all(self) -> impl Iterator<Item = Pos> {
        (0..self.len()).map(move |i| self.pos(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRID: Grid = Grid { width: 20, height: 10 };

    #[test]
    fn axial_round_trip() {
        for pos in GRID.all() {
            let (q, r) = pos.axial();
            assert_eq!(Pos::from_axial(q, r), pos);
        }
    }

    #[test]
    fn six_neighbors_at_distance_one() {
        let center = Pos::new(5, 5);
        let neighbors: Vec<_> = GRID.neighbors(center).collect();
        assert_eq!(neighbors.len(), 6);
        assert!(neighbors.iter().all(|n| GRID.distance(center, *n) == 1));
    }

    #[test]
    fn wraps_east_west_but_not_north_south() {
        assert_eq!(GRID.distance(Pos::new(0, 4), Pos::new(19, 4)), 1);
        assert_eq!(GRID.neighbors(Pos::new(3, 0)).count(), 4);
        assert_eq!(GRID.within(Pos::new(5, 5), 1).len(), 7);
        assert_eq!(GRID.within(Pos::new(5, 5), 3).len(), 37);
    }
}

//! Map art. Each kind of thing on the map has its own module and its own
//! art state (textures load lazily from `assets/`):
//!   terrain   — biomes, hills, mountains, features, cliffs, rivers
//!   resources — resources, improvements, roads, barbarian camps
//!   buildings — districts, wonders, city centers
//!   units     — land, sea, and air units
//!   banner    — city banners
//!   anim, fx  — animation schedule (walks, fights, fades) and its effects
//! `map_view` decides what to draw where and in which order.

pub mod anim;
pub mod banner;
pub mod buildings;
pub mod fx;
pub mod hex;
pub mod resources;
pub mod terrain;
pub mod units;

use egui::Pos2;
use std::path::PathBuf;
use wayciv_core::{Game, PlayerId, Pos};

pub struct Art {
    pub terrain: terrain::TerrainArt,
    pub resources: resources::ResourceArt,
    pub buildings: buildings::BuildingArt,
    pub units: units::UnitArt,
}

impl Art {
    /// Cheap: nothing is loaded until first drawn.
    pub fn new(assets: PathBuf) -> Self {
        Self {
            terrain: terrain::TerrainArt::new(assets.clone()),
            resources: resources::ResourceArt::new(assets.clone()),
            buildings: buildings::BuildingArt::new(assets.clone()),
            units: units::UnitArt::new(assets),
        }
    }
}

/// One tile being drawn: where it is on screen and how big.
#[derive(Clone, Copy)]
pub struct TileCtx<'a> {
    pub game: &'a Game,
    /// The player looking at the map (fog, revealed resources).
    pub viewer: PlayerId,
    pub pos: Pos,
    /// Screen center of the hex.
    pub c: Pos2,
    /// Hex radius in pixels (8 to 70).
    pub z: f32,
}

impl TileCtx<'_> {
    pub fn tile(&self) -> &wayciv_core::map::Tile {
        self.game.map.tile(self.pos)
    }

    pub fn neighbor(&self, dir: usize) -> Option<Pos> {
        hex::neighbor(&self.game.map.grid, self.pos, dir)
    }

    /// Whether names and small details are legible at this zoom.
    pub fn detail(&self) -> bool {
        self.z >= 22.0
    }
}

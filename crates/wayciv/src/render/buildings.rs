//! Districts, wonders, and city centers.

use super::TileCtx;
use super::hex::hex;
use crate::palette::{self, rgb};
use egui::{Align2, Color32, FontId, Pos2, Stroke};
use std::path::PathBuf;

pub struct BuildingArt {
    #[allow(dead_code)]
    assets: PathBuf,
}

impl BuildingArt {
    pub fn new(assets: PathBuf) -> Self {
        Self { assets }
    }
}

/// Layer 4: a district (other than a city center) or a wonder on the tile.
pub fn draw_district(_art: &mut BuildingArt, painter: &egui::Painter, t: &TileCtx) {
    let (game, c, z) = (t.game, t.c, t.z);
    let content = &game.rules.content;
    let tile = t.tile();
    if let Some(d) = tile.district {
        let district = &content.districts[d];
        if !district.city_center {
            let base = district.replaces.as_deref().unwrap_or(&district.id);
            let (color, label) = palette::district(base);
            painter.add(hex(
                c,
                z * 0.62,
                color,
                Stroke::new(1.5, Color32::from_gray(30)),
            ));
            if z > 12.0 {
                painter.text(
                    c,
                    Align2::CENTER_CENTER,
                    label,
                    FontId::proportional(z * 0.42),
                    Color32::from_gray(20),
                );
            }
        }
    }
    if tile.wonder.is_some() {
        painter.add(hex(
            c,
            z * 0.62,
            palette::GOLD,
            Stroke::new(1.5, Color32::from_rgb(90, 60, 10)),
        ));
        if z > 12.0 {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "W",
                FontId::proportional(z * 0.5),
                Color32::from_rgb(60, 40, 0),
            );
        }
    }
}

/// Layer 6: a city center, under its banner. `c` is the tile's screen center.
pub fn draw_city(
    _art: &mut BuildingArt,
    painter: &egui::Painter,
    game: &wayciv_core::Game,
    city: &wayciv_core::state::City,
    c: Pos2,
    z: f32,
) {
    let owner = &game.players[city.owner];
    painter.add(hex(
        c,
        z * 0.7,
        rgb(owner.primary),
        Stroke::new(2.0, rgb(owner.secondary)),
    ));
    painter.circle_filled(c, z * 0.18, rgb(owner.secondary));
}

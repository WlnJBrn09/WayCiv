//! Units: land and sea units in their tile slot, aircraft at their base.

use super::hex::abbreviation;
use crate::icons::Icons;
use crate::palette::rgb;
use egui::{Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, vec2};
use std::path::PathBuf;

pub struct UnitArt {
    #[allow(dead_code)]
    assets: PathBuf,
}

impl UnitArt {
    pub fn new(assets: PathBuf) -> Self {
        Self { assets }
    }
}

/// A land or sea unit centered on `at`. `radius` is about 0.42 hex radii for
/// the military unit and 0.3 for civilians (which share the tile's bottom row).
#[allow(clippy::too_many_arguments)]
pub fn draw_unit(
    _art: &mut UnitArt,
    icons: &mut Icons,
    painter: &egui::Painter,
    game: &wayciv_core::Game,
    u: &wayciv_core::state::Unit,
    at: Pos2,
    radius: f32,
    selected: bool,
) {
    let owner = &game.players[u.owner];
    let primary = rgb(owner.primary);
    let secondary = rgb(owner.secondary);
    if selected {
        painter.circle_stroke(at, radius + 4.0, Stroke::new(3.0, Color32::WHITE));
    }
    let ring = if u.moves > 0 {
        secondary
    } else {
        secondary.gamma_multiply(0.5)
    };
    painter.circle(
        at,
        radius,
        primary,
        Stroke::new((radius * 0.15).max(1.5), ring),
    );
    let ty = game.unit_type(u);
    let inner = radius * 0.82;
    let drew = radius > 9.0
        && icons.paint(
            painter,
            "unit",
            &ty.id,
            Rect::from_center_size(at, vec2(inner * 2.0, inner * 2.0)),
        );
    if !drew {
        painter.text(
            at,
            Align2::CENTER_CENTER,
            abbreviation(&ty.name),
            FontId::proportional(radius * 0.85),
            secondary,
        );
    }
    // Civilians have no health bar: they are alive or dead.
    if u.hp < 100 && game.is_military(u) {
        let bar = Rect::from_min_size(at + vec2(-radius, radius + 2.0), vec2(radius * 2.0, 3.0));
        painter.rect_filled(bar, 1.0, Color32::from_gray(30));
        let color = if u.hp > 60 {
            Color32::from_rgb(80, 210, 90)
        } else if u.hp > 30 {
            Color32::from_rgb(230, 200, 60)
        } else {
            Color32::from_rgb(220, 60, 50)
        };
        painter.rect_filled(
            Rect::from_min_size(bar.min, vec2(radius * 2.0 * u.hp as f32 / 100.0, 3.0)),
            1.0,
            color,
        );
    }
    if u.fortified {
        painter.rect_filled(
            Rect::from_center_size(
                at + vec2(radius * 0.8, -radius * 0.8),
                vec2(radius * 0.5, radius * 0.5),
            ),
            2.0,
            Color32::from_rgb(200, 200, 200),
        );
    }
}

/// An aircraft at its base: a small marker at `at` of size `r`.
pub fn draw_aircraft(
    _art: &mut UnitArt,
    _icons: &mut Icons,
    painter: &egui::Painter,
    game: &wayciv_core::Game,
    u: &wayciv_core::state::Unit,
    at: Pos2,
    r: f32,
    selected: bool,
) {
    let owner = &game.players[u.owner];
    let diamond = vec![
        at + vec2(0.0, -r),
        at + vec2(r, 0.0),
        at + vec2(0.0, r),
        at + vec2(-r, 0.0),
    ];
    if selected {
        painter.circle_stroke(at, r + 4.0, Stroke::new(2.5, Color32::WHITE));
    }
    let ring = if u.moves > 0 {
        rgb(owner.secondary)
    } else {
        rgb(owner.secondary).gamma_multiply(0.5)
    };
    painter.add(Shape::convex_polygon(
        diamond,
        rgb(owner.primary),
        Stroke::new(1.5, ring),
    ));
    let name = &game.unit_type(u).name;
    painter.text(
        at,
        Align2::CENTER_CENTER,
        abbreviation(name),
        FontId::proportional(r * 0.8),
        rgb(owner.secondary),
    );
}

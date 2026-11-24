//! On-map strategic overlays:
//! - Tile yield icons (Food, Production, Gold, Science, Culture, Faith)
//! - Map lenses (Settler with fresh-water & recommended spots, Empire, Religion, Resources)
//! - Construction blueprint markers for districts/wonders currently being built
//! - Worked-tile citizen markers when a city is selected

use super::hex::{corners, hex};
use super::overlay::{self, lens_tile};
use crate::palette::{self, BAD, GOLD, GOOD, TEAL, TEXT, YIELD_COLORS, rgb, with_alpha};
use crate::ui::glyph::{self, Glyph};
use egui::{Align2, Color32, Mesh, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, pos2, vec2};
use std::collections::HashSet;
use wayciv_core::state::Production;
use wayciv_core::{CityId, Game, PlayerId, Pos};

/// Active strategic map lens.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MapLens {
    #[default]
    None,
    /// Fresh water, coastal, dry, and blocked settle sites (auto-active for Settlers).
    Settler,
    /// Vivid civilization territory tinting for clear political borders.
    Empire,
    /// Majority religion colors over cities and territories.
    Religion,
    /// Highlights unimproved resources and unexplored tribal villages.
    Resources,
}

impl MapLens {
    pub fn name(self) -> &'static str {
        match self {
            MapLens::None => "Normal",
            MapLens::Settler => "Settler Lens",
            MapLens::Empire => "Empire Lens",
            MapLens::Religion => "Religion Lens",
            MapLens::Resources => "Resources & Villages",
        }
    }
}

/// Visual display options for the map (toggled via hotkeys, minimap bar, or Settings).
#[derive(Clone, Copy, Debug)]
pub struct MapSettings {
    pub show_yields: bool,
    pub show_resources: bool,
    pub grid: overlay::GridMode,
    pub lens: MapLens,
    pub atmosphere: bool,
    pub use_3d: bool,
}

impl Default for MapSettings {
    fn default() -> Self {
        Self {
            show_yields: true,
            show_resources: true,
            grid: overlay::GridMode::Subtle,
            lens: MapLens::None,
            atmosphere: true,
            use_3d: std::env::var_os("WAYCIV_3D").is_some(),
        }
    }
}

/// Computes the effective lens (Settler Lens auto-engages when a Settler is selected
/// and no manual lens is overriding it).
pub fn effective_lens(settings: &MapSettings, game: &Game, selected_unit: Option<wayciv_core::UnitId>) -> MapLens {
    if settings.lens != MapLens::None {
        return settings.lens;
    }
    if let Some(id) = selected_unit
        && let Some(u) = game.units.get(&id)
        && u.owner == game.human()
        && game.unit_type(u).found_city
    {
        return MapLens::Settler;
    }
    MapLens::None
}

/// Draws the active Map Lens over the visible tiles, returning recommended city
/// positions (for drawing star markers above terrain/features) if Settler Lens is active.
pub fn draw_lens(
    painter: &Painter,
    game: &Game,
    viewer: PlayerId,
    lens: MapLens,
    tiles: &[(Pos, Pos2)],
    z: f32,
    now: f32,
) -> Vec<(Pos, Pos2)> {
    if lens == MapLens::None {
        return Vec::new();
    }
    let mut mesh = Mesh::default();
    let mut recommended: Vec<(Pos, Pos2)> = Vec::new();
    let pulse = 0.5 + 0.5 * (now * 2.8).sin();

    match lens {
        MapLens::None => {}
        MapLens::Settler => {
            // Rank top settle spots on screen so we can mark the best 3 with gold stars.
            let mut scored: Vec<(i32, Pos, Pos2)> = Vec::new();
            let mut seen_pos: HashSet<Pos> = HashSet::new();
            for &(pos, c) in tiles {
                if game.map.is_water(&game.rules, pos) {
                    continue;
                }
                match game.can_settle_pos(viewer, pos) {
                    Ok(water) => {
                        let (fill, rim) = if water >= 5 {
                            (
                                Color32::from_rgba_unmultiplied(46, 206, 108, 92),
                                Color32::from_rgba_unmultiplied(110, 245, 160, 170),
                            )
                        } else if water >= 3 {
                            (
                                Color32::from_rgba_unmultiplied(52, 184, 212, 82),
                                Color32::from_rgba_unmultiplied(110, 224, 245, 150),
                            )
                        } else {
                            (
                                Color32::from_rgba_unmultiplied(210, 178, 86, 54),
                                Color32::from_rgba_unmultiplied(230, 198, 108, 110),
                            )
                        };
                        lens_tile(&mut mesh, c, z, fill, rim);
                        if seen_pos.insert(pos) && game.map.tile(pos).owner.is_none() {
                            let score = settle_site_score(game, viewer, pos, water);
                            if score > 22 {
                                scored.push((score, pos, c));
                            }
                        }
                    }
                    Err(reason) => {
                        if reason != "Impassable" {
                            // Subtle cool crimson outline + whisper-soft tint so blocked
                            // tiles stay legible without muddying lush green forests!
                            lens_tile(
                                &mut mesh,
                                c,
                                z,
                                Color32::from_rgba_unmultiplied(192, 46, 54, 30),
                                Color32::from_rgba_unmultiplied(238, 86, 78, 92),
                            );
                        }
                    }
                }
            }
            scored.sort_by_key(|&(s, _, _)| std::cmp::Reverse(s));
            // Pick up to 3 well-spaced recommended spots.
            for &(_, pos, c) in &scored {
                if recommended.len() >= 3 {
                    break;
                }
                if recommended
                    .iter()
                    .all(|&(p, _)| game.map.grid.distance(p, pos) >= 3)
                {
                    recommended.push((pos, c));
                }
            }
        }
        MapLens::Empire => {
            for &(pos, c) in tiles {
                if let Some(owner) = game.map.tile(pos).owner {
                    let p = &game.players[owner];
                    let col = rgb(p.primary);
                    let sec = rgb(p.secondary);
                    lens_tile(&mut mesh, c, z, with_alpha(col, 104), with_alpha(sec, 95));
                }
            }
        }
        MapLens::Religion => {
            for &(pos, c) in tiles {
                let tile = game.map.tile(pos);
                let religion = tile
                    .owning_city
                    .and_then(|cid| game.cities.get(&cid))
                    .and_then(|city| city.majority_religion());
                if let Some(r) = religion {
                    let rc = game.rules.content.religions[r].color;
                    let col = if rc.iter().all(|&v| v > 200) {
                        YIELD_COLORS[5]
                    } else {
                        rgb(rc)
                    };
                    lens_tile(&mut mesh, c, z, with_alpha(col, 105), with_alpha(col, 180));
                }
            }
        }
        MapLens::Resources => {
            let player = &game.players[viewer];
            for &(pos, c) in tiles {
                let tile = game.map.tile(pos);
                if game.villages.contains(&pos) {
                    lens_tile(
                        &mut mesh,
                        c,
                        z,
                        Color32::from_rgba_unmultiplied(255, 212, 84, (80.0 + 45.0 * pulse) as u8),
                        GOLD,
                    );
                    continue;
                }
                if let Some(r) = tile.resource {
                    let res = &game.rules.content.resources[r];
                    if player.meets(&game.rules, res.prereq_tech.as_deref()) {
                        let improved = tile.improvement.is_some() && !tile.pillaged;
                        let base = match res.class.as_str() {
                            "RESOURCECLASS_LUXURY" => Color32::from_rgb(196, 114, 236),
                            "RESOURCECLASS_STRATEGIC" => Color32::from_rgb(242, 92, 78),
                            _ => Color32::from_rgb(106, 212, 98),
                        };
                        let a = if improved {
                            52
                        } else {
                            (84.0 + 44.0 * pulse) as u8
                        };
                        lens_tile(&mut mesh, c, z, with_alpha(base, a), with_alpha(base, if improved { 120 } else { 235 }));
                    }
                }
            }
        }
    }
    if !mesh.is_empty() {
        painter.add(Shape::mesh(mesh));
    }
    recommended
}

/// Draws golden recommended-site star medallions (above terrain/features).
pub fn draw_recommended_sites(painter: &Painter, spots: &[(Pos, Pos2)], z: f32, now: f32) {
    if spots.is_empty() || z < 16.0 {
        return;
    }
    let pulse = 0.5 + 0.5 * (now * 3.0).sin();
    let r = (z * 0.25).clamp(8.0, 15.0);
    for &(_, c) in spots {
        let p = c + vec2(0.0, -z * 0.26 - 2.0 * pulse);
        painter.circle_filled(p + vec2(0.0, 2.0), r + 1.5, Color32::from_black_alpha(130));
        painter.circle_filled(p, r, Color32::from_rgb(18, 36, 28));
        painter.circle_stroke(p, r, Stroke::new(2.0, GOLD));
        glyph::paint(painter, Glyph::Star, p, r * 0.62, GOLD);
    }
}

/// Heuristic quality score of founding a city on `pos` (sums 1-ring yields + water + resources).
fn settle_site_score(game: &Game, viewer: PlayerId, pos: Pos, water: i32) -> i32 {
    let mut score = water * 4;
    for p in game.map.grid.within(pos, 1) {
        let y = game.tile_yields(viewer, p);
        score += y[0] * 3 + y[1] * 3 + y[2] + y[3] * 2 + y[4] * 2 + y[5];
        if game.map.tile(p).resource.is_some() {
            score += 4;
        }
    }
    score
}

/// Draws crisp Civ VI-style tile yield pips along the lower arc of revealed tiles.
pub fn draw_tile_yields(
    painter: &Painter,
    game: &Game,
    viewer: PlayerId,
    tiles: &[(Pos, Pos2)],
    z: f32,
) {
    if z < 23.0 {
        return;
    }
    let fade = ((z - 23.0) / 10.0).clamp(0.0, 1.0);

    for &(pos, c) in tiles {
        // Skip city centers (banner & city art own the tile).
        if game.city_at(pos).is_some() {
            continue;
        }
        let y = game.tile_yields(viewer, pos);
        let mut entries: Vec<(usize, i32)> = Vec::with_capacity(6);
        for (i, &val) in y.iter().enumerate() {
            if val > 0 {
                entries.push((i, val));
            }
        }
        if entries.is_empty() {
            continue;
        }
        // Slightly soften yields under an occupying visible unit so the unit stays clear.
        let has_unit = game.players[viewer].visible[game.map.grid.index(pos)]
            && game.units_at(pos).next().is_some();
        let tile_fade = fade * if has_unit { 0.78 } else { 1.0 };
        let count = entries.len();
        let r = (z * 0.132).clamp(4.8, 8.2);
        let step = r * 2.18;
        let base_y = c.y + z * 0.53;
        let start_x = c.x - (count as f32 - 1.0) * step * 0.5;
        for (idx, (yi, val)) in entries.into_iter().enumerate() {
            let p = pos2(start_x + idx as f32 * step, base_y);
            // Civ VI embossed circular yield medallions: rich saturated fill,
            // soft drop shadow, glossy top sheen, dark metallic rim, crisp icon/numeral.
            let disc_col = match yi {
                0 => Color32::from_rgb(76, 184, 64),   // Food emerald
                1 => Color32::from_rgb(228, 126, 36),  // Production terracotta-orange
                2 => Color32::from_rgb(236, 188, 36),  // Gold coin
                3 => Color32::from_rgb(48, 158, 232),  // Science cerulean
                4 => Color32::from_rgb(164, 78, 206),  // Culture royal violet
                _ => Color32::from_rgb(214, 226, 240), // Faith pearl
            }
            .gamma_multiply(tile_fade);
            let dark_edge = Color32::from_rgba_unmultiplied(12, 16, 22, (220.0 * tile_fade) as u8);
            // Soft grounded shadow under medallion.
            painter.circle_filled(
                p + vec2(0.0, 1.3),
                r + 0.6,
                Color32::from_black_alpha((135.0 * tile_fade) as u8),
            );
            // Rich colored medallion body.
            painter.circle_filled(p, r, disc_col);
            // Glossy upper specular highlight.
            painter.circle_filled(
                p + vec2(0.0, -r * 0.28),
                r * 0.62,
                Color32::from_white_alpha((58.0 * tile_fade) as u8),
            );
            // Crisp metallic dark outer rim.
            painter.circle_stroke(p, r, Stroke::new(1.15, dark_edge));
            let fg = if yi == 5 {
                Color32::from_rgb(28, 42, 68).gamma_multiply(tile_fade)
            } else {
                Color32::WHITE.gamma_multiply(tile_fade)
            };
            if val == 1 {
                glyph::paint(
                    painter,
                    crate::ui::theme::yield_glyph(yi),
                    p + vec2(0.0, 0.7),
                    r * 0.64,
                    Color32::from_black_alpha((110.0 * tile_fade) as u8),
                );
                glyph::paint(painter, crate::ui::theme::yield_glyph(yi), p, r * 0.64, fg);
            } else {
                // Faint embossed icon behind crisp shadowed bold numeral (Civ VI style).
                glyph::paint(
                    painter,
                    crate::ui::theme::yield_glyph(yi),
                    p,
                    r * 0.62,
                    Color32::from_black_alpha((48.0 * tile_fade) as u8),
                );
                let txt = val.to_string();
                let font = palette::bold((r * 1.42).max(9.0));
                painter.text(
                    p + vec2(0.0, 0.9),
                    Align2::CENTER_CENTER,
                    &txt,
                    font.clone(),
                    Color32::from_black_alpha((185.0 * tile_fade) as u8),
                );
                painter.text(p, Align2::CENTER_CENTER, txt, font, fg);
            }
        }
    }
}

/// Draws architectural blueprint scaffolding + progress ring on any tile where
/// one of the player's cities is currently constructing a District or Wonder.
pub fn draw_construction_sites(
    painter: &Painter,
    game: &Game,
    viewer: PlayerId,
    screens_fn: impl Fn(Pos) -> Vec<Pos2>,
    z: f32,
    now: f32,
) {
    if z < 14.0 {
        return;
    }
    let pulse = 0.5 + 0.5 * (now * 2.2).sin();
    for city in game.player_cities(viewer) {
        let Some(item) = &city.production else { continue };
        let (pos, is_wonder, label) = match item {
            Production::District(d, p) => (*p, false, game.rules.content.districts[*d].name.as_str()),
            Production::Wonder(w, p) => (*p, true, game.rules.content.buildings[*w].name.as_str()),
            _ => continue,
        };
        if pos == city.pos {
            continue;
        }
        let cost = game.production_cost(viewer, item).max(1);
        let done = city
            .progress
            .get(&Game::production_key(item))
            .copied()
            .unwrap_or(0);
        let frac = (done as f32 / cost as f32).clamp(0.0, 1.0);
        let accent = if is_wonder { GOLD } else { TEAL };
        for c in screens_fn(pos) {
            painter.add(hex(
                c,
                z * 0.86,
                with_alpha(accent, (26.0 + 18.0 * pulse) as u8),
                Stroke::new((z * 0.055).clamp(1.5, 2.8), with_alpha(accent, 210)),
            ));
            let r = (z * 0.30).clamp(8.5, 16.0);
            painter.circle_filled(c, r + 1.5, Color32::from_black_alpha(150));
            painter.circle_filled(c, r, Color32::from_rgb(14, 24, 38));
            crate::ui::theme::ring(painter, c, r - 1.5, frac, 2.8, accent);
            glyph::paint(
                painter,
                if is_wonder { Glyph::Star } else { Glyph::Build },
                c,
                r * 0.52,
                accent,
            );
            if z >= 28.0 {
                let tag_c = c + vec2(0.0, -(r + 11.0));
                let galley = painter.layout_no_wrap(
                    label.to_string(),
                    palette::bold((z * 0.24).clamp(10.0, 13.0)),
                    TEXT,
                );
                let pill = Rect::from_center_size(tag_c, galley.size() + vec2(12.0, 5.0));
                painter.rect_filled(pill, pill.height() * 0.5, Color32::from_black_alpha(185));
                painter.rect_stroke(
                    pill,
                    pill.height() * 0.5,
                    Stroke::new(1.0, with_alpha(accent, 180)),
                    StrokeKind::Inside,
                );
                painter.galley(tag_c - galley.size() * 0.5, galley, TEXT);
            }
        }
    }
}

/// Draws crisp Civ VI-style worked citizen icons on tiles worked by the selected city.
pub fn draw_worked_citizens(
    painter: &Painter,
    game: &Game,
    city_id: CityId,
    screens_fn: impl Fn(Pos) -> Vec<Pos2>,
    z: f32,
) {
    let Some(city) = game.cities.get(&city_id) else {
        return;
    };
    let r = (z * 0.23).clamp(7.0, 14.0);
    for &pos in &city.worked {
        for c in screens_fn(pos) {
            let at = c + vec2(0.0, -z * 0.14);
            painter.circle_filled(at + vec2(0.0, 2.0), r + 1.5, Color32::from_black_alpha(120));
            painter.circle_filled(at, r, Color32::from_rgb(16, 46, 36));
            painter.circle_stroke(at, r, Stroke::new(1.8, GOOD));
            glyph::paint(painter, Glyph::Population, at, r * 0.58, Color32::WHITE);
        }
    }
    // Also subtly outline the city's 3-ring workable boundary for owned tiles.
    let mut edge_mesh = Mesh::default();
    for pos in game.city_tiles(city_id) {
        if pos == city.pos {
            continue;
        }
        for c in screens_fn(pos) {
            let pts = corners(c, z * 0.96);
            super::mesh::outline(
                &mut edge_mesh,
                &pts,
                1.1,
                Color32::from_rgba_unmultiplied(255, 240, 180, 44),
            );
        }
    }
    if !edge_mesh.is_empty() {
        painter.add(Shape::mesh(edge_mesh));
    }
}

/// Returns the accent color for a district's primary adjacency yield.
pub fn district_yield_color(game: &Game, district: usize) -> Color32 {
    let d = &game.rules.content.districts[district];
    if let Some(first) = d.adjacencies.first() {
        return YIELD_COLORS[first.yield_index.min(5)];
    }
    if d.id == "DISTRICT_DATACENTER" {
        return crate::palette::COMPUTE;
    }
    GOOD
}

/// Draws a subtle pulsing warning ring around enemy cities or units in bombard range.
pub fn draw_bombard_targets(
    painter: &Painter,
    game: &Game,
    city_id: CityId,
    screens_fn: impl Fn(Pos) -> Vec<Pos2>,
    z: f32,
    now: f32,
) {
    let Some(city) = game.cities.get(&city_id) else {
        return;
    };
    let pulse = 0.5 + 0.5 * (now * 3.4).sin();
    let mut marks = Mesh::default();
    for pos in game.map.grid.within(city.pos, 2) {
        if game.city_can_attack(city_id, pos) {
            for c in screens_fn(pos) {
                super::mesh::fill(
                    &mut marks,
                    &corners(c, z * 0.97),
                    Color32::from_rgba_unmultiplied(230, 50, 40, (32.0 + 28.0 * pulse) as u8),
                );
                overlay::mark(
                    &mut marks,
                    c,
                    z,
                    Color32::from_rgba_unmultiplied(245, 72, 56, (120.0 + 70.0 * pulse) as u8),
                    BAD,
                    2.6,
                );
            }
        }
    }
    if !marks.is_empty() {
        painter.add(Shape::mesh(marks));
    }
}

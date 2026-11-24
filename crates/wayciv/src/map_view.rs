//! The hex map: what to draw where and in which order, plus mouse and
//! keyboard input. The art itself lives in `render`; what is moving or
//! fading right now comes from `render::anim` (the `Anims` in the session).

use crate::hud::Session;
use crate::palette::{self, with_alpha};
use crate::render::anim::{ease_out, ease_out_back, smooth};
use crate::render::hex::{SQRT3, corners, hex};
use crate::render::{mesh, overlay};
use crate::render::units::UnitFx;
use crate::render::{self, Art, TileCtx};
use egui::{Align2, Color32, Mesh, Pos2, Rect, Sense, Shape, Stroke, Vec2, vec2};
use std::borrow::Cow;
use std::collections::HashMap;
use wayciv_core::state::Unit;
use wayciv_core::{Game, Pos, Production};

pub use crate::render::hex::world;

pub struct Camera {
    /// Center in world units (hex radius 1).
    pub center: Vec2,
    /// Hex radius in pixels.
    pub zoom: f32,
    /// Target hex radius for smooth cursor-anchored zoom easing.
    pub target_zoom: f32,
    /// Screen point anchored during mouse-wheel zoom easing.
    pub zoom_anchor: Option<Pos2>,
}

/// Screen positions where tile `pos` appears (east-west wrap copies).
fn screens(s: &Session, rect: Rect, pos: Pos) -> Vec<Pos2> {
    let zoom = s.cam.zoom;
    let width = SQRT3 * s.game.map.grid.width as f32;
    let base = world(pos) - s.cam.center;
    let mut out = Vec::new();
    let reach = (rect.width() / (width * zoom)).ceil() as i32 + 1;
    for k in -reach..=reach {
        let p = rect.center() + (base + vec2(width * k as f32, 0.0)) * zoom;
        if p.x > rect.left() - zoom * 2.0
            && p.x < rect.right() + zoom * 2.0
            && p.y > rect.top() - zoom * 2.0
            && p.y < rect.bottom() + zoom * 2.0
        {
            out.push(p);
        }
    }
    out
}

/// The tile under a screen point.
pub fn tile_at(s: &Session, rect: Rect, point: Pos2) -> Option<Pos> {
    let w = (point - rect.center()) / s.cam.zoom + s.cam.center;
    let q = (SQRT3 / 3.0 * w.x - w.y / 3.0) as f64;
    let r = (2.0 / 3.0 * w.y) as f64;
    let (x, z) = (q, r);
    let y = -x - z;
    let (mut rx, ry, mut rz) = (x.round(), y.round(), z.round());
    let (dx, dy, dz) = ((rx - x).abs(), (ry - y).abs(), (rz - z).abs());
    if dx > dy && dx > dz {
        rx = -ry - rz;
    } else if dy <= dz {
        rz = -rx - ry;
    }
    s.game
        .map
        .grid
        .normalize(Pos::from_axial(rx as i32, rz as i32))
}

/// Eases the camera to `pos`.
pub fn center_on(s: &mut Session, pos: Pos) {
    let width = SQRT3 * s.game.map.grid.width as f32;
    s.anim.pan_to(s.cam.center, world(pos), width);
}

/// Eases the camera so a city sits centered in the visible map area left of the City Panel.
pub fn center_on_city(s: &mut Session, pos: Pos) {
    let width = SQRT3 * s.game.map.grid.width as f32;
    let panel_shift_px = (s.view.width() * 0.22).clamp(150.0, 235.0);
    let target = world(pos) + vec2(panel_shift_px / s.cam.zoom.max(12.0), 0.0);
    s.anim.pan_to(s.cam.center, target, width);
}

/// Puts the camera on `pos` at once (a new game, test hooks).
pub fn snap_to(s: &mut Session, pos: Pos) {
    s.anim.cancel_pan();
    s.cam.center = world(pos);
}

/// Snaps the camera so a city sits centered left of the City Panel (used by test hooks).
pub fn snap_to_city(s: &mut Session, pos: Pos) {
    s.anim.cancel_pan();
    let panel_shift_px = (s.view.width() * 0.22).clamp(150.0, 235.0);
    s.cam.center = world(pos) + vec2(panel_shift_px / s.cam.zoom.max(12.0), 0.0);
}

pub fn show(s: &mut Session, ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(rect, Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::from_rgb(8, 10, 14));

    // Advance the animation clock, learn what the engine did since the last
    // frame, and let a camera pan move the view (input cancels it).
    s.anim.begin();
    s.anim.sync(&mut s.game);
    if let Some(center) = s.anim.pan_center() {
        s.cam.center = center;
    }
    handle_input(s, ui, rect, &response);
    let started = std::time::Instant::now();
    let mut icons = std::mem::replace(&mut s.icons, crate::icons::Icons::new(Default::default()));
    let mut art = std::mem::replace(&mut s.art, Art::new(Default::default()));
    let mut banners_out = Vec::new();
    let mut facing_updates = Vec::new();
    let alive = draw(
        s,
        rect,
        &response,
        &painter,
        &mut icons,
        &mut art,
        &mut banners_out,
        &mut facing_updates,
    );
    s.icons = icons;
    s.art = art;
    s.city_banners = banners_out;
    for (uid, left) in facing_updates {
        s.unit_facing_left.insert(uid, left);
    }
    // Rich structured tile tooltip (drawn after restoring s.icons so resource icons paint!).
    if let Some(pos) = s.hover
        && s.game.players[s.me()].revealed[s.game.map.grid.index(pos)]
        && response.hovered()
    {
        response.clone().on_hover_ui_at_pointer(|ui| {
            crate::ui::tactical::tile_tooltip_ui(s, ui, pos);
        });
    }
    s.anim.frame_cost = started.elapsed().as_secs_f32();

    // Repaint while something plays; ambient life (water, rivers, idle
    // breathing, drifting clouds) ticks at a modest rate that backs off
    // when frames are slow.
    let ctx = ui.ctx();
    if s.anim.active() {
        ctx.request_repaint();
    } else if alive {
        let gap = (4.0 * s.anim.frame_cost).clamp(AMBIENT_GAP, 0.25);
        ctx.request_repaint_after(std::time::Duration::from_secs_f32(gap));
    }
}

/// Seconds between ambient repaints at best (about 25 frames a second).
const AMBIENT_GAP: f32 = 0.04;

/// The rows of tiles that can be on screen, with a margin.
fn visible_rows(s: &Session, rect: Rect) -> (i32, i32) {
    let half = rect.height() / 2.0 / s.cam.zoom + 2.0;
    let top = ((s.cam.center.y - half) / 1.5).floor() as i32 - 1;
    let bottom = ((s.cam.center.y + half) / 1.5).ceil() as i32 + 1;
    (top.max(0), bottom.min(s.game.map.grid.height - 1))
}

/// Where a unit stands on its tile: the offset of its center from the tile
/// center and its size, both in hex radii.
#[derive(Clone, Copy)]
struct Slot {
    off: Vec2,
    radius: f32,
    air: bool,
}

/// The slots of the units on one tile, in order: the military unit in the
/// middle, civilians along the bottom, aircraft across the top.
fn slots(game: &Game, units: &[&Unit]) -> Vec<Slot> {
    let (mut civilians, mut aircraft) = (0, 0);
    // In a city the garrison stands aside in the lower corner, a little
    // smaller, so the city itself stays in view.
    let city = units.first().is_some_and(|u| game.city_at(u.pos).is_some());
    units
        .iter()
        .map(|u| {
            if game.is_air(u) {
                let off = vec2(-0.45 + 0.3 * aircraft as f32, -0.62);
                aircraft += 1;
                Slot {
                    off,
                    radius: 0.22,
                    air: true,
                }
            } else if game.is_military(u) {
                Slot {
                    off: if city { vec2(0.4, 0.36) } else { vec2(0.0, -0.05) },
                    radius: if city { 0.34 } else { 0.42 },
                    air: false,
                }
            } else {
                civilians += 1;
                Slot {
                    off: vec2(-0.45 + 0.25 * ((civilians - 1) % 4) as f32, if city { 0.5 } else { 0.4 }),
                    radius: if city { 0.26 } else { 0.3 },
                    air: false,
                }
            }
        })
        .collect()
}

/// Where `unit` stands on `tile` among the units there (as if it were there).
fn walking_slot(game: &Game, by_tile: &HashMap<Pos, Vec<&Unit>>, unit: &Unit, tile: Pos) -> Slot {
    let mut here: Vec<&Unit> = by_tile
        .get(&tile)
        .map_or(Vec::new(), |v| v.iter().copied().filter(|u| u.id != unit.id).collect());
    here.push(unit);
    slots(game, &here)[here.len() - 1]
}

/// Draws the whole map; returns whether anything ambient is on screen (so
/// the caller keeps repainting at a low rate).
#[allow(clippy::too_many_arguments)]
fn draw(
    s: &Session,
    rect: Rect,
    response: &egui::Response,
    painter: &egui::Painter,
    icons: &mut crate::icons::Icons,
    art: &mut Art,
    banners_out: &mut Vec<(wayciv_core::CityId, Rect)>,
    facing_updates: &mut Vec<(wayciv_core::UnitId, bool)>,
) -> bool {
    let game = &s.game;
    let anim = &s.anim;
    let now = anim.now as f32;
    let me = game.human();
    let player = &game.players[me];
    let grid = game.map.grid;
    let z = s.cam.zoom;
    let ambient = z >= 10.0;
    let ctx = |pos: Pos, c: Pos2| TileCtx {
        game,
        viewer: me,
        pos,
        c,
        z,
    };
    art.terrain.set_clock(now, ambient);

    // 0. The unexplored world, under everything.
    render::veil::unknown(&mut art.veil, painter, rect, s.cam.center, z, now);

    // 1. Terrain: biomes, relief, features, cliffs.
    let (row0, row1) = visible_rows(s, rect);
    let mut tiles: Vec<(Pos, Pos2)> = Vec::new();
    for y in row0..=row1 {
        for x in 0..grid.width {
            let pos = Pos::new(x, y);
            if !player.revealed[grid.index(pos)] {
                continue;
            }
            for c in screens(s, rect, pos) {
                tiles.push((pos, c));
            }
        }
    }
    let use_3d = s.map_settings.use_3d
        && render::view3d::draw_terrain_pass(&mut art.pack3d, &art.assets_dir, painter, game, me, &tiles, z);
    if !use_3d {
        for &(pos, c) in &tiles {
            render::terrain::draw_tile(&mut art.terrain, painter, &ctx(pos, c));
        }
        render::terrain::finish(&mut art.terrain, painter);

        // 2. Rivers, then roads.
        for &(pos, c) in &tiles {
            render::terrain::draw_river(&mut art.terrain, painter, &ctx(pos, c));
        }
    }
    for &(pos, c) in &tiles {
        render::resources::draw_road(&mut art.resources, painter, &ctx(pos, c));
    }

    // 2b. The hex grid, delicate by default so the painted world shines through.
    let strength = ((z - 12.0) / 14.0).clamp(0.0, 1.0);
    if strength > 0.0 && s.map_settings.grid != overlay::GridMode::Off {
        let mut layer = Mesh::default();
        for &(pos, c) in &tiles {
            overlay::grid(&mut layer, &ctx(pos, c), strength, s.map_settings.grid);
        }
        painter.add(Shape::mesh(layer));
    }

    // 2c. Active strategic map lens (Settler auto-lens, Empire, Religion, Resources).
    let active_lens = render::yields::effective_lens(&s.map_settings, game, s.selected_unit);
    let recommended_spots =
        render::yields::draw_lens(painter, game, me, active_lens, &tiles, z, now);

    // 3. Territory: tint and border ribbons (new ones fade in), then the
    // soft edge where the explored world meets the unknown.
    let mut layer = Mesh::default();
    for &(pos, c) in &tiles {
        overlay::territory(&mut layer, &ctx(pos, c), anim.claim(pos));
    }
    painter.add(Shape::mesh(layer));
    let mut layer = Mesh::default();
    for &(pos, c) in &tiles {
        render::veil::frontier(&mut layer, &ctx(pos, c));
    }
    painter.add(Shape::mesh(layer));

    // 4. Districts, wonders, improvements, resources; fog on top. New
    // districts and improvements pop in; fog and newly seen tiles fade.
    for &(pos, c) in &tiles {
        let t = ctx(pos, c);
        let seen = player.visible[grid.index(pos)];
        if use_3d {
            let k = anim.pop(pos).map_or(1.0, ease_out_back);
            if k > 0.04 {
                render::view3d::draw_tile_props(&art.pack3d, painter, &t, k);
            }
        } else {
            match anim.pop(pos) {
                None => {
                    render::buildings::draw_district(&mut art.buildings, painter, &t);
                    render::resources::draw_improvement(&mut art.resources, painter, &t);
                }
                Some(p) => {
                    let k = ease_out_back(p);
                    if k > 0.04 {
                        let scaled = TileCtx { z: z * k, ..t };
                        render::buildings::draw_district(&mut art.buildings, painter, &scaled);
                        render::resources::draw_improvement(&mut art.resources, painter, &scaled);
                    }
                }
            }
            if s.map_settings.show_resources {
                render::resources::draw_resource(&mut art.resources, icons, painter, &t);
            }
        }
        let fog = anim.fog(pos, seen);
        if fog > 0.01 {
            painter.add(render::veil::fog(&t, fog));
        }
        let shown = anim.reveal(pos);
        if shown < 0.999 {
            painter.add(hex(
                c,
                z * 1.03,
                with_alpha(render::veil::UNKNOWN, (255.0 * (1.0 - shown)) as u8),
                Stroke::NONE,
            ));
        }
    }
    // Recommended settle spots & on-map construction blueprints & tile yields.
    render::yields::draw_recommended_sites(painter, &recommended_spots, z, now);
    render::yields::draw_construction_sites(painter, game, me, |p| screens(s, rect, p), z, now);
    if s.map_settings.show_yields {
        render::yields::draw_tile_yields(painter, game, me, &tiles, z);
    }
    if s.map_settings.atmosphere {
        if ambient {
            render::veil::cloud_shadows(painter, rect, s.cam.center, z, now);
        }
        render::veil::lighting(painter, rect);
    }

    // 5. Highlights: the selected tile, reachable tiles, attack targets,
    // building sites and the path. They breathe gently.
    let pulse = 0.5 + 0.5 * (now * 2.6).sin();
    let alpha = |base: f32, swing: f32| (base + swing * pulse) as u8;
    let mut marks = Mesh::default();
    let mut path: Vec<Pos2> = Vec::new();
    let mut path_turns: Vec<i32> = Vec::new();
    let mut path_attack = false;
    let mut badges: Vec<(Pos2, f32, Color32, bool)> = Vec::new();
    if let Some(unit) = s.selected_unit.and_then(|id| game.units.get(&id)) {
        for &pos in &s.reachable {
            for c in screens(s, rect, pos) {
                overlay::mark(
                    &mut marks,
                    c,
                    z,
                    Color32::from_rgba_unmultiplied(110, 200, 255, alpha(44.0, 26.0)),
                    Color32::from_rgba_unmultiplied(176, 230, 255, alpha(130.0, 50.0)),
                    1.4,
                );
            }
        }
        for pos in grid.within(unit.pos, game.unit_type(unit).range.max(1)) {
            if game.can_attack(unit, pos) {
                for c in screens(s, rect, pos) {
                    mesh::fill(&mut marks, &corners(c, z * 0.97), Color32::from_rgba_unmultiplied(220, 40, 30, alpha(22.0, 20.0)));
                    overlay::mark(
                        &mut marks,
                        c,
                        z,
                        Color32::from_rgba_unmultiplied(236, 54, 40, alpha(100.0, 70.0)),
                        Color32::from_rgb(255, 98, 82),
                        2.4,
                    );
                }
            }
        }
        if let Some(goal) = s.hover
            && game.can_attack(unit, goal)
        {
            path_attack = true;
            if let (Some(&a), Some(b)) = (
                screens(s, rect, unit.pos).first(),
                screens(s, rect, goal).into_iter().min_by_key(|p| {
                    screens(s, rect, unit.pos)
                        .first()
                        .map(|q| (p.distance(*q) * 10.0) as i32)
                        .unwrap_or(0)
                }),
            ) {
                path.push(a);
                path.push(b);
            }
        } else if !s.path_preview.is_empty() {
            path_turns = game.path_step_turns(unit, &s.path_preview);
            let mut prev = screens(s, rect, unit.pos).first().copied();
            path.extend(prev);
            for &step in &s.path_preview {
                let here = screens(s, rect, step)
                    .into_iter()
                    .min_by_key(|p| prev.map(|q| (p.distance(q) * 10.0) as i32).unwrap_or(0));
                path.extend(here);
                prev = here;
            }
        }
    }
    if let Some(item) = &s.placing
        && let Some(city) = s.selected_city
    {
        let sites = match item {
            Production::District(d, _) => game.district_sites(city, *d),
            Production::Wonder(w, _) => game.wonder_sites(city, *w),
            _ => Vec::new(),
        };
        let accent = match item {
            Production::District(d, _) => render::yields::district_yield_color(game, *d),
            _ => palette::GOLD,
        };
        let best_extra = sites
            .iter()
            .filter_map(|&pos| match item {
                Production::District(d, _) => Some(game.adjacency(*d, pos).iter().sum::<f32>()),
                _ => None,
            })
            .fold(0.0f32, f32::max);
        for pos in sites {
            for c in screens(s, rect, pos) {
                let extra = if let Production::District(d, _) = item {
                    game.adjacency(*d, pos).iter().sum::<f32>()
                } else {
                    0.0
                };
                let is_best = extra > 0.0 && (extra - best_extra).abs() < 0.01;
                overlay::mark(
                    &mut marks,
                    c,
                    z,
                    Color32::from_rgba_unmultiplied(70, 230, 120, alpha(72.0, 38.0)),
                    if is_best { palette::GOLD } else { Color32::from_rgb(112, 244, 144) },
                    if is_best { 2.6 } else { 1.9 },
                );
                if extra > 0.0 {
                    badges.push((c, extra, accent, is_best));
                }
            }
        }
    }
    if s.city_targeting
        && let Some(city_id) = s.selected_city
    {
        render::yields::draw_bombard_targets(painter, game, city_id, |p| screens(s, rect, p), z, now);
    }
    // The selected unit's or city's own tile wears a gold frame.
    let selected = s
        .selected_unit
        .and_then(|id| game.units.get(&id))
        .map(|u| u.pos)
        .or_else(|| s.selected_city.and_then(|id| game.cities.get(&id)).map(|c| c.pos));
    if let Some(pos) = selected {
        for c in screens(s, rect, pos) {
            overlay::mark(
                &mut marks,
                c,
                z,
                with_alpha(palette::GOLD, alpha(60.0, 50.0)),
                with_alpha(palette::GOLD, 240),
                2.0,
            );
        }
    }
    painter.add(Shape::mesh(marks));
    for (c, extra, accent, is_best) in badges {
        let r = (z * 0.31).max(10.0);
        if is_best {
            painter.circle_stroke(
                c,
                r + 3.0 + 1.5 * pulse,
                Stroke::new(2.0, with_alpha(palette::GOLD, (130.0 + 90.0 * pulse) as u8)),
            );
        }
        painter.circle_filled(c, r + 1.5, Color32::from_black_alpha(135));
        painter.circle_filled(c, r, Color32::from_rgb(14, 24, 38));
        painter.circle_stroke(c, r, Stroke::new(2.0, if is_best { palette::GOLD } else { accent }));
        painter.text(
            c,
            Align2::CENTER_CENTER,
            format!("+{extra:.0}"),
            palette::bold(r * 1.15),
            accent.lerp_to_gamma(Color32::WHITE, 0.35),
        );
    }
    if path.len() > 1 {
        let (dot, step) = ((z * 0.078).clamp(2.2, 4.6), (z * 0.32).max(10.5));
        if path_attack {
            // Crimson ranged/melee attack line + target reticle.
            let (a, b) = (path[0], path[1]);
            painter.line_segment([a, b], Stroke::new(dot * 2.2 + 3.0, Color32::from_black_alpha(90)));
            for (p, _) in overlay::along(&path, step, now * z * 0.9) {
                painter.circle_filled(p, dot + 1.2, Color32::from_black_alpha(130));
                painter.circle_filled(p, dot, Color32::from_rgb(255, 110, 96));
            }
            let r = z * (0.28 + 0.04 * pulse);
            painter.circle_stroke(b, r, Stroke::new(4.5, Color32::from_black_alpha(120)));
            painter.circle_stroke(b, r, Stroke::new(2.6, palette::BAD));
            crate::ui::glyph::paint(painter, crate::ui::glyph::Glyph::Swords, b, r * 0.62, palette::BAD);
        } else {
            // Split path into this-turn (cyan-white) and future-turn (amber-gold)
            // segments, and stamp numbered turn-end medallions (1, 2, 3...).
            let mut turn_ends: Vec<(Pos2, i32)> = Vec::new();
            for (i, seg) in path.windows(2).enumerate() {
                let tnum = path_turns.get(i).copied().unwrap_or(1);
                let next_t = path_turns.get(i + 1).copied();
                let seg_col = if tnum <= 1 {
                    Color32::from_rgb(180, 240, 255)
                } else {
                    palette::GOLD
                };
                painter.line_segment(
                    [seg[0], seg[1]],
                    Stroke::new(dot * 2.0 + 3.0, Color32::from_black_alpha(68)),
                );
                for (p, _) in overlay::along(seg, step, now * z * 0.72) {
                    painter.circle_filled(p, dot + 1.2, Color32::from_black_alpha(125));
                    painter.circle_filled(p, dot, seg_col);
                }
                if next_t != Some(tnum) {
                    turn_ends.push((seg[1], tnum));
                }
            }
            for (idx, &(pt, tnum)) in turn_ends.iter().enumerate() {
                let is_last = idx + 1 == turn_ends.len();
                let r = (z * if is_last { 0.25 } else { 0.21 }).clamp(8.5, 14.5);
                let rim = if tnum <= 1 { Color32::from_rgb(130, 232, 255) } else { palette::GOLD };
                if is_last {
                    painter.circle_stroke(
                        pt,
                        r + 2.5 + 1.5 * pulse,
                        Stroke::new(2.0, with_alpha(rim, (120.0 + 80.0 * pulse) as u8)),
                    );
                }
                painter.circle_filled(pt + vec2(0.0, 1.5), r + 1.2, Color32::from_black_alpha(130));
                painter.circle_filled(pt, r, Color32::from_rgb(14, 24, 40));
                painter.circle_stroke(pt, r, Stroke::new(2.0, rim));
                painter.text(
                    pt,
                    Align2::CENTER_CENTER,
                    tnum.to_string(),
                    palette::bold(r * 1.15),
                    Color32::WHITE,
                );
            }
        }
    }
    if let Some(city_id) = s.selected_city {
        render::yields::draw_worked_citizens(painter, game, city_id, |p| screens(s, rect, p), z);
    }

    // 6. Cities, units, then city banners on top.
    for city in game.cities.values() {
        if !player.revealed[grid.index(city.pos)] {
            continue;
        }
        let k = anim.pop(city.pos).map_or(1.0, ease_out_back);
        if k <= 0.04 {
            continue;
        }
        for c in screens(s, rect, city.pos) {
            // Soft architectural ground shadow & subtle warm golden hearth aura.
            if z >= 14.0 {
                let mut aura = Mesh::default();
                mesh::radial(
                    &mut aura,
                    c + vec2(0.06 * z, 0.14 * z),
                    vec2(0.85 * z * k, 0.56 * z * k),
                    Color32::from_black_alpha(66),
                );
                if city.capital {
                    mesh::radial(
                        &mut aura,
                        c,
                        vec2(0.95 * z * k, 0.72 * z * k),
                        Color32::from_rgba_unmultiplied(255, 224, 130, 32),
                    );
                }
                painter.add(Shape::mesh(aura));
            }
            if use_3d {
                render::view3d::draw_city_3d(&art.pack3d, painter, game, city, c, z * k);
            } else {
                render::buildings::draw_city(&mut art.buildings, painter, game, city, c, z * k);
            }
            // Subtle rising hearth/chimney smoke wisps above thriving cities (pop >= 3).
            if ambient && z >= 26.0 && city.population >= 3 {
                for wisp in 0..3 {
                    let phase = (now * 0.28 + wisp as f32 * 0.333 + city.pos.x as f32 * 0.17).fract();
                    let fade = (phase * 4.0).min(1.0) * (1.0 - phase);
                    if fade > 0.03 {
                        let drift = vec2(
                            (0.08 + phase * 0.26 + (now * 0.9 + wisp as f32).sin() * 0.03) * z,
                            (-0.18 - phase * 0.56) * z,
                        );
                        let rad = (0.06 + phase * 0.12) * z;
                        painter.circle_filled(
                            c + drift,
                            rad,
                            Color32::from_rgba_unmultiplied(238, 232, 218, (fade * 52.0) as u8),
                        );
                    }
                }
            }
        }
    }
    let mut by_tile: HashMap<Pos, Vec<&Unit>> = HashMap::new();
    for u in game.units.values() {
        by_tile.entry(u.pos).or_default().push(u);
    }
    let idle = UnitFx {
        time: if ambient { now } else { 0.0 },
        ..UnitFx::REST
    };
    let mut any_unit = false;
    for &(pos, c) in &tiles {
        if !player.visible[grid.index(pos)] {
            continue;
        }
        let Some(units) = by_tile.get(&pos) else {
            continue;
        };
        any_unit = true;
        for (u, slot) in units.iter().zip(slots(game, units)) {
            let selected = s.selected_unit == Some(u.id);
            if slot.air {
                // Aircraft: small diamonds across the top of their base.
                render::units::draw_aircraft(
                    &mut art.units,
                    icons,
                    painter,
                    game,
                    u,
                    c + slot.off * z,
                    z * slot.radius,
                    selected,
                );
            } else if anim.pose(u.id).is_none() {
                let unit_idle = UnitFx {
                    flip_x: s.unit_facing_left.get(&u.id).copied().unwrap_or(false),
                    ..idle
                };
                // (Units with a walk, lunge or fade playing are drawn below.)
                if !use_3d
                    || !render::view3d::draw_unit_3d(
                        &art.pack3d,
                        icons,
                        painter,
                        game,
                        u,
                        c + slot.off * z,
                        z * slot.radius,
                        selected,
                        &unit_idle,
                    )
                {
                    render::units::draw_unit(
                        &mut art.units,
                        icons,
                        painter,
                        game,
                        u,
                        c + slot.off * z,
                        z * slot.radius,
                        selected,
                        &unit_idle,
                    );
                }
            }
        }
    }
    // 6a. Units in motion, front to back: walks, lunges, hits, deaths.
    let mut moving: Vec<(f32, Cow<Unit>, Pos2, f32, UnitFx, bool)> = Vec::new();
    for (id, pose) in anim.animated() {
        let Some(unit) = game.units.get(&id).or_else(|| anim.ghosts.get(&id)) else {
            continue;
        };
        if game.is_air(unit) {
            continue;
        }
        let unit = match pose.hp {
            Some(hp) => {
                let mut u = unit.clone();
                u.hp = hp;
                Cow::Owned(u)
            }
            None => Cow::Borrowed(unit),
        };
        let slot = walking_slot(game, &by_tile, &unit, pose.tile);
        let naval = game.unit_type(&unit).domain == "DOMAIN_SEA";
        let (mut off, mut radius) = (slot.off, slot.radius);
        let (mut lift, mut tilt) = (pose.lift, 0.0);
        let mut flip_x = s.unit_facing_left.get(&id).copied().unwrap_or(false);
        if let Some(walk) = pose.walk {
            // Glide from the unit's place on the start tile to its place on
            // the end tile, so it never jumps when its slot changes.
            let start = walking_slot(game, &by_tile, &unit, walk.from);
            off = start.off + (off - start.off) * walk.blend;
            radius = start.radius + (radius - start.radius) * walk.blend;
            if walk.heading.x.abs() > 0.12 {
                flip_x = walk.heading.x < 0.0;
                facing_updates.push((id, flip_x));
            }
            let step = walk.stride * std::f32::consts::TAU;
            if naval {
                // Ships roll gently instead of stepping.
                tilt = 0.03 * (step * 0.5).sin() * walk.pace;
            } else {
                // Two steps a tile: a small bob, a lean into the walk, and
                // a sway from foot to foot.
                lift += 0.025 * (step).sin().abs() * walk.pace;
                tilt = (0.07 * walk.heading.x + 0.045 * step.sin()) * walk.pace;
            }
        }
        let fx = UnitFx {
            time: if ambient { now } else { 0.0 },
            alpha: pose.alpha,
            flash: pose.flash,
            scale: pose.scale,
            lift: lift * z,
            tilt,
            flip_x,
        };
        for c in screens(s, rect, pose.tile) {
            let at = c + (off + pose.offset) * z;
            moving.push((
                at.y,
                unit.clone(),
                at,
                z * radius,
                fx,
                s.selected_unit == Some(id),
            ));
        }
    }
    moving.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, unit, at, radius, fx, selected) in &moving {
        any_unit = true;
        if !use_3d
            || !render::view3d::draw_unit_3d(
                &art.pack3d,
                icons,
                painter,
                game,
                unit,
                *at,
                *radius,
                *selected,
                fx,
            )
        {
            render::units::draw_unit(
                &mut art.units,
                icons,
                painter,
                game,
                unit,
                *at,
                *radius,
                *selected,
                fx,
            );
        }
    }
    let pointer_pos = response.hover_pos();
    for city in game.cities.values() {
        if !player.revealed[grid.index(city.pos)] {
            continue;
        }
        let pop = anim.pop(city.pos);
        // A new city's banner drops in once the city has risen.
        let (offset, faded) = match pop {
            Some(p) if p < 0.45 => continue,
            Some(p) => {
                let k = (p - 0.45) / 0.55;
                (vec2(0.0, -(1.0 - ease_out(k)) * z * 0.7), smooth(k))
            }
            None => (Vec2::ZERO, 1.0),
        };
        let mut banner_painter = painter.clone();
        banner_painter.set_opacity(faded);
        for c in screens(s, rect, city.pos) {
            let hovered_prev = pointer_pos.is_some_and(|p| {
                s.city_banners
                    .iter()
                    .any(|(id, r)| *id == city.id && r.contains(p))
            });
            let hit = render::banner::draw_city_banner(
                &banner_painter,
                game,
                city,
                c + offset,
                z,
                city.owner == me,
                hovered_prev && city.owner == me,
                s.selected_city == Some(city.id),
            );
            banners_out.push((city.id, hit));
        }
    }
    // 6b. Your trade routes: dashed gold lines between the cities.
    for route in game.trade_routes_of(me) {
        let (Some(a), Some(b)) = (
            game.cities.get(&route.origin),
            game.cities.get(&route.destination),
        ) else {
            continue;
        };
        let Some(&from) = screens(s, rect, a.pos).first() else {
            continue;
        };
        if let Some(to) = screens(s, rect, b.pos)
            .into_iter()
            .min_by_key(|p| (p.distance(from) * 10.0) as i32)
        {
            painter.add(Shape::line_segment([from, to], Stroke::new(4.5, Color32::from_black_alpha(70))));
            painter.add(Shape::dashed_line(
                &[from, to],
                Stroke::new(2.2, palette::GOLD),
                8.0,
                6.0,
            ));
        }
    }
    // 6c. Effects: shots in flight, impacts, damage numbers, dust, sparkles.
    for fx in &anim.fx {
        if let Some(u) = fx.progress(anim.now) {
            for c in screens(s, rect, fx.at) {
                render::fx::draw(painter, fx, u, c, z);
            }
        }
    }

    // 7. Hover outline (structured tooltip is rendered in `show` after art/icons return).
    if let Some(pos) = s.hover {
        let mut marks = Mesh::default();
        for c in screens(s, rect, pos) {
            overlay::mark(
                &mut marks,
                c,
                z,
                Color32::from_rgba_unmultiplied(255, 244, 214, alpha(34.0, 18.0)),
                Color32::from_rgba_unmultiplied(255, 250, 236, 240),
                2.0,
            );
        }
        painter.add(Shape::mesh(marks));
    }
    ambient && (any_unit || art.terrain.animated())
}

/// The smallest zoom at which the map is still at least as wide as the view.
fn min_zoom(s: &Session, rect: Rect) -> f32 {
    let width = SQRT3 * s.game.map.grid.width as f32;
    (rect.width() / width).max(8.0)
}

fn handle_input(s: &mut Session, ui: &egui::Ui, rect: Rect, response: &egui::Response) {
    // Pan: left/middle drag or arrow keys. Zoom: smooth wheel toward the pointer.
    if response.dragged() || response.dragged_by(egui::PointerButton::Middle) {
        s.cam.center -= response.drag_delta() / s.cam.zoom;
        s.anim.cancel_pan();
    }
    let (scroll, keys, dt) = ui.input(|i| {
        let mut keys = Vec2::ZERO;
        for (key, d) in [
            (egui::Key::ArrowLeft, vec2(-1.0, 0.0)),
            (egui::Key::ArrowRight, vec2(1.0, 0.0)),
            (egui::Key::ArrowUp, vec2(0.0, -1.0)),
            (egui::Key::ArrowDown, vec2(0.0, 1.0)),
        ] {
            if i.key_down(key) {
                keys += d;
            }
        }
        (
            if response.hovered() {
                i.smooth_scroll_delta.y
            } else {
                0.0
            },
            keys,
            i.unstable_dt.clamp(0.004, 0.05),
        )
    });
    if !ui.ctx().egui_wants_keyboard_input() && keys != Vec2::ZERO {
        s.cam.center += keys * 0.5;
        s.anim.cancel_pan();
        ui.ctx().request_repaint();
    }
    let min_z = min_zoom(s, rect);
    if scroll != 0.0 {
        s.anim.cancel_pan();
        s.cam.target_zoom = (s.cam.target_zoom * (1.0 + scroll * 0.0018)).clamp(min_z, 70.0);
        s.cam.zoom_anchor = Some(response.hover_pos().unwrap_or(rect.center()));
    }
    s.cam.target_zoom = s.cam.target_zoom.clamp(min_z, 70.0);
    if (s.cam.target_zoom - s.cam.zoom).abs() > 0.05 {
        let anchor = s.cam.zoom_anchor.unwrap_or(rect.center());
        let before = (anchor - rect.center()) / s.cam.zoom + s.cam.center;
        let alpha = 1.0 - (-16.0 * dt).exp();
        s.cam.zoom += (s.cam.target_zoom - s.cam.zoom) * alpha;
        let after = (anchor - rect.center()) / s.cam.zoom + s.cam.center;
        s.cam.center += before - after;
        ui.ctx().request_repaint();
    } else {
        s.cam.zoom = s.cam.target_zoom;
        s.cam.zoom_anchor = None;
    }
    // Never zoom out so far that the wrapping map shows itself twice.
    s.cam.zoom = s.cam.zoom.max(min_z);
    let height = 1.5 * s.game.map.grid.height as f32;
    s.cam.center.y = s.cam.center.y.clamp(0.0, height);
    let width = SQRT3 * s.game.map.grid.width as f32;
    s.cam.center.x = s.cam.center.x.rem_euclid(width);

    let hover = response.hover_pos().and_then(|p| tile_at(s, rect, p));
    if hover != s.hover {
        s.hover = hover;
        s.update_path_preview();
    }
    if response.clicked() {
        // Clicking directly on an own city banner opens that city (unless placing/targeting).
        let banner_hit = if s.placing.is_none() && !s.city_targeting {
            response.interact_pointer_pos().and_then(|p| {
                s.city_banners
                    .iter()
                    .rev()
                    .find(|(_, r)| r.contains(p))
                    .map(|&(id, _)| id)
            })
        } else {
            None
        };
        if let Some(cid) = banner_hit
            && s.game.cities.get(&cid).is_some_and(|c| c.owner == s.me())
        {
            s.select_city(Some(cid));
        } else if let Some(pos) = hover {
            s.left_click(pos);
        }
    }
    if response.secondary_clicked()
        && let Some(pos) = hover
    {
        s.right_click(pos);
    }
}

/// Land tiles near `center` with nobody on them, nearest first.
fn free_land(game: &Game, center: Pos, from: i32, to: i32) -> Vec<Pos> {
    let grid = game.map.grid;
    let mut tiles: Vec<Pos> = grid
        .within(center, to)
        .into_iter()
        .filter(|&p| {
            let d = grid.distance(center, p);
            d >= from
                && game.map.is_land(&game.rules, p)
                && !game.map.is_impassable(&game.rules, p)
                && game.units_at(p).next().is_none()
                && game.city_at(p).is_none()
        })
        .collect();
    tiles.sort_by_key(|&p| grid.distance(center, p));
    tiles
}

/// Test hook (`WAYCIV_ANIM_DEMO=name`): stages a scene with the engine's own
/// calls, so the animations can be screenshotted. Scenes: `walk`, `fight`
/// (melee), `shoot` (archer), `siege` (catapult), `found` (the Settler
/// founds a city), `build` (a Builder improves a tile), `pan` (camera moves
/// to the nearest foreign city), `turn` (End Turn), and `all` (walk, shoot,
/// siege and found at once).
pub fn demo(s: &mut Session, name: &str) {
    let me = s.game.human();
    let Some(anchor) = s.game.player_units(me).next().map(|u| u.pos) else {
        return;
    };
    let war = s.game.barbarian();
    if let Some(bar) = war {
        s.game.players[me].relations[bar].at_war = true;
        s.game.players[bar].relations[me].at_war = true;
    }
    let warrior = s
        .game
        .player_units(me)
        .find(|u| s.game.is_military(u))
        .map(|u| u.id);
    let settler = s
        .game
        .player_units(me)
        .find(|u| s.game.unit_type(u).found_city)
        .map(|u| u.id);
    let free = free_land(&s.game, anchor, 1, 4);
    let spawn = |s: &mut Session, owner, id: &str, pos| s.game.spawn(owner, id, pos);
    let all = name == "all";
    snap_to(s, anchor);
    if (all || name == "walk")
        && let Some(w) = warrior
    {
        s.select_unit(Some(w));
        let from = s.game.units[&w].pos;
        let goal = s
            .game
            .reachable(&s.game.units[&w])
            .into_iter()
            .max_by_key(|&p| s.game.map.grid.distance(from, p));
        if let Some(goal) = goal {
            s.right_click(goal);
        }
    }
    if name == "fight"
        && let (Some(w), Some(bar)) = (warrior, war)
    {
        let from = s.game.units[&w].pos;
        if let Some(&at) = free_land(&s.game, from, 1, 1).first() {
            spawn(s, bar, "UNIT_WARRIOR", at);
            let r = s.game.attack(w, at);
            s.result(r);
        }
    }
    if let (true, Some(bar)) = (all || name == "shoot" || name == "siege", war) {
        let kind = if name == "siege" || all { "UNIT_CATAPULT" } else { "UNIT_ARCHER" };
        let spots: Vec<Pos> = free.iter().copied().filter(|p| s.game.units_at(*p).next().is_none()).collect();
        if let Some(&shooter_at) = spots.first()
            && let Some(&target_at) = free_land(&s.game, shooter_at, 2, 2)
                .iter()
                .find(|p| !spots.first().is_some_and(|f| f == *p) && s.game.units_at(**p).next().is_none())
            && let Some(shooter) = spawn(s, me, kind, shooter_at)
        {
            spawn(s, bar, "UNIT_WARRIOR", target_at);
            let r = s.game.attack(shooter, target_at);
            s.result(r);
        }
    }
    if (all || name == "found")
        && let Some(st) = settler
    {
        let r = s.game.found_city(st);
        s.result(r);
    }
    if name == "build" {
        let cities: Vec<_> = s.game.player_cities(me).map(|c| c.pos).collect();
        for city in cities {
            for tile in s.game.map.grid.within(city, 2) {
                let Some(imp) = s.game.valid_improvements(me, tile).first().copied() else {
                    continue;
                };
                if let Some(b) = spawn(s, me, "UNIT_BUILDER", tile) {
                    snap_to(s, tile);
                    let r = s.game.build_improvement(b, imp);
                    s.result(r);
                    return;
                }
            }
        }
    }
    if name == "pan" {
        let far = s
            .game
            .cities
            .values()
            .filter(|c| c.owner != me)
            .max_by_key(|c| s.game.map.grid.distance(anchor, c.pos))
            .map(|c| c.pos);
        if let Some(far) = far {
            center_on(s, far);
        }
    }
    if name == "turn" {
        s.game.end_turn();
        s.refresh_selection();
    }
}

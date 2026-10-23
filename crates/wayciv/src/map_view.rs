//! The hex map: rendering and mouse/keyboard input.

use crate::hud::Session;
use crate::palette::{self, rgb, with_alpha};
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2, pos2, vec2};
use wayciv_core::{Pos, Production};

const SQRT3: f32 = 1.732_050_8;

/// Axial neighbor offsets, matching `wayciv_core::hex`: E, NE, NW, W, SW, SE.
const DIRS: [(i32, i32); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];
/// Corner pairs of the edge facing each direction (corner i at 60*i-30 degrees).
const EDGES: [(usize, usize); 6] = [(0, 1), (5, 0), (4, 5), (3, 4), (2, 3), (1, 2)];

pub struct Camera {
    /// Center in world units (hex radius 1).
    pub center: Vec2,
    /// Hex radius in pixels.
    pub zoom: f32,
}

pub fn world(pos: Pos) -> Vec2 {
    vec2(SQRT3 * (pos.x as f32 + 0.5 * (pos.y & 1) as f32), 1.5 * pos.y as f32)
}

fn corners(center: Pos2, r: f32) -> [Pos2; 6] {
    std::array::from_fn(|i| {
        let angle = (60.0 * i as f32 - 30.0).to_radians();
        center + vec2(angle.cos(), angle.sin()) * r
    })
}

fn hex(center: Pos2, r: f32, fill: Color32, stroke: Stroke) -> Shape {
    Shape::convex_polygon(corners(center, r).to_vec(), fill, stroke)
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
        if p.x > rect.left() - zoom * 2.0 && p.x < rect.right() + zoom * 2.0 && p.y > rect.top() - zoom * 2.0 && p.y < rect.bottom() + zoom * 2.0 {
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
    s.game.map.grid.normalize(Pos::from_axial(rx as i32, rz as i32))
}

pub fn center_on(s: &mut Session, pos: Pos) {
    s.cam.center = world(pos);
}

fn neighbor(s: &Session, pos: Pos, dir: usize) -> Option<Pos> {
    let (q, r) = pos.axial();
    s.game.map.grid.normalize(Pos::from_axial(q + DIRS[dir].0, r + DIRS[dir].1))
}

fn abbreviation(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.len() >= 2 {
        words.iter().take(2).filter_map(|w| w.chars().next()).collect()
    } else {
        name.chars().take(2).collect()
    }
}

pub fn show(s: &mut Session, ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(rect, Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::from_rgb(8, 10, 14));

    handle_input(s, ui, rect, &response);
    let mut icons = std::mem::replace(&mut s.icons, crate::icons::Icons::new(Default::default()));
    draw(s, ui, rect, &response, &painter, &mut icons);
    s.icons = icons;
}

fn draw(s: &Session, ui: &egui::Ui, rect: Rect, response: &egui::Response, painter: &egui::Painter, icons: &mut crate::icons::Icons) {
    let _ = ui;
    let game = &s.game;
    let rules = &game.rules;
    let content = &rules.content;
    let me = game.human();
    let player = &game.players[me];
    let grid = game.map.grid;
    let z = s.cam.zoom;
    let detail = z >= 22.0;
    let outline = Stroke::new(if z > 14.0 { 1.0 } else { 0.5 }, Color32::from_rgba_unmultiplied(0, 0, 0, 90));

    // 1. Terrain, features, relief.
    let mut tiles: Vec<(Pos, Pos2)> = Vec::new();
    for pos in grid.all() {
        if !player.revealed[grid.index(pos)] {
            continue;
        }
        for c in screens(s, rect, pos) {
            tiles.push((pos, c));
        }
    }
    for &(pos, c) in &tiles {
        let tile = game.map.tile(pos);
        let terrain = &content.terrains[tile.terrain];
        let mut color = palette::biome(&terrain.id);
        if terrain.hills {
            color = color.gamma_multiply(1.08);
        }
        painter.add(hex(c, z, color, outline));
        if terrain.hills && !terrain.mountain {
            // Two rounded humps, darker than the biome color.
            for (dx, dy, w) in [(-0.3, 0.32, 0.3), (0.28, 0.38, 0.24)] {
                let base = c + vec2(dx * z, dy * z);
                let hump: Vec<Pos2> = (0..=8)
                    .map(|i| {
                        let a = std::f32::consts::PI * (1.0 + i as f32 / 8.0);
                        base + vec2(a.cos() * w * z, a.sin() * w * z * 0.8)
                    })
                    .collect();
                painter.add(Shape::convex_polygon(hump, color.gamma_multiply(0.72), Stroke::new(1.0, color.gamma_multiply(0.5))));
            }
        }
        if terrain.mountain {
            let peak = vec![c + vec2(-0.6 * z, 0.45 * z), c + vec2(0.0, -0.55 * z), c + vec2(0.6 * z, 0.45 * z)];
            painter.add(Shape::convex_polygon(peak, Color32::from_rgb(110, 105, 100), Stroke::new(1.0, Color32::from_rgb(60, 55, 50))));
            let cap = vec![c + vec2(-0.2 * z, -0.18 * z), c + vec2(0.0, -0.55 * z), c + vec2(0.2 * z, -0.18 * z)];
            painter.add(Shape::convex_polygon(cap, Color32::from_rgb(245, 245, 250), Stroke::NONE));
        }
        if let Some(f) = tile.feature {
            draw_feature(painter, c, z, &content.features[f].id, content.features[f].natural_wonder, detail.then_some(content.features[f].name.as_str()));
        }
        if tile.cliffs {
            // Cliffs: rocky edges toward the water.
            let pts = corners(c, z * 0.96);
            for dir in 0..6 {
                if neighbor(s, pos, dir).is_some_and(|n| game.map.is_water(rules, n)) {
                    let (a, b) = EDGES[dir];
                    painter.line_segment([pts[a], pts[b]], Stroke::new((z * 0.16).max(2.0), Color32::from_rgb(92, 78, 66)));
                    let inner = corners(c, z * 0.84);
                    painter.line_segment([inner[a], inner[b]], Stroke::new((z * 0.05).max(1.0), Color32::from_rgb(150, 135, 115)));
                }
            }
        }
    }

    // 2. Rivers.
    let river = Stroke::new((z * 0.12).max(1.5), Color32::from_rgb(60, 130, 220));
    for &(pos, c) in &tiles {
        let tile = game.map.tile(pos);
        if !tile.river {
            continue;
        }
        // Known flow (Earth map): one segment to the downstream neighbor, so
        // rivers draw as clean lines; into the sea they stop at the shore.
        if let Some(dir) = tile.river_dir {
            let dir = dir as usize;
            let reach = if neighbor(s, pos, dir).is_some_and(|n| game.map.is_water(rules, n)) { 0.5 } else { 1.0 };
            painter.line_segment([c, c + world_dir(dir) * z * reach], river);
            continue;
        }
        let mut drew = false;
        for dir in 0..6 {
            if let Some(n) = neighbor(s, pos, dir)
                && (game.map.tile(n).river || game.map.is_water(rules, n)) {
                    let off = world_dir(dir) * z * 0.5;
                    painter.line_segment([c, c + off], river);
                    drew = true;
                }
        }
        if !drew {
            painter.circle_filled(c, z * 0.1, Color32::from_rgb(60, 130, 220));
        }
    }

    // 2b. Roads: brown lines toward neighboring roads and cities.
    for &(pos, c) in &tiles {
        let road = |p: Pos| game.map.tile(p).road || game.city_at(p).is_some();
        if !game.map.tile(pos).road {
            continue;
        }
        for dir in 0..6 {
            if neighbor(s, pos, dir).is_some_and(road) {
                let off = world_dir(dir) * z * 0.5;
                painter.line_segment([c, c + off], Stroke::new((z * 0.1).max(1.5), Color32::from_rgb(150, 105, 55)));
            }
        }
    }

    // 3. Territory: tint and borders.
    for &(pos, c) in &tiles {
        let tile = game.map.tile(pos);
        let Some(owner) = tile.owner else { continue };
        let o = &game.players[owner];
        let primary = rgb(o.primary);
        painter.add(hex(c, z, with_alpha(primary, 46), Stroke::NONE));
        let pts = corners(c, z * 0.93);
        for dir in 0..6 {
            let other = neighbor(s, pos, dir).and_then(|n| game.map.tile(n).owner);
            if other != Some(owner) {
                let (a, b) = EDGES[dir];
                painter.line_segment([pts[a], pts[b]], Stroke::new((z * 0.13).max(2.0), primary));
                let inner = corners(c, z * 0.82);
                painter.line_segment([inner[a], inner[b]], Stroke::new((z * 0.05).max(1.0), rgb(o.secondary)));
            }
        }
    }

    // 4. Districts, wonders, improvements, resources.
    for &(pos, c) in &tiles {
        let tile = game.map.tile(pos);
        if let Some(d) = tile.district {
            let district = &content.districts[d];
            if !district.city_center {
                let base = district.replaces.as_deref().unwrap_or(&district.id);
                let (color, label) = palette::district(base);
                painter.add(hex(c, z * 0.62, color, Stroke::new(1.5, Color32::from_gray(30))));
                if z > 12.0 {
                    painter.text(c, Align2::CENTER_CENTER, label, FontId::proportional(z * 0.42), Color32::from_gray(20));
                }
            }
        }
        if tile.wonder.is_some() {
            painter.add(hex(c, z * 0.62, palette::GOLD, Stroke::new(1.5, Color32::from_rgb(90, 60, 10))));
            if z > 12.0 {
                painter.text(c, Align2::CENTER_CENTER, "W", FontId::proportional(z * 0.5), Color32::from_rgb(60, 40, 0));
            }
        }
        if game.camps.contains_key(&pos) {
            // Barbarian camp: a red tent.
            let tent = vec![c + vec2(0.0, -0.45 * z), c + vec2(0.45 * z, 0.35 * z), c + vec2(-0.45 * z, 0.35 * z)];
            painter.add(Shape::convex_polygon(tent, Color32::from_rgb(190, 45, 35), Stroke::new(2.0, Color32::BLACK)));
            painter.line_segment([c + vec2(0.0, -0.1 * z), c + vec2(0.0, 0.35 * z)], Stroke::new(2.0, Color32::BLACK));
        } else if let Some(i) = tile.improvement
            && detail {
                let name = &content.improvements[i].name;
                let label = abbreviation(name);
                let at = c + vec2(0.0, 0.55 * z);
                painter.rect_filled(Rect::from_center_size(at, vec2(z * 0.7, z * 0.32)), 3.0, Color32::from_rgba_unmultiplied(70, 45, 20, 220));
                painter.text(at, Align2::CENTER_CENTER, label, FontId::proportional(z * 0.24), Color32::from_rgb(240, 220, 180));
            }
        if let Some(r) = tile.resource {
            let res = &content.resources[r];
            if player.meets(rules, res.prereq_tech.as_deref(), None) && z > 10.0 {
                let color = match res.class.as_str() {
                    "RESOURCECLASS_LUXURY" => Color32::from_rgb(190, 90, 220),
                    "RESOURCECLASS_STRATEGIC" => Color32::from_rgb(220, 70, 60),
                    _ => Color32::from_rgb(80, 190, 80),
                };
                let at = c + vec2(0.42 * z, -0.42 * z);
                let radius = if z > 16.0 { z * 0.27 } else { z * 0.2 };
                let drew = z > 16.0 && icons.paint(painter, "resource", &res.id, Rect::from_center_size(at, vec2(radius * 2.0, radius * 2.0)));
                if drew {
                    painter.circle_stroke(at, radius, Stroke::new((z * 0.06).max(1.5), color));
                } else {
                    painter.circle(at, z * 0.2, color, Stroke::new(1.0, Color32::BLACK));
                    if detail {
                        painter.text(at, Align2::CENTER_CENTER, abbreviation(&res.name), FontId::proportional(z * 0.2), Color32::WHITE);
                    }
                }
            }
        }
        if !player.visible[grid.index(pos)] {
            painter.add(hex(c, z, Color32::from_rgba_unmultiplied(0, 0, 0, 120), Stroke::NONE));
        }
    }

    // 5. Highlights: reachable tiles, attack targets, district sites, path.
    if let Some(unit) = s.selected_unit.and_then(|id| game.units.get(&id)) {
        for &pos in &s.reachable {
            for c in screens(s, rect, pos) {
                painter.add(hex(c, z * 0.9, Color32::from_rgba_unmultiplied(120, 200, 255, 40), Stroke::new(1.0, Color32::from_rgba_unmultiplied(150, 220, 255, 120))));
            }
        }
        for pos in grid.within(unit.pos, game.unit_type(unit).range.max(1)) {
            if game.can_attack(unit, pos) {
                for c in screens(s, rect, pos) {
                    painter.add(hex(c, z * 0.9, Color32::from_rgba_unmultiplied(230, 50, 40, 60), Stroke::new(2.0, Color32::from_rgb(240, 60, 50))));
                }
            }
        }
        let mut prev = screens(s, rect, unit.pos).first().copied();
        for &step in &s.path_preview {
            let here = screens(s, rect, step).into_iter().min_by_key(|p| prev.map(|q| (p.distance(q) * 10.0) as i32).unwrap_or(0));
            if let (Some(a), Some(b)) = (prev, here) {
                painter.line_segment([a, b], Stroke::new(3.0, Color32::from_rgb(255, 255, 255)));
            }
            prev = here;
        }
    }
    if let Some(item) = &s.placing
        && let Some(city) = s.selected_city {
            let sites = match item {
                Production::District(d, _) => game.district_sites(city, *d),
                Production::Wonder(w, _) => game.wonder_sites(city, *w),
                _ => Vec::new(),
            };
            for pos in sites {
                for c in screens(s, rect, pos) {
                    let extra = if let Production::District(d, _) = item { game.adjacency(*d, pos).iter().sum::<f32>() } else { 0.0 };
                    painter.add(hex(c, z * 0.9, Color32::from_rgba_unmultiplied(80, 230, 120, 60), Stroke::new(2.0, Color32::from_rgb(90, 240, 130))));
                    if extra > 0.0 {
                        painter.text(c, Align2::CENTER_CENTER, format!("+{extra:.0}"), FontId::proportional(z * 0.45), Color32::WHITE);
                    }
                }
            }
        }
    if let Some(city) = s.selected_city.and_then(|id| game.cities.get(&id)) {
        for &pos in &city.worked {
            for c in screens(s, rect, pos) {
                painter.circle_stroke(c, z * 0.25, Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 160)));
            }
        }
    }

    // 6. Units, then city banners on top.
    for pos in grid.all() {
        if !player.visible[grid.index(pos)] {
            continue;
        }
        let units: Vec<_> = game.units_at(pos).collect();
        if units.is_empty() {
            continue;
        }
        for c in screens(s, rect, pos) {
            let mut civilian_slot = 0;
            let mut air_slot = 0;
            for u in &units {
                if game.is_air(u) {
                    // Aircraft: small diamonds across the top of their base.
                    let at = c + vec2(-0.45 * z + 0.3 * z * air_slot as f32, -0.62 * z);
                    air_slot += 1;
                    draw_aircraft(painter, game, u, at, z * 0.22, s.selected_unit == Some(u.id));
                    continue;
                }
                let military = game.is_military(u);
                let (at, radius) = if military { (c + vec2(0.0, -0.05 * z), z * 0.42) } else {
                    civilian_slot += 1;
                    (c + vec2(-0.45 * z + 0.25 * z * ((civilian_slot - 1) % 4) as f32, 0.4 * z), z * 0.3)
                };
                draw_unit(painter, icons, game, u, at, radius, s.selected_unit == Some(u.id));
            }
        }
    }

    for city in game.cities.values() {
        if !player.revealed[grid.index(city.pos)] {
            continue;
        }
        let owner = &game.players[city.owner];
        for c in screens(s, rect, city.pos) {
            painter.add(hex(c, z * 0.7, rgb(owner.primary), Stroke::new(2.0, rgb(owner.secondary))));
            painter.circle_filled(c, z * 0.18, rgb(owner.secondary));
            draw_city_banner(painter, game, city, c, z, city.owner == me);
        }
    }
    // 6b. Your trade routes: dashed gold lines between the cities.
    for route in game.trade_routes_of(me) {
        let (Some(a), Some(b)) = (game.cities.get(&route.origin), game.cities.get(&route.destination)) else { continue };
        let Some(&from) = screens(s, rect, a.pos).first() else { continue };
        if let Some(to) = screens(s, rect, b.pos).into_iter().min_by_key(|p| (p.distance(from) * 10.0) as i32) {
            painter.add(Shape::dashed_line(&[from, to], Stroke::new(2.0, palette::GOLD), 8.0, 6.0));
        }
    }

    // 7. Hover outline and tooltip.
    if let Some(pos) = s.hover {
        for c in screens(s, rect, pos) {
            painter.add(hex(c, z, Color32::TRANSPARENT, Stroke::new(2.0, Color32::WHITE)));
        }
        if player.revealed[grid.index(pos)] && response.hovered() {
            let text = crate::hud::tile_tooltip(s, pos);
            response.clone().on_hover_ui_at_pointer(|ui| {
                ui.set_max_width(320.0);
                ui.label(text);
            });
        }
    }
}

fn world_dir(dir: usize) -> Vec2 {
    let (q, r) = DIRS[dir];
    vec2(SQRT3 * (q as f32 + r as f32 / 2.0), 1.5 * r as f32)
}

pub fn draw_feature(painter: &egui::Painter, c: Pos2, z: f32, id: &str, natural_wonder: bool, label: Option<&str>) {
    if natural_wonder {
        let star: Vec<Pos2> = (0..10)
            .map(|i| {
                let r = if i % 2 == 0 { 0.55 } else { 0.24 } * z;
                let a = (-90.0 + 36.0 * i as f32).to_radians();
                c + vec2(a.cos(), a.sin()) * r
            })
            .collect();
        painter.add(Shape::closed_line(star.clone(), Stroke::new(2.0, Color32::from_rgb(120, 80, 0))));
        painter.add(Shape::convex_polygon(vec![c + vec2(0.0, -0.5 * z), c + vec2(0.45 * z, 0.35 * z), c + vec2(-0.45 * z, 0.35 * z)], Color32::from_rgba_unmultiplied(255, 210, 60, 140), Stroke::NONE));
        if let Some(label) = label {
            painter.text(c + vec2(0.0, 0.7 * z), Align2::CENTER_TOP, label, FontId::proportional(z * 0.3), Color32::from_rgb(255, 230, 140));
        }
        return;
    }
    match id {
        "FEATURE_FOREST" => {
            for (dx, dy) in [(-0.35, 0.05), (0.0, -0.3), (0.35, 0.05), (0.0, 0.35)] {
                let base = c + vec2(dx * z, dy * z);
                let tree = vec![base + vec2(0.0, -0.22 * z), base + vec2(0.16 * z, 0.12 * z), base + vec2(-0.16 * z, 0.12 * z)];
                painter.add(Shape::convex_polygon(tree, Color32::from_rgb(30, 90, 35), Stroke::new(1.0, Color32::from_rgb(15, 50, 20))));
            }
        }
        "FEATURE_JUNGLE" => {
            for (dx, dy) in [(-0.35, -0.1), (0.05, -0.35), (0.35, 0.0), (-0.1, 0.3), (0.3, 0.35)] {
                painter.circle(c + vec2(dx * z, dy * z), z * 0.17, Color32::from_rgb(25, 110, 70), Stroke::new(1.0, Color32::from_rgb(10, 60, 40)));
            }
        }
        "FEATURE_MARSH" => {
            for dy in [-0.25, 0.05, 0.35] {
                painter.line_segment([c + vec2(-0.4 * z, dy * z), c + vec2(0.4 * z, dy * z)], Stroke::new((z * 0.07).max(1.0), Color32::from_rgb(60, 120, 170)));
            }
        }
        "FEATURE_FLOODPLAINS" => {
            // Desert floodplains: a green, fertile strip on the sand.
            painter.add(Shape::convex_polygon(vec![c + vec2(-0.6 * z, -0.12 * z), c + vec2(0.6 * z, -0.28 * z), c + vec2(0.6 * z, 0.08 * z), c + vec2(-0.6 * z, 0.24 * z)], Color32::from_rgb(120, 165, 70), Stroke::NONE));
            for dy in [-0.32, 0.36] {
                painter.line_segment([c + vec2(-0.45 * z, dy * z), c + vec2(0.45 * z, dy * z)], Stroke::new((z * 0.05).max(1.0), Color32::from_rgb(140, 120, 70)));
            }
        }
        "FEATURE_FLOODPLAINS_GRASSLAND" | "FEATURE_FLOODPLAINS_PLAINS" => {
            for dy in [-0.25, 0.0, 0.25] {
                painter.line_segment([c + vec2(-0.45 * z, dy * z), c + vec2(0.45 * z, (dy + 0.08) * z)], Stroke::new((z * 0.06).max(1.0), Color32::from_rgb(120, 105, 60)));
            }
        }
        "FEATURE_TAIGA" => {
            // Narrow, dark conifers.
            for (dx, dy) in [(-0.38, 0.1), (-0.12, -0.25), (0.14, 0.12), (0.38, -0.2), (0.0, 0.42)] {
                let base = c + vec2(dx * z, dy * z);
                let tree = vec![base + vec2(0.0, -0.28 * z), base + vec2(0.1 * z, 0.12 * z), base + vec2(-0.1 * z, 0.12 * z)];
                painter.add(Shape::convex_polygon(tree, Color32::from_rgb(28, 78, 66), Stroke::new(1.0, Color32::from_rgb(12, 40, 34))));
            }
        }
        "FEATURE_DUNES" => {
            // Wind-shaped sand ridges.
            for (dx, dy) in [(-0.2, -0.25), (0.15, 0.05), (-0.15, 0.35)] {
                let arc: Vec<Pos2> = (0..=8)
                    .map(|i| {
                        let t = i as f32 / 8.0;
                        c + vec2((dx - 0.3 + 0.6 * t) * z, (dy - 0.12 * (std::f32::consts::PI * t).sin()) * z)
                    })
                    .collect();
                painter.add(Shape::line(arc, Stroke::new((z * 0.07).max(1.0), Color32::from_rgb(190, 145, 70))));
            }
        }
        "FEATURE_GLACIER" => {
            painter.add(hex(c, z * 0.9, Color32::from_rgb(232, 242, 252), Stroke::new(1.0, Color32::from_rgb(180, 205, 235))));
            for (a, b) in [((-0.4, -0.1), (0.1, 0.05)), ((0.0, 0.3), (0.35, 0.15)), ((-0.1, -0.4), (0.2, -0.3))] {
                painter.line_segment([c + vec2(a.0 * z, a.1 * z), c + vec2(b.0 * z, b.1 * z)], Stroke::new((z * 0.04).max(1.0), Color32::from_rgb(140, 175, 215)));
            }
        }
        "FEATURE_OASIS" => {
            painter.circle_filled(c + vec2(0.0, 0.1 * z), z * 0.25, Color32::from_rgb(60, 150, 210));
            painter.circle_filled(c + vec2(0.18 * z, -0.15 * z), z * 0.12, Color32::from_rgb(40, 130, 50));
        }
        "FEATURE_ICE" => {
            painter.add(hex(c, z * 0.85, Color32::from_rgba_unmultiplied(235, 245, 255, 200), Stroke::new(1.0, Color32::from_rgb(170, 200, 230))));
        }
        _ => {}
    }
}

fn draw_city_banner(painter: &egui::Painter, game: &wayciv_core::Game, city: &wayciv_core::state::City, c: Pos2, z: f32, mine: bool) {
    let owner = &game.players[city.owner];
    let primary = rgb(owner.primary);
    let secondary = rgb(owner.secondary);
    let font = FontId::proportional((z * 0.42).clamp(10.0, 18.0));
    let name = if city.capital { format!("* {}", city.name) } else { city.name.clone() };
    let galley = painter.layout_no_wrap(name, font.clone(), secondary);
    let h = galley.size().y + 6.0;
    let pop_w = h;
    let w = galley.size().x + pop_w + 12.0;
    let top = c + vec2(-w / 2.0, -z * 1.05 - h);
    let banner = Rect::from_min_size(top, vec2(w, h));
    painter.rect_filled(banner, 4.0, primary);
    painter.rect_stroke(banner, 4.0, Stroke::new(1.5, secondary), StrokeKind::Inside);
    let pop_rect = Rect::from_min_size(top, vec2(pop_w, h));
    painter.rect_filled(pop_rect, 4.0, secondary);
    painter.text(pop_rect.center(), Align2::CENTER_CENTER, city.population.to_string(), font.clone(), primary);
    painter.galley(top + vec2(pop_w + 6.0, 3.0), galley, secondary);
    let mut y = banner.bottom() + 2.0;
    if mine
        && let Some(item) = &city.production {
            let cost = game.production_cost(city.owner, item).max(1);
            let done = city.progress.get(&wayciv_core::Game::production_key(item)).copied().unwrap_or(0);
            let bar = Rect::from_min_size(pos2(banner.left(), y), vec2(w, 4.0));
            painter.rect_filled(bar, 1.0, Color32::from_gray(30));
            painter.rect_filled(Rect::from_min_size(bar.min, vec2(w * (done as f32 / cost as f32).min(1.0), 4.0)), 1.0, Color32::from_rgb(240, 150, 60));
            y += 6.0;
        }
    if city.loyalty < wayciv_core::loyalty::MAX_LOYALTY && game.loyalty_applies(city.id) {
        // Loyalty: purple, turning red when the city is close to defecting.
        let bar = Rect::from_min_size(pos2(banner.left(), y), vec2(w, 4.0));
        painter.rect_filled(bar, 1.0, Color32::from_gray(30));
        let color = if city.loyalty > 25 { Color32::from_rgb(170, 110, 230) } else { Color32::from_rgb(230, 70, 70) };
        painter.rect_filled(Rect::from_min_size(bar.min, vec2(w * city.loyalty as f32 / 100.0, 4.0)), 1.0, color);
        y += 6.0;
    }
    if city.hp < 200 || city.wall_hp > 0 {
        let bar = Rect::from_min_size(pos2(banner.left(), y), vec2(w, 4.0));
        painter.rect_filled(bar, 1.0, Color32::from_gray(30));
        painter.rect_filled(Rect::from_min_size(bar.min, vec2(w * city.hp as f32 / 200.0, 4.0)), 1.0, Color32::from_rgb(80, 210, 90));
        if city.wall_hp > 0 {
            let walls = Rect::from_min_size(pos2(banner.left(), y + 5.0), vec2(w * (city.wall_hp as f32 / game.max_walls(city.id).max(1) as f32).min(1.0), 3.0));
            painter.rect_filled(walls, 1.0, Color32::from_rgb(120, 170, 240));
        }
    }
}

fn draw_unit(painter: &egui::Painter, icons: &mut crate::icons::Icons, game: &wayciv_core::Game, u: &wayciv_core::state::Unit, at: Pos2, radius: f32, selected: bool) {
    let owner = &game.players[u.owner];
    let primary = rgb(owner.primary);
    let secondary = rgb(owner.secondary);
    if selected {
        painter.circle_stroke(at, radius + 4.0, Stroke::new(3.0, Color32::WHITE));
    }
    let ring = if u.moves > 0 { secondary } else { secondary.gamma_multiply(0.5) };
    painter.circle(at, radius, primary, Stroke::new((radius * 0.15).max(1.5), ring));
    let ty = game.unit_type(u);
    let inner = radius * 0.82;
    let drew = radius > 9.0 && icons.paint(painter, "unit", &ty.id, Rect::from_center_size(at, vec2(inner * 2.0, inner * 2.0)));
    if !drew {
        painter.text(at, Align2::CENTER_CENTER, abbreviation(&ty.name), FontId::proportional(radius * 0.85), secondary);
    }
    // Civilians have no health bar: they are alive or dead.
    if u.hp < 100 && game.is_military(u) {
        let bar = Rect::from_min_size(at + vec2(-radius, radius + 2.0), vec2(radius * 2.0, 3.0));
        painter.rect_filled(bar, 1.0, Color32::from_gray(30));
        let color = if u.hp > 60 { Color32::from_rgb(80, 210, 90) } else if u.hp > 30 { Color32::from_rgb(230, 200, 60) } else { Color32::from_rgb(220, 60, 50) };
        painter.rect_filled(Rect::from_min_size(bar.min, vec2(radius * 2.0 * u.hp as f32 / 100.0, 3.0)), 1.0, color);
    }
    if u.fortified {
        painter.rect_filled(Rect::from_center_size(at + vec2(radius * 0.8, -radius * 0.8), vec2(radius * 0.5, radius * 0.5)), 2.0, Color32::from_rgb(200, 200, 200));
    }
}

/// The smallest zoom at which the map is still at least as wide as the view.
fn min_zoom(s: &Session, rect: Rect) -> f32 {
    let width = SQRT3 * s.game.map.grid.width as f32;
    (rect.width() / width).max(8.0)
}

fn handle_input(s: &mut Session, ui: &egui::Ui, rect: Rect, response: &egui::Response) {
    // Pan: drag or keyboard. Zoom: wheel toward the pointer.
    if response.dragged() {
        s.cam.center -= response.drag_delta() / s.cam.zoom;
    }
    let (scroll, keys) = ui.input(|i| {
        let mut keys = Vec2::ZERO;
        if !i.focused || i.modifiers.any() {
            // no-op
        }
        for (key, d) in [(egui::Key::A, vec2(-1.0, 0.0)), (egui::Key::D, vec2(1.0, 0.0)), (egui::Key::W, vec2(0.0, -1.0)), (egui::Key::S, vec2(0.0, 1.0)), (egui::Key::ArrowLeft, vec2(-1.0, 0.0)), (egui::Key::ArrowRight, vec2(1.0, 0.0)), (egui::Key::ArrowUp, vec2(0.0, -1.0)), (egui::Key::ArrowDown, vec2(0.0, 1.0))] {
            if i.key_down(key) {
                keys += d;
            }
        }
        (if response.hovered() { i.smooth_scroll_delta.y } else { 0.0 }, keys)
    });
    if !ui.ctx().egui_wants_keyboard_input() && keys != Vec2::ZERO {
        s.cam.center += keys * 0.5;
        ui.ctx().request_repaint();
    }
    if scroll != 0.0 {
        let pointer = response.hover_pos().unwrap_or(rect.center());
        let before = (pointer - rect.center()) / s.cam.zoom + s.cam.center;
        s.cam.zoom = (s.cam.zoom * (1.0 + scroll * 0.0015)).clamp(min_zoom(s, rect), 70.0);
        let after = (pointer - rect.center()) / s.cam.zoom + s.cam.center;
        s.cam.center += before - after;
    }
    // Never zoom out so far that the wrapping map shows itself twice.
    s.cam.zoom = s.cam.zoom.max(min_zoom(s, rect));
    let height = 1.5 * s.game.map.grid.height as f32;
    s.cam.center.y = s.cam.center.y.clamp(0.0, height);
    let width = SQRT3 * s.game.map.grid.width as f32;
    s.cam.center.x = s.cam.center.x.rem_euclid(width);

    let hover = response.hover_pos().and_then(|p| tile_at(s, rect, p));
    if hover != s.hover {
        s.hover = hover;
        s.update_path_preview();
    }
    if response.clicked()
        && let Some(pos) = hover {
            s.left_click(pos);
        }
    if response.secondary_clicked()
        && let Some(pos) = hover {
            s.right_click(pos);
        }
}

fn draw_aircraft(painter: &egui::Painter, game: &wayciv_core::Game, u: &wayciv_core::state::Unit, at: Pos2, r: f32, selected: bool) {
    let owner = &game.players[u.owner];
    let diamond = vec![at + vec2(0.0, -r), at + vec2(r, 0.0), at + vec2(0.0, r), at + vec2(-r, 0.0)];
    if selected {
        painter.circle_stroke(at, r + 4.0, Stroke::new(2.5, Color32::WHITE));
    }
    let ring = if u.moves > 0 { rgb(owner.secondary) } else { rgb(owner.secondary).gamma_multiply(0.5) };
    painter.add(Shape::convex_polygon(diamond, rgb(owner.primary), Stroke::new(1.5, ring)));
    let name = &game.unit_type(u).name;
    painter.text(at, Align2::CENTER_CENTER, abbreviation(name), FontId::proportional(r * 0.8), rgb(owner.secondary));
}

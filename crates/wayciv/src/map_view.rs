//! The hex map: what to draw where and in which order, plus mouse and
//! keyboard input. The art itself lives in `render`.

use crate::hud::Session;
use crate::palette::{self, rgb, with_alpha};
use crate::render::hex::{EDGES, SQRT3, corners, hex};
use crate::render::{self, Art, TileCtx};
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, Vec2, vec2};
use wayciv_core::{Pos, Production};

pub use crate::render::hex::world;

pub struct Camera {
    /// Center in world units (hex radius 1).
    pub center: Vec2,
    /// Hex radius in pixels.
    pub zoom: f32,
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

pub fn center_on(s: &mut Session, pos: Pos) {
    s.cam.center = world(pos);
}

fn neighbor(s: &Session, pos: Pos, dir: usize) -> Option<Pos> {
    render::hex::neighbor(&s.game.map.grid, pos, dir)
}

pub fn show(s: &mut Session, ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(rect, Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::from_rgb(8, 10, 14));

    handle_input(s, ui, rect, &response);
    let mut icons = std::mem::replace(&mut s.icons, crate::icons::Icons::new(Default::default()));
    let mut art = std::mem::replace(&mut s.art, Art::new(Default::default()));
    draw(s, rect, &response, &painter, &mut icons, &mut art);
    s.icons = icons;
    s.art = art;
}

fn draw(
    s: &Session,
    rect: Rect,
    response: &egui::Response,
    painter: &egui::Painter,
    icons: &mut crate::icons::Icons,
    art: &mut Art,
) {
    let game = &s.game;
    let me = game.human();
    let player = &game.players[me];
    let grid = game.map.grid;
    let z = s.cam.zoom;
    let ctx = |pos: Pos, c: Pos2| TileCtx {
        game,
        viewer: me,
        pos,
        c,
        z,
    };

    // 1. Terrain: biomes, relief, features, cliffs.
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
        render::terrain::draw_tile(&mut art.terrain, painter, &ctx(pos, c));
    }

    // 2. Rivers, then roads.
    for &(pos, c) in &tiles {
        render::terrain::draw_river(&mut art.terrain, painter, &ctx(pos, c));
    }
    for &(pos, c) in &tiles {
        render::resources::draw_road(&mut art.resources, painter, &ctx(pos, c));
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
                painter.line_segment(
                    [inner[a], inner[b]],
                    Stroke::new((z * 0.05).max(1.0), rgb(o.secondary)),
                );
            }
        }
    }

    // 4. Districts, wonders, improvements, resources; fog on top.
    for &(pos, c) in &tiles {
        let t = ctx(pos, c);
        render::buildings::draw_district(&mut art.buildings, painter, &t);
        render::resources::draw_improvement(&mut art.resources, painter, &t);
        render::resources::draw_resource(&mut art.resources, icons, painter, &t);
        if !player.visible[grid.index(pos)] {
            painter.add(hex(
                c,
                z,
                Color32::from_rgba_unmultiplied(0, 0, 0, 120),
                Stroke::NONE,
            ));
        }
    }

    // 5. Highlights: reachable tiles, attack targets, district sites, path.
    if let Some(unit) = s.selected_unit.and_then(|id| game.units.get(&id)) {
        for &pos in &s.reachable {
            for c in screens(s, rect, pos) {
                painter.add(hex(
                    c,
                    z * 0.9,
                    Color32::from_rgba_unmultiplied(120, 200, 255, 40),
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(150, 220, 255, 120)),
                ));
            }
        }
        for pos in grid.within(unit.pos, game.unit_type(unit).range.max(1)) {
            if game.can_attack(unit, pos) {
                for c in screens(s, rect, pos) {
                    painter.add(hex(
                        c,
                        z * 0.9,
                        Color32::from_rgba_unmultiplied(230, 50, 40, 60),
                        Stroke::new(2.0, Color32::from_rgb(240, 60, 50)),
                    ));
                }
            }
        }
        let mut prev = screens(s, rect, unit.pos).first().copied();
        for &step in &s.path_preview {
            let here = screens(s, rect, step)
                .into_iter()
                .min_by_key(|p| prev.map(|q| (p.distance(q) * 10.0) as i32).unwrap_or(0));
            if let (Some(a), Some(b)) = (prev, here) {
                painter.line_segment([a, b], Stroke::new(3.0, Color32::from_rgb(255, 255, 255)));
            }
            prev = here;
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
        for pos in sites {
            for c in screens(s, rect, pos) {
                let extra = if let Production::District(d, _) = item {
                    game.adjacency(*d, pos).iter().sum::<f32>()
                } else {
                    0.0
                };
                painter.add(hex(
                    c,
                    z * 0.9,
                    Color32::from_rgba_unmultiplied(80, 230, 120, 60),
                    Stroke::new(2.0, Color32::from_rgb(90, 240, 130)),
                ));
                if extra > 0.0 {
                    painter.text(
                        c,
                        Align2::CENTER_CENTER,
                        format!("+{extra:.0}"),
                        FontId::proportional(z * 0.45),
                        Color32::WHITE,
                    );
                }
            }
        }
    }
    if let Some(city) = s.selected_city.and_then(|id| game.cities.get(&id)) {
        for &pos in &city.worked {
            for c in screens(s, rect, pos) {
                painter.circle_stroke(
                    c,
                    z * 0.25,
                    Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 160)),
                );
            }
        }
    }

    // 6. Cities, units, then city banners on top.
    for city in game.cities.values() {
        if player.revealed[grid.index(city.pos)] {
            for c in screens(s, rect, city.pos) {
                render::buildings::draw_city(&mut art.buildings, painter, game, city, c, z);
            }
        }
    }
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
                    render::units::draw_aircraft(
                        &mut art.units,
                        icons,
                        painter,
                        game,
                        u,
                        at,
                        z * 0.22,
                        s.selected_unit == Some(u.id),
                    );
                    continue;
                }
                let military = game.is_military(u);
                let (at, radius) = if military {
                    (c + vec2(0.0, -0.05 * z), z * 0.42)
                } else {
                    civilian_slot += 1;
                    (
                        c + vec2(
                            -0.45 * z + 0.25 * z * ((civilian_slot - 1) % 4) as f32,
                            0.4 * z,
                        ),
                        z * 0.3,
                    )
                };
                render::units::draw_unit(
                    &mut art.units,
                    icons,
                    painter,
                    game,
                    u,
                    at,
                    radius,
                    s.selected_unit == Some(u.id),
                );
            }
        }
    }
    for city in game.cities.values() {
        if !player.revealed[grid.index(city.pos)] {
            continue;
        }
        for c in screens(s, rect, city.pos) {
            render::banner::draw_city_banner(painter, game, city, c, z, city.owner == me);
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
            painter.add(Shape::dashed_line(
                &[from, to],
                Stroke::new(2.0, palette::GOLD),
                8.0,
                6.0,
            ));
        }
    }

    // 7. Hover outline and tooltip.
    if let Some(pos) = s.hover {
        for c in screens(s, rect, pos) {
            painter.add(hex(
                c,
                z,
                Color32::TRANSPARENT,
                Stroke::new(2.0, Color32::WHITE),
            ));
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
        )
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
        && let Some(pos) = hover
    {
        s.left_click(pos);
    }
    if response.secondary_clicked()
        && let Some(pos) = hover
    {
        s.right_click(pos);
    }
}

//! The minimap: a cheap downscaled render of the tiles you have revealed.
//! It frames the explored area (so it is useful from turn one) or shows the
//! whole world; click or drag to recenter the camera. A button column beside
//! it zooms the map, toggles the framing and jumps to your selection.

use super::theme;
use super::widgets::{self, Tool};
use crate::hud::Session;
use crate::map_view;
use crate::palette::{self, GOLD_DIM};
use crate::render::hex::{SQRT3, neighbor};
use crate::render::veil::UNKNOWN;
use egui::{
    Color32, ColorImage, Rect, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, pos2,
    vec2,
};
use wayciv_core::Pos;

/// Width of the map part of the minimap on screen.
pub const WIDTH: f32 = 236.0;
/// Width of the button column and its gap.
const TOOLS: f32 = 28.0 + 6.0;
const PAD: f32 = 8.0;
/// The smallest framed area, in tile columns.
const MIN_COLS: i32 = 22;

/// The part of the world the minimap shows: `cols` x `rows` tiles from column
/// `x0` (wrapping east-west) and row `y0` (may poke past the map's edge).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Region {
    x0: i32,
    y0: i32,
    cols: i32,
    rows: i32,
}

#[derive(Default)]
pub struct Minimap {
    texture: Option<TextureHandle>,
    key: (u32, usize, usize, usize, bool),
    region: Region,
    /// Show the whole world instead of the explored area.
    pub whole: bool,
}

/// Size of the map part on screen for this map (it keeps the world's shape).
pub fn size(s: &Session) -> egui::Vec2 {
    let grid = s.game.map.grid;
    let ww = SQRT3 * grid.width as f32;
    let hw = 1.5 * grid.height as f32;
    vec2(WIDTH, WIDTH * hw / ww)
}

/// Size of the whole minimap plate: map, button column and padding.
pub fn outer_size(s: &Session) -> egui::Vec2 {
    let m = size(s);
    vec2(m.x + TOOLS + PAD * 2.0, m.y.max(4.0 * 28.0 + 3.0 * 4.0) + PAD * 2.0)
}

/// Where the explored part of the map is, with a margin, shaped like the
/// minimap box (`aspect` = width / height in world units).
fn frame(s: &Session, aspect: f32, whole: bool) -> Region {
    let game = &s.game;
    let grid = game.map.grid;
    let (gw, gh) = (grid.width as i32, grid.height as i32);
    if whole {
        return Region { x0: 0, y0: 0, cols: gw, rows: gh };
    }
    let player = &game.players[game.human()];
    let mut rows = vec![false; gh as usize];
    let mut cols = vec![false; gw as usize];
    for pos in grid.all() {
        if player.revealed[grid.index(pos)] {
            rows[pos.y as usize] = true;
            cols[pos.x as usize] = true;
        }
    }
    let (Some(r0), Some(r1)) = (
        rows.iter().position(|&r| r).map(|r| r as i32),
        rows.iter().rposition(|&r| r).map(|r| r as i32),
    ) else {
        return Region { x0: 0, y0: 0, cols: gw, rows: gh };
    };
    // Columns wrap: the explored span is everything except the longest
    // run of unexplored columns.
    let mut best = (0, 0); // (gap length, gap start)
    let mut run = 0;
    let mut start = 0;
    for i in 0..gw * 2 {
        if cols[(i % gw) as usize] {
            run = 0;
        } else {
            if run == 0 {
                start = i;
            }
            run += 1;
            if run > best.0 && run < gw {
                best = (run, start);
            }
        }
    }
    let (span, first) = if best.0 == 0 {
        (gw, 0)
    } else {
        (gw - best.0, (best.1 + best.0) % gw)
    };
    let margin = 4;
    let cols_need = (span + 2 * margin).max(MIN_COLS);
    let rows_need = (r1 - r0 + 1) + 2 * margin;
    // World size per tile: SQRT3 across, 1.5 down.
    let cols_for_rows = (rows_need as f32 * 1.5 / SQRT3 * aspect).ceil() as i32;
    let cols_n = cols_need.max(cols_for_rows);
    if cols_n >= gw {
        return Region { x0: 0, y0: 0, cols: gw, rows: gh };
    }
    let rows_n = ((cols_n as f32 * SQRT3 / aspect) / 1.5).ceil() as i32;
    let cx = first + span / 2;
    let cy = (r0 + r1) / 2;
    let x0 = (cx - cols_n / 2).rem_euclid(gw);
    let mut y0 = cy - rows_n / 2;
    if rows_n < gh {
        y0 = y0.clamp(0, gh - rows_n);
    }
    Region { x0, y0, cols: cols_n, rows: rows_n }
}

impl Minimap {
    fn refresh(&mut self, ctx: &egui::Context, s: &Session) {
        let game = &s.game;
        let me = game.human();
        let player = &game.players[me];
        let grid = game.map.grid;
        let revealed = player.revealed.iter().filter(|&&r| r).count();
        let visible = player.visible.iter().filter(|&&r| r).count();
        let key = (game.turn, revealed, visible, game.cities.len(), self.whole);
        if self.texture.is_some() && self.key == key {
            return;
        }
        let msize = size(s);
        let region = frame(s, msize.x / msize.y, self.whole);
        let gw = grid.width as i32;
        // Crisper pixels when few tiles share the box.
        let px: i32 = if (WIDTH / region.cols as f32) >= 4.0 { 6 } else { 3 };
        let (w, h) = ((region.cols * px) as usize, (region.rows * px) as usize);
        let mut image = ColorImage::filled([w, h], UNKNOWN);
        let mut put = |x: i32, y: i32, c: Color32, size: i32| {
            for dy in 0..size {
                for dx in 0..size {
                    let (ix, iy) = (x + dx, y + dy);
                    if ix >= 0 && iy >= 0 && (ix as usize) < w && (iy as usize) < h {
                        image.pixels[iy as usize * w + ix as usize] = c;
                    }
                }
            }
        };
        // Texture coordinates of a tile, or None when it's outside the region.
        let place = |pos: Pos| -> Option<(i32, i32)> {
            let dx = (pos.x - region.x0).rem_euclid(gw);
            let dy = pos.y - region.y0;
            (dx < region.cols && dy >= 0 && dy < region.rows).then(|| {
                (
                    dx * px + if pos.y & 1 == 1 { px / 2 } else { 0 },
                    dy * px,
                )
            })
        };
        for pos in grid.all() {
            let i = grid.index(pos);
            if !player.revealed[i] {
                continue;
            }
            let Some((x, y)) = place(pos) else { continue };
            let tile = game.map.tile(pos);
            let terrain = &game.rules.content.terrains[tile.terrain];
            let mut color = if terrain.mountain {
                Color32::from_rgb(112, 112, 116)
            } else {
                palette::biome(&terrain.id)
            };
            if terrain.hills {
                color = color.gamma_multiply(0.85);
            }
            if let Some(owner) = tile.owner {
                // Territory: a wash of the owner's color, strongest on the border.
                let o = palette::rgb(game.players[owner].primary);
                let edge = (0..6).any(|d| {
                    neighbor(&grid, pos, d).is_some_and(|n| game.map.tile(n).owner != Some(owner))
                });
                color = mix(color, o, if edge { 0.92 } else { 0.5 });
            }
            if !player.visible[i] {
                // Fog: the same sepia wash as on the map.
                color = mix(color, Color32::from_rgb(96, 82, 62), 0.38).gamma_multiply(0.78);
            }
            put(x, y, color, px);
        }
        for city in game.cities.values() {
            if !player.revealed[grid.index(city.pos)] {
                continue;
            }
            let Some((x, y)) = place(city.pos) else { continue };
            let o = &game.players[city.owner];
            let edge = px / 3 + 1;
            put(x - edge / 2 - 1, y - edge / 2 - 1, Color32::from_black_alpha(255), px + edge + 2);
            put(x - edge / 2, y - edge / 2, Color32::WHITE, px + edge);
            put(x, y, palette::rgb(o.primary), px - 1);
        }
        self.key = key;
        self.region = region;
        let filter = if px == 6 {
            TextureOptions::NEAREST
        } else {
            TextureOptions::LINEAR
        };
        self.texture = Some(ctx.load_texture("minimap", image, filter));
    }
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let f = |x: u8, y: u8| (x as f32 * (1.0 - t) + y as f32 * t) as u8;
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}

/// Draws the minimap with its bottom-right corner at `corner`.
pub fn show(s: &mut Session, ctx: &egui::Context, corner: egui::Pos2) {
    let outer = outer_size(s);
    let mut minimap = std::mem::take(&mut s.minimap);
    minimap.refresh(ctx, s);
    let mut toggle_whole = false;
    let mut zoom = 1.0;
    let mut recenter = false;
    egui::Area::new(egui::Id::unique("minimap"))
        .fixed_pos(corner - outer)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let (plate, _) = ui.allocate_exact_size(outer, Sense::hover());
            let painter = ui.painter().clone();
            theme::plate(&painter, plate, 12.0);
            // Button column on the left.
            let mut y = plate.top() + PAD;
            let x = plate.left() + PAD;
            let mut tool = |ui: &mut egui::Ui, t: Tool, active: bool, name: &str, tip: &str| {
                let r = Rect::from_min_size(pos2(x, y), vec2(28.0, 28.0));
                y += 32.0;
                widgets::tool_button_at(ui, r, t, active, name, tip, None)
            };
            if tool(ui, Tool::Plus, false, "Zoom in", "Zoom the map in.") {
                zoom = 1.3;
            }
            if tool(ui, Tool::Minus, false, "Zoom out", "Zoom the map out.") {
                zoom = 1.0 / 1.3;
            }
            if tool(
                ui,
                Tool::Globe,
                minimap.whole,
                if minimap.whole { "Frame explored area" } else { "Show whole world" },
                if minimap.whole {
                    "Zoom the minimap in on the part of the world you have explored."
                } else {
                    "Show the whole world in the minimap."
                },
            ) {
                toggle_whole = true;
            }
            if tool(
                ui,
                Tool::Target,
                false,
                "Center on selection",
                "Center the map on the selected unit or city, or on your capital.",
            ) {
                recenter = true;
            }

            let area = Rect::from_min_max(
                pos2(plate.left() + PAD + TOOLS, plate.top() + PAD),
                plate.right_bottom() - vec2(PAD, PAD),
            );
            let region = minimap.region;
            let (ww, wh) = (SQRT3 * region.cols as f32, 1.5 * region.rows as f32);
            let scale = (area.width() / ww).min(area.height() / wh);
            let inner = Rect::from_center_size(area.center(), vec2(ww * scale, wh * scale));
            let response = ui.interact(inner, ui.scope_id().with("minimap_map"), Sense::click_and_drag());
            let map_painter = painter.with_clip_rect(inner);
            map_painter.rect_filled(inner, 0.0, UNKNOWN);
            if let Some(tex) = &minimap.texture {
                map_painter.image(
                    tex.id(),
                    inner,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            // The camera's view, drawn again on each side for east-west wrap.
            let grid = s.game.map.grid;
            let world_w = SQRT3 * grid.width as f32;
            let view = s.view.size() / s.cam.zoom;
            let half = view * scale / 2.0;
            let cx = (s.cam.center.x - SQRT3 * region.x0 as f32).rem_euclid(world_w);
            let cy = s.cam.center.y - 1.5 * region.y0 as f32;
            for k in [-1.0, 0.0, 1.0] {
                let c = inner.min + vec2((cx + k * world_w) * scale, cy * scale);
                let r = Rect::from_center_size(c, half * 2.0);
                map_painter.rect_stroke(r, 2.0, Stroke::new(3.5, Color32::from_black_alpha(140)), StrokeKind::Middle);
                map_painter.rect_stroke(r, 2.0, Stroke::new(1.5, Color32::WHITE), StrokeKind::Middle);
            }
            // An inset shadow and a gold rim frame the chart.
            for (k, a) in [(0.0, 90), (2.0, 50), (4.0, 24)] {
                map_painter.rect_stroke(inner.shrink(k), 0.0, Stroke::new(2.0, Color32::from_black_alpha(a)), StrokeKind::Inside);
            }
            painter.rect_stroke(inner, 4.0, Stroke::new(1.0, GOLD_DIM), StrokeKind::Outside);
            if (response.clicked() || response.dragged() || response.is_pointer_button_down_on())
                && let Some(p) = response.interact_pointer_pos()
            {
                let u = ((p.x - inner.left()) / inner.width()).clamp(0.0, 1.0);
                let v = ((p.y - inner.top()) / inner.height()).clamp(0.0, 1.0);
                let wx = SQRT3 * region.x0 as f32 + u * ww;
                let wy = 1.5 * region.y0 as f32 + v * wh;
                let row = (wy / 1.5).round().clamp(0.0, grid.height as f32 - 1.0) as i32;
                let col = ((wx / SQRT3 - 0.5 * (row & 1) as f32).round() as i32)
                    .rem_euclid(grid.width as i32);
                map_view::center_on(s, Pos::new(col, row));
            }
            response.on_hover_text("Click or drag to move the camera");
        });
    if toggle_whole {
        minimap.whole = !minimap.whole;
    }
    s.minimap = minimap;
    if zoom != 1.0 {
        s.cam.zoom = (s.cam.zoom * zoom).min(70.0);
    }
    if recenter {
        let target = s
            .selected_unit
            .and_then(|id| s.game.units.get(&id))
            .map(|u| u.pos)
            .or_else(|| {
                s.selected_city
                    .and_then(|id| s.game.cities.get(&id))
                    .map(|c| c.pos)
            })
            .or_else(|| {
                s.game
                    .player_cities(s.me())
                    .find(|c| c.capital)
                    .or_else(|| s.game.player_cities(s.me()).next())
                    .map(|c| c.pos)
            })
            .or_else(|| s.game.player_units(s.me()).next().map(|u| u.pos));
        if let Some(p) = target {
            map_view::center_on(s, p);
        }
    }
}

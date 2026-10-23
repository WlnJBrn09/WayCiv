//! The minimap: a cheap downscaled render of the tiles you have revealed.
//! Click or drag to recenter the camera.

use super::theme;
use crate::hud::Session;
use crate::map_view;
use crate::palette::{self, GOLD, GOLD_DIM, NAVY};
use crate::render::hex::SQRT3;
use egui::{
    Align2, Color32, ColorImage, Rect, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions,
    pos2, vec2,
};
use wayciv_core::Pos;

/// Pixels per tile in the minimap texture.
const PX: usize = 3;
/// Width of the minimap on screen.
pub const WIDTH: f32 = 236.0;

#[derive(Default)]
pub struct Minimap {
    texture: Option<TextureHandle>,
    key: (u32, usize, usize, usize),
}

impl Minimap {
    fn refresh(&mut self, ctx: &egui::Context, s: &Session) -> Option<&TextureHandle> {
        let game = &s.game;
        let me = game.human();
        let player = &game.players[me];
        let grid = game.map.grid;
        let revealed = player.revealed.iter().filter(|&&r| r).count();
        let visible = player.visible.iter().filter(|&&r| r).count();
        let key = (game.turn, revealed, visible, game.cities.len());
        if self.texture.is_some() && self.key == key {
            return self.texture.as_ref();
        }
        let (gw, gh) = (grid.width as usize, grid.height as usize);
        let (w, h) = (gw * PX, gh * PX);
        let mut image = ColorImage::filled([w, h], Color32::from_rgb(15, 22, 34));
        let mut put = |x: usize, y: usize, c: Color32, size: usize| {
            for dy in 0..size {
                for dx in 0..size {
                    let px = (x + dx) % w;
                    let py = y + dy;
                    if py < h {
                        image.pixels[py * w + px] = c;
                    }
                }
            }
        };
        for pos in grid.all() {
            let i = grid.index(pos);
            if !player.revealed[i] {
                continue;
            }
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
                let o = palette::rgb(game.players[owner].primary);
                color = mix(color, o, 0.55);
            }
            if !player.visible[i] {
                color = color.gamma_multiply(0.62);
            }
            let x = pos.x as usize * PX + if pos.y & 1 == 1 { PX / 2 + 1 } else { 0 };
            put(x, pos.y as usize * PX, color, PX);
        }
        for city in game.cities.values() {
            if !player.revealed[grid.index(city.pos)] {
                continue;
            }
            let o = &game.players[city.owner];
            let x = city.pos.x as usize * PX + if city.pos.y & 1 == 1 { PX / 2 + 1 } else { 0 };
            let y = city.pos.y as usize * PX;
            put(x.saturating_sub(1), y.saturating_sub(1), Color32::WHITE, PX + 1);
            put(x, y, palette::rgb(o.primary), PX - 1);
        }
        self.key = key;
        self.texture = Some(ctx.load_texture("minimap", image, TextureOptions::LINEAR));
        self.texture.as_ref()
    }
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let f = |x: u8, y: u8| (x as f32 * (1.0 - t) + y as f32 * t) as u8;
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}

/// Size of the minimap on screen for this map.
pub fn size(s: &Session) -> egui::Vec2 {
    let grid = s.game.map.grid;
    let ww = SQRT3 * grid.width as f32;
    let hw = 1.5 * grid.height as f32;
    vec2(WIDTH, WIDTH * hw / ww)
}

/// Draws the minimap with its bottom-right corner at `corner`.
pub fn show(s: &mut Session, ctx: &egui::Context, corner: egui::Pos2) {
    let map_size = size(s);
    let pad = 8.0;
    let outer = vec2(map_size.x + pad * 2.0, map_size.y + pad * 2.0);
    let mut minimap = std::mem::take(&mut s.minimap);
    minimap.refresh(ctx, s);
    egui::Area::new(egui::Id::unique("minimap"))
        .fixed_pos(corner - outer)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(outer, Sense::click_and_drag());
            let painter = ui.painter().clone();
            theme::plate(&painter, rect, 12.0);
            let inner = Rect::from_min_size(rect.min + vec2(pad, pad), map_size);
            let map_painter = painter.with_clip_rect(inner);
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
            let ww = SQRT3 * grid.width as f32;
            let hw = 1.5 * grid.height as f32;
            let to_mini = |v: egui::Vec2| vec2(v.x / ww * inner.width(), v.y / hw * inner.height());
            let view = s.view.size() / s.cam.zoom;
            let half = to_mini(view) / 2.0;
            let center = inner.min + to_mini(s.cam.center);
            for k in [-1.0, 0.0, 1.0] {
                let c = center + vec2(k * inner.width(), 0.0);
                let r = Rect::from_center_size(c, half * 2.0);
                map_painter.rect_stroke(r, 2.0, Stroke::new(1.5, Color32::WHITE), StrokeKind::Middle);
            }
            painter.rect_stroke(inner, 4.0, Stroke::new(1.0, GOLD_DIM), StrokeKind::Outside);
            if (response.clicked() || response.dragged() || response.is_pointer_button_down_on())
                && let Some(p) = response.interact_pointer_pos()
            {
                let u = ((p.x - inner.left()) / inner.width()).clamp(0.0, 1.0);
                let v = ((p.y - inner.top()) / inner.height()).clamp(0.0, 1.0);
                let (wx, wy) = (u * ww, v * hw);
                let row = (wy / 1.5).round().clamp(0.0, grid.height as f32 - 1.0) as i32;
                let col = (wx / SQRT3 - 0.5 * (row & 1) as f32)
                    .round()
                    .clamp(0.0, grid.width as f32 - 1.0) as i32;
                map_view::center_on(s, Pos::new(col, row));
            }
            response.on_hover_text("Click or drag to move the camera");
            let _ = (GOLD, NAVY, Align2::CENTER_CENTER);
        });
    s.minimap = minimap;
}

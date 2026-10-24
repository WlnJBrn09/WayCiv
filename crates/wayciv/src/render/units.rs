//! Units: land and sea units in their tile slot, aircraft at their base.
//!
//! Each unit type is a little 3D model rendered to a sprite with Blender
//! (`tools/units`, output in `assets/units/<id>.png`).  A second image,
//! `<id>_mask.png`, holds the team-coloured parts (grey shading with alpha);
//! it is tinted with the owner's colour and drawn over the sprite.  Around the
//! model: a soft ground shadow, a selection ring, an owner-coloured badge with
//! a class symbol, a health bar (military only) and a fortified marker.
//! When a sprite is missing we fall back to the Codex icon, then to the class
//! symbol on a disc.  Nothing here panics on missing files.
//!
//! Units are alive: `UnitFx` carries the animation state (idle breathing,
//! ships bobbing, the pulsing selection ring, hop, hit flash, fade).

use crate::icons::Icons;
use crate::palette::rgb;
use egui::{
    Color32, Pos2, Rect, Shape, Stroke, TextureFilter, TextureHandle, TextureOptions, pos2, vec2,
};
use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI};
use std::path::PathBuf;

/// Below this radius (px) units are a plain disc with a vector symbol.
const TINY: f32 = 9.0;

struct Sprite {
    beauty: TextureHandle,
    mask: Option<TextureHandle>,
    /// Canvas width in model units (bigger ships get bigger canvases).
    ortho: f32,
    /// Where the ground point sits in the canvas, 0 = top, 1 = bottom.
    anchor: f32,
    /// The opaque part of the image, in 0..1 canvas coordinates.
    bounds: Rect,
}

pub struct UnitArt {
    assets: PathBuf,
    /// `id -> (ortho, anchor)` from `assets/units/manifest.tsv`, read on first use.
    manifest: Option<HashMap<String, (f32, f32)>>,
    sprites: HashMap<String, Option<Sprite>>,
    /// The Settler's visible height and width in hex radii: every unit is
    /// drawn this tall (and at most a little wider) so none of them towers
    /// over the tiles behind it.
    settler_size: Option<(f32, f32)>,
}

impl UnitArt {
    pub fn new(assets: PathBuf) -> Self {
        Self {
            assets,
            manifest: None,
            sprites: HashMap::new(),
            settler_size: None,
        }
    }

    fn manifest(&mut self) -> &HashMap<String, (f32, f32)> {
        let dir = self.assets.join("units");
        self.manifest.get_or_insert_with(|| {
            let mut map = HashMap::new();
            if let Ok(text) = std::fs::read_to_string(dir.join("manifest.tsv")) {
                for line in text.lines().filter(|l| !l.starts_with('#')) {
                    let mut f = line.split('\t');
                    if let (Some(id), Some(o), Some(a)) = (f.next(), f.next(), f.next())
                        && let (Ok(o), Ok(a)) = (o.parse(), a.parse())
                    {
                        map.insert(id.to_string(), (o, a));
                    }
                }
            }
            map
        })
    }

    /// Loads the sprite for a unit id (lowercase) the first time; true when it exists.
    fn ensure(&mut self, ctx: &egui::Context, id: &str) -> bool {
        if !self.sprites.contains_key(id) {
            let loaded = self.load(ctx, id);
            self.sprites.insert(id.to_string(), loaded);
        }
        self.sprites.get(id).is_some_and(|s| s.is_some())
    }

    /// The first existing sprite among `ids`.
    fn find(&mut self, ctx: &egui::Context, ids: &[String]) -> Option<&Sprite> {
        let hit = ids.iter().find(|id| self.ensure(ctx, id))?;
        self.sprites.get(hit).and_then(|s| s.as_ref())
    }

    fn load(&mut self, ctx: &egui::Context, id: &str) -> Option<Sprite> {
        let dir = self.assets.join("units");
        let opts = TextureOptions::LINEAR.with_mipmap_mode(Some(TextureFilter::Linear));
        let read = |path: PathBuf, name: String| {
            let img = image::open(path).ok()?.to_rgba8();
            let size = [img.width() as usize, img.height() as usize];
            let texture = ctx.load_texture(
                name,
                egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()),
                opts,
            );
            Some((texture, opaque_bounds(&img)))
        };
        let (beauty, bounds) = read(dir.join(format!("{id}.png")), format!("unit/{id}"))?;
        let mask = read(dir.join(format!("{id}_mask.png")), format!("unit/{id}_mask")).map(|m| m.0);
        let (ortho, anchor) = self.manifest().get(id).copied().unwrap_or((2.6, 0.78));
        Some(Sprite {
            beauty,
            mask,
            ortho,
            anchor,
            bounds,
        })
    }
}

/// What a unit does, for its badge symbol.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    Melee,
    Ranged,
    Cavalry,
    Siege,
    Naval,
    Air,
    Civilian,
    Religious,
    Great,
}

pub fn class_of(ty: &wayciv_core::rules::Unit) -> Class {
    let id = ty.id.as_str();
    let promo = ty.promotion_class.as_deref().unwrap_or("");
    if id.starts_with("UNIT_GREAT_") || id == "UNIT_COMANDANTE_GENERAL" {
        return Class::Great;
    }
    if ty.spread_charges > 0
        || ty.found_religion
        || matches!(
            id,
            "UNIT_MISSIONARY" | "UNIT_APOSTLE" | "UNIT_INQUISITOR" | "UNIT_GURU"
        )
    {
        return Class::Religious;
    }
    let armed = ty.combat > 0 || ty.ranged_combat > 0 || ty.bombard > 0;
    match ty.domain.as_str() {
        "DOMAIN_AIR" => return Class::Air,
        "DOMAIN_SEA" => {
            return if armed {
                Class::Naval
            } else {
                Class::Civilian
            };
        }
        _ => {}
    }
    match promo {
        "PROMOTION_CLASS_SIEGE" => Class::Siege,
        "PROMOTION_CLASS_RANGED" => Class::Ranged,
        "PROMOTION_CLASS_LIGHT_CAVALRY"
        | "PROMOTION_CLASS_HEAVY_CAVALRY"
        | "PROMOTION_CLASS_RECON" => Class::Cavalry,
        "PROMOTION_CLASS_MELEE"
        | "PROMOTION_CLASS_ANTI_CAVALRY"
        | "PROMOTION_CLASS_MONK"
        | "PROMOTION_CLASS_NIHANG" => Class::Melee,
        _ if matches!(id, "UNIT_BATTERING_RAM" | "UNIT_SIEGE_TOWER") => Class::Siege,
        _ if armed => Class::Melee,
        _ => Class::Civilian,
    }
}

/// Sprite ids to try for a unit type: its own, then the unit it replaces.
fn sprite_ids(game: &wayciv_core::Game, ty: &wayciv_core::rules::Unit) -> Vec<String> {
    let mut ids = vec![ty.id.to_lowercase()];
    let mut cur = ty;
    for _ in 0..3 {
        let Some(rep) = cur.replaces.as_deref() else {
            break;
        };
        ids.push(rep.to_lowercase());
        match game.rules.content.units.iter().find(|u| u.id == rep) {
            Some(next) => cur = next,
            None => break,
        }
    }
    ids
}

/// Darkens a colour (units that have no moves left).
fn dim(c: Color32, on: bool) -> Color32 {
    if on { c.gamma_multiply(0.62) } else { c }
}

/// Canvas size of a sprite in px for a unit slot of `radius`.
/// The box around a sprite's visible pixels, in 0..1 canvas coordinates.
fn opaque_bounds(img: &image::RgbaImage) -> Rect {
    let (w, h) = img.dimensions();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] > 24 {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x + 1);
            y1 = y1.max(y + 1);
        }
    }
    if x1 <= x0 || y1 <= y0 {
        return Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    }
    Rect::from_min_max(
        pos2(x0 as f32 / w as f32, y0 as f32 / h as f32),
        pos2(x1 as f32 / w as f32, y1 as f32 / h as f32),
    )
}

/// Canvas size in pixels that makes `sprite` exactly `height` pixels tall
/// above its ground point, but never wider than `max_width`.
fn uniform_px(sprite: &Sprite, height: f32, max_width: f32) -> f32 {
    let above = (sprite.anchor - sprite.bounds.top()).max(0.05);
    let wide = sprite.bounds.width().max(0.05);
    (height / above).min(max_width / wide)
}

impl UnitArt {
    /// The Settler's visible (height, width) in radii, measured once from
    /// its sprite (the width includes its baked-in ground shadow).
    fn settler_size(&mut self, ctx: &egui::Context) -> (f32, f32) {
        if let Some(size) = self.settler_size {
            return size;
        }
        let size = self
            .find(ctx, &["unit_settler".to_string()])
            .map(|s| {
                let px = canvas_px(1.0, s.ortho);
                (px * (s.anchor - s.bounds.top()).max(0.05), px * s.bounds.width())
            })
            .unwrap_or((2.3, 4.2));
        self.settler_size = Some(size);
        size
    }
}

fn canvas_px(radius: f32, ortho: f32) -> f32 {
    // Figures are drawn larger than the slot; big ships stay inside the tile.
    let boost = (1.35 - 0.06 * (ortho - 2.4)).clamp(1.0, 1.35);
    radius * 2.0 * (1.0 + 0.28 * ortho) * boost
}

/// Animation state for one unit drawing (see `anim::Pose`). `REST` draws it
/// still; the map passes the clock so idle motion plays.
#[derive(Clone, Copy)]
pub struct UnitFx {
    /// Seconds on the animation clock, for idle motion (0: none).
    pub time: f32,
    pub alpha: f32,
    /// Red hit flash, 0 to 1.
    pub flash: f32,
    pub scale: f32,
    /// Pixels the unit hovers above the ground (hop, walking bob).
    pub lift: f32,
}

impl UnitFx {
    pub const REST: UnitFx = UnitFx {
        time: 0.0,
        alpha: 1.0,
        flash: 0.0,
        scale: 1.0,
        lift: 0.0,
    };
}

/// A stable 0..TAU phase per unit so a crowd doesn't breathe in step.
fn phase(id: u32) -> f32 {
    let h = id.wrapping_mul(0x9E37_79B1) ^ (id >> 7).wrapping_mul(0x85EB_CA77);
    (h >> 8 & 0xffff) as f32 / 65535.0 * std::f32::consts::TAU
}

fn red_glow(strength: f32) -> Color32 {
    Color32::from_rgb(255, 70, 50).gamma_multiply((strength * 0.85).clamp(0.0, 1.0)).additive()
}

#[allow(clippy::too_many_arguments)]
fn draw_sprite(
    sprite: &Sprite,
    painter: &egui::Painter,
    ground: Pos2,
    px: f32,
    team: Color32,
    spent: bool,
    stretch: f32,
    flash: f32,
) {
    let min = ground - vec2(px * 0.5, px * sprite.anchor * stretch);
    let rect = Rect::from_min_size(min, vec2(px, px * stretch));
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    painter.image(sprite.beauty.id(), rect, uv, dim(Color32::WHITE, spent));
    if let Some(mask) = &sprite.mask {
        painter.image(mask.id(), rect, uv, dim(team, spent));
    }
    if flash > 0.0 {
        painter.image(sprite.beauty.id(), rect, uv, red_glow(flash));
    }
}

fn ground_shadow(painter: &egui::Painter, ground: Pos2, rx: f32, alpha: u8) {
    for (k, a) in [(1.0, 0.35), (0.78, 0.35), (0.55, 0.4)] {
        painter.add(Shape::ellipse_filled(
            ground,
            vec2(rx * k, rx * k * 0.42),
            Color32::from_black_alpha((alpha as f32 * a) as u8),
        ));
    }
}

/// The ring under the selected unit: breathes, and a second ring ripples out.
fn selection_ring(painter: &egui::Painter, ground: Pos2, rx: f32, time: f32) {
    let pulse = if time > 0.0 { 0.5 + 0.5 * (time * 3.4).sin() } else { 1.0 };
    let r = vec2(rx * (1.0 + 0.03 * pulse), rx * 0.46 * (1.0 + 0.03 * pulse));
    painter.add(Shape::ellipse_filled(
        ground,
        r,
        Color32::from_white_alpha((20.0 + 22.0 * pulse) as u8),
    ));
    painter.add(Shape::ellipse_stroke(
        ground,
        r,
        Stroke::new(2.5, Color32::WHITE.gamma_multiply(0.78 + 0.22 * pulse)),
    ));
    if time > 0.0 {
        let k = (time * 0.8).fract();
        let out = vec2(rx * (1.0 + 0.35 * k), rx * 0.46 * (1.0 + 0.35 * k));
        painter.add(Shape::ellipse_stroke(
            ground,
            out,
            Stroke::new(1.5, Color32::WHITE.gamma_multiply(0.45 * (1.0 - k))),
        ));
    }
}

/// A land or sea unit centered on `at`. `radius` is about 0.42 hex radii for
/// the military unit and 0.3 for civilians (which share the tile's bottom row).
#[allow(clippy::too_many_arguments)]
pub fn draw_unit(
    art: &mut UnitArt,
    icons: &mut Icons,
    painter: &egui::Painter,
    game: &wayciv_core::Game,
    u: &wayciv_core::state::Unit,
    at: Pos2,
    radius: f32,
    selected: bool,
    fx: &UnitFx,
) {
    let owner = &game.players[u.owner];
    let primary = rgb(owner.primary);
    let secondary = rgb(owner.secondary);
    let ty = game.unit_type(u);
    let class = class_of(ty);
    let spent = u.moves == 0;
    let military = game.is_military(u);
    let mut faded = painter.clone();
    if fx.alpha < 1.0 {
        faded.set_opacity(fx.alpha);
    }
    let painter = &faded;
    // Idle life: land units breathe, ships ride the swell.
    let ph = phase(u.id);
    let naval = matches!(class, Class::Naval) || ty.domain == "DOMAIN_SEA";
    let (stretch, lift) = if fx.time > 0.0 && !is_still(u) {
        if naval {
            (1.0, fx.lift + radius * 0.07 * (fx.time * 1.3 + ph).sin() + radius * 0.07)
        } else {
            (1.0 + 0.014 * (fx.time * 1.7 + ph).sin(), fx.lift)
        }
    } else {
        (1.0, fx.lift)
    };
    let ground_at = at;
    let at = at - vec2(0.0, lift);

    if radius < TINY {
        // Tiny zoom: owner-coloured disc with the class symbol.
        if selected {
            let pulse = 0.5 + 0.5 * (fx.time * 3.4).sin();
            painter.circle_stroke(
                at,
                radius + 3.0,
                Stroke::new(2.5, Color32::WHITE.gamma_multiply(0.75 + 0.25 * pulse)),
            );
        }
        painter.circle(
            at,
            radius,
            dim(primary, spent),
            Stroke::new((radius * 0.16).max(1.0), dim(secondary, spent)),
        );
        draw_glyph(painter, class, at, radius * 0.62, dim(secondary, spent));
        if fx.flash > 0.0 {
            painter.circle_filled(at, radius, red_glow(fx.flash));
        }
        hp_bar(u, painter, ground_at, radius, military);
        return;
    }

    // The shadow and ring stay on the ground while the unit hops.
    let ground = ground_at + vec2(0.0, radius * 0.55);
    let hover = ground - vec2(0.0, lift);
    let ids = sprite_ids(game, ty);
    let (height, width) = art.settler_size(painter.ctx());
    let has_sprite = if let Some(s) = art.find(painter.ctx(), &ids) {
        // Every unit stands as tall as a Settler; long ships are kept to
        // about a Settler's width.
        let px = uniform_px(s, height * radius, width * radius * 1.1);
        let rx = (radius * (0.8 + 0.12 * s.ortho)).min(radius * 1.7).min(px * s.bounds.width() * 0.5);
        if selected {
            selection_ring(painter, ground, rx, fx.time);
        }
        let high = (1.0 - lift / (radius * 3.0)).clamp(0.6, 1.0);
        ground_shadow(painter, ground, rx * 0.9 * high, if spent { 70 } else { 95 });
        draw_sprite(s, painter, hover, px * fx.scale, primary, spent, stretch, fx.flash);
        true
    } else {
        // No model: Codex icon on a disc, else the class symbol.
        if selected {
            selection_ring(painter, ground, radius * 0.85, fx.time);
        }
        painter.circle(
            at,
            radius,
            dim(primary, spent),
            Stroke::new((radius * 0.15).max(1.5), dim(secondary, spent)),
        );
        let inner = radius * 0.82;
        let drew = icons.paint(
            painter,
            "unit",
            &ty.id,
            Rect::from_center_size(at, vec2(inner * 2.0, inner * 2.0)),
        );
        if !drew {
            draw_glyph(painter, class, at, radius * 0.6, dim(secondary, spent));
        }
        if fx.flash > 0.0 {
            painter.circle_filled(at, radius, red_glow(fx.flash));
        }
        false
    };

    // Owner flag with the class symbol on the top left; fortified shield top right.
    if has_sprite {
        let br = (radius * 0.4).max(6.0);
        draw_badge(painter, class, at + vec2(-radius * 0.82, -radius * 0.78), br, primary, secondary, spent);
        if u.fortified {
            draw_fortified(painter, at + vec2(radius * 0.82, -radius * 0.78), br * 0.9, spent);
        }
    } else if u.fortified {
        draw_fortified(painter, at + vec2(radius * 0.8, -radius * 0.8), radius * 0.4, spent);
    }
    hp_bar(u, painter, ground_at, radius, military);
}

/// Fortified units stand at attention: no idle breathing.
fn is_still(u: &wayciv_core::state::Unit) -> bool {
    u.fortified
}

fn hp_bar(u: &wayciv_core::state::Unit, painter: &egui::Painter, at: Pos2, radius: f32, military: bool) {
    // Civilians have no health bar: they are alive or dead.
    if u.hp >= 100 || !military {
        return;
    }
    let w = radius * 1.7;
    let h = (radius * 0.14).clamp(2.5, 5.0);
    let bar = Rect::from_min_size(at + vec2(-w / 2.0, radius * 1.1 + 2.0), vec2(w, h));
    painter.rect_filled(bar.expand(1.0), 1.5, Color32::from_black_alpha(200));
    let color = if u.hp > 60 {
        Color32::from_rgb(80, 210, 90)
    } else if u.hp > 30 {
        Color32::from_rgb(230, 200, 60)
    } else {
        Color32::from_rgb(220, 60, 50)
    };
    painter.rect_filled(
        Rect::from_min_size(bar.min, vec2(w * u.hp.max(0) as f32 / 100.0, h)),
        1.0,
        color,
    );
}

/// An owner-coloured round flag with a class symbol.
fn draw_badge(painter: &egui::Painter, class: Class, c: Pos2, r: f32, primary: Color32, secondary: Color32, spent: bool) {
    let ink = dim(secondary, spent);
    painter.circle_filled(c + vec2(0.0, 1.0), r + 1.0, Color32::from_black_alpha(90));
    painter.circle(c, r, dim(primary, spent), Stroke::new((r * 0.16).max(1.0), ink));
    draw_glyph(painter, class, c, r * 0.64, ink);
}

fn draw_fortified(painter: &egui::Painter, c: Pos2, r: f32, spent: bool) {
    let col = dim(Color32::from_rgb(214, 218, 224), spent);
    let edge = Stroke::new(1.2, Color32::from_rgb(60, 66, 76));
    let pts = vec![
        c + vec2(-r * 0.8, -r * 0.85),
        c + vec2(r * 0.8, -r * 0.85),
        c + vec2(r * 0.8, r * 0.1),
        c + vec2(0.0, r),
        c + vec2(-r * 0.8, r * 0.1),
    ];
    painter.add(Shape::convex_polygon(pts, col, edge));
    painter.line_segment([c + vec2(0.0, -r * 0.6), c + vec2(0.0, r * 0.55)], edge);
}

/// Class symbols drawn as vectors inside a circle of radius `r` centered on `c`.
pub fn draw_glyph(painter: &egui::Painter, class: Class, c: Pos2, r: f32, color: Color32) {
    let w = (r * 0.2).max(1.0);
    let st = Stroke::new(w, color);
    let p = |x: f32, y: f32| c + vec2(x * r, y * r);
    match class {
        Class::Melee => {
            // Crossed swords.
            painter.line_segment([p(-0.8, 0.8), p(0.8, -0.8)], st);
            painter.line_segment([p(0.8, 0.8), p(-0.8, -0.8)], st);
            painter.line_segment([p(-0.62, 0.2), p(-0.2, 0.62)], st);
            painter.line_segment([p(0.62, 0.2), p(0.2, 0.62)], st);
        }
        Class::Ranged => {
            // Bow and arrow.
            let pts: Vec<Pos2> = (0..=10)
                .map(|i| {
                    let t = -1.0 + 0.2 * i as f32;
                    p(-0.45 + 0.5 * t * t, t * 0.9)
                })
                .collect();
            painter.add(Shape::line(pts, st));
            painter.line_segment([p(0.05, -0.9), p(0.05, 0.9)], Stroke::new(w * 0.6, color));
            painter.line_segment([p(-0.3, 0.0), p(0.8, 0.0)], st);
            painter.add(Shape::convex_polygon(
                vec![p(0.95, 0.0), p(0.55, -0.25), p(0.55, 0.25)],
                color,
                Stroke::NONE,
            ));
        }
        Class::Cavalry => {
            // Horseshoe, opening downwards.
            let pts: Vec<Pos2> = (0..=12)
                .map(|i| {
                    let a = PI * (0.12 + 1.76 * i as f32 / 12.0);
                    p(0.7 * -a.cos(), -0.05 - 0.75 * a.sin())
                })
                .collect();
            painter.add(Shape::line(pts, Stroke::new(w * 1.5, color)));
            for (x, y) in [(-0.62, -0.45), (0.62, -0.45), (-0.5, 0.15), (0.5, 0.15)] {
                painter.circle_filled(p(x, y), w * 0.45, color);
            }
        }
        Class::Siege => {
            // Catapult arm throwing a stone.
            painter.line_segment([p(-0.8, 0.75), p(0.8, 0.75)], st);
            painter.line_segment([p(-0.5, 0.75), p(0.5, -0.35)], Stroke::new(w * 1.2, color));
            painter.circle_filled(p(0.62, -0.62), r * 0.24, color);
            painter.circle_stroke(p(-0.45, 0.6), r * 0.2, Stroke::new(w * 0.8, color));
        }
        Class::Naval => {
            // Anchor.
            painter.line_segment([p(0.0, -0.55), p(0.0, 0.8)], st);
            painter.circle_stroke(p(0.0, -0.7), r * 0.18, Stroke::new(w * 0.9, color));
            painter.line_segment([p(-0.4, -0.25), p(0.4, -0.25)], st);
            let pts: Vec<Pos2> = (0..=8)
                .map(|i| {
                    let a = PI * i as f32 / 8.0;
                    p(0.75 * a.cos(), 0.15 + 0.62 * a.sin())
                })
                .collect();
            painter.add(Shape::line(pts, st));
        }
        Class::Air => {
            // Plane seen from above.
            let body = vec![p(0.0, -0.95), p(0.14, -0.5), p(0.14, 0.7), p(0.0, 0.9), p(-0.14, 0.7), p(-0.14, -0.5)];
            painter.add(Shape::convex_polygon(body, color, Stroke::NONE));
            let wings = vec![p(0.0, -0.25), p(0.95, 0.25), p(0.95, 0.42), p(0.0, 0.2), p(-0.95, 0.42), p(-0.95, 0.25)];
            painter.add(Shape::convex_polygon(wings, color, Stroke::NONE));
            let tail = vec![p(0.0, 0.6), p(0.45, 0.85), p(0.45, 0.95), p(0.0, 0.85), p(-0.45, 0.95), p(-0.45, 0.85)];
            painter.add(Shape::convex_polygon(tail, color, Stroke::NONE));
        }
        Class::Civilian => {
            // A worker: head and shoulders.
            painter.circle_filled(p(0.0, -0.4), r * 0.3, color);
            let mut body: Vec<Pos2> = (0..=8)
                .map(|i| {
                    let a = PI * (1.0 + i as f32 / 8.0);
                    p(0.7 * a.cos(), 0.85 + 0.75 * a.sin())
                })
                .collect();
            body.push(p(0.7, 0.85));
            body.push(p(-0.7, 0.85));
            painter.add(Shape::convex_polygon(body, color, Stroke::NONE));
        }
        Class::Religious => {
            // A cross with sparks around it.
            painter.line_segment([p(0.0, -0.8), p(0.0, 0.85)], st);
            painter.line_segment([p(-0.5, -0.25), p(0.5, -0.25)], st);
            for (x, y) in [(-0.8, -0.8), (0.8, -0.8), (-0.8, 0.0), (0.8, 0.0)] {
                painter.circle_filled(p(x * 0.8, y * 0.8), w * 0.4, color);
            }
        }
        Class::Great => {
            // A five-pointed star.
            let pts: Vec<Pos2> = (0..10)
                .map(|i| {
                    let a = -FRAC_PI_2 + PI * i as f32 / 5.0;
                    let rr = if i % 2 == 0 { 1.0 } else { 0.42 };
                    p(rr * a.cos(), rr * a.sin() + 0.05)
                })
                .collect();
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
        }
    }
}

/// An aircraft at its base: a small model at `at` of size `r`.
#[allow(clippy::too_many_arguments)]
pub fn draw_aircraft(
    art: &mut UnitArt,
    _icons: &mut Icons,
    painter: &egui::Painter,
    game: &wayciv_core::Game,
    u: &wayciv_core::state::Unit,
    at: Pos2,
    r: f32,
    selected: bool,
) {
    let owner = &game.players[u.owner];
    let primary = rgb(owner.primary);
    let secondary = rgb(owner.secondary);
    let ty = game.unit_type(u);
    let spent = u.moves == 0;
    if selected {
        painter.circle_stroke(at, r * 1.25 + 2.0, Stroke::new(2.5, Color32::WHITE));
    }
    let ids = sprite_ids(game, ty);
    let sprite = if r >= 5.0 { art.find(painter.ctx(), &ids) } else { None };
    if let Some(s) = sprite {
        // The plane hovers above a soft shadow.
        ground_shadow(painter, at + vec2(r * 0.2, r * 1.0), r * 0.75, 70);
        let px = r * 2.0 * (0.62 + 0.2 * s.ortho);
        draw_sprite(s, painter, at + vec2(0.0, px * (s.anchor - 0.5)), px, primary, spent, 1.0, 0.0);
        draw_badge(painter, Class::Air, at + vec2(-r * 0.95, -r * 0.8), (r * 0.62).max(5.0), primary, secondary, spent);
        hp_bar(u, painter, at + vec2(0.0, -r * 0.55), r * 0.9, game.is_military(u));
    } else {
        painter.circle(
            at,
            r,
            dim(primary, spent),
            Stroke::new((r * 0.16).max(1.0), dim(secondary, spent)),
        );
        draw_glyph(painter, Class::Air, at, r * 0.62, dim(secondary, spent));
    }
}

//! Districts, wonders, and city centers.
//!
//! The art is rendered with Blender (see `tools/buildings`) into
//! `assets/buildings/*.png`: one sprite per district (up to three levels,
//! depending on how many of its buildings the city has), per wonder, and per
//! city center (four eras by four population tiers, plus capitals and
//! walls). City sprites come with a `_tint.png` layer that is painted again
//! in the owner's color. Textures load on first use and come in three sizes
//! so zoomed-out views stay smooth. When a sprite is missing, or the map is
//! zoomed far out, a simple vector drawing takes over.

use super::TileCtx;
use super::hex::{corners, hex};
use crate::palette::{self, rgb};
use egui::{
    Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, TextureHandle, TextureOptions, pos2, vec2,
};
use std::collections::HashMap;
use std::path::PathBuf;
use wayciv_core::{Game, PlayerId, Pos};

/// Sprite canvas, in hex radii: 2.2 wide, 2.75 tall, with the hex center
/// 60% of the way down (see `tools/buildings/bl.py`).
const SPRITE_W: f32 = 2.2;
const SPRITE_H: f32 = 2.75;
const GROUND_FRAC: f32 = 0.60;
/// Below this hex radius the vector fallback is used.
const SPRITE_MIN_Z: f32 = 12.0;

/// One loaded sprite in several sizes (largest first).
struct Sprite {
    lods: Vec<(u32, TextureHandle)>,
}

impl Sprite {
    /// The smallest texture that is still about as wide as `width_px`.
    fn pick(&self, width_px: f32) -> &TextureHandle {
        let mut best = &self.lods[0].1;
        for (w, tex) in &self.lods {
            if *w as f32 >= width_px * 0.85 {
                best = tex;
            }
        }
        best
    }
}

#[derive(Default)]
struct FrameCache {
    frame: Option<u64>,
    /// District position -> level (1 to 3: no buildings, one, two or more).
    districts: HashMap<Pos, u8>,
    eras: HashMap<PlayerId, u8>,
}

pub struct BuildingArt {
    dir: PathBuf,
    sprites: HashMap<String, Option<Sprite>>,
    cache: FrameCache,
}

impl BuildingArt {
    pub fn new(assets: PathBuf) -> Self {
        Self {
            dir: assets.join("buildings"),
            sprites: HashMap::new(),
            cache: FrameCache::default(),
        }
    }

    fn load(&mut self, ctx: &egui::Context, name: &str) -> Option<&Sprite> {
        if !self.sprites.contains_key(name) {
            let sprite = load_sprite(ctx, &self.dir, name);
            self.sprites.insert(name.to_string(), sprite);
        }
        self.sprites.get(name).and_then(|s| s.as_ref())
    }

    /// Paint sprite `name` for a tile centered at `c`; false if it is missing.
    fn paint(
        &mut self,
        painter: &egui::Painter,
        name: &str,
        c: Pos2,
        z: f32,
        tint: Option<Color32>,
    ) -> bool {
        let ctx = painter.ctx().clone();
        let ppp = ctx.pixels_per_point();
        let rect = Rect::from_min_size(
            pos2(c.x - SPRITE_W * z / 2.0, c.y - GROUND_FRAC * SPRITE_H * z),
            vec2(SPRITE_W * z, SPRITE_H * z),
        );
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        let Some(sprite) = self.load(&ctx, name) else {
            return false;
        };
        painter.image(sprite.pick(rect.width() * ppp).id(), rect, uv, Color32::WHITE);
        if let Some(color) = tint
            && let Some(layer) = self.load(&ctx, &format!("{name}_tint"))
        {
            painter.image(layer.pick(rect.width() * ppp).id(), rect, uv, color);
        }
        true
    }

    fn has(&mut self, ctx: &egui::Context, name: &str) -> bool {
        self.load(ctx, name).is_some()
    }

    /// Once per frame: which district has how many buildings, and each
    /// owner's era.
    fn begin(&mut self, ctx: &egui::Context, game: &Game) {
        let frame = ctx.cumulative_pass_nr();
        if self.cache.frame == Some(frame) {
            return;
        }
        self.cache.frame = Some(frame);
        self.cache.eras.clear();
        self.cache.districts.clear();
        let content = &game.rules.content;
        for city in game.cities.values() {
            for cd in &city.districts {
                let district = &content.districts[cd.district];
                let count = city
                    .buildings
                    .iter()
                    .filter(|&&b| {
                        let b = &content.buildings[b];
                        let want = b.prereq_district.as_deref();
                        !b.wonder
                            && want.is_some()
                            && (want == Some(district.id.as_str())
                                || want == district.replaces.as_deref())
                    })
                    .count();
                self.cache.districts.insert(cd.pos, 1 + count.min(2) as u8);
            }
        }
    }

    fn era(&mut self, game: &Game, owner: PlayerId) -> u8 {
        *self.cache.eras.entry(owner).or_insert_with(|| {
            let p = game.game_progress(owner);
            if p < 0.22 {
                0
            } else if p < 0.5 {
                1
            } else if p < 0.78 {
                2
            } else {
                3
            }
        })
    }
}

fn load_sprite(ctx: &egui::Context, dir: &std::path::Path, name: &str) -> Option<Sprite> {
    let img = image::open(dir.join(format!("{name}.png"))).ok()?.to_rgba8();
    let mut lods = Vec::new();
    let mut cur = img;
    loop {
        let (w, h) = (cur.width(), cur.height());
        let size = [w as usize, h as usize];
        let tex = ctx.load_texture(
            format!("buildings/{name}/{w}"),
            egui::ColorImage::from_rgba_unmultiplied(size, cur.as_raw()),
            TextureOptions::LINEAR,
        );
        lods.push((w, tex));
        if w <= 64 || lods.len() >= 3 {
            break;
        }
        cur = image::imageops::resize(&cur, w / 2, h / 2, image::imageops::FilterType::Triangle);
    }
    Some(Sprite { lods })
}

/// The base district a unique district replaces.
fn base_district(d: &civ6_data::content::District) -> &str {
    if let Some(r) = &d.replaces {
        return r;
    }
    match d.id.as_str() {
        "DISTRICT_ACROPOLIS" => "DISTRICT_THEATER",
        "DISTRICT_BATH" => "DISTRICT_AQUEDUCT",
        "DISTRICT_HANSA" => "DISTRICT_INDUSTRIAL_ZONE",
        "DISTRICT_LAVRA" => "DISTRICT_HOLY_SITE",
        "DISTRICT_MBANZA" => "DISTRICT_NEIGHBORHOOD",
        "DISTRICT_STREET_CARNIVAL" => "DISTRICT_ENTERTAINMENT_COMPLEX",
        "DISTRICT_ROYAL_NAVY_DOCKYARD" => "DISTRICT_HARBOR",
        "DISTRICT_HIPPODROME" | "DISTRICT_OPPIDUM" | "DISTRICT_THANH" => "DISTRICT_ENCAMPMENT",
        "DISTRICT_OBSERVATORY" => "DISTRICT_CAMPUS",
        _ => &d.id,
    }
}

fn slug(id: &str) -> String {
    id.trim_start_matches("DISTRICT_")
        .trim_start_matches("BUILDING_")
        .to_lowercase()
}

/// Layer 4: a district (other than a city center) or a wonder on the tile.
pub fn draw_district(art: &mut BuildingArt, painter: &egui::Painter, t: &TileCtx) {
    let (game, c, z) = (t.game, t.c, t.z);
    let content = &game.rules.content;
    let tile = t.tile();
    if let Some(w) = tile.wonder {
        if z >= SPRITE_MIN_Z {
            let name = format!("wonder_{}", slug(&content.buildings[w].id));
            if art.paint(painter, &name, c, z, None)
                || art.paint(painter, "wonder_generic", c, z, None)
            {
                return;
            }
        }
        fallback_wonder(painter, c, z);
        return;
    }
    let Some(d) = tile.district else { return };
    let district = &content.districts[d];
    if district.city_center {
        return;
    }
    let base = base_district(district);
    if z >= SPRITE_MIN_Z {
        art.begin(painter.ctx(), game);
        let level = art.cache.districts.get(&t.pos).copied().unwrap_or(1);
        for lv in (1..=level).rev() {
            let own = format!("district_{}_{lv}", slug(&district.id));
            let shared = format!("district_{}_{lv}", slug(base));
            if (district.id != base && art.paint(painter, &own, c, z, None))
                || art.paint(painter, &shared, c, z, None)
            {
                return;
            }
        }
    }
    fallback_district(painter, c, z, base);
}

/// Layer 6: a city center, under its banner. `c` is the tile's screen center.
pub fn draw_city(
    art: &mut BuildingArt,
    painter: &egui::Painter,
    game: &Game,
    city: &wayciv_core::state::City,
    c: Pos2,
    z: f32,
) {
    let owner = &game.players[city.owner];
    let primary = rgb(owner.primary);
    if z >= SPRITE_MIN_Z {
        let ctx = painter.ctx().clone();
        art.begin(&ctx, game);
        let era = art.era(game, city.owner);
        let tier = match city.population {
            ..=3 => 1,
            4..=8 => 2,
            9..=15 => 3,
            _ => 4,
        };
        let generic = format!("city_{era}_{tier}");
        let capital = format!("capital_{era}_{tier}");
        let name = if city.capital && art.has(&ctx, &capital) {
            capital
        } else {
            generic
        };
        if art.has(&ctx, &name) {
            let walls = (city.wall_hp > 0).then(|| format!("walls_{}", era.saturating_sub(1).min(2)));
            if let Some(w) = &walls {
                art.paint(painter, &format!("{w}_back"), c, z, None);
            }
            art.paint(painter, &name, c, z, Some(primary));
            if let Some(w) = &walls {
                art.paint(painter, &format!("{w}_front"), c, z, None);
            }
            return;
        }
    }
    fallback_city(painter, c, z, primary, rgb(owner.secondary), city.capital, city.wall_hp > 0);
}

// ---------------------------------------------------------------------------
// Vector fallbacks: used when zoomed far out or when a sprite is missing.
// ---------------------------------------------------------------------------

fn shade(c: Color32, f: f32) -> Color32 {
    let s = |v: u8| ((v as f32 * f).clamp(0.0, 255.0)) as u8;
    Color32::from_rgb(s(c.r()), s(c.g()), s(c.b()))
}

/// A little block standing on `base` (bottom center), with a lit roof.
fn mini_block(painter: &egui::Painter, base: Pos2, w: f32, h: f32, color: Color32) {
    let (hw, d) = (w / 2.0, w * 0.28);
    let top = base - vec2(0.0, h);
    painter.add(Shape::convex_polygon(
        vec![
            base + vec2(-hw, 0.0),
            base + vec2(hw, 0.0),
            top + vec2(hw, 0.0),
            top + vec2(-hw, 0.0),
        ],
        shade(color, 0.8),
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![
            top + vec2(-hw, 0.0),
            top + vec2(hw, 0.0),
            top + vec2(hw - d, -d),
            top + vec2(-hw - d, -d),
        ],
        shade(color, 1.15),
        Stroke::NONE,
    ));
}

fn fallback_district(painter: &egui::Painter, c: Pos2, z: f32, base: &str) {
    let (color, label) = palette::district(base);
    painter.add(hex(
        c,
        z * 0.66,
        shade(color, 0.72),
        Stroke::new(1.5, Color32::from_gray(30)),
    ));
    if z > 14.0 {
        mini_block(painter, c + vec2(-z * 0.18, z * 0.28), z * 0.38, z * 0.32, color);
        mini_block(painter, c + vec2(z * 0.22, z * 0.12), z * 0.3, z * 0.46, shade(color, 1.1));
    } else {
        painter.add(hex(c, z * 0.4, color, Stroke::NONE));
    }
    if z > 11.0 {
        painter.text(
            c + vec2(0.0, z * 0.02),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(z * 0.4),
            if z > 14.0 {
                Color32::WHITE
            } else {
                Color32::from_gray(20)
            },
        );
    }
}

fn fallback_wonder(painter: &egui::Painter, c: Pos2, z: f32) {
    let edge = Color32::from_rgb(90, 60, 10);
    painter.add(hex(c, z * 0.66, shade(palette::GOLD, 0.7), Stroke::new(1.5, edge)));
    if z > 11.0 {
        // A little temple: a pediment over three columns.
        let top = c + vec2(0.0, -z * 0.42);
        painter.add(Shape::convex_polygon(
            vec![top, c + vec2(z * 0.36, -z * 0.12), c + vec2(-z * 0.36, -z * 0.12)],
            palette::GOLD,
            Stroke::new(1.0, edge),
        ));
        for dx in [-0.24, 0.0, 0.24] {
            painter.rect_filled(
                Rect::from_min_max(
                    c + vec2(z * (dx - 0.04), -z * 0.08),
                    c + vec2(z * (dx + 0.04), z * 0.34),
                ),
                0.0,
                Color32::from_rgb(250, 235, 190),
            );
        }
    } else {
        painter.add(hex(c, z * 0.36, palette::GOLD, Stroke::NONE));
    }
}

fn fallback_city(
    painter: &egui::Painter,
    c: Pos2,
    z: f32,
    primary: Color32,
    secondary: Color32,
    capital: bool,
    walls: bool,
) {
    let ring = if walls {
        Color32::from_rgb(170, 160, 140)
    } else {
        secondary
    };
    painter.add(hex(c, z * 0.72, primary, Stroke::new((z * 0.1).max(2.0), ring)));
    if z > 10.0 {
        mini_block(painter, c + vec2(-z * 0.26, z * 0.3), z * 0.3, z * 0.26, Color32::from_rgb(220, 205, 170));
        mini_block(painter, c + vec2(z * 0.28, z * 0.28), z * 0.28, z * 0.22, Color32::from_rgb(200, 180, 150));
        mini_block(painter, c + vec2(0.0, z * 0.18), z * 0.34, z * 0.42, Color32::from_rgb(235, 225, 195));
    } else {
        painter.circle_filled(c, z * 0.22, secondary);
    }
    if capital {
        let star = corners(c + vec2(0.0, -z * 0.5), z * 0.16);
        painter.add(Shape::convex_polygon(
            star.to_vec(),
            palette::GOLD,
            Stroke::new(1.0, Color32::BLACK),
        ));
    }
}

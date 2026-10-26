//! City banners, Civ VI style: a population circle ringed with growth
//! progress (and the turns until the city grows), the city name on a plate in
//! the owner's colors with a capital star, the current production with its
//! turns, and loyalty and health bars.

use crate::palette::{self, rgb};
use crate::ui::glyph::{self, Glyph};
use crate::ui::theme;
use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, pos2, vec2};
use std::cell::RefCell;
use std::collections::HashMap;
use wayciv_core::state::Production;

/// What a banner needs from the city report, kept for the turn: computing it
/// every frame for every city would be wasteful.
#[derive(Clone)]
struct Cached {
    key: (u32, i32, i32, usize, Option<String>, i32),
    turns_to_grow: Option<i32>,
    surplus: f32,
    threshold: i32,
    production_turns: i32,
}

thread_local! {
    static CACHE: RefCell<HashMap<wayciv_core::CityId, Cached>> = RefCell::new(HashMap::new());
    static ICONS: RefCell<crate::icons::Icons> =
        RefCell::new(crate::icons::Icons::new(crate::portraits::assets_dir()));
}

fn details(game: &wayciv_core::Game, city: &wayciv_core::state::City) -> Cached {
    let prod_key = city.production.as_ref().map(wayciv_core::Game::production_key);
    let key = (
        game.turn,
        city.population,
        city.food,
        city.buildings.len() + city.districts.len() + city.worked.len(),
        prod_key,
        city.progress.values().sum::<i32>(),
    );
    if let Some(c) = CACHE.with(|c| c.borrow().get(&city.id).cloned())
        && c.key == key
    {
        return c;
    }
    let report = game.city_report(city.id);
    let production_turns = city
        .production
        .as_ref()
        .map(|item| {
            let cost = game.production_cost(city.owner, item);
            let done = city
                .progress
                .get(&wayciv_core::Game::production_key(item))
                .copied()
                .unwrap_or(0);
            let rate = game.production_toward(city.id, item).max(0.1);
            ((cost - done).max(0) as f32 / rate).ceil() as i32
        })
        .unwrap_or(0);
    let cached = Cached {
        key,
        turns_to_grow: report.turns_to_grow,
        surplus: report.yields[0] - report.food_consumed,
        threshold: report.growth_threshold,
        production_turns,
    };
    CACHE.with(|c| c.borrow_mut().insert(city.id, cached.clone()));
    cached
}

fn luminance(c: Color32) -> f32 {
    0.299 * c.r() as f32 + 0.587 * c.g() as f32 + 0.114 * c.b() as f32
}

fn item_glyph(item: &Production) -> Glyph {
    match item {
        Production::Unit(_) => Glyph::Swords,
        Production::Building(_) => Glyph::Housing,
        Production::District(..) => Glyph::Hex,
        Production::Wonder(..) => Glyph::Star,
        Production::Project(_) => Glyph::Production,
    }
}

pub fn draw_city_banner(
    painter: &egui::Painter,
    game: &wayciv_core::Game,
    city: &wayciv_core::state::City,
    c: Pos2,
    z: f32,
    mine: bool,
) {
    let owner = &game.players[city.owner];
    let primary = rgb(owner.primary);
    let secondary = rgb(owner.secondary);
    // Keep the name readable whatever the civ colors are.
    let text_color = if (luminance(primary) - luminance(secondary)).abs() > 90.0 {
        secondary
    } else if luminance(primary) > 140.0 {
        Color32::from_rgb(12, 18, 30)
    } else {
        Color32::WHITE
    };
    let fs = (z * 0.46).clamp(11.0, 20.0);
    let pr = fs * 1.2 + 3.0;
    let ph = fs * 1.5 + 4.0;
    let name_font = FontId::proportional(fs);
    let galley = painter.layout_no_wrap(city.name.clone(), name_font, text_color);
    let star_w = if city.capital { fs + 4.0 } else { 0.0 };
    let name_x = pr * 0.75 + 6.0;
    let content = name_x + star_w + galley.size().x + 12.0;
    let info = if mine { Some(details(game, city)) } else { None };
    let production = if mine { city.production.as_ref() } else { None };
    let prod_r = ph * 0.82;
    let total = pr + content + if production.is_some() { 2.0 * prod_r } else { 0.0 };
    let x0 = c.x - total / 2.0;
    let cx = x0 + pr;
    let cy = c.y - z * 1.05 - pr + 4.0;

    // Plate: a pill with a soft shadow, lit from above, trimmed in the
    // civilization's second color.
    let plate_right = cx + content + if production.is_some() { prod_r } else { 0.0 };
    let plate = Rect::from_min_max(pos2(cx, cy - ph / 2.0), pos2(plate_right, cy + ph / 2.0));
    let radius = ph / 2.0;
    shadow(painter, plate, radius);
    painter.rect_filled(plate, radius, primary);
    let (top, bottom) = plate.split_top_bottom_at_fraction(0.5);
    let round = radius as u8;
    painter.rect_filled(top, CornerRadius { nw: round, ne: round, sw: 0, se: 0 }, Color32::from_white_alpha(30));
    painter.rect_filled(bottom, CornerRadius { nw: 0, ne: 0, sw: round, se: round }, Color32::from_black_alpha(36));
    painter.rect_stroke(plate, radius, Stroke::new(1.0, Color32::from_black_alpha(190)), StrokeKind::Outside);
    painter.rect_stroke(plate, radius, Stroke::new(1.8, secondary), StrokeKind::Inside);
    painter.rect_stroke(
        plate.shrink(2.8),
        (radius - 2.8).max(0.0),
        Stroke::new(1.0, Color32::from_white_alpha(44)),
        StrokeKind::Inside,
    );
    let mut tx = cx + name_x;
    if city.capital {
        glyph::paint(painter, Glyph::Star, pos2(tx + fs * 0.42, cy + 0.8), fs * 0.5, Color32::from_black_alpha(120));
        glyph::paint(painter, Glyph::Star, pos2(tx + fs * 0.42, cy), fs * 0.5, palette::GOLD);
        tx += star_w;
    }
    // A hairline shadow keeps the name crisp on any plate color.
    let ink = if luminance(text_color) > 128.0 {
        Color32::from_black_alpha(150)
    } else {
        Color32::from_white_alpha(70)
    };
    let name_at = pos2(tx, cy - galley.size().y / 2.0);
    painter.galley_with_override_text_color(name_at + vec2(0.0, 1.0), galley.clone(), ink);
    painter.galley(name_at, galley, text_color);

    // Population circle with the growth ring.
    let pop = pos2(cx, cy);
    medallion(painter, pop, pr);
    let ring_w = (fs * 0.28).max(3.0);
    let ring_r = pr - ring_w / 2.0 - 1.0;
    match &info {
        Some(d) => {
            let frac = city.food as f32 / d.threshold.max(1) as f32;
            let color = if d.surplus < 0.0 {
                Color32::from_rgb(240, 90, 80)
            } else {
                palette::GOOD
            };
            theme::ring(painter, pop, ring_r, frac, ring_w, color);
        }
        None => {
            painter.circle_stroke(pop, ring_r, Stroke::new(ring_w, secondary));
        }
    }
    theme::text_bold(
        painter,
        pop + vec2(0.0, 1.0),
        Align2::CENTER_CENTER,
        city.population,
        fs * 1.25,
        Color32::from_black_alpha(140),
    );
    theme::text_bold(painter, pop, Align2::CENTER_CENTER, city.population, fs * 1.25, Color32::WHITE);
    // Turns until growth, under the circle.
    if let Some(d) = &info
        && fs >= 12.0
    {
        let label = match d.turns_to_grow {
            Some(t) => t.to_string(),
            None if d.surplus < 0.0 => "!".to_string(),
            None => "-".to_string(),
        };
        let tag_c = pos2(cx, cy + pr + fs * 0.35);
        let tag = Rect::from_center_size(tag_c, vec2(fs * 2.1, fs * 1.05));
        capsule(painter, tag, palette::GOOD);
        glyph::paint(painter, Glyph::Food, pos2(tag.left() + fs * 0.55, tag_c.y), fs * 0.38, palette::GOOD);
        theme::text_bold(
            painter,
            pos2(tag.right() - fs * 0.55, tag_c.y),
            Align2::CENTER_CENTER,
            label,
            fs * 0.8,
            Color32::WHITE,
        );
    }

    // Production: icon with a progress ring, turns in a tag.
    if let (Some(item), Some(d)) = (production, &info) {
        let pc = pos2(plate_right, cy);
        let cost = game.production_cost(city.owner, item).max(1);
        let done = city
            .progress
            .get(&wayciv_core::Game::production_key(item))
            .copied()
            .unwrap_or(0);
        medallion(painter, pc, prod_r);
        theme::ring(
            painter,
            pc,
            prod_r - 2.5,
            done as f32 / cost as f32,
            3.0,
            Color32::from_rgb(240, 150, 60),
        );
        let icon = Rect::from_center_size(pc, vec2(prod_r * 1.25, prod_r * 1.25));
        let drawn = match item {
            Production::Unit(u) => {
                let id = game.rules.content.units[*u].id.clone();
                ICONS.with(|icons| icons.borrow_mut().paint(painter, "unit", &id, icon))
            }
            _ => false,
        };
        if !drawn {
            glyph::paint(painter, item_glyph(item), pc, prod_r * 0.5, Color32::from_rgb(240, 170, 80));
        }
        if fs >= 12.0 {
            let tag_c = pos2(pc.x, pc.y + prod_r + fs * 0.1);
            let tag = Rect::from_center_size(tag_c, vec2(fs * 1.7, fs * 1.0));
            capsule(painter, tag, Color32::from_rgb(240, 150, 60));
            theme::text_bold(
                painter,
                tag_c,
                Align2::CENTER_CENTER,
                format!("{}t", d.production_turns),
                fs * 0.76,
                Color32::WHITE,
            );
        }
    }

    // Bars: loyalty, then health and walls, under the plate.
    let bar_left = cx + pr * 0.9;
    let bar_w = (plate_right - bar_left).max(20.0);
    let mut y = cy + ph / 2.0 + 3.0;
    let bar_h = (fs * 0.3).max(4.0);
    if city.loyalty < wayciv_core::loyalty::MAX_LOYALTY && game.loyalty_applies(city.id) {
        // Loyalty: purple, turning red when the city is close to defecting.
        let color = if city.loyalty > 25 {
            Color32::from_rgb(176, 120, 236)
        } else {
            Color32::from_rgb(236, 70, 70)
        };
        let bar = Rect::from_min_size(pos2(bar_left, y), vec2(bar_w, bar_h));
        meter(painter, bar, city.loyalty as f32 / 100.0, color);
        y += bar_h + 2.0;
    }
    if city.hp < 200 || city.wall_hp > 0 {
        // Health: green, then amber, then red as the city is battered.
        let frac = city.hp as f32 / 200.0;
        let color = if frac >= 0.6 {
            Color32::from_rgb(86, 214, 96)
        } else if frac >= 0.3 {
            Color32::from_rgb(244, 196, 62)
        } else {
            Color32::from_rgb(238, 74, 66)
        };
        let bar = Rect::from_min_size(pos2(bar_left, y), vec2(bar_w, bar_h));
        meter(painter, bar, frac, color);
        if city.wall_hp > 0 {
            let walls = Rect::from_min_size(pos2(bar_left, y + bar_h + 1.5), vec2(bar_w, bar_h * 0.7));
            let held = (city.wall_hp as f32 / game.max_walls(city.id).max(1) as f32).min(1.0);
            meter(painter, walls, held, Color32::from_rgb(124, 176, 244));
        }
    }
}

/// A soft drop shadow under a rounded rect: a few widening, fading layers.
fn shadow(painter: &egui::Painter, rect: Rect, radius: f32) {
    for (grow, alpha) in [(0.5, 70), (2.5, 38), (5.0, 18)] {
        painter.rect_filled(
            rect.expand(grow).translate(vec2(0.0, 2.5)),
            radius + grow,
            Color32::from_black_alpha(alpha),
        );
    }
}

/// A round navy badge with a gold rim, lit from above, standing on a soft
/// shadow: the base of the population and production circles.
fn medallion(painter: &egui::Painter, c: Pos2, r: f32) {
    for (grow, alpha) in [(0.5, 80), (2.5, 40), (5.0, 18)] {
        painter.circle_filled(c + vec2(0.0, 2.5), r + grow, Color32::from_black_alpha(alpha));
    }
    painter.circle_filled(c, r, Color32::from_rgb(10, 17, 30));
    painter.circle_filled(c - vec2(0.0, r * 0.3), r * 0.62, Color32::from_white_alpha(9));
    painter.circle_stroke(c, r + 1.0, Stroke::new(1.0, Color32::from_black_alpha(190)));
    painter.circle_stroke(c, r, Stroke::new(2.0, palette::GOLD_DIM));
    painter.circle_stroke(c, r - 1.8, Stroke::new(1.0, palette::with_alpha(palette::GOLD, 70)));
}

/// A small dark pill with a colored trim, for the numbers under the circles.
fn capsule(painter: &egui::Painter, rect: Rect, trim: Color32) {
    painter.rect_filled(rect.translate(vec2(0.0, 1.5)), rect.height() / 2.0, Color32::from_black_alpha(90));
    painter.rect_filled(rect, rect.height() / 2.0, Color32::from_rgb(10, 17, 30));
    painter.rect_stroke(rect, rect.height() / 2.0, Stroke::new(1.2, trim), StrokeKind::Inside);
}

/// A thin meter: a dark outlined track and a lit fill.
fn meter(painter: &egui::Painter, rect: Rect, frac: f32, color: Color32) {
    let r = rect.height() / 2.0;
    painter.rect_filled(rect, r, Color32::from_black_alpha(200));
    let fill = Rect::from_min_size(
        rect.min,
        vec2((rect.width() * frac.clamp(0.0, 1.0)).max(rect.height()), rect.height()),
    );
    painter.rect_filled(fill.shrink(0.5), r, color);
    let (top, _) = fill.shrink(1.0).split_top_bottom_at_fraction(0.42);
    painter.rect_filled(top, r, Color32::from_white_alpha(56));
    painter.rect_stroke(rect, r, Stroke::new(1.0, Color32::from_white_alpha(46)), StrokeKind::Outside);
}

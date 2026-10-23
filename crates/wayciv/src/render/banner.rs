//! City banners, Civ VI style: a population circle ringed with growth
//! progress (and the turns until the city grows), the city name on a plate in
//! the owner's colors with a capital star, the current production with its
//! turns, and loyalty and health bars.

use crate::palette::{self, rgb};
use crate::ui::glyph::{self, Glyph};
use crate::ui::theme;
use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind, pos2, vec2};
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

    // Plate.
    let plate_right = cx + content + if production.is_some() { prod_r } else { 0.0 };
    let plate = Rect::from_min_max(pos2(cx, cy - ph / 2.0), pos2(plate_right, cy + ph / 2.0));
    painter.rect_filled(plate.translate(vec2(0.0, 2.0)), ph / 2.0, Color32::from_black_alpha(90));
    painter.rect_filled(plate, ph / 2.0, primary);
    painter.rect_stroke(plate, ph / 2.0, Stroke::new(1.8, secondary), StrokeKind::Inside);
    let mut tx = cx + name_x;
    if city.capital {
        glyph::paint(painter, Glyph::Star, pos2(tx + fs * 0.42, cy), fs * 0.5, palette::GOLD);
        tx += star_w;
    }
    painter.galley(pos2(tx, cy - galley.size().y / 2.0), galley, text_color);

    // Population circle with the growth ring.
    painter.circle_filled(pos2(cx, cy + 2.0), pr + 1.0, Color32::from_black_alpha(100));
    painter.circle_filled(pos2(cx, cy), pr, Color32::from_rgb(12, 20, 34));
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
            theme::ring(painter, pos2(cx, cy), ring_r, frac, ring_w, color);
        }
        None => {
            painter.circle_stroke(pos2(cx, cy), ring_r, Stroke::new(ring_w, secondary));
        }
    }
    painter.circle_stroke(pos2(cx, cy), pr, Stroke::new(1.5, palette::GOLD_DIM));
    theme::text_bold(
        painter,
        pos2(cx, cy),
        Align2::CENTER_CENTER,
        city.population,
        fs * 1.25,
        Color32::WHITE,
    );
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
        painter.rect_filled(tag, fs * 0.52, Color32::from_rgb(12, 20, 34));
        painter.rect_stroke(tag, fs * 0.52, Stroke::new(1.2, palette::GOOD), StrokeKind::Inside);
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
        painter.circle_filled(pc + vec2(0.0, 2.0), prod_r + 1.0, Color32::from_black_alpha(100));
        painter.circle_filled(pc, prod_r, Color32::from_rgb(12, 20, 34));
        theme::ring(
            painter,
            pc,
            prod_r - 2.5,
            done as f32 / cost as f32,
            3.0,
            Color32::from_rgb(240, 150, 60),
        );
        painter.circle_stroke(pc, prod_r, Stroke::new(1.5, palette::GOLD_DIM));
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
            painter.rect_filled(tag, fs * 0.5, Color32::from_rgb(12, 20, 34));
            painter.rect_stroke(tag, fs * 0.5, Stroke::new(1.2, Color32::from_rgb(240, 150, 60)), StrokeKind::Inside);
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
    let bar_h = (fs * 0.28).max(4.0);
    if city.loyalty < wayciv_core::loyalty::MAX_LOYALTY && game.loyalty_applies(city.id) {
        // Loyalty: purple, turning red when the city is close to defecting.
        let bar = Rect::from_min_size(pos2(bar_left, y), vec2(bar_w, bar_h));
        painter.rect_filled(bar, bar_h / 2.0, Color32::from_black_alpha(190));
        let color = if city.loyalty > 25 {
            Color32::from_rgb(176, 120, 236)
        } else {
            Color32::from_rgb(236, 70, 70)
        };
        painter.rect_filled(
            Rect::from_min_size(bar.min, vec2((bar_w * city.loyalty as f32 / 100.0).max(bar_h), bar_h)),
            bar_h / 2.0,
            color,
        );
        y += bar_h + 2.0;
    }
    if city.hp < 200 || city.wall_hp > 0 {
        let bar = Rect::from_min_size(pos2(bar_left, y), vec2(bar_w, bar_h));
        painter.rect_filled(bar, bar_h / 2.0, Color32::from_black_alpha(190));
        painter.rect_filled(
            Rect::from_min_size(bar.min, vec2((bar_w * city.hp as f32 / 200.0).max(bar_h), bar_h)),
            bar_h / 2.0,
            Color32::from_rgb(80, 210, 90),
        );
        if city.wall_hp > 0 {
            let walls = Rect::from_min_size(
                pos2(bar_left, y + bar_h + 1.0),
                vec2(
                    (bar_w * (city.wall_hp as f32 / game.max_walls(city.id).max(1) as f32).min(1.0)).max(2.0),
                    bar_h * 0.7,
                ),
            );
            painter.rect_filled(walls, 1.0, Color32::from_rgb(120, 170, 240));
        }
    }
}

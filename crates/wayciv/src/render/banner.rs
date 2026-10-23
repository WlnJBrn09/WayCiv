//! City banners: name, population, production, loyalty, and health.

use crate::palette::rgb;
use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind, pos2, vec2};

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
    let font = FontId::proportional((z * 0.42).clamp(10.0, 18.0));
    let name = if city.capital {
        format!("* {}", city.name)
    } else {
        city.name.clone()
    };
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
    painter.text(
        pop_rect.center(),
        Align2::CENTER_CENTER,
        city.population.to_string(),
        font.clone(),
        primary,
    );
    painter.galley(top + vec2(pop_w + 6.0, 3.0), galley, secondary);
    let mut y = banner.bottom() + 2.0;
    if mine && let Some(item) = &city.production {
        let cost = game.production_cost(city.owner, item).max(1);
        let done = city
            .progress
            .get(&wayciv_core::Game::production_key(item))
            .copied()
            .unwrap_or(0);
        let bar = Rect::from_min_size(pos2(banner.left(), y), vec2(w, 4.0));
        painter.rect_filled(bar, 1.0, Color32::from_gray(30));
        painter.rect_filled(
            Rect::from_min_size(bar.min, vec2(w * (done as f32 / cost as f32).min(1.0), 4.0)),
            1.0,
            Color32::from_rgb(240, 150, 60),
        );
        y += 6.0;
    }
    if city.loyalty < wayciv_core::loyalty::MAX_LOYALTY && game.loyalty_applies(city.id) {
        // Loyalty: purple, turning red when the city is close to defecting.
        let bar = Rect::from_min_size(pos2(banner.left(), y), vec2(w, 4.0));
        painter.rect_filled(bar, 1.0, Color32::from_gray(30));
        let color = if city.loyalty > 25 {
            Color32::from_rgb(170, 110, 230)
        } else {
            Color32::from_rgb(230, 70, 70)
        };
        painter.rect_filled(
            Rect::from_min_size(bar.min, vec2(w * city.loyalty as f32 / 100.0, 4.0)),
            1.0,
            color,
        );
        y += 6.0;
    }
    if city.hp < 200 || city.wall_hp > 0 {
        let bar = Rect::from_min_size(pos2(banner.left(), y), vec2(w, 4.0));
        painter.rect_filled(bar, 1.0, Color32::from_gray(30));
        painter.rect_filled(
            Rect::from_min_size(bar.min, vec2(w * city.hp as f32 / 200.0, 4.0)),
            1.0,
            Color32::from_rgb(80, 210, 90),
        );
        if city.wall_hp > 0 {
            let walls = Rect::from_min_size(
                pos2(banner.left(), y + 5.0),
                vec2(
                    w * (city.wall_hp as f32 / game.max_walls(city.id).max(1) as f32).min(1.0),
                    3.0,
                ),
            );
            painter.rect_filled(walls, 1.0, Color32::from_rgb(120, 170, 240));
        }
    }
}

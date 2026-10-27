//! The big moments: the full-screen reveal when you complete a wonder, and
//! the end of the game (who won, graphs of every civilization over time,
//! and your timeline).

use super::glyph::Glyph;
use super::{anim, theme};
use crate::hud::Session;
use crate::palette::{self, GOLD, GOLD_DIM, TEXT, TEXT_DIM, rgb};
use egui::{Align2, Color32, Id, Pos2, Rect, RichText, Sense, Shape, Stroke, pos2, vec2};
use wayciv_core::state::Victory;

/// Shows the first pending wonder reveal. Returns true while one is shown.
pub fn wonder_reveal(s: &mut Session, ctx: &egui::Context) -> bool {
    let Some(&(wonder, city)) = s.game.reveals.first() else {
        return false;
    };
    let content = s.game.rules.content.clone();
    let w = &content.buildings[wonder];
    let city_name = s
        .game
        .cities
        .get(&city)
        .map(|c| c.name.clone())
        .unwrap_or_default();
    let year = crate::hud::year(&s.game);
    let tex = s.art.buildings.wonder_texture(ctx, &w.id);
    let key = Id::unique(("wonder_reveal", wonder));
    let t = ctx.input(|i| i.time) as f32;
    let mut close = false;
    let shown = egui::Modal::new(key)
        .backdrop_color(Color32::TRANSPARENT)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            theme::plate_ui(ui, 24, |ui| {
                ui.set_width(620.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("WONDER COMPLETED").font(palette::bold(15.0)).color(TEXT_DIM));
                    ui.label(theme::title(&w.name, 40.0));
                    let art_h = (ctx.content_rect().height() - 360.0).clamp(220.0, 420.0);
                    let (rect, _) = ui.allocate_exact_size(vec2(art_h * 0.8 + 80.0, art_h), Sense::hover());
                    let painter = ui.painter();
                    // A slowly turning gold sunburst behind the wonder.
                    let c = rect.center();
                    for k in 0..16 {
                        let a = t * 0.15 + k as f32 * std::f32::consts::TAU / 16.0;
                        let r = rect.height() * 0.62;
                        let spread = 0.09;
                        painter.add(Shape::convex_polygon(
                            vec![
                                c,
                                c + vec2((a - spread).cos(), (a - spread).sin()) * r,
                                c + vec2((a + spread).cos(), (a + spread).sin()) * r,
                            ],
                            palette::with_alpha(GOLD, 22),
                            Stroke::NONE,
                        ));
                    }
                    painter.circle_filled(c, rect.height() * 0.34, palette::with_alpha(GOLD, 26));
                    match &tex {
                        Some(tex) => {
                            let size = tex.size_vec2();
                            let scale = (rect.height() / size.y).min(rect.width() / size.x);
                            let r = Rect::from_center_size(c, size * scale);
                            painter.image(
                                tex.id(),
                                r,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        }
                        None => super::glyph::paint(painter, Glyph::Star, c, 40.0, GOLD),
                    }
                    ui.label(theme::bold(format!("{city_name} · {year}"), 18.0).color(TEXT));
                    ui.add_space(4.0);
                    ui.label(RichText::new(&w.description).size(15.0).color(TEXT_DIM));
                    ui.add_space(10.0);
                    if theme::pill_button(ui, Some(Glyph::Chevrons), "Continue", palette::TEAL, 180.0)
                        .clicked()
                    {
                        close = true;
                    }
                });
            });
        });
    anim::window_fx(ctx, key, shown.response.layer_id, shown.response.rect);
    if close || ctx.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Escape)) {
        s.game.reveals.remove(0);
    }
    ctx.request_repaint();
    true
}

const STATS: [&str; 5] = ["Score", "Science", "Culture", "Military", "Cities"];

/// The end of the game. Returns true while it is shown.
pub fn game_over(s: &mut Session, ctx: &egui::Context) -> bool {
    let Some((winner, victory)) = s.game.winner.clone() else {
        return false;
    };
    if s.game_over_seen {
        return false;
    }
    let me = s.me();
    let won = winner == me;
    let kind = match victory {
        Victory::Science => "Science",
        Victory::Culture => "Culture",
        Victory::Domination => "Domination",
        Victory::Religious => "Religious",
        Victory::Score => "Score",
    };
    let civ = s.game.players[winner].civ_name.clone();
    let leader = s.game.players[winner].name.clone();
    let year = crate::hud::year(&s.game);
    let key = Id::unique("game_over");
    let shown = egui::Modal::new(key)
        .backdrop_color(Color32::TRANSPARENT)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            theme::plate_ui(ui, 24, |ui| {
                let width = (ctx.content_rect().width() - 80.0).min(1100.0);
                ui.set_width(width);
                ui.vertical_centered(|ui| {
                    ui.label(theme::title(if won { "Victory!" } else { "Defeat" }, 48.0));
                    let line = if won {
                        format!("{civ} has won a {kind} Victory in {year}. History will remember {leader}.")
                    } else {
                        format!("{leader} of {civ} has won a {kind} Victory in {year}.")
                    };
                    ui.label(RichText::new(line).size(18.0).color(TEXT));
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    for (i, name) in STATS.iter().enumerate() {
                        if super::cards::choice(ui, name, GOLD, s.graph_stat == i) {
                            s.graph_stat = i;
                        }
                    }
                });
                let height = (ctx.content_rect().height() - 380.0).clamp(200.0, 420.0);
                let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
                graph(s, ui, rect);
                legend(s, ui);
                ui.add_space(6.0);
                let p = &s.game.players[me];
                let moments = p.era.moments.iter().filter(|m| m.score > 0).count();
                let ages: Vec<String> = p
                    .era
                    .past
                    .iter()
                    .map(|(era, _, age)| format!("{}: {}", s.game.era_name(*era), age.name()))
                    .collect();
                ui.label(
                    RichText::new(format!(
                        "Your story: {moments} historic moments, {} policy cards kept, {} great people. Ages: {}.",
                        p.cards.count(),
                        p.great_people_earned,
                        if ages.is_empty() { "-".into() } else { ages.join(", ") }
                    ))
                    .color(TEXT_DIM),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    if theme::pill_button(ui, Some(Glyph::Hex), "Look at the World", palette::TEAL, 200.0)
                        .clicked()
                    {
                        s.game_over_seen = true;
                    }
                    if theme::pill_button(ui, Some(Glyph::Chevrons), "Main Menu", GOLD, 160.0).clicked() {
                        s.quit_to_menu = true;
                    }
                });
            });
        });
    anim::window_fx(ctx, key, shown.response.layer_id, shown.response.rect);
    true
}

/// One line per major civilization (met ones, and yourself) for the chosen
/// statistic over the whole game.
fn graph(s: &Session, ui: &egui::Ui, rect: Rect) {
    let painter = ui.painter();
    painter.rect_filled(rect, 8.0, Color32::from_black_alpha(90));
    let history = &s.game.history;
    if history.len() < 2 {
        theme::text(painter, rect.center(), Align2::CENTER_CENTER, "Not enough history yet.", 15.0, TEXT_DIM);
        return;
    }
    let me = s.me();
    let value = |st: &wayciv_core::history::PlayerStats| match s.graph_stat {
        0 => st.score,
        1 => st.science,
        2 => st.culture,
        3 => st.military,
        _ => st.cities,
    } as f32;
    let shown: Vec<usize> = (0..s.game.players.len())
        .filter(|&p| s.game.players[p].is_major() && (p == me || s.game.players[me].relations[p].met))
        .collect();
    let max = history
        .iter()
        .flat_map(|h| shown.iter().map(move |&p| h.players.get(p).map_or(0.0, value)))
        .fold(1.0f32, f32::max);
    let inner = rect.shrink2(vec2(44.0, 16.0));
    let n = history.len() as f32 - 1.0;
    let at = |i: usize, v: f32| -> Pos2 {
        pos2(
            inner.left() + inner.width() * i as f32 / n,
            inner.bottom() - inner.height() * v / max,
        )
    };
    // Grid lines and labels.
    for k in 0..=4 {
        let v = max * k as f32 / 4.0;
        let y = inner.bottom() - inner.height() * k as f32 / 4.0;
        painter.line_segment([pos2(inner.left(), y), pos2(inner.right(), y)], Stroke::new(1.0, palette::with_alpha(GOLD_DIM, 50)));
        theme::text(painter, pos2(inner.left() - 6.0, y), Align2::RIGHT_CENTER, format!("{v:.0}"), 12.0, TEXT_DIM);
    }
    let first = history.first().map_or(1, |h| h.turn);
    let last = history.last().map_or(1, |h| h.turn);
    theme::text(painter, pos2(inner.left(), inner.bottom() + 9.0), Align2::LEFT_CENTER, format!("Turn {first}"), 12.0, TEXT_DIM);
    theme::text(painter, pos2(inner.right(), inner.bottom() + 9.0), Align2::RIGHT_CENTER, format!("Turn {last}"), 12.0, TEXT_DIM);
    // At most ~400 points per line.
    let step = (history.len() / 400).max(1);
    for &p in &shown {
        let color = readable(rgb(s.game.players[p].primary));
        let points: Vec<Pos2> = history
            .iter()
            .enumerate()
            .step_by(step)
            .filter(|(_, h)| h.players.get(p).is_some_and(|x| x.alive))
            .map(|(i, h)| at(i, value(&h.players[p])))
            .collect();
        let width = if p == me { 3.0 } else { 1.8 };
        painter.add(Shape::line(points, Stroke::new(width, color)));
    }
}

/// Lifts dark civilization colors so their lines show on the dark graph.
fn readable(c: Color32) -> Color32 {
    let luma = 0.3 * c.r() as f32 + 0.59 * c.g() as f32 + 0.11 * c.b() as f32;
    if luma >= 110.0 {
        return c;
    }
    let t = (110.0 - luma) / 110.0 * 0.7;
    let mix = |v: u8| (v as f32 + (255.0 - v as f32) * t) as u8;
    Color32::from_rgb(mix(c.r()), mix(c.g()), mix(c.b()))
}

fn legend(s: &Session, ui: &mut egui::Ui) {
    let me = s.me();
    ui.horizontal_wrapped(|ui| {
        for p in s.game.players.iter().filter(|p| p.is_major() && (p.id == me || s.game.players[me].relations[p.id].met)) {
            let (r, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 5.0, readable(rgb(p.primary)));
            let mut text = RichText::new(if p.id == me { format!("{} (you)", p.civ_name) } else { p.civ_name.clone() });
            if !p.alive {
                text = text.strikethrough().color(TEXT_DIM);
            } else {
                text = text.color(TEXT);
            }
            ui.label(text);
            ui.add_space(8.0);
        }
    });
}

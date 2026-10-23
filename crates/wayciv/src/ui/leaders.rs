//! The leader ribbon (top right) and the Leaders panel (L).

use super::{MARGIN, TOP_BAR, theme};
use crate::hud::Session;
use crate::palette::{self, BAD, GOLD, GOLD_DIM, GOOD, NAVY, NAVY_RAISED, TEAL, TEXT, TEXT_DIM};
use crate::portraits::Portraits;
use crate::screens::Panel;
use egui::{Align2, Color32, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use wayciv_core::PlayerId;

/// How you stand with another leader.
pub struct Relationship {
    pub status: &'static str,
    pub status_color: Color32,
    pub mood: &'static str,
    pub mood_color: Color32,
    /// Their opinion of you, -100..100.
    pub opinion: i32,
    pub detail: String,
}

pub fn relationship(s: &Session, other: PlayerId) -> Relationship {
    let me = s.me();
    let r = &s.game.players[me].relations[other];
    let theirs = &s.game.players[other].relations[me];
    let (status, status_color, detail) = if r.at_war {
        (
            "At war",
            BAD,
            format!("At war for {} turns.", s.game.turn.saturating_sub(r.since)),
        )
    } else if r.friendship {
        ("Friends", GOOD, "You have declared friendship.".to_string())
    } else if r.denounced || theirs.denounced {
        (
            "Denounced",
            Color32::from_rgb(240, 150, 60),
            if r.denounced {
                "You denounced them.".to_string()
            } else {
                "They denounced you.".to_string()
            },
        )
    } else {
        (
            "At peace",
            TEAL,
            format!("At peace for {} turns.", s.game.turn.saturating_sub(r.since)),
        )
    };
    let opinion = s.game.opinion(other, me);
    let (mood, mood_color) = match opinion {
        o if o >= 40 => ("Very friendly", GOOD),
        o if o >= 15 => ("Friendly", Color32::from_rgb(160, 220, 110)),
        o if o > -15 => ("Neutral", TEXT_DIM),
        o if o > -40 => ("Unfriendly", Color32::from_rgb(240, 160, 70)),
        _ => ("Hostile", BAD),
    };
    Relationship {
        status,
        status_color,
        mood,
        mood_color,
        opinion,
        detail,
    }
}

/// Every major civilization you have met, in player order.
pub fn met(s: &Session) -> Vec<PlayerId> {
    let me = s.me();
    s.game
        .players
        .iter()
        .filter(|o| o.id != me && o.is_major() && s.game.players[me].relations[o.id].met)
        .map(|o| o.id)
        .collect()
}

/// Portraits of everyone you've met, top right. Click opens diplomacy.
pub fn ribbon(s: &mut Session, portraits: &mut Portraits, ctx: &egui::Context) {
    let list = met(s);
    if list.is_empty() {
        return;
    }
    let d = 56.0;
    let gap = 8.0;
    let size = vec2(
        list.len() as f32 * (d + gap) + gap + 6.0,
        d + 20.0,
    );
    egui::Area::new(egui::Id::unique("leader_ribbon"))
        .anchor(Align2::RIGHT_TOP, vec2(-MARGIN, TOP_BAR + MARGIN))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            let painter = ui.painter().clone();
            theme::plate(&painter, rect, 14.0);
            let mut x = rect.left() + gap + 3.0 + d / 2.0;
            for other in list.iter().rev() {
                let other = *other;
                let c = pos2(x, rect.center().y);
                let (leader_id, leader_name, colors, civ, alive) = {
                    let o = &s.game.players[other];
                    let leader = &s.game.rules.content.leaders[o.leader];
                    (
                        leader.id.clone(),
                        o.name.clone(),
                        (o.primary, o.secondary),
                        o.civ_name.clone(),
                        o.alive,
                    )
                };
                let rel = relationship(s, other);
                let hit = Rect::from_center_size(c, vec2(d, d));
                let response = ui.interact(hit, ui.scope_id().with(("ribbon", other)), Sense::click());
                theme::portrait_disc(&painter, portraits, c, d / 2.0 - 3.0, &leader_id, &leader_name, colors);
                if !alive {
                    painter.circle_filled(c, d / 2.0 - 3.0, Color32::from_black_alpha(170));
                }
                painter.circle_stroke(
                    c,
                    d / 2.0 - 1.0,
                    Stroke::new(
                        if response.hovered() { 4.0 } else { 3.0 },
                        if response.hovered() { GOLD } else { rel.status_color },
                    ),
                );
                painter.circle_filled(c + vec2(d * 0.34, d * 0.34), 7.0, NAVY);
                painter.circle_filled(c + vec2(d * 0.34, d * 0.34), 5.0, rel.status_color);
                let tip = format!(
                    "{leader_name} of {civ}\n{} - {}\n{}\nClick for diplomacy.",
                    rel.status, rel.mood, rel.detail
                );
                if response.on_hover_text(tip).clicked() {
                    s.windows.open(Panel::Diplomacy(other));
                }
                x += d + gap;
            }
        });
}

/// The Leaders panel: every met civ with portrait, status and opinion.
pub fn panel(s: &mut Session, portraits: &mut Portraits, ui: &mut Ui) {
    let list = met(s);
    let me = s.me();
    if list.is_empty() {
        ui.add_space(20.0);
        ui.label(
            egui::RichText::new("You haven't met any other leaders yet. Explore to find them.")
                .size(17.0)
                .color(TEXT_DIM),
        );
    }
    egui::ScrollArea::vertical().max_height(460.0).show(ui, |ui| {
        for other in list {
            row(s, portraits, ui, other);
            ui.add_space(4.0);
        }
    });
    let unmet = s
        .game
        .players
        .iter()
        .filter(|o| {
            o.id != me && o.is_major() && o.alive && !s.game.players[me].relations[o.id].met
        })
        .count();
    if unmet > 0 {
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(format!(
                "{unmet} other civilization{} not yet met.",
                if unmet == 1 { "" } else { "s" }
            ))
            .color(TEXT_DIM),
        );
    }
}

fn row(s: &mut Session, portraits: &mut Portraits, ui: &mut Ui, other: PlayerId) {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width().max(620.0), 92.0), Sense::hover());
    let painter = ui.painter().clone();
    let o = s.game.players[other].clone();
    let leader = s.game.rules.content.leaders[o.leader].clone();
    let rel = relationship(s, other);
    painter.rect_filled(rect, 12.0, NAVY_RAISED);
    painter.rect_stroke(rect, 12.0, Stroke::new(1.5, palette::with_alpha(GOLD_DIM, 190)), StrokeKind::Inside);
    let c = pos2(rect.left() + 52.0, rect.center().y);
    theme::portrait_disc(&painter, portraits, c, 36.0, &leader.id, &leader.name, (o.primary, o.secondary));
    painter.circle_stroke(c, 38.0, Stroke::new(3.5, rel.status_color));
    // Civ color swatch.
    painter.circle_filled(c + vec2(26.0, 28.0), 9.0, NAVY);
    painter.circle_filled(c + vec2(26.0, 28.0), 7.0, palette::rgb(o.primary));

    let x = rect.left() + 104.0;
    theme::text_bold(&painter, pos2(x, rect.top() + 24.0), Align2::LEFT_CENTER, &leader.name, 20.0, GOLD);
    theme::text(&painter, pos2(x, rect.top() + 46.0), Align2::LEFT_CENTER, &o.civ_name, 15.0, TEXT);

    // Status chip.
    let chip_w = 92.0;
    let chip = Rect::from_min_size(pos2(x, rect.top() + 58.0), vec2(chip_w, 24.0));
    painter.rect_filled(chip, 12.0, rel.status_color.gamma_multiply(0.35));
    painter.rect_stroke(chip, 12.0, Stroke::new(1.5, rel.status_color), StrokeKind::Inside);
    theme::text_bold(&painter, chip.center(), Align2::CENTER_CENTER, rel.status, 14.0, TEXT);

    // Opinion bar: -100 .. 100.
    let bx = rect.right() - 12.0 - 104.0 - 24.0 - 190.0;
    let bar = Rect::from_min_size(pos2(bx, rect.top() + 52.0), vec2(190.0, 10.0));
    painter.rect_filled(bar, 5.0, Color32::from_white_alpha(24));
    let mid = bar.center().x;
    let end = mid + bar.width() / 2.0 * rel.opinion as f32 / 100.0;
    let (l, r) = if end >= mid { (mid, end) } else { (end, mid) };
    painter.rect_filled(
        Rect::from_min_max(pos2(l, bar.top()), pos2(r.max(l + 2.0), bar.bottom())),
        3.0,
        rel.mood_color,
    );
    painter.line_segment([pos2(mid, bar.top() - 3.0), pos2(mid, bar.bottom() + 3.0)], Stroke::new(1.5, TEXT_DIM));
    theme::text(&painter, pos2(bx, rect.top() + 34.0), Align2::LEFT_CENTER, "Their opinion of you", 13.0, TEXT_DIM);
    theme::text_bold(&painter, pos2(bx, rect.top() + 76.0), Align2::LEFT_CENTER, rel.mood, 15.0, rel.mood_color);
    theme::text(&painter, pos2(bx + 190.0, rect.top() + 76.0), Align2::RIGHT_CENTER, format!("{:+}", rel.opinion), 13.0, TEXT_DIM);

    // Score and cities.
    let cities = s.game.player_cities(other).count();
    let score = s.game.score(other);
    theme::text(
        &painter,
        pos2(bx, rect.top() + 14.0),
        Align2::LEFT_CENTER,
        format!("Score {score}   Cities {cities}"),
        13.0,
        TEXT_DIM,
    );

    // Diplomacy button.
    let btn = Rect::from_center_size(pos2(rect.right() - 62.0, rect.center().y), vec2(104.0, 36.0));
    let br = ui.interact(btn, ui.scope_id().with(("dip", other)), Sense::click());
    let hovered = br.hovered();
    painter.rect_filled(btn, 18.0, if hovered { palette::NAVY_HOVER } else { NAVY });
    painter.rect_stroke(btn, 18.0, Stroke::new(2.0, if hovered { GOLD } else { GOLD_DIM }), StrokeKind::Inside);
    theme::text_bold(&painter, btn.center(), Align2::CENTER_CENTER, "Diplomacy", 15.0, TEXT);
    if br.on_hover_text(rel.detail.clone()).clicked() {
        s.windows.open(Panel::Diplomacy(other));
    }
    let _ = response;
}

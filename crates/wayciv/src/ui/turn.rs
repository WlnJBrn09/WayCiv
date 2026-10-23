//! The big circular Next Turn / action button, bottom right.

use super::glyph::Glyph;
use super::{MARGIN, glyph, theme};
use crate::hud::Session;
use crate::palette::{self, GOLD, GOLD_DIM, NAVY, TEXT};
use egui::{Align2, Color32, FontId, Sense, Stroke, pos2, vec2};
use wayciv_core::turn::Blocker;

/// Diameter of the button.
pub const DIAMETER: f32 = 124.0;

/// Draws the button and returns the left edge of its footprint.
pub fn show(s: &mut Session, ctx: &egui::Context) -> f32 {
    let me = s.me();
    let blockers = s.game.blockers(me);
    let screen = ctx.content_rect();
    let half = DIAMETER / 2.0;
    let center = pos2(screen.right() - MARGIN - DIAMETER / 2.0, screen.bottom() - MARGIN - DIAMETER / 2.0);
    let origin = center - vec2(half, half);
    let (label, glyph, color) = match blockers.first() {
        Some(b) => (
            s.game.blocker_text(b),
            match b {
                Blocker::ChooseResearch => Glyph::Science,
                Blocker::ChooseCivic => Glyph::Culture,
                Blocker::ChoosePantheon => Glyph::Faith,
                Blocker::ChooseAiResearch => Glyph::Chip,
                Blocker::ChooseProduction(_) => Glyph::Production,
                Blocker::UnitNeedsOrders(_) => Glyph::Shield,
            },
            Color32::from_rgb(46, 86, 152),
        ),
        None => ("Next Turn".to_string(), Glyph::Chevrons, Color32::from_rgb(34, 140, 92)),
    };
    let winner = s.game.winner.as_ref().map(|(w, v)| {
        format!("{} won a {v:?} victory!", s.game.players[*w].civ_name)
    });
    let mut end_anyway = false;
    let mut next_unit = false;
    let mut clicked = false;
    egui::Area::new(egui::Id::unique("turn_button"))
        .fixed_pos(origin)
        .constrain(false)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(half * 2.0, half * 2.0), Sense::hover());
            let painter = ui.painter().clone();
            let c = rect.center();
            let response = ui.interact(
                egui::Rect::from_center_size(c, vec2(DIAMETER, DIAMETER)),
                ui.scope_id().with("big"),
                Sense::click(),
            );
            let r = DIAMETER / 2.0;
            let hovered = response.hovered();
            painter.circle_filled(c + vec2(0.0, 4.0), r + 2.0, Color32::from_black_alpha(110));
            painter.circle_filled(c, r, NAVY);
            painter.circle_stroke(c, r - 1.0, Stroke::new(3.0, if hovered { GOLD } else { GOLD_DIM }));
            let face = if hovered { color.gamma_multiply(1.18) } else { color };
            painter.circle_filled(c, r - 7.0, face);
            // A soft highlight on the upper half.
            theme::arc(&painter, c, r - 12.0, 0.5, 3.0, Color32::from_white_alpha(46));
            painter.circle_stroke(c, r - 7.0, Stroke::new(1.5, palette::with_alpha(GOLD, 120)));
            glyph::paint(&painter, glyph, c - vec2(0.0, 33.0), 12.0, Color32::WHITE);
            let lines = wrap(&painter, &label, 19.0, DIAMETER * 0.74);
            let line_h = 21.0;
            let top = c.y - 10.0 - (lines.len() as f32 - 1.0) * line_h / 2.0 + 6.0;
            for (i, l) in lines.iter().enumerate() {
                theme::text_bold(&painter, pos2(c.x, top + i as f32 * line_h), Align2::CENTER_CENTER, l, 19.0, Color32::WHITE);
            }
            theme::keycap(&painter, c + vec2(0.0, r - 22.0), "Enter");
            let tip = match &winner {
                Some(w) => format!("{w}\n\n{label} (Enter)"),
                None => format!("{label} (Enter)"),
            };
            if response.on_hover_text(tip).clicked() {
                clicked = true;
            }

            // Satellites: next unit, and end the turn despite unit orders.
            let sat = |ui: &mut egui::Ui, dir: egui::Vec2, g: Glyph, key: Option<&str>, enabled: bool, tip: &str| {
                let sc = c + dir * (r + 6.0);
                let rect = egui::Rect::from_center_size(sc, vec2(38.0, 38.0));
                let resp = ui.interact(rect, ui.scope_id().with(tip), Sense::click());
                theme::paint_round_button(ui.painter(), rect, &resp, g, TEXT, key, enabled, false);
                resp.on_hover_text(tip).clicked() && enabled
            };
            let d = std::f32::consts::FRAC_1_SQRT_2;
            if sat(ui, vec2(-d, -d), Glyph::Boot, None, true, "Next unit (.)") {
                next_unit = true;
            }
            let anyway = !blockers.is_empty();
            if sat(
                ui,
                vec2(0.0 * d, -1.0),
                Glyph::Hourglass,
                None,
                anyway,
                "End turn anyway (skips remaining unit orders)",
            ) && anyway
            {
                end_anyway = true;
            }
        });
    if clicked {
        s.end_turn_clicked();
    }
    if next_unit {
        s.next_unit();
    }
    if end_anyway {
        let only_units = blockers.iter().all(|b| matches!(b, Blocker::UnitNeedsOrders(_)));
        if only_units {
            s.end_turn();
        } else {
            s.toast("Choose research, civics, and production first.");
        }
    }
    let _ = FontId::default();
    center.x - DIAMETER / 2.0
}

/// Greedy word wrap by measured width.
fn wrap(painter: &egui::Painter, text: &str, size: f32, width: f32) -> Vec<String> {
    let font = FontId::proportional(size);
    let w = |s: &str| painter.layout_no_wrap(s.to_string(), font.clone(), TEXT).size().x;
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(last) if w(&format!("{last} {word}")) <= width => {
                last.push(' ');
                last.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

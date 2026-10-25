//! The big circular Next Turn / action button, bottom right. It breathes a
//! soft green glow when the turn is ready to end, sends gold ripples when
//! something needs a decision, and a banner announces each new turn.

use super::glyph::Glyph;
use super::{MARGIN, TOP_BAR, anim, glyph, theme};
use crate::hud::Session;
use crate::palette::{self, GOLD, GOLD_DIM, NAVY, TEXT};
use egui::{Align2, Color32, FontId, Id, LayerId, Order, Rect, Sense, Stroke, pos2, vec2};
use wayciv_core::turn::Blocker;

/// Diameter of the button.
pub const DIAMETER: f32 = 124.0;
/// How long the new-turn banner stays up.
const BANNER_SECS: f32 = 2.6;
const READY: Color32 = Color32::from_rgb(70, 214, 140);

/// Draws the button and returns the left edge of its footprint.
pub fn show(s: &mut Session, ctx: &egui::Context) -> f32 {
    let me = s.me();
    let blockers = s.game.blockers(me);
    let screen = ctx.content_rect();
    let half = DIAMETER / 2.0;
    let center = pos2(
        screen.right() - MARGIN - DIAMETER / 2.0,
        screen.bottom() - MARGIN - DIAMETER / 2.0,
    );
    let origin = center - vec2(half, half);
    let (label, glyph, color) = match blockers.first() {
        Some(b) => (
            s.game.blocker_text(b),
            match b {
                Blocker::ChooseResearch => Glyph::Science,
                Blocker::ChoosePantheon => Glyph::Faith,
                Blocker::ChooseAiResearch => Glyph::Chip,
                Blocker::ChooseProduction(_) => Glyph::Production,
                Blocker::UnitNeedsOrders(_) => Glyph::Shield,
            },
            Color32::from_rgb(46, 86, 152),
        ),
        None => (
            "Next Turn".to_string(),
            Glyph::Chevrons,
            Color32::from_rgb(34, 140, 92),
        ),
    };
    let winner = s
        .game
        .winner
        .as_ref()
        .map(|(w, v)| format!("{} won a {v:?} victory!", s.game.players[*w].civ_name));
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
            let ctx = ui.ctx().clone();
            let id = response.id;
            let hovered = response.hovered();
            let hot = anim::hover(&ctx, id.with("hot"), hovered, 0.15);
            let down = anim::hover(&ctx, id.with("down"), response.is_pointer_button_down_on(), 0.07);
            if response.clicked() {
                anim::trigger(&ctx, id.with("pop"));
            }
            // The face pops when it changes to a different task.
            if anim::changed(&ctx, id.with("label"), anim::hash(&label)) {
                anim::trigger(&ctx, id.with("swap"));
            }
            let swap = anim::progress(&ctx, id.with("swap"), 0.45).map_or(0.0, anim::bump);
            let r = DIAMETER / 2.0 * (1.0 + 0.035 * hot - 0.06 * down + 0.07 * swap);
            glows(&painter, &ctx, c, r, blockers.is_empty());
            painter.circle_filled(c + vec2(0.0, 4.0), r + 2.0, Color32::from_black_alpha(110));
            painter.circle_filled(c, r, NAVY);
            painter.circle_stroke(
                c,
                r - 1.0,
                Stroke::new(3.0, anim::mix(GOLD_DIM, GOLD, hot)),
            );
            let face = color.gamma_multiply(1.0 + 0.18 * hot);
            painter.circle_filled(c, r - 7.0, face);
            // A soft highlight on the upper half.
            theme::arc(
                &painter,
                c,
                r - 12.0,
                0.5,
                3.0,
                Color32::from_white_alpha(46),
            );
            painter.circle_stroke(c, r - 7.0, Stroke::new(1.5, palette::with_alpha(GOLD, 120)));
            // The glyph sits above the label; long labels get a smaller font
            // so the two never overlap.
            let mut size = 19.0;
            let mut lines = wrap(&painter, &label, size, DIAMETER * 0.74);
            if lines.len() > 2 {
                size = 16.0;
                lines = wrap(&painter, &label, size, DIAMETER * 0.8);
            }
            let line_h = size + 2.0;
            let glyph_h = 28.0;
            let top = c.y - 8.0 - (glyph_h + lines.len() as f32 * line_h) / 2.0;
            glyph::paint(&painter, glyph, pos2(c.x, top + 11.0), 11.0, Color32::WHITE);
            for (i, l) in lines.iter().enumerate() {
                let y = top + glyph_h + (i as f32 + 0.5) * line_h;
                theme::text_bold(
                    &painter,
                    pos2(c.x, y),
                    Align2::CENTER_CENTER,
                    l,
                    size,
                    Color32::WHITE,
                );
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
            let sat = |ui: &mut egui::Ui,
                       dir: egui::Vec2,
                       g: Glyph,
                       key: Option<&str>,
                       enabled: bool,
                       tip: &str| {
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
        let only_units = blockers
            .iter()
            .all(|b| matches!(b, Blocker::UnitNeedsOrders(_)));
        if only_units {
            s.end_turn();
        } else {
            s.toast("Choose research and production first.");
        }
    }
    banner(s, ctx);
    center.x - DIAMETER / 2.0
}

/// Glow behind the button: green breathing when the turn can end, gold
/// ripples when a decision is waiting, and a ring when a new turn begins.
fn glows(painter: &egui::Painter, ctx: &egui::Context, c: egui::Pos2, r: f32, ready: bool) {
    anim::ambient(ctx);
    let t = anim::now(ctx) as f32;
    if ready {
        let w = anim::wave(ctx, 2.8);
        for (grow, a) in [(18.0, 7.0), (12.0, 10.0), (6.0, 15.0)] {
            painter.circle_filled(c, r + grow * (0.8 + 0.4 * w), READY.gamma_multiply(a * (0.5 + w) / 255.0 * 2.0));
        }
        theme::glow_circle(painter, c, r, READY, 0.35 + 0.5 * w);
    } else {
        theme::glow_circle(painter, c, r, GOLD, 0.55);
        for k in 0..2 {
            let p = (t / 1.5 + k as f32 * 0.5).fract();
            let e = anim::ease_out(p);
            painter.circle_stroke(
                c,
                r + 2.0 + 26.0 * e,
                Stroke::new(1.0 + 3.0 * (1.0 - p), GOLD.gamma_multiply((1.0 - p).powf(1.5) * 0.85)),
            );
        }
    }
    if let Some(p) = anim::progress(ctx, flash_id(), 0.9) {
        let e = anim::ease_out(p);
        painter.circle_stroke(
            c,
            r + 4.0 + 46.0 * e,
            Stroke::new(4.0 * (1.0 - e) + 1.0, Color32::WHITE.gamma_multiply((1.0 - e) * 0.8)),
        );
    }
}

fn flash_id() -> Id {
    Id::unique("turn_flash")
}

/// "Turn 42 · 2350 BC", fading and sliding in at the top when a turn begins.
fn banner(s: &Session, ctx: &egui::Context) {
    let id = Id::unique("turn_banner");
    if anim::changed(ctx, id.with("turn"), s.game.turn as u64) {
        anim::trigger(ctx, id);
        anim::trigger(ctx, flash_id());
    }
    let Some(p) = anim::progress(ctx, id, 1.0) else {
        return;
    };
    let secs = p * BANNER_SECS;
    let a_in = anim::ease_out(secs / 0.3);
    let a_out = 1.0 - anim::ease_in_out((secs - (BANNER_SECS - 0.5)) / 0.5);
    let a = a_in * a_out;
    let dy = (1.0 - a_in) * -20.0 - (1.0 - a_out) * 10.0;
    let painter = ctx.layer_painter(LayerId::new(Order::Foreground, id.with("layer")));
    let ga = |c: Color32| c.gamma_multiply(a);
    let turn = painter.layout_no_wrap(format!("Turn {}", s.game.turn), FontId::proportional(32.0), ga(GOLD));
    let year = painter.layout_no_wrap(crate::hud::year(&s.game), FontId::proportional(22.0), ga(TEXT));
    let gap = 26.0;
    let w = turn.size().x + year.size().x + gap + 70.0;
    let screen = ctx.content_rect();
    let plate = Rect::from_center_size(
        pos2(screen.center().x, TOP_BAR + 100.0 + dy),
        vec2(w, 62.0),
    );
    painter.rect_filled(plate.translate(vec2(0.0, 4.0)), 31.0, Color32::from_black_alpha((90.0 * a) as u8));
    painter.rect_filled(plate, 31.0, ga(palette::with_alpha(NAVY, 235)));
    painter.rect_stroke(plate, 31.0, Stroke::new(2.0, ga(GOLD)), egui::StrokeKind::Inside);
    painter.rect_stroke(
        plate.shrink(4.0),
        27.0,
        Stroke::new(1.0, ga(palette::with_alpha(palette::TEAL, 90))),
        egui::StrokeKind::Inside,
    );
    // A glint sweeps across once.
    let sweep = anim::ease_in_out((secs - 0.15) / 0.8);
    if sweep > 0.0 && sweep < 1.0 {
        let x = anim::lerp(plate.left() - 20.0, plate.right() + 20.0, sweep);
        let clipped = painter.with_clip_rect(plate);
        for (half, alpha) in [(34.0, 12.0), (20.0, 18.0), (8.0, 28.0)] {
            clipped.rect_filled(
                Rect::from_min_size(pos2(x - half, plate.top()), vec2(half * 2.0, plate.height())),
                0.0,
                Color32::from_white_alpha((alpha * a) as u8),
            );
        }
    }
    // Rules reaching out to both sides, fading away.
    for side in [-1.0f32, 1.0] {
        let x0 = if side < 0.0 { plate.left() } else { plate.right() };
        let reach = 90.0 * anim::ease_out(secs / 0.6);
        for i in 0..18 {
            let (f0, f1) = (i as f32 / 18.0, (i + 1) as f32 / 18.0);
            painter.line_segment(
                [pos2(x0 + side * (8.0 + reach * f0), plate.center().y), pos2(x0 + side * (8.0 + reach * f1), plate.center().y)],
                Stroke::new(1.5, ga(GOLD).gamma_multiply(1.0 - f0)),
            );
        }
    }
    let y = plate.center().y;
    let x = plate.left() + 35.0;
    let ty = y - turn.size().y / 2.0;
    painter.galley(pos2(x + 0.7, ty), turn.clone(), ga(GOLD));
    painter.galley(pos2(x, ty), turn.clone(), ga(GOLD));
    let dot = pos2(x + turn.size().x + gap / 2.0, y);
    painter.circle_filled(dot, 3.0, ga(GOLD_DIM));
    painter.galley(pos2(x + turn.size().x + gap, y - year.size().y / 2.0), year, ga(TEXT));
}

/// Greedy word wrap by measured width.
fn wrap(painter: &egui::Painter, text: &str, size: f32, width: f32) -> Vec<String> {
    let font = FontId::proportional(size);
    let w = |s: &str| {
        painter
            .layout_no_wrap(s.to_string(), font.clone(), TEXT)
            .size()
            .x
    };
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

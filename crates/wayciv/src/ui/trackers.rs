//! The circular Research and AI trackers under the top bar. Their
//! progress rings ease to new values, and finishing an item plays a short
//! burst with its name.

use super::glyph::{self, Glyph};
use super::{MARGIN, TOP_BAR, anim, theme};
use crate::hud::Session;
use crate::palette::{self, BAD, COMPUTE, GOLD, GOLD_DIM, NAVY, TEXT, TEXT_DIM, YIELD_COLORS};
use crate::screens::Panel;
use egui::{Align2, Color32, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use wayciv_core::research::Tree;

const WIDTH: f32 = 290.0;
const HEIGHT: f32 = 84.0;
/// How long the completion burst lasts.
const BURST_SECS: f32 = 1.9;

struct Tracker {
    tree: Tree,
    panel: Panel,
    kind: &'static str,
    accent: Color32,
    glyph: Glyph,
    key: &'static str,
    choose: &'static str,
}

pub fn show(s: &mut Session, ctx: &egui::Context) {
    let me = s.me();
    let mut trackers = vec![
        Tracker {
            tree: Tree::Tech,
            panel: Panel::Tech,
            kind: "tech",
            accent: YIELD_COLORS[3],
            glyph: Glyph::Science,
            key: "S",
            choose: "Choose Research",
        },
    ];
    if s.game.ai_tree_unlocked(me) {
        trackers.push(Tracker {
            tree: Tree::Ai,
            panel: Panel::Ai,
            kind: "ai",
            accent: COMPUTE,
            glyph: Glyph::Chip,
            key: "A",
            choose: "Choose AI Research",
        });
    }
    egui::Area::new(egui::Id::unique("trackers"))
        .fixed_pos(pos2(MARGIN, TOP_BAR + MARGIN))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for t in &trackers {
                tracker(s, ui, t);
            }
        });
}

fn tracker(s: &mut Session, ui: &mut egui::Ui, t: &Tracker) {
    let me = s.me();
    let (rect, response) = ui.allocate_exact_size(vec2(WIDTH, HEIGHT), Sense::click());
    let painter = ui.painter().clone();
    let c = pos2(rect.left() + 40.0, rect.center().y);
    let ctx = ui.ctx().clone();
    let id = response.id;
    let hot = anim::hover(&ctx, id.with("hot"), response.hovered(), 0.14);
    let radius = 35.0 * (1.0 + 0.03 * hot);
    let current = s.game.current_research(me, t.tree);

    // Finishing the item that was being researched sets off the burst.
    let cur_id = id.with("current");
    let prev = ctx.data(|d| d.get_temp::<Option<usize>>(cur_id));
    ctx.data_mut(|d| d.insert_temp(cur_id, current));
    if let Some(Some(old)) = prev
        && current != Some(old)
        && s.game.known(me, t.tree, old)
    {
        let name = s.game.tree_items(t.tree)[old].name.clone();
        let at = anim::now(&ctx);
        ctx.data_mut(|d| d.insert_temp(id.with("burst"), (at, name)));
    }
    let burst = ctx
        .data(|d| d.get_temp::<(f64, String)>(id.with("burst")))
        .and_then(|(at, name)| {
            let p = (anim::now(&ctx) - at) as f32 / BURST_SECS;
            (0.0..1.0).contains(&p).then_some((p, name))
        });
    if burst.is_some() {
        ctx.request_repaint();
    }

    // Name plate behind the disc.
    let plate = Rect::from_min_max(
        pos2(c.x, c.y - 24.0),
        pos2(rect.right(), c.y + 24.0),
    );
    theme::glow(&painter, plate, 12.0, GOLD, hot * 0.6);
    painter.add(egui::Shadow {
        offset: [0, 3],
        blur: 10,
        spread: 0,
        color: Color32::from_black_alpha(130),
    }
    .as_shape(plate, egui::CornerRadius::same(12)));
    painter.add(theme::gradient_rect(
        plate,
        12.0,
        palette::PLATE_TOP,
        palette::PLATE_BOTTOM,
    ));
    painter.rect_stroke(
        plate,
        12.0,
        Stroke::new(anim::lerp(1.5, 2.0, hot), anim::mix(GOLD_DIM, GOLD, hot)),
        StrokeKind::Inside,
    );
    theme::disc(&painter, c, radius, NAVY, anim::mix(GOLD_DIM, GOLD, hot));

    let tx = c.x + radius + 10.0;
    let mut tip = String::new();
    match current {
        Some(i) => {
            let item = s.game.tree_items(t.tree)[i].clone();
            let cost = s.game.research_cost(t.tree, i).max(1);
            let done = s.game.progress(me, t.tree, i);
            let frac = done as f32 / cost as f32;
            let income = s.game.income(me);
            let rate = match t.tree {
                Tree::Tech => income.yields[3],
                Tree::Ai => s.game.compute(me),
            };
            let turns = if rate > 0.0 {
                let left = (cost - done).max(0) as f32;
                let n = (left / rate).ceil() as i32;
                format!("{n} turn{}", if n == 1 { "" } else { "s" })
            } else {
                "no Compute".to_string()
            };
            // Rings ease to their new value; a new item starts in place.
            let shown = anim::smooth(&ctx, id.with(("ring", i)), frac, 7.0);
            theme::ring(&painter, c, radius - 3.0, shown, 5.0, t.accent);
            if (shown - frac).abs() > 0.004 && shown > 0.01 {
                // A bright dot rides the leading edge while it moves.
                let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * shown;
                let tip = c + vec2(a.cos(), a.sin()) * (radius - 3.0);
                painter.circle_filled(tip, 5.0, t.accent.gamma_multiply(0.35));
                painter.circle_filled(tip, 2.6, Color32::WHITE);
            }
            draw_icon(s, &painter, ui.ctx(), t, &item.id, c);
            let name = fit(&painter, &item.name, 17.0, plate.right() - tx - 34.0);
            theme::text_bold(&painter, pos2(tx, c.y - 10.0), Align2::LEFT_CENTER, name, 17.0, TEXT);
            theme::text(&painter, pos2(tx, c.y + 11.0), Align2::LEFT_CENTER, &turns, 15.0, t.accent);
            tip = format!(
                "{}\n{done}/{cost}, +{rate:.1} per turn\nClick or press {} to change.",
                item.name, t.key
            );
        }
        None => {
            if burst.is_some() {
                theme::ring(&painter, c, radius - 3.0, 1.0, 5.0, t.accent);
            } else {
                // Nothing chosen: the red ring pulses to ask for a pick.
                anim::ambient(&ctx);
                let w = anim::wave(&ctx, 1.4);
                theme::ring(&painter, c, radius - 3.0, 1.0, 5.0, BAD.gamma_multiply(0.55 + 0.45 * w));
                theme::glow_circle(&painter, c, radius, BAD, 0.5 * w);
            }
            glyph::paint(&painter, t.glyph, c, 16.0, t.accent.gamma_multiply(0.6));
            theme::text_bold(
                &painter,
                pos2(tx, c.y - 10.0),
                Align2::LEFT_CENTER,
                t.choose,
                16.0,
                GOLD,
            );
            theme::text(
                &painter,
                pos2(tx, c.y + 11.0),
                Align2::LEFT_CENTER,
                "Click to pick",
                14.0,
                TEXT_DIM,
            );
        }
    }
    theme::keycap(&painter, pos2(plate.right() - 20.0, c.y - 10.0), t.key);
    if let Some((p, name)) = burst {
        celebrate(&painter, c, radius, plate, p, t.accent, &name);
    }
    if response.clicked() {
        s.windows.toggle(t.panel);
    }
    if !tip.is_empty() {
        response.on_hover_text(tip);
    }
}

/// The completion burst: a flash, spreading rings, sparks, and the name of
/// what was finished sliding over the plate.
fn celebrate(painter: &egui::Painter, c: egui::Pos2, radius: f32, plate: Rect, p: f32, accent: Color32, name: &str) {
    let e = anim::ease_out(p);
    let flash = (1.0 - p * 3.5).max(0.0);
    painter.circle_filled(c, radius - 2.0, Color32::WHITE.gamma_multiply(flash * 0.7));
    for k in 0..2 {
        let q = (p * 1.4 - k as f32 * 0.22).clamp(0.0, 1.0);
        if q > 0.0 && q < 1.0 {
            let e = anim::ease_out(q);
            painter.circle_stroke(
                c,
                radius + 40.0 * e,
                Stroke::new(4.0 * (1.0 - q) + 0.5, accent.gamma_multiply(1.0 - q)),
            );
        }
    }
    for i in 0..14 {
        let a = i as f32 / 14.0 * std::f32::consts::TAU + 0.2;
        let speed = 0.65 + 0.35 * ((i * 7) % 5) as f32 / 4.0;
        let d = vec2(a.cos(), a.sin());
        let dist = radius + 2.0 + 46.0 * speed * e;
        let fade = (1.0 - p).powf(1.3);
        painter.line_segment(
            [c + d * (dist - 7.0 * (1.0 - p)), c + d * dist],
            Stroke::new(2.4 * (1.0 - p) + 0.6, anim::mix(accent, Color32::WHITE, 0.5).gamma_multiply(fade)),
        );
        painter.circle_filled(c + d * dist, 2.6 * (1.0 - p) + 0.4, Color32::WHITE.gamma_multiply(fade));
    }
    let secs = p * BURST_SECS;
    let a = anim::ease_out(secs / 0.2) * (1.0 - anim::ease_in_out((secs - (BURST_SECS - 0.5)) / 0.5));
    if a > 0.0 {
        let mut plate = plate.translate(vec2(0.0, (1.0 - anim::ease_out(secs / 0.25)) * 10.0));
        plate.min.x = c.x + radius - 6.0;
        painter.rect_filled(plate, 12.0, palette::with_alpha(NAVY, 245).gamma_multiply(a));
        painter.rect_stroke(plate, 12.0, Stroke::new(2.0, GOLD.gamma_multiply(a)), StrokeKind::Inside);
        let mid = plate.center();
        let name = fit(painter, name, 17.0, plate.width() - 24.0);
        theme::text_bold(painter, mid + vec2(0.0, -9.0), Align2::CENTER_CENTER, name, 17.0, TEXT.gamma_multiply(a));
        theme::text(painter, mid + vec2(0.0, 11.0), Align2::CENTER_CENTER, "Complete!", 14.5, accent.gamma_multiply(a));
    }
}

fn draw_icon(
    s: &mut Session,
    painter: &egui::Painter,
    ctx: &egui::Context,
    t: &Tracker,
    id: &str,
    c: egui::Pos2,
) {
    match s.icons.get(ctx, t.kind, id) {
        Some(tex) => theme::image_circle(
            painter,
            tex.id(),
            c,
            27.0,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        ),
        None => glyph::paint(painter, t.glyph, c, 17.0, t.accent),
    }
}

/// Truncates `text` with an ellipsis so it fits in `width`.
pub fn fit(painter: &egui::Painter, text: &str, size: f32, width: f32) -> String {
    let font = egui::FontId::proportional(size);
    let w = |s: &str| painter.layout_no_wrap(s.to_string(), font.clone(), TEXT).size().x;
    if w(text) <= width {
        return text.to_string();
    }
    let mut out: String = text.to_string();
    while out.chars().count() > 1 && w(&format!("{out}...")) > width {
        out.pop();
    }
    format!("{}...", out.trim_end())
}

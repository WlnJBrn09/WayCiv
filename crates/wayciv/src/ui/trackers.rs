//! The circular Research, Civic and AI trackers under the top bar.

use super::glyph::{self, Glyph};
use super::{MARGIN, TOP_BAR, theme};
use crate::hud::Session;
use crate::palette::{self, BAD, COMPUTE, GOLD, GOLD_DIM, NAVY, TEXT, TEXT_DIM, YIELD_COLORS};
use crate::screens::Panel;
use egui::{Align2, Color32, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use wayciv_core::research::Tree;

const WIDTH: f32 = 290.0;
const HEIGHT: f32 = 84.0;

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
        Tracker {
            tree: Tree::Civic,
            panel: Panel::Civic,
            kind: "civic",
            accent: YIELD_COLORS[4],
            glyph: Glyph::Culture,
            key: "C",
            choose: "Choose Civic",
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
    let radius = 35.0;
    let current = s.game.current_research(me, t.tree);

    // Name plate behind the disc.
    let plate = Rect::from_min_max(
        pos2(c.x, c.y - 24.0),
        pos2(rect.right(), c.y + 24.0),
    );
    painter.rect_filled(plate, 12.0, palette::with_alpha(NAVY, 238));
    painter.rect_stroke(
        plate,
        12.0,
        Stroke::new(
            if response.hovered() { 2.0 } else { 1.5 },
            if response.hovered() { GOLD } else { GOLD_DIM },
        ),
        StrokeKind::Inside,
    );
    theme::disc(&painter, c, radius, NAVY, GOLD_DIM);

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
                Tree::Civic => income.yields[4],
                Tree::Ai => s.game.compute(me),
            };
            let turns = if rate > 0.0 {
                let left = (cost - done).max(0) as f32;
                let n = (left / rate).ceil() as i32;
                format!("{n} turn{}", if n == 1 { "" } else { "s" })
            } else {
                "no Compute".to_string()
            };
            theme::ring(&painter, c, radius - 3.0, frac, 5.0, t.accent);
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
            theme::ring(&painter, c, radius - 3.0, 1.0, 5.0, BAD);
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
    if response.clicked() {
        s.windows.toggle(t.panel);
    }
    if !tip.is_empty() {
        response.on_hover_text(tip);
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

//! The top bar: yields with per-turn values, trade routes, total population,
//! turn, year and era, and the menu buttons.

use super::glyph::Glyph;
use super::{TOP_BAR, theme};
use crate::hud::Session;
use crate::palette::{
    self, BAD, GOLD, GOLD_DIM, GOOD, NAVY, NAVY_RAISED, TEXT, TEXT_DIM, YIELD_COLORS,
};
use crate::screens::Panel;
use egui::{Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

struct Chip {
    glyph: Glyph,
    color: Color32,
    main: String,
    sub: Option<(String, Color32)>,
    tip: String,
    /// Higher survives longer when the window is narrow.
    priority: u8,
    id: &'static str,
}

pub fn show(s: &mut Session, ui: &mut Ui) {
    egui::Panel::top("top_bar")
        .exact_size(TOP_BAR)
        .frame(egui::Frame::new().fill(palette::with_alpha(NAVY, 250)))
        .show(ui, |ui| bar(s, ui));
}

fn chips(s: &Session) -> Vec<Chip> {
    let me = s.me();
    let game = &s.game;
    let income = game.income(me);
    let p = &game.players[me];
    let y = income.yields;
    let routes = game.trade_routes_of(me).count();
    let capacity = game.trade_capacity(me);
    let population: i32 = game.player_cities(me).map(|c| c.population).sum();
    let net = income.net_gold;
    vec![
        Chip {
            glyph: Glyph::Science,
            color: YIELD_COLORS[3],
            main: format!("+{:.1}", y[3]),
            sub: None,
            tip: "Science per turn. Press S to choose research.".into(),
            priority: 8,
            id: "science",
        },
        Chip {
            glyph: Glyph::Culture,
            color: YIELD_COLORS[4],
            main: format!("+{:.1}", y[4]),
            sub: None,
            tip: "Culture per turn. Press C to choose a civic.".into(),
            priority: 7,
            id: "culture",
        },
        Chip {
            glyph: Glyph::Faith,
            color: YIELD_COLORS[5],
            main: format!("{}", p.faith),
            sub: Some((format!("+{:.0}", y[5]), TEXT_DIM)),
            tip: "Faith stored, and per turn. Press R for religion.".into(),
            priority: 5,
            id: "faith",
        },
        Chip {
            glyph: Glyph::Gold,
            color: YIELD_COLORS[2],
            main: format!("{}", p.gold),
            sub: Some((format!("{net:+}"), if net < 0 { BAD } else { GOOD })),
            tip: format!(
                "Gold stored, and net per turn after {} upkeep.",
                income.maintenance
            ),
            priority: 9,
            id: "gold",
        },
        Chip {
            glyph: Glyph::Tourism,
            color: Color32::from_rgb(236, 128, 170),
            main: format!("+{}", income.tourism),
            sub: None,
            tip: "Tourism per turn.".into(),
            priority: 2,
            id: "tourism",
        },
        Chip {
            glyph: Glyph::Trade,
            color: Color32::from_rgb(222, 184, 120),
            main: format!("{routes}/{capacity}"),
            sub: None,
            tip: "Trade routes in use / capacity. Build Traders (Foreign Trade civic); Commercial Hubs and Harbors add capacity.".into(),
            priority: 3,
            id: "trade",
        },
        Chip {
            glyph: Glyph::Population,
            color: Color32::from_rgb(236, 226, 200),
            main: format!("{population}"),
            sub: None,
            tip: String::new(),
            priority: 6,
            id: "population",
        },
    ]
}

fn bar(s: &mut Session, ui: &mut Ui) {
    let rect = ui.max_rect();
    let painter = ui.painter().clone();
    painter.line_segment(
        [rect.left_bottom() - vec2(0.0, 1.0), rect.right_bottom() - vec2(0.0, 1.0)],
        Stroke::new(2.0, GOLD_DIM),
    );
    painter.line_segment(
        [rect.left_bottom() - vec2(0.0, 4.0), rect.right_bottom() - vec2(0.0, 4.0)],
        Stroke::new(1.0, palette::with_alpha(palette::TEAL, 60)),
    );
    let cy = rect.center().y - 1.0;
    let me = s.me();

    // Right side: menu buttons, then the turn block.
    let unread = s.unread_notifications();
    let menu: [(Glyph, &str, &str, Option<Panel>, Color32); 8] = [
        (Glyph::Bell, "Notifications", "N", Some(Panel::Notifications), GOLD),
        (Glyph::Crown, "Leaders", "L", Some(Panel::Leaders), GOLD),
        (Glyph::Temple, "Government", "G", Some(Panel::Government), GOLD),
        (Glyph::Faith, "Religion", "R", Some(Panel::Religion), YIELD_COLORS[5]),
        (Glyph::Star, "Great People", "P", Some(Panel::GreatPeople), GOLD),
        (Glyph::City, "City-States", "E", Some(Panel::CityStates), GOLD),
        (Glyph::Cup, "Victory Progress", "V", Some(Panel::Victory), GOLD),
        (Glyph::Hex, "Map Key", "K", None, TEXT),
    ];
    let d = 38.0;
    let gap = 6.0;
    let menu_w = menu.len() as f32 * d + (menu.len() - 1) as f32 * gap;
    let mut x = rect.right() - 14.0 - menu_w;
    let menu_left = x;
    for (g, name, key, panel, color) in menu {
        let r = Rect::from_min_size(pos2(x, cy - d / 2.0), vec2(d, d));
        let response = ui.interact(r, ui.scope_id().with(("menu", name)), Sense::click());
        let active = match panel {
            Some(p) => s.windows.is_open(p),
            None => s.windows.legend,
        };
        theme::paint_round_button(&painter, r, &response, g, color, None, true, active);
        if panel == Some(Panel::Notifications) && unread > 0 {
            theme::badge(
                &painter,
                r.right_top() + vec2(-3.0, 4.0),
                &if unread > 99 { "99+".into() } else { unread.to_string() },
                BAD,
            );
        }
        let response = response.on_hover_text(format!("{name} ({key})"));
        if response.clicked() {
            match panel {
                Some(p) => s.windows.toggle(p),
                None => s.windows.legend = !s.windows.legend,
            }
        }
        x += d + gap;
    }

    // Turn, year, era.
    let game = &s.game;
    let era = game
        .rules
        .content
        .eras
        .get(game.era_of(me))
        .map(|e| e.name.clone())
        .unwrap_or_default();
    let year = crate::hud::year(game.turn);
    let turn_font = FontId::proportional(20.0);
    let turn_galley =
        painter.layout_no_wrap(format!("Turn {}", game.turn), turn_font, GOLD);
    let sub_text = |with_era: bool| {
        if with_era && !era.is_empty() {
            format!("{year}  -  {era}")
        } else {
            year.clone()
        }
    };
    let chips = chips(s);
    let total = |chips: &[Chip]| -> f32 { chips.iter().map(|c| chip_width(&painter, c)).sum::<f32>() + 8.0 * chips.len() as f32 };
    let mut with_era = true;
    let mut sub_galley = painter.layout_no_wrap(sub_text(true), FontId::proportional(13.0), TEXT_DIM);
    let mut block_w = turn_galley.size().x.max(sub_galley.size().x);
    let mut avail = menu_left - 24.0 - block_w - (rect.left() + 12.0);
    if avail < total(&chips) * 0.7 {
        with_era = false;
        sub_galley = painter.layout_no_wrap(sub_text(false), FontId::proportional(13.0), TEXT_DIM);
        block_w = turn_galley.size().x.max(sub_galley.size().x);
        avail = menu_left - 24.0 - block_w - (rect.left() + 12.0);
    }
    let _ = with_era;
    let block_x = menu_left - 18.0 - block_w;
    painter.galley(pos2(block_x + block_w - turn_galley.size().x, cy - 19.0), turn_galley, GOLD);
    painter.galley(pos2(block_x + block_w - sub_galley.size().x, cy + 3.0), sub_galley, TEXT_DIM);

    // Left side: yield chips, dropping the least important when narrow.
    let mut chips = chips;
    while total(&chips) > avail && chips.len() > 3 {
        let (i, _) = chips
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| c.priority)
            .unwrap();
        chips.remove(i);
    }
    let mut x = rect.left() + 12.0;
    for chip in &chips {
        let w = chip_width(&painter, chip);
        let r = Rect::from_min_size(pos2(x, cy - 18.0), vec2(w, 36.0));
        let response = ui.interact(r, ui.scope_id().with(("chip", chip.id)), Sense::hover());
        draw_chip(&painter, r, chip, response.hovered());
        if chip.id == "population" {
            response.on_hover_ui(|ui| population_tip(s, ui));
        } else {
            response.on_hover_text(&chip.tip);
        }
        x += w + 8.0;
    }
}

fn chip_width(painter: &egui::Painter, chip: &Chip) -> f32 {
    let main = painter.layout_no_wrap(chip.main.clone(), FontId::proportional(18.0), TEXT);
    let mut w = 40.0 + main.size().x + 12.0;
    if let Some((sub, _)) = &chip.sub {
        w += 5.0 + painter.layout_no_wrap(sub.clone(), FontId::proportional(14.0), TEXT).size().x;
    }
    w
}

fn draw_chip(painter: &egui::Painter, r: Rect, chip: &Chip, hovered: bool) {
    painter.rect_filled(r, 18.0, NAVY_RAISED);
    painter.rect_stroke(
        r,
        18.0,
        Stroke::new(1.5, if hovered { GOLD } else { palette::with_alpha(GOLD_DIM, 200) }),
        StrokeKind::Inside,
    );
    let ic = pos2(r.left() + 19.0, r.center().y);
    painter.circle_filled(ic, 14.0, Color32::from_black_alpha(90));
    super::glyph::paint(painter, chip.glyph, ic, 9.5, chip.color);
    let tx = r.left() + 40.0;
    let rect = theme::text_bold(
        painter,
        pos2(tx, r.center().y),
        Align2::LEFT_CENTER,
        &chip.main,
        18.0,
        TEXT,
    );
    if let Some((sub, color)) = &chip.sub {
        theme::text(
            painter,
            pos2(rect.right() + 5.0, r.center().y + 1.0),
            Align2::LEFT_CENTER,
            sub,
            14.0,
            *color,
        );
    }
}

fn population_tip(s: &Session, ui: &mut Ui) {
    let me = s.me();
    ui.label(
        egui::RichText::new("Population")
            .strong()
            .color(GOLD),
    );
    let mut total = 0;
    for c in s.game.player_cities(me) {
        total += c.population;
        let report = s.game.city_report(c.id);
        let grow = match report.turns_to_grow {
            Some(t) => format!("grows in {t}"),
            None if report.yields[0] - report.food_consumed < 0.0 => "starving".into(),
            None => "not growing".into(),
        };
        ui.label(format!(
            "{}{}: {} citizens ({grow})",
            if c.capital { "* " } else { "" },
            c.name,
            c.population
        ));
    }
    ui.label(format!("Total: {total} citizens in your cities."));
}

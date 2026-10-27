//! The top bar: yields with per-turn values, trade routes, total population,
//! turn, year and era, and the menu buttons.

use super::glyph::Glyph;
use super::{TOP_BAR, theme};
use crate::hud::Session;
use crate::palette::{
    self, BAD, GOLD, GOLD_DIM, GOLD_HI, GOOD, NAVY_RAISED, TEXT, TEXT_DIM, YIELD_COLORS,
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
        .frame(egui::Frame::NONE)
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
    let (dark, golden) = game.era_thresholds(me);
    let projected = game.projected_age(me);
    vec![
        Chip {
            glyph: Glyph::Hourglass,
            color: palette::age_color(p.era.age),
            main: format!("{}", p.era.score),
            sub: Some((format!("/{golden}"), palette::age_color(projected))),
            tip: format!(
                "Era score this era. You are in a {}. Dark Age below {dark}, Golden Age at {golden}: heading for a {}. Press T for the timeline.",
                p.era.age.name(),
                projected.name()
            ),
            priority: 4,
            id: "era",
        },
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
            tip: "Culture per turn. Culture grows your borders and draws tourists.".into(),
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
            tip: "Trade routes in use / capacity. Build Traders; the Palace, Commercial Hubs, Harbors, and some policies add capacity.".into(),
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
    painter.add(theme::gradient_rect(
        rect,
        0.0,
        Color32::from_rgb(27, 44, 70),
        Color32::from_rgb(8, 14, 26),
    ));
    // Beveled gold trim along the bottom edge.
    let edge = |dy: f32, width: f32, color: Color32| {
        painter.line_segment(
            [
                rect.left_bottom() - vec2(0.0, dy),
                rect.right_bottom() - vec2(0.0, dy),
            ],
            Stroke::new(width, color),
        );
    };
    edge(0.5, 1.0, Color32::from_black_alpha(230));
    edge(2.0, 2.0, GOLD_DIM);
    edge(3.8, 1.0, palette::with_alpha(GOLD_HI, 110));
    edge(6.5, 1.0, palette::with_alpha(palette::TEAL, 46));
    let cy = rect.center().y - 1.0;
    let me = s.me();

    // Right side: menu buttons, then the turn block.
    let unread = s.unread_notifications();
    let ai = s.game.ai_tree_unlocked(me);
    let _ = ai;
    let menu: [MenuItem; 9] = [
        MenuItem::new(Glyph::Bell, "Notifications", "Alerts", "N", Some(Panel::Notifications), GOLD, "What happened recently"),
        MenuItem::new(Glyph::Crown, "Leaders", "Leaders", "L", Some(Panel::Leaders), GOLD, "Other leaders, how they feel about you, diplomacy"),
        MenuItem::new(Glyph::Temple, "Government", "Policies", "G", Some(Panel::Government), GOLD, "Your government and policy cards"),
        MenuItem::new(Glyph::Faith, "Religion", "Religion", "R", Some(Panel::Religion), YIELD_COLORS[5], "Pantheon, founding a religion, the world's faiths"),
        MenuItem::new(Glyph::Star, "Great People", "Great People", "P", Some(Panel::GreatPeople), GOLD, "Who comes next and how close you are"),
        MenuItem::new(Glyph::City, "City-States", "City-States", "E", Some(Panel::CityStates), GOLD, "Send envoys for bonuses"),
        MenuItem::new(Glyph::Cup, "Victory Progress", "Victory", "V", Some(Panel::Victory), GOLD, "How close everyone is to winning"),
        MenuItem::new(Glyph::Hourglass, "Eras and Timeline", "Eras", "T", Some(Panel::Eras), palette::AGE_GOLDEN, "Your age, era score, dedications, and historic moments"),
        MenuItem::new(Glyph::Hex, "Map Key", "Map Key", "K", None, TEXT, "What the map colors and symbols mean"),
    ];
    let gap = 6.0;
    let width_of = |tier: Tier| -> f32 {
        menu.iter()
            .map(|m| m.width(&painter, tier))
            .sum::<f32>()
            + (menu.len() - 1) as f32 * gap
    };

    // Turn, year, era.
    let game = &s.game;
    let era = game
        .rules
        .content
        .eras
        .get(game.era_of(me))
        .map(|e| e.name.clone())
        .unwrap_or_default();
    let year = crate::hud::year(game);
    let turn_galley = painter.layout_no_wrap(format!("Turn {}", game.turn), palette::display(21.0), GOLD);
    let sub_text = |with_era: bool| {
        if with_era && !era.is_empty() {
            format!("{year}  -  {era}")
        } else {
            year.clone()
        }
    };
    let chips = chips(s);
    let total = |chips: &[Chip]| -> f32 {
        chips.iter().map(|c| chip_width(&painter, c)).sum::<f32>() + 8.0 * chips.len() as f32
    };
    // The most readable layout in which every yield chip still fits: labelled
    // buttons with their hotkeys, labelled buttons, then icons alone.
    let (mut tier, mut with_era) = (Tier::Icons, false);
    'pick: for t in [Tier::Labels, Tier::LabelsOnly, Tier::Captions, Tier::Icons] {
        for e in [true, false] {
            let sub = painter.layout_no_wrap(sub_text(e), FontId::proportional(13.0), TEXT_DIM);
            let block = turn_galley.size().x.max(sub.size().x);
            let need = 12.0 + total(&chips) + 24.0 + block + 26.0 + width_of(t) + 20.0;
            if need <= rect.width() {
                (tier, with_era) = (t, e);
                break 'pick;
            }
        }
    }
    let sub_galley = painter.layout_no_wrap(sub_text(with_era), FontId::proportional(13.0), TEXT_DIM);
    let block_w = turn_galley.size().x.max(sub_galley.size().x);

    let menu_w = width_of(tier);
    let menu_left = rect.right() - 20.0 - menu_w;
    let mut x = menu_left;
    if tier != Tier::Captions {
        // A dark tray the buttons sit in.
        let tray = Rect::from_min_size(
            pos2(menu_left - 8.0, cy - 23.0),
            vec2(menu_w + 16.0, 46.0),
        );
        painter.add(theme::gradient_rect(
            tray,
            23.0,
            Color32::from_rgb(5, 10, 20),
            Color32::from_rgb(14, 24, 40),
        ));
        painter.rect_stroke(
            tray,
            23.0,
            Stroke::new(1.0, palette::with_alpha(GOLD_DIM, 170)),
            StrokeKind::Inside,
        );
    }
    for m in menu {
        let w = m.width(&painter, tier);
        let h = if tier == Tier::Captions { 46.0 } else { 36.0 };
        let r = Rect::from_min_size(pos2(x, cy - h / 2.0 + if h > 40.0 { 1.0 } else { 0.0 }), vec2(w, h));
        let response = ui.interact(r, ui.scope_id().with(("menu", m.name)), Sense::click());
        let active = match m.panel {
            Some(p) => s.windows.is_open(p),
            None => s.windows.legend,
        };
        m.paint(&painter, r, &response, tier, active);
        if m.panel == Some(Panel::Notifications) && unread > 0 {
            theme::badge(
                &painter,
                if tier == Tier::Captions {
                    pos2(r.center().x + 14.0, r.top() + 6.0)
                } else {
                    r.right_top() + vec2(-3.0, 4.0)
                },
                &if unread > 99 {
                    "99+".into()
                } else {
                    unread.to_string()
                },
                BAD,
            );
        }
        let response = response
            .on_hover_text(super::widgets::tooltip(
                m.name,
                &format!("{}. Click again to close.", m.what),
                Some(m.key),
                None,
            ))
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if response.clicked() {
            match m.panel {
                Some(p) => s.windows.toggle(p),
                None => s.windows.legend = !s.windows.legend,
            }
        }
        x += w + gap;
    }

    let block_x = menu_left - 26.0 - block_w;
    painter.galley(
        pos2(block_x + block_w - turn_galley.size().x, cy - 20.0),
        turn_galley,
        GOLD,
    );
    painter.galley(
        pos2(block_x + block_w - sub_galley.size().x, cy + 4.0),
        sub_galley,
        TEXT_DIM,
    );
    // A fine rule between the turn block and the menu buttons.
    painter.line_segment(
        [pos2(menu_left - 15.0, cy - 17.0), pos2(menu_left - 15.0, cy + 17.0)],
        Stroke::new(1.0, palette::with_alpha(GOLD_DIM, 150)),
    );
    let avail = block_x - 24.0 - (rect.left() + 12.0);

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

/// How the menu buttons are drawn: icon, name and hotkey; icon and name;
/// or the icon alone with its hotkey.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tier {
    Labels,
    LabelsOnly,
    /// Round button with its name underneath, in small type.
    Captions,
    Icons,
}

struct MenuItem {
    glyph: Glyph,
    name: &'static str,
    label: &'static str,
    key: &'static str,
    panel: Option<Panel>,
    color: Color32,
    what: &'static str,
}

impl MenuItem {
    fn new(
        glyph: Glyph,
        name: &'static str,
        label: &'static str,
        key: &'static str,
        panel: Option<Panel>,
        color: Color32,
        what: &'static str,
    ) -> Self {
        MenuItem { glyph, name, label, key, panel, color, what }
    }

    fn label_width(&self, painter: &egui::Painter) -> f32 {
        painter
            .layout_no_wrap(self.label.to_string(), FontId::proportional(14.5), TEXT)
            .size()
            .x
    }

    fn width(&self, painter: &egui::Painter, tier: Tier) -> f32 {
        match tier {
            Tier::Icons => 38.0,
            Tier::Captions => {
                let g = painter.layout_no_wrap(self.label.to_string(), FontId::proportional(11.5), TEXT);
                (g.size().x + 10.0).max(38.0)
            }
            Tier::LabelsOnly => 36.0 + self.label_width(painter) + 12.0,
            Tier::Labels => 36.0 + self.label_width(painter) + 8.0 + 18.0 + 10.0,
        }
    }

    fn paint(&self, painter: &egui::Painter, r: Rect, response: &egui::Response, tier: Tier, active: bool) {
        if tier == Tier::Captions {
            let c = pos2(r.center().x, r.top() + 17.0);
            let circle = Rect::from_center_size(c, vec2(32.0, 32.0));
            theme::paint_round_button(painter, circle, response, self.glyph, self.color, None, true, active);
            theme::text(
                painter,
                pos2(r.center().x, r.bottom() - 6.0),
                Align2::CENTER_CENTER,
                self.label,
                11.5,
                if response.hovered() || active { GOLD } else { TEXT },
            );
            return;
        }
        if tier == Tier::Icons {
            let d = r.height().min(r.width());
            let r = Rect::from_center_size(r.center(), vec2(d, d));
            theme::paint_round_button(painter, r, response, self.glyph, self.color, Some(self.key), true, active);
            return;
        }
        let hovered = response.hovered();
        let fill = if active {
            palette::TEAL_DIM
        } else if hovered {
            palette::NAVY_HOVER
        } else {
            NAVY_RAISED
        };
        theme::well(painter, r, 18.0, fill, Color32::TRANSPARENT);
        painter.rect_stroke(
            r,
            18.0,
            Stroke::new(1.5, if hovered || active { GOLD } else { GOLD_DIM }),
            StrokeKind::Inside,
        );
        let ic = pos2(r.left() + 19.0, r.center().y);
        theme::disc(painter, ic, 14.0, Color32::from_rgb(8, 14, 24), GOLD_DIM);
        super::glyph::paint(painter, self.glyph, ic, 9.0, self.color);
        let text = theme::text(
            painter,
            pos2(r.left() + 38.0, r.center().y),
            Align2::LEFT_CENTER,
            self.label,
            14.5,
            TEXT,
        );
        if tier == Tier::Labels {
            theme::keycap(painter, pos2(text.right() + 8.0 + 9.0, r.center().y), self.key);
        }
    }
}

fn chip_width(painter: &egui::Painter, chip: &Chip) -> f32 {
    let main = painter.layout_no_wrap(chip.main.clone(), FontId::proportional(18.0), TEXT);
    let mut w = 40.0 + main.size().x + 12.0;
    if let Some((sub, _)) = &chip.sub {
        w += 5.0
            + painter
                .layout_no_wrap(sub.clone(), FontId::proportional(14.0), TEXT)
                .size()
                .x;
    }
    w
}

fn draw_chip(painter: &egui::Painter, r: Rect, chip: &Chip, hovered: bool) {
    theme::well(
        painter,
        r,
        18.0,
        if hovered { palette::NAVY_HOVER } else { NAVY_RAISED },
        Color32::TRANSPARENT,
    );
    painter.rect_stroke(
        r,
        18.0,
        Stroke::new(
            1.5,
            if hovered {
                GOLD
            } else {
                palette::with_alpha(GOLD_DIM, 200)
            },
        ),
        StrokeKind::Inside,
    );
    let ic = pos2(r.left() + 19.0, r.center().y);
    theme::disc(painter, ic, 14.0, Color32::from_rgb(8, 14, 24), GOLD_DIM);
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
    ui.label(theme::title("Population", 17.0));
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

//! The city panel, right side: citizens and growth, yields, loyalty, the
//! current production with a progress ring, and what the city can build.

use super::glyph::Glyph;
use super::widgets::{self, Tool};
use super::{MARGIN, TOP_BAR, glyph, theme};
use crate::hud::Session;
use crate::palette::{
    self, BAD, COMPUTE, GOLD, GOLD_DIM, GOOD, NAVY, NAVY_HOVER, NAVY_RAISED, TEXT, TEXT_DIM,
    YIELD_COLORS,
};
use egui::{Align2, Color32, Rect, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use wayciv_core::state::Production;
use wayciv_core::{CityId, Game};

pub const WIDTH: f32 = 400.0;

/// The lowest edge of the panel: above the minimap and the Next Turn button.
pub fn bottom_limit(s: &Session, screen: Rect) -> f32 {
    (screen.bottom() - MARGIN - super::minimap::outer_size(s).y - 10.0)
        .min(screen.bottom() - 170.0)
}

const TABS: [(&str, Glyph); 7] = [
    ("Units", Glyph::Swords),
    ("Districts", Glyph::Hex),
    ("Buildings", Glyph::Housing),
    ("Wonders", Glyph::Star),
    ("Projects", Glyph::Production),
    ("Faith", Glyph::Faith),
    ("Info", Glyph::Crown),
];

pub fn show(s: &mut Session, ctx: &egui::Context) {
    let Some(id) = s.selected_city else { return };
    if !s.game.cities.contains_key(&id) {
        s.select_city(None);
        return;
    }
    let screen = ctx.content_rect();
    // The leader ribbon sits to the left of the panel, so the panel gets the
    // whole right edge from the top bar down to the minimap.
    let top = TOP_BAR + MARGIN;
    let height = (bottom_limit(s, screen) - top).max(300.0);
    egui::Area::new(egui::Id::unique("city_panel"))
        .fixed_pos(pos2(screen.right() - MARGIN - WIDTH, top))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            theme::plate_frame().show(ui, |ui| {
                ui.set_width(WIDTH - 26.0);
                ui.set_height(height - 26.0);
                panel(s, ui, id);
            });
        });
}

fn panel(s: &mut Session, ui: &mut Ui, id: CityId) {
    let report = s.game.city_report(id);
    let c = s.game.cities[&id].clone();
    let me = s.me();
    ui.spacing_mut().item_spacing.y = 6.0;

    // Header: previous and next city, name, capital star, close.
    let mut go: Option<CityId> = None;
    ui.horizontal(|ui| {
        let mine: Vec<CityId> = {
            let mut v: Vec<CityId> = s.game.player_cities(me).map(|c| c.id).collect();
            v.sort();
            v
        };
        let at = mine.iter().position(|&c| c == id).unwrap_or(0);
        if mine.len() > 1 {
            let prev = mine[(at + mine.len() - 1) % mine.len()];
            let next = mine[(at + 1) % mine.len()];
            if widgets::tool_button(ui, 30.0, Tool::Prev, false, "Previous city", &s.game.cities[&prev].name, None) {
                go = Some(prev);
            }
            if widgets::tool_button(ui, 30.0, Tool::Next, false, "Next city", &s.game.cities[&next].name, None) {
                go = Some(next);
            }
        }
        if c.capital {
            let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
            glyph::paint(ui.painter(), Glyph::Star, rect.center(), 11.0, GOLD);
        }
        ui.label(RichText::new(&c.name).size(25.0).strong().color(GOLD));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if theme::round_button(ui, 32.0, Glyph::Cross, TEXT, None, true)
                .on_hover_text("Close the city panel (Esc)")
                .clicked()
            {
                s.select_city(None);
            }
        });
    });
    if let Some(next) = go {
        s.select_city(Some(next));
        let pos = s.game.cities[&next].pos;
        crate::map_view::center_on(s, pos);
        return;
    }

    // The details (growth, yields, status) fold away to leave room for the
    // production list; short windows start folded.
    let details_id = ui.scope_id().with("city_details");
    let details = ui
        .data(|d| d.get_temp::<bool>(details_id))
        .unwrap_or(ui.available_height() > 620.0);
    {
        let galley_text = if details {
            "Hide details".to_string()
        } else {
            format!(
                "Show details: {} citizens, {:+.1} food, {:.1} production",
                c.population,
                report.yields[0] - report.food_consumed,
                report.yields[1]
            )
        };
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::click());
        let painter = ui.painter();
        painter.rect_filled(
            rect,
            13.0,
            if response.hovered() { NAVY_HOVER } else { Color32::from_white_alpha(12) },
        );
        let cx = rect.left() + 16.0;
        // A chevron: down when folded, up when open.
        let d = if details { -1.0 } else { 1.0 };
        let p = |x: f32, y: f32| pos2(cx + x, rect.center().y + y * d);
        painter.line_segment([p(-5.0, -2.5), p(0.0, 2.5)], Stroke::new(2.0, GOLD));
        painter.line_segment([p(0.0, 2.5), p(5.0, -2.5)], Stroke::new(2.0, GOLD));
        let text = super::trackers::fit(painter, &galley_text, 14.0, rect.width() - 40.0);
        theme::text(painter, pos2(rect.left() + 30.0, rect.center().y), Align2::LEFT_CENTER, text, 14.0, TEXT_DIM);
        if response
            .on_hover_text(if details {
                "Hide growth, yields and status to make room for the production list."
            } else {
                "Show growth, yields, housing, amenities and loyalty."
            })
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            ui.data_mut(|d| d.insert_temp(details_id, !details));
        }
    }

    if details {
    growth_block(s, ui, id, &c, &report);

    // Yields: icon over number.
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 50.0), Sense::hover());
    let painter = ui.painter().clone();
    painter.rect_filled(rect, 12.0, Color32::from_white_alpha(12));
    let col_w = rect.width() / 6.0;
    for i in 0..6 {
        let x = rect.left() + col_w * (i as f32 + 0.5);
        glyph::paint(
            &painter,
            theme::yield_glyph(i),
            pos2(x, rect.top() + 14.0),
            9.0,
            YIELD_COLORS[i],
        );
        theme::text_bold(
            &painter,
            pos2(x, rect.top() + 36.0),
            Align2::CENTER_CENTER,
            format!("{:.1}", report.yields[i]),
            15.0,
            TEXT,
        );
        let tip = format!(
            "{} {:.1} per turn",
            wayciv_core::rules::YIELD_NAMES[i],
            report.yields[i]
        );
        ui.interact(
            Rect::from_min_size(
                pos2(x - col_w / 2.0, rect.top()),
                vec2(col_w, rect.height()),
            ),
            ui.scope_id().with(("yield", i)),
            Sense::hover(),
        )
        .on_hover_text(tip);
    }

    status_rows(s, ui, id, &c);
    }

    ui.add_space(2.0);
    production_block(s, ui, id, &c);

    // Tabs.
    let options = s.game.available_production(id);
    let faith = s.game.faith_units(id);
    let count = |f: &dyn Fn(&Production) -> bool| options.iter().filter(|p| f(p)).count();
    let counts: [usize; 7] = [
        count(&|p| matches!(p, Production::Unit(_))),
        count(&|p| matches!(p, Production::District(..))),
        count(&|p| matches!(p, Production::Building(_))),
        count(&|p| matches!(p, Production::Wonder(..))),
        count(&|p| matches!(p, Production::Project(_))),
        faith.len(),
        1,
    ];
    let visible: Vec<usize> = (0..7).filter(|&i| counts[i] > 0).collect();
    if !visible.contains(&s.city_tab) {
        s.city_tab = visible.first().copied().unwrap_or(6);
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(5.0, 5.0);
        for &i in &visible {
            let (name, g) = TABS[i];
            let label = if i == 6 {
                name.to_string()
            } else {
                format!("{name} {}", counts[i])
            };
            let selected = s.city_tab == i;
            let galley = ui
                .painter()
                .layout_no_wrap(label, egui::FontId::proportional(14.5), TEXT);
            let (rect, response) =
                ui.allocate_exact_size(vec2(galley.size().x + 40.0, 30.0), Sense::click());
            let painter = ui.painter();
            let fill = if selected {
                palette::TEAL_DIM
            } else if response.hovered() {
                NAVY_HOVER
            } else {
                NAVY_RAISED
            };
            painter.rect_filled(rect, 15.0, fill);
            painter.rect_stroke(
                rect,
                15.0,
                Stroke::new(1.5, if selected { GOLD } else { GOLD_DIM }),
                StrokeKind::Inside,
            );
            glyph::paint(
                painter,
                g,
                pos2(rect.left() + 17.0, rect.center().y),
                8.0,
                if selected { GOLD } else { TEXT_DIM },
            );
            painter.galley(
                pos2(rect.left() + 31.0, rect.center().y - galley.size().y / 2.0),
                galley,
                TEXT,
            );
            if response.clicked() {
                s.city_tab = i;
            }
        }
    });

    egui::ScrollArea::vertical()
        .id_salt("city_list")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .auto_shrink([false, false])
        .max_height(ui.available_height())
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 5.0;
            match s.city_tab {
                5 => faith_list(s, ui, id, &faith),
                6 => info(s, ui, id, &c),
                tab => {
                    let items: Vec<&Production> = options
                        .iter()
                        .filter(|p| {
                            matches!(
                                (tab, p),
                                (0, Production::Unit(_))
                                    | (1, Production::District(..))
                                    | (2, Production::Building(_))
                                    | (3, Production::Wonder(..))
                                    | (4, Production::Project(_))
                            )
                        })
                        .collect();
                    for item in items {
                        item_row(s, ui, id, &c, item, me);
                    }
                }
            }
        });
}

fn growth_block(
    s: &Session,
    ui: &mut Ui,
    _id: CityId,
    c: &wayciv_core::state::City,
    report: &wayciv_core::city::CityReport,
) {
    let _ = s;
    let surplus = report.yields[0] - report.food_consumed;
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), 104.0), Sense::hover());
    let painter = ui.painter().clone();
    painter.rect_filled(rect, 14.0, Color32::from_white_alpha(12));
    let center = pos2(rect.left() + 56.0, rect.center().y);
    let frac = c.food as f32 / report.growth_threshold.max(1) as f32;
    theme::disc(
        &painter,
        center,
        40.0,
        Color32::from_rgb(16, 30, 48),
        GOLD_DIM,
    );
    theme::ring(&painter, center, 45.0, frac, 7.0, GOOD);
    theme::text_bold(
        &painter,
        center + vec2(0.0, -2.0),
        Align2::CENTER_CENTER,
        c.population,
        38.0,
        Color32::WHITE,
    );
    theme::text(
        &painter,
        center + vec2(0.0, 22.0),
        Align2::CENTER_CENTER,
        "CITIZENS",
        10.5,
        TEXT_DIM,
    );
    let x = rect.left() + 118.0;
    theme::text_bold(
        &painter,
        pos2(x, rect.top() + 20.0),
        Align2::LEFT_CENTER,
        format!("{} citizens", c.population),
        19.0,
        Color32::WHITE,
    );
    let (grow, color) = match report.turns_to_grow {
        Some(t) => (
            format!("Grows in {t} turn{}", if t == 1 { "" } else { "s" }),
            GOOD,
        ),
        None if surplus < 0.0 => ("Starving!".to_string(), BAD),
        None => ("Not growing".to_string(), Color32::from_rgb(240, 160, 70)),
    };
    theme::text_bold(
        &painter,
        pos2(x, rect.top() + 44.0),
        Align2::LEFT_CENTER,
        grow,
        17.0,
        color,
    );
    let bar = Rect::from_min_size(
        pos2(x, rect.top() + 62.0),
        vec2(rect.right() - x - 14.0, 10.0),
    );
    theme::bar(&painter, bar, frac, GOOD);
    theme::text(
        &painter,
        pos2(x, rect.top() + 86.0),
        Align2::LEFT_CENTER,
        format!(
            "Food {}/{}   {:+.1} per turn",
            c.food, report.growth_threshold, surplus
        ),
        14.0,
        if surplus < 0.0 { BAD } else { TEXT_DIM },
    );
    response.on_hover_text(format!(
        "Each citizen eats 2 food. {:.1} food produced, {:.1} eaten.\nGrowth slows near the housing limit ({}) and stops 4 above it.",
        report.yields[0], report.food_consumed, report.housing
    ));
}

fn status_rows(s: &mut Session, ui: &mut Ui, id: CityId, c: &wayciv_core::state::City) {
    let report = s.game.city_report(id);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
        let crowded = c.population >= report.housing;
        chip(
            ui,
            Glyph::Housing,
            if crowded { BAD } else { TEXT },
            format!("Housing {}", report.housing),
            &format!(
                "Housing {}: growth slows to half one below it, to a quarter at it, and stops at +4.\nBuild Aqueducts, Neighborhoods and housing buildings.",
                report.housing
            ),
        );
        let diff = report.amenities - report.amenities_needed;
        chip(
            ui,
            Glyph::Amenity,
            if diff < 0 { BAD } else if diff > 0 { GOOD } else { TEXT },
            format!("Amenities {}/{}", report.amenities, report.amenities_needed),
            "Amenities / needed. A surplus boosts yields; a shortfall lowers them and can stop growth.",
        );
        chip(
            ui,
            Glyph::Shield,
            if c.hp < 200 { GOLD } else { TEXT },
            format!("HP {}/200", c.hp),
            &format!("City health. Walls {}.", c.wall_hp),
        );
        if c.wall_hp > 0 {
            chip(ui, Glyph::City, Color32::from_rgb(130, 180, 240), format!("Walls {}", c.wall_hp), "Wall strength");
        }
        let compute = s.game.city_compute(c);
        if compute > 0.0 {
            chip(ui, Glyph::Chip, COMPUTE, format!("+{compute:.0} Compute"), "Datacenter Compute for the AI tree");
        }
        if let Some(r) = c.majority_religion() {
            chip(
                ui,
                Glyph::Faith,
                YIELD_COLORS[5],
                s.game.rules.content.religions[r].name.clone(),
                "Majority religion",
            );
        }
    });
    if s.game.loyalty_applies(id) {
        let loyalty = s.game.loyalty_report(id);
        let breakdown: Vec<String> = loyalty
            .lines
            .iter()
            .map(|(name, v)| format!("{name}: {v:+}"))
            .collect();
        let color = if c.loyalty > 50 {
            Color32::from_rgb(190, 150, 240)
        } else if c.loyalty > 25 {
            GOLD
        } else {
            BAD
        };
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
        let painter = ui.painter();
        glyph::paint(
            painter,
            Glyph::Loyalty,
            pos2(rect.left() + 11.0, rect.center().y),
            8.5,
            color,
        );
        let label = theme::text(
            painter,
            pos2(rect.right(), rect.center().y),
            Align2::RIGHT_CENTER,
            format!("Loyalty {}/100 ({:+}/turn)", c.loyalty, loyalty.per_turn),
            14.0,
            color,
        );
        let bar = Rect::from_min_max(
            pos2(rect.left() + 28.0, rect.center().y - 5.0),
            pos2(label.left() - 10.0, rect.center().y + 5.0),
        );
        theme::bar(painter, bar, c.loyalty as f32 / 100.0, color);
        response.on_hover_text(format!(
            "At 0 the city joins the civilization pulling on it hardest.\n{}",
            breakdown.join("\n")
        ));
    }
    if s.game.max_walls(id) > 0
        && !c.has_attacked
        && theme::pill_button(ui, Some(Glyph::Bow), "City Ranged Attack", TEXT, 0.0).clicked()
    {
        s.city_targeting = true;
    }
}

fn chip(ui: &mut Ui, g: Glyph, color: Color32, text: String, tip: &str) {
    let galley = ui
        .painter()
        .layout_no_wrap(text, egui::FontId::proportional(14.5), TEXT);
    let (rect, response) =
        ui.allocate_exact_size(vec2(galley.size().x + 34.0, 26.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 13.0, Color32::from_white_alpha(14));
    glyph::paint(
        painter,
        g,
        pos2(rect.left() + 14.0, rect.center().y),
        8.0,
        color,
    );
    painter.galley(
        pos2(rect.left() + 27.0, rect.center().y - galley.size().y / 2.0),
        galley,
        if color == TEXT { TEXT } else { color },
    );
    response.on_hover_text(tip);
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

/// Paints an item's icon (unit art, else a symbol) centered at `c`.
fn item_icon(
    s: &mut Session,
    painter: &egui::Painter,
    item: &Production,
    c: egui::Pos2,
    size: f32,
) {
    if let Production::Unit(u) = item {
        let id = s.game.rules.content.units[*u].id.clone();
        if s.icons.paint(
            painter,
            "unit",
            &id,
            Rect::from_center_size(c, vec2(size, size)),
        ) {
            return;
        }
    }
    glyph::paint(painter, item_glyph(item), c, size * 0.36, YIELD_COLORS[1]);
}

fn production_block(s: &mut Session, ui: &mut Ui, id: CityId, c: &wayciv_core::state::City) {
    let me = s.me();
    let buyable = c
        .production
        .as_ref()
        .is_some_and(|item| s.game.purchase_cost(id, item).is_some());
    let (rect, _) = ui.allocate_exact_size(
        vec2(ui.available_width(), if buyable { 96.0 } else { 84.0 }),
        Sense::hover(),
    );
    let painter = ui.painter().clone();
    painter.rect_filled(
        rect,
        14.0,
        Color32::from_rgb(40, 28, 16).gamma_multiply(1.0),
    );
    painter.rect_stroke(
        rect,
        14.0,
        Stroke::new(1.5, palette::with_alpha(YIELD_COLORS[1], 150)),
        StrokeKind::Inside,
    );
    let center = pos2(rect.left() + 46.0, rect.top() + 42.0);
    theme::disc(
        &painter,
        center,
        31.0,
        Color32::from_rgb(20, 30, 46),
        GOLD_DIM,
    );
    let Some(item) = &c.production else {
        glyph::paint(
            &painter,
            Glyph::Production,
            center,
            15.0,
            YIELD_COLORS[1].gamma_multiply(0.6),
        );
        theme::ring(&painter, center, 35.0, 1.0, 5.0, GOLD);
        theme::text_bold(
            &painter,
            pos2(rect.left() + 94.0, rect.top() + 32.0),
            Align2::LEFT_CENTER,
            "Choose production",
            18.0,
            GOLD,
        );
        theme::text(
            &painter,
            pos2(rect.left() + 94.0, rect.top() + 54.0),
            Align2::LEFT_CENTER,
            "Pick something from the list below",
            13.5,
            TEXT_DIM,
        );
        return;
    };
    let cost = s.game.production_cost(me, item).max(1);
    let done = c
        .progress
        .get(&Game::production_key(item))
        .copied()
        .unwrap_or(0);
    let rate = s.game.production_toward(id, item).max(0.1);
    let turns = ((cost - done).max(0) as f32 / rate).ceil() as i32;
    theme::ring(
        &painter,
        center,
        35.0,
        done as f32 / cost as f32,
        5.5,
        YIELD_COLORS[1],
    );
    item_icon(s, &painter, item, center, 46.0);
    let x = rect.left() + 94.0;
    let name = super::trackers::fit(
        &painter,
        &s.game.item_name(item),
        18.0,
        rect.right() - x - 10.0,
    );
    theme::text_bold(
        &painter,
        pos2(x, rect.top() + 22.0),
        Align2::LEFT_CENTER,
        name,
        18.0,
        Color32::WHITE,
    );
    theme::text(
        &painter,
        pos2(x, rect.top() + 44.0),
        Align2::LEFT_CENTER,
        format!(
            "{done}/{cost}   {turns} turn{}",
            if turns == 1 { "" } else { "s" }
        ),
        15.0,
        YIELD_COLORS[1],
    );
    if let Some(price) = s.game.purchase_cost(id, item) {
        let gold = s.game.players[me].gold;
        let afford = gold >= price;
        let btn = Rect::from_min_size(pos2(x, rect.bottom() - 40.0), vec2(190.0, 30.0));
        let response = ui.interact(btn, ui.scope_id().with("buy_now"), Sense::click());
        let hovered = response.hovered() && afford;
        let fade = |c: Color32| if afford { c } else { c.gamma_multiply(0.5) };
        painter.rect_filled(btn, 15.0, fade(if hovered { NAVY_HOVER } else { NAVY }));
        painter.rect_stroke(
            btn,
            15.0,
            Stroke::new(1.5, fade(if hovered { GOLD } else { GOLD_DIM })),
            StrokeKind::Inside,
        );
        glyph::paint(
            &painter,
            Glyph::Gold,
            pos2(btn.left() + 17.0, btn.center().y),
            9.0,
            fade(YIELD_COLORS[2]),
        );
        theme::text_bold(
            &painter,
            pos2(btn.left() + 34.0, btn.center().y),
            Align2::LEFT_CENTER,
            format!("Buy now for {price}"),
            15.0,
            fade(TEXT),
        );
        let tip = if afford {
            format!("Buy {} now for {price} Gold (you have {gold}).", s.game.item_name(item))
        } else {
            format!("Not enough Gold: {price} needed, you have {gold}.")
        };
        if response.on_hover_text(tip).clicked() && afford {
            let item = item.clone();
            let r = s.game.purchase(id, item);
            s.result(r);
        }
    }
}

fn item_row(
    s: &mut Session,
    ui: &mut Ui,
    id: CityId,
    c: &wayciv_core::state::City,
    item: &Production,
    me: usize,
) {
    let cost = s.game.production_cost(me, item);
    let rate = s.game.production_toward(id, item).max(0.1);
    let done = c
        .progress
        .get(&Game::production_key(item))
        .copied()
        .unwrap_or(0);
    let turns = ((cost - done).max(0) as f32 / rate).ceil() as i32;
    let name = s.game.item_name(item);
    let price = s.game.purchase_cost(id, item);
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::click());
    let painter = ui.painter().clone();
    let current = c.production.as_ref() == Some(item);
    let hovered = response.hovered();
    painter.rect_filled(rect, 12.0, if hovered { NAVY_HOVER } else { NAVY_RAISED });
    painter.rect_stroke(
        rect,
        12.0,
        Stroke::new(
            if hovered || current { 2.0 } else { 1.0 },
            if hovered || current {
                GOLD
            } else {
                palette::with_alpha(GOLD_DIM, 140)
            },
        ),
        StrokeKind::Inside,
    );
    let ic = pos2(rect.left() + 28.0, rect.center().y);
    painter.circle_filled(ic, 21.0, Color32::from_rgb(14, 24, 40));
    item_icon(s, &painter, item, ic, 34.0);
    if done > 0 {
        theme::arc(
            &painter,
            ic,
            22.0,
            done as f32 / cost.max(1) as f32,
            3.0,
            YIELD_COLORS[1],
        );
    }
    let x = rect.left() + 58.0;
    let right_pad = if price.is_some() { 104.0 } else { 14.0 };
    let name_full = name.clone();
    let name = super::trackers::fit(&painter, &name, 16.0, rect.right() - x - right_pad);
    theme::text_bold(
        &painter,
        pos2(x, rect.top() + 18.0),
        Align2::LEFT_CENTER,
        name,
        16.0,
        TEXT,
    );
    glyph::paint(
        &painter,
        Glyph::Production,
        pos2(x + 7.0, rect.top() + 37.0),
        6.5,
        YIELD_COLORS[1],
    );
    theme::text(
        &painter,
        pos2(x + 18.0, rect.top() + 37.0),
        Align2::LEFT_CENTER,
        format!("{cost}   {turns} turn{}", if turns == 1 { "" } else { "s" }),
        13.5,
        TEXT_DIM,
    );
    // A visible Buy button: the price in gold, dimmed when you can't pay.
    let mut buy_clicked = false;
    if let Some(p) = price {
        let gold = s.game.players[me].gold;
        let afford = gold >= p;
        let btn = Rect::from_min_size(
            pos2(rect.right() - 92.0, rect.center().y - 15.0),
            vec2(82.0, 30.0),
        );
        let br = ui.interact(btn, ui.scope_id().with(("buy", Game::production_key(item))), Sense::click());
        let hovered = br.hovered() && afford;
        let fade = |c: Color32| if afford { c } else { c.gamma_multiply(0.5) };
        painter.rect_filled(btn, 15.0, fade(if hovered { NAVY_HOVER } else { NAVY }));
        painter.rect_stroke(
            btn,
            15.0,
            Stroke::new(1.5, fade(if hovered { GOLD } else { GOLD_DIM })),
            StrokeKind::Inside,
        );
        glyph::paint(
            &painter,
            Glyph::Gold,
            pos2(btn.left() + 14.0, btn.center().y),
            8.0,
            fade(YIELD_COLORS[2]),
        );
        theme::text_bold(
            &painter,
            pos2(btn.left() + 27.0, btn.center().y),
            Align2::LEFT_CENTER,
            format!("Buy {p}"),
            14.0,
            fade(YIELD_COLORS[2]),
        );
        let tip = if afford {
            format!("Buy {name_full} now for {p} Gold (you have {gold}).")
        } else {
            format!("Not enough Gold: {p} needed, you have {gold}.")
        };
        if br.on_hover_text(tip).clicked() && afford {
            buy_clicked = true;
        }
    }
    let tip = match price {
        Some(p) => format!(
            "{}\n\nClick to build this. Press Buy (or right-click) to buy it now for {p} Gold.",
            item_tooltip(&s.game, item)
        ),
        None => item_tooltip(&s.game, item),
    };
    let response = response.on_hover_text(tip);
    if (response.secondary_clicked() || buy_clicked) && price.is_some() {
        let r = s.game.purchase(id, item.clone());
        if r.is_ok() {
            s.toast(format!("Bought {}.", s.game.item_name(item)));
        }
        s.result(r);
    }
    if response.clicked() && !buy_clicked {
        match item {
            Production::District(..) | Production::Wonder(..) => s.placing = Some(item.clone()),
            _ => match s.game.set_production(id, item.clone()) {
                // Like Civ VI, the city panel closes once production is chosen.
                Ok(()) => {
                    let city = s.game.cities[&id].name.clone();
                    s.toast(format!("{city} is building {}.", s.game.item_name(item)));
                    s.select_city(None);
                }
                Err(e) => s.toast(e),
            },
        }
    }
}

fn faith_list(s: &mut Session, ui: &mut Ui, id: CityId, faith: &[usize]) {
    ui.label(
        RichText::new("Buy with Faith")
            .strong()
            .color(YIELD_COLORS[5]),
    );
    for &unit in faith {
        let cost = s.game.faith_cost(unit);
        let name = s.game.rules.content.units[unit].name.clone();
        let unit_id = s.game.rules.content.units[unit].id.clone();
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
        let painter = ui.painter().clone();
        painter.rect_filled(
            rect,
            12.0,
            if response.hovered() {
                NAVY_HOVER
            } else {
                NAVY_RAISED
            },
        );
        painter.rect_stroke(rect, 12.0, Stroke::new(1.0, GOLD_DIM), StrokeKind::Inside);
        let ic = pos2(rect.left() + 26.0, rect.center().y);
        if !s.icons.paint(
            &painter,
            "unit",
            &unit_id,
            Rect::from_center_size(ic, vec2(34.0, 34.0)),
        ) {
            glyph::paint(&painter, Glyph::Faith, ic, 12.0, YIELD_COLORS[5]);
        }
        theme::text_bold(
            &painter,
            pos2(rect.left() + 56.0, rect.center().y),
            Align2::LEFT_CENTER,
            name,
            16.0,
            TEXT,
        );
        glyph::paint(
            &painter,
            Glyph::Faith,
            pos2(rect.right() - 62.0, rect.center().y),
            8.0,
            YIELD_COLORS[5],
        );
        theme::text_bold(
            &painter,
            pos2(rect.right() - 48.0, rect.center().y),
            Align2::LEFT_CENTER,
            cost,
            15.0,
            YIELD_COLORS[5],
        );
        if response.on_hover_text("Click to buy with Faith").clicked() {
            let r = s.game.buy_with_faith(id, unit);
            s.result(r);
        }
    }
}

fn info(s: &mut Session, ui: &mut Ui, id: CityId, c: &wayciv_core::state::City) {
    let routes: Vec<_> = s
        .game
        .trade_routes
        .iter()
        .filter(|r| r.origin == id)
        .cloned()
        .collect();
    if !routes.is_empty() {
        theme::heading(ui, "Trade routes");
        for r in routes {
            let y = s.game.route_yields(r.owner, r.destination);
            let parts: Vec<String> = (0..6)
                .filter(|&i| y[i] > 0.0)
                .map(|i| format!("+{:.0} {}", y[i], wayciv_core::rules::YIELD_NAMES[i]))
                .collect();
            let dest = s
                .game
                .cities
                .get(&r.destination)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            ui.label(format!(
                "To {dest}: {} ({} turns left)",
                parts.join(", "),
                r.turns_left
            ));
        }
    }
    theme::heading(ui, "Districts and buildings");
    let content = &s.game.rules.content;
    let mut any = false;
    for d in &c.districts {
        if !content.districts[d.district].city_center {
            ui.label(format!("{}", content.districts[d.district].name));
            any = true;
        }
    }
    for &b in &c.buildings {
        ui.label(content.buildings[b].name.clone());
        any = true;
    }
    if !any {
        ui.label(RichText::new("Nothing built yet.").color(TEXT_DIM));
    }
}

pub fn item_tooltip(game: &Game, item: &Production) -> String {
    let content = &game.rules.content;
    match item {
        Production::Unit(i) => {
            let u = &content.units[*i];
            format!(
                "{}\nStrength {}  Ranged {}  Moves {}\n{}",
                u.name, u.combat, u.ranged_combat, u.moves, u.description
            )
        }
        Production::Building(i) | Production::Wonder(i, _) => {
            let b = &content.buildings[*i];
            let y: Vec<String> = (0..6)
                .filter(|&k| b.yields[k] != 0)
                .map(|k| format!("+{} {}", b.yields[k], wayciv_core::rules::YIELD_NAMES[k]))
                .collect();
            format!("{}\n{}\n{}", b.name, y.join(", "), b.description)
        }
        Production::District(i, _) => {
            let d = &content.districts[*i];
            format!("{}\n{}", d.name, d.description)
        }
        Production::Project(i) => {
            let p = &content.projects[*i];
            format!("{}\n{}", p.name, p.description)
        }
    }
}

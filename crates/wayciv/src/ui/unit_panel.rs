//! The unit panel, bottom left: portrait, name, moves, strength, HP, and
//! labelled action buttons with their hotkeys. The best action for the unit
//! (Found City for a Settler) is the big green one; unavailable actions stay
//! visible, dimmed, and say why.

use super::glyph::Glyph;
use super::widgets::{self, Btn, Kind, Tool};
use super::{MARGIN, glyph, theme};
use crate::hud::Session;
use crate::palette::{self, BAD, GOLD, GOOD, TEXT, TEXT_DIM, rgb};
use egui::{Align2, Color32, Rect, RichText, Sense, Stroke, Ui, pos2, vec2};
use wayciv_core::actions::{MIN_CITY_DISTANCE, UnitAction};
use wayciv_core::turn::Blocker;

/// Width of the text column next to the portrait: as wide as fits left of
/// the minimap, up to 540.
fn column_width(s: &Session, screen: Rect) -> f32 {
    let right = screen.right() - super::turn::DIAMETER - MARGIN - 14.0 - super::minimap::outer_size(s).x - 12.0;
    (right - (MARGIN + 24.0 + 124.0 + 10.0)).clamp(300.0, 540.0)
}

pub fn show(s: &mut Session, ctx: &egui::Context) {
    let unit_selected = s
        .selected_unit
        .is_some_and(|id| s.game.units.contains_key(&id));
    if s.selected_city.is_some() {
        return;
    }
    let screen = ctx.content_rect();
    let col = column_width(s, screen);
    egui::Area::new(egui::Id::unique("unit_panel"))
        .anchor(Align2::LEFT_BOTTOM, vec2(MARGIN, -MARGIN))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            if unit_selected {
                theme::plate_ui(ui, 12, |ui| card(s, ui, col, screen));
            } else {
                hint(s, ui, col + 124.0);
            }
        });
}

/// Units that still need orders this turn.
fn waiting_units(s: &Session, except: Option<wayciv_core::UnitId>) -> usize {
    s.game
        .blockers(s.me())
        .iter()
        .filter(|b| matches!(b, Blocker::UnitNeedsOrders(u) if Some(*u) != except))
        .count()
}

/// Shown when nothing is selected: what to click, and a Next Unit button
/// while some unit still needs orders.
fn hint(s: &mut Session, ui: &mut Ui, width: f32) {
    let waiting = waiting_units(s, None);
    let mut next = false;
    theme::plate_ui(ui, 12, |ui| {
            ui.set_max_width(width.min(520.0));
            ui.label(RichText::new("Click a unit or city to select it.").color(TEXT));
            ui.label(
                RichText::new(
                    "Right-click a tile to move or attack. Drag or use the arrow keys to scroll the map; the mouse wheel zooms.",
                )
                .size(13.5)
                .color(TEXT_DIM),
            );
            if waiting > 0 {
                ui.add_space(4.0);
                next = widgets::pill(
                    ui,
                    Btn::new(&format!("Next Unit ({waiting} waiting)"))
                        .glyph(Glyph::Boot, TEXT)
                        .key(Some("U"))
                        .kind(Kind::Primary)
                        .tip("Select the next unit that needs orders and center the map on it."),
                );
            }
    });
    if next {
        s.next_unit();
    }
}

fn stat(ui: &mut Ui, g: Glyph, color: Color32, text: String, tip: &str) {
    let galley = ui
        .painter()
        .layout_no_wrap(text, egui::FontId::proportional(16.0), TEXT);
    let (rect, response) =
        ui.allocate_exact_size(vec2(galley.size().x + 36.0, 28.0), Sense::hover());
    let painter = ui.painter();
    theme::well(painter, rect, 14.0, palette::NAVY_RAISED, palette::with_alpha(palette::GOLD_DIM, 90));
    glyph::paint(
        painter,
        g,
        pos2(rect.left() + 15.0, rect.center().y),
        8.5,
        color,
    );
    painter.galley(
        pos2(rect.left() + 29.0, rect.center().y - galley.size().y / 2.0),
        galley,
        TEXT,
    );
    response.on_hover_text(tip);
}

/// Symbol, hotkey and color for each order.
fn action_style(a: &UnitAction) -> (Glyph, Option<&'static str>, Color32) {
    match a {
        UnitAction::FoundCity => (Glyph::City, Some("B"), GOLD),
        UnitAction::Fortify => (Glyph::Shield, Some("F"), TEXT),
        UnitAction::Sleep => (Glyph::Sleep, Some("Z"), TEXT),
        UnitAction::Skip => (Glyph::Skip, Some("Space"), TEXT),
        UnitAction::Delete => (Glyph::Cross, None, BAD),
        UnitAction::FoundReligion | UnitAction::SpreadReligion => {
            (Glyph::Faith, None, palette::YIELD_COLORS[5])
        }
        UnitAction::Activate => (Glyph::Star, None, GOLD),
        UnitAction::Build(_) => (Glyph::Build, None, TEXT),
        UnitAction::TradeRoute(_) => (Glyph::Trade, None, palette::YIELD_COLORS[2]),
    }
}

/// What the order does, for its tooltip.
fn action_tip(s: &Session, id: wayciv_core::UnitId, a: &UnitAction) -> String {
    match a {
        UnitAction::FoundCity => "Settle here: this tile becomes the city center.".into(),
        UnitAction::Fortify => "Stay put and dig in for a defense bonus. The unit stops asking for orders.".into(),
        UnitAction::Sleep => "Rest until something comes near. The unit stops asking for orders.".into(),
        UnitAction::Skip => "Give no orders this turn; the unit asks again next turn.".into(),
        UnitAction::Delete => "Disband this unit for good.".into(),
        UnitAction::FoundReligion => "Found a religion at this Holy Site: choose a name and beliefs.".into(),
        UnitAction::SpreadReligion => "Spread your religion to the nearby cities.".into(),
        UnitAction::Activate => s
            .game
            .units
            .get(&id)
            .and_then(|u| u.great_person)
            .map(|gp| s.game.great_person_effect(gp))
            .unwrap_or_default(),
        UnitAction::Build(i) => {
            let imp = &s.game.rules.content.improvements[*i];
            let y: Vec<String> = (0..6)
                .filter(|&k| imp.yields[k] != 0)
                .map(|k| format!("{:+} {}", imp.yields[k], wayciv_core::rules::YIELD_NAMES[k]))
                .collect();
            let mut tip = format!("Build a {} on this tile (uses a charge and all moves).", imp.name);
            if !y.is_empty() {
                tip.push_str(&format!("\nYields {}.", y.join(", ")));
            }
            tip
        }
        UnitAction::TradeRoute(_) => "Send this Trader on a route between the two cities.".into(),
    }
}

/// Why an order can't be used right now, if it can't.
fn unavailable(s: &Session, id: wayciv_core::UnitId, a: &UnitAction) -> Option<String> {
    let u = s.game.units.get(&id)?;
    match a {
        UnitAction::FoundCity if !s.game.can_found_city(id) => {
            let rules = &s.game.rules;
            Some(if u.moves <= 0 {
                "No movement left this turn.".into()
            } else if !s.game.map.is_land(rules, u.pos) || s.game.map.is_impassable(rules, u.pos) {
                "Cities can only be founded on passable land.".into()
            } else if s.game.map.tile(u.pos).owner.is_some_and(|o| o != u.owner) {
                "This land belongs to another civilization.".into()
            } else {
                format!(
                    "Too close to another city: cities must be at least {MIN_CITY_DISTANCE} tiles apart. Move the Settler farther away."
                )
            })
        }
        UnitAction::Build(_) if u.moves <= 0 => Some("No movement left this turn.".into()),
        _ => None,
    }
}

fn card(s: &mut Session, ui: &mut Ui, col: f32, screen: Rect) {
    let Some(id) = s.selected_unit else { return };
    let Some(unit) = s.game.units.get(&id).cloned() else {
        return;
    };
    let ty = s.game.unit_type(&unit).clone();
    let owner = s.game.players[unit.owner].clone();
    let mine = unit.owner == s.me();
    let military = s.game.is_military(&unit);
    let mut deselect = false;
    ui.horizontal_top(|ui| {
        // Portrait: unit icon in a disc, ringed in the owner's color and HP.
        let (rect, _) = ui.allocate_exact_size(vec2(124.0, 124.0), Sense::hover());
        let painter = ui.painter().clone();
        let c = rect.center();
        theme::disc(&painter, c, 50.0, Color32::from_rgb(18, 32, 52), rgb(owner.primary));
        painter.circle_stroke(c, 50.0, Stroke::new(5.0, rgb(owner.primary)));
        painter.circle_stroke(c, 45.0, Stroke::new(1.5, rgb(owner.secondary)));
        if military {
            let frac = unit.hp as f32 / 100.0;
            let color = if frac > 0.6 { GOOD } else if frac > 0.3 { GOLD } else { BAD };
            theme::ring(&painter, c, 56.0, frac, 5.0, color);
        } else {
            painter.circle_stroke(c, 56.0, Stroke::new(2.0, palette::GOLD_DIM));
        }
        let icon_rect = Rect::from_center_size(c, vec2(78.0, 78.0));
        if !s.icons.paint(&painter, "unit", &ty.id, icon_rect) {
            glyph::paint(&painter, if military { Glyph::Swords } else { Glyph::Boot }, c, 28.0, TEXT);
        }

        ui.vertical(|ui| {
            ui.set_width(col);
            ui.horizontal(|ui| {
                ui.label(theme::title(&ty.name, 27.0));
                if let Some(gp) = unit.great_person {
                    ui.label(
                        RichText::new(&s.game.rules.content.great_people[gp].name)
                            .size(16.0)
                            .color(GOLD),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if widgets::tool_button(
                        ui,
                        28.0,
                        Tool::Close,
                        false,
                        "Deselect",
                        "Put this unit away.",
                        Some("Esc"),
                    ) {
                        deselect = true;
                    }
                });
            });
            ui.label(RichText::new(&owner.civ_name).size(14.0).color(TEXT_DIM));
            ui.add_space(2.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                stat(ui, Glyph::Boot, TEXT, format!("{}/{}", unit.moves, ty.moves), "Movement left / per turn");
                if ty.combat > 0 {
                    stat(ui, Glyph::Swords, palette::YIELD_COLORS[1], ty.combat.to_string(), "Combat strength");
                }
                if ty.ranged_combat > 0 {
                    stat(
                        ui,
                        Glyph::Bow,
                        palette::YIELD_COLORS[1],
                        format!("{} (range {})", ty.ranged_combat, ty.range),
                        "Ranged strength and range",
                    );
                }
                if military {
                    let color = if unit.hp > 60 { GOOD } else if unit.hp > 30 { GOLD } else { BAD };
                    stat(ui, Glyph::Heart, color, format!("{}/100", unit.hp), "Hit points");
                } else {
                    stat(ui, Glyph::Heart, TEXT_DIM, "Civilian".into(), "Civilians have no health: one attack kills them");
                }
                if ty.build_charges > 0 || ty.spread_charges > 0 {
                    stat(ui, Glyph::Build, TEXT, format!("{}", unit.charges), "Charges left");
                }
                if unit.level > 0 || unit.xp > 0 {
                    stat(ui, Glyph::Star, GOLD, format!("Lv {} ({} XP)", unit.level, unit.xp), "Promotion level and experience");
                }
                if unit.fortified {
                    stat(ui, Glyph::Shield, GOOD, "Fortified".into(), "Dug in for a defense bonus");
                } else if unit.sleeping {
                    stat(ui, Glyph::Sleep, TEXT_DIM, "Sleeping".into(), "Resting; wakes when an enemy comes near");
                }
            });
            if let Some(gp) = unit.great_person {
                widgets::hint_label(ui, &s.game.great_person_effect(gp), col);
            }
            if s.game.is_air(&unit) {
                widgets::hint_label(
                    ui,
                    &format!(
                        "Operation range {}. Right-click a target to strike, or another Aerodrome to rebase.",
                        ty.range
                    ),
                    col,
                );
            }
            if ty.make_trade_route && mine {
                let used = s.game.trade_routes_of(unit.owner).count() as i32;
                let capacity = s.game.trade_capacity(unit.owner);
                if used >= capacity {
                    widgets::hint_label(
                        ui,
                        &format!(
                            "All {capacity} trade route slots are in use. Commercial Hubs and Harbors add more."
                        ),
                        col,
                    );
                } else if s.game.trade_destinations(id).is_empty() {
                    widgets::hint_label(ui, "Move this Trader into one of your cities to start a trade route.", col);
                }
            }
            if mine {
                actions(s, ui, id, col, screen);
            }
        });
    });
    if deselect {
        s.select_unit(None);
    }
}

fn actions(s: &mut Session, ui: &mut Ui, id: wayciv_core::UnitId, col: f32, screen: Rect) {
    let list = s.game.unit_actions(id);
    let is_primary = |a: &UnitAction| {
        matches!(
            a,
            UnitAction::FoundCity
                | UnitAction::FoundReligion
                | UnitAction::SpreadReligion
                | UnitAction::Activate
        )
    };
    let is_list = |a: &UnitAction| matches!(a, UnitAction::Build(_) | UnitAction::TradeRoute(_));
    let primary: Vec<UnitAction> = list.iter().filter(|a| is_primary(a)).cloned().collect();
    let basic: Vec<UnitAction> = list
        .iter()
        .filter(|a| !is_primary(a) && !is_list(a))
        .cloned()
        .collect();
    let builds: Vec<UnitAction> = list
        .iter()
        .filter(|a| matches!(a, UnitAction::Build(_)))
        .cloned()
        .collect();
    let routes: Vec<UnitAction> = list
        .iter()
        .filter(|a| matches!(a, UnitAction::TradeRoute(_)))
        .cloned()
        .collect();
    let mut chosen: Option<UnitAction> = None;
    let mut next_unit = false;
    let button = |ui: &mut Ui, s: &Session, a: &UnitAction, kind: Kind, label: Option<String>| -> Option<UnitAction> {
        let (g, key, color) = action_style(a);
        let name = label.unwrap_or_else(|| s.game.action_name(a));
        let kind = if matches!(a, UnitAction::Delete) { Kind::Danger } else { kind };
        let b = Btn::new(&name)
            .glyph(g, color)
            .key(key)
            .kind(kind)
            .tip(action_tip(s, id, a))
            .disabled(unavailable(s, id, a));
        widgets::pill(ui, b).then(|| a.clone())
    };
    ui.add_space(4.0);
    // Everything in one wrapping row, the best action first.
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 2.0);
        ui.horizontal_wrapped(
            |ui| {
                ui.set_max_width(col);
                for a in &primary {
                    chosen = button(ui, s, a, Kind::Primary, None).or(chosen.take());
                }
                for a in &basic {
                    if matches!(a, UnitAction::Delete) {
                        continue;
                    }
                    chosen = button(ui, s, a, Kind::Normal, None).or(chosen.take());
                }
                let others_waiting = waiting_units(s, Some(id)) > 0;
                if others_waiting
                    && widgets::pill(
                        ui,
                        Btn::new("Next Unit")
                            .glyph(Glyph::Boot, TEXT)
                            .key(Some("U"))
                            .tip("Select the next unit that needs orders and center the map on it."),
                    )
                {
                    next_unit = true;
                }
                if basic.contains(&UnitAction::Delete) {
                    // Disbanding asks twice: the first click arms the button.
                    let key = ui.scope_id().with(("disband", id));
                    let now = ui.input(|i| i.time);
                    let armed = ui
                        .data(|d| d.get_temp::<f64>(key))
                        .is_some_and(|t| now - t < 4.0);
                    if armed {
                        ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
                    }
                    let b = Btn::new(if armed { "Click again to disband" } else { "Disband" })
                        .glyph(Glyph::Cross, BAD)
                        .kind(Kind::Danger)
                        .tip("Disband this unit for good. Asks you to click twice.");
                    if widgets::pill(ui, b) {
                        if armed {
                            ui.data_mut(|d| d.remove::<f64>(key));
                            chosen = Some(UnitAction::Delete);
                        } else {
                            ui.data_mut(|d| d.insert_temp(key, now));
                        }
                    }
                }
            },
        );
    });
    // Lists scroll in place, with the scrollbar always showing, so nothing
    // is ever cut off without a hint.
    let max_h = (screen.height() - 598.0).clamp(76.0, 160.0);
    let list_section = |ui: &mut Ui, s: &Session, title: &str, items: &[UnitAction], salt: &str, short: bool| -> Option<UnitAction> {
        let mut picked = None;
        if items.is_empty() {
            return picked;
        }
        ui.add_space(4.0);
        ui.label(theme::title(format!("{title} ({})", items.len()), 15.0));
        ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
        egui::ScrollArea::vertical()
            .max_height(max_h)
            .id_salt(salt)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                ui.set_width(col - 14.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                    for a in items {
                        let label = match (short, a) {
                            (true, UnitAction::Build(i)) => {
                                Some(s.game.rules.content.improvements[*i].name.clone())
                            }
                            _ => None,
                        };
                        picked = button(ui, s, a, Kind::Normal, label).or(picked.take());
                    }
                });
            });
        picked
    };
    chosen = list_section(ui, s, "Build", &builds, "unit_builds", true).or(chosen.take());
    chosen = list_section(ui, s, "Trade routes", &routes, "unit_routes", false).or(chosen.take());
    ui.add_space(2.0);
    widgets::hint_label(ui, "Right-click a tile to move or attack.", col);

    if next_unit {
        s.next_unit();
        return;
    }
    if let Some(UnitAction::FoundReligion) = chosen {
        // Founding asks for a name and beliefs first.
        s.windows.open(crate::screens::Panel::Religion);
        s.windows.found = Some(crate::ui::menus::FoundDraft::new(id));
        return;
    }
    if let Some(action) = chosen {
        let r = s.game.perform(id, &action);
        s.result(r);
        s.refresh_selection();
        if !s.game.units.contains_key(&id)
            || matches!(action, UnitAction::Skip | UnitAction::Fortify)
        {
            s.next_unit();
        }
    }
}

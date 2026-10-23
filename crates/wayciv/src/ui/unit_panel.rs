//! The unit panel, bottom left: portrait, name, moves, strength, HP, and
//! round action buttons with their hotkeys.

use super::glyph::Glyph;
use super::{MARGIN, glyph, theme};
use crate::hud::Session;
use crate::palette::{self, BAD, GOLD, GOOD, NAVY, TEXT, TEXT_DIM, rgb};
use egui::{Align2, Color32, Rect, RichText, Sense, Stroke, Ui, pos2, vec2};
use wayciv_core::actions::UnitAction;
use wayciv_core::turn::Blocker;

pub fn show(s: &mut Session, ctx: &egui::Context) {
    let unit_selected = s
        .selected_unit
        .is_some_and(|id| s.game.units.contains_key(&id));
    if s.selected_city.is_some() {
        return;
    }
    egui::Area::new(egui::Id::unique("unit_panel"))
        .anchor(Align2::LEFT_BOTTOM, vec2(MARGIN, -MARGIN))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            if unit_selected {
                theme::plate_frame().show(ui, |ui| card(s, ui));
            } else {
                hint(s, ui);
            }
        });
}

/// Shown when nothing is selected: a one-line reminder of the controls.
fn hint(s: &Session, ui: &mut Ui) {
    let me = s.me();
    let waiting = s
        .game
        .blockers(me)
        .iter()
        .any(|b| matches!(b, Blocker::UnitNeedsOrders(_)));
    egui::Frame::new()
        .fill(palette::with_alpha(NAVY, 220))
        .stroke(Stroke::new(1.5, palette::GOLD_DIM))
        .corner_radius(12.0)
        .inner_margin(egui::Margin::symmetric(14, 8))
        .show(ui, |ui| {
            ui.label(
                RichText::new(if waiting {
                    "Click a unit or city. Press . for the next unit that needs orders."
                } else {
                    "Click a unit or city. Right-click to move or attack."
                })
                .color(TEXT_DIM),
            );
            ui.label(
                RichText::new(
                    "N notifications   L leaders   S research   C civics   R religion   Arrows scroll",
                )
                .size(13.0)
                .color(TEXT_DIM),
            );
        });
}

fn stat(ui: &mut Ui, g: Glyph, color: Color32, text: String, tip: &str) {
    let galley = ui
        .painter()
        .layout_no_wrap(text, egui::FontId::proportional(16.0), TEXT);
    let (rect, response) = ui.allocate_exact_size(vec2(galley.size().x + 36.0, 28.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 14.0, Color32::from_white_alpha(14));
    glyph::paint(painter, g, pos2(rect.left() + 15.0, rect.center().y), 8.5, color);
    painter.galley(pos2(rect.left() + 29.0, rect.center().y - galley.size().y / 2.0), galley, TEXT);
    response.on_hover_text(tip);
}

fn action_style(a: &UnitAction) -> (Glyph, Option<&'static str>, Color32) {
    match a {
        UnitAction::FoundCity => (Glyph::City, Some("B"), GOLD),
        UnitAction::Fortify => (Glyph::Shield, Some("F"), TEXT),
        UnitAction::Sleep => (Glyph::Sleep, Some("Z"), TEXT),
        UnitAction::Skip => (Glyph::Skip, Some("Space"), TEXT),
        UnitAction::Delete => (Glyph::Cross, None, BAD),
        UnitAction::FoundReligion | UnitAction::SpreadReligion => (Glyph::Faith, None, palette::YIELD_COLORS[5]),
        UnitAction::Activate => (Glyph::Star, None, GOLD),
        UnitAction::Build(_) => (Glyph::Build, None, TEXT),
        UnitAction::TradeRoute(_) => (Glyph::Trade, None, palette::YIELD_COLORS[2]),
    }
}

fn card(s: &mut Session, ui: &mut Ui) {
    let Some(id) = s.selected_unit else { return };
    let Some(unit) = s.game.units.get(&id).cloned() else {
        return;
    };
    let ty = s.game.unit_type(&unit).clone();
    let owner = s.game.players[unit.owner].clone();
    let mine = unit.owner == s.me();
    let military = s.game.is_military(&unit);
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
            ui.set_width(380.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(&ty.name).size(24.0).strong().color(Color32::WHITE));
                if let Some(gp) = unit.great_person {
                    ui.label(
                        RichText::new(&s.game.rules.content.great_people[gp].name)
                            .size(16.0)
                            .color(GOLD),
                    );
                }
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
            });
            if let Some(gp) = unit.great_person {
                ui.label(RichText::new(s.game.great_person_effect(gp)).italics().size(13.5).color(TEXT_DIM));
            }
            if s.game.is_air(&unit) {
                ui.label(
                    RichText::new(format!(
                        "Operation range {}. Right-click a target to strike, or another Aerodrome to rebase.",
                        ty.range
                    ))
                    .italics()
                    .size(13.5)
                    .color(TEXT_DIM),
                );
            }
            if ty.make_trade_route && s.game.trade_destinations(id).is_empty() {
                ui.label(
                    RichText::new("Move this Trader into one of your cities to start a trade route.")
                        .italics()
                        .size(13.5)
                        .color(TEXT_DIM),
                );
            }
            if mine {
                actions(s, ui, id);
            }
        });
    });
}

fn actions(s: &mut Session, ui: &mut Ui, id: wayciv_core::UnitId) {
    let list = s.game.unit_actions(id);
    let (round, wide): (Vec<UnitAction>, Vec<UnitAction>) = list
        .into_iter()
        .partition(|a| !matches!(a, UnitAction::Build(_) | UnitAction::TradeRoute(_)));
    let mut chosen: Option<UnitAction> = None;
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
        for a in round {
            let (g, key, color) = action_style(&a);
            let name = s.game.action_name(&a);
            let tip = match key {
                Some(k) => format!("{name} ({k})"),
                None => name,
            };
            if theme::round_button(ui, 48.0, g, color, key, true)
                .on_hover_text(tip)
                .clicked()
            {
                chosen = Some(a);
            }
        }
    });
    if !wide.is_empty() {
        egui::ScrollArea::vertical().max_height(78.0).id_salt("unit_wide_actions").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                for a in wide {
                    let (g, _, color) = action_style(&a);
                    let name = s.game.action_name(&a);
                    if theme::pill_button(ui, Some(g), &name, color, 0.0).clicked() {
                        chosen = Some(a);
                    }
                }
            });
        });
    }
    if let Some(action) = chosen {
        let r = s.game.perform(id, &action);
        s.result(r);
        s.refresh_selection();
        if !s.game.units.contains_key(&id) || matches!(action, UnitAction::Skip | UnitAction::Fortify) {
            s.next_unit();
        }
    }
}

//! Tactical HUD overlays that make gameplay immediately clear and intuitive:
//! - Structured Tile Inspection Tooltip (yields, terrain, resource improvement hints, units)
//! - Civ VI-style Bottom-Center Combat Preview Card (attacker vs defender, HP bars, modifiers)
//! - Map Lens Legend & Turn-1 Founding Guidance Pill
//! - In-Game Settings & Controls Reference Modal

use super::glyph::{self, Glyph};
use super::{MARGIN, TOP_BAR, theme, widgets};
use crate::hud::Session;
use crate::palette::{
    self, BAD, GOLD, GOLD_DIM, GOOD, NAVY, NAVY_RAISED, TEAL, TEXT, TEXT_DIM, YIELD_COLORS, rgb,
    with_alpha,
};
use crate::render::yields::{MapLens, effective_lens};
use egui::{Align2, Color32, Id, Rect, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use wayciv_core::Pos;
use wayciv_core::combat::{AttackKind, CombatBreakdown};
use wayciv_core::rules::YIELD_NAMES;
use wayciv_core::state::Production;

// ------------------------------------------------------------ Tile Tooltip

/// Paints a rich, structured HUD tooltip card for `pos`.
pub fn tile_tooltip_ui(s: &mut Session, ui: &mut Ui, pos: Pos) {
    let me = s.me();
    let tile = s.game.map.tile(pos).clone();
    let content = &s.game.rules.content;
    let terrain = &content.terrains[tile.terrain];
    let relief = if terrain.mountain {
        " · Mountains"
    } else if terrain.hills {
        " · Hills"
    } else {
        ""
    };
    let title = format!("{}{relief}", palette::biome_name(&terrain.id));
    let feat_name = tile.feature.map(|f| content.features[f].name.clone());
    let yields = s.game.tile_yields(me, pos);
    let def_bonus = s.game.tile_defense_bonus(pos);
    let move_cost = if s.game.map.is_impassable(&s.game.rules, pos) {
        None
    } else if s.game.map.is_water(&s.game.rules, pos) {
        Some(1)
    } else {
        Some(s.game.tile_base_move_cost(pos))
    };

    ui.set_max_width(330.0);
    ui.spacing_mut().item_spacing = vec2(6.0, 5.0);

    // Header: Terrain + Feature + Movement/Defense badges.
    ui.horizontal_wrapped(|ui| {
        ui.label(theme::bold(&title, 16.0).color(GOLD));
        if let Some(f) = &feat_name {
            ui.label(RichText::new(format!("· {f}")).size(14.5).color(TEXT));
        }
    });

    // Quick terrain tags (River, Cliffs, Defense, Move cost, Fresh Water).
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(5.0, 4.0);
        if let Some(mc) = move_cost {
            mini_pill(ui, &format!("{mc} MP"), TEXT_DIM);
        } else {
            mini_pill(ui, "Impassable", BAD);
        }
        if def_bonus > 0 {
            mini_pill(ui, &format!("+{def_bonus} Def"), GOOD);
        }
        if tile.river {
            mini_pill(ui, "River", Color32::from_rgb(90, 185, 250));
        }
        if tile.cliffs {
            mini_pill(ui, "Cliffs", Color32::from_rgb(198, 174, 146));
        }
        if tile.road {
            mini_pill(ui, "Road (1 MP)", Color32::from_rgb(218, 188, 134));
        }
        if !s.game.map.is_water(&s.game.rules, pos) && s.game.is_fresh_water(pos) {
            mini_pill(ui, "Fresh Water", Color32::from_rgb(80, 224, 164));
        }
    });

    // Yield strip.
    let active_yields: Vec<(usize, i32)> = (0..6)
        .filter(|&i| yields[i] != 0)
        .map(|i| (i, yields[i]))
        .collect();
    if !active_yields.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 4.0);
            for (yi, val) in active_yields {
                let col = YIELD_COLORS[yi];
                let (rect, _) = ui.allocate_exact_size(vec2(46.0, 22.0), Sense::hover());
                let p = ui.painter();
                p.rect_filled(rect, 11.0, Color32::from_rgb(18, 30, 48));
                p.rect_stroke(rect, 11.0, Stroke::new(1.0, with_alpha(col, 140)), StrokeKind::Inside);
                glyph::paint(p, theme::yield_glyph(yi), pos2(rect.left() + 12.0, rect.center().y), 6.2, col);
                theme::text_bold(
                    p,
                    pos2(rect.left() + 24.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    format!("{val}"),
                    13.5,
                    TEXT,
                );
            }
        });
    }

    // Resource + actionable improvement hint.
    if let Some(r) = tile.resource {
        let res = &content.resources[r];
        if s.game.players[me].meets(&s.game.rules, res.prereq_tech.as_deref()) {
            let (cls, col) = match res.class.as_str() {
                "RESOURCECLASS_LUXURY" => ("Luxury", Color32::from_rgb(196, 114, 236)),
                "RESOURCECLASS_STRATEGIC" => ("Strategic", Color32::from_rgb(242, 92, 78)),
                _ => ("Bonus", Color32::from_rgb(106, 212, 98)),
            };
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
                let p = ui.painter().clone();
                if !s.icons.paint(&p, "resource", &res.id, rect) {
                    p.circle_filled(rect.center(), 7.0, col);
                }
                ui.label(theme::bold(&res.name, 14.5).color(col));
                mini_pill(ui, cls, col);
            });
            if tile.improvement.is_none() {
                if let Some(imp) = content
                    .improvements
                    .iter()
                    .find(|imp| imp.buildable && imp.valid_resources.contains(&res.id))
                {
                    let extra = if res.class == "RESOURCECLASS_LUXURY" {
                        " (+1 Amenity in 4 cities)"
                    } else if res.class == "RESOURCECLASS_STRATEGIC" {
                        " (unlocks advanced units)"
                    } else {
                        ""
                    };
                    ui.label(
                        RichText::new(format!("Improve with Builder -> {}{extra}", imp.name))
                            .size(12.5)
                            .color(TEXT_DIM),
                    );
                }
            }
        }
    }

    // Improvement / District / Wonder / Tribal Village / Camp.
    if s.game.villages.contains(&pos) {
        ui.label(theme::bold("Tribal Village · Move any unit here for a gift!", 13.5).color(GOLD));
    }
    if s.game.camps.contains_key(&pos) {
        ui.label(theme::bold("Barbarian Camp · Clear for 50 Gold", 13.5).color(BAD));
    }
    if let Some(i) = tile.improvement {
        let name = &content.improvements[i].name;
        if tile.pillaged {
            ui.label(theme::bold(format!("{name} (Pillaged — repair with Builder)"), 13.5).color(BAD));
        } else {
            ui.label(RichText::new(format!("Improvement: {name}")).size(13.5).color(GOOD));
        }
    }
    if let Some(d) = tile.district.filter(|&d| !content.districts[d].city_center) {
        let dname = &content.districts[d].name;
        ui.label(theme::bold(format!("District: {dname}"), 13.5).color(TEAL));
    }
    if let Some(w) = tile.wonder {
        let wname = &content.buildings[w].name;
        ui.label(theme::bold(format!("Wonder: {wname}"), 14.0).color(GOLD));
    }

    // District / Wonder placement feedback (adjacency breakdown + clearing warning).
    if let (Some(item), Some(city_id)) = (&s.placing, s.selected_city) {
        let valid = match item {
            Production::District(d, _) => s.game.district_sites(city_id, *d).contains(&pos),
            Production::Wonder(w, _) => s.game.wonder_sites(city_id, *w).contains(&pos),
            _ => false,
        };
        if valid {
            ui.separator();
            if let Production::District(d, _) = item {
                let parts = s.game.adjacency_breakdown(*d, pos);
                if parts.is_empty() {
                    ui.label(RichText::new("Adjacency: +0 (no adjacent bonuses here)").size(13.0).color(TEXT_DIM));
                } else {
                    ui.label(theme::bold("Adjacency bonuses:", 13.5).color(GOOD));
                    for (yi, amt, reason) in parts {
                        ui.label(
                            RichText::new(format!("  +{amt} {} from {reason}", YIELD_NAMES[yi]))
                                .size(13.0)
                                .color(YIELD_COLORS[yi]),
                        );
                    }
                }
            }
            if let Some(i) = tile.improvement {
                ui.label(
                    RichText::new(format!("Warning: removes existing {}", content.improvements[i].name))
                        .size(12.5)
                        .color(Color32::from_rgb(245, 180, 80)),
                );
            }
            if let Some(f) = tile.feature
                && !content.features[f].natural_wonder
            {
                ui.label(
                    RichText::new(format!("Clears {}", content.features[f].name))
                        .size(12.5)
                        .color(Color32::from_rgb(245, 180, 80)),
                );
            }
        }
    }

    // Settler lens guidance.
    if effective_lens(&s.map_settings, &s.game, s.selected_unit) == MapLens::Settler
        && !s.game.map.is_water(&s.game.rules, pos)
    {
        match s.game.can_settle_pos(me, pos) {
            Ok(5) => {
                ui.label(theme::bold("Fresh Water · Starts with 5 Housing", 13.0).color(GOOD));
            }
            Ok(3) => {
                ui.label(theme::bold("Coastal · Starts with 3 Housing", 13.0).color(TEAL));
            }
            Ok(_) => {
                ui.label(RichText::new("No Water · Starts with 2 Housing").size(13.0).color(GOLD));
            }
            Err(why) => {
                ui.label(RichText::new(format!("Cannot settle: {why}")).size(13.0).color(BAD));
            }
        }
    }

    // Ownership & worked status.
    if let Some(owner) = tile.owner {
        let o = &s.game.players[owner];
        let city_name = tile
            .owning_city
            .and_then(|c| s.game.cities.get(&c))
            .map(|c| c.name.clone())
            .unwrap_or_default();
        let worked = tile
            .owning_city
            .and_then(|c| s.game.cities.get(&c))
            .is_some_and(|c| c.worked.contains(&pos));
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 5.0, rgb(o.primary));
            ui.label(RichText::new(format!("{} ({city_name})", o.civ_name)).size(13.0).color(TEXT_DIM));
            if worked && owner == me {
                mini_pill(ui, "Worked", GOOD);
            }
        });
    }

    // Visible units on this tile.
    if s.game.players[me].visible[s.game.map.grid.index(pos)] {
        let units: Vec<_> = s.game.units_at(pos).cloned().collect();
        if !units.is_empty() {
            ui.separator();
            for u in units {
                let ty = s.game.unit_type(&u);
                let owner = &s.game.players[u.owner];
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                    ui.painter().circle_filled(r.center(), 5.0, rgb(owner.primary));
                    ui.label(theme::bold(&ty.name, 13.5).color(TEXT));
                    ui.label(RichText::new(&owner.civ_name).size(12.5).color(TEXT_DIM));
                    if s.game.is_military(&u) {
                        let s_str = if ty.ranged_combat > 0 {
                            format!("{} / {}r · {} HP", ty.combat, ty.ranged_combat, u.hp)
                        } else {
                            format!("{} Str · {} HP", ty.combat, u.hp)
                        };
                        ui.label(RichText::new(s_str).size(12.5).color(GOLD));
                    } else {
                        ui.label(RichText::new("Civilian").size(12.5).color(TEXT_DIM));
                    }
                });
            }
        }
    }

    // Smart contextual action hint at the bottom.
    if let Some(unit) = s.selected_unit.and_then(|id| s.game.units.get(&id)) {
        if unit.owner == me && pos != unit.pos {
            if s.game.can_attack(unit, pos) {
                ui.label(theme::bold("Right-click to attack", 13.0).color(BAD));
            } else if !s.path_preview.is_empty() {
                let turns = s.game.path_step_turns(unit, &s.path_preview);
                let total = turns.last().copied().unwrap_or(1);
                let txt = if total <= 1 {
                    "Right-click to move here this turn".to_string()
                } else {
                    format!("Right-click to move here ({total} turns)")
                };
                ui.label(RichText::new(txt).size(12.5).color(TEAL));
            }
        }
    } else if s.game.city_at(pos).is_some_and(|c| c.owner == me) {
        ui.label(RichText::new("Left-click to open city").size(12.5).color(TEAL));
    }
}

fn mini_pill(ui: &mut Ui, label: &str, color: Color32) {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), palette::bold(11.5), color);
    let (rect, _) = ui.allocate_exact_size(galley.size() + vec2(12.0, 5.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, rect.height() * 0.5, with_alpha(color, 26));
    p.rect_stroke(
        rect,
        rect.height() * 0.5,
        Stroke::new(1.0, with_alpha(color, 110)),
        StrokeKind::Inside,
    );
    p.galley(rect.center() - galley.size() * 0.5, galley, color);
}

// ------------------------------------------------------------ Combat Preview Card

/// Civ VI-style bottom-center Combat Preview Panel whenever hovering an attackable target.
pub fn combat_preview(s: &mut Session, ctx: &egui::Context) {
    let Some(target) = s.hover else { return };
    if s.windows.any_open() || crate::screens::modal_up(s) {
        return;
    }
    let breakdown: Option<CombatBreakdown> = if s.city_targeting {
        s.selected_city
            .and_then(|cid| s.game.city_combat_breakdown(cid, target))
    } else {
        s.selected_unit.and_then(|uid| {
            let u = s.game.units.get(&uid)?;
            (u.owner == s.me() && s.game.can_attack(u, target))
                .then(|| s.game.combat_breakdown(uid, target))
                .flatten()
        })
    };
    let Some(b) = breakdown else { return };

    let screen = ctx.content_rect();
    let card_w = (screen.width() - 40.0).clamp(440.0, 560.0);
    // On wide monitors sit cleanly between the unit panel and minimap; on
    // narrower/tiled windows float just above the bottom HUD with zero overlap.
    let lift = if screen.width() < 1420.0 {
        super::minimap::total_height(s).max(245.0) + MARGIN + 14.0
    } else {
        MARGIN + 16.0
    };
    let bottom_y = screen.bottom() - lift;
    egui::Area::new(Id::unique("combat_preview"))
        .pivot(Align2::CENTER_BOTTOM)
        .fixed_pos(pos2(screen.center().x, bottom_y))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(ctx, |ui| {
            theme::plate_ui(ui, 12, |ui| {
                ui.set_width(card_w - 24.0);
                // Outcome banner.
                let (verdict, vcol) = combat_verdict(&b);
                let kind_tag = match b.preview.kind {
                    AttackKind::Melee => "MELEE COMBAT",
                    AttackKind::Ranged => "RANGED STRIKE · NO COUNTERATTACK",
                    AttackKind::Bombard => "SIEGE BOMBARD · FULL WALL DAMAGE",
                };
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(kind_tag).font(palette::bold(11.5)).color(TEXT_DIM));
                    ui.label(theme::bold(&verdict, 20.0).color(vcol));
                });
                let y = ui.cursor().top() + 2.0;
                theme::divider(
                    ui.painter(),
                    ui.min_rect().left(),
                    ui.max_rect().right(),
                    y,
                    with_alpha(vcol, 190),
                );
                ui.add_space(8.0);

                // Two columns: Attacker (left) vs Defender (right).
                let col_w = (ui.available_width() - 20.0) * 0.5;
                ui.horizontal_top(|ui| {
                    combat_side(
                        s,
                        ui,
                        col_w,
                        &b.attacker_name,
                        b.attacker_owner,
                        b.attacker_unit_id.as_deref(),
                        b.preview.attack,
                        b.attacker_hp,
                        b.attacker_max_hp,
                        b.preview.damage_to_attacker,
                        0,
                        0,
                        &b.attacker_mods,
                        true,
                    );
                    ui.add_space(6.0);
                    // VS badge in the middle.
                    let (vr, _) = ui.allocate_exact_size(vec2(8.0, 68.0), Sense::hover());
                    ui.painter().line_segment(
                        [pos2(vr.center().x, vr.top() + 4.0), pos2(vr.center().x, vr.bottom() - 4.0)],
                        Stroke::new(1.0, with_alpha(GOLD_DIM, 140)),
                    );
                    ui.add_space(6.0);
                    combat_side(
                        s,
                        ui,
                        col_w,
                        &b.defender_name,
                        b.defender_owner,
                        b.defender_unit_id.as_deref(),
                        b.preview.defense,
                        b.defender_hp,
                        b.defender_max_hp,
                        b.preview.damage_to_defender,
                        b.defender_wall_hp,
                        b.defender_max_walls,
                        &b.defender_mods,
                        false,
                    );
                });
            });
        });
}

fn combat_verdict(b: &CombatBreakdown) -> (String, Color32) {
    if b.is_civilian {
        return ("Capture / Eliminate Civilian".into(), GOOD);
    }
    let kills = b.preview.damage_to_defender >= b.defender_hp && b.defender_wall_hp <= 0;
    let dies = b.preview.damage_to_attacker >= b.attacker_hp;
    if kills && !dies {
        return (
            if b.is_city {
                "City Capture Possible!".into()
            } else {
                "Decisive Victory (Target Eliminated)".into()
            },
            GOOD,
        );
    }
    if dies && !kills {
        return ("Major Defeat (Unit Risked!)".into(), BAD);
    }
    let diff = b.preview.attack - b.preview.defense;
    if b.preview.kind != AttackKind::Melee {
        return ("Safe Ranged Attack".into(), TEAL);
    }
    if diff >= 10 {
        ("Decisive Victory".into(), GOOD)
    } else if diff >= 4 {
        ("Major Victory".into(), Color32::from_rgb(154, 224, 108))
    } else if diff >= 1 {
        ("Minor Victory".into(), GOLD)
    } else if diff == 0 {
        ("Stalemate".into(), GOLD)
    } else if diff >= -4 {
        ("Minor Defeat".into(), Color32::from_rgb(242, 152, 74))
    } else {
        ("Major Defeat".into(), BAD)
    }
}

#[allow(clippy::too_many_arguments)]
fn combat_side(
    s: &mut Session,
    ui: &mut Ui,
    w: f32,
    name: &str,
    owner: usize,
    unit_id: Option<&str>,
    strength: i32,
    hp: i32,
    max_hp: i32,
    dmg: i32,
    wall_hp: i32,
    max_walls: i32,
    mods: &[wayciv_core::combat::CombatMod],
    is_attacker: bool,
) {
    let civ_col = rgb(s.game.players[owner].primary);
    let civ_name = s.game.players[owner].civ_name.clone();
    ui.vertical(|ui| {
        ui.set_width(w);
        ui.horizontal(|ui| {
            // Portrait disc.
            let (rect, _) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::hover());
            let p = ui.painter().clone();
            theme::disc(&p, rect.center(), 18.0, NAVY_RAISED, civ_col);
            let painted = unit_id.is_some_and(|id| {
                s.icons.paint(&p, "unit", id, Rect::from_center_size(rect.center(), vec2(28.0, 28.0)))
            });
            if !painted {
                glyph::paint(
                    &p,
                    if unit_id.is_none() { Glyph::City } else { Glyph::Swords },
                    rect.center(),
                    10.0,
                    TEXT,
                );
            }
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(theme::bold(name, 15.0).color(TEXT));
                ui.label(RichText::new(civ_name).size(12.0).color(TEXT_DIM));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (sr, _) = ui.allocate_exact_size(vec2(42.0, 28.0), Sense::hover());
                let p = ui.painter();
                let col = if is_attacker { GOLD } else { Color32::from_rgb(130, 185, 245) };
                theme::well(p, sr, 14.0, NAVY, col);
                theme::text_bold(p, sr.center(), Align2::CENTER_CENTER, strength, 16.0, col);
            });
        });
        ui.add_space(4.0);
        // Predicted HP bar showing remaining HP vs flashing red damage taken.
        let after = (hp - dmg).max(0);
        let (bar_r, _) = ui.allocate_exact_size(vec2(w, 12.0), Sense::hover());
        let p = ui.painter();
        p.rect_filled(bar_r, 6.0, Color32::from_black_alpha(180));
        let cur_frac = (hp as f32 / max_hp.max(1) as f32).clamp(0.0, 1.0);
        let rem_frac = (after as f32 / max_hp.max(1) as f32).clamp(0.0, 1.0);
        if cur_frac > 0.0 {
            let full = Rect::from_min_size(bar_r.min, vec2(bar_r.width() * cur_frac, bar_r.height())).shrink(1.0);
            p.rect_filled(full, 5.0, BAD);
        }
        if rem_frac > 0.0 {
            let rem = Rect::from_min_size(bar_r.min, vec2(bar_r.width() * rem_frac, bar_r.height())).shrink(1.0);
            let hp_col = if rem_frac > 0.6 { GOOD } else if rem_frac > 0.3 { GOLD } else { Color32::from_rgb(240, 130, 60) };
            p.rect_filled(rem, 5.0, hp_col);
        }
        p.rect_stroke(bar_r, 6.0, Stroke::new(1.0, with_alpha(GOLD_DIM, 130)), StrokeKind::Outside);
        ui.label(
            RichText::new(if dmg > 0 {
                format!("HP: {hp} -> {after}  (-{dmg})")
            } else {
                format!("HP: {hp}/{max_hp}  (no damage)")
            })
            .size(12.5)
            .color(if dmg > 0 { Color32::from_rgb(245, 160, 145) } else { GOOD }),
        );
        if max_walls > 0 && wall_hp > 0 {
            ui.label(
                RichText::new(format!("Walls: {wall_hp}/{max_walls}"))
                    .size(12.0)
                    .color(Color32::from_rgb(130, 185, 245)),
            );
        }
        // Modifier chips.
        if !mods.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 3.0);
                for m in mods.iter().take(4) {
                    let (txt, col) = if m.delta > 0 {
                        (format!("+{} {}", m.delta, m.label), GOOD)
                    } else if m.delta < 0 {
                        (format!("{} {}", m.delta, m.label), BAD)
                    } else {
                        (m.label.clone(), TEXT_DIM)
                    };
                    mini_pill(ui, &txt, col);
                }
            });
        }
    });
}

// ------------------------------------------------------------ Map Lens Legend & Turn-1 Guide

/// Compact top-center legend pill whenever a Map Lens is active, OR Turn-1
/// settle guidance before the capital is founded.
pub fn top_overlays(s: &mut Session, ctx: &egui::Context) {
    if s.windows.big_open() || crate::screens::modal_up(s) || s.placing.is_some() || s.city_targeting {
        return;
    }
    let lens = effective_lens(&s.map_settings, &s.game, s.selected_unit);
    let me = s.me();
    let no_cities = s.game.player_cities(me).next().is_none();
    if lens == MapLens::None && !no_cities {
        return;
    }
    let screen = ctx.content_rect();
    let left = MARGIN + 290.0 + 16.0;
    let right = screen.right()
        - MARGIN
        - if s.selected_city.is_some() {
            super::city_panel::WIDTH + MARGIN
        } else {
            0.0
        };
    if right - left < 240.0 {
        return;
    }
    let cx = (left + right) * 0.5;
    // Stay cleanly below the Leader HUD Ribbon when met leaders are shown.
    let ribbon_clear = if !super::leaders::met(s).is_empty() {
        112.0
    } else {
        0.0
    };
    let mut y = TOP_BAR + MARGIN + ribbon_clear;

    if no_cities {
        let shown = egui::Area::new(Id::unique("turn1_guide"))
            .pivot(Align2::CENTER_TOP)
            .fixed_pos(pos2(cx, y))
            .order(egui::Order::Middle)
            .interactable(false)
            .show(ctx, |ui| {
                theme::plate_ui(ui, 10, |ui| {
                    ui.set_max_width((right - left - 24.0).clamp(240.0, 560.0));
                    ui.horizontal_wrapped(|ui| {
                        let (r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                        glyph::paint(ui.painter(), Glyph::Star, r.center(), 8.5, GOLD);
                        ui.label(
                            theme::bold("Found Your Capital:", 14.5).color(GOLD),
                        );
                        ui.label(
                            RichText::new("Green tiles have Fresh Water (+5 Housing). Stars mark top spots. Press B to settle.")
                                .size(13.5)
                                .color(TEXT),
                        );
                    });
                });
            });
        y += shown.response.rect.height() + 6.0;
    }

    if lens != MapLens::None {
        let manual = s.map_settings.lens != MapLens::None;
        let mut clear_lens = false;
        egui::Area::new(Id::unique("lens_legend"))
            .pivot(Align2::CENTER_TOP)
            .fixed_pos(pos2(cx, y))
            .order(egui::Order::Middle)
            .show(ctx, |ui| {
                theme::plate_ui(ui, 8, |ui| {
                    ui.set_max_width((right - left - 24.0).clamp(220.0, 560.0));
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(8.0, 4.0);
                        ui.label(theme::bold(lens.name(), 13.5).color(GOLD));
                        match lens {
                            MapLens::Settler => {
                                swatch_label(ui, Color32::from_rgb(56, 214, 118), "Fresh Water (+5)");
                                swatch_label(ui, Color32::from_rgb(60, 196, 224), "Coast (+3)");
                                swatch_label(ui, Color32::from_rgb(218, 184, 92), "Dry (+2)");
                                swatch_label(ui, Color32::from_rgb(220, 66, 60), "Blocked");
                            }
                            MapLens::Empire => {
                                ui.label(RichText::new("Civilization borders & territory").size(12.5).color(TEXT_DIM));
                            }
                            MapLens::Religion => {
                                ui.label(RichText::new("Cities & land shaded by majority faith").size(12.5).color(TEXT_DIM));
                            }
                            MapLens::Resources => {
                                swatch_label(ui, Color32::from_rgb(106, 212, 98), "Bonus");
                                swatch_label(ui, Color32::from_rgb(196, 114, 236), "Luxury");
                                swatch_label(ui, Color32::from_rgb(242, 92, 78), "Strategic");
                                swatch_label(ui, GOLD, "Village");
                            }
                            MapLens::None => {}
                        }
                        if manual && widgets::tool_button(ui, 22.0, widgets::Tool::Close, false, "Close lens", "Return to normal map view", Some("Esc")) {
                            clear_lens = true;
                        }
                    });
                });
            });
        if clear_lens {
            s.map_settings.lens = MapLens::None;
        }
    }
}

fn swatch_label(ui: &mut Ui, color: Color32, text: &str) {
    let (r, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
    ui.painter().circle_filled(r.center(), 5.0, color);
    ui.label(RichText::new(text).size(12.5).color(TEXT));
}

// ------------------------------------------------------------ Settings & Controls Modal

/// In-game Settings, Graphics, and Controls Reference window (Gear icon / F1).
pub fn settings_window(s: &mut Session, ui: &mut Ui) -> bool {
    let mut close = false;
    ui.spacing_mut().item_spacing.y = 8.0;

    theme::heading(ui, "Display & Map Graphics");
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
        if theme::toggle_pill(ui, "Tile Yields (Y)", s.map_settings.show_yields)
            .on_hover_text("Show Food, Production, Gold, Science, Culture, and Faith icons directly on map tiles.")
            .clicked()
        {
            s.map_settings.show_yields = !s.map_settings.show_yields;
        }
        if theme::toggle_pill(ui, "Resource Icons", s.map_settings.show_resources)
            .on_hover_text("Show resource badges and models on the map.")
            .clicked()
        {
            s.map_settings.show_resources = !s.map_settings.show_resources;
        }
        if theme::toggle_pill(ui, "Atmosphere & Clouds", s.map_settings.atmosphere)
            .on_hover_text("Drifting cloud shadows and golden sunlight lighting pass.")
            .clicked()
        {
            s.map_settings.atmosphere = !s.map_settings.atmosphere;
        }
    });
    ui.horizontal(|ui| {
        ui.label(RichText::new("Hex Grid:").color(TEXT_DIM));
        for mode in [
            crate::render::overlay::GridMode::Subtle,
            crate::render::overlay::GridMode::Crisp,
            crate::render::overlay::GridMode::Off,
        ] {
            if theme::toggle_pill(ui, mode.label(), s.map_settings.grid == mode).clicked() {
                s.map_settings.grid = mode;
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label(RichText::new("Map Lens:").color(TEXT_DIM));
        for lens in [
            MapLens::None,
            MapLens::Settler,
            MapLens::Empire,
            MapLens::Religion,
            MapLens::Resources,
        ] {
            if theme::toggle_pill(ui, lens.name(), s.map_settings.lens == lens).clicked() {
                s.map_settings.lens = lens;
            }
        }
    });

    theme::heading(ui, "Controls & Shortcuts");
    egui::Grid::new("controls_ref")
        .num_columns(4)
        .spacing([14.0, 5.0])
        .show(ui, |ui| {
            let pairs = [
                ("Left-Click", "Select unit / city / banner", "Right-Click", "Move or attack"),
                ("Drag / Arrows", "Pan the camera", "Mouse Wheel / + -", "Zoom in / out"),
                ("Enter", "Next decision / End Turn", "U or .", "Next unit needing orders"),
                ("B", "Found City (Settler)", "F / Z / Space", "Fortify / Sleep / Skip"),
                ("Y", "Toggle Tile Yields", "H", "Cycle Hex Grid style"),
                ("4 / 5 / 6 / 7", "Settler / Empire / Religion / Res Lens", "S / A", "Tech Tree / AI Tree"),
                ("G / R / P / E", "Gov / Religion / Great People / CS", "L / N / V / T", "Leaders / Alerts / Victory / Eras"),
                ("K", "Map Key", "Esc", "Cancel / Close window"),
            ];
            for (k1, d1, k2, d2) in pairs {
                ui.label(theme::bold(k1, 13.5).color(GOLD));
                ui.label(RichText::new(d1).size(13.0).color(TEXT));
                ui.label(theme::bold(k2, 13.5).color(GOLD));
                ui.label(RichText::new(d2).size(13.0).color(TEXT));
                ui.end_row();
            }
        });

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        if theme::pill_button(ui, Some(Glyph::Chevrons), "Resume Game", TEAL, 160.0).clicked() {
            close = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let key = ui.scope_id().with("confirm_quit");
            let now = ui.input(|i| i.time);
            let armed = ui
                .data(|d| d.get_temp::<f64>(key))
                .is_some_and(|t| now - t < 4.0);
            let label = if armed { "Click again to Main Menu" } else { "Return to Main Menu" };
            if theme::pill_button(ui, Some(Glyph::Cross), label, BAD, 190.0).clicked() {
                if armed {
                    s.quit_to_menu = true;
                    close = true;
                } else {
                    ui.data_mut(|d| d.insert_temp(key, now));
                }
            }
        });
    });
    close
}

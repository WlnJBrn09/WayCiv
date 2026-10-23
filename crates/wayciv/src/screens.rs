//! Full-screen-style windows: tech and civic trees, government, religion,
//! great people, city-states, victory, diplomacy, and leader pop-ups.

use crate::hud::Session;
use crate::palette::{GOLD, YIELD_COLORS, rgb};
use crate::portraits::Portraits;
use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, vec2};
use std::collections::HashMap;
use wayciv_core::diplomacy::DiploEvent;
use wayciv_core::research::Tree;
use wayciv_core::{Game, PlayerId};

#[derive(Default)]
pub struct Windows {
    pub tech: bool,
    pub civic: bool,
    pub ai: bool,
    pub government: bool,
    pub religion: bool,
    pub great_people: bool,
    pub city_states: bool,
    pub victory: bool,
    pub legend: bool,
    pub diplomacy: Option<PlayerId>,
    pub policy_draft: Option<Vec<usize>>,
}

impl Windows {
    pub fn close_all(&mut self) {
        let legend = self.legend;
        *self = Windows { legend, ..Default::default() };
    }
}

pub fn windows(s: &mut Session, portraits: &mut Portraits, ctx: &egui::Context) {
    for tree in [Tree::Tech, Tree::Civic, Tree::Ai] {
        let mut open = match tree {
            Tree::Tech => s.windows.tech,
            Tree::Civic => s.windows.civic,
            Tree::Ai => s.windows.ai,
        };
        if open {
            let title = match tree {
                Tree::Tech => "Technology Tree",
                Tree::Civic => "Civics Tree",
                Tree::Ai => "Artificial Intelligence Tree",
            };
            let mut picked = false;
            egui::Window::new(title).open(&mut open).default_size([1100.0, 620.0]).show(ctx, |ui| picked = research_tree(s, ui, tree));
            // Choosing something closes the tree.
            open &= !picked;
            match tree {
                Tree::Tech => s.windows.tech &= open,
                Tree::Civic => s.windows.civic &= open,
                Tree::Ai => s.windows.ai &= open,
            }
        }
    }
    let mut open = s.windows.government;
    if open {
        egui::Window::new("Government").open(&mut open).default_size([760.0, 560.0]).show(ctx, |ui| government(s, ui));
        s.windows.government &= open;
    }
    let mut open = s.windows.religion;
    if open {
        egui::Window::new("Religion").open(&mut open).default_size([620.0, 520.0]).show(ctx, |ui| religion(s, ui));
        s.windows.religion &= open;
    }
    let mut open = s.windows.great_people;
    if open {
        egui::Window::new("Great People").open(&mut open).default_size([620.0, 420.0]).show(ctx, |ui| great_people(s, ui));
        s.windows.great_people &= open;
    }
    let mut open = s.windows.city_states;
    if open {
        egui::Window::new("City-States").open(&mut open).default_size([620.0, 420.0]).show(ctx, |ui| city_states(s, ui));
        s.windows.city_states &= open;
    }
    let mut open = s.windows.victory;
    if open {
        egui::Window::new("Victory Progress").open(&mut open).default_size([620.0, 420.0]).show(ctx, |ui| victory(s, ui));
        s.windows.victory &= open;
    }
    if let Some(other) = s.windows.diplomacy {
        let mut open = true;
        let name = s.game.players[other].name.clone();
        egui::Window::new(name).open(&mut open).default_size([760.0, 520.0]).collapsible(false).show(ctx, |ui| diplomacy(s, portraits, ui, other));
        if !open {
            s.windows.diplomacy = None;
        }
    }
    if let Some((other, event)) = s.popup {
        leader_popup(s, portraits, ctx, other, event);
    }
}

/// Columns: depth in the prerequisite graph. Rows: Civ VI's own UITreeRow.
fn layout(items: &[wayciv_core::rules::Research]) -> HashMap<usize, (i32, i32)> {
    let mut depth: HashMap<usize, i32> = HashMap::new();
    fn d(i: usize, items: &[wayciv_core::rules::Research], memo: &mut HashMap<usize, i32>) -> i32 {
        if let Some(&v) = memo.get(&i) {
            return v;
        }
        let v = items[i].prereqs.iter().filter_map(|p| items.iter().position(|x| &x.id == p)).map(|j| d(j, items, memo) + 1).max().unwrap_or(0);
        memo.insert(i, v);
        v
    }
    for i in 0..items.len() {
        d(i, items, &mut depth);
    }
    let mut out = HashMap::new();
    let mut used: HashMap<(i32, i32), bool> = HashMap::new();
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by_key(|&i| (depth[&i], items[i].tree_row));
    for i in order {
        let col = depth[&i];
        let mut row = items[i].tree_row;
        while used.contains_key(&(col, row)) {
            row += 1;
        }
        used.insert((col, row), true);
        out.insert(i, (col, row));
    }
    out
}

/// Things a tech or civic unlocks, for its tooltip.
fn unlocks(game: &Game, tree: Tree, id: &str) -> Vec<String> {
    let content = &game.rules.content;
    let matches = |tech: &Option<String>, civic: &Option<String>| match tree {
        Tree::Tech => tech.as_deref() == Some(id),
        Tree::Civic => civic.as_deref() == Some(id),
        Tree::Ai => false,
    };
    let mut out: Vec<String> = Vec::new();
    out.extend(content.units.iter().filter(|u| u.unique_to.is_none() && matches(&u.prereq_tech, &u.prereq_civic)).map(|u| u.name.clone()));
    out.extend(content.buildings.iter().filter(|b| b.unique_to.is_none() && matches(&b.prereq_tech, &b.prereq_civic)).map(|b| b.name.clone()));
    out.extend(content.districts.iter().filter(|d| d.unique_to.is_none() && matches(&d.prereq_tech, &d.prereq_civic)).map(|d| d.name.clone()));
    out.extend(content.governments.iter().filter(|g| matches(&None, &g.prereq_civic)).map(|g| g.name.clone()));
    if tree == Tree::Tech && id == wayciv_core::additions::TECH_AI {
        out.push("the AI tree".into());
    }
    out
}

/// Draws a tree; returns true when the player picked something to research.
fn research_tree(s: &mut Session, ui: &mut egui::Ui, tree: Tree) -> bool {
    let me = s.me();
    let mut picked = false;
    let items = s.game.tree_items(tree).to_vec();
    let positions = layout(&items);
    let min_row = positions.values().map(|p| p.1).min().unwrap_or(0);
    let max_col = positions.values().map(|p| p.0).max().unwrap_or(0);
    let max_row = positions.values().map(|p| p.1).max().unwrap_or(0);
    let (w, h, gx, gy) = (230.0, 58.0, 270.0, 72.0);
    let current = s.game.current_research(me, tree);
    let accent = match tree {
        Tree::Tech => YIELD_COLORS[3],
        Tree::Civic => YIELD_COLORS[4],
        Tree::Ai => crate::palette::COMPUTE,
    };
    let (icon_kind, tree_icon) = match tree {
        Tree::Tech => ("tech", "TREE_TECH"),
        Tree::Civic => ("civic", "TREE_CIVIC"),
        Tree::Ai => ("ai", "TREE_AI"),
    };
    ui.horizontal(|ui| {
        s.icons.show(ui, "tree", tree_icon, 36.0);
        ui.vertical(|ui| {
            ui.label("Click an available item to research it. Gold: current. Bright: available. Green: known.");
            if tree == Tree::Ai {
                let compute = s.game.compute(me);
                if !s.game.ai_tree_unlocked(me) {
                    ui.label(RichText::new("Research Artificial Intelligence to open this tree.").color(GOLD));
                } else {
                    ui.label(RichText::new(format!("Compute +{compute:.0}/turn. Datacenters produce Compute: +6 each, +2 per adjacent district, +3 next to a river or the coast.")).color(crate::palette::COMPUTE));
                }
            }
        });
    });
    egui::ScrollArea::both().show(ui, |ui| {
        let size = vec2(gx * (max_col + 1) as f32 + 20.0, gy * (max_row - min_row + 1) as f32 + 20.0);
        let (area, _) = ui.allocate_exact_size(size, Sense::hover());
        let origin = area.min + vec2(10.0, 10.0);
        let node_rect = |i: usize| {
            let (c, r) = positions[&i];
            Rect::from_min_size(origin + vec2(c as f32 * gx, (r - min_row) as f32 * gy), vec2(w, h))
        };
        let painter = ui.painter().clone();
        for (i, item) in items.iter().enumerate() {
            for p in &item.prereqs {
                if let Some(j) = items.iter().position(|x| &x.id == p) {
                    let (a, b) = (node_rect(j), node_rect(i));
                    let from = a.right_center();
                    let to = b.left_center();
                    let mid = Pos2::new((from.x + to.x) / 2.0, from.y);
                    painter.line_segment([from, mid], Stroke::new(1.5, Color32::from_gray(90)));
                    painter.line_segment([mid, Pos2::new(mid.x, to.y)], Stroke::new(1.5, Color32::from_gray(90)));
                    painter.line_segment([Pos2::new(mid.x, to.y), to], Stroke::new(1.5, Color32::from_gray(90)));
                }
            }
        }
        for (i, item) in items.iter().enumerate() {
            let rect = node_rect(i);
            let known = s.game.known(me, tree, i);
            let available = s.game.researchable(me, tree, i);
            let boosted = match tree {
                Tree::Tech => s.game.players[me].tech_boosted[i],
                Tree::Civic => s.game.players[me].civic_boosted[i],
                Tree::Ai => false,
            };
            let fill = if Some(i) == current {
                Color32::from_rgb(110, 85, 30)
            } else if known {
                Color32::from_rgb(40, 80, 50)
            } else if available {
                Color32::from_rgb(40, 60, 95)
            } else {
                Color32::from_rgb(35, 38, 46)
            };
            let response = ui.interact(rect, ui.scope_id().with(("node", i)), Sense::click());
            painter.rect_filled(rect, 6.0, fill);
            painter.rect_stroke(rect, 6.0, Stroke::new(if response.hovered() { 2.0 } else { 1.0 }, if Some(i) == current { GOLD } else { accent.gamma_multiply(0.6) }), StrokeKind::Inside);
            let icon = Rect::from_min_size(rect.min + vec2(5.0, 5.0), vec2(h - 10.0, h - 10.0));
            let text_x = if s.icons.paint(&painter, icon_kind, &item.id, icon) { h } else { 8.0 };
            painter.text(rect.min + vec2(text_x, 6.0), Align2::LEFT_TOP, &item.name, FontId::proportional(15.0), Color32::WHITE);
            let cost = s.game.research_cost(tree, i);
            let status = if known { "Known".into() } else { format!("{cost} ({})", s.research_turns(tree, i)) };
            painter.text(rect.min + vec2(text_x, 26.0), Align2::LEFT_TOP, status, FontId::proportional(12.0), Color32::from_gray(190));
            if item.boost.is_some() {
                let mark = if boosted { "Boosted!" } else { "Boost: hover" };
                painter.text(rect.min + vec2(text_x, 41.0), Align2::LEFT_TOP, mark, FontId::proportional(11.0), if boosted { GOLD } else { Color32::from_gray(140) });
            }
            let progress = s.game.progress(me, tree, i);
            if !known && progress > 0 {
                let bar = Rect::from_min_size(rect.left_bottom() + vec2(0.0, -4.0), vec2(w * (progress as f32 / cost.max(1) as f32).min(1.0), 4.0));
                painter.rect_filled(bar, 1.0, accent);
            }
            let mut tip = item.name.clone();
            if tree == Tree::Ai {
                if let Some(t) = wayciv_core::additions::AI_TREE.iter().find(|t| t.id == item.id) {
                    tip += &format!("\n{}", t.description);
                }
            } else {
                let unlocked = unlocks(&s.game, tree, &item.id);
                if !unlocked.is_empty() {
                    tip += &format!("\nUnlocks: {}", unlocked.join(", "));
                }
            }
            if let Some(b) = &item.boost {
                tip += &format!("\nBoost (50%): {b}");
            }
            let response = response.on_hover_text(tip);
            if response.clicked() && available {
                let r = s.game.set_research(me, tree, i);
                picked = r.is_ok();
                s.result(r);
            }
        }
    });
    picked
}

fn government(s: &mut Session, ui: &mut egui::Ui) {
    let me = s.me();
    let content = s.game.rules.content.clone_lists();
    let current = s.game.players[me].government;
    ui.label(RichText::new("Government").heading().color(GOLD));
    let available = s.game.available_governments(me);
    if available.is_empty() {
        ui.label("Research the Code of Laws civic to adopt your first government (Chiefdom).");
        return;
    }
    ui.horizontal_wrapped(|ui| {
        for g in available {
            let gov = &content.governments[g];
            let selected = current == Some(g);
            let slots: Vec<String> = gov.slots.iter().map(|(t, n)| format!("{n} {}", slot_name(t))).collect();
            let response = ui.selectable_label(selected, format!("{}\n{}", gov.name, slots.join(", "))).on_hover_text(&gov.bonus);
            if response.clicked() && !selected {
                let r = s.game.set_government(me, g);
                s.result(r);
                s.windows.policy_draft = None;
            }
        }
    });
    let Some(g) = s.game.players[me].government else { return };
    ui.label(RichText::new(&content.governments[g].bonus).italics());
    ui.separator();
    let draft = s.windows.policy_draft.get_or_insert_with(|| s.game.players[me].policies.clone()).clone();
    let slots = s.game.policy_slots(me);
    let free = s.game.players[me].policy_change_free;
    ui.label(format!(
        "Slots: {}.  {}",
        slots.iter().map(|(t, n)| format!("{n} {}", slot_name(t))).collect::<Vec<_>>().join(", "),
        if free { "Changing policies is free this turn." } else { "Changes cost 25 Gold each until your next civic." }
    ));
    let available = s.game.available_policies(me);
    let mut next = draft.clone();
    egui::ScrollArea::vertical().max_height(ui.available_height() - 40.0).show(ui, |ui| {
        for kind in ["SLOT_MILITARY", "SLOT_ECONOMIC", "SLOT_DIPLOMATIC", "SLOT_GREAT_PERSON"] {
            let list: Vec<usize> = available.iter().copied().filter(|&p| content.policies[p].slot == kind).collect();
            if list.is_empty() {
                continue;
            }
            ui.label(RichText::new(slot_name(kind)).strong().color(GOLD));
            for p in list {
                let policy = &content.policies[p];
                let mut on = next.contains(&p);
                let active = s.game.rules.content.modifiers_of(&policy.id).any(wayciv_core::effects::is_active);
                let tag = if active { "" } else { "  (description only)" };
                if ui.checkbox(&mut on, format!("{}{tag}", policy.name)).on_hover_text(&policy.description).changed() {
                    if on {
                        next.push(p);
                    } else {
                        next.retain(|&x| x != p);
                    }
                }
            }
        }
    });
    s.windows.policy_draft = Some(next.clone());
    if ui.button(RichText::new("Confirm Policies").strong()).clicked() {
        match s.game.set_policies(me, next) {
            Ok(()) => {
                s.game.players[me].policy_change_free = false;
                s.toast("Policies updated.");
            }
            Err(e) => s.toast(e),
        }
    }
}

fn slot_name(slot: &str) -> &'static str {
    match slot {
        "SLOT_MILITARY" => "Military",
        "SLOT_ECONOMIC" => "Economic",
        "SLOT_DIPLOMATIC" => "Diplomatic",
        "SLOT_GREAT_PERSON" => "Great Person",
        "SLOT_WILDCARD" => "Wildcard",
        _ => "Other",
    }
}

fn religion(s: &mut Session, ui: &mut egui::Ui) {
    let me = s.me();
    let p = s.game.players[me].clone();
    let content = s.game.rules.content.clone_lists();
    ui.horizontal(|ui| {
        s.icons.show(ui, "tree", "TREE_RELIGION", 36.0);
        ui.label(format!("Faith: {}", p.faith));
    });
    match p.pantheon {
        Some(b) => {
            ui.horizontal(|ui| {
                s.icons.show(ui, "belief", &content.beliefs[b].id, 40.0);
                ui.label(RichText::new(format!("Pantheon: {}", content.beliefs[b].name)).color(GOLD));
            });
            ui.label(&content.beliefs[b].description);
        }
        None => {
            ui.label(RichText::new(format!("Choose a pantheon belief ({} Faith).", wayciv_core::religion::PANTHEON_COST)).color(GOLD));
            let can = s.game.can_choose_pantheon(me);
            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                for b in s.game.beliefs_of_class("BELIEF_CLASS_PANTHEON") {
                    ui.horizontal(|ui| {
                        s.icons.show(ui, "belief", &content.beliefs[b].id, 28.0);
                        if ui.add_enabled(can, egui::Button::new(&content.beliefs[b].name)).clicked() {
                            let r = s.game.choose_pantheon(me, b);
                            s.result(r);
                        }
                        ui.label(RichText::new(&content.beliefs[b].description).small());
                    });
                }
            });
        }
    }
    ui.separator();
    match p.religion {
        Some(r) => {
            ui.label(RichText::new(format!("You founded {}.", content.religions[r].name)).color(GOLD));
            for &b in &p.beliefs {
                ui.horizontal(|ui| {
                    s.icons.show(ui, "belief", &content.beliefs[b].id, 28.0);
                    ui.label(format!("{}: {}", content.beliefs[b].name, content.beliefs[b].description));
                });
            }
        }
        None => {
            ui.label("Earn a Great Prophet (Holy Site districts generate Prophet points), then use it on a Holy Site to found a religion.");
        }
    }
    ui.separator();
    ui.label(RichText::new("Religions of the world").strong());
    for (&r, &(founder, _)) in &s.game.religions {
        let cities = s.game.cities.values().filter(|c| c.majority_religion() == Some(r)).count();
        ui.label(format!("{} - founded by {}, majority in {cities} cities", content.religions[r].name, s.game.players[founder].civ_name));
    }
}

fn great_people(s: &mut Session, ui: &mut egui::Ui) {
    let me = s.me();
    let rates: HashMap<usize, i32> = s.game.great_person_rates(me).into_iter().collect();
    let content = &s.game.rules.content;
    egui::Grid::new("gp").striped(true).num_columns(4).show(ui, |ui| {
        ui.strong("Class");
        ui.strong("Points");
        ui.strong("Next");
        ui.strong("Effect");
        ui.end_row();
        for (class, gpc) in content.great_person_classes.iter().enumerate() {
            let points = s.game.players[me].great_person_points.get(&class).copied().unwrap_or(0);
            let rate = rates.get(&class).copied().unwrap_or(0);
            let next = s.game.next_great_person(class);
            ui.label(&gpc.name);
            ui.label(format!("{points}/{} (+{rate})", s.game.great_person_cost(class)));
            ui.label(next.map(|g| content.great_people[g].name.clone()).unwrap_or("-".into()));
            ui.label(next.map(|g| s.game.great_person_effect(g)).unwrap_or_default());
            ui.end_row();
        }
    });
}

fn city_states(s: &mut Session, ui: &mut egui::Ui) {
    let me = s.me();
    ui.label(format!("Envoys available: {}", s.game.players[me].envoys_available));
    ui.label("1 envoy: +2 of the city-state's yield in your capital. 3 and 6 envoys: +1 in every city. Suzerain (most envoys, at least 3): +1 more.");
    let states: Vec<PlayerId> = s.game.players.iter().filter(|p| p.is_city_state() && s.game.players[me].relations[p.id].met).map(|p| p.id).collect();
    egui::Grid::new("cs").striped(true).num_columns(5).show(ui, |ui| {
        for cs in states {
            let p = s.game.players[cs].clone();
            ui.label(RichText::new(&p.name).color(rgb(p.primary).gamma_multiply(1.4)).strong());
            ui.label(p.city_state_category.to_lowercase());
            ui.label(format!("Your envoys: {}", s.game.players[me].envoys.get(&cs).copied().unwrap_or(0)));
            ui.label(match s.game.suzerain(cs) {
                Some(x) => format!("Suzerain: {}", s.game.players[x].civ_name),
                None => "No suzerain".into(),
            });
            if !p.alive {
                ui.label("Conquered");
            } else if ui.add_enabled(s.game.players[me].envoys_available > 0, egui::Button::new("Send Envoy")).clicked() {
                let r = s.game.send_envoy(me, cs);
                s.result(r);
            }
            ui.end_row();
        }
    });
}

fn victory(s: &mut Session, ui: &mut egui::Ui) {
    let me = s.me();
    for (name, progress) in s.game.victory_progress(me) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(name).strong().color(GOLD));
            ui.label(progress);
        });
    }
    ui.separator();
    ui.label(RichText::new("Scores").strong());
    let mut players: Vec<&wayciv_core::state::Player> = s.game.alive_majors().filter(|p| p.id == me || s.game.players[me].relations[p.id].met).collect();
    players.sort_by_key(|p| std::cmp::Reverse(s.game.score(p.id)));
    for p in players {
        ui.label(format!("{} ({}) - {}", p.civ_name, p.name, s.game.score(p.id)));
    }
}

fn diplomacy(s: &mut Session, portraits: &mut Portraits, ui: &mut egui::Ui, other: PlayerId) {
    let me = s.me();
    let o = s.game.players[other].clone();
    let leader = s.game.rules.content.leaders[o.leader].clone();
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(360.0, 360.0), Sense::hover());
        portraits.paint(ui, rect, &leader.id, &leader.name, (o.primary, o.secondary));
        ui.vertical(|ui| {
            ui.heading(RichText::new(&leader.name).color(GOLD).size(26.0));
            ui.label(RichText::new(&o.civ_name).size(18.0));
            ui.add_space(6.0);
            ui.label(RichText::new(format!("\"{}\"", s.game.dialogue(other, DiploEvent::Greeting))).italics().size(15.0));
            ui.add_space(6.0);
            let r = &s.game.players[me].relations[other];
            let status = if r.at_war { "At war" } else if r.friendship { "Declared friends" } else { "At peace" };
            let opinion = s.game.opinion(other, me);
            let mood = match opinion {
                o if o >= 25 => "Friendly",
                o if o >= -10 => "Neutral",
                _ => "Unfriendly",
            };
            ui.label(format!("{status}. Their attitude: {mood}."));
            if let Some((name, desc)) = &leader.agenda {
                ui.label(RichText::new(format!("Agenda: {name}")).strong());
                ui.label(desc);
            }
            for t in &leader.traits {
                ui.label(RichText::new(&t.name).strong().color(GOLD)).on_hover_text(&t.description);
            }
        });
    });
    ui.separator();
    let at_war = s.game.players[me].at_war_with(other);
    ui.horizontal(|ui| {
        if !at_war {
            if !s.game.players[me].relations[other].friendship && ui.button("Declare Friendship").clicked() {
                let accepted = s.game.propose_friendship(me, other);
                let line = s.game.dialogue(other, if accepted { DiploEvent::AcceptFriendFromHuman } else { DiploEvent::RejectFriendFromHuman });
                s.toast(format!("{}: \"{line}\"", leader.name));
            }
            if !s.game.players[me].relations[other].denounced && ui.button("Denounce").clicked() {
                s.game.denounce(me, other);
                let line = s.game.dialogue(other, DiploEvent::DenounceFromHuman);
                s.toast(format!("{}: \"{line}\"", leader.name));
            }
            if ui.button(RichText::new("Declare War").color(Color32::from_rgb(240, 90, 80))).clicked() {
                s.game.declare_war(me, other);
                let line = s.game.dialogue(other, DiploEvent::DeclareWarFromHuman);
                s.toast(format!("{}: \"{line}\"", leader.name));
            }
        } else {
            let can = s.game.can_make_peace(me, other);
            if ui.add_enabled(can, egui::Button::new("Propose Peace")).on_disabled_hover_text("Wars last at least 10 turns.").clicked() {
                let accepted = s.game.propose_peace(me, other);
                let line = s.game.dialogue(other, if accepted { DiploEvent::MakePeaceAccept } else { DiploEvent::MakePeaceRefuse });
                s.toast(format!("{}: \"{line}\"", leader.name));
            }
        }
    });
}

fn leader_popup(s: &mut Session, portraits: &mut Portraits, ctx: &egui::Context, other: PlayerId, event: DiploEvent) {
    let me = s.me();
    let o = s.game.players[other].clone();
    let leader = s.game.rules.content.leaders[o.leader].clone();
    let line = s.game.dialogue(other, event);
    let mut close = false;
    egui::Modal::new(egui::Id::unique("leader_popup")).show(ctx, |ui| {
        ui.set_width(820.0);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(420.0, 420.0), Sense::hover());
            portraits.paint(ui, rect, &leader.id, &leader.name, (o.primary, o.secondary));
            ui.vertical(|ui| {
                ui.heading(RichText::new(&leader.name).color(GOLD).size(28.0));
                ui.label(RichText::new(&o.civ_name).size(18.0));
                ui.add_space(12.0);
                ui.label(RichText::new(format!("\"{line}\"")).italics().size(17.0));
                ui.add_space(16.0);
                match event {
                    DiploEvent::DeclareFriendFromAi => {
                        ui.label(format!("{} proposes a declaration of friendship.", leader.name));
                        ui.horizontal(|ui| {
                            if ui.button("Accept").clicked() {
                                s.game.propose_friendship(other, me);
                                close = true;
                            }
                            if ui.button("Decline").clicked() {
                                close = true;
                            }
                        });
                    }
                    DiploEvent::DeclareWarFromAi => {
                        if ui.button(RichText::new("So be it.").size(16.0)).clicked() {
                            close = true;
                        }
                    }
                    _ => {
                        if ui.button(RichText::new("Farewell").size(16.0)).clicked() {
                            close = true;
                        }
                    }
                }
            });
        });
    });
    if close {
        s.popup = None;
    }
}

/// Helper so screens can borrow content lists while mutating the game.
trait CloneLists {
    fn clone_lists(&self) -> Lists;
}

pub struct Lists {
    pub governments: Vec<civ6_data::content::Government>,
    pub policies: Vec<civ6_data::content::Policy>,
    pub beliefs: Vec<civ6_data::content::Belief>,
    pub religions: Vec<civ6_data::content::Religion>,
}

impl CloneLists for civ6_data::content::Content {
    fn clone_lists(&self) -> Lists {
        Lists { governments: self.governments.clone(), policies: self.policies.clone(), beliefs: self.beliefs.clone(), religions: self.religions.clone() }
    }
}

#[allow(dead_code)]
fn unused(_: &Game) {}

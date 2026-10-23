//! Full-screen-style windows: tech and civic trees, government, religion,
//! great people, city-states, victory, diplomacy, and leader pop-ups.

use crate::hud::Session;
use crate::palette::{self, GOLD, YIELD_COLORS, rgb};
use crate::portraits::Portraits;
use crate::ui::glyph::Glyph;
use crate::ui::{self, TOP_BAR, theme};
use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, pos2, vec2};
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
    pub leaders: bool,
    pub notifications: bool,
    pub legend: bool,
    pub diplomacy: Option<PlayerId>,
    pub policy_draft: Option<Vec<usize>>,
}

/// A window or panel the player can open. Only one is open at a time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Panel {
    Tech,
    Civic,
    Ai,
    Government,
    Religion,
    GreatPeople,
    CityStates,
    Victory,
    Leaders,
    Notifications,
    Diplomacy(PlayerId),
}

impl Windows {
    pub fn close_all(&mut self) {
        let legend = self.legend;
        *self = Windows {
            legend,
            ..Default::default()
        };
    }

    pub fn any_open(&self) -> bool {
        self.tech
            || self.civic
            || self.ai
            || self.government
            || self.religion
            || self.great_people
            || self.city_states
            || self.victory
            || self.leaders
            || self.notifications
            || self.diplomacy.is_some()
    }

    pub fn is_open(&self, panel: Panel) -> bool {
        match panel {
            Panel::Tech => self.tech,
            Panel::Civic => self.civic,
            Panel::Ai => self.ai,
            Panel::Government => self.government,
            Panel::Religion => self.religion,
            Panel::GreatPeople => self.great_people,
            Panel::CityStates => self.city_states,
            Panel::Victory => self.victory,
            Panel::Leaders => self.leaders,
            Panel::Notifications => self.notifications,
            Panel::Diplomacy(p) => self.diplomacy == Some(p),
        }
    }

    /// Opens `panel` and closes every other window.
    pub fn open(&mut self, panel: Panel) {
        self.close_all();
        match panel {
            Panel::Tech => self.tech = true,
            Panel::Civic => self.civic = true,
            Panel::Ai => self.ai = true,
            Panel::Government => self.government = true,
            Panel::Religion => self.religion = true,
            Panel::GreatPeople => self.great_people = true,
            Panel::CityStates => self.city_states = true,
            Panel::Victory => self.victory = true,
            Panel::Leaders => self.leaders = true,
            Panel::Notifications => self.notifications = true,
            Panel::Diplomacy(p) => self.diplomacy = Some(p),
        }
    }

    /// Closes `panel` if it is open, otherwise opens it alone.
    pub fn toggle(&mut self, panel: Panel) {
        if self.is_open(panel) {
            self.close_all();
        } else {
            self.open(panel);
        }
    }
}

/// A themed window: no title bar, our own header with an icon, the title, a
/// hotkey hint and a close button. Returns false once it has been closed.
fn themed_window(
    ctx: &egui::Context,
    title: &str,
    glyph: Glyph,
    key: Option<&str>,
    size: egui::Vec2,
    add: impl FnOnce(&mut egui::Ui),
) -> bool {
    let screen = ctx.content_rect();
    let size = size.min(vec2(screen.width() - 40.0, screen.height() - TOP_BAR - 40.0));
    let pos = pos2(
        screen.center().x - size.x / 2.0,
        TOP_BAR + (screen.height() - TOP_BAR - size.y) / 2.0,
    );
    let mut open = true;
    egui::Window::new(title)
        .title_bar(false)
        .frame(theme::plate_frame().inner_margin(16))
        .default_size(size)
        .default_pos(pos)
        .collapsible(false)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::hover());
                theme::disc(ui.painter(), rect.center(), 16.0, palette::NAVY_RAISED, palette::GOLD_DIM);
                ui::glyph::paint(ui.painter(), glyph, rect.center(), 9.0, GOLD);
                ui.label(RichText::new(title).size(26.0).strong().color(GOLD));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::round_button(ui, 34.0, Glyph::Cross, palette::TEXT, None, true)
                        .on_hover_text("Close (Esc)")
                        .clicked()
                    {
                        open = false;
                    }
                    if let Some(k) = key {
                        let (rect, _) = ui.allocate_exact_size(vec2(30.0, 22.0), Sense::hover());
                        theme::keycap(ui.painter(), rect.center(), k);
                    }
                });
            });
            let y = ui.cursor().top() + 2.0;
            ui.painter().line_segment(
                [pos2(ui.min_rect().left(), y), pos2(ui.max_rect().right(), y)],
                Stroke::new(1.5, palette::GOLD_DIM),
            );
            ui.add_space(8.0);
            add(ui);
        });
    open
}

pub fn windows(s: &mut Session, portraits: &mut Portraits, ctx: &egui::Context) {
    for tree in [Tree::Tech, Tree::Civic, Tree::Ai] {
        let is_open = match tree {
            Tree::Tech => s.windows.tech,
            Tree::Civic => s.windows.civic,
            Tree::Ai => s.windows.ai,
        };
        if !is_open {
            continue;
        }
        let (title, glyph, key) = match tree {
            Tree::Tech => ("Technology Tree", Glyph::Science, "S"),
            Tree::Civic => ("Civics Tree", Glyph::Culture, "C"),
            Tree::Ai => ("Artificial Intelligence Tree", Glyph::Chip, "A"),
        };
        let mut picked = false;
        let open = themed_window(ctx, title, glyph, Some(key), vec2(1700.0, 1000.0), |ui| {
            picked = research_tree(s, ui, tree)
        });
        // Choosing something closes the tree.
        let open = open && !picked;
        match tree {
            Tree::Tech => s.windows.tech &= open,
            Tree::Civic => s.windows.civic &= open,
            Tree::Ai => s.windows.ai &= open,
        }
    }
    if s.windows.government {
        let open = themed_window(ctx, "Government", Glyph::Temple, Some("G"), vec2(800.0, 600.0), |ui| government(s, ui));
        s.windows.government &= open;
    }
    if s.windows.religion {
        let open = themed_window(ctx, "Religion", Glyph::Faith, Some("R"), vec2(680.0, 560.0), |ui| religion(s, ui));
        s.windows.religion &= open;
    }
    if s.windows.great_people {
        let open = themed_window(ctx, "Great People", Glyph::Star, Some("P"), vec2(760.0, 420.0), |ui| great_people(s, ui));
        s.windows.great_people &= open;
    }
    if s.windows.city_states {
        let open = themed_window(ctx, "City-States", Glyph::City, Some("E"), vec2(760.0, 440.0), |ui| city_states(s, ui));
        s.windows.city_states &= open;
    }
    if s.windows.victory {
        let open = themed_window(ctx, "Victory Progress", Glyph::Cup, Some("V"), vec2(680.0, 460.0), |ui| victory(s, ui));
        s.windows.victory &= open;
    }
    if s.windows.leaders {
        let open = themed_window(ctx, "Leaders", Glyph::Crown, Some("L"), vec2(700.0, 540.0), |ui| ui::leaders::panel(s, portraits, ui));
        s.windows.leaders &= open;
    }
    if let Some(other) = s.windows.diplomacy {
        let name = s.game.players[other].name.clone();
        let open = themed_window(ctx, &name, Glyph::Crown, None, vec2(860.0, 560.0), |ui| diplomacy(s, portraits, ui, other));
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
        let v = items[i]
            .prereqs
            .iter()
            .filter_map(|p| items.iter().position(|x| &x.id == p))
            .map(|j| d(j, items, memo) + 1)
            .max()
            .unwrap_or(0);
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
    out.extend(
        content
            .units
            .iter()
            .filter(|u| u.unique_to.is_none() && matches(&u.prereq_tech, &u.prereq_civic))
            .map(|u| u.name.clone()),
    );
    out.extend(
        content
            .buildings
            .iter()
            .filter(|b| b.unique_to.is_none() && matches(&b.prereq_tech, &b.prereq_civic))
            .map(|b| b.name.clone()),
    );
    out.extend(
        content
            .districts
            .iter()
            .filter(|d| d.unique_to.is_none() && matches(&d.prereq_tech, &d.prereq_civic))
            .map(|d| d.name.clone()),
    );
    out.extend(
        content
            .governments
            .iter()
            .filter(|g| matches(&None, &g.prereq_civic))
            .map(|g| g.name.clone()),
    );
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
    let (w, h, gx, gy) = (248.0, 64.0, 296.0, 78.0);
    let current = s.game.current_research(me, tree);
    let accent = match tree {
        Tree::Tech => YIELD_COLORS[3],
        Tree::Civic => YIELD_COLORS[4],
        Tree::Ai => crate::palette::COMPUTE,
    };
    let (icon_kind, fallback) = match tree {
        Tree::Tech => ("tech", Glyph::Science),
        Tree::Civic => ("civic", Glyph::Culture),
        Tree::Ai => ("ai", Glyph::Chip),
    };
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Click an available item to research it.")
                .color(palette::TEXT),
        );
        for (name, color) in [
            ("current", GOLD),
            ("available", accent),
            ("known", palette::GOOD),
        ] {
            let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
            ui.painter().rect_filled(rect, 4.0, color.gamma_multiply(0.8));
            ui.label(RichText::new(name).color(palette::TEXT_DIM).size(13.5));
        }
    });
    if tree == Tree::Ai {
        let compute = s.game.compute(me);
        if !s.game.ai_tree_unlocked(me) {
            ui.label(RichText::new("Research Artificial Intelligence to open this tree.").color(GOLD));
        } else {
            ui.label(RichText::new(format!("Compute +{compute:.0}/turn. Datacenters produce Compute: +6 each, +2 per adjacent district, +3 next to a river or the coast.")).color(crate::palette::COMPUTE));
        }
    }
    egui::ScrollArea::both().show(ui, |ui| {
        let size = vec2(
            gx * (max_col + 1) as f32 + 20.0,
            gy * (max_row - min_row + 1) as f32 + 20.0,
        );
        let (area, _) = ui.allocate_exact_size(size, Sense::hover());
        let origin = area.min + vec2(10.0, 10.0);
        let node_rect = |i: usize| {
            let (c, r) = positions[&i];
            Rect::from_min_size(
                origin + vec2(c as f32 * gx, (r - min_row) as f32 * gy),
                vec2(w, h),
            )
        };
        let painter = ui.painter().clone();
        for (i, item) in items.iter().enumerate() {
            for p in &item.prereqs {
                if let Some(j) = items.iter().position(|x| &x.id == p) {
                    let (a, b) = (node_rect(j), node_rect(i));
                    let from = a.right_center();
                    let to = b.left_center();
                    let mid = Pos2::new((from.x + to.x) / 2.0, from.y);
                    let line = if s.game.known(me, tree, j) {
                        Stroke::new(2.0, palette::GOLD_DIM)
                    } else {
                        Stroke::new(1.5, Color32::from_rgb(54, 66, 84))
                    };
                    painter.add(egui::Shape::line(
                        vec![from, mid, Pos2::new(mid.x, to.y), to],
                        line,
                    ));
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
            let is_current = Some(i) == current;
            let (fill, edge, text) = if is_current {
                (Color32::from_rgb(70, 54, 20), GOLD, Color32::WHITE)
            } else if known {
                (Color32::from_rgb(20, 58, 50), palette::GOOD.gamma_multiply(0.55), Color32::from_gray(225))
            } else if available {
                (Color32::from_rgb(26, 48, 80), accent, Color32::WHITE)
            } else {
                (Color32::from_rgb(15, 22, 34), Color32::from_rgb(46, 58, 76), Color32::from_gray(130))
            };
            let response = ui.interact(rect, ui.scope_id().with(("node", i)), Sense::click());
            let hovered = response.hovered() && (available || known);
            painter.rect_filled(rect, 14.0, if hovered { fill.gamma_multiply(1.35) } else { fill });
            painter.rect_stroke(
                rect,
                14.0,
                Stroke::new(if hovered || is_current { 2.5 } else { 1.5 }, edge),
                StrokeKind::Inside,
            );
            let ic = rect.left_center() + vec2(h / 2.0, 0.0);
            painter.circle_filled(ic, h / 2.0 - 7.0, Color32::from_rgb(10, 18, 30));
            if !s.icons.paint(
                &painter,
                icon_kind,
                &item.id,
                Rect::from_center_size(ic, vec2(h - 18.0, h - 18.0)),
            ) {
                ui::glyph::paint(&painter, fallback, ic, 13.0, if known || available { accent } else { Color32::from_gray(100) });
            }
            painter.circle_stroke(ic, h / 2.0 - 7.0, Stroke::new(1.5, edge));
            let text_x = h + 4.0;
            let name = ui::trackers::fit(&painter, &item.name, 15.5, w - text_x - 10.0);
            painter.text(
                rect.min + vec2(text_x, 17.0),
                Align2::LEFT_CENTER,
                name,
                FontId::proportional(15.5),
                text,
            );
            let cost = s.game.research_cost(tree, i);
            let (status, status_color) = if known {
                ("Known".to_string(), palette::GOOD)
            } else if available || is_current {
                (format!("{cost}   {}", s.research_turns(tree, i)), accent)
            } else {
                (format!("{cost}"), Color32::from_gray(120))
            };
            painter.text(
                rect.min + vec2(text_x, 37.0),
                Align2::LEFT_CENTER,
                status,
                FontId::proportional(13.5),
                status_color,
            );
            if item.boost.is_some() && !known {
                painter.text(
                    rect.min + vec2(text_x, 53.0),
                    Align2::LEFT_CENTER,
                    if boosted { "Boosted!" } else { "Boost: hover" },
                    FontId::proportional(11.5),
                    if boosted { GOLD } else { Color32::from_gray(120) },
                );
            }
            let progress = s.game.progress(me, tree, i);
            if !known && progress > 0 {
                let bar = Rect::from_min_size(
                    rect.left_bottom() + vec2(16.0, -7.0),
                    vec2((w - 32.0) * (progress as f32 / cost.max(1) as f32).min(1.0), 3.5),
                );
                painter.rect_filled(bar, 2.0, accent);
            }
            let mut tip = item.name.clone();
            if tree == Tree::Ai {
                if let Some(t) = wayciv_core::additions::AI_TREE
                    .iter()
                    .find(|t| t.id == item.id)
                {
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
    let available = s.game.available_governments(me);
    if available.is_empty() {
        ui.label("Research the Code of Laws civic to adopt your first government (Chiefdom).");
        return;
    }
    ui.horizontal_wrapped(|ui| {
        for g in available {
            let gov = &content.governments[g];
            let selected = current == Some(g);
            let slots: Vec<String> = gov
                .slots
                .iter()
                .map(|(t, n)| format!("{n} {}", slot_name(t)))
                .collect();
            let response = ui
                .selectable_label(selected, format!("{}\n{}", gov.name, slots.join(", ")))
                .on_hover_text(&gov.bonus);
            if response.clicked() && !selected {
                let r = s.game.set_government(me, g);
                s.result(r);
                s.windows.policy_draft = None;
            }
        }
    });
    let Some(g) = s.game.players[me].government else {
        return;
    };
    ui.label(RichText::new(&content.governments[g].bonus).italics());
    ui.separator();
    let draft = s
        .windows
        .policy_draft
        .get_or_insert_with(|| s.game.players[me].policies.clone())
        .clone();
    let slots = s.game.policy_slots(me);
    let free = s.game.players[me].policy_change_free;
    ui.label(format!(
        "Slots: {}.  {}",
        slots
            .iter()
            .map(|(t, n)| format!("{n} {}", slot_name(t)))
            .collect::<Vec<_>>()
            .join(", "),
        if free {
            "Changing policies is free this turn."
        } else {
            "Changes cost 25 Gold each until your next civic."
        }
    ));
    let available = s.game.available_policies(me);
    let mut next = draft.clone();
    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 40.0)
        .show(ui, |ui| {
            for kind in [
                "SLOT_MILITARY",
                "SLOT_ECONOMIC",
                "SLOT_DIPLOMATIC",
                "SLOT_GREAT_PERSON",
            ] {
                let list: Vec<usize> = available
                    .iter()
                    .copied()
                    .filter(|&p| content.policies[p].slot == kind)
                    .collect();
                if list.is_empty() {
                    continue;
                }
                ui.label(RichText::new(slot_name(kind)).strong().color(GOLD));
                for p in list {
                    let policy = &content.policies[p];
                    let mut on = next.contains(&p);
                    let active = s
                        .game
                        .rules
                        .content
                        .modifiers_of(&policy.id)
                        .any(wayciv_core::effects::is_active);
                    let tag = if active { "" } else { "  (description only)" };
                    if ui
                        .checkbox(&mut on, format!("{}{tag}", policy.name))
                        .on_hover_text(&policy.description)
                        .changed()
                    {
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
    if ui
        .button(RichText::new("Confirm Policies").strong())
        .clicked()
    {
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
                ui.label(
                    RichText::new(format!("Pantheon: {}", content.beliefs[b].name)).color(GOLD),
                );
            });
            ui.label(&content.beliefs[b].description);
        }
        None => {
            ui.label(
                RichText::new(format!(
                    "Choose a pantheon belief ({} Faith).",
                    wayciv_core::religion::PANTHEON_COST
                ))
                .color(GOLD),
            );
            let can = s.game.can_choose_pantheon(me);
            egui::ScrollArea::vertical()
                .max_height(260.0)
                .show(ui, |ui| {
                    for b in s.game.beliefs_of_class("BELIEF_CLASS_PANTHEON") {
                        ui.horizontal(|ui| {
                            s.icons.show(ui, "belief", &content.beliefs[b].id, 28.0);
                            if ui
                                .add_enabled(can, egui::Button::new(&content.beliefs[b].name))
                                .clicked()
                            {
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
            ui.label(
                RichText::new(format!("You founded {}.", content.religions[r].name)).color(GOLD),
            );
            for &b in &p.beliefs {
                ui.horizontal(|ui| {
                    s.icons.show(ui, "belief", &content.beliefs[b].id, 28.0);
                    ui.label(format!(
                        "{}: {}",
                        content.beliefs[b].name, content.beliefs[b].description
                    ));
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
        let cities = s
            .game
            .cities
            .values()
            .filter(|c| c.majority_religion() == Some(r))
            .count();
        ui.label(format!(
            "{} - founded by {}, majority in {cities} cities",
            content.religions[r].name, s.game.players[founder].civ_name
        ));
    }
}

fn great_people(s: &mut Session, ui: &mut egui::Ui) {
    let me = s.me();
    let rates: HashMap<usize, i32> = s.game.great_person_rates(me).into_iter().collect();
    let content = &s.game.rules.content;
    egui::Grid::new("gp")
        .striped(true)
        .num_columns(4)
        .show(ui, |ui| {
            ui.strong("Class");
            ui.strong("Points");
            ui.strong("Next");
            ui.strong("Effect");
            ui.end_row();
            for (class, gpc) in content.great_person_classes.iter().enumerate() {
                let points = s.game.players[me]
                    .great_person_points
                    .get(&class)
                    .copied()
                    .unwrap_or(0);
                let rate = rates.get(&class).copied().unwrap_or(0);
                let next = s.game.next_great_person(class);
                ui.label(&gpc.name);
                ui.label(format!(
                    "{points}/{} (+{rate})",
                    s.game.great_person_cost(class)
                ));
                ui.label(
                    next.map(|g| content.great_people[g].name.clone())
                        .unwrap_or("-".into()),
                );
                ui.label(
                    next.map(|g| s.game.great_person_effect(g))
                        .unwrap_or_default(),
                );
                ui.end_row();
            }
        });
}

fn city_states(s: &mut Session, ui: &mut egui::Ui) {
    let me = s.me();
    ui.label(format!(
        "Envoys available: {}",
        s.game.players[me].envoys_available
    ));
    ui.label("1 envoy: +2 of the city-state's yield in your capital. 3 and 6 envoys: +1 in every city. Suzerain (most envoys, at least 3): +1 more.");
    let states: Vec<PlayerId> = s
        .game
        .players
        .iter()
        .filter(|p| p.is_city_state() && s.game.players[me].relations[p.id].met)
        .map(|p| p.id)
        .collect();
    egui::Grid::new("cs")
        .striped(true)
        .num_columns(5)
        .show(ui, |ui| {
            for cs in states {
                let p = s.game.players[cs].clone();
                ui.label(
                    RichText::new(&p.name)
                        .color(rgb(p.primary).gamma_multiply(1.4))
                        .strong(),
                );
                ui.label(p.city_state_category.to_lowercase());
                ui.label(format!(
                    "Your envoys: {}",
                    s.game.players[me].envoys.get(&cs).copied().unwrap_or(0)
                ));
                ui.label(match s.game.suzerain(cs) {
                    Some(x) => format!("Suzerain: {}", s.game.players[x].civ_name),
                    None => "No suzerain".into(),
                });
                if !p.alive {
                    ui.label("Conquered");
                } else if ui
                    .add_enabled(
                        s.game.players[me].envoys_available > 0,
                        egui::Button::new("Send Envoy"),
                    )
                    .clicked()
                {
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
    let mut players: Vec<&wayciv_core::state::Player> = s
        .game
        .alive_majors()
        .filter(|p| p.id == me || s.game.players[me].relations[p.id].met)
        .collect();
    players.sort_by_key(|p| std::cmp::Reverse(s.game.score(p.id)));
    for p in players {
        ui.label(format!(
            "{} ({}) - {}",
            p.civ_name,
            p.name,
            s.game.score(p.id)
        ));
    }
}

fn diplomacy(s: &mut Session, portraits: &mut Portraits, ui: &mut egui::Ui, other: PlayerId) {
    let me = s.me();
    let o = s.game.players[other].clone();
    let leader = s.game.rules.content.leaders[o.leader].clone();
    let rel = ui::leaders::relationship(s, other);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(300.0, 300.0), Sense::hover());
        let painter = ui.painter().clone();
        theme::portrait_disc(&painter, portraits, rect.center(), 140.0, &leader.id, &leader.name, (o.primary, o.secondary));
        painter.circle_stroke(rect.center(), 143.0, Stroke::new(5.0, rel.status_color));
        painter.circle_stroke(rect.center(), 148.0, Stroke::new(1.5, palette::GOLD_DIM));
        ui.vertical(|ui| {
            ui.set_width(420.0);
            ui.label(RichText::new(&leader.name).color(GOLD).size(30.0).strong());
            ui.label(RichText::new(&o.civ_name).size(19.0));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                theme::tag(ui, rel.status, rel.status_color);
                theme::tag(ui, rel.mood, rel.mood_color);
            });
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "\"{}\"",
                    s.game.dialogue(other, DiploEvent::Greeting)
                ))
                .italics()
                .size(15.5),
            );
            ui.add_space(4.0);
            ui.label(RichText::new(&rel.detail).color(palette::TEXT_DIM));
            if let Some((name, desc)) = &leader.agenda {
                ui.add_space(4.0);
                ui.label(RichText::new(format!("Agenda: {name}")).strong().color(GOLD));
                ui.label(desc);
            }
            for t in &leader.traits {
                ui.label(RichText::new(&t.name).strong().color(GOLD))
                    .on_hover_text(&t.description);
            }
        });
    });
    ui.separator();
    let at_war = s.game.players[me].at_war_with(other);
    ui.horizontal(|ui| {
        if !at_war {
            if !s.game.players[me].relations[other].friendship
                && ui.button("Declare Friendship").clicked()
            {
                let accepted = s.game.propose_friendship(me, other);
                let line = s.game.dialogue(
                    other,
                    if accepted {
                        DiploEvent::AcceptFriendFromHuman
                    } else {
                        DiploEvent::RejectFriendFromHuman
                    },
                );
                s.toast(format!("{}: \"{line}\"", leader.name));
            }
            if !s.game.players[me].relations[other].denounced && ui.button("Denounce").clicked() {
                s.game.denounce(me, other);
                let line = s.game.dialogue(other, DiploEvent::DenounceFromHuman);
                s.toast(format!("{}: \"{line}\"", leader.name));
            }
            if ui
                .button(RichText::new("Declare War").color(Color32::from_rgb(240, 90, 80)))
                .clicked()
            {
                s.game.declare_war(me, other);
                let line = s.game.dialogue(other, DiploEvent::DeclareWarFromHuman);
                s.toast(format!("{}: \"{line}\"", leader.name));
            }
        } else {
            let can = s.game.can_make_peace(me, other);
            if ui
                .add_enabled(can, egui::Button::new("Propose Peace"))
                .on_disabled_hover_text("Wars last at least 10 turns.")
                .clicked()
            {
                let accepted = s.game.propose_peace(me, other);
                let line = s.game.dialogue(
                    other,
                    if accepted {
                        DiploEvent::MakePeaceAccept
                    } else {
                        DiploEvent::MakePeaceRefuse
                    },
                );
                s.toast(format!("{}: \"{line}\"", leader.name));
            }
        }
    });
}

fn leader_popup(
    s: &mut Session,
    portraits: &mut Portraits,
    ctx: &egui::Context,
    other: PlayerId,
    event: DiploEvent,
) {
    let me = s.me();
    let o = s.game.players[other].clone();
    let leader = s.game.rules.content.leaders[o.leader].clone();
    let line = s.game.dialogue(other, event);
    let mut close = false;
    egui::Modal::new(egui::Id::unique("leader_popup")).frame(theme::plate_frame().inner_margin(20)).show(ctx, |ui| {
        ui.set_width(860.0);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(380.0, 380.0), Sense::hover());
            let painter = ui.painter().clone();
            theme::portrait_disc(&painter, portraits, rect.center(), 180.0, &leader.id, &leader.name, (o.primary, o.secondary));
            painter.circle_stroke(rect.center(), 184.0, Stroke::new(5.0, palette::GOLD));
            ui.vertical(|ui| {
                ui.set_width(420.0);
                ui.heading(RichText::new(&leader.name).color(GOLD).size(28.0));
                ui.label(RichText::new(&o.civ_name).size(18.0));
                ui.add_space(12.0);
                ui.label(RichText::new(format!("\"{line}\"")).italics().size(17.0));
                ui.add_space(16.0);
                match event {
                    DiploEvent::DeclareFriendFromAi => {
                        ui.label(format!(
                            "{} proposes a declaration of friendship.",
                            leader.name
                        ));
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
        Lists {
            governments: self.governments.clone(),
            policies: self.policies.clone(),
            beliefs: self.beliefs.clone(),
            religions: self.religions.clone(),
        }
    }
}

#[allow(dead_code)]
fn unused(_: &Game) {}

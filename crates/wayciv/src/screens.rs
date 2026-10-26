//! Full-screen-style windows: the tech and AI trees, government, religion,
//! great people, city-states, victory, diplomacy, and leader pop-ups.

use crate::hud::Session;
use crate::palette::{self, GOLD, YIELD_COLORS};
use crate::portraits::{self, Portraits};
use crate::ui::glyph::Glyph;
use crate::ui::{self, TOP_BAR, anim, theme};
use egui::{
    Align2, Color32, FontId, Id, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, pos2, vec2,
};
use std::collections::HashMap;
use wayciv_core::diplomacy::DiploEvent;
use wayciv_core::research::Tree;
use wayciv_core::{Game, PlayerId};

#[derive(Default)]
pub struct Windows {
    pub tech: bool,
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
    /// A religion being founded with a Great Prophet.
    pub found: Option<ui::menus::FoundDraft>,
}

/// A window or panel the player can open. Only one is open at a time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Panel {
    Tech,
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

    /// A full window (not the Notifications panel or the map key) is open.
    pub fn big_open(&self) -> bool {
        self.any_open() && !self.notifications
    }

    pub fn is_open(&self, panel: Panel) -> bool {
        match panel {
            Panel::Tech => self.tech,
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
/// It fades and scales in, and its remains fade out (see `anim::ghosts`).
fn themed_window(
    ctx: &egui::Context,
    title: &str,
    glyph: Glyph,
    key: Option<&str>,
    size: egui::Vec2,
    add: impl FnOnce(&mut egui::Ui),
) -> bool {
    let screen = ctx.content_rect();
    let size = size.min(vec2(
        screen.width() - 40.0,
        screen.height() - TOP_BAR - 40.0,
    ));
    let center = pos2(
        screen.center().x,
        TOP_BAR + (screen.height() - TOP_BAR) / 2.0,
    );
    let mut open = true;
    // The window is `size` wide and hugs its contents up to `size` tall.
    let shown = egui::Window::new(title)
        .title_bar(false)
        .frame(egui::Frame::NONE)
        .resizable(false)
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(center)
        .collapsible(false)
        .fade_in(false)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            theme::plate_ui(ui, 20, |ui| {
                ui.set_width(size.x - 40.0);
                ui.set_max_height(size.y - 40.0);
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(48.0, 48.0), Sense::hover());
                    theme::disc(
                        ui.painter(),
                        rect.center(),
                        22.0,
                        palette::NAVY_RAISED,
                        palette::GOLD,
                    );
                    ui::glyph::paint(ui.painter(), glyph, rect.center(), 11.0, GOLD);
                    ui.add_space(6.0);
                    ui.label(theme::title(title, 30.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::round_button(ui, 38.0, Glyph::Cross, palette::TEXT, None, true)
                            .on_hover_text("Close (Esc)")
                            .clicked()
                        {
                            open = false;
                        }
                        if let Some(k) = key {
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(30.0, 22.0), Sense::hover());
                            theme::keycap(ui.painter(), rect.center(), k);
                        }
                    });
                });
                let y = ui.cursor().top() + 6.0;
                theme::divider(
                    ui.painter(),
                    ui.min_rect().left(),
                    ui.max_rect().right(),
                    y,
                    palette::with_alpha(GOLD, 230),
                );
                ui.add_space(16.0);
                add(ui);
            });
        });
    if let Some(r) = shown {
        anim::window_fx(ctx, Id::unique(("window", title)), r.response.layer_id, r.response.rect);
    }
    open
}

pub fn windows(s: &mut Session, portraits: &mut Portraits, ctx: &egui::Context) {
    let before = (s.windows.big_open(), s.popup.is_some());
    anim::backdrop(ctx, before.0 || before.1, if before.1 { 120 } else { 90 }, before.1);
    windows_inner(s, portraits, ctx);
    anim::ghosts(ctx);
    // A window closed by a click needs one more frame to start fading out.
    if before != (s.windows.big_open(), s.popup.is_some()) {
        ctx.request_repaint();
    }
}

fn windows_inner(s: &mut Session, portraits: &mut Portraits, ctx: &egui::Context) {
    for tree in [Tree::Tech, Tree::Ai] {
        let is_open = match tree {
            Tree::Tech => s.windows.tech,
            Tree::Ai => s.windows.ai,
        };
        if !is_open {
            continue;
        }
        let (title, glyph, key) = match tree {
            Tree::Tech => ("Technology Tree", Glyph::Science, "S"),
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
            Tree::Ai => s.windows.ai &= open,
        }
    }
    // The empire menus close themselves once their choice is made.
    type Menu = fn(&mut Session, &mut egui::Ui) -> bool;
    let menus: [(Panel, &str, Glyph, &str, egui::Vec2, Menu); 5] = [
        (
            Panel::Government,
            "Government",
            Glyph::Temple,
            "G",
            vec2(1100.0, 860.0),
            ui::menus::government,
        ),
        (
            Panel::Religion,
            "Religion",
            Glyph::Faith,
            "R",
            vec2(1000.0, 820.0),
            ui::menus::religion,
        ),
        (
            Panel::GreatPeople,
            "Great People",
            Glyph::Star,
            "P",
            vec2(1100.0, 760.0),
            ui::menus::great_people,
        ),
        (
            Panel::CityStates,
            "City-States",
            Glyph::City,
            "E",
            vec2(1100.0, 720.0),
            ui::menus::city_states,
        ),
        (
            Panel::Victory,
            "Victory Progress",
            Glyph::Cup,
            "V",
            vec2(760.0, 720.0),
            ui::menus::victory,
        ),
    ];
    for (panel, title, glyph, key, size, menu) in menus {
        if !s.windows.is_open(panel) {
            continue;
        }
        let mut done = false;
        let open = themed_window(ctx, title, glyph, Some(key), size, |ui| done = menu(s, ui));
        if !open || done {
            s.windows.close_all();
        }
    }
    if s.windows.leaders {
        let open = themed_window(
            ctx,
            "Leaders",
            Glyph::Crown,
            Some("L"),
            vec2(700.0, 540.0),
            |ui| ui::leaders::panel(s, portraits, ui),
        );
        s.windows.leaders &= open;
    }
    if let Some(other) = s.windows.diplomacy {
        let open = themed_window(ctx, "Diplomacy", Glyph::Crown, None, vec2(1040.0, 640.0), |ui| {
            diplomacy(s, portraits, ui, other)
        });
        if !open {
            s.windows.diplomacy = None;
        }
    }
    if let Some((other, event)) = s.popup {
        leader_popup(s, portraits, ctx, other, event);
    }
}

/// Columns: depth in the prerequisite graph. Rows: each item's `row`.
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

/// Things a tech unlocks, for its tooltip.
fn unlocks(game: &Game, tree: Tree, id: &str) -> Vec<String> {
    let content = &game.rules.content;
    let matches = |tech: &Option<String>| match tree {
        Tree::Tech => tech.as_deref() == Some(id),
        Tree::Ai => false,
    };
    let mut out: Vec<String> = Vec::new();
    out.extend(
        content
            .units
            .iter()
            .filter(|u| u.unique_to.is_none() && matches(&u.prereq_tech))
            .map(|u| u.name.clone()),
    );
    out.extend(
        content
            .buildings
            .iter()
            .filter(|b| b.unique_to.is_none() && matches(&b.prereq_tech))
            .map(|b| b.name.clone()),
    );
    out.extend(
        content
            .districts
            .iter()
            .filter(|d| d.unique_to.is_none() && matches(&d.prereq_tech))
            .map(|d| d.name.clone()),
    );
    out.extend(
        content
            .improvements
            .iter()
            .filter(|i| i.buildable && i.unique_to.is_none() && matches(&i.prereq_tech))
            .map(|i| i.name.clone()),
    );
    if tree == Tree::Tech && id == wayciv_core::additions::TECH_AI {
        out.push("the AI tree".into());
    }
    out
}

/// Unit types (ids) a tech unlocks.
fn unlocked_units(game: &Game, tree: Tree, id: &str) -> Vec<String> {
    game.rules
        .content
        .units
        .iter()
        .filter(|u| {
            u.unique_to.is_none()
                && match tree {
                    Tree::Tech => u.prereq_tech.as_deref() == Some(id),
                    Tree::Ai => false,
                }
        })
        .map(|u| u.id.clone())
        .collect()
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
        Tree::Ai => crate::palette::COMPUTE,
    };
    let (icon_kind, fallback) = match tree {
        Tree::Tech => ("tech", Glyph::Science),
        Tree::Ai => ("ai", Glyph::Chip),
    };
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Click a bright item to research it next. Dim items need the ones before them first. Hover for what each unlocks.")
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
            ui.label(
                RichText::new("Research Artificial Intelligence to open this tree.").color(GOLD),
            );
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
        let ctx = ui.ctx().clone();
        // The node under the pointer lights up its links to the items around it.
        let pointer = ctx
            .pointer_hover_pos()
            .filter(|p| ui.clip_rect().contains(*p));
        let hover_node = pointer.and_then(|p| (0..items.len()).find(|&i| node_rect(i).contains(p)));
        for (i, item) in items.iter().enumerate() {
            for p in &item.prereqs {
                if let Some(j) = items.iter().position(|x| &x.id == p) {
                    let (a, b) = (node_rect(j), node_rect(i));
                    let from = a.right_center();
                    let to = b.left_center();
                    let mid = Pos2::new((from.x + to.x) / 2.0, from.y);
                    let lit = hover_node == Some(i) || hover_node == Some(j);
                    let lit_t = anim::hover(&ctx, ui.scope_id().with(("link", i, j)), lit, 0.15);
                    let mut line = if s.game.known(me, tree, j) {
                        Stroke::new(2.0, palette::GOLD_DIM)
                    } else {
                        Stroke::new(1.5, Color32::from_rgb(54, 66, 84))
                    };
                    line.width += 1.2 * lit_t;
                    line.color = anim::mix(line.color, GOLD, lit_t);
                    let path = vec![from, mid, Pos2::new(mid.x, to.y), to];
                    painter.add(egui::Shape::line(
                        path.clone(),
                        Stroke::new(line.width + 3.0, Color32::from_black_alpha(140)),
                    ));
                    painter.add(egui::Shape::line(path, line));
                    painter.circle_filled(to, 3.5, line.color);
                }
            }
        }
        for (i, item) in items.iter().enumerate() {
            let base = node_rect(i);
            let known = s.game.known(me, tree, i);
            let available = s.game.researchable(me, tree, i);
            let boosted = match tree {
                Tree::Tech => s.game.players[me].tech_boosted[i],
                Tree::Ai => false,
            };
            let is_current = Some(i) == current;
            let (fill, edge, text) = if is_current {
                (Color32::from_rgb(70, 54, 20), GOLD, Color32::WHITE)
            } else if known {
                (
                    Color32::from_rgb(20, 58, 50),
                    palette::GOOD.gamma_multiply(0.55),
                    Color32::from_gray(225),
                )
            } else if available {
                (Color32::from_rgb(26, 48, 80), accent, Color32::WHITE)
            } else {
                (
                    Color32::from_rgb(15, 22, 34),
                    Color32::from_rgb(46, 58, 76),
                    Color32::from_gray(130),
                )
            };
            let response = ui.interact(base, ui.scope_id().with(("node", i)), Sense::click());
            let hovered = response.hovered() && (available || known);
            let id = response.id;
            let hot = anim::hover(&ctx, id.with("hot"), hovered, 0.13);
            let down = anim::hover(
                &ctx,
                id.with("down"),
                available && response.is_pointer_button_down_on(),
                0.07,
            );
            if response.clicked() && available {
                anim::trigger(&ctx, id.with("pop"));
            }
            // Hovering lifts a node; the current one breathes a gold glow.
            let rect = base
                .translate(vec2(0.0, -1.5 * hot * (1.0 - down)))
                .shrink(1.2 * down);
            let on_screen = ui.is_rect_visible(rect);
            if on_screen {
                if is_current {
                    anim::ambient(&ctx);
                    theme::glow(&painter, rect, 14.0, GOLD, 0.3 + 0.55 * anim::wave(&ctx, 2.4));
                }
                theme::glow(&painter, rect, 14.0, edge, hot * 0.65);
                if let Some(p) = anim::progress(&ctx, id.with("pop"), 0.35) {
                    let e = anim::ease_out(p);
                    painter.rect_stroke(
                        rect.expand(1.0 + 10.0 * e),
                        14.0 + 10.0 * e,
                        Stroke::new(2.5, GOLD.gamma_multiply((1.0 - e) * 0.85)),
                        StrokeKind::Outside,
                    );
                }
            }
            theme::well(
                &painter,
                rect,
                14.0,
                anim::mix(fill, fill.gamma_multiply(1.3), hot),
                Color32::TRANSPARENT,
            );
            painter.rect_stroke(
                rect,
                14.0,
                Stroke::new(
                    anim::lerp(1.5, 2.5, hot.max(if is_current { 1.0 } else { 0.0 })),
                    anim::mix(edge, Color32::WHITE, hot * 0.3),
                ),
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
                ui::glyph::paint(
                    &painter,
                    fallback,
                    ic,
                    13.0,
                    if known || available {
                        accent
                    } else {
                        Color32::from_gray(100)
                    },
                );
            }
            painter.circle_stroke(ic, h / 2.0 - 7.0, Stroke::new(1.5, edge));
            painter.circle_stroke(
                ic,
                h / 2.0 - 5.5,
                Stroke::new(1.0, Color32::from_black_alpha(150)),
            );
            let text_x = h + 4.0;
            let name = ui::trackers::fit(&painter, &item.name, 15.5, w - text_x - 30.0);
            painter.text(
                rect.min + vec2(text_x, 17.0),
                Align2::LEFT_CENTER,
                name,
                palette::bold(15.0),
                text,
            );
            if known {
                // A check mark in the corner of researched items.
                let c = rect.right_top() + vec2(-17.0, 17.0);
                theme::disc(&painter, c, 8.5, palette::GOOD.gamma_multiply(0.5), palette::GOOD);
                painter.add(egui::Shape::line(
                    vec![c + vec2(-3.5, 0.0), c + vec2(-1.0, 3.0), c + vec2(3.5, -3.0)],
                    Stroke::new(2.0, Color32::WHITE),
                ));
            }
            let cost = s.game.research_cost(tree, i);
            let (status, status_color) = if known {
                ("Researched".to_string(), palette::GOOD)
            } else if is_current {
                (format!("Researching: {}", s.research_turns(tree, i)), GOLD)
            } else {
                (
                    s.research_turns(tree, i),
                    if available {
                        accent
                    } else {
                        Color32::from_gray(120)
                    },
                )
            };
            painter.text(
                rect.min + vec2(text_x, 36.0),
                Align2::LEFT_CENTER,
                status,
                FontId::proportional(13.5),
                status_color,
            );
            // What it unlocks, as unit icons along the right edge.
            let unit_icons: Vec<String> = if tree == Tree::Ai {
                Vec::new()
            } else {
                unlocked_units(&s.game, tree, &item.id)
            };
            for (k, id) in unit_icons.iter().rev().take(3).enumerate() {
                let c = pos2(rect.right() - 16.0 - k as f32 * 22.0, rect.min.y + 36.0);
                painter.circle_filled(c, 10.0, Color32::from_rgb(10, 18, 30));
                s.icons.paint(
                    &painter,
                    "unit",
                    id,
                    Rect::from_center_size(c, vec2(19.0, 19.0)),
                );
            }
            if let Some(b) = item.boost.as_ref().filter(|_| !known) {
                let label = if boosted {
                    "Boosted (50% done)".to_string()
                } else {
                    ui::trackers::fit(&painter, &format!("Boost: {b}"), 11.5, w - text_x - 12.0)
                };
                painter.text(
                    rect.min + vec2(text_x, 50.0),
                    Align2::LEFT_CENTER,
                    label,
                    FontId::proportional(11.5),
                    if boosted {
                        GOLD
                    } else {
                        Color32::from_gray(140)
                    },
                );
            }
            let progress = s.game.progress(me, tree, i);
            if !known && progress > 0 {
                let frac = (progress as f32 / cost.max(1) as f32).min(1.0);
                let frac = anim::smooth(&ctx, id.with("bar"), frac, 8.0);
                theme::bar(
                    &painter,
                    Rect::from_min_size(rect.left_bottom() + vec2(16.0, -7.0), vec2(w - 32.0, 4.0)),
                    frac,
                    accent,
                );
            }
            let mut tip = if known {
                item.name.clone()
            } else {
                format!(
                    "{} ({cost} {})",
                    item.name,
                    match tree {
                        Tree::Tech => "Science",
                        Tree::Ai => "Compute",
                    }
                )
            };
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
            if available {
                tip += "\n\nClick to research this next.";
            } else if !known && !is_current {
                tip += "\n\nResearch the items before it first.";
            }
            let response = response.on_hover_text(tip);
            let response = if available {
                response.on_hover_cursor(egui::CursorIcon::PointingHand)
            } else {
                response
            };
            if response.clicked() && available {
                let r = s.game.set_research(me, tree, i);
                picked = r.is_ok();
                s.result(r);
            }
        }
    });
    picked
}

/// A leader or civilization trait: a gold name over its description.
fn trait_block(ui: &mut egui::Ui, name: &str, desc: &str) {
    theme::well_ui(
        ui,
        10,
        palette::NAVY_RAISED,
        palette::with_alpha(palette::GOLD_DIM, 150),
        |ui| {
            ui.set_width(ui.available_width());
            ui.label(theme::bold(name, 15.0).color(GOLD));
            if !desc.is_empty() {
                ui.label(RichText::new(desc).size(14.0).color(palette::TEXT_DIM));
            }
        },
    );
}

/// A leader's quotation in a well with an opening quotation mark.
fn quote(ui: &mut egui::Ui, line: &str, size: f32) {
    theme::well_ui(
        ui,
        14,
        Color32::from_rgb(14, 24, 40),
        palette::with_alpha(palette::TEAL, 90),
        |ui| {
            ui.set_width(ui.available_width());
            ui.add(
                egui::Label::new(
                    RichText::new(format!("\u{201C}{line}\u{201D}"))
                        .italics()
                        .size(size)
                        .color(palette::TEXT),
                )
                .wrap(),
            );
        },
    );
}

/// Their opinion of you as a bar from hostile to friendly, with score and
/// city count beside it.
fn opinion(ui: &mut egui::Ui, rel: &ui::leaders::Relationship, score: i32, cities: usize) {
    theme::well_ui(
        ui,
        10,
        palette::NAVY_RAISED,
        palette::with_alpha(palette::GOLD_DIM, 150),
        |ui| {
            ui.set_width(ui.available_width());
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::hover());
            let painter = ui.painter();
            theme::text(painter, rect.left_top() + vec2(0.0, 8.0), Align2::LEFT_CENTER, "Their opinion of you", 13.5, palette::TEXT_DIM);
            theme::text_bold(painter, rect.right_top() + vec2(0.0, 8.0), Align2::RIGHT_CENTER, format!("{}  ({:+})", rel.mood, rel.opinion), 14.5, rel.mood_color);
            let bar = Rect::from_min_size(rect.left_top() + vec2(0.0, 24.0), vec2(rect.width(), 10.0));
            painter.rect_filled(bar, 5.0, Color32::from_black_alpha(120));
            let mid = bar.center().x;
            let end = mid + bar.width() / 2.0 * rel.opinion as f32 / 100.0;
            let (l, r) = if end >= mid { (mid, end) } else { (end, mid) };
            painter.add(theme::gradient_rect(
                Rect::from_min_max(pos2(l, bar.top() + 1.0), pos2(r.max(l + 3.0), bar.bottom() - 1.0)),
                4.0,
                rel.mood_color.lerp_to_gamma(Color32::WHITE, 0.3),
                rel.mood_color.gamma_multiply(0.7),
            ));
            painter.line_segment(
                [pos2(mid, bar.top() - 3.0), pos2(mid, bar.bottom() + 3.0)],
                Stroke::new(1.5, palette::TEXT_DIM),
            );
            painter.rect_stroke(bar, 5.0, Stroke::new(1.0, Color32::from_black_alpha(170)), StrokeKind::Outside);
            ui.label(
                RichText::new(format!("Score {score}   Cities {cities}"))
                    .size(13.5)
                    .color(palette::TEXT_DIM),
            );
        },
    );
}

fn diplomacy(s: &mut Session, portraits: &mut Portraits, ui: &mut egui::Ui, other: PlayerId) {
    let me = s.me();
    let o = s.game.players[other].clone();
    let leader = s.game.rules.content.leaders[o.leader].clone();
    let rel = ui::leaders::relationship(s, other);
    let at_war = s.game.players[me].at_war_with(other);
    let side = (ui.available_height() - 110.0).clamp(240.0, 440.0);
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            let frame = portraits::FRAME;
            let (rect, _) = ui.allocate_exact_size(
                vec2(side + 2.0 * frame, side + 2.0 * frame),
                Sense::hover(),
            );
            portraits.paint(
                ui,
                rect.shrink(frame),
                &leader.id,
                &leader.name,
                (o.primary, o.secondary),
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                theme::tag(ui, rel.status, rel.status_color);
                theme::tag(ui, rel.mood, rel.mood_color);
            });
        });
        ui.add_space(10.0);
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical()
                .max_height(side + 40.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    ui.label(theme::title(&leader.name, 34.0));
                    ui.horizontal(|ui| {
                        for color in [o.primary, o.secondary] {
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                            theme::disc(
                                ui.painter(),
                                rect.center(),
                                7.0,
                                palette::rgb(color),
                                palette::GOLD_DIM,
                            );
                        }
                        ui.label(theme::bold(&o.civ_name, 19.0));
                    });
                    let greeting = s.game.dialogue(other, DiploEvent::Greeting);
                    quote(ui, &greeting, 17.0);
                    ui.label(RichText::new(&rel.detail).color(palette::TEXT_DIM));
                    opinion(ui, &rel, s.game.score(other), s.game.player_cities(other).count());
                    if let Some((name, desc)) = &leader.agenda {
                        trait_block(ui, &format!("Agenda: {name}"), desc);
                    }
                    for t in &leader.traits {
                        trait_block(ui, &t.name, &t.description);
                    }
                });
        });
    });
    let y = ui.cursor().top() + 4.0;
    theme::divider(
        ui.painter(),
        ui.min_rect().left(),
        ui.max_rect().right(),
        y,
        palette::with_alpha(palette::GOLD_DIM, 220),
    );
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        let say = |s: &mut Session, event: DiploEvent| {
            let line = s.game.dialogue(other, event);
            s.toast(format!("{}: \"{line}\"", leader.name));
        };
        if !at_war {
            if !s.game.players[me].relations[other].friendship
                && theme::pill_button(
                    ui,
                    Some(Glyph::Heart),
                    "Declare Friendship",
                    palette::GOOD,
                    170.0,
                )
                .clicked()
            {
                let accepted = s.game.propose_friendship(me, other);
                say(
                    s,
                    if accepted {
                        DiploEvent::AcceptFriendFromHuman
                    } else {
                        DiploEvent::RejectFriendFromHuman
                    },
                );
            }
            if !s.game.players[me].relations[other].denounced
                && theme::pill_button(
                    ui,
                    Some(Glyph::Bell),
                    "Denounce",
                    Color32::from_rgb(240, 150, 60),
                    140.0,
                )
                .clicked()
            {
                s.game.denounce(me, other);
                say(s, DiploEvent::DenounceFromHuman);
            }
            if theme::pill_button(ui, Some(Glyph::Swords), "Declare War", palette::BAD, 150.0)
                .clicked()
            {
                s.game.declare_war(me, other);
                say(s, DiploEvent::DeclareWarFromHuman);
            }
        } else {
            let can = s.game.can_make_peace(me, other);
            let response = theme::pill_button_if(
                ui,
                Some(Glyph::Shield),
                "Propose Peace",
                palette::TEAL,
                170.0,
                can,
            )
            .on_disabled_hover_text("Wars last at least 10 turns.");
            if response.clicked() {
                let accepted = s.game.propose_peace(me, other);
                say(
                    s,
                    if accepted {
                        DiploEvent::MakePeaceAccept
                    } else {
                        DiploEvent::MakePeaceRefuse
                    },
                );
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
    let key = Id::unique("leader_popup");
    // `windows` dims the screen itself so the backdrop eases in and out.
    let shown = egui::Modal::new(key)
        .backdrop_color(Color32::TRANSPARENT)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            theme::plate_ui(ui, 24, |ui| {
                ui.set_width(880.0);
                let side = (ctx.content_rect().height() - 220.0).clamp(240.0, 380.0);
                ui.horizontal_top(|ui| {
                    let frame = portraits::FRAME;
                    let (rect, _) = ui.allocate_exact_size(
                        vec2(side + 2.0 * frame, side + 2.0 * frame),
                        Sense::hover(),
                    );
                    portraits.paint(
                        ui,
                        rect.shrink(frame),
                        &leader.id,
                        &leader.name,
                        (o.primary, o.secondary),
                    );
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 10.0;
                        ui.label(theme::title(&leader.name, 34.0));
                        ui.label(theme::bold(&o.civ_name, 19.0));
                        quote(ui, &line, 18.0);
                        if event == DiploEvent::DeclareFriendFromAi {
                            ui.label(format!(
                                "{} proposes a declaration of friendship.",
                                leader.name
                            ));
                        }
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 12.0;
                            match event {
                                DiploEvent::DeclareFriendFromAi => {
                                    if theme::pill_button(
                                        ui,
                                        Some(Glyph::Heart),
                                        "Accept",
                                        palette::GOOD,
                                        130.0,
                                    )
                                    .clicked()
                                    {
                                        s.game.propose_friendship(other, me);
                                        close = true;
                                    }
                                    if theme::pill_button(
                                        ui,
                                        None,
                                        "Decline",
                                        palette::TEXT_DIM,
                                        130.0,
                                    )
                                    .clicked()
                                    {
                                        close = true;
                                    }
                                }
                                DiploEvent::DeclareWarFromAi => {
                                    if theme::pill_button(
                                        ui,
                                        Some(Glyph::Swords),
                                        "So be it.",
                                        palette::BAD,
                                        150.0,
                                    )
                                    .clicked()
                                    {
                                        close = true;
                                    }
                                }
                                _ => {
                                    if theme::pill_button(
                                        ui,
                                        None,
                                        "Farewell",
                                        palette::TEXT,
                                        150.0,
                                    )
                                    .clicked()
                                    {
                                        close = true;
                                    }
                                }
                            }
                        });
                    });
                });
            });
        });
    anim::window_fx(ctx, key, shown.response.layer_id, shown.response.rect);
    if close {
        s.popup = None;
    }
}

#[allow(dead_code)]
fn unused(_: &Game) {}

/// The game's name in gold display type with a soft shadow, centered on `c`.
fn wordmark(painter: &egui::Painter, c: Pos2, size: f32) {
    theme::text_display(
        painter,
        c + vec2(0.0, size * 0.04),
        Align2::CENTER_CENTER,
        "WAYCIV",
        size,
        Color32::from_black_alpha(170),
    );
    theme::text_display(painter, c, Align2::CENTER_CENTER, "WAYCIV", size, palette::GOLD_HI);
}

/// The loading screen: the wordmark over a hexagon emblem circled by a
/// spinner.
pub fn loading(ui: &mut egui::Ui) {
    let rect = ui.max_rect();
    let painter = ui.painter().clone();
    theme::backdrop(&painter, rect);
    let t = ui.input(|i| i.time) as f32;
    let c = pos2(rect.center().x, rect.center().y - 70.0);
    theme::disc(&painter, c, 44.0, palette::NAVY_RAISED, GOLD);
    ui::glyph::paint(&painter, Glyph::Hex, c, 18.0, GOLD);
    // A comet of gold circling the emblem.
    let head = t * 3.2;
    for i in 0..24 {
        let a = head - i as f32 * 0.07;
        let alpha = (255.0 * (1.0 - i as f32 / 24.0).powi(2)) as u8;
        painter.circle_filled(
            c + vec2(a.cos(), a.sin()) * 58.0,
            3.6 - i as f32 * 0.1,
            palette::with_alpha(palette::GOLD_HI, alpha),
        );
    }
    wordmark(&painter, pos2(rect.center().x, c.y + 120.0), 76.0);
    theme::divider(
        &painter,
        rect.center().x - 260.0,
        rect.center().x + 260.0,
        c.y + 178.0,
        palette::with_alpha(GOLD, 220),
    );
    theme::text(
        &painter,
        pos2(rect.center().x, c.y + 212.0),
        Align2::CENTER_CENTER,
        "Loading...",
        17.0,
        palette::TEXT_DIM,
    );
    ui.ctx().request_repaint();
}

/// Shown when the game's content can't be read.
pub fn failed(ui: &mut egui::Ui, error: &str) {
    let rect = ui.max_rect();
    theme::backdrop(ui.painter(), rect);
    let width = 760.0_f32.min(rect.width() - 40.0);
    let top = (rect.center().y - 150.0).max(rect.top() + 100.0);
    wordmark(ui.painter(), pos2(rect.center().x, top - 56.0), 56.0);
    let area = Rect::from_min_size(
        pos2(rect.center().x - width / 2.0, top),
        vec2(width, rect.bottom() - top),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(area), |ui| {
        theme::plate_ui(ui, 26, |ui| {
            ui.set_width(width - 52.0);
            ui.label(theme::title("WayCiv couldn't load its content", 30.0));
            let y = ui.cursor().top() + 6.0;
            theme::divider(
                ui.painter(),
                ui.min_rect().left(),
                ui.max_rect().right(),
                y,
                palette::with_alpha(GOLD, 220),
            );
            ui.add_space(18.0);
            theme::well_ui(
                ui,
                12,
                Color32::from_rgb(40, 16, 18),
                palette::with_alpha(palette::BAD, 160),
                |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(error).size(14.5).color(palette::BAD));
                },
            );
            ui.add_space(12.0);
            ui.label(
                RichText::new("The content is compiled into the game from crates/wayciv-data/data; this means a data file has an error. Rebuild after fixing it.")
                    .size(15.5)
                    .color(palette::TEXT),
            );
        });
    });
}

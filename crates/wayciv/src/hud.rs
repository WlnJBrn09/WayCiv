//! The in-game screen: top bar, leader ribbon, unit and city panels, End
//! Turn button, notifications, and leader pop-ups.

use crate::App;
use crate::map_view::{self, Camera};
use crate::palette::{self, GOLD, PANEL, PANEL_LIGHT, YIELD_COLORS, rgb};
use crate::screens::{self, Windows};
use egui::{Align2, Color32, RichText, Sense, vec2};
use wayciv_core::diplomacy::DiploEvent;
use wayciv_core::research::Tree;
use wayciv_core::rules::YIELD_NAMES;
use wayciv_core::state::Production;
use wayciv_core::turn::Blocker;
use wayciv_core::{CityId, Game, PlayerId, Pos, UnitId};

pub struct Session {
    pub game: Game,
    pub cam: Camera,
    pub selected_unit: Option<UnitId>,
    pub selected_city: Option<CityId>,
    pub hover: Option<Pos>,
    pub reachable: Vec<Pos>,
    pub path_preview: Vec<Pos>,
    /// A district or wonder waiting for the player to pick its tile.
    pub placing: Option<Production>,
    /// The selected city is choosing a ranged-attack target.
    pub city_targeting: bool,
    pub windows: Windows,
    pub popup: Option<(PlayerId, DiploEvent)>,
    pub message: Option<(String, std::time::Instant)>,
    pub notifications_seen: usize,
    pub icons: crate::icons::Icons,
    pub art: crate::render::Art,
}

impl Session {
    pub fn new(game: Game) -> Self {
        let human = game.human();
        let start = game
            .player_units(human)
            .next()
            .map(|u| u.pos)
            .unwrap_or_default();
        let mut s = Session {
            game,
            cam: Camera {
                center: map_view::world(start),
                zoom: 34.0,
            },
            selected_unit: None,
            selected_city: None,
            hover: None,
            reachable: Vec::new(),
            path_preview: Vec::new(),
            placing: None,
            city_targeting: false,
            windows: Windows::default(),
            popup: None,
            message: None,
            notifications_seen: 0,
            icons: crate::icons::Icons::new(crate::portraits::assets_dir()),
            art: crate::render::Art::new(crate::portraits::assets_dir()),
        };
        let first = s
            .game
            .player_units(human)
            .find(|u| s.game.unit_type(u).found_city)
            .map(|u| u.id);
        s.select_unit(first);
        s
    }

    pub fn me(&self) -> PlayerId {
        self.game.human()
    }

    pub fn toast(&mut self, text: impl Into<String>) {
        self.message = Some((text.into(), std::time::Instant::now()));
    }

    pub fn result(&mut self, r: Result<(), String>) {
        if let Err(e) = r {
            self.toast(e);
        }
    }

    pub fn select_unit(&mut self, unit: Option<UnitId>) {
        self.selected_unit = unit;
        self.selected_city = None;
        self.placing = None;
        self.city_targeting = false;
        self.refresh_selection();
    }

    pub fn select_city(&mut self, city: Option<CityId>) {
        self.selected_city = city;
        self.selected_unit = None;
        self.placing = None;
        self.city_targeting = false;
        self.reachable.clear();
        self.path_preview.clear();
    }

    pub fn refresh_selection(&mut self) {
        if self
            .selected_unit
            .is_some_and(|id| !self.game.units.contains_key(&id))
        {
            self.selected_unit = None;
        }
        self.reachable = self
            .selected_unit
            .and_then(|id| self.game.units.get(&id))
            .map(|u| self.game.reachable(u))
            .unwrap_or_default();
        self.update_path_preview();
    }

    pub fn update_path_preview(&mut self) {
        self.path_preview.clear();
        if let (Some(unit), Some(goal)) = (
            self.selected_unit.and_then(|id| self.game.units.get(&id)),
            self.hover,
        ) && unit.owner == self.me()
            && goal != unit.pos
            && let Some(path) = self.game.find_path(unit, goal)
        {
            self.path_preview = path;
        }
    }

    pub fn left_click(&mut self, pos: Pos) {
        let me = self.me();
        // Placing a district or wonder.
        if let (Some(item), Some(city)) = (self.placing.clone(), self.selected_city) {
            let placed = match item {
                Production::District(d, _) => Production::District(d, pos),
                Production::Wonder(w, _) => Production::Wonder(w, pos),
                other => other,
            };
            match self.game.set_production(city, placed) {
                Ok(()) => self.placing = None,
                Err(e) => self.toast(e),
            }
            return;
        }
        if self.city_targeting {
            if let Some(city) = self.selected_city {
                let r = self.game.city_attack(city, pos);
                self.result(r);
            }
            self.city_targeting = false;
            return;
        }
        if let Some(city) = self
            .game
            .city_at(pos)
            .filter(|c| c.owner == me)
            .map(|c| c.id)
        {
            // Clicking an own city again (or a city with no own units) opens it.
            let units: Vec<UnitId> = self
                .game
                .units_at(pos)
                .filter(|u| u.owner == me)
                .map(|u| u.id)
                .collect();
            if units.is_empty()
                || self.selected_city.is_none()
                    && self.selected_unit.is_some_and(|u| units.contains(&u))
            {
                self.select_city(Some(city));
                return;
            }
        }
        let mut units: Vec<UnitId> = self
            .game
            .units_at(pos)
            .filter(|u| u.owner == me)
            .map(|u| u.id)
            .collect();
        units.sort();
        if units.is_empty() {
            if let Some(city) = self.game.city_at(pos).map(|c| c.id) {
                if self.game.cities[&city].owner == me {
                    self.select_city(Some(city));
                }
            } else {
                self.select_unit(None);
                self.select_city(None);
            }
            return;
        }
        let next = match self
            .selected_unit
            .and_then(|s| units.iter().position(|&u| u == s))
        {
            Some(i) => units[(i + 1) % units.len()],
            None => units[0],
        };
        self.select_unit(Some(next));
    }

    pub fn right_click(&mut self, pos: Pos) {
        self.placing = None;
        let Some(unit) = self.selected_unit else {
            return;
        };
        let r = self.game.move_unit(unit, pos);
        self.result(r);
        self.refresh_selection();
    }

    /// Selects the next unit that needs orders.
    pub fn next_unit(&mut self) {
        let me = self.me();
        let waiting: Vec<UnitId> = self
            .game
            .blockers(me)
            .into_iter()
            .filter_map(|b| {
                if let Blocker::UnitNeedsOrders(u) = b {
                    Some(u)
                } else {
                    None
                }
            })
            .collect();
        let next = match self
            .selected_unit
            .and_then(|s| waiting.iter().position(|&u| u == s))
        {
            Some(i) => waiting.get(i + 1).or(waiting.first()).copied(),
            None => waiting.first().copied(),
        };
        if let Some(next) = next {
            self.select_unit(Some(next));
            let pos = self.game.units[&next].pos;
            map_view::center_on(self, pos);
        }
    }

    /// The End Turn button: resolve the first blocker, or end the turn.
    pub fn end_turn_clicked(&mut self) {
        let me = self.me();
        match self.game.blockers(me).first().cloned() {
            Some(Blocker::ChooseResearch) => self.windows.tech = true,
            Some(Blocker::ChooseCivic) => self.windows.civic = true,
            Some(Blocker::ChoosePantheon) => self.windows.religion = true,
            Some(Blocker::ChooseAiResearch) => self.windows.ai = true,
            Some(Blocker::ChooseProduction(city)) => {
                self.select_city(Some(city));
                let pos = self.game.cities[&city].pos;
                map_view::center_on(self, pos);
            }
            Some(Blocker::UnitNeedsOrders(unit)) => {
                self.select_unit(Some(unit));
                let pos = self.game.units[&unit].pos;
                map_view::center_on(self, pos);
            }
            None => self.end_turn(),
        }
    }

    pub fn end_turn(&mut self) {
        self.game.end_turn();
        self.refresh_selection();
        if self.selected_unit.is_none() && self.selected_city.is_none() {
            self.next_unit();
        }
    }
}

fn year(turn: u32) -> String {
    // Civ VI standard-speed calendar, simplified.
    let mut y: i32 = -4000;
    for t in 1..turn {
        y += match t {
            0..=75 => 40,
            76..=135 => 25,
            136..=195 => 20,
            196..=255 => 10,
            256..=315 => 5,
            316..=375 => 2,
            _ => 1,
        };
    }
    if y < 0 {
        format!("{} BC", -y)
    } else {
        format!("{y} AD")
    }
}

pub fn tile_tooltip(s: &Session, pos: Pos) -> String {
    let game = &s.game;
    let content = &game.rules.content;
    let me = s.me();
    let tile = game.map.tile(pos);
    let terrain = &content.terrains[tile.terrain];
    let mut lines = vec![];
    let relief = if terrain.mountain {
        " (Mountains)"
    } else if terrain.hills {
        " (Hills)"
    } else {
        ""
    };
    lines.push(format!("{}{relief}", palette::biome_name(&terrain.id)));
    if let Some(f) = tile.feature {
        lines.push(content.features[f].name.clone());
    }
    if tile.river {
        lines.push("River".into());
    }
    if tile.cliffs {
        lines.push("Cliffs (+3 defense)".into());
    }
    if let Some(r) = tile.resource {
        let res = &content.resources[r];
        if game.players[me].meets(&game.rules, res.prereq_tech.as_deref(), None) {
            let class = res
                .class
                .trim_start_matches("RESOURCECLASS_")
                .to_lowercase();
            lines.push(format!("{} ({class} resource)", res.name));
        }
    }
    if let Some(i) = tile.improvement {
        lines.push(format!(
            "{}{}",
            content.improvements[i].name,
            if tile.pillaged { " (pillaged)" } else { "" }
        ));
    }
    if let Some(d) = tile.district.filter(|&d| !content.districts[d].city_center) {
        lines.push(content.districts[d].name.clone());
    }
    if let Some(w) = tile.wonder {
        lines.push(content.buildings[w].name.clone());
    }
    let y = game.tile_yields(me, pos);
    let yields: Vec<String> = (0..6)
        .filter(|&i| y[i] != 0)
        .map(|i| format!("{} {}", y[i], YIELD_NAMES[i]))
        .collect();
    if !yields.is_empty() {
        lines.push(yields.join(", "));
    }
    if let Some(owner) = tile.owner {
        let city = tile
            .owning_city
            .and_then(|c| game.cities.get(&c))
            .map(|c| c.name.clone())
            .unwrap_or_default();
        lines.push(format!(
            "Territory of {} ({city})",
            game.players[owner].civ_name
        ));
    }
    if game.players[me].visible[game.map.grid.index(pos)] {
        for u in game.units_at(pos) {
            let ty = game.unit_type(u);
            if game.is_military(u) {
                let strength = if ty.ranged_combat > 0 {
                    format!("{} / ranged {}", ty.combat, ty.ranged_combat)
                } else {
                    ty.combat.to_string()
                };
                lines.push(format!(
                    "{} {} - strength {strength}, {} HP",
                    game.players[u.owner].civ_name, ty.name, u.hp
                ));
            } else {
                lines.push(format!(
                    "{} {} (civilian: one attack kills it)",
                    game.players[u.owner].civ_name, ty.name
                ));
            }
        }
    }
    if let Some(unit) = s.selected_unit
        && game
            .units
            .get(&unit)
            .is_some_and(|u| game.can_attack(u, pos))
        && let Some(p) = game.preview(unit, pos)
    {
        lines.push(format!(
            "Attack: {} vs {}. Expected damage dealt {}, taken {}.",
            p.attack, p.defense, p.damage_to_defender, p.damage_to_attacker
        ));
    }
    lines.join("\n")
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let crate::Stage::Playing(session) = &mut app.stage else {
        return;
    };
    let s: &mut super::hud::Session = session;
    let me = s.me();

    // Pop-ups queued by the engine (first meetings, declarations of war...).
    if s.popup.is_none() && !s.game.popups.is_empty() {
        s.popup = Some(s.game.popups.remove(0));
    }

    shortcuts(s, &ctx);
    top_bar(s, &mut app.portraits, ui);
    egui::Panel::bottom("bottom_bar")
        .exact_size(150.0)
        .frame(egui::Frame::new().fill(PANEL).inner_margin(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width((ui.available_width() - 330.0).max(300.0));
                    if s.selected_city.is_some() {
                        city_summary(s, ui);
                    } else {
                        unit_panel(s, ui);
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    end_turn_button(s, ui)
                });
            });
        });
    if s.selected_city.is_some() {
        egui::Panel::left("city_panel")
            .exact_size(360.0)
            .frame(egui::Frame::new().fill(PANEL).inner_margin(8))
            .show(ui, |ui| city_panel(s, ui));
    }
    egui::Panel::right("notifications")
        .exact_size(250.0)
        .frame(egui::Frame::new().fill(PANEL).inner_margin(8))
        .show(ui, |ui| notifications(s, ui));
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ui, |ui| {
            map_view::show(s, ui);
            legend(s, ui);
        });

    screens::windows(s, &mut app.portraits, &ctx);
    if let Some((text, at)) = &s.message {
        if at.elapsed().as_secs_f32() < 3.0 {
            egui::Area::new(egui::Id::unique("toast"))
                .anchor(Align2::CENTER_TOP, vec2(0.0, 90.0))
                .show(&ctx, |ui| {
                    egui::Frame::popup(ui.style())
                        .show(ui, |ui| ui.label(RichText::new(text).size(16.0)));
                });
            ctx.request_repaint();
        } else {
            s.message = None;
        }
    }
    let _ = me;
}

fn shortcuts(s: &mut Session, ctx: &egui::Context) {
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    let (enter, escape, space, fortify, found) = ctx.input(|i| {
        (
            i.key_pressed(egui::Key::Enter),
            i.key_pressed(egui::Key::Escape),
            i.key_pressed(egui::Key::Space),
            i.key_pressed(egui::Key::F),
            i.key_pressed(egui::Key::B),
        )
    });
    if enter {
        s.end_turn_clicked();
    }
    if escape {
        s.placing = None;
        s.city_targeting = false;
        s.select_unit(None);
        s.windows.close_all();
    }
    if let Some(unit) = s.selected_unit {
        if space {
            s.game.skip(unit);
            s.next_unit();
        }
        if fortify {
            s.game.fortify(unit);
            s.next_unit();
        }
        if found && s.game.can_found_city(unit) {
            let r = s.game.found_city(unit);
            s.result(r);
            s.refresh_selection();
        }
    }
}

fn top_bar(s: &mut Session, portraits: &mut crate::portraits::Portraits, ui: &mut egui::Ui) {
    let me = s.me();
    let income = s.game.income(me);
    egui::Panel::top("top_bar").exact_size(64.0).frame(egui::Frame::new().fill(PANEL).inner_margin(6)).show(ui, |ui| {
        ui.horizontal_centered(|ui| {
            let p = &s.game.players[me];
            let y = income.yields;
            let stat = |ui: &mut egui::Ui, color: Color32, text: String, tip: &str| {
                ui.label(RichText::new(text).color(color).size(16.0).strong()).on_hover_text(tip);
            };
            stat(ui, YIELD_COLORS[3], format!("Science +{:.1}", y[3]), "Science per turn");
            stat(ui, YIELD_COLORS[4], format!("Culture +{:.1}", y[4]), "Culture per turn");
            stat(ui, YIELD_COLORS[5], format!("Faith {} (+{:.0})", p.faith, y[5]), "Faith");
            stat(ui, YIELD_COLORS[2], format!("Gold {} ({:+})", p.gold, income.net_gold), "Gold, and net per turn after upkeep");
            stat(ui, Color32::from_rgb(230, 120, 160), format!("Tourism {}", income.tourism), "Tourism per turn");
            let routes = s.game.trade_routes_of(me).count();
            stat(ui, Color32::from_rgb(220, 180, 110), format!("Trade {}/{}", routes, s.game.trade_capacity(me)), "Trade routes in use / capacity. Build Traders (Foreign Trade civic); Commercial Hubs and Harbors add capacity.");
            ui.separator();
            let research = p.research.map(|i| {
                let t = &s.game.rules.content.techs[i];
                let left = (s.game.research_cost(Tree::Tech, i) - p.tech_progress[i]).max(0) as f32;
                format!("{} ({}t)", t.name, (left / y[3].max(0.1)).ceil())
            });
            if ui.button(RichText::new(research.unwrap_or("Choose Research".into())).color(YIELD_COLORS[3])).clicked() {
                s.windows.tech = true;
            }
            let civic = p.civic.map(|i| {
                let t = &s.game.rules.content.civics[i];
                let left = (s.game.research_cost(Tree::Civic, i) - p.civic_progress[i]).max(0) as f32;
                format!("{} ({}t)", t.name, (left / y[4].max(0.1)).ceil())
            });
            if ui.button(RichText::new(civic.unwrap_or("Choose Civic".into())).color(YIELD_COLORS[4])).clicked() {
                s.windows.civic = true;
            }
            if s.game.ai_tree_unlocked(me) {
                let compute = s.game.compute(me);
                let ai = p.ai_research.map(|i| {
                    let left = (s.game.research_cost(Tree::Ai, i) - p.ai_progress[i]).max(0) as f32;
                    let turns = if compute > 0.0 { format!("{}t", (left / compute).ceil()) } else { "no Compute".into() };
                    format!("{} ({turns})", s.game.rules.ai_tree[i].name)
                });
                let label = RichText::new(ai.unwrap_or("Choose AI Research".into())).color(crate::palette::COMPUTE);
                if ui.button(label).on_hover_text(format!("AI tree. Compute +{compute:.0}/turn from Datacenters.")).clicked() {
                    s.windows.ai = true;
                }
            }
            for (label, flag) in [
                ("Government", &mut s.windows.government),
                ("Religion", &mut s.windows.religion),
                ("Great People", &mut s.windows.great_people),
                ("City-States", &mut s.windows.city_states),
                ("Victory", &mut s.windows.victory),
            ] {
                if ui.button(label).clicked() {
                    *flag = !*flag;
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("Turn {}  |  {}", s.game.turn, year(s.game.turn))).size(16.0).color(GOLD));
                ui.separator();
                // Leader ribbon: everyone you've met.
                let met: Vec<PlayerId> = s
                    .game
                    .players
                    .iter()
                    .filter(|o| o.id != me && o.is_major() && s.game.players[me].relations[o.id].met)
                    .map(|o| o.id)
                    .collect();
                for other in met.into_iter().rev() {
                    let o = &s.game.players[other];
                    let leader = &s.game.rules.content.leaders[o.leader];
                    let (rect, response) = ui.allocate_exact_size(vec2(50.0, 50.0), Sense::click());
                    portraits.paint(ui, rect, &leader.id, &leader.name, (o.primary, o.secondary));
                    if !o.alive {
                        ui.painter().rect_filled(rect, 4.0, Color32::from_black_alpha(170));
                    }
                    if s.game.players[me].at_war_with(other) {
                        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(3.0, Color32::from_rgb(230, 40, 30)), egui::StrokeKind::Outside);
                    }
                    let status = if s.game.players[me].at_war_with(other) { "At war" } else if s.game.players[me].relations[other].friendship { "Friends" } else { "At peace" };
                    if response.on_hover_text(format!("{} of {} ({status})", o.name, o.civ_name)).clicked() {
                        s.windows.diplomacy = Some(other);
                    }
                }
                let leader = &s.game.rules.content.leaders[s.game.players[me].leader];
                let (rect, _) = ui.allocate_exact_size(vec2(50.0, 50.0), Sense::hover());
                let pl = &s.game.players[me];
                portraits.paint(ui, rect, &leader.id, &leader.name, (pl.primary, pl.secondary));
            });
        });
    });
}

fn end_turn_button(s: &mut Session, ui: &mut egui::Ui) {
    let me = s.me();
    let blockers = s.game.blockers(me);
    let (text, color) = match blockers.first() {
        Some(b) => (s.game.blocker_text(b), Color32::from_rgb(70, 90, 140)),
        None => ("End Turn".to_string(), Color32::from_rgb(40, 120, 70)),
    };
    ui.vertical(|ui| {
        if let Some((winner, victory)) = &s.game.winner {
            let name = s.game.players[*winner].civ_name.clone();
            ui.label(
                RichText::new(format!("{name} won a {victory:?} victory!"))
                    .color(GOLD)
                    .size(18.0),
            );
        }
        let button = egui::Button::new(RichText::new(text).size(22.0).strong())
            .fill(color)
            .min_size(vec2(300.0, 70.0));
        if ui.add(button).on_hover_text("Enter").clicked() {
            s.end_turn_clicked();
        }
        ui.horizontal(|ui| {
            if ui.button("Next Unit").clicked() {
                s.next_unit();
            }
            if !blockers.is_empty()
                && ui
                    .button("End Turn Anyway")
                    .on_hover_text("Skips remaining unit orders")
                    .clicked()
            {
                let only_units = blockers
                    .iter()
                    .all(|b| matches!(b, Blocker::UnitNeedsOrders(_)));
                if only_units {
                    s.end_turn();
                } else {
                    s.toast("Choose research, civics, and production first.");
                }
            }
        });
    });
}

fn unit_panel(s: &mut Session, ui: &mut egui::Ui) {
    let Some(id) = s.selected_unit else {
        ui.label(RichText::new("Left-click a unit or city to select it. Right-click to move or attack. Drag or WASD to scroll; mouse wheel to zoom.").color(Color32::from_gray(170)));
        return;
    };
    let Some(unit) = s.game.units.get(&id).cloned() else {
        return;
    };
    let ty = s.game.unit_type(&unit).clone();
    let owner = &s.game.players[unit.owner];
    ui.horizontal(|ui| {
        let (swatch, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
        ui.painter().rect_filled(swatch, 3.0, rgb(owner.primary));
        ui.painter().rect_stroke(
            swatch,
            3.0,
            egui::Stroke::new(1.5, rgb(owner.secondary)),
            egui::StrokeKind::Inside,
        );
        ui.label(
            RichText::new(&ty.name)
                .size(20.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.label(RichText::new(format!("({})", owner.civ_name)).color(Color32::from_gray(160)));
        if let Some(gp) = unit.great_person {
            ui.label(RichText::new(&s.game.rules.content.great_people[gp].name).color(GOLD));
        }
    });
    // Civilians have no health: they are alive or dead.
    let health = if s.game.is_military(&unit) {
        format!("   HP {}/100", unit.hp)
    } else {
        "   Civilian".to_string()
    };
    let mut stats = format!("Moves {}/{}{health}", unit.moves, ty.moves);
    if ty.combat > 0 {
        stats += &format!("   Strength {}", ty.combat);
    }
    if ty.ranged_combat > 0 {
        stats += &format!("   Ranged {} (range {})", ty.ranged_combat, ty.range);
    }
    if ty.build_charges > 0 || ty.spread_charges > 0 {
        stats += &format!("   Charges {}", unit.charges);
    }
    if unit.level > 0 || unit.xp > 0 {
        stats += &format!("   Level {} ({} XP)", unit.level, unit.xp);
    }
    ui.label(stats);
    if let Some(gp) = unit.great_person {
        ui.label(RichText::new(s.game.great_person_effect(gp)).italics());
    }
    if s.game.is_air(&unit) {
        ui.label(RichText::new(format!("Operation range {}. Right-click a target in range to strike, or another Aerodrome to rebase.", ty.range)).italics());
    }
    if ty.make_trade_route && s.game.trade_destinations(id).is_empty() {
        ui.label(
            RichText::new("Move this Trader into one of your cities to start a trade route.")
                .italics(),
        );
    }
    if unit.owner != s.me() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        for action in s.game.unit_actions(id) {
            let name = s.game.action_name(&action);
            if ui.button(name).clicked() {
                let r = s.game.perform(id, &action);
                s.result(r);
                s.refresh_selection();
                if !s.game.units.contains_key(&id)
                    || matches!(
                        action,
                        wayciv_core::actions::UnitAction::Skip
                            | wayciv_core::actions::UnitAction::Fortify
                    )
                {
                    s.next_unit();
                }
            }
        }
    });
}

fn city_summary(s: &mut Session, ui: &mut egui::Ui) {
    let Some(id) = s.selected_city else { return };
    let report = s.game.city_report(id);
    let c = &s.game.cities[&id];
    ui.label(
        RichText::new(format!("{} (population {})", c.name, c.population))
            .size(20.0)
            .strong(),
    );
    ui.horizontal(|ui| {
        for i in 0..6 {
            ui.label(
                RichText::new(format!("{} {:.1}", YIELD_NAMES[i], report.yields[i]))
                    .color(YIELD_COLORS[i]),
            );
        }
    });
    if s.placing.is_some() {
        ui.label(RichText::new("Click a highlighted tile to place it (Esc to cancel). Numbers show adjacency bonuses.").color(GOLD));
    }
    if s.city_targeting {
        ui.label(RichText::new("Click an enemy unit within 2 tiles to bombard it.").color(GOLD));
    }
}

fn city_panel(s: &mut Session, ui: &mut egui::Ui) {
    let Some(id) = s.selected_city else { return };
    if !s.game.cities.contains_key(&id) {
        s.select_city(None);
        return;
    }
    let report = s.game.city_report(id);
    let c = s.game.cities[&id].clone();
    let me = s.me();
    ui.horizontal(|ui| {
        ui.heading(RichText::new(&c.name).color(GOLD));
        if ui.button("Close").clicked() {
            s.select_city(None);
        }
    });
    if s.selected_city.is_none() {
        return;
    }
    let surplus = report.yields[0] - report.food_consumed;
    let grow = report
        .turns_to_grow
        .map(|t| format!("grows in {t} turns"))
        .unwrap_or_else(|| {
            if surplus < 0.0 {
                "starving!".into()
            } else {
                "not growing".into()
            }
        });
    ui.label(format!(
        "Population {} - food {}/{} ({surplus:+.1}), {grow}",
        c.population, c.food, report.growth_threshold
    ));
    ui.label(format!(
        "Housing {}   Amenities {}/{}   HP {}/200   Walls {}",
        report.housing, report.amenities, report.amenities_needed, c.hp, c.wall_hp
    ));
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
            Color32::from_rgb(240, 90, 80)
        };
        ui.label(
            RichText::new(format!(
                "Loyalty {}/100 ({:+}/turn)",
                c.loyalty, loyalty.per_turn
            ))
            .color(color),
        )
        .on_hover_text(format!(
            "At 0 the city joins the civilization pulling on it hardest.\n{}",
            breakdown.join("\n")
        ));
    }
    let compute = s.game.city_compute(&c);
    if compute > 0.0 {
        ui.label(
            RichText::new(format!("Datacenter: +{compute:.0} Compute"))
                .color(crate::palette::COMPUTE),
        );
    }
    if let Some(r) = c.majority_religion() {
        ui.label(format!(
            "Religion: {}",
            s.game.rules.content.religions[r].name
        ));
    }
    if s.game.max_walls(id) > 0 && !c.has_attacked && ui.button("City Ranged Attack").clicked() {
        s.city_targeting = true;
    }
    ui.separator();
    if let Some(item) = &c.production {
        let cost = s.game.production_cost(me, item);
        let done = c
            .progress
            .get(&Game::production_key(item))
            .copied()
            .unwrap_or(0);
        let rate = s.game.production_toward(id, item).max(0.1);
        ui.label(
            RichText::new(format!(
                "Producing: {} ({}/{}, {} turns)",
                s.game.item_name(item),
                done,
                cost,
                ((cost - done) as f32 / rate).ceil()
            ))
            .color(YIELD_COLORS[1])
            .strong(),
        );
        if let Some(price) = s.game.purchase_cost(id, item)
            && ui.button(format!("Buy now for {price} Gold")).clicked()
        {
            let r = s.game.purchase(id, item.clone());
            s.result(r);
        }
    } else {
        ui.label(
            RichText::new("Choose what to produce:")
                .color(GOLD)
                .strong(),
        );
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        let options = s.game.available_production(id);
        for (title, filter) in [
            ("Districts", 0u8),
            ("Buildings", 1),
            ("Wonders", 2),
            ("Units", 3),
            ("Projects", 4),
        ] {
            let items: Vec<&Production> = options
                .iter()
                .filter(|p| match (filter, p) {
                    (0, Production::District(..)) => true,
                    (1, Production::Building(_)) => true,
                    (2, Production::Wonder(..)) => true,
                    (3, Production::Unit(_)) => true,
                    (4, Production::Project(_)) => true,
                    _ => false,
                })
                .collect();
            if items.is_empty() {
                continue;
            }
            ui.label(RichText::new(title).strong().color(Color32::from_gray(200)));
            for item in items {
                let cost = s.game.production_cost(me, item);
                let rate = s.game.production_toward(id, item).max(0.1);
                let done = c
                    .progress
                    .get(&Game::production_key(item))
                    .copied()
                    .unwrap_or(0);
                let turns = ((cost - done).max(0) as f32 / rate).ceil();
                let name = s.game.item_name(item);
                let price = s.game.purchase_cost(id, item);
                let response = ui
                    .horizontal(|ui| {
                        if let Production::Unit(u) = item {
                            let unit_id = s.game.rules.content.units[*u].id.clone();
                            s.icons.show(ui, "unit", &unit_id, 22.0);
                        }
                        let label = match price {
                            Some(p) => format!("{name}  -  {cost} ({turns}t)  |  {p} Gold"),
                            None => format!("{name}  -  {cost} ({turns}t)"),
                        };
                        ui.add(egui::Button::new(label).min_size(vec2(300.0, 22.0)))
                    })
                    .inner;
                let tip = match price {
                    Some(p) => format!(
                        "{}\n\nRight-click to buy now for {p} Gold.",
                        item_tooltip(&s.game, item)
                    ),
                    None => item_tooltip(&s.game, item),
                };
                let response = response.on_hover_text(tip);
                if response.secondary_clicked() && price.is_some() {
                    let r = s.game.purchase(id, item.clone());
                    if r.is_ok() {
                        s.toast(format!("Bought {name}."));
                    }
                    s.result(r);
                }
                if response.clicked() {
                    match item {
                        Production::District(..) | Production::Wonder(..) => {
                            s.placing = Some(item.clone());
                        }
                        _ => {
                            let r = s.game.set_production(id, item.clone());
                            s.result(r);
                        }
                    }
                }
            }
        }
        let faith_units = s.game.faith_units(id);
        if !faith_units.is_empty() {
            ui.label(
                RichText::new("Buy with Faith")
                    .strong()
                    .color(YIELD_COLORS[5]),
            );
            for unit in faith_units {
                let cost = s.game.faith_cost(unit);
                if ui
                    .button(format!(
                        "{} - {cost} Faith",
                        s.game.rules.content.units[unit].name
                    ))
                    .clicked()
                {
                    let r = s.game.buy_with_faith(id, unit);
                    s.result(r);
                }
            }
        }
        let routes: Vec<_> = s
            .game
            .trade_routes
            .iter()
            .filter(|r| r.origin == id)
            .cloned()
            .collect();
        if !routes.is_empty() {
            ui.separator();
            ui.label(RichText::new("Trade routes").strong());
            for r in routes {
                let y = s.game.route_yields(r.owner, r.destination);
                let parts: Vec<String> = (0..6)
                    .filter(|&i| y[i] > 0.0)
                    .map(|i| format!("+{:.0} {}", y[i], YIELD_NAMES[i]))
                    .collect();
                let dest = s
                    .game
                    .cities
                    .get(&r.destination)
                    .map(|c| c.name.clone())
                    .unwrap_or_default();
                ui.label(format!(
                    "  to {dest}: {} ({} turns left)",
                    parts.join(", "),
                    r.turns_left
                ));
            }
        }
        ui.separator();
        ui.label(RichText::new("Districts and buildings").strong());
        let content = &s.game.rules.content;
        for d in &c.districts {
            if !content.districts[d.district].city_center {
                ui.label(format!("  {}", content.districts[d.district].name));
            }
        }
        for &b in &c.buildings {
            ui.label(format!("  {}", content.buildings[b].name));
        }
    });
}

fn item_tooltip(game: &Game, item: &Production) -> String {
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
                .map(|k| format!("+{} {}", b.yields[k], YIELD_NAMES[k]))
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

fn notifications(s: &mut Session, ui: &mut egui::Ui) {
    let me = s.me();
    ui.label(RichText::new("Notifications").strong().color(GOLD));
    egui::ScrollArea::vertical().show(ui, |ui| {
        let items: Vec<_> = s
            .game
            .notifications
            .iter()
            .rev()
            .filter(|n| n.player == me)
            .take(40)
            .cloned()
            .collect();
        for n in items {
            let current = n.turn + 1 >= s.game.turn;
            let text = RichText::new(format!("T{}: {}", n.turn, n.text)).color(if current {
                Color32::WHITE
            } else {
                Color32::from_gray(140)
            });
            let response = ui.add(egui::Label::new(text).sense(Sense::click()));
            if let Some(pos) = n.pos
                && response
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
            {
                map_view::center_on(s, pos);
            }
        }
    });
}

fn legend(s: &mut Session, ui: &mut egui::Ui) {
    let rect = ui.max_rect();
    let art = std::cell::RefCell::new(std::mem::replace(
        &mut s.art,
        crate::render::Art::new(Default::default()),
    ));
    let swatch = |p: &egui::Painter, r: egui::Rect, terrain: &str, feature: Option<&str>| {
        crate::render::terrain::legend_swatch(
            &mut art.borrow_mut().terrain,
            p,
            terrain,
            feature,
            r,
        );
    };
    egui::Area::new(egui::Id::unique("legend")).fixed_pos(rect.left_bottom() + vec2(8.0, -8.0)).pivot(Align2::LEFT_BOTTOM).show(ui.ctx(), |ui| {
        egui::Frame::new().fill(palette::with_alpha(PANEL_LIGHT, 230)).corner_radius(6.0).inner_margin(6).show(ui, |ui| {
            if ui.small_button(if s.windows.legend { "Hide map key" } else { "Map key" }).clicked() {
                s.windows.legend = !s.windows.legend;
            }
            if s.windows.legend {
                egui::Grid::new("legend_grid").num_columns(2).spacing([10.0, 2.0]).show(ui, |ui| {
                    let mut column = 0;
                    let mut cell = |ui: &mut egui::Ui, draw: &dyn Fn(&egui::Painter, egui::Rect), name: &str| {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                            draw(ui.painter(), r);
                            ui.label(name);
                        });
                        column += 1;
                        if column % 2 == 0 {
                            ui.end_row();
                        }
                    };
                    for (id, name) in palette::LEGEND {
                        cell(ui, &|p, r| swatch(p, r, id, None), name);
                    }
                    for (feature, terrain, name) in palette::LEGEND_FEATURES {
                        cell(ui, &|p, r| swatch(p, r, terrain, Some(feature)), name);
                    }
                    cell(ui, &|p, r| {
                        swatch(p, r, "TERRAIN_GRASS", None);
                        p.line_segment([r.left_bottom() + vec2(2.0, -3.0), r.right_bottom() + vec2(-2.0, -3.0)], egui::Stroke::new(3.0, Color32::from_rgb(92, 78, 66)));
                    }, "Cliffs");
                    cell(ui, &|p, r| {
                        swatch(p, r, "TERRAIN_PLAINS", None);
                        p.line_segment([r.left_center(), r.right_center()], egui::Stroke::new(2.5, Color32::from_rgb(60, 130, 220)));
                    }, "River");
                });
                ui.label("Bumps: hills. Gray peaks: mountains. Star: natural wonder.");
                ui.label("Resources: icons ringed green (bonus), purple (luxury), red (strategic).");
                ui.label("Colored edges: borders in each civilization's colors.");
                ui.label("Brown lines: roads. Red tents: barbarian camps.");
                ui.label("Diamonds: aircraft at their Aerodrome. Gold dashes: your trade routes.");
            }
        });
    });
    s.art = art.into_inner();
}

impl Session {
    pub fn research_turns(&self, tree: Tree, i: usize) -> String {
        let me = self.me();
        let income = self.game.income(me);
        let rate = match tree {
            Tree::Tech => income.yields[3],
            Tree::Civic => income.yields[4],
            Tree::Ai => self.game.compute(me),
        }
        .max(0.1);
        let left =
            (self.game.research_cost(tree, i) - self.game.progress(me, tree, i)).max(0) as f32;
        format!("{}t", (left / rate).ceil())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wayciv_core::actions::UnitAction;

    fn session() -> Option<Session> {
        let rules = wayciv_core::load_rules().ok()?;
        let game = wayciv_core::new_game(
            rules,
            wayciv_core::Settings {
                seed: 3,
                ..Default::default()
            },
        );
        Some(Session::new(game))
    }

    #[test]
    fn opening_turns_follow_civ_flow() {
        let Some(mut s) = session() else {
            eprintln!("skipped: Civilization VI is not installed");
            return;
        };
        let me = s.me();
        // The settler starts selected; found the capital with it.
        let settler = s.selected_unit.expect("settler selected at start");
        assert!(s.game.unit_type(&s.game.units[&settler]).found_city);
        s.game.perform(settler, &UnitAction::FoundCity).unwrap();
        let city = s.game.player_cities(me).next().expect("capital founded").id;

        // End Turn first asks for research, then civics, then production.
        let blockers = s.game.blockers(me);
        assert!(blockers.contains(&Blocker::ChooseResearch));
        assert!(blockers.contains(&Blocker::ChooseProduction(city)));
        s.end_turn_clicked();
        assert!(
            s.windows.tech,
            "End Turn opens the tech tree when research is unset"
        );
        let tech = (0..s.game.rules.content.techs.len())
            .find(|&i| s.game.researchable(me, Tree::Tech, i))
            .unwrap();
        s.game.set_research(me, Tree::Tech, tech).unwrap();
        let civic = (0..s.game.rules.content.civics.len())
            .find(|&i| s.game.researchable(me, Tree::Civic, i))
            .unwrap();
        s.game.set_research(me, Tree::Civic, civic).unwrap();

        // Clicking the city tile selects it; choose a Warrior.
        let pos = s.game.cities[&city].pos;
        s.select_unit(None);
        s.left_click(pos);
        assert_eq!(s.selected_city, Some(city));
        let warrior = s.game.available_production(city).into_iter().find(|p| matches!(p, Production::Unit(i) if s.game.rules.content.units[*i].id == "UNIT_WARRIOR")).unwrap();
        s.game.set_production(city, warrior).unwrap();

        // Right-click moves the starting warrior one tile.
        let warrior_unit = s
            .game
            .player_units(me)
            .find(|u| s.game.is_military(u))
            .map(|u| u.id)
            .unwrap();
        s.select_unit(Some(warrior_unit));
        let target = *s.reachable.first().expect("warrior can move");
        s.right_click(target);
        assert_ne!(s.game.units[&warrior_unit].pos, pos, "warrior moved");

        // Fortify, then End Turn advances the game.
        s.game.fortify(warrior_unit);
        assert!(
            s.game.blockers(me).is_empty(),
            "no blockers left: {:?}",
            s.game.blockers(me)
        );
        let turn = s.game.turn;
        s.end_turn_clicked();
        assert_eq!(s.game.turn, turn + 1);
        assert!(
            s.game.cities[&city].progress.values().any(|&p| p > 0),
            "production progressed"
        );
    }
}

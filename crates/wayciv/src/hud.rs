//! The in-game screen: the `Session` (game state plus UI state), click and
//! turn logic, and the frame that lays out the HUD. The HUD pieces
//! themselves live in `ui/`.

use crate::App;
use crate::map_view::{self, Camera};
use crate::palette::{self, GOLD};
use crate::screens::{self, Panel, Windows};
use crate::ui;
use egui::{Align2, Color32, RichText, Sense, pos2, vec2};
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
    /// Unread count and the fading toasts for new notifications.
    pub notifications: ui::notify::Cursor,
    pub minimap: ui::minimap::Minimap,
    /// The screen rect the map is drawn in.
    pub view: egui::Rect,
    /// Which tab of the city panel is showing.
    pub city_tab: usize,
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
            notifications: Default::default(),
            minimap: Default::default(),
            view: egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1500.0, 860.0)),
            city_tab: 0,
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

    /// Notifications for you that you haven't opened the list for yet.
    pub fn unread_notifications(&self) -> usize {
        self.notifications.unread
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
            match self.game.set_production(city, placed.clone()) {
                Ok(()) => {
                    let name = self.game.cities[&city].name.clone();
                    self.toast(format!(
                        "{name} is building {}.",
                        self.game.item_name(&placed)
                    ));
                    self.select_city(None);
                }
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
            Some(Blocker::ChooseResearch) => self.windows.open(Panel::Tech),
            Some(Blocker::ChooseCivic) => self.windows.open(Panel::Civic),
            Some(Blocker::ChoosePantheon) => self.windows.open(Panel::Religion),
            Some(Blocker::ChooseAiResearch) => self.windows.open(Panel::Ai),
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

/// The calendar year of the current turn, e.g. "1600 BC" (stretched to fit
/// the game's length).
pub fn year(game: &Game) -> String {
    let y = game.calendar_year(game.turn);
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
    let s: &mut Session = session;

    // Pop-ups queued by the engine (first meetings, declarations of war...).
    if s.popup.is_none() && !s.game.popups.is_empty() {
        s.popup = Some(s.game.popups.remove(0));
    }

    ui::keys::handle(s, &ctx);
    ui::topbar::show(s, ui);
    // The map fills everything under the top bar; the HUD floats over it.
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ui, |ui| {
            s.view = ui.max_rect();
            map_view::show(s, ui);
        });

    let screen = ctx.content_rect();
    ui::trackers::show(s, &ctx);
    ui::leaders::ribbon(s, &mut app.portraits, &ctx);
    ui::city_panel::show(s, &ctx);
    ui::unit_panel::show(s, &ctx);
    let turn_left = ui::turn::show(s, &ctx);
    ui::minimap::show(
        s,
        &ctx,
        pos2(turn_left - 14.0, screen.bottom() - ui::MARGIN),
    );
    ui::notify::toasts(s, &ctx);
    ui::notify::panel(s, &ctx);
    legend(s, &ctx);
    banners(s, &ctx);

    screens::windows(s, &mut app.portraits, &ctx);
}

/// Instructions while placing a district or wonder, or aiming the city's
/// ranged attack (with a Cancel button), and the short error toast.
fn banners(s: &mut Session, ctx: &egui::Context) {
    let hint = if let Some(item) = &s.placing {
        Some(format!(
            "Place {}: click a highlighted tile. Numbers show adjacency bonuses.",
            s.game.item_name(item)
        ))
    } else if s.city_targeting {
        Some("Click an enemy unit within 2 tiles to bombard it.".to_string())
    } else {
        None
    };
    let screen = ctx.content_rect();
    // The middle column between the trackers on the left and the leader
    // ribbon / city panel on the right.
    let left = ui::MARGIN + 290.0 + 14.0;
    let right = screen.right()
        - ui::MARGIN
        - if s.selected_city.is_some() {
            ui::city_panel::WIDTH + ui::MARGIN
        } else {
            0.0
        };
    let max_w = (right - left - 24.0).clamp(300.0, 900.0);
    let center_x = ((left + right) / 2.0).min(screen.center().x.max(left + max_w / 2.0));
    let center_x = center_x.max(left + max_w / 2.0);
    let mut y = ui::TOP_BAR + ui::MARGIN;
    let mut cancel = false;
    if let Some(text) = hint {
        let stacked = max_w < 560.0;
        let r = egui::Area::new(egui::Id::unique("hint_banner"))
            .pivot(Align2::CENTER_TOP)
            .fixed_pos(pos2(center_x, y))
            .order(egui::Order::Middle)
            .show(ctx, |ui| {
                ui::theme::plate_frame().show(ui, |ui| {
                    ui.set_max_width(max_w);
                    let pill = |ui: &mut egui::Ui| {
                        ui::widgets::pill(
                            ui,
                            ui::widgets::Btn::new("Cancel")
                                .glyph(ui::glyph::Glyph::Cross, palette::BAD)
                                .key(Some("Esc"))
                                .tip("Stop and go back without choosing a tile."),
                        )
                    };
                    let label = |ui: &mut egui::Ui, w: f32| {
                        ui.scope(|ui| {
                            ui.set_max_width(w);
                            ui.add(
                                egui::Label::new(RichText::new(&text).size(16.0).color(GOLD)).wrap(),
                            );
                        });
                    };
                    if stacked {
                        label(ui, max_w);
                        cancel = pill(ui);
                    } else {
                        ui.horizontal(|ui| {
                            label(ui, max_w - 150.0);
                            cancel = pill(ui);
                        });
                    }
                });
            });
        y += r.response.rect.height() + 8.0;
    }
    if cancel {
        s.placing = None;
        s.city_targeting = false;
    }
    if let Some((text, at)) = &s.message {
        if at.elapsed().as_secs_f32() < 3.5 {
            egui::Area::new(egui::Id::unique("toast"))
                .pivot(Align2::CENTER_TOP)
                .fixed_pos(pos2(center_x, y))
                .order(egui::Order::Tooltip)
                .interactable(false)
                .show(ctx, |ui| {
                    ui::theme::plate_frame().show(ui, |ui| {
                        ui.set_max_width(max_w);
                        ui.label(RichText::new(text).size(16.0));
                    });
                });
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            s.message = None;
        }
    }
}

/// The map key (K): terrain swatches and what the map marks mean.
fn legend(s: &mut Session, ctx: &egui::Context) {
    if !s.windows.legend {
        return;
    }
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
    let mut close = false;
    egui::Area::new(egui::Id::unique("legend"))
        .fixed_pos(pos2(ui::MARGIN + 310.0, ui::TOP_BAR + ui::MARGIN))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            ui::theme::plate_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Map Key").size(20.0).strong().color(GOLD));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui::theme::round_button(
                            ui,
                            28.0,
                            ui::glyph::Glyph::Cross,
                            palette::TEXT,
                            None,
                            true,
                        )
                        .on_hover_text("Close (K)")
                        .clicked()
                        {
                            close = true;
                        }
                    });
                });
                let grid_h = (ctx.content_rect().height() - ui::TOP_BAR - ui::MARGIN - 330.0).max(180.0);
                ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                egui::ScrollArea::vertical().max_height(grid_h).auto_shrink([true, true]).show(ui, |ui| {
                egui::Grid::new("legend_grid")
                    .num_columns(2)
                    .spacing([14.0, 4.0])
                    .show(ui, |ui| {
                        let mut column = 0;
                        let mut cell = |ui: &mut egui::Ui,
                                        draw: &dyn Fn(&egui::Painter, egui::Rect),
                                        name: &str| {
                            ui.horizontal(|ui| {
                                let (r, _) =
                                    ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
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
                        cell(
                            ui,
                            &|p, r| {
                                swatch(p, r, "TERRAIN_GRASS", None);
                                p.line_segment(
                                    [
                                        r.left_bottom() + vec2(2.0, -3.0),
                                        r.right_bottom() + vec2(-2.0, -3.0),
                                    ],
                                    egui::Stroke::new(3.0, Color32::from_rgb(92, 78, 66)),
                                );
                            },
                            "Cliffs",
                        );
                        cell(
                            ui,
                            &|p, r| {
                                swatch(p, r, "TERRAIN_PLAINS", None);
                                p.line_segment(
                                    [r.left_center(), r.right_center()],
                                    egui::Stroke::new(2.5, Color32::from_rgb(60, 130, 220)),
                                );
                            },
                            "River",
                        );
                    });
                ui.add_space(4.0);
                for line in [
                    "Bumps: hills. Gray peaks: mountains. Star: natural wonder.",
                    "Resources: icons ringed green (bonus), purple (luxury), red (strategic).",
                    "Colored edges: borders in each civilization's colors.",
                    "Brown lines: roads. Red tents: barbarian camps.",
                    "Diamonds: aircraft at their Aerodrome. Gold dashes: your trade routes.",
                ] {
                    ui.label(RichText::new(line).size(13.5).color(palette::TEXT_DIM));
                }
                });
            });
        });
    s.art = art.into_inner();
    if close {
        s.windows.legend = false;
    }
}

impl Session {
    pub fn research_turns(&self, tree: Tree, i: usize) -> String {
        let me = self.me();
        let income = self.game.income(me);
        let rate = match tree {
            Tree::Tech => income.yields[3],
            Tree::Civic => income.yields[4],
            Tree::Ai => self.game.compute(me),
        };
        if rate <= 0.0 {
            return match tree {
                Tree::Tech => "Needs Science",
                Tree::Civic => "Needs Culture",
                Tree::Ai => "Needs Compute",
            }
            .into();
        }
        let left =
            (self.game.research_cost(tree, i) - self.game.progress(me, tree, i)).max(0) as f32;
        match (left / rate).ceil() as i32 {
            1 => "1 turn".into(),
            n => format!("{n} turns"),
        }
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

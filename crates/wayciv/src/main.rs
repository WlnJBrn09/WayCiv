//! WayCiv: a Civilization VI-style game in Rust.

mod hud;
mod icons;
mod map_view;
mod palette;
mod portraits;
mod render;
mod screens;
mod setup_screen;
mod ui;

use std::sync::{Arc, mpsc};
use wayciv_core::{Game, Rules};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WayCiv")
            .with_app_id("wayciv")
            .with_inner_size([1500.0, 920.0])
            .with_min_inner_size([1100.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native("WayCiv", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

pub enum Stage {
    Loading(mpsc::Receiver<Result<Arc<Rules>, String>>),
    Failed(String),
    Setup(setup_screen::Setup),
    Playing(Box<hud::Session>),
}

pub struct App {
    pub stage: Stage,
    pub rules: Option<Arc<Rules>>,
    pub portraits: portraits::Portraits,
    /// Test hook: `WAYCIV_SNAPSHOT=out.png` saves a rendered frame and exits.
    snapshot: Option<(String, u32)>,
    /// `WAYCIV_SNAPSHOT_AT` and `WAYCIV_AT`: when the game started and the
    /// timed screenshots and actions still to come (milliseconds).
    clock: Option<f64>,
    /// Virtual time (seconds) while those hooks are set: 60 fps per frame, so
    /// animations replay the same however slowly frames are rendered.
    fake_time: Option<f64>,
    /// `WAYCIV_POINTER=x,y`: a pointer parked there (to see hover states).
    pointer: Option<egui::Pos2>,
    shots: Vec<u64>,
    actions: Vec<(u64, String)>,
    /// Test hook: `WAYCIV_UI_SIZE=WxH`.
    ui_size: Option<(f32, f32)>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        palette::style(&cc.egui_ctx);
        let (tx, rx) = mpsc::channel();
        let ctx = cc.egui_ctx.clone();
        std::thread::spawn(move || {
            let result = wayciv_core::load_rules().map_err(|e| e.to_string());
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        let snapshot = std::env::var("WAYCIV_SNAPSHOT").ok().map(|p| (p, 0));
        let ms = |s: &str| s.trim().parse::<u64>().ok();
        let mut shots: Vec<u64> = std::env::var("WAYCIV_SNAPSHOT_AT")
            .map(|v| v.split(',').filter_map(ms).collect())
            .unwrap_or_default();
        shots.sort();
        let mut actions: Vec<(u64, String)> = std::env::var("WAYCIV_AT")
            .map(|v| {
                v.split(',')
                    .filter_map(|a| a.split_once(':'))
                    .filter_map(|(t, a)| Some((ms(t)?, a.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        actions.sort_by_key(|a| a.0);
        Self {
            stage: Stage::Loading(rx),
            rules: None,
            portraits: portraits::Portraits::new(),
            snapshot,
            clock: None,
            pointer: std::env::var("WAYCIV_POINTER").ok().and_then(|p| {
                let (x, y) = p.split_once(',')?;
                Some(egui::pos2(x.parse().ok()?, y.parse().ok()?))
            }),
            fake_time: (!shots.is_empty() || !actions.is_empty()).then_some(0.0),
            shots,
            actions,
            ui_size: std::env::var("WAYCIV_UI_SIZE").ok().and_then(|s| {
                let (w, h) = s.split_once('x')?;
                Some((w.parse().ok()?, h.parse().ok()?))
            }),
        }
    }

    pub fn start(&mut self, game: Game) {
        self.stage = Stage::Playing(Box::new(hud::Session::new(game)));
    }

    /// Test hook: starts a game straight away. `WAYCIV_SEED` picks the map,
    /// `WAYCIV_LEADER` the leader, `WAYCIV_TURNS` lets the AI play your civ
    /// for that many turns, `WAYCIV_OPEN` opens a screen, `WAYCIV_REVEAL`
    /// reveals the whole map, and `WAYCIV_CENTER=x,y` centers the view.
    /// `WAYCIV_OPEN` is one of city, tech, civic, ai, religion, found (a
    /// Great Prophet ready to found a religion), government,
    /// victory, great_people, city_states, leaders, notifications, legend,
    /// diplomacy or popup. `WAYCIV_TOASTS=1` replays the last few
    /// notifications as new ones (toasts and the unread badge).
    /// `WAYCIV_FAITH=n` sets your Faith. `WAYCIV_GOLD=n` sets your Gold.
    /// `WAYCIV_PLACE=district|bombard` starts placing a district or aiming
    /// the city's ranged attack. `WAYCIV_MEETALL=1` meets every other civilization.
    /// `WAYCIV_SETTLE=1` founds the capital with the starting Settler.
    /// `WAYCIV_SELECT=<unit>` selects a unit of that type (`builder`,
    /// `UNIT_TRADER`, `great_engineer`...), spawning one in the capital (a
    /// Builder goes on the owned tile with the most improvements) when you
    /// have none. `WAYCIV_CITY_TAB=n` picks the city panel tab.
    /// `WAYCIV_UI_SIZE=WxH` lays the HUD out as if the screen were at most
    /// that big (the window is scaled to fit), to test small monitors.
    /// `WAYCIV_SNAPSHOT_AT=0,120,400`
    /// saves frames at those milliseconds after the game starts (the last
    /// one to `WAYCIV_SNAPSHOT`, the others as `<name>_<ms>.png`), and
    /// `WAYCIV_AT=300:endturn,600:research` runs actions then (see
    /// `ui::anim::demo`). `WAYCIV_POINTER=x,y` parks the pointer there.
    fn autostart(&mut self, rules: &Arc<Rules>) {
        let env = |k: &str| std::env::var(k).ok();
        let settings = wayciv_core::Settings {
            seed: env("WAYCIV_SEED").and_then(|s| s.parse().ok()).unwrap_or(7),
            human_leader: env("WAYCIV_LEADER").unwrap_or(wayciv_core::rules::REAGAN.to_string()),
            map_type: if env("WAYCIV_MAP").as_deref() == Some("random") {
                wayciv_core::MapType::Random
            } else {
                wayciv_core::MapType::Earth
            },
            size: match env("WAYCIV_SIZE").as_deref() {
                Some("small") => wayciv_core::MapSize::Small,
                Some("standard") => wayciv_core::MapSize::Standard,
                _ => wayciv_core::MapSize::Large,
            },
            opponents: 9,
            city_states: 12,
            length: env("WAYCIV_LENGTH")
                .and_then(|l| wayciv_core::GameLength::parse(&l))
                .unwrap_or_default(),
            ..Default::default()
        };
        let mut game = wayciv_core::new_game(rules.clone(), settings);
        let turns: u32 = env("WAYCIV_TURNS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if turns > 0 {
            let human = game.human();
            game.players[human].human = false;
            for _ in 0..turns {
                game.end_turn();
            }
            game.players[human].human = true;
            game.popups.clear();
        }
        if let Some(faith) = env("WAYCIV_FAITH").and_then(|f| f.parse().ok()) {
            let human = game.human();
            game.players[human].faith = faith;
        }
        if let Some(gold) = env("WAYCIV_GOLD").and_then(|f| f.parse().ok()) {
            let human = game.human();
            game.players[human].gold = gold;
        }
        if env("WAYCIV_SETTLE").is_some() {
            let human = game.human();
            let settler = game
                .player_units(human)
                .find(|u| game.unit_type(u).found_city)
                .map(|u| u.id);
            if let Some(id) = settler {
                let _ = game.found_city(id);
            }
        }
        if env("WAYCIV_MEETALL").is_some() {
            let human = game.human();
            for o in 0..game.players.len() {
                if o != human && game.players[o].is_major() {
                    game.players[human].relations[o].met = true;
                    game.players[o].relations[human].met = true;
                }
            }
        }
        if env("WAYCIV_REVEAL").is_some() {
            let human = game.human();
            game.players[human]
                .revealed
                .iter_mut()
                .for_each(|r| *r = true);
            game.players[human]
                .visible
                .iter_mut()
                .for_each(|r| *r = true);
        }
        self.start(game);
        if let Stage::Playing(s) = &mut self.stage {
            let human = s.game.human();
            let first_city = s.game.player_cities(human).next().map(|c| (c.id, c.pos));
            if let Some(city) = first_city {
                map_view_center(s, city.1);
                if env("WAYCIV_OPEN").as_deref() == Some("city") {
                    s.select_city(Some(city.0));
                }
            }
            use screens::Panel;
            if let Some(tab) = env("WAYCIV_CITY_TAB").and_then(|t| t.parse().ok()) {
                s.city_tab = tab;
            }
            match (env("WAYCIV_PLACE").as_deref(), first_city) {
                (Some("district"), Some((id, _))) => {
                    s.select_city(Some(id));
                    s.placing = s
                        .game
                        .available_production(id)
                        .into_iter()
                        .find(|p| matches!(p, wayciv_core::Production::District(..)));
                }
                (Some("bombard"), Some((id, _))) => {
                    s.select_city(Some(id));
                    s.city_targeting = true;
                }
                _ => {}
            }
            if let Some(kind) = env("WAYCIV_SELECT") {
                select_hook(s, &kind, first_city.map(|c| c.1));
            }
            if env("WAYCIV_TOASTS").is_some() {
                s.notifications.replay(&s.game, 5);
            }
            match env("WAYCIV_OPEN").as_deref() {
                Some("tech") => s.windows.open(Panel::Tech),
                Some("civic") => s.windows.open(Panel::Civic),
                Some("ai") => s.windows.open(Panel::Ai),
                Some("government") => s.windows.open(Panel::Government),
                Some("religion") => s.windows.open(Panel::Religion),
                Some("found") => {
                    // A Great Prophet on a Holy Site next to the capital.
                    let me = s.game.human();
                    let grid = s.game.map.grid;
                    let site = first_city.and_then(|(_, c)| {
                        grid.neighbors(c)
                            .find(|&p| s.game.map.tile(p).owner == Some(me))
                    });
                    let holy = s.game.rules.idx("DISTRICT_HOLY_SITE");
                    if let (Some(site), Some(holy)) = (site, holy) {
                        s.game.map.tile_mut(site).district = Some(holy);
                        if s.game.players[me].pantheon.is_none() {
                            let b = s.game.beliefs_of_class("BELIEF_CLASS_PANTHEON")[0];
                            s.game.players[me].pantheon = Some(b);
                        }
                        if let Some(u) = s.game.spawn(me, "UNIT_GREAT_PROPHET", site) {
                            s.windows.open(Panel::Religion);
                            s.windows.found = Some(ui::menus::FoundDraft::new(u));
                        }
                    }
                }
                Some("victory") => s.windows.open(Panel::Victory),
                Some("great_people") => s.windows.open(Panel::GreatPeople),
                Some("city_states") => s.windows.open(Panel::CityStates),
                Some("leaders") => s.windows.open(Panel::Leaders),
                Some("notifications") => s.windows.open(Panel::Notifications),
                Some("legend") => s.windows.legend = true,
                Some("diplomacy") => {
                    let me = s.game.human();
                    if let Some(o) =
                        (0..s.game.players.len()).find(|&o| o != me && s.game.players[o].is_major())
                    {
                        s.windows.open(Panel::Diplomacy(o));
                    }
                }
                Some("popup") => {
                    let me = s.game.human();
                    let other = (0..s.game.players.len())
                        .find(|&o| o != me && s.game.players[o].is_major())
                        .unwrap_or(1);
                    s.popup = Some((other, wayciv_core::diplomacy::DiploEvent::FirstMeet));
                }
                _ => {}
            }
            if let Some(z) = env("WAYCIV_ZOOM").and_then(|z| z.parse().ok()) {
                s.cam.zoom = z;
            }
            if let Some((x, y)) = env("WAYCIV_CENTER").and_then(|c| {
                c.split_once(',')
                    .and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?)))
            }) {
                map_view_center(s, wayciv_core::Pos::new(x, y));
            }
        }
    }

    fn snapshot_hook(&mut self, ctx: &egui::Context) {
        let Some((path, frames)) = &mut self.snapshot else {
            return;
        };
        if matches!(self.stage, Stage::Loading(_)) {
            ctx.request_repaint();
            return;
        }
        *frames += 1;
        let now = self.fake_time.unwrap_or(0.0);
        let ms = ((now - *self.clock.get_or_insert(now)) * 1000.0) as u64;
        if let Stage::Playing(s) = &mut self.stage {
            while self.actions.first().is_some_and(|a| ms >= a.0) {
                ui::anim::demo(s, &self.actions.remove(0).1);
            }
        }
        let shot = if self.shots.is_empty() {
            (*frames == 30).then(|| (path.clone(), true))
        } else if ms >= self.shots[0] {
            let at = self.shots.remove(0);
            let last = self.shots.is_empty();
            Some((if last { path.clone() } else { path.replace(".png", &format!("_{at}.png")) }, last))
        } else {
            None
        };
        if let Some((path, last)) = shot {
            let ctx2 = ctx.clone();
            let callback = egui::ScreenshotCallback::new(move |image| {
                let [w, h] = image.size;
                let pixels: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
                if let Some(buffer) = image::RgbaImage::from_raw(w as u32, h as u32, pixels) {
                    let _ = buffer.save(&path);
                }
                if last {
                    ctx2.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ctx2.request_repaint();
            });
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
        }
        ctx.request_repaint();
    }
}

/// `WAYCIV_SELECT`: selects (spawning if needed) one of your units.
fn select_hook(s: &mut hud::Session, kind: &str, city: Option<wayciv_core::Pos>) {
    let me = s.game.human();
    let want = kind.to_uppercase().trim_start_matches("UNIT_").to_string();
    let matches = |id: &str| id.trim_start_matches("UNIT_") == want || id == kind;
    let found = s
        .game
        .player_units(me)
        .find(|u| matches(&s.game.unit_type(u).id))
        .map(|u| u.id);
    let id = found.or_else(|| {
        let full = format!("UNIT_{want}");
        let base = city.or_else(|| s.game.player_units(me).next().map(|u| u.pos))?;
        let at = if want == "BUILDER" {
            // The owned tile where a Builder has the most to build.
            let grid = s.game.map.grid;
            grid.all()
                .filter(|&p| s.game.map.tile(p).owner == Some(me))
                .max_by_key(|&p| s.game.valid_improvements(me, p).len())
                .unwrap_or(base)
        } else {
            base
        };
        let id = s.game.spawn(me, &full, at)?;
        if want.starts_with("GREAT_") && want != "GREAT_PROPHET" {
            if let Some(u) = s.game.units.get_mut(&id) {
                u.great_person = Some(0);
            }
        }
        Some(id)
    });
    if let Some(id) = id {
        // Fresh orders; a Builder goes where it has the most to build.
        let best = (want == "BUILDER").then(|| {
            let grid = s.game.map.grid;
            grid.all()
                .filter(|&p| s.game.map.tile(p).owner == Some(me))
                .max_by_key(|&p| s.game.valid_improvements(me, p).len())
        });
        let moves = s.game.unit_type(&s.game.units[&id]).moves;
        if let Some(u) = s.game.units.get_mut(&id) {
            u.moves = moves;
            if let Some(Some(p)) = best {
                u.pos = p;
            }
        }
        s.select_unit(Some(id));
        if let Some(u) = s.game.units.get(&id) {
            let pos = u.pos;
            map_view_center(s, pos);
        }
    }
}

fn map_view_center(s: &mut hud::Session, pos: wayciv_core::Pos) {
    map_view::center_on(s, pos);
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if let Some(t) = &mut self.fake_time {
            *t += 1.0 / 60.0;
            raw_input.time = Some(*t);
        }
        if let Some(p) = self.pointer {
            raw_input.events.push(egui::Event::PointerMoved(p));
        }
    }

    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Stage::Loading(rx) = &self.stage {
            match rx.try_recv() {
                Ok(Ok(rules)) => {
                    self.rules = Some(rules.clone());
                    self.stage = Stage::Setup(setup_screen::Setup::new(&rules));
                    if std::env::var_os("WAYCIV_AUTOSTART").is_some() {
                        self.autostart(&rules);
                    }
                }
                Ok(Err(e)) => self.stage = Stage::Failed(e),
                Err(_) => {}
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some((w, h)) = self.ui_size {
            let ctx = ui.ctx();
            let physical = ctx.content_rect().size() * ctx.pixels_per_point();
            let ppp = (physical.x / w).max(physical.y / h);
            if (ctx.pixels_per_point() - ppp).abs() > 0.001 {
                ctx.set_pixels_per_point(ppp);
            }
        }
        self.snapshot_hook(ui.ctx());
        match &mut self.stage {
            Stage::Loading(_) => {
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.heading("Loading Civilization VI content from your install...")
                    });
                });
            }
            Stage::Failed(error) => {
                let error = error.clone();
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.heading("WayCiv couldn't load the game's content");
                    ui.label(error);
                    ui.label("WayCiv reads leaders, units and techs from an installed copy of Civilization VI. Set CIV6_DIR to the game's install folder if it isn't in the default Steam location.");
                });
            }
            Stage::Setup(_) => setup_screen::show(self, ui),
            Stage::Playing(_) => hud::show(self, ui),
        }
    }
}

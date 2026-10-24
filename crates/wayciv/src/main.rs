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
        Self {
            stage: Stage::Loading(rx),
            rules: None,
            portraits: portraits::Portraits::new(),
            snapshot,
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
    /// `WAYCIV_FAITH=n` sets your Faith.
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
        if *frames == 30 {
            let path = path.clone();
            let ctx2 = ctx.clone();
            let callback = egui::ScreenshotCallback::new(move |image| {
                let [w, h] = image.size;
                let pixels: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
                if let Some(buffer) = image::RgbaImage::from_raw(w as u32, h as u32, pixels) {
                    let _ = buffer.save(&path);
                }
                ctx2.send_viewport_cmd(egui::ViewportCommand::Close);
                ctx2.request_repaint();
            });
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
        }
        ctx.request_repaint();
    }
}

fn map_view_center(s: &mut hud::Session, pos: wayciv_core::Pos) {
    map_view::center_on(s, pos);
}

impl eframe::App for App {
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

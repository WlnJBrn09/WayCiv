//! The "Create Game" screen: pick a leader and game settings.

use crate::App;
use crate::palette;
use crate::portraits::{self, Portraits};
use crate::ui::{cards, theme};
use egui::{Color32, Rect, Sense, pos2, vec2};
use std::sync::Arc;
use wayciv_core::{GameLength, MapSize, MapType, Rules, Settings};

pub struct Setup {
    pub leader: usize,
    pub map_type: MapType,
    pub size: MapSize,
    pub length: GameLength,
    pub opponents: usize,
    pub city_states: usize,
    pub seed: u64,
}

impl Setup {
    pub fn new(rules: &Arc<Rules>) -> Self {
        let leader = rules
            .content
            .leaders
            .iter()
            .position(|l| l.id == wayciv_core::rules::REAGAN)
            .unwrap_or(0);
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(1)
            % 100_000;
        Self {
            leader,
            map_type: MapType::Earth,
            size: MapSize::Large,
            length: GameLength::default(),
            opponents: 9,
            city_states: 12,
            seed,
        }
    }
}

/// Width of the details column on the right.
const DETAILS_W: f32 = 470.0;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let rules = app.rules.clone().expect("rules loaded");
    let content = &rules.content;
    let crate::Stage::Setup(setup) = &mut app.stage else {
        return;
    };
    let mut start = false;
    theme::backdrop(ui.painter(), ui.max_rect());

    egui::Panel::right("setup_details")
        .exact_size(DETAILS_W)
        .frame(egui::Frame::NONE.inner_margin(egui::Margin {
            left: 0,
            right: 18,
            top: 18,
            bottom: 18,
        }))
        .show_separator_line(false)
        .show(ui, |ui| {
            theme::plate_ui(ui, 20, |ui| {
                ui.set_min_size(ui.available_size());
                start = details(&mut app.portraits, &rules, setup, ui);
            });
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.inner_margin(egui::Margin {
            left: 18,
            right: 18,
            top: 18,
            bottom: 18,
        }))
        .show(ui, |ui| {
            theme::plate_ui(ui, 22, |ui| {
                ui.set_min_size(ui.available_size());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(theme::title("Choose Your Leader", 36.0));
                        ui.label(
                            egui::RichText::new(
                                "Every leader brings a civilization with its own bonuses, unique units and buildings.",
                            )
                            .size(14.5)
                            .color(palette::TEXT_DIM),
                        );
                    });
                });
                let y = ui.cursor().top() + 8.0;
                theme::divider(
                    ui.painter(),
                    ui.min_rect().left(),
                    ui.max_rect().right(),
                    y,
                    palette::with_alpha(palette::GOLD, 220),
                );
                ui.add_space(20.0);
                leader_grid(&mut app.portraits, &rules, setup, ui);
            });
        });

    if start {
        let settings = Settings {
            seed: setup.seed,
            map_type: setup.map_type,
            size: setup.size,
            human_leader: content.leaders[setup.leader].id.clone(),
            opponents: setup.opponents,
            city_states: setup.city_states,
            length: setup.length,
            ..Settings::default()
        };
        let game = wayciv_core::new_game(rules.clone(), settings);
        app.start(game);
    }
}

/// Every leader as a framed portrait with a nameplate; click to choose.
fn leader_grid(portraits: &mut Portraits, rules: &Rules, setup: &mut Setup, ui: &mut egui::Ui) {
    let content = &rules.content;
    let frame = portraits::FRAME;
    // The largest portraits that still fit every leader on screen at once.
    let cell = |tile: f32| vec2(tile + 2.0 * frame + 18.0, tile + 2.0 * frame + 56.0);
    let fits = |tile: f32| {
        let per_row = ((ui.available_width() / cell(tile).x).floor() as usize).max(1);
        let rows = content.leaders.len().div_ceil(per_row);
        rows as f32 * cell(tile).y <= ui.available_height()
    };
    let tile = [128.0, 120.0, 112.0, 104.0, 96.0]
        .into_iter()
        .find(|&t| fits(t))
        .unwrap_or(96.0);
    let pitch = cell(tile);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let per_row = ((ui.available_width() / pitch.x).floor() as usize).max(1);
            let mut order: Vec<usize> = (0..content.leaders.len()).collect();
            order.sort_by_key(|&i| content.leaders[i].name.clone());
            // Center the grid in the space left over.
            let used = per_row as f32 * pitch.x;
            let indent = ((ui.available_width() - used) / 2.0).max(0.0);
            for row in order.chunks(per_row) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    ui.add_space(indent);
                    for &i in row {
                        let leader = &content.leaders[i];
                        let (cell, response) = ui.allocate_exact_size(pitch, Sense::click());
                        let image = Rect::from_min_size(
                            pos2(cell.center().x - tile / 2.0, cell.top() + frame + 4.0),
                            vec2(tile, tile),
                        );
                        let selected = setup.leader == i;
                        let hovered = response.hovered();
                        let painter = ui.painter().clone();
                        if selected || hovered {
                            painter.add(egui::Shadow {
                                offset: [0, 0],
                                blur: 22,
                                spread: 2,
                                color: palette::with_alpha(
                                    palette::GOLD,
                                    if selected { 150 } else { 70 },
                                ),
                            }
                            .as_shape(image.expand(frame), egui::CornerRadius::same(4)));
                        }
                        portraits.paint(
                            ui,
                            image,
                            &leader.id,
                            &leader.name,
                            (leader.primary_color, leader.secondary_color),
                        );
                        if !selected && !hovered {
                            painter.rect_filled(
                                image.expand(frame),
                                4.0,
                                Color32::from_black_alpha(46),
                            );
                        }
                        if selected {
                            painter.rect_stroke(
                                image.expand(frame + 2.0),
                                6.0,
                                egui::Stroke::new(2.0, palette::GOLD_HI),
                                egui::StrokeKind::Outside,
                            );
                        }
                        let civ = rules
                            .civilization(&leader.civilization)
                            .map(|c| c.name.as_str())
                            .unwrap_or("");
                        let name = fit(ui, &leader.name, 14.5, pitch.x - 6.0);
                        theme::text_bold(
                            &painter,
                            pos2(cell.center().x, image.bottom() + frame + 8.0),
                            egui::Align2::CENTER_TOP,
                            name,
                            14.5,
                            if selected { palette::GOLD_HI } else { palette::TEXT },
                        );
                        theme::text(
                            &painter,
                            pos2(cell.center().x, image.bottom() + frame + 27.0),
                            egui::Align2::CENTER_TOP,
                            civ,
                            12.5,
                            if selected { palette::GOLD } else { palette::TEXT_DIM },
                        );
                        if response.clicked() {
                            setup.leader = i;
                        }
                    }
                });
            }
        });
}

/// The right-hand column: the chosen leader's portrait and traits, the game
/// settings and the Start button. Returns true when Start was pressed.
fn details(portraits: &mut Portraits, rules: &Rules, setup: &mut Setup, ui: &mut egui::Ui) -> bool {
    let content = &rules.content;
    let leader = &content.leaders[setup.leader];
    let civ = rules.civilization(&leader.civilization);
    let mut start = false;
    let frame = portraits::FRAME;

    // The settings and Start button stay at the bottom; the rest scrolls above.
    egui::Panel::bottom("setup_settings")
        .frame(egui::Frame::NONE)
        .show_separator_line(false)
        .resizable(false)
        .show(ui, |ui| {
            ui.add_space(6.0);
            let y = ui.cursor().top() + 2.0;
            theme::divider(
                ui.painter(),
                ui.min_rect().left(),
                ui.max_rect().right(),
                y,
                palette::with_alpha(palette::GOLD_DIM, 220),
            );
            ui.add_space(12.0);
            settings(setup, ui);
            ui.add_space(14.0);
            start = cards::action_button_wide(ui, "Start Game", true, "", ui.available_width());
        });

    // On short windows the portrait sits beside the name instead of above it.
    let compact = ui.available_height() < 440.0;
    let side = if compact {
        84.0
    } else {
        (ui.available_width() - 2.0 * frame).min(ui.available_height() * 0.5)
    };
    let mut header = |ui: &mut egui::Ui| {
        let (rect, _) = ui.allocate_exact_size(
            vec2(side + 2.0 * frame, side + 2.0 * frame),
            Sense::hover(),
        );
        portraits.paint(
            ui,
            rect.shrink(frame),
            &leader.id,
            &leader.name,
            (leader.primary_color, leader.secondary_color),
        );
    };
    let names = |ui: &mut egui::Ui| {
        ui.label(theme::title(&leader.name, if compact { 26.0 } else { 30.0 }));
        if let Some(civ) = civ {
            ui.horizontal(|ui| {
                if !compact {
                    let text_w = ui
                        .painter()
                        .layout_no_wrap(civ.name.clone(), palette::bold(18.0), palette::TEXT)
                        .size()
                        .x;
                    ui.add_space(((ui.available_width() - text_w - 44.0) / 2.0).max(0.0));
                }
                for color in [leader.primary_color, leader.secondary_color] {
                    let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                    theme::disc(ui.painter(), r.center(), 7.0, palette::rgb(color), palette::GOLD_DIM);
                }
                ui.label(theme::bold(&civ.name, 18.0));
            });
        }
    };
    if compact {
        ui.horizontal(|ui| {
            header(ui);
            ui.vertical(names);
        });
    } else {
        ui.vertical_centered(|ui| {
            header(ui);
            ui.add_space(6.0);
            names(ui);
        });
    }
    ui.add_space(8.0);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            let block = |ui: &mut egui::Ui, name: &str, desc: &str, gold: bool| {
                theme::well_ui(
                    ui,
                    10,
                    palette::NAVY_RAISED,
                    palette::with_alpha(palette::GOLD_DIM, 140),
                    |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(theme::bold(name, 15.0).color(if gold {
                            palette::GOLD
                        } else {
                            palette::TEXT
                        }));
                        if !desc.is_empty() {
                            ui.label(
                                egui::RichText::new(desc).size(13.5).color(palette::TEXT_DIM),
                            );
                        }
                    },
                );
            };
            for t in &leader.traits {
                block(ui, &t.name, &t.description, true);
            }
            if let Some(civ) = civ {
                for t in &civ.traits {
                    block(ui, &t.name, &t.description, false);
                }
            }
            if let Some((name, desc)) = &leader.agenda {
                block(ui, &format!("Agenda: {name}"), desc, true);
            }
        });
    start
}

/// A labeled row of the settings block.
fn setting(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
        let (rect, _) = ui.allocate_exact_size(vec2(126.0, 30.0), Sense::hover());
        theme::text(
            ui.painter(),
            rect.left_center(),
            egui::Align2::LEFT_CENTER,
            label,
            14.5,
            palette::TEXT_DIM,
        );
        ui.horizontal_wrapped(add);
    });
}

fn settings(setup: &mut Setup, ui: &mut egui::Ui) {
    ui.spacing_mut().item_spacing.y = 7.0;
    setting(ui, "Map", |ui| {
        for (kind, name, tip) in [
            (
                MapType::Earth,
                "Earth",
                "Real geography and biomes. Every leader starts at their historical capital.",
            ),
            (MapType::Random, "Random", "A randomly generated world."),
        ] {
            if theme::toggle_pill(ui, name, setup.map_type == kind)
                .on_hover_text(tip)
                .clicked()
            {
                setup.map_type = kind;
            }
        }
    });
    setting(ui, "Map size", |ui| {
        for size in [MapSize::Small, MapSize::Standard, MapSize::Large] {
            let grid = match setup.map_type {
                MapType::Earth => wayciv_core::earth::projection(size).grid,
                MapType::Random => size.grid(),
            };
            let name = match size {
                MapSize::Small => "Small",
                MapSize::Standard => "Standard",
                MapSize::Large => "Giant",
            };
            if theme::toggle_pill(ui, name, setup.size == size)
                .on_hover_text(format!("{}x{} tiles", grid.width, grid.height))
                .clicked()
            {
                setup.size = size;
            }
        }
    });
    setting(ui, "Game length", |ui| {
        for length in GameLength::ALL {
            let hint = match length.turns() {
                Some(t) => format!(
                    "The game ends after turn {t}. Research, production, and growth are paced to match."
                ),
                None => "No turn limit. Paced like a 500-turn game.".to_string(),
            };
            if theme::toggle_pill(ui, &length.name(), setup.length == length)
                .on_hover_text(hint)
                .clicked()
            {
                setup.length = length;
            }
        }
    });
    setting(ui, "Rival civilizations", |ui| {
        ui.spacing_mut().slider_width = 170.0;
        ui.add(egui::Slider::new(&mut setup.opponents, 1..=11));
    });
    setting(ui, "City-states", |ui| {
        ui.spacing_mut().slider_width = 170.0;
        ui.add(egui::Slider::new(&mut setup.city_states, 0..=16));
    });
    setting(ui, "Map seed", |ui| {
        ui.add(egui::DragValue::new(&mut setup.seed));
        if theme::pill_button(ui, None, "Roll", palette::TEXT, 0.0)
            .on_hover_text("Pick a new random map seed")
            .clicked()
        {
            setup.seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos() as u64 % 100_000)
                .unwrap_or(1);
        }
    });
}

/// `text` shortened with an ellipsis to fit `width` pixels.
fn fit(ui: &egui::Ui, text: &str, size: f32, width: f32) -> String {
    let font = egui::FontId::proportional(size);
    let measure = |t: &str| {
        ui.painter()
            .layout_no_wrap(t.to_string(), font.clone(), Color32::WHITE)
            .size()
            .x
    };
    if measure(text) <= width - 4.0 {
        return text.to_string();
    }
    let mut chars: Vec<char> = text.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let candidate: String = chars.iter().collect::<String>() + "...";
        if measure(&candidate) <= width - 4.0 {
            return candidate;
        }
    }
    String::new()
}

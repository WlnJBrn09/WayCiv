//! The "Create Game" screen: pick a leader and game settings.

use crate::App;
use crate::palette;
use egui::{Color32, RichText, Sense, vec2};
use std::sync::Arc;
use wayciv_core::{MapSize, MapType, Rules, Settings};

pub struct Setup {
    pub leader: usize,
    pub map_type: MapType,
    pub size: MapSize,
    pub opponents: usize,
    pub city_states: usize,
    pub seed: u64,
}

impl Setup {
    pub fn new(rules: &Arc<Rules>) -> Self {
        let leader = rules.content.leaders.iter().position(|l| l.id == wayciv_core::rules::REAGAN).unwrap_or(0);
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(1) % 100_000;
        Self { leader, map_type: MapType::Earth, size: MapSize::Large, opponents: 9, city_states: 12, seed }
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let rules = app.rules.clone().expect("rules loaded");
    let content = &rules.content;
    let crate::Stage::Setup(setup) = &mut app.stage else { return };
    let mut start = false;

    egui::Panel::right("setup_details").exact_size(430.0).show(ui, |ui| {
        let leader = &content.leaders[setup.leader];
        let civ = rules.civilization(&leader.civilization);
        ui.add_space(8.0);
        let (rect, _) = ui.allocate_exact_size(vec2(410.0, 410.0), Sense::hover());
        app.portraits.paint(ui, rect, &leader.id, &leader.name, (leader.primary_color, leader.secondary_color));
        ui.add_space(6.0);
        ui.heading(RichText::new(&leader.name).color(palette::GOLD).size(24.0));
        if let Some(civ) = civ {
            ui.label(RichText::new(&civ.name).size(17.0));
        }
        egui::ScrollArea::vertical().max_height(ui.available_height() - 190.0).show(ui, |ui| {
            for t in &leader.traits {
                ui.label(RichText::new(&t.name).strong().color(palette::GOLD));
                if !t.description.is_empty() {
                    ui.label(&t.description);
                }
                ui.add_space(4.0);
            }
            if let Some(civ) = civ {
                for t in &civ.traits {
                    ui.label(RichText::new(&t.name).strong());
                    if !t.description.is_empty() {
                        ui.label(&t.description);
                    }
                    ui.add_space(4.0);
                }
            }
            if let Some((name, desc)) = &leader.agenda {
                ui.label(RichText::new(format!("Agenda: {name}")).italics());
                ui.label(desc);
            }
        });
        ui.separator();
        egui::Grid::new("settings").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            ui.label("Map");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut setup.map_type, MapType::Earth, "Earth").on_hover_text("Real geography and biomes. Every leader starts at their historical capital.");
                ui.selectable_value(&mut setup.map_type, MapType::Random, "Random").on_hover_text("A randomly generated world.");
            });
            ui.end_row();
            ui.label("Map size");
            let size_name = |size: MapSize| {
                let grid = match setup.map_type {
                    MapType::Earth => wayciv_core::earth::projection(size).grid,
                    MapType::Random => size.grid(),
                };
                let name = match size {
                    MapSize::Small => "Small",
                    MapSize::Standard => "Standard",
                    MapSize::Large => "Giant",
                };
                format!("{name} ({}x{})", grid.width, grid.height)
            };
            egui::ComboBox::from_id_salt("size")
                .selected_text(size_name(setup.size))
                .show_ui(ui, |ui| {
                    for size in [MapSize::Small, MapSize::Standard, MapSize::Large] {
                        ui.selectable_value(&mut setup.size, size, size_name(size));
                    }
                });
            ui.end_row();
            ui.label("Rival civilizations");
            ui.add(egui::Slider::new(&mut setup.opponents, 1..=11));
            ui.end_row();
            ui.label("City-states");
            ui.add(egui::Slider::new(&mut setup.city_states, 0..=16));
            ui.end_row();
            ui.label("Map seed");
            ui.add(egui::DragValue::new(&mut setup.seed));
            ui.end_row();
        });
        ui.add_space(8.0);
        let button = egui::Button::new(RichText::new("Start Game").size(22.0).strong()).min_size(vec2(410.0, 46.0)).fill(Color32::from_rgb(40, 90, 60));
        if ui.add(button).clicked() {
            start = true;
        }
    });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading(RichText::new("Choose Your Leader").size(28.0).color(palette::GOLD));
        ui.add_space(6.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            let tile = 150.0;
            let per_row = ((ui.available_width() + 8.0) / (tile + 8.0)).floor().max(1.0) as usize;
            let mut order: Vec<usize> = (0..content.leaders.len()).collect();
            order.sort_by_key(|&i| content.leaders[i].name.clone());
            for row in order.chunks(per_row) {
                ui.horizontal(|ui| {
                    for &i in row {
                        let leader = &content.leaders[i];
                        let (rect, response) = ui.allocate_exact_size(vec2(tile, tile + 40.0), Sense::click());
                        let image = egui::Rect::from_min_size(rect.min, vec2(tile, tile));
                        app.portraits.paint(ui, image, &leader.id, &leader.name, (leader.primary_color, leader.secondary_color));
                        let selected = setup.leader == i;
                        if selected || response.hovered() {
                            let color = if selected { palette::GOLD } else { Color32::from_gray(200) };
                            ui.painter().rect_stroke(image, 4.0, egui::Stroke::new(3.0, color), egui::StrokeKind::Outside);
                        }
                        let civ = rules.civilization(&leader.civilization).map(|c| c.name.as_str()).unwrap_or("");
                        let name = fit(ui, &leader.name, 14.0, tile);
                        ui.painter().text(image.left_bottom() + vec2(tile / 2.0, 4.0), egui::Align2::CENTER_TOP, name, egui::FontId::proportional(14.0), Color32::WHITE);
                        ui.painter().text(image.left_bottom() + vec2(tile / 2.0, 22.0), egui::Align2::CENTER_TOP, civ, egui::FontId::proportional(12.0), Color32::from_gray(170));
                        if response.hovered() {
                            response.clone().on_hover_text(&leader.name);
                        }
                        if response.clicked() {
                            setup.leader = i;
                        }
                    }
                });
                ui.add_space(6.0);
            }
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
            ..Settings::default()
        };
        let game = wayciv_core::new_game(rules.clone(), settings);
        app.start(game);
    }
}

/// `text` shortened with an ellipsis to fit `width` pixels.
fn fit(ui: &egui::Ui, text: &str, size: f32, width: f32) -> String {
    let font = egui::FontId::proportional(size);
    let measure = |t: &str| ui.painter().layout_no_wrap(t.to_string(), font.clone(), Color32::WHITE).size().x;
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

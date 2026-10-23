//! Leader portraits from `assets/leaders/`, loaded on first use and
//! downscaled for display.

use egui::{Color32, TextureHandle, TextureOptions};
use std::collections::HashMap;
use std::path::PathBuf;

const MAX_SIZE: u32 = 512;

pub struct Portraits {
    dir: PathBuf,
    cache: HashMap<String, Option<TextureHandle>>,
}

/// WayCiv's asset folder: WAYCIV_ASSETS, then the installed copy
/// (~/.local/share/wayciv/assets, see install.sh), then assets next to the
/// binary, then the source tree.
pub fn assets_dir() -> PathBuf {
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")));
    std::env::var_os("WAYCIV_ASSETS")
        .map(PathBuf::from)
        .or_else(|| data_home.map(|d| d.join("wayciv/assets")).filter(|p| p.is_dir()))
        .or_else(|| std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("assets"))).filter(|p| p.is_dir()))
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets"))
}

impl Portraits {
    pub fn new() -> Self {
        Self { dir: assets_dir().join("leaders"), cache: HashMap::new() }
    }

    /// `LEADER_CATHERINE_DE_MEDICI_ALT` -> `leader_catherine_de_medici.png`.
    fn file(&self, leader: &str) -> PathBuf {
        let mut slug = leader.to_lowercase();
        for suffix in ["_alt", "_china", "_mongolia"] {
            if let Some(stripped) = slug.strip_suffix(suffix) {
                slug = stripped.to_string();
            }
        }
        self.dir.join(format!("{slug}.png"))
    }

    pub fn get(&mut self, ctx: &egui::Context, leader: &str) -> Option<TextureHandle> {
        if let Some(cached) = self.cache.get(leader) {
            return cached.clone();
        }
        let texture = image::open(self.file(leader)).ok().map(|img| {
            let img = img.thumbnail(MAX_SIZE, MAX_SIZE).to_rgba8();
            let size = [img.width() as usize, img.height() as usize];
            ctx.load_texture(leader, egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()), TextureOptions::LINEAR)
        });
        self.cache.insert(leader.to_string(), texture.clone());
        texture
    }

    /// Draws a leader's portrait in `rect`; a framed monogram in the civ's
    /// colors when no portrait exists.
    pub fn paint(&mut self, ui: &egui::Ui, rect: egui::Rect, leader: &str, name: &str, colors: ([u8; 3], [u8; 3])) {
        let painter = ui.painter();
        let primary = Color32::from_rgb(colors.0[0], colors.0[1], colors.0[2]);
        let secondary = Color32::from_rgb(colors.1[0], colors.1[1], colors.1[2]);
        match self.get(ui.ctx(), leader) {
            Some(texture) => {
                painter.image(texture.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
            }
            None => {
                painter.rect_filled(rect, 4.0, primary);
                let initials: String = name.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect();
                painter.text(rect.center(), egui::Align2::CENTER_CENTER, initials, egui::FontId::proportional(rect.height() * 0.4), secondary);
            }
        }
        painter.rect_stroke(rect, 4.0, egui::Stroke::new(2.0, primary), egui::StrokeKind::Inside);
    }
}

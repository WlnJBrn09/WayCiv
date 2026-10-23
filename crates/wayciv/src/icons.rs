//! Icons from `assets/icons/<kind>/<id>.png` (generated with Codex; see
//! tools/icons): resources, units, techs, civics, AI tree items, beliefs,
//! and the trees themselves. Loaded on first use; missing icons are skipped.

use egui::{Color32, Rect, TextureHandle, TextureOptions, pos2};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Icons {
    dir: PathBuf,
    cache: HashMap<(&'static str, String), Option<TextureHandle>>,
}

impl Icons {
    pub fn new(assets: PathBuf) -> Self {
        Self {
            dir: assets.join("icons"),
            cache: HashMap::new(),
        }
    }

    pub fn get(
        &mut self,
        ctx: &egui::Context,
        kind: &'static str,
        id: &str,
    ) -> Option<TextureHandle> {
        let key = (kind, id.to_string());
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }
        let path = self
            .dir
            .join(kind)
            .join(format!("{}.png", id.to_lowercase()));
        let texture = image::open(path).ok().map(|img| {
            let img = img.to_rgba8();
            let size = [img.width() as usize, img.height() as usize];
            ctx.load_texture(
                format!("{kind}/{id}"),
                egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()),
                TextureOptions::LINEAR,
            )
        });
        self.cache.insert(key, texture.clone());
        texture
    }

    /// Paints an icon in `rect`; returns false when there is none.
    pub fn paint(
        &mut self,
        painter: &egui::Painter,
        kind: &'static str,
        id: &str,
        rect: Rect,
    ) -> bool {
        match self.get(painter.ctx(), kind, id) {
            Some(texture) => {
                painter.image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                true
            }
            None => false,
        }
    }

    /// An icon as an inline widget (for lists).
    pub fn show(&mut self, ui: &mut egui::Ui, kind: &'static str, id: &str, size: f32) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
        self.paint(ui.painter(), kind, id, rect);
    }
}

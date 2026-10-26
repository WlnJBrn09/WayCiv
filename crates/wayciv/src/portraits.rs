//! Leader portraits from `assets/leaders/`, loaded on first use and
//! downscaled for display.

use crate::palette::{self, GOLD_HI, GOLD_LO};
use crate::ui::theme;
use egui::{Color32, TextureHandle, TextureOptions};
use std::collections::HashMap;
use std::path::PathBuf;

const MAX_SIZE: u32 = 512;

/// Width of the gold picture frame around a portrait.
pub const FRAME: f32 = 7.0;

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
        .or_else(|| {
            data_home
                .map(|d| d.join("wayciv/assets"))
                .filter(|p| p.is_dir())
        })
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|e| e.parent().map(|p| p.join("assets")))
                .filter(|p| p.is_dir())
        })
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets"))
}

impl Portraits {
    pub fn new() -> Self {
        Self {
            dir: assets_dir().join("leaders"),
            cache: HashMap::new(),
        }
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
            ctx.load_texture(
                leader,
                egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()),
                TextureOptions::LINEAR,
            )
        });
        self.cache.insert(leader.to_string(), texture.clone());
        texture
    }

    /// Draws a leader's portrait in `rect` inside a gold picture frame (which
    /// extends [`FRAME`] pixels beyond `rect`) with a thin line in the civ's
    /// color around the painting; a monogram in the civ's colors when no
    /// portrait exists.
    pub fn paint(
        &mut self,
        ui: &egui::Ui,
        rect: egui::Rect,
        leader: &str,
        name: &str,
        colors: ([u8; 3], [u8; 3]),
    ) {
        let painter = ui.painter();
        let primary = Color32::from_rgb(colors.0[0], colors.0[1], colors.0[2]);
        let secondary = Color32::from_rgb(colors.1[0], colors.1[1], colors.1[2]);
        let frame = rect.expand(FRAME);
        let radius = 4.0;
        painter.add(egui::Shadow {
            offset: [0, 4],
            blur: 12,
            spread: 0,
            color: Color32::from_black_alpha(150),
        }
        .as_shape(frame, egui::CornerRadius::same(radius as u8)));
        painter.add(theme::gradient_rect(frame, radius, GOLD_HI, GOLD_LO));
        painter.rect_stroke(
            frame,
            radius,
            egui::Stroke::new(1.0, Color32::from_black_alpha(220)),
            egui::StrokeKind::Outside,
        );
        // Bevel: a dark channel inside the gold, then the civ-colored line.
        let channel = rect.expand(FRAME * 0.42);
        painter.add(theme::gradient_rect(
            channel,
            2.0,
            Color32::from_rgb(40, 30, 12),
            Color32::from_rgb(70, 54, 22),
        ));
        painter.rect_filled(rect.expand(1.5), 1.0, primary);
        match self.get(ui.ctx(), leader) {
            Some(texture) => {
                painter.image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                // A soft vignette, deeper toward the lower edge.
                for (inset, alpha) in [(0.0, 90u8), (2.0, 50), (5.0, 24)] {
                    painter.rect_stroke(
                        rect.shrink(inset + 1.0),
                        0.0,
                        egui::Stroke::new(2.0, Color32::from_black_alpha(alpha)),
                        egui::StrokeKind::Inside,
                    );
                }
                painter.add(theme::gradient_rect(
                    egui::Rect::from_min_max(
                        egui::pos2(rect.left(), rect.center().y + rect.height() * 0.18),
                        rect.max,
                    ),
                    0.0,
                    Color32::TRANSPARENT,
                    Color32::from_black_alpha(70),
                ));
            }
            None => {
                painter.rect_filled(rect, 0.0, primary);
                let initials: String = name
                    .split_whitespace()
                    .filter_map(|w| w.chars().next())
                    .take(2)
                    .collect();
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    initials,
                    palette::bold(rect.height() * 0.4),
                    secondary,
                );
            }
        }
    }
}

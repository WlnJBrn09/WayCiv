//! Texture atlases for the terrain art. The egui wgpu backend does not
//! build mipmaps, so each atlas keeps a few pre-filtered levels (cells of
//! 256, 128, 64 and 32 px) and the renderer picks one by zoom, which keeps
//! zoomed-out tiles from shimmering.

use egui::{Color32, ColorImage, TextureFilter, TextureHandle, TextureId, TextureOptions};
use std::path::Path;

pub const CELL: usize = 256;
pub const LEVELS: usize = 4;

pub struct Atlas {
    name: &'static str,
    pub w: usize,
    pub h: usize,
    /// Premultiplied pixels of level 0.
    base: Vec<Color32>,
    handles: [Option<TextureHandle>; LEVELS],
}

/// A decoded PNG as premultiplied pixels.
pub struct Decoded {
    pub w: usize,
    pub h: usize,
    pub px: Vec<Color32>,
}

pub fn decode(path: &Path) -> Option<Decoded> {
    let img = image::open(path).ok()?.to_rgba8();
    let (w, h) = (img.width() as usize, img.height() as usize);
    let px = img
        .pixels()
        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    Some(Decoded { w, h, px })
}

impl Atlas {
    pub fn new(name: &'static str, w: usize, h: usize, base: Vec<Color32>) -> Self {
        debug_assert_eq!(base.len(), w * h);
        Self {
            name,
            w,
            h,
            base,
            handles: [None, None, None, None],
        }
    }

    /// Cells per row and per column at level 0.
    pub fn cols(&self) -> usize {
        self.w / CELL
    }

    /// The texture for a level, uploading it on first use.
    pub fn texture(&mut self, ctx: &egui::Context, level: usize) -> TextureId {
        let level = level.min(LEVELS - 1);
        if self.handles[level].is_none() {
            let f = 1usize << level;
            let (w, h) = (self.w / f, self.h / f);
            let px = if f == 1 {
                self.base.clone()
            } else {
                let mut out = Vec::with_capacity(w * h);
                for y in 0..h {
                    for x in 0..w {
                        let mut s = [0u32; 4];
                        for dy in 0..f {
                            let row = (y * f + dy) * self.w + x * f;
                            for dx in 0..f {
                                let p = self.base[row + dx].to_array();
                                for k in 0..4 {
                                    s[k] += p[k] as u32;
                                }
                            }
                        }
                        let n = (f * f) as u32;
                        out.push(Color32::from_rgba_premultiplied(
                            (s[0] / n) as u8,
                            (s[1] / n) as u8,
                            (s[2] / n) as u8,
                            (s[3] / n) as u8,
                        ));
                    }
                }
                out
            };
            let options = TextureOptions {
                magnification: TextureFilter::Linear,
                minification: TextureFilter::Linear,
                wrap_mode: egui::TextureWrapMode::ClampToEdge,
                mipmap_mode: Some(TextureFilter::Linear),
            };
            self.handles[level] = Some(ctx.load_texture(
                format!("terrain/{}/{level}", self.name),
                ColorImage::new([w, h], px),
                options,
            ));
        }
        self.handles[level].as_ref().map(|h| h.id()).unwrap()
    }
}

/// Which level to use when a cell covers `onscreen` physical pixels.
pub fn level_for(onscreen: f32) -> usize {
    let mut level = 0;
    while level + 1 < LEVELS && (CELL >> (level + 1)) as f32 >= onscreen * 0.75 {
        level += 1;
    }
    level
}

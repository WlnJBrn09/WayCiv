//! Notifications: hidden by default. `N` opens the list; new ones also pop up
//! as small toasts at the right edge that fade after a few seconds and stay
//! clear of the middle of the map.

use super::glyph::Glyph;
use super::{MARGIN, TOP_BAR, glyph, theme};
use crate::hud::Session;
use crate::map_view;
use crate::palette::{self, BAD, GOLD, GOLD_DIM, NAVY, NAVY_HOVER, NAVY_RAISED, TEXT, TEXT_DIM};
use egui::{Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use std::time::{Duration, Instant};
use wayciv_core::Pos;

const TOAST_LIFE: f32 = 6.0;
const TOAST_FADE: f32 = 1.2;
const MAX_TOASTS: usize = 4;

pub struct Toast {
    pub text: String,
    pub pos: Option<Pos>,
    pub at: Instant,
}

/// Where we are in `game.notifications` (which is trimmed from the front).
#[derive(Default)]
pub struct Cursor {
    initialized: bool,
    marker: Option<(u32, usize, String)>,
    /// Notifications for you that arrived since the panel was last open.
    pub unread: usize,
    pub toasts: Vec<Toast>,
}

impl Cursor {
    /// Test hook: treats the last `n` notifications as not yet seen.
    pub fn replay(&mut self, game: &wayciv_core::Game, n: usize) {
        self.initialized = true;
        let mine: Vec<usize> = (0..game.notifications.len())
            .filter(|&i| game.notifications[i].player == game.human())
            .collect();
        self.marker = mine
            .len()
            .checked_sub(n)
            .and_then(|k| mine[k].checked_sub(1))
            .map(|i| &game.notifications[i])
            .map(|n| (n.turn, n.player, n.text.clone()));
    }

    /// Picks up notifications that arrived since the last call.
    pub fn ingest(&mut self, game: &wayciv_core::Game, me: usize) {
        let list = &game.notifications;
        let key = |n: &wayciv_core::state::Notification| (n.turn, n.player, n.text.clone());
        let start = if !self.initialized {
            self.initialized = true;
            list.len()
        } else {
            match &self.marker {
                None => 0,
                Some(m) => list
                    .iter()
                    .rposition(|n| &key(n) == m)
                    .map(|i| i + 1)
                    .unwrap_or(0),
            }
        };
        for n in &list[start.min(list.len())..] {
            if n.player == me {
                self.unread += 1;
                self.toasts.push(Toast {
                    text: n.text.clone(),
                    pos: n.pos,
                    at: Instant::now(),
                });
            }
        }
        self.marker = list.last().map(key).or(self.marker.take());
        self.toasts
            .retain(|t| t.at.elapsed() < Duration::from_secs_f32(TOAST_LIFE));
        let extra = self.toasts.len().saturating_sub(MAX_TOASTS * 2);
        self.toasts.drain(..extra);
    }
}

/// Top-left corner of the right-hand stack (toasts and the panel).
fn stack_origin(s: &Session, ctx: &egui::Context, width: f32) -> egui::Pos2 {
    let screen = ctx.content_rect();
    let mut x = screen.right() - MARGIN - width;
    if s.selected_city.is_some() {
        x -= super::city_panel::WIDTH + MARGIN;
    }
    let ribbon = if super::leaders::met(s).is_empty() { 0.0 } else { 90.0 };
    pos2(x, TOP_BAR + MARGIN + ribbon)
}

pub fn toasts(s: &mut Session, ctx: &egui::Context) {
    s.notifications.ingest(&s.game, s.me());
    if s.notifications.toasts.is_empty() {
        return;
    }
    if s.windows.notifications {
        s.notifications.toasts.clear();
        return;
    }
    ctx.request_repaint_after(Duration::from_millis(50));
    let width = 320.0;
    let origin = stack_origin(s, ctx, width);
    let shown: Vec<(usize, String, Option<Pos>, f32)> = s
        .notifications
        .toasts
        .iter()
        .enumerate()
        .rev()
        .take(MAX_TOASTS)
        .map(|(i, t)| {
            let age = t.at.elapsed().as_secs_f32();
            let alpha = ((TOAST_LIFE - age) / TOAST_FADE).clamp(0.0, 1.0);
            (i, t.text.clone(), t.pos, alpha)
        })
        .collect();
    let hidden = s.notifications.toasts.len().saturating_sub(MAX_TOASTS);
    let mut jump = None;
    egui::Area::new(egui::Id::unique("toasts"))
        .fixed_pos(origin)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            for (i, text, pos, alpha) in &shown {
                let font = FontId::proportional(14.5);
                let galley = ui.painter().layout(
                    text.clone(),
                    font,
                    TEXT.gamma_multiply(*alpha),
                    width - 62.0,
                );
                let h = (galley.size().y + 18.0).max(44.0);
                let (rect, response) = ui.allocate_exact_size(vec2(width, h), Sense::click());
                let painter = ui.painter();
                let a = |c: Color32| c.gamma_multiply(*alpha);
                painter.rect_filled(rect, 12.0, a(palette::with_alpha(NAVY, 235)));
                painter.rect_stroke(
                    rect,
                    12.0,
                    Stroke::new(1.5, a(if response.hovered() { GOLD } else { GOLD_DIM })),
                    StrokeKind::Inside,
                );
                glyph::paint(painter, Glyph::Bell, pos2(rect.left() + 24.0, rect.center().y), 9.0, a(GOLD));
                painter.galley(pos2(rect.left() + 46.0, rect.center().y - galley.size().y / 2.0), galley, TEXT);
                if response.clicked() {
                    s_jump(&mut jump, *pos, *i);
                }
            }
            if hidden > 0 {
                ui.label(
                    egui::RichText::new(format!("+{hidden} more - press N"))
                        .color(TEXT_DIM)
                        .size(13.0),
                );
            }
        });
    if let Some((pos, i)) = jump {
        if let Some(pos) = pos {
            map_view::center_on(s, pos);
        }
        if i < s.notifications.toasts.len() {
            s.notifications.toasts.remove(i);
        }
    }
}

fn s_jump(jump: &mut Option<(Option<Pos>, usize)>, pos: Option<Pos>, i: usize) {
    *jump = Some((pos, i));
}

/// The Notifications panel (N): newest first; clicking jumps to the location.
pub fn panel(s: &mut Session, ctx: &egui::Context) {
    if !s.windows.notifications {
        return;
    }
    s.notifications.unread = 0;
    s.notifications.toasts.clear();
    let me = s.me();
    let width = 380.0;
    let origin = stack_origin(s, ctx, width);
    let max_h = (ctx.content_rect().height() - origin.y - 170.0).clamp(220.0, 560.0);
    let items: Vec<_> = s
        .game
        .notifications
        .iter()
        .rev()
        .filter(|n| n.player == me)
        .take(60)
        .cloned()
        .collect();
    let turn = s.game.turn;
    let mut jump = None;
    let mut close = false;
    egui::Area::new(egui::Id::unique("notifications_panel"))
        .fixed_pos(origin)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            theme::plate_frame().show(ui, |ui| {
                ui.set_width(width - 26.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Notifications").size(20.0).strong().color(GOLD));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::round_button(ui, 30.0, Glyph::Cross, TEXT, None, true)
                            .on_hover_text("Close (N or Esc)")
                            .clicked()
                        {
                            close = true;
                        }
                    });
                });
                ui.add_space(2.0);
                if items.is_empty() {
                    ui.label(egui::RichText::new("Nothing to report yet.").color(TEXT_DIM));
                }
                egui::ScrollArea::vertical().max_height(max_h).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    for n in &items {
                        let current = n.turn + 1 >= turn;
                        let color = if current { TEXT } else { TEXT_DIM };
                        let galley = ui.painter().layout(
                            n.text.clone(),
                            FontId::proportional(14.5),
                            color,
                            ui.available_width() - 74.0,
                        );
                        let h = galley.size().y + 14.0;
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::click());
                        let painter = ui.painter();
                        painter.rect_filled(
                            rect,
                            9.0,
                            if response.hovered() && n.pos.is_some() { NAVY_HOVER } else { NAVY_RAISED },
                        );
                        if current {
                            painter.rect_filled(
                                Rect::from_min_size(rect.min + vec2(0.0, 6.0), vec2(4.0, h - 12.0)),
                                2.0,
                                GOLD,
                            );
                        }
                        theme::text(painter, pos2(rect.left() + 12.0, rect.top() + 7.0), Align2::LEFT_TOP, format!("T{}", n.turn), 12.5, GOLD_DIM);
                        painter.galley(pos2(rect.left() + 46.0, rect.top() + 7.0), galley, color);
                        if n.pos.is_some() {
                            glyph::paint(painter, Glyph::Arrow, pos2(rect.right() - 16.0, rect.center().y), 7.0, GOLD);
                        }
                        if n.pos.is_some() && response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            jump = n.pos;
                        }
                    }
                });
            });
        });
    if let Some(pos) = jump {
        map_view::center_on(s, pos);
    }
    if close {
        s.windows.close_all();
    }
    let _ = BAD;
}

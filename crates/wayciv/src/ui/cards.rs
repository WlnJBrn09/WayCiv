//! Building blocks of the menus: clickable cards laid out in grids, the
//! "what to do here" instruction strip, progress rows and the big action
//! button. Every window uses these so they look and behave the same: a
//! card that can be clicked lights up under the mouse, a chosen card has a
//! gold frame, and a card you can't pick yet is dimmed with the reason in
//! its tooltip.

use super::glyph::{self, Glyph};
use super::theme;
use crate::icons::Icons;
use crate::palette::{self, GOLD, GOLD_DIM, NAVY_HOVER, NAVY_RAISED, TEXT, TEXT_DIM};
use egui::{
    Align2, Color32, CursorIcon, FontId, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2,
};

/// Gap between cards.
pub const GAP: f32 = 10.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Can be clicked.
    Open,
    /// Already chosen: gold frame (can still be clicked, e.g. to remove it).
    Chosen,
    /// Can't be picked right now: dimmed, not clickable.
    Locked,
    /// Only shows information.
    Info,
}

pub struct Card<'a> {
    /// An icon from `assets/icons/<kind>/<id>.png`.
    pub icon: Option<(&'static str, &'a str)>,
    /// Drawn when there is no icon.
    pub glyph: Glyph,
    pub title: &'a str,
    pub body: &'a str,
    /// Type color: the strip on the left and the glyph.
    pub accent: Color32,
    pub state: State,
    /// A small label in the top-right corner ("Current", "25 Gold").
    pub corner: Option<(&'a str, Color32)>,
    /// A dim last line.
    pub footer: Option<&'a str>,
    /// A progress bar along the bottom: (fraction, color).
    pub bar: Option<(f32, Color32)>,
}

impl<'a> Card<'a> {
    pub fn new(title: &'a str, body: &'a str) -> Self {
        Card {
            icon: None,
            glyph: Glyph::Star,
            title,
            body,
            accent: GOLD,
            state: State::Open,
            corner: None,
            footer: None,
            bar: None,
        }
    }
    pub fn icon(mut self, kind: &'static str, id: &'a str) -> Self {
        self.icon = Some((kind, id));
        self
    }
    pub fn glyph(mut self, g: Glyph) -> Self {
        self.glyph = g;
        self
    }
    pub fn accent(mut self, c: Color32) -> Self {
        self.accent = c;
        self
    }
    pub fn state(mut self, s: State) -> Self {
        self.state = s;
        self
    }
    pub fn corner(mut self, label: &'a str, color: Color32) -> Self {
        self.corner = Some((label, color));
        self
    }
    pub fn footer(mut self, s: &'a str) -> Self {
        self.footer = Some(s);
        self
    }
    pub fn bar(mut self, frac: f32, color: Color32) -> Self {
        self.bar = Some((frac, color));
        self
    }

    /// Draws the card `width` wide; its height follows the text.
    pub fn show(&self, ui: &mut Ui, icons: &mut Icons, width: f32) -> Response {
        let icon_d = 52.0;
        let pad = 10.0;
        let text_x = pad + 6.0 + icon_d + 10.0;
        let text_w = (width - text_x - pad).max(40.0);
        let locked = self.state == State::Locked;
        let dim = |c: Color32| if locked { c.gamma_multiply(0.5) } else { c };
        let painter = ui.painter().clone();
        let corner_w = self.corner.map_or(0.0, |(label, _)| {
            painter
                .layout_no_wrap(label.to_string(), FontId::proportional(12.0), TEXT)
                .size()
                .x
                + 18.0
        });
        let title = painter.layout(
            self.title.to_string(),
            FontId::proportional(16.5),
            dim(TEXT),
            text_w - corner_w,
        );
        let body = painter.layout(
            self.body.to_string(),
            FontId::proportional(13.5),
            dim(TEXT_DIM),
            text_w,
        );
        let footer = self.footer.map(|f| {
            painter.layout(
                f.to_string(),
                FontId::proportional(12.5),
                dim(self.accent.gamma_multiply(0.9)),
                text_w,
            )
        });
        let text_h = title.size().y
            + if self.body.is_empty() {
                0.0
            } else {
                3.0 + body.size().y
            }
            + footer.as_ref().map_or(0.0, |f| 4.0 + f.size().y);
        let bar_h = if self.bar.is_some() { 14.0 } else { 0.0 };
        let height = (text_h + 2.0 * pad + 2.0).max(icon_d + 2.0 * pad) + bar_h;
        let sense = match self.state {
            State::Open | State::Chosen => Sense::click(),
            State::Locked | State::Info => Sense::hover(),
        };
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), sense);
        let clickable = matches!(self.state, State::Open | State::Chosen);
        let hovered = response.hovered() && clickable;
        let chosen = self.state == State::Chosen;
        let fill = if chosen {
            Color32::from_rgb(58, 48, 22)
        } else if hovered {
            NAVY_HOVER
        } else if locked {
            Color32::from_rgb(16, 24, 38)
        } else {
            NAVY_RAISED
        };
        painter.rect_filled(rect, 12.0, fill);
        // The type strip on the left.
        painter.rect_filled(
            Rect::from_min_size(rect.min + vec2(0.0, 10.0), vec2(4.0, rect.height() - 20.0)),
            2.0,
            dim(self.accent),
        );
        let edge = if chosen {
            Stroke::new(2.5, GOLD)
        } else if hovered {
            Stroke::new(2.0, GOLD)
        } else if locked {
            Stroke::new(1.0, Color32::from_rgb(44, 56, 74))
        } else {
            Stroke::new(1.5, GOLD_DIM.gamma_multiply(0.8))
        };
        painter.rect_stroke(rect, 12.0, edge, StrokeKind::Inside);
        let ic = pos2(
            rect.left() + pad + 6.0 + icon_d / 2.0,
            rect.top() + pad + icon_d / 2.0,
        );
        painter.circle_filled(ic, icon_d / 2.0, Color32::from_rgb(10, 18, 30));
        let painted = self.icon.is_some_and(|(kind, id)| {
            icons.paint(
                &painter,
                kind,
                id,
                Rect::from_center_size(ic, vec2(icon_d - 4.0, icon_d - 4.0)),
            )
        });
        if !painted {
            glyph::paint(&painter, self.glyph, ic, 13.0, dim(self.accent));
        }
        if locked {
            painter.circle_filled(ic, icon_d / 2.0, Color32::from_black_alpha(120));
        }
        painter.circle_stroke(
            ic,
            icon_d / 2.0,
            Stroke::new(1.5, if chosen { GOLD } else { dim(GOLD_DIM) }),
        );
        let mut y = rect.top() + pad + 1.0;
        let x = rect.left() + text_x;
        let title_h = title.size().y;
        painter.galley(pos2(x, y), title.clone(), TEXT);
        painter.galley(pos2(x + 0.5, y), title, TEXT);
        y += title_h + 3.0;
        if !self.body.is_empty() {
            let h = body.size().y;
            painter.galley(pos2(x, y), body, TEXT_DIM);
            y += h + 4.0;
        }
        if let Some(f) = footer {
            painter.galley(pos2(x, y), f, TEXT_DIM);
        }
        if let Some((frac, color)) = self.bar {
            theme::bar(
                &painter,
                Rect::from_min_size(
                    pos2(rect.left() + pad + 6.0, rect.bottom() - pad - 6.0),
                    vec2(rect.width() - 2.0 * pad - 6.0, 7.0),
                ),
                frac,
                dim(color),
            );
        }
        if let Some((label, color)) = self.corner {
            let galley =
                painter.layout_no_wrap(label.to_string(), FontId::proportional(12.0), color);
            let r = Rect::from_min_size(
                pos2(
                    rect.right() - pad - galley.size().x - 12.0,
                    rect.top() + pad - 1.0,
                ),
                vec2(galley.size().x + 12.0, 20.0),
            );
            painter.rect_filled(r, 10.0, color.gamma_multiply(0.18));
            painter.rect_stroke(
                r,
                10.0,
                Stroke::new(1.0, color.gamma_multiply(0.7)),
                StrokeKind::Inside,
            );
            painter.galley(r.center() - galley.size() / 2.0, galley, color);
        }
        if clickable {
            response.on_hover_cursor(CursorIcon::PointingHand)
        } else {
            response
        }
    }
}

/// Lays out `count` cards in as many columns of at least `min_width` as
/// fit; `f(ui, index, width)` draws one.
pub fn grid(ui: &mut Ui, min_width: f32, count: usize, mut f: impl FnMut(&mut Ui, usize, f32)) {
    let avail = ui.available_width();
    let columns = (((avail + GAP) / (min_width + GAP)).floor() as usize).clamp(1, count.max(1));
    let width = ((avail - GAP * (columns as f32 - 1.0)) / columns as f32).floor();
    let mut i = 0;
    while i < count {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = GAP;
            for j in i..(i + columns).min(count) {
                f(ui, j, width);
            }
        });
        ui.add_space(GAP - ui.spacing().item_spacing.y);
        i += columns;
    }
}

/// The strip under a window's title that says what to do next.
pub fn instruction(ui: &mut Ui, text: &str) {
    let width = ui.available_width();
    let galley = ui.painter().layout(
        text.to_string(),
        FontId::proportional(16.0),
        TEXT,
        width - 52.0,
    );
    let (rect, _) = ui.allocate_exact_size(vec2(width, galley.size().y + 18.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 10.0, palette::with_alpha(palette::TEAL_DIM, 110));
    painter.rect_stroke(
        rect,
        10.0,
        Stroke::new(1.0, palette::with_alpha(palette::TEAL, 120)),
        StrokeKind::Inside,
    );
    let c = pos2(rect.left() + 22.0, rect.center().y);
    painter.circle_filled(c, 10.0, palette::TEAL);
    glyph::paint(painter, Glyph::Arrow, c, 5.5, palette::NAVY);
    painter.galley(
        pos2(rect.left() + 42.0, rect.center().y - galley.size().y / 2.0),
        galley,
        TEXT,
    );
    ui.add_space(6.0);
}

/// A section title with a short dim explanation after it.
pub fn section(ui: &mut Ui, title: &str, hint: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).size(18.0).strong().color(GOLD));
        if !hint.is_empty() {
            ui.label(egui::RichText::new(hint).size(13.5).color(TEXT_DIM));
        }
    });
    let y = ui.cursor().top() + 1.0;
    let left = ui.min_rect().left();
    ui.painter().line_segment(
        [pos2(left, y), pos2(left + ui.available_width(), y)],
        Stroke::new(1.0, palette::with_alpha(GOLD_DIM, 150)),
    );
    ui.add_space(8.0);
}

/// A labeled progress bar: `label` on the left, `value` on the right, a
/// bar underneath and a dim `hint` line.
pub fn progress(
    ui: &mut Ui,
    g: Glyph,
    color: Color32,
    label: &str,
    value: &str,
    frac: f32,
    hint: &str,
) {
    let width = ui.available_width();
    let hint_galley = (!hint.is_empty()).then(|| {
        ui.painter().layout(
            hint.to_string(),
            FontId::proportional(13.0),
            TEXT_DIM,
            width - 40.0,
        )
    });
    let h = 44.0 + hint_galley.as_ref().map_or(0.0, |g| g.size().y + 2.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, h), Sense::hover());
    let painter = ui.painter();
    let c = pos2(rect.left() + 14.0, rect.top() + 13.0);
    glyph::paint(painter, g, c, 8.0, color);
    theme::text_bold(
        painter,
        pos2(rect.left() + 32.0, c.y),
        Align2::LEFT_CENTER,
        label,
        16.0,
        TEXT,
    );
    theme::text(
        painter,
        pos2(rect.right(), c.y),
        Align2::RIGHT_CENTER,
        value,
        14.5,
        color,
    );
    theme::bar(
        painter,
        Rect::from_min_size(
            pos2(rect.left() + 32.0, rect.top() + 26.0),
            vec2(width - 32.0, 8.0),
        ),
        frac,
        color,
    );
    if let Some(g) = hint_galley {
        painter.galley(pos2(rect.left() + 32.0, rect.top() + 40.0), g, TEXT_DIM);
    }
    let _ = response;
    ui.add_space(4.0);
}

/// The window's main action ("Confirm Policies", "Found Religion"): wide,
/// gold when it can be pressed, gray with the reason in its tooltip when
/// it can't.
pub fn action_button(ui: &mut Ui, label: &str, enabled: bool, why_not: &str) -> bool {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), FontId::proportional(18.0), TEXT);
    let w = (galley.size().x + 60.0).max(240.0);
    let (rect, response) = ui.allocate_exact_size(
        vec2(w, 46.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let hovered = response.hovered() && enabled;
    let painter = ui.painter();
    let (fill, edge, text) = if enabled {
        (
            if hovered {
                Color32::from_rgb(120, 92, 30)
            } else {
                Color32::from_rgb(92, 70, 24)
            },
            GOLD,
            Color32::WHITE,
        )
    } else {
        (
            Color32::from_rgb(28, 36, 50),
            Color32::from_rgb(60, 70, 86),
            Color32::from_gray(130),
        )
    };
    painter.rect_filled(rect, 23.0, fill);
    painter.rect_stroke(rect, 23.0, Stroke::new(2.0, edge), StrokeKind::Inside);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), FontId::proportional(18.0), text);
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, text);
    let response = if enabled {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else if !why_not.is_empty() {
        response.on_hover_text(why_not)
    } else {
        response
    };
    response.clicked() && enabled
}

/// A pill toggle used for short choices such as a religion's name.
pub fn choice(ui: &mut Ui, label: &str, color: Color32, selected: bool) -> bool {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), FontId::proportional(15.5), TEXT);
    let (rect, response) =
        ui.allocate_exact_size(vec2(galley.size().x + 44.0, 34.0), Sense::click());
    let painter = ui.painter();
    let hovered = response.hovered();
    painter.rect_filled(
        rect,
        17.0,
        if selected {
            Color32::from_rgb(58, 48, 22)
        } else if hovered {
            NAVY_HOVER
        } else {
            NAVY_RAISED
        },
    );
    painter.rect_stroke(
        rect,
        17.0,
        Stroke::new(
            if selected { 2.5 } else { 1.5 },
            if selected || hovered { GOLD } else { GOLD_DIM },
        ),
        StrokeKind::Inside,
    );
    painter.circle_filled(pos2(rect.left() + 17.0, rect.center().y), 6.0, color);
    painter.galley(
        pos2(rect.left() + 30.0, rect.center().y - galley.size().y / 2.0),
        galley,
        TEXT,
    );
    response.on_hover_cursor(CursorIcon::PointingHand).clicked()
}

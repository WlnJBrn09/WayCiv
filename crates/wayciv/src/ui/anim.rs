//! A small animation toolkit: easing curves, eased hover/value trackers kept
//! in egui's memory, one-shot pulses, and the fade/scale effect windows use
//! to open and close. Nothing here repaints while idle: every helper asks
//! for another frame only while something is still moving (or, for the
//! slow ambient glows, at a modest `AMBIENT_MS`).

use egui::emath::TSTransform;
use egui::epaint::ClippedShape;
use egui::{
    Color32, Context, Id, LayerId, Order, Pos2, Rect, Shape, epaint::ColorMode, layers::ShapeIdx, vec2,
};
use std::f32::consts::{PI, TAU};
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

/// How long windows take to open and to close.
pub const OPEN_SECS: f32 = 0.2;
pub const CLOSE_SECS: f32 = 0.14;
/// Frame interval for ambient glows (breathing, pulses).
pub const AMBIENT_MS: u64 = 33;

// ------------------------------------------------------------------ easing

/// Fast start, soft landing.
pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}

/// Slow start and end.
pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Overshoots past 1 and settles: for pops and bounces.
pub fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) - 1.0;
    let c = 1.9;
    1.0 + t * t * ((c + 1.0) * t + c)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Blends two colors in gamma space.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    a.lerp_to_gamma(b, t.clamp(0.0, 1.0))
}

/// A slow 0..1 wave with the given period in seconds.
pub fn wave(ctx: &Context, period: f32) -> f32 {
    0.5 - 0.5 * (now(ctx) as f32 / period * TAU).cos()
}

/// A bump that rises and falls over `t` in 0..1 (0 at both ends).
pub fn bump(t: f32) -> f32 {
    (t.clamp(0.0, 1.0) * PI).sin()
}

// ------------------------------------------------------------------- clock

pub fn now(ctx: &Context) -> f64 {
    ctx.input(|i| i.time)
}

/// Asks for another frame soon, for glows that never settle.
pub fn ambient(ctx: &Context) {
    ctx.request_repaint_after(Duration::from_millis(AMBIENT_MS));
}

pub fn hash(value: &impl Hash) -> u64 {
    use std::hash::Hasher;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut h);
    h.finish()
}

// ---------------------------------------------------------------- trackers

/// 0..1 easing toward `on` over `secs`; starts settled on its first call.
pub fn hover(ctx: &Context, id: Id, on: bool, secs: f32) -> f32 {
    ctx.animate_bool_with_time_and_easing(id, on, secs, egui::emath::easing::cubic_out)
}

/// Eases toward `target` (exponential approach, `rate` per second); the
/// first call returns `target` so new things appear in place.
pub fn smooth(ctx: &Context, id: Id, target: f32, rate: f32) -> f32 {
    let dt = ctx.input(|i| i.stable_dt).min(0.1);
    let Some(cur) = ctx.data(|d| d.get_temp::<f32>(id)) else {
        ctx.data_mut(|d| d.insert_temp(id, target));
        return target;
    };
    let mut next = target + (cur - target) * (-rate * dt).exp();
    if (next - target).abs() < 0.002 * target.abs().max(1.0) {
        next = target;
    } else {
        ctx.request_repaint();
    }
    ctx.data_mut(|d| d.insert_temp(id, next));
    next
}

/// Starts a one-shot animation.
pub fn trigger(ctx: &Context, id: Id) {
    let t = now(ctx);
    ctx.data_mut(|d| d.insert_temp(id, t));
}

/// Progress 0..1 of a one-shot animation started with `trigger`; `None`
/// when it isn't running.
pub fn progress(ctx: &Context, id: Id, secs: f32) -> Option<f32> {
    let start = ctx.data(|d| d.get_temp::<f64>(id))?;
    let p = (now(ctx) - start) as f32 / secs;
    if p >= 1.0 || p < 0.0 {
        return None;
    }
    ctx.request_repaint();
    Some(p)
}

/// True when `key` differs from the last call's (never on the first call).
pub fn changed(ctx: &Context, id: Id, key: u64) -> bool {
    let old = ctx.data(|d| d.get_temp::<u64>(id));
    ctx.data_mut(|d| d.insert_temp(id, key));
    old.is_some_and(|o| o != key)
}

// ------------------------------------------------------------ shape fading

/// Multiplies the opacity of every color in `shape` by `a`.
pub fn fade_shape(shape: &mut Shape, a: f32) {
    let f = |c: &mut Color32| *c = c.gamma_multiply(a);
    let stroke = |s: &mut egui::Stroke| s.color = s.color.gamma_multiply(a);
    let path_stroke = |s: &mut egui::epaint::PathStroke| {
        if let ColorMode::Solid(c) = &mut s.color {
            *c = c.gamma_multiply(a);
        }
    };
    match shape {
        Shape::Noop | Shape::Callback(_) => {}
        Shape::Vec(v) => v.iter_mut().for_each(|s| fade_shape(s, a)),
        Shape::Circle(c) => {
            f(&mut c.fill);
            stroke(&mut c.stroke);
        }
        Shape::Ellipse(e) => {
            f(&mut e.fill);
            stroke(&mut e.stroke);
        }
        Shape::LineSegment { stroke: s, .. } => stroke(s),
        Shape::Path(p) => {
            f(&mut p.fill);
            path_stroke(&mut p.stroke);
        }
        Shape::Band(b) => {
            f(&mut b.fill);
            stroke(&mut b.stroke);
        }
        Shape::Rect(r) => {
            f(&mut r.fill);
            stroke(&mut r.stroke);
        }
        Shape::Text(t) => {
            t.opacity_factor *= a;
            stroke(&mut t.underline);
        }
        Shape::Mesh(m) => Arc::make_mut(m)
            .vertices
            .iter_mut()
            .for_each(|v| v.color = v.color.gamma_multiply(a)),
        Shape::QuadraticBezier(b) => {
            f(&mut b.fill);
            path_stroke(&mut b.stroke);
        }
        Shape::CubicBezier(b) => {
            f(&mut b.fill);
            path_stroke(&mut b.stroke);
        }
    }
}

/// Fades everything already painted into `layer`.
pub fn fade_layer(ctx: &Context, layer: LayerId, a: f32) {
    ctx.graphics_mut(|g| {
        let Some(list) = g.get_mut(layer) else { return };
        let n = list.all_entries().len();
        for i in 0..n {
            list.mutate_shape(ShapeIdx(i), |cs| fade_shape(&mut cs.shape, a));
        }
    });
}

/// Scales `layer`'s graphics about `center` and shifts them by `dy`.
fn zoom_layer(ctx: &Context, layer: LayerId, center: Pos2, scale: f32, dy: f32) {
    let t = TSTransform::new(center.to_vec2() * (1.0 - scale) + vec2(0.0, dy), scale);
    ctx.transform_layer_shapes_after_rounding(layer, t);
}

// ----------------------------------------------------------- window effects

/// The last drawn state of a window, kept so it can fade out after closing.
#[derive(Clone)]
struct Ghost {
    frame: u64,
    time: f64,
    rect: Rect,
    shapes: Arc<Vec<ClippedShape>>,
}

#[derive(Clone, Copy)]
struct Seen {
    frame: u64,
    since: f64,
}

fn registry() -> Id {
    Id::unique("anim_ghosts")
}

/// Call right after showing a window or panel: fades and scales it in
/// (`OPEN_SECS` after it first appears) and records it so `ghosts` can fade
/// it out once it is gone. `key` must be stable; `rect` is the window's rect.
/// `from_edge` picks a slide direction for panels that dock at a screen edge.
pub fn window_fx(ctx: &Context, key: Id, layer: LayerId, rect: Rect) {
    let frame = ctx.cumulative_frame_nr();
    let time = now(ctx);
    let seen = ctx.data(|d| d.get_temp::<Seen>(key.with("seen")));
    let since = match seen {
        Some(s) if s.frame + 1 >= frame => s.since,
        _ => time,
    };
    ctx.data_mut(|d| d.insert_temp(key.with("seen"), Seen { frame, since }));
    // Remember how it looks now, before the opening effect touches it.
    let shapes: Vec<ClippedShape> = ctx.graphics(|g| {
        g.get(layer)
            .map(|l| l.all_entries().cloned().collect())
            .unwrap_or_default()
    });
    ctx.data_mut(|d| {
        d.insert_temp(
            key.with("ghost"),
            Ghost {
                frame,
                time,
                rect,
                shapes: Arc::new(shapes),
            },
        );
        let mut keys: Vec<Id> = d.get_temp(registry()).unwrap_or_default();
        if !keys.contains(&key) {
            keys.push(key);
            d.insert_temp(registry(), keys);
        }
    });
    let t = ((time - since) as f32 / OPEN_SECS).clamp(0.0, 1.0);
    if t < 1.0 {
        ctx.request_repaint();
        let e = ease_out(t);
        fade_layer(ctx, layer, e);
        zoom_layer(ctx, layer, rect.center(), lerp(0.96, 1.0, e), (1.0 - e) * 8.0);
    }
}

/// Draws the fading remains of windows that closed within `CLOSE_SECS`.
/// Call once per frame after every window has been shown.
pub fn ghosts(ctx: &Context) {
    let keys: Vec<Id> = ctx.data(|d| d.get_temp(registry())).unwrap_or_default();
    if keys.is_empty() {
        return;
    }
    let frame = ctx.cumulative_frame_nr();
    let time = now(ctx);
    let mut keep = Vec::new();
    for key in keys {
        let Some(g) = ctx.data(|d| d.get_temp::<Ghost>(key.with("ghost"))) else {
            continue;
        };
        if g.frame >= frame {
            keep.push(key);
            continue;
        }
        let t = (time - g.time) as f32 / CLOSE_SECS;
        if t >= 1.0 {
            ctx.data_mut(|d| {
                d.remove::<Ghost>(key.with("ghost"));
                d.remove::<Seen>(key.with("seen"));
            });
            continue;
        }
        keep.push(key);
        let layer = LayerId::new(Order::Foreground, key.with("ghost_layer"));
        let a = 1.0 - ease_in_out(t);
        ctx.graphics_mut(|gl| {
            let list = gl.entry(layer);
            for cs in g.shapes.iter() {
                let mut shape = cs.shape.clone();
                fade_shape(&mut shape, a);
                list.add(cs.clip_rect, shape);
            }
        });
        zoom_layer(ctx, layer, g.rect.center(), lerp(1.0, 0.98, ease_out(t)), 0.0);
        ctx.request_repaint();
    }
    ctx.data_mut(|d| d.insert_temp(registry(), keep));
}

/// Dims everything under the windows (below the top bar), eased in and out.
pub fn backdrop(ctx: &Context, on: bool, alpha: u8) {
    let t = hover(ctx, Id::unique("anim_backdrop"), on, 0.22);
    if t <= 0.0 {
        return;
    }
    let mut rect = ctx.content_rect();
    rect.min.y += super::TOP_BAR;
    ctx.layer_painter(LayerId::new(Order::Middle, Id::unique("anim_dim")))
        .rect_filled(rect, 0.0, Color32::from_black_alpha((alpha as f32 * t) as u8));
}

/// Test hook: `WAYCIV_AT=<ms>:<action>,...` runs an action that many
/// milliseconds after the game starts: `endturn`, `research` (finish the
/// current research, then end the turn), `toast`, `open:<panel>`, `close`.
pub fn demo(s: &mut crate::hud::Session, action: &str) {
    use crate::screens::Panel;
    use wayciv_core::research::Tree;
    let me = s.me();
    match action {
        "endturn" => s.end_turn(),
        "research" => {
            for tree in [Tree::Tech, Tree::Civic] {
                if let Some(i) = s.game.current_research(me, tree) {
                    s.game.boost(me, tree, i, 100);
                }
            }
            s.end_turn();
        }
        "toast" => s.notifications.replay(&s.game, 3),
        "close" => s.windows.close_all(),
        other => {
            if let Some(p) = other.strip_prefix("open:") {
                match p {
                    "tech" => s.windows.open(Panel::Tech),
                    "civic" => s.windows.open(Panel::Civic),
                    "government" => s.windows.open(Panel::Government),
                    "religion" => s.windows.open(Panel::Religion),
                    "notifications" => s.windows.open(Panel::Notifications),
                    "victory" => s.windows.open(Panel::Victory),
                    "leaders" => s.windows.open(Panel::Leaders),
                    _ => {}
                }
            }
        }
    }
}

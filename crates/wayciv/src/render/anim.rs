//! Map animation bookkeeping: the clock, unit tracks (walks, lunges, hits,
//! deaths), one-off effects, tile fades, and the camera pan.
//!
//! The rules move units and settle fights instantly. `Anims::sync` turns the
//! engine's `Event`s into a schedule: each unit with something to play gets a
//! `Track` (a chain of segments plus hit flashes), and the map draws it at its
//! `Pose` instead of where the game says it is until the track runs out.
//! Everything is cheap and only active while it is playing. Nothing here
//! paints (see `fx.rs` for the effects); times are seconds on `Anims::now`.

use super::hex::{SQRT3, world};
use egui::{Vec2, vec2};
use std::collections::HashMap;
use std::f32::consts::TAU;
use wayciv_core::combat::AttackKind;
use wayciv_core::events::{Built, Event, Striker};
use wayciv_core::state::Unit;
use wayciv_core::{Game, PlayerId, Pos, UnitId};

// ---- Easing -----------------------------------------------------------------

pub fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

/// Smoothstep: slow at both ends.
pub fn smooth(t: f32) -> f32 {
    let t = clamp01(t);
    t * t * (3.0 - 2.0 * t)
}

/// Fast start, gentle stop.
pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - clamp01(t)).powi(3)
}

pub fn ease_in_out(t: f32) -> f32 {
    let t = clamp01(t);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Overshoots 1 a little, then settles (for things that pop in).
pub fn ease_out_back(t: f32) -> f32 {
    let (c1, c3) = (1.4, 2.4);
    let t = clamp01(t) - 1.0;
    1.0 + c3 * t * t * t + c1 * t * t
}

/// Distance travelled at progress `u` when speed ramps up over the first
/// and down over the last `r` of the time and is constant between: a walk
/// that eases into its first step and out of its last.
pub fn trapezoid(u: f32, r: f32) -> f32 {
    let u = clamp01(u);
    let v = 1.0 / (1.0 - r);
    if u < r {
        v * u * u / (2.0 * r)
    } else if u <= 1.0 - r {
        v * (r / 2.0 + (u - r))
    } else {
        1.0 - v * (1.0 - u) * (1.0 - u) / (2.0 * r)
    }
}

/// The world offset from tile `a` to tile `b`, going the short way around
/// the east-west wrap (`width` is the map's width in world units).
pub fn delta(width: f32, a: Pos, b: Pos) -> Vec2 {
    let mut d = world(b) - world(a);
    d.x -= width * (d.x / width).round();
    d
}

// ---- Tunables -----------------------------------------------------------------

/// Seconds a whole batch of events may take before it is sped up.
const BATCH_CAP: f64 = 2.4;
/// But never faster than this fraction of normal speed.
const MIN_SPEED: f64 = 0.4;
const PATH_BASE: f64 = 0.12;
const PATH_PER_STEP: f64 = 0.12;
const PATH_MAX: f64 = 0.9;
const LUNGE: f64 = 0.36;
/// The lunge connects this far in.
const LUNGE_HIT: f64 = 0.15;
const RECOIL: f64 = 0.3;
const HIT: f64 = 0.34;
const DIE: f64 = 0.5;
const WORK: f64 = 0.9;
const APPEAR: f64 = 0.4;
pub const POP: f64 = 0.55;
pub const CLAIM: f64 = 0.9;
pub const REVEAL: f64 = 0.7;
pub const FOG: f64 = 0.4;

// ---- Unit tracks --------------------------------------------------------------

/// One stretch of a unit's animation.
#[derive(Clone, Debug)]
pub enum Move {
    Idle { at: Pos },
    /// Walks the polyline `pts` (world coordinates, ending at tile `to`).
    Path {
        to: Pos,
        pts: Vec<Vec2>,
        fade_in: bool,
        fade_out: bool,
    },
    /// Lunges at a target `toward` (world offset) and back.
    Lunge { at: Pos, toward: Vec2 },
    /// The kick of a ranged shot, away from the target.
    Recoil { at: Pos, away: Vec2 },
    /// A builder working: hops in place.
    Work { at: Pos },
    Die { at: Pos },
    Appear { at: Pos },
}

impl Move {
    /// The tile the unit stands on afterwards.
    fn end_tile(&self) -> Pos {
        match self {
            Move::Path { to, .. } => *to,
            Move::Idle { at }
            | Move::Lunge { at, .. }
            | Move::Recoil { at, .. }
            | Move::Work { at }
            | Move::Die { at }
            | Move::Appear { at } => *at,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Seg {
    pub start: f64,
    pub dur: f64,
    pub what: Move,
}

/// A flash and shake from taking a hit; `dir` points away from the attacker.
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub start: f64,
    pub dir: Vec2,
}

/// Where and how a unit is drawn right now. Offsets are in hex radii.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub tile: Pos,
    pub offset: Vec2,
    /// Height above the ground.
    pub lift: f32,
    pub alpha: f32,
    /// Red hit flash, 0 to 1.
    pub flash: f32,
    pub scale: f32,
    /// Hit points to show instead of the unit's real ones.
    pub hp: Option<i32>,
    /// Tiles walked so far in this stretch (drives the step bob), if walking.
    pub stride: Option<f32>,
}

impl Pose {
    fn at(tile: Pos) -> Self {
        Pose {
            tile,
            offset: Vec2::ZERO,
            lift: 0.0,
            alpha: 1.0,
            flash: 0.0,
            scale: 1.0,
            hp: None,
            stride: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Track {
    pub segs: Vec<Seg>,
    pub hits: Vec<Hit>,
    /// Hit points to show until the given time.
    pub hp: Vec<(f64, i32)>,
}

impl Track {
    /// When the last segment ends.
    pub fn end(&self) -> f64 {
        self.segs.last().map_or(0.0, |s| s.start + s.dur)
    }

    /// The pose at `now`, or none once the track is over.
    pub fn pose(&self, now: f64) -> Option<Pose> {
        let first = self.segs.first()?;
        if now >= self.end() {
            return None;
        }
        let mut pose = if now < first.start {
            seg_pose(first, 0.0)
        } else {
            let i = self
                .segs
                .iter()
                .rposition(|s| s.start <= now)
                .unwrap_or_default();
            let s = &self.segs[i];
            seg_pose(s, ((now - s.start) / s.dur.max(1e-6)) as f32)
        };
        for h in &self.hits {
            let v = ((now - h.start) / HIT) as f32;
            if (0.0..1.0).contains(&v) {
                pose.flash = pose.flash.max(1.0 - v);
                let shake = (v * TAU * 2.5).sin() * (1.0 - v);
                pose.offset += h.dir * (0.12 * (1.0 - v).powi(2)) + vec2(shake * 0.07, 0.0);
            }
        }
        pose.hp = self.hp.iter().find(|(until, _)| now < *until).map(|h| h.1);
        Some(pose)
    }
}

/// The pose a segment gives at progress `u` (0 to 1).
fn seg_pose(seg: &Seg, u: f32) -> Pose {
    let u = clamp01(u);
    match &seg.what {
        Move::Idle { at } => Pose::at(*at),
        Move::Path {
            to,
            pts,
            fade_in,
            fade_out,
        } => {
            let mut pose = Pose::at(*to);
            let total: f32 = pts.windows(2).map(|w| (w[1] - w[0]).length()).sum();
            let mut left = trapezoid(u, 0.14) * total;
            let walked = left;
            let mut at = pts[0];
            for w in pts.windows(2) {
                let len = (w[1] - w[0]).length();
                if left <= len || len <= 0.0 {
                    at = w[0] + (w[1] - w[0]) * if len > 0.0 { left / len } else { 0.0 };
                    break;
                }
                left -= len;
                at = w[1];
            }
            pose.offset = at - pts[pts.len() - 1];
            pose.stride = Some(walked / SQRT3);
            if *fade_in {
                pose.alpha = smooth(u / 0.35);
            }
            if *fade_out {
                pose.alpha = pose.alpha.min(1.0 - smooth((u - 0.6) / 0.4));
            }
            pose
        }
        Move::Lunge { at, toward } => {
            // Out fast to just short of the target, then back.
            let reach = 0.52;
            let k = if u < 0.42 {
                ease_out(u / 0.42)
            } else {
                1.0 - ease_in_out((u - 0.42) / 0.58)
            };
            Pose {
                offset: *toward * (reach * k),
                lift: 0.05 * (u * std::f32::consts::PI).sin(),
                ..Pose::at(*at)
            }
        }
        Move::Recoil { at, away } => {
            let k = if u < 0.25 { u / 0.25 } else { 1.0 - (u - 0.25) / 0.75 };
            let k = smooth(k);
            Pose {
                offset: away.normalized() * (0.09 * k),
                scale: 1.0 + 0.04 * k,
                ..Pose::at(*at)
            }
        }
        Move::Work { at } => {
            // Three hops, a little squashed on the landing.
            let hop = (u * TAU * 1.5).sin().abs();
            Pose {
                lift: 0.09 * hop,
                scale: 1.0 - 0.03 * (1.0 - hop),
                ..Pose::at(*at)
            }
        }
        Move::Die { at } => Pose {
            alpha: 1.0 - smooth(u),
            lift: -0.2 * ease_out(u),
            scale: 1.0 - 0.18 * ease_out(u),
            flash: (1.0 - u * 2.5).max(0.0) * 0.6,
            ..Pose::at(*at)
        },
        Move::Appear { at } => Pose {
            alpha: smooth(u * 2.0),
            scale: 0.55 + 0.45 * ease_out_back(u),
            ..Pose::at(*at)
        },
    }
}

// ---- Effects --------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shot {
    Arrow,
    Stone,
    Shell,
}

#[derive(Clone, Debug)]
pub enum FxKind {
    /// Flies from the tile `at` by `to` (a world offset).
    Projectile { to: Vec2, shot: Shot },
    Impact { big: bool },
    /// A number or word that rises and fades.
    Float { text: String, color: [u8; 3], big: bool },
    /// Dust or a shock ring on the ground; `color` tints it.
    Ring { color: Option<[u8; 3]>, size: f32 },
    /// A small puff of dust at a unit's feet.
    Puff,
    Sparkle { seed: u32 },
}

#[derive(Clone, Debug)]
pub struct Fx {
    pub start: f64,
    pub dur: f64,
    pub at: Pos,
    pub kind: FxKind,
}

impl Fx {
    /// 0 to 1 over the effect's life, or none if it isn't playing.
    pub fn progress(&self, now: f64) -> Option<f32> {
        let u = ((now - self.start) / self.dur) as f32;
        (0.0..1.0).contains(&u).then_some(u)
    }
}

// ---- The camera ---------------------------------------------------------------

struct Pan {
    from: Vec2,
    to: Vec2,
    /// Stamped on the first frame after the request (the clock may be stale).
    start: Option<f64>,
    dur: f64,
}

// ---- The animation state ---------------------------------------------------------

pub struct Anims {
    /// Seconds since the game screen opened (or a fixed step per frame).
    pub now: f64,
    fixed_step: Option<f64>,
    epoch: std::time::Instant,
    tracks: HashMap<UnitId, Track>,
    /// Units that are gone from the game but still dying on screen.
    pub ghosts: HashMap<UnitId, Unit>,
    pub fx: Vec<Fx>,
    pan: Option<Pan>,
    /// Tiles whose district, wonder, improvement or city is popping in,
    /// and when it starts.
    pops: HashMap<Pos, f64>,
    /// Territory changes: tile -> when its new owner's tint began to fade in.
    claimed: HashMap<Pos, f64>,
    revealed_at: HashMap<Pos, f64>,
    /// Fog changes: tile -> (when, now visible).
    fog: HashMap<Pos, (f64, bool)>,
    owners: Vec<Option<PlayerId>>,
    revealed: Vec<bool>,
    visible: Vec<bool>,
    synced: bool,
    /// Seconds the last map frame took to build (spaces ambient repaints).
    pub frame_cost: f32,
}

impl Default for Anims {
    fn default() -> Self {
        Self::new()
    }
}

impl Anims {
    /// `WAYCIV_ANIM_STEP=ms` advances the clock by a fixed step per frame
    /// (for reproducible test screenshots).
    pub fn new() -> Self {
        Self {
            now: 0.0,
            fixed_step: std::env::var("WAYCIV_ANIM_STEP")
                .ok()
                .and_then(|s| s.parse::<f64>().ok())
                .map(|ms| ms / 1000.0),
            epoch: std::time::Instant::now(),
            tracks: HashMap::new(),
            ghosts: HashMap::new(),
            fx: Vec::new(),
            pan: None,
            pops: HashMap::new(),
            claimed: HashMap::new(),
            revealed_at: HashMap::new(),
            fog: HashMap::new(),
            owners: Vec::new(),
            revealed: Vec::new(),
            visible: Vec::new(),
            synced: false,
            frame_cost: 0.0,
        }
    }

    /// Advances the clock; call once per frame before anything else.
    pub fn begin(&mut self) {
        self.now = match self.fixed_step {
            Some(step) => self.now + step,
            None => self.epoch.elapsed().as_secs_f64(),
        };
        let now = self.now;
        self.tracks.retain(|_, t| t.end() > now);
        let tracks = &self.tracks;
        self.ghosts.retain(|id, _| tracks.contains_key(id));
        self.fx.retain(|f| f.start + f.dur > now);
        self.pops.retain(|_, s| *s + POP > now);
        self.claimed.retain(|_, s| *s + CLAIM > now);
        self.revealed_at.retain(|_, s| *s + REVEAL > now);
        self.fog.retain(|_, (s, _)| *s + FOG > now);
    }

    /// Whether anything is playing (the map must keep repainting).
    pub fn active(&self) -> bool {
        !(self.tracks.is_empty()
            && self.fx.is_empty()
            && self.pan.is_none()
            && self.pops.is_empty()
            && self.claimed.is_empty()
            && self.revealed_at.is_empty()
            && self.fog.is_empty())
    }

    pub fn pose(&self, unit: UnitId) -> Option<Pose> {
        self.tracks.get(&unit)?.pose(self.now)
    }

    /// Units with an active track.
    pub fn animated(&self) -> impl Iterator<Item = (UnitId, Pose)> + '_ {
        self.tracks
            .iter()
            .filter_map(|(id, t)| t.pose(self.now).map(|p| (*id, p)))
    }

    // ---- Tile fades

    /// How far a pop-in has got: none when not popping, 0 while waiting.
    pub fn pop(&self, pos: Pos) -> Option<f32> {
        let start = *self.pops.get(&pos)?;
        Some(((self.now - start) / POP).clamp(0.0, 1.0) as f32)
    }

    /// 0 to 1: how much of a freshly claimed tile's tint to show.
    pub fn claim(&self, pos: Pos) -> f32 {
        match self.claimed.get(&pos) {
            Some(start) => smooth(((self.now - start) / CLAIM) as f32),
            None => 1.0,
        }
    }

    /// 0 to 1: how far a freshly revealed tile has come out of the dark.
    pub fn reveal(&self, pos: Pos) -> f32 {
        match self.revealed_at.get(&pos) {
            Some(start) => smooth(((self.now - start) / REVEAL) as f32),
            None => 1.0,
        }
    }

    /// The fog's strength (0 clear, 1 full) given whether the tile is
    /// visible now, easing between the two as sight changes.
    pub fn fog(&self, pos: Pos, visible: bool) -> f32 {
        let target = if visible { 0.0 } else { 1.0 };
        match self.fog.get(&pos) {
            Some((start, _)) => {
                let u = smooth(((self.now - start) / FOG) as f32);
                target + (1.0 - target - target) * (1.0 - u)
            }
            None => target,
        }
    }

    // ---- The camera

    /// Starts easing the camera from `from` to the world point `to` (taking
    /// the short way around the wrap).
    pub fn pan_to(&mut self, from: Vec2, to: Vec2, width: f32) {
        let mut to = to;
        to.x -= width * ((to.x - from.x) / width).round();
        let dist = (to - from).length();
        if dist < 0.05 {
            self.pan = None;
            return;
        }
        self.pan = Some(Pan {
            from,
            to,
            start: None,
            dur: (0.22 + 0.014 * dist as f64).min(0.6),
        });
    }

    /// The camera center for this frame while a pan is playing.
    pub fn pan_center(&mut self) -> Option<Vec2> {
        let now = self.now;
        let pan = self.pan.as_mut()?;
        let start = *pan.start.get_or_insert(now);
        let u = ((now - start) / pan.dur) as f32;
        let at = pan.from + (pan.to - pan.from) * ease_in_out(u);
        if u >= 1.0 {
            self.pan = None;
        }
        Some(at)
    }

    pub fn cancel_pan(&mut self) {
        self.pan = None;
    }

    // ---- Syncing with the game

    /// Takes the engine's events and notes what changed on the map since the
    /// last frame (borders, discovered tiles, fog). The first call only
    /// records the starting state.
    pub fn sync(&mut self, game: &mut Game) {
        let viewer = game.human();
        let events = game.take_events();
        let grid = game.map.grid;
        let player = &game.players[viewer];
        let n = grid.len();
        if !self.synced || self.owners.len() != n {
            self.owners = game.map.tiles.iter().map(|t| t.owner).collect();
            self.revealed = player.revealed.clone();
            self.visible = player.visible.clone();
            self.synced = true;
            return;
        }
        let now = self.now;
        // Territory.
        let changed = game
            .map
            .tiles
            .iter()
            .zip(&self.owners)
            .any(|(t, o)| t.owner != *o);
        if changed {
            for (i, t) in game.map.tiles.iter().enumerate() {
                if t.owner != self.owners[i] {
                    if t.owner.is_some() {
                        self.claimed.insert(game.map.grid.pos(i), now);
                    } else {
                        self.claimed.remove(&game.map.grid.pos(i));
                    }
                    self.owners[i] = t.owner;
                }
            }
        }
        // Discovery and fog.
        if player.revealed != self.revealed {
            for i in 0..n {
                if player.revealed[i] && !self.revealed[i] {
                    self.revealed_at.insert(game.map.grid.pos(i), now);
                }
            }
            self.revealed = player.revealed.clone();
        }
        if player.visible != self.visible {
            for i in 0..n {
                if player.visible[i] != self.visible[i] && self.revealed[i] {
                    self.fog.insert(game.map.grid.pos(i), (now, player.visible[i]));
                }
            }
            self.visible = player.visible.clone();
        }
        if !events.is_empty() {
            self.ingest(game, viewer, events);
        }
    }

    /// Schedules `events` (in order) starting now.
    pub fn ingest(&mut self, game: &Game, viewer: PlayerId, events: Vec<Event>) {
        let now = self.now;
        let mut b = Batch::new(game, viewer);
        for (id, t) in &self.tracks {
            b.free.insert(*id, (t.end() - now).max(0.0));
            if let Some(s) = t.segs.last() {
                b.rest.insert(*id, s.what.end_tile());
            }
        }
        for e in events {
            b.event(e);
        }
        b.finish();
        // Speed the batch up if it would take too long.
        let k = (BATCH_CAP / b.total.max(1e-6)).clamp(MIN_SPEED, 1.0);
        for (id, segs) in b.segs {
            let track = self.tracks.entry(id).or_default();
            for mut s in segs {
                s.start = now + s.start * k;
                s.dur *= k;
                track.segs.push(s);
            }
            for mut h in b.hits.remove(&id).unwrap_or_default() {
                h.start = now + h.start * k;
                track.hits.push(h);
            }
            for (until, hp) in b.hp.remove(&id).unwrap_or_default() {
                track.hp.push((now + until * k, hp));
            }
        }
        for u in b.ghosts {
            self.ghosts.insert(u.id, u);
        }
        for mut f in b.fx {
            f.start = now + f.start * k;
            if matches!(f.kind, FxKind::Projectile { .. }) {
                f.dur *= k;
            }
            self.fx.push(f);
        }
        for (pos, start) in b.pops {
            self.pops.insert(pos, now + start * k);
        }
    }
}

// ---- Scheduling ---------------------------------------------------------------------

/// Events being turned into a schedule, in batch time (seconds from the
/// first frame, before any speed-up).
struct Batch<'a> {
    game: &'a Game,
    viewer: PlayerId,
    width: f32,
    segs: HashMap<UnitId, Vec<Seg>>,
    /// When each unit is next free, and where it stands then.
    free: HashMap<UnitId, f64>,
    rest: HashMap<UnitId, Pos>,
    dead: Vec<UnitId>,
    hits: HashMap<UnitId, Vec<Hit>>,
    hp: HashMap<UnitId, Vec<(f64, i32)>>,
    /// When the last hit on a unit's flash is over.
    hit_end: HashMap<UnitId, f64>,
    ghosts: Vec<Unit>,
    fx: Vec<Fx>,
    pops: Vec<(Pos, f64)>,
    /// When a unit last arrived on, or a strike last landed on, a tile.
    arrive: HashMap<Pos, f64>,
    strike: HashMap<Pos, f64>,
    total: f64,
}

impl<'a> Batch<'a> {
    fn new(game: &'a Game, viewer: PlayerId) -> Self {
        Batch {
            game,
            viewer,
            width: SQRT3 * game.map.grid.width as f32,
            segs: HashMap::new(),
            free: HashMap::new(),
            rest: HashMap::new(),
            dead: Vec::new(),
            hits: HashMap::new(),
            hp: HashMap::new(),
            hit_end: HashMap::new(),
            ghosts: Vec::new(),
            fx: Vec::new(),
            pops: Vec::new(),
            arrive: HashMap::new(),
            strike: HashMap::new(),
            total: 0.0,
        }
    }

    fn visible(&self, pos: Pos) -> bool {
        self.game.players[self.viewer].visible[self.game.map.grid.index(pos)]
    }

    fn free(&self, unit: UnitId) -> f64 {
        self.free.get(&unit).copied().unwrap_or(0.0)
    }

    /// Appends a segment for `unit`, which must be free by `start`
    /// (it waits in place until then). Returns when the segment ends.
    fn push(&mut self, unit: UnitId, at: Pos, start: f64, dur: f64, what: Move) -> f64 {
        if self.dead.contains(&unit) {
            return start;
        }
        let rest = self.rest.get(&unit).copied().unwrap_or(at);
        let free = self.free(unit);
        let segs = self.segs.entry(unit).or_default();
        if start > free + 1e-9 {
            segs.push(Seg {
                start: free,
                dur: start - free,
                what: Move::Idle { at: rest },
            });
        }
        let start = start.max(free);
        self.rest.insert(unit, what.end_tile());
        segs.push(Seg { start, dur, what });
        self.free.insert(unit, start + dur);
        self.total = self.total.max(start + dur);
        start + dur
    }

    fn fx(&mut self, start: f64, dur: f64, at: Pos, kind: FxKind) {
        self.fx.push(Fx {
            start,
            dur,
            at,
            kind,
        });
        self.total = self.total.max(start.min(BATCH_CAP));
    }

    fn event(&mut self, e: Event) {
        let game = self.game;
        match e {
            Event::Moved { unit, from, to } => {
                let (vis_from, vis_to) = (self.visible(from), self.visible(to));
                if !(vis_from || vis_to) || self.dead.contains(&unit) {
                    return;
                }
                let dw = delta(self.width, from, to);
                // Extend a walk that is still being queued.
                if let Some(Seg {
                    start,
                    dur,
                    what: Move::Path { to: end, pts, fade_out, .. },
                }) = self.segs.get_mut(&unit).and_then(|s| s.last_mut())
                    && *end == from
                {
                    let last = *pts.last().unwrap();
                    pts.push(last + dw);
                    *end = to;
                    *fade_out = !vis_to;
                    *dur = path_dur(pts.len() - 1);
                    let finish = *start + *dur;
                    self.free.insert(unit, finish);
                    self.rest.insert(unit, to);
                    self.arrive.insert(to, finish);
                    self.total = self.total.max(finish);
                    return;
                }
                self.rest.insert(unit, from);
                let begin = world(from);
                let path = Move::Path {
                    to,
                    pts: vec![begin, begin + dw],
                    fade_in: !vis_from,
                    fade_out: !vis_to,
                };
                let start = self.free(unit);
                let end = self.push(unit, from, start, path_dur(1), path);
                self.arrive.insert(to, end);
            }
            Event::Attack {
                striker,
                target,
                kind,
                defender,
                dealt,
                taken,
            } => {
                let (from, attacker) = match &striker {
                    Striker::Unit(u) => (u.pos, Some(u.as_ref())),
                    Striker::City(_, pos) => (*pos, None),
                };
                if !(self.visible(from) || self.visible(target)) {
                    return;
                }
                let toward = delta(self.width, from, target);
                let dist = toward.length() / SQRT3;
                let mut start = 0.0f64;
                if let Some(a) = attacker {
                    start = start.max(self.free(a.id));
                    self.rest.entry(a.id).or_insert(from);
                }
                if let Some(d) = &defender {
                    start = start.max(self.free(d.id));
                    self.rest.entry(d.id).or_insert(d.pos);
                }
                // Anyone who walked up to this tile finishes first.
                if let Some(t) = self.arrive.get(&target) {
                    start = start.max(*t);
                }
                let melee = kind == AttackKind::Melee;
                let hit_at;
                if melee {
                    hit_at = start + LUNGE_HIT;
                    if let Some(a) = attacker {
                        self.push(a.id, from, start, LUNGE, Move::Lunge { at: from, toward });
                    }
                } else {
                    let flight = 0.2 + 0.07 * dist as f64;
                    let shot_at = start + 0.12;
                    hit_at = shot_at + flight;
                    if let Some(a) = attacker {
                        self.push(
                            a.id,
                            from,
                            start,
                            RECOIL,
                            Move::Recoil {
                                at: from,
                                away: -toward,
                            },
                        );
                    }
                    let shot = shot_for(game, &striker, kind);
                    self.fx(
                        shot_at,
                        flight,
                        from,
                        FxKind::Projectile { to: toward, shot },
                    );
                }
                self.strike.insert(target, hit_at);
                self.fx(
                    hit_at,
                    0.35,
                    target,
                    FxKind::Impact {
                        big: kind == AttackKind::Bombard,
                    },
                );
                let viewer = self.viewer;
                let mine = move |owner: PlayerId| owner == viewer;
                // The defender flashes and shows its health from before.
                let mut victim_mine = false;
                if let Some(d) = &defender {
                    victim_mine = mine(d.owner);
                    self.push(d.id, d.pos, start, hit_at - start, Move::Idle { at: d.pos });
                    self.hits.entry(d.id).or_default().push(Hit {
                        start: hit_at,
                        dir: toward.normalized(),
                    });
                    self.hp.entry(d.id).or_default().push((hit_at, d.hp));
                    self.hit_end.insert(d.id, hit_at + HIT);
                } else if let Some(city) = game.city_at(target) {
                    victim_mine = mine(city.owner);
                }
                let big = dealt >= 30;
                self.fx(
                    hit_at + 0.04,
                    1.0,
                    target,
                    FxKind::Float {
                        text: if dealt >= 100 && defender.as_ref().is_some_and(|d| !game.is_military(d)) {
                            "Slain".into()
                        } else {
                            format!("-{dealt}")
                        },
                        color: if victim_mine {
                            [255, 96, 84]
                        } else {
                            [255, 220, 120]
                        },
                        big,
                    },
                );
                // A melee attacker takes the counter-blow.
                if let (true, Some(a)) = (melee && taken > 0, attacker) {
                    let at = hit_at + 0.1;
                    self.hits.entry(a.id).or_default().push(Hit {
                        start: at,
                        dir: -toward.normalized(),
                    });
                    self.hp.entry(a.id).or_default().push((at, a.hp));
                    self.hit_end.insert(a.id, at + HIT);
                    self.fx(
                        at + 0.04,
                        1.0,
                        from,
                        FxKind::Float {
                            text: format!("-{taken}"),
                            color: if mine(a.owner) {
                                [255, 96, 84]
                            } else {
                                [255, 220, 120]
                            },
                            big: false,
                        },
                    );
                }
            }
            Event::Died { unit } => {
                if !self.visible(unit.pos) {
                    return;
                }
                let id = unit.id;
                let mut start = self.free(id);
                if let Some(t) = self.strike.get(&unit.pos) {
                    start = start.max(*t + 0.12);
                } else if let Some(t) = self.arrive.get(&unit.pos) {
                    start = start.max(*t);
                }
                if let Some(t) = self.hit_end.get(&id) {
                    start = start.max(*t - HIT + 0.2);
                }
                self.rest.entry(id).or_insert(unit.pos);
                self.push(id, unit.pos, start, DIE, Move::Die { at: unit.pos });
                self.dead.push(id);
                let mut ghost = (*unit).clone();
                ghost.hp = ghost.hp.max(0);
                self.ghosts.push(ghost);
            }
            Event::Spawned { unit, pos } => {
                if self.visible(pos) {
                    self.rest.insert(unit, pos);
                    let at = self.free(unit);
                    self.push(unit, pos, at, APPEAR, Move::Appear { at: pos });
                }
            }
            Event::CityFounded { pos, .. } => {
                if self.visible(pos) {
                    self.pops.push((pos, 0.0));
                    self.fx(0.12, 0.9, pos, FxKind::Ring { color: None, size: 1.5 });
                    self.fx(0.12, 0.7, pos, FxKind::Puff);
                    self.fx(0.35, 0.9, pos, FxKind::Sparkle { seed: pos.x as u32 * 7 + pos.y as u32 });
                    self.total = self.total.max(POP);
                }
            }
            Event::CityCaptured { owner, pos, .. } => {
                if self.visible(pos) {
                    let at = self.arrive.get(&pos).copied().unwrap_or(0.0);
                    let color = self.game.players[owner].primary;
                    self.fx(at, 0.9, pos, FxKind::Ring { color: Some(color), size: 1.9 });
                    self.fx(at + 0.1, 0.9, pos, FxKind::Sparkle { seed: pos.x as u32 * 5 + pos.y as u32 });
                }
            }
            Event::Built { pos, what } => {
                if self.visible(pos) {
                    if matches!(what, Built::District | Built::Wonder) {
                        self.pops.push((pos, 0.0));
                        self.fx(0.1, 0.7, pos, FxKind::Puff);
                    }
                    self.fx(
                        0.3,
                        1.0,
                        pos,
                        FxKind::Sparkle {
                            seed: pos.x as u32 * 13 + pos.y as u32 * 3,
                        },
                    );
                }
            }
            Event::Improved { unit, pos } => {
                if self.visible(pos) {
                    let id = unit.id;
                    let start = self.free(id);
                    self.rest.entry(id).or_insert(pos);
                    let end = self.push(id, pos, start, WORK, Move::Work { at: pos });
                    for k in 0..3 {
                        self.fx(start + WORK * (0.33 * (k as f64 + 1.0)), 0.5, pos, FxKind::Puff);
                    }
                    // The new improvement waits for the builder, then pops in.
                    self.pops.push((pos, end));
                    self.fx(end, 1.0, pos, FxKind::Sparkle { seed: pos.x as u32 * 11 + pos.y as u32 });
                    if !game.units.contains_key(&id) {
                        self.ghosts.push((*unit).clone());
                        self.push(id, pos, end, 0.35, Move::Die { at: pos });
                        self.dead.push(id);
                    }
                }
            }
        }
    }

    /// Makes every unit's track outlast its hit flashes.
    fn finish(&mut self) {
        let ends: Vec<(UnitId, f64)> = self.hit_end.iter().map(|(u, t)| (*u, *t)).collect();
        for (id, end) in ends {
            let free = self.free(id);
            if free < end - 1e-9
                && !self.dead.contains(&id)
                && let Some(&at) = self.rest.get(&id)
            {
                self.segs.entry(id).or_default().push(Seg {
                    start: free,
                    dur: end - free,
                    what: Move::Idle { at },
                });
                self.free.insert(id, end);
            }
        }
        self.total = self.free.values().copied().fold(self.total, f64::max);
    }
}

/// How long a walk of `steps` tiles takes.
fn path_dur(steps: usize) -> f64 {
    (PATH_BASE + PATH_PER_STEP * steps as f64).min(PATH_MAX)
}

/// What a striker fires.
fn shot_for(game: &Game, striker: &Striker, kind: AttackKind) -> Shot {
    use super::units::{Class, class_of};
    match striker {
        Striker::City(..) => Shot::Arrow,
        Striker::Unit(u) => {
            let ty = &game.rules.content.units[u.kind];
            match (class_of(ty), kind) {
                (Class::Naval | Class::Air, _) => Shot::Shell,
                (_, AttackKind::Bombard) => {
                    let id = ty.id.as_str();
                    if ["CANNON", "ARTILLERY", "ROCKET", "HOWITZER", "BOMBARD"]
                        .iter()
                        .any(|k| id.contains(k))
                    {
                        Shot::Shell
                    } else {
                        Shot::Stone
                    }
                }
                _ => Shot::Arrow,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_stays_in_range_and_hits_the_ends() {
        for f in [smooth, ease_out, ease_in_out] {
            assert_eq!(f(0.0), 0.0);
            assert!((f(1.0) - 1.0).abs() < 1e-6);
            let mut last = 0.0;
            for i in 0..=100 {
                let v = f(i as f32 / 100.0);
                assert!(v >= last - 1e-6, "monotonic");
                last = v;
            }
        }
        assert!(ease_out_back(0.7) > 1.0, "overshoots");
        assert!((ease_out_back(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn trapezoid_covers_the_distance() {
        assert_eq!(trapezoid(0.0, 0.14), 0.0);
        assert!((trapezoid(1.0, 0.14) - 1.0).abs() < 1e-6);
        let mut last = 0.0;
        for i in 0..=200 {
            let v = trapezoid(i as f32 / 200.0, 0.14);
            assert!(v >= last);
            last = v;
        }
        // Continuous at the joins.
        let r = 0.14;
        assert!((trapezoid(r - 1e-4, r) - trapezoid(r + 1e-4, r)).abs() < 1e-3);
    }

    #[test]
    fn deltas_wrap_east_west() {
        let width = SQRT3 * 20.0;
        let d = delta(width, Pos::new(19, 4), Pos::new(0, 4));
        assert!((d.x - SQRT3).abs() < 1e-4, "{d:?}");
        let d = delta(width, Pos::new(3, 4), Pos::new(4, 4));
        assert!((d.x - SQRT3).abs() < 1e-4);
    }

    fn walk(from: f32, to: f32) -> Track {
        Track {
            segs: vec![Seg {
                start: 1.0,
                dur: 0.5,
                what: Move::Path {
                    to: Pos::new(5, 5),
                    pts: vec![vec2(from, 0.0), vec2(to, 0.0)],
                    fade_in: false,
                    fade_out: false,
                },
            }],
            ..Default::default()
        }
    }

    #[test]
    fn a_walk_arrives_and_then_ends() {
        let t = walk(0.0, 1.7);
        // Before it starts the unit waits at its first point.
        let p = t.pose(0.5).unwrap();
        assert!((p.offset.x + 1.7).abs() < 1e-5);
        // Halfway it is halfway.
        let p = t.pose(1.25).unwrap();
        assert!((p.offset.x + 0.85).abs() < 0.05, "{p:?}");
        // Near the end it is almost home, and then the track is over.
        assert!(t.pose(1.499).unwrap().offset.x.abs() < 0.01);
        assert!(t.pose(1.5).is_none());
    }

    #[test]
    fn a_hit_flashes_then_fades() {
        let mut t = walk(0.0, 0.0);
        t.hits.push(Hit {
            start: 1.1,
            dir: vec2(1.0, 0.0),
        });
        assert_eq!(t.pose(1.05).unwrap().flash, 0.0);
        assert!(t.pose(1.11).unwrap().flash > 0.9);
        assert_eq!(t.pose(1.1 + HIT + 0.01).unwrap().flash, 0.0);
    }

    #[test]
    fn health_shown_until_the_hit_lands() {
        let mut t = walk(0.0, 0.0);
        t.hp.push((1.2, 80));
        assert_eq!(t.pose(1.1).unwrap().hp, Some(80));
        assert_eq!(t.pose(1.3).unwrap().hp, None);
    }

    #[test]
    fn dying_fades_out() {
        let seg = Seg {
            start: 0.0,
            dur: 1.0,
            what: Move::Die { at: Pos::new(1, 1) },
        };
        assert!(seg_pose(&seg, 0.0).alpha > 0.99);
        assert!(seg_pose(&seg, 1.0).alpha < 0.01);
        assert!(seg_pose(&seg, 1.0).lift < 0.0, "sinks");
    }

    #[test]
    fn pans_ease_and_take_the_short_way() {
        let mut a = Anims::new();
        let width = 100.0;
        a.pan_to(vec2(98.0, 5.0), vec2(2.0, 5.0), width);
        assert_eq!(a.pan.as_ref().unwrap().to.x, 102.0);
        a.now = 10.0;
        assert_eq!(a.pan_center().unwrap(), vec2(98.0, 5.0));
        a.now = 10.0 + a.pan.as_ref().unwrap().dur / 2.0;
        assert!((a.pan_center().unwrap().x - 100.0).abs() < 1e-3);
        a.now = 20.0;
        assert_eq!(a.pan_center().unwrap(), vec2(102.0, 5.0));
        assert!(a.pan.is_none());
        // A tiny move needs no pan.
        a.pan_to(vec2(1.0, 1.0), vec2(1.01, 1.0), width);
        assert!(a.pan.is_none());
    }

    /// A real game, to check the schedule the events produce.
    fn game() -> Option<Game> {
        let rules = wayciv_core::load_rules().ok()?;
        Some(wayciv_core::new_game(
            rules,
            wayciv_core::Settings {
                seed: 3,
                ..Default::default()
            },
        ))
    }

    #[test]
    fn steps_merge_into_one_walk_and_a_fight_lunges() {
        let Some(mut game) = game() else {
            eprintln!("skipped: content failed to load");
            return;
        };
        let me = game.human();
        for v in game.players[me].visible.iter_mut() {
            *v = true;
        }
        let a = Pos::new(10, 10);
        let b = Pos::new(11, 10);
        let c = Pos::new(12, 10);
        let mut anims = Anims::new();
        anims.now = 5.0;
        anims.ingest(
            &game,
            me,
            vec![
                Event::Moved { unit: 7, from: a, to: b },
                Event::Moved { unit: 7, from: b, to: c },
            ],
        );
        let t = &anims.tracks[&7];
        assert_eq!(t.segs.len(), 1, "two steps are one walk");
        let Move::Path { pts, .. } = &t.segs[0].what else {
            panic!("a walk")
        };
        assert_eq!(pts.len(), 3);
        assert!((t.segs[0].dur - path_dur(2)).abs() < 1e-9);
        assert_eq!(t.segs[0].start, 5.0);
        assert!(anims.active());
        // Later it is over and the unit is drawn normally again.
        anims.now = 5.0 + path_dur(2) + 0.01;
        anims.begin_for_test();
        assert!(!anims.active());
        assert!(anims.pose(7).is_none());
    }

    #[test]
    fn a_long_walk_is_capped() {
        let Some(mut game) = game() else {
            eprintln!("skipped: content failed to load");
            return;
        };
        let me = game.human();
        for v in game.players[me].visible.iter_mut() {
            *v = true;
        }
        let mut anims = Anims::new();
        // Forty steps by one unit are still one capped walk.
        let events: Vec<Event> = (0..40)
            .map(|i| Event::Moved {
                unit: 1,
                from: Pos::new(i % 2 + 10, 10),
                to: Pos::new((i + 1) % 2 + 10, 10),
            })
            .collect();
        anims.ingest(&game, me, events);
        let end = anims.tracks[&1].end();
        assert!(end <= BATCH_CAP + 1.0, "{end}");
    }
}

impl Anims {
    /// Tests: prune like `begin` without touching the clock.
    #[cfg(test)]
    fn begin_for_test(&mut self) {
        let now = self.now;
        self.tracks.retain(|_, t| t.end() > now);
        self.fx.retain(|f| f.start + f.dur > now);
    }
}

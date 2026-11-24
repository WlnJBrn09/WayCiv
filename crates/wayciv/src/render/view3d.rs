//! Real-time 3D mesh renderer for the 533 Blender assets in `assets/models3d.pack`.
//! Renders 3D terrain hexes, bedrock skirts, hills, mountains, river & shoreline
//! connection overlays, 3D features, natural wonders, resources, improvements,
//! standalone buildings, districts, constructed wonders, cities, and team-tinted
//! 3D units with directional sun + hemisphere lighting and depth sorting.

use crate::render::TileCtx;
use crate::render::units::UnitFx;
use egui::{Color32, Mesh, Painter, Pos2, Shape, pos2, vec2};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use wayciv_core::state::{City, Unit};
use wayciv_core::{Game, Grid, Pos};

#[derive(Clone, Copy)]
pub struct Tri3D {
    pub v0: u16,
    pub v1: u16,
    pub v2: u16,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub flags: u8,
}

pub struct Model3D {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<Tri3D>,
}

#[derive(Default)]
pub struct Pack3D {
    loaded: bool,
    models: Vec<Model3D>,
    lookup: HashMap<String, usize>,
}

struct DrawTri {
    depth: f32,
    p0: Pos2,
    p1: Pos2,
    p2: Pos2,
    col: Color32,
}

impl Pack3D {
    pub fn ensure_loaded(&mut self, assets_dir: &Path) -> bool {
        if self.loaded {
            return !self.models.is_empty();
        }
        self.loaded = true;
        let path = assets_dir.join("models3d.pack");
        let Ok(bytes) = fs::read(&path) else {
            return false;
        };
        if bytes.len() < 12 || &bytes[0..4] != b"WC3D" {
            return false;
        }
        let mut cur = 4usize;
        let read_u16 = |b: &[u8], c: &mut usize| -> u16 {
            let v = u16::from_le_bytes([b[*c], b[*c + 1]]);
            *c += 2;
            v
        };
        let read_u32 = |b: &[u8], c: &mut usize| -> u32 {
            let v = u32::from_le_bytes([b[*c], b[*c + 1], b[*c + 2], b[*c + 3]]);
            *c += 4;
            v
        };
        let read_f32 = |b: &[u8], c: &mut usize| -> f32 {
            let v = f32::from_le_bytes([b[*c], b[*c + 1], b[*c + 2], b[*c + 3]]);
            *c += 4;
            v
        };
        let num_meshes = read_u32(&bytes, &mut cur) as usize;
        let num_keys = read_u32(&bytes, &mut cur) as usize;
        self.models.reserve(num_meshes);
        for _ in 0..num_meshes {
            if cur + 8 > bytes.len() {
                break;
            }
            let nv = read_u32(&bytes, &mut cur) as usize;
            let nt = read_u32(&bytes, &mut cur) as usize;
            let mut verts = Vec::with_capacity(nv);
            for _ in 0..nv {
                verts.push([
                    read_f32(&bytes, &mut cur),
                    read_f32(&bytes, &mut cur),
                    read_f32(&bytes, &mut cur),
                ]);
            }
            let mut tris = Vec::with_capacity(nt);
            for _ in 0..nt {
                let v0 = read_u16(&bytes, &mut cur);
                let v1 = read_u16(&bytes, &mut cur);
                let v2 = read_u16(&bytes, &mut cur);
                let r = bytes[cur];
                let g = bytes[cur + 1];
                let b = bytes[cur + 2];
                let flags = bytes[cur + 3];
                cur += 4;
                tris.push(Tri3D { v0, v1, v2, r, g, b, flags });
            }
            self.models.push(Model3D { verts, tris });
        }
        for _ in 0..num_keys {
            if cur + 6 > bytes.len() {
                break;
            }
            let klen = read_u16(&bytes, &mut cur) as usize;
            let midx = read_u32(&bytes, &mut cur) as usize;
            if cur + klen > bytes.len() {
                break;
            }
            if let Ok(s) = std::str::from_utf8(&bytes[cur..cur + klen]) {
                self.lookup.insert(s.to_string(), midx);
            }
            cur += klen;
        }
        !self.models.is_empty()
    }

    pub fn get(&self, key: &str) -> Option<&Model3D> {
        self.lookup.get(key).and_then(|&i| self.models.get(i))
    }

    pub fn get_variant(&self, id: &str, variant: usize) -> Option<&Model3D> {
        let k = format!("{id}#{}", variant % 3);
        self.get(&k).or_else(|| self.get(id))
    }
}

fn rotate_mask(mask: u8, k: usize) -> u8 {
    let m = mask & 63;
    if k == 0 {
        m
    } else {
        ((m << k) | (m >> (6 - k))) & 63
    }
}

fn canonical_mask(mask: u8) -> (u8, usize) {
    let mut best = 64u8;
    let mut steps = 0usize;
    for k in 0..6 {
        let r = rotate_mask(mask, k);
        if r < best {
            best = r;
            steps = (6 - k) % 6;
        }
    }
    (best, steps)
}

fn ccw_neighbor(grid: Grid, p: Pos, edge: usize) -> Option<Pos> {
    let odd = p.y & 1;
    let (nx, ny) = match edge {
        0 => (p.x + 1, p.y),
        1 => (p.x + odd, p.y - 1),
        2 => (p.x + odd - 1, p.y - 1),
        3 => (p.x - 1, p.y),
        4 => (p.x + odd - 1, p.y + 1),
        _ => (p.x + odd, p.y + 1),
    };
    grid.normalize(Pos::new(nx, ny))
}

#[allow(clippy::too_many_arguments)]
fn append_model(
    out: &mut Vec<DrawTri>,
    model: &Model3D,
    center: Pos2,
    scale: f32,
    yaw_rad: f32,
    lift: f32,
    z_scale: f32,
    team_rgb: [u8; 3],
    dim: f32,
    layer_bias: f32,
) {
    let (sy, cy) = yaw_rad.sin_cos();
    let elev_y = 0.82 * z_scale;
    // Sun from NW-upper-left, matching the Cycles preview lighting.
    let sun = [-0.46f32, 0.50, 0.73];
    for t in &model.tris {
        let a = model.verts[t.v0 as usize];
        let b = model.verts[t.v1 as usize];
        let c = model.verts[t.v2 as usize];

        let tx = |v: [f32; 3]| -> [f32; 3] {
            [
                v[0] * cy - v[1] * sy,
                v[0] * sy + v[1] * cy,
                v[2] * z_scale + lift,
            ]
        };
        let p0 = tx(a);
        let p1 = tx(b);
        let p2 = tx(c);

        let ux = p1[0] - p0[0];
        let uy = p1[1] - p0[1];
        let uz = p1[2] - p0[2];
        let vx = p2[0] - p0[0];
        let vy = p2[1] - p0[1];
        let vz = p2[2] - p0[2];
        let mut nx = uy * vz - uz * vy;
        let mut ny = uz * vx - ux * vz;
        let mut nz = ux * vy - uy * vx;
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len < 1e-8 {
            continue;
        }
        nx /= len;
        ny /= len;
        nz /= len;
        let avg_y = (p0[1] + p1[1] + p2[1]) * (1.0 / 3.0);
        let avg_z = (p0[2] + p1[2] + p2[2]) * (1.0 / 3.0);
        // Cull downward bottom caps and north-facing back skirt faces on hex tiles.
        if avg_z < -0.005 && (nz < -0.8 || ny > 0.05) {
            continue;
        }
        // Ensure upward/viewer-facing normal for thin two-sided leaves/sails.
        if avg_z >= 0.0 && nz < -0.35 && -ny + nz < 0.0 {
            nx = -nx;
            ny = -ny;
            nz = -nz;
        }

        let ndotl = (nx * sun[0] + ny * sun[1] + nz * sun[2]).max(0.0);
        let hemi = 0.60 + 0.14 * nz.clamp(-1.0, 1.0);
        let shade = (hemi + 0.28 * ndotl).clamp(0.45, 1.04) * dim;

        let (br, bg, bb) = if t.flags & 1 != 0 {
            (team_rgb[0] as f32, team_rgb[1] as f32, team_rgb[2] as f32)
        } else {
            (t.r as f32, t.g as f32, t.b as f32)
        };
        let col = Color32::from_rgb(
            (br * shade).clamp(0.0, 255.0) as u8,
            (bg * shade).clamp(0.0, 255.0) as u8,
            (bb * shade).clamp(0.0, 255.0) as u8,
        );

        let proj = |p: [f32; 3]| -> Pos2 {
            pos2(center.x + p[0] * scale, center.y - p[1] * scale - p[2] * scale * elev_y)
        };
        let skirt_penalty = if avg_z < -0.005 { -2.0 } else { 0.0 };
        let depth = layer_bias + skirt_penalty - avg_y * 0.65 + avg_z * 0.85;
        out.push(DrawTri {
            depth,
            p0: proj(p0),
            p1: proj(p1),
            p2: proj(p2),
            col,
        });
    }
}

fn flush_tris(painter: &Painter, mut tris: Vec<DrawTri>) {
    if tris.is_empty() {
        return;
    }
    tris.sort_unstable_by(|a, b| a.depth.total_cmp(&b.depth));
    let mut mesh = Mesh::default();
    mesh.vertices.reserve(tris.len() * 3);
    mesh.indices.reserve(tris.len() * 3);
    for (i, t) in tris.into_iter().enumerate() {
        let base = (i * 3) as u32;
        mesh.colored_vertex(t.p0, t.col);
        mesh.colored_vertex(t.p1, t.col);
        mesh.colored_vertex(t.p2, t.col);
        mesh.add_triangle(base, base + 1, base + 2);
    }
    painter.add(Shape::mesh(mesh));
}

pub fn draw_terrain_pass(
    pack: &mut Pack3D,
    assets_dir: &Path,
    painter: &Painter,
    game: &Game,
    viewer: usize,
    tiles: &[(Pos, Pos2)],
    z: f32,
) -> bool {
    if !pack.ensure_loaded(assets_dir) {
        return false;
    }
    let content = &game.rules.content;
    let grid = game.map.grid;
    let vis = &game.players[viewer].visible;
    let mut tris = Vec::with_capacity(tiles.len() * 180);
    // Sort tiles north-to-south so southern tiles naturally overlap northern ones.
    let mut ordered: Vec<(Pos, Pos2)> = tiles.to_vec();
    ordered.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));

    for (rank, &(pos, c)) in ordered.iter().enumerate() {
        let tile = game.map.tile(pos);
        let seen = vis[grid.index(pos)];
        let dim = if seen { 1.0 } else { 0.58 };
        let variant = ((pos.x as usize).wrapping_mul(17) + (pos.y as usize).wrapping_mul(31)) % 3;
        let terrain_id = &content.terrains[tile.terrain].id;
        let is_water = game.map.is_water(&game.rules, pos);
        let hills = content.terrains[tile.terrain].hills;
        let mountain = content.terrains[tile.terrain].mountain;
        let row_bias = rank as f32 * 4.0;

        if let Some(m) = pack.get_variant(terrain_id, variant) {
            append_model(&mut tris, m, c, z, 0.0, 0.0, 1.0, [255, 255, 255], dim, row_bias);
        }
        if !is_water && !hills && !mountain {
            let mut shore_mask = 0u8;
            let mut river_mask = 0u8;
            for e in 0..6 {
                if let Some(np) = ccw_neighbor(grid, pos, e) {
                    if game.map.is_water(&game.rules, np) {
                        shore_mask |= 1 << e;
                    }
                    if tile.river {
                        let nt = game.map.tile(np);
                        let connected = match tile.river_dir {
                            Some(d) => {
                                (d as usize) == e
                                    || (nt.river && nt.river_dir == Some(((e + 3) % 6) as u8))
                            }
                            None => nt.river || game.map.is_water(&game.rules, np),
                        };
                        if connected {
                            river_mask |= 1 << e;
                        }
                    }
                }
            }
            if tile.river && river_mask == 0 {
                river_mask = 1;
            }
            if shore_mask > 0 {
                let (can, steps) = canonical_mask(shore_mask);
                if let Some(sm) = pack.get(&format!("SHORE_{can:02}")) {
                    let yaw = steps as f32 * std::f32::consts::FRAC_PI_3;
                    append_model(&mut tris, sm, c, z, yaw, 0.004, 1.0, [255, 255, 255], dim, row_bias + 1.5);
                }
            }
            let has_floodplains = tile
                .feature
                .is_some_and(|f| content.features[f].id.starts_with("FEATURE_FLOODPLAINS"));
            if river_mask > 0 && !has_floodplains {
                let (can, steps) = canonical_mask(river_mask);
                if let Some(rm) = pack.get(&format!("RIVER_{can:02}")) {
                    let yaw = steps as f32 * std::f32::consts::FRAC_PI_3;
                    append_model(&mut tris, rm, c, z, yaw, 0.006, 1.0, [255, 255, 255], dim, row_bias + 2.0);
                }
            }
        }
    }
    flush_tris(painter, tris);
    true
}

pub fn draw_tile_props(
    pack: &Pack3D,
    painter: &Painter,
    t: &TileCtx,
    pop_scale: f32,
) {
    let game = t.game;
    let content = &game.rules.content;
    let tile = t.tile();
    let seen = game.players[t.viewer].visible[game.map.grid.index(t.pos)];
    let dim = if seen { 1.0 } else { 0.58 };
    let hills = content.terrains[tile.terrain].hills;
    let surf = if hills { 0.16 } else { 0.0 };
    let variant = ((t.pos.x as usize).wrapping_mul(17) + (t.pos.y as usize).wrapping_mul(31)) % 3;
    let z = t.z * pop_scale;
    let owner_rgb = tile
        .owner
        .map(|o| game.players[o].primary)
        .unwrap_or([220, 190, 110]);
    let mut tris = Vec::new();

    if let Some(w) = tile.wonder {
        let wid = &content.buildings[w].id;
        if let Some(m) = pack.get(wid) {
            append_model(&mut tris, m, t.c, z, 0.0, surf, 1.0, owner_rgb, dim, 1.0);
        }
    } else if let Some(d) = tile.district {
        let did = &content.districts[d].id;
        if did != "DISTRICT_CITY_CENTER"
            && let Some(m) = pack.get(did)
        {
            append_model(&mut tris, m, t.c, z, 0.0, surf, 1.0, owner_rgb, dim, 1.0);
        }
    } else {
        if let Some(f) = tile.feature {
            let fid = &content.features[f].id;
            if let Some(m) = pack.get_variant(fid, variant) {
                append_model(&mut tris, m, t.c, z, 0.0, surf, 1.0, owner_rgb, dim, 0.5);
            }
        }
        if let Some(imp) = tile.improvement {
            let iid = &content.improvements[imp].id;
            if let Some(m) = pack.get_variant(iid, variant) {
                append_model(&mut tris, m, t.c, z, 0.0, surf + 0.02, 1.0, owner_rgb, dim, 0.8);
            }
        } else if let Some(res) = tile.resource
            && game.players[t.viewer].meets(&game.rules, content.resources[res].prereq_tech.as_deref())
        {
            let rid = &content.resources[res].id;
            if let Some(m) = pack.get(rid) {
                append_model(&mut tris, m, t.c, z, 0.0, surf + 0.02, 1.0, owner_rgb, dim, 0.7);
            }
        }
    }
    flush_tris(painter, tris);
}

pub fn draw_city_3d(
    pack: &Pack3D,
    painter: &Painter,
    game: &Game,
    city: &City,
    c: Pos2,
    z: f32,
) {
    let era = game.era_of(city.owner).min(3);
    let tier = (1 + city.population / 4).clamp(1, 4);
    let key = format!("{}_{era}_{tier}", if city.capital { "capital" } else { "city" });
    let owner_rgb = game.players[city.owner].primary;
    let content = &game.rules.content;
    let hills = content.terrains[game.map.tile(city.pos).terrain].hills;
    let surf = if hills { 0.16 } else { 0.02 };
    let mut tris = Vec::new();
    if let Some(m) = pack.get(&key) {
        append_model(&mut tris, m, c, z, 0.0, surf, 1.0, owner_rgb, 1.0, 2.0);
    }
    let has_b = |id: &str| city.buildings.iter().any(|&b| content.buildings[b].id == id);
    let wall_id = if has_b("BUILDING_STAR_FORT") {
        Some("BUILDING_STAR_FORT")
    } else if has_b("BUILDING_CASTLE") {
        Some("BUILDING_CASTLE")
    } else if has_b("BUILDING_WALLS") {
        Some("BUILDING_WALLS")
    } else {
        None
    };
    if let Some(wid) = wall_id
        && let Some(wm) = pack.get(wid)
    {
        append_model(&mut tris, wm, c, z, 0.0, surf, 1.0, owner_rgb, 1.0, 2.2);
    }
    flush_tris(painter, tris);
}

#[allow(clippy::too_many_arguments)]
pub fn draw_unit_3d(
    pack: &Pack3D,
    icons: &mut crate::icons::Icons,
    painter: &Painter,
    game: &Game,
    unit: &Unit,
    c: Pos2,
    radius: f32,
    selected: bool,
    fx: &UnitFx,
) -> bool {
    let uid = &game.unit_type(unit).id;
    let Some(m) = pack.get(uid) else {
        return false;
    };
    let owner = &game.players[unit.owner];
    let ground_at = c;
    let ground = ground_at + vec2(0.0, radius * 0.55);
    let hover = ground - vec2(0.0, fx.lift);
    let spent = unit.moves == 0 && unit.owner == game.human();
    let ty = game.unit_type(unit);
    let class = crate::render::units::class_of(ty);
    let military = game.is_military(unit);
    let primary = Color32::from_rgb(owner.primary[0], owner.primary[1], owner.primary[2]);
    let secondary = Color32::from_rgb(owner.secondary[0], owner.secondary[1], owner.secondary[2]);

    if selected {
        crate::render::units::selection_ring(painter, ground, radius * 0.9, fx.time);
    }
    crate::render::units::ground_shadow(painter, ground, radius * 0.72, if spent { 75 } else { 105 });

    let bob = 0.018 * (fx.time * 2.8 + unit.id as f32).sin();
    let scale = radius * 1.65 * fx.scale;
    let yaw = -0.65 + fx.tilt * 2.5;
    let mut tris = Vec::with_capacity(m.tris.len());
    let dim = (if spent { 0.72 } else { 1.0 }) + fx.flash * 0.35;
    append_model(&mut tris, m, hover, scale, yaw, bob, 1.0, owner.primary, dim, 3.0);
    flush_tris(painter, tris);

    let at = ground_at - vec2(0.0, fx.lift);
    let br = (radius * 0.4).max(6.0);
    crate::render::units::draw_badge(
        painter,
        class,
        at + vec2(-radius * 0.82, -radius * 0.78),
        br,
        primary,
        secondary,
        spent,
    );
    if unit.fortified {
        crate::render::units::draw_fortified(painter, at + vec2(radius * 0.82, -radius * 0.78), br * 0.9, spent);
    }
    let _ = icons;
    crate::render::units::hp_bar(unit, painter, ground_at, radius, military);
    true
}

//! The world map and its generator: fractal continents, latitude and
//! moisture biomes, hills and mountains, Civ VI features and resources
//! placed by their own valid-terrain rules, and a few natural wonders.

use crate::hex::{Grid, Pos};
use crate::rng::Rng;
use crate::rules::Rules;

pub type PlayerId = usize;
pub type CityId = u32;

#[derive(Clone, Debug, Default)]
pub struct Tile {
    pub terrain: usize,
    pub feature: Option<usize>,
    pub resource: Option<usize>,
    pub improvement: Option<usize>,
    pub pillaged: bool,
    pub river: bool,
    /// Which neighbor the river flows to (`hex` direction order), when known.
    pub river_dir: Option<u8>,
    /// Steep coast (Earth map): +3 defense for units standing on it.
    pub cliffs: bool,
    /// Roads (built by traders) make every land tile cost 1 move.
    pub road: bool,
    pub owner: Option<PlayerId>,
    /// City that owns this tile (border growth assigns it).
    pub owning_city: Option<CityId>,
    /// District built on this tile (index into `content.districts`).
    pub district: Option<usize>,
    /// Wonder built on this tile (index into `content.buildings`).
    pub wonder: Option<usize>,
}

/// The kind of world: Earth (real geography and historical starts) or a
/// randomly generated one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapType {
    Earth,
    Random,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapSize {
    Small,
    Standard,
    Large,
}

impl MapSize {
    pub fn grid(self) -> Grid {
        match self {
            MapSize::Small => Grid {
                width: 52,
                height: 32,
            },
            MapSize::Standard => Grid {
                width: 70,
                height: 42,
            },
            MapSize::Large => Grid {
                width: 90,
                height: 54,
            },
        }
    }
}

#[derive(Clone)]
pub struct Map {
    pub grid: Grid,
    pub tiles: Vec<Tile>,
}

impl Map {
    pub fn tile(&self, pos: Pos) -> &Tile {
        &self.tiles[self.grid.index(pos)]
    }

    pub fn tile_mut(&mut self, pos: Pos) -> &mut Tile {
        let index = self.grid.index(pos);
        &mut self.tiles[index]
    }

    pub fn is_water(&self, rules: &Rules, pos: Pos) -> bool {
        rules.content.terrains[self.tile(pos).terrain].water
    }

    pub fn is_land(&self, rules: &Rules, pos: Pos) -> bool {
        !self.is_water(rules, pos)
    }

    pub fn is_coastal_land(&self, rules: &Rules, pos: Pos) -> bool {
        self.is_land(rules, pos) && self.grid.neighbors(pos).any(|n| self.is_water(rules, n))
    }

    /// Land next to the ocean: a body of water of at least `LAKE_MAX + 1`
    /// tiles (smaller ones are lakes).
    pub fn touches_ocean(&self, rules: &Rules, pos: Pos) -> bool {
        const LAKE_MAX: usize = 9;
        self.is_land(rules, pos)
            && self.grid.neighbors(pos).any(|start| {
                if !self.is_water(rules, start) {
                    return false;
                }
                // Flood the water body, stopping once it is too big for a lake.
                let mut seen = vec![start];
                let mut next = 0;
                while next < seen.len() && seen.len() <= LAKE_MAX {
                    for n in self.grid.neighbors(seen[next]) {
                        if self.is_water(rules, n) && !seen.contains(&n) {
                            seen.push(n);
                        }
                    }
                    next += 1;
                }
                seen.len() > LAKE_MAX
            })
    }

    /// Mountains and impassable features (most natural wonders).
    pub fn is_impassable(&self, rules: &Rules, pos: Pos) -> bool {
        let tile = self.tile(pos);
        rules.content.terrains[tile.terrain].mountain
            || tile
                .feature
                .is_some_and(|f| rules.content.features[f].impassable)
    }

    /// Base yields of a tile before its city's modifiers: terrain, feature,
    /// river, revealed resource, and improvement.
    pub fn base_yields(&self, rules: &Rules, pos: Pos, resource_visible: bool) -> [i32; 6] {
        let tile = self.tile(pos);
        let content = &rules.content;
        let mut yields = content.terrains[tile.terrain].yields;
        if let Some(feature) = tile.feature {
            yields = add(yields, content.features[feature].yields);
        }
        if tile.river {
            yields[crate::rules::GOLD] += 1;
        }
        if let Some(resource) = tile.resource.filter(|_| resource_visible) {
            yields = add(yields, content.resources[resource].yields);
        }
        if let Some(improvement) = tile.improvement.filter(|_| !tile.pillaged) {
            yields = add(yields, content.improvements[improvement].yields);
        }
        yields
    }
}

pub fn add(a: [i32; 6], b: [i32; 6]) -> [i32; 6] {
    std::array::from_fn(|i| a[i] + b[i])
}

/// Smooth periodic value noise (wraps across `period` horizontally).
struct Noise {
    period: usize,
    rows: usize,
    lattice: Vec<f64>,
}

impl Noise {
    fn new(rng: &mut Rng, period: usize, rows: usize) -> Self {
        Self {
            period,
            rows,
            lattice: (0..period * (rows + 1)).map(|_| rng.unit()).collect(),
        }
    }

    fn at(&self, x: f64, y: f64) -> f64 {
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (smooth(x - x0), smooth(y - y0));
        let get = |ix: i64, iy: i64| {
            let ix = ix.rem_euclid(self.period as i64) as usize;
            let iy = (iy.max(0) as usize).min(self.rows);
            self.lattice[iy * self.period + ix]
        };
        let (ix, iy) = (x0 as i64, y0 as i64);
        let top = lerp(get(ix, iy), get(ix + 1, iy), fx);
        let bottom = lerp(get(ix, iy + 1), get(ix + 1, iy + 1), fx);
        lerp(top, bottom, fy)
    }
}

fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Fractal noise in [0, 1] that wraps east-west.
fn fractal(rng: &mut Rng, grid: Grid, base_cells: usize, octaves: usize) -> Vec<f64> {
    let layers: Vec<Noise> = (0..octaves)
        .map(|o| {
            let cells = base_cells << o;
            Noise::new(
                rng,
                cells,
                cells * grid.height as usize / grid.width as usize + 2,
            )
        })
        .collect();
    grid.all()
        .map(|pos| {
            // Odd rows sit half a hex to the right.
            let fx = (pos.x as f64 + if pos.y & 1 == 1 { 0.5 } else { 0.0 }) / grid.width as f64;
            let fy = pos.y as f64 / grid.width as f64;
            let mut value = 0.0;
            let mut weight = 0.0;
            for (o, layer) in layers.iter().enumerate() {
                let amplitude = 0.5f64.powi(o as i32);
                value += layer.at(fx * layer.period as f64, fy * layer.period as f64) * amplitude;
                weight += amplitude;
            }
            value / weight
        })
        .collect()
}

/// Value at the given quantile of `values` (0..1).
fn quantile(values: &[f64], q: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    sorted[((sorted.len() - 1) as f64 * q) as usize]
}

pub fn generate(rules: &Rules, size: MapSize, rng: &mut Rng) -> Map {
    let grid = size.grid();
    let terrain = |id: &str| rules.idx(id).expect("terrain");
    let feature = |id: &str| rules.idx(id);

    let elevation = fractal(rng, grid, 5, 5);
    let moisture = fractal(rng, grid, 4, 4);
    let heat_noise = fractal(rng, grid, 6, 3);

    // Push land away from the poles so continents sit in the middle.
    let shaped: Vec<f64> = grid
        .all()
        .zip(&elevation)
        .map(|(pos, e)| {
            let latitude = (pos.y as f64 / (grid.height - 1) as f64 - 0.5).abs() * 2.0;
            e - 0.35 * latitude.powi(4)
        })
        .collect();
    let sea_level = quantile(&shaped, 0.62);
    let land: Vec<f64> = shaped.iter().copied().filter(|e| *e >= sea_level).collect();
    let hill_level = quantile(&land, 0.72);
    let mountain_level = quantile(&land, 0.94);

    let mut tiles = vec![Tile::default(); grid.len()];
    for pos in grid.all() {
        let i = grid.index(pos);
        let latitude = (pos.y as f64 / (grid.height - 1) as f64 - 0.5).abs() * 2.0;
        let heat = 1.0 - latitude + (heat_noise[i] - 0.5) * 0.25;
        let wet = moisture[i];
        tiles[i].terrain = if shaped[i] < sea_level {
            terrain("TERRAIN_OCEAN")
        } else {
            let base = if heat < 0.12 {
                "SNOW"
            } else if heat < 0.28 {
                "TUNDRA"
            } else if heat > 0.62 && wet < 0.42 {
                "DESERT"
            } else if wet < 0.5 {
                "PLAINS"
            } else {
                "GRASS"
            };
            let relief = if shaped[i] >= mountain_level {
                "_MOUNTAIN"
            } else if shaped[i] >= hill_level {
                "_HILLS"
            } else {
                ""
            };
            terrain(&format!("TERRAIN_{base}{relief}"))
        };
    }
    let mut map = Map { grid, tiles };

    // Coast: water next to land; also a few shallow tiles further out.
    let coast = terrain("TERRAIN_COAST");
    let water_positions: Vec<Pos> = grid.all().filter(|&p| map.is_water(rules, p)).collect();
    for &pos in &water_positions {
        if grid.neighbors(pos).any(|n| map.is_land(rules, n)) {
            map.tile_mut(pos).terrain = coast;
        }
    }
    // Deep sea: open water more than three tiles from any land.
    if let Some(deep) = rules.idx("TERRAIN_DEEP_OCEAN") {
        let land: Vec<Pos> = grid.all().filter(|&p| map.is_land(rules, p)).collect();
        for &pos in &water_positions {
            if map.tile(pos).terrain != coast
                && !grid.within(pos, 3).iter().any(|p| land.contains(p))
            {
                map.tile_mut(pos).terrain = deep;
            }
        }
    }

    // Features.
    let (taiga, dunes, glacier) = (
        feature("FEATURE_TAIGA"),
        feature("FEATURE_DUNES"),
        feature("FEATURE_GLACIER"),
    );
    let (forest, jungle, marsh, oasis, ice, floodplains) = (
        feature("FEATURE_FOREST"),
        feature("FEATURE_JUNGLE"),
        feature("FEATURE_MARSH"),
        feature("FEATURE_OASIS"),
        feature("FEATURE_ICE"),
        feature("FEATURE_FLOODPLAINS"),
    );
    for pos in grid.all() {
        let i = grid.index(pos);
        let latitude = (pos.y as f64 / (grid.height - 1) as f64 - 0.5).abs() * 2.0;
        let terrain = &rules.content.terrains[map.tile(pos).terrain];
        let id = terrain.id.as_str();
        let wet = moisture[i];
        let chosen = if terrain.water {
            (latitude > 0.86 && rng.chance(0.6))
                .then_some(ice)
                .flatten()
        } else if terrain.mountain {
            None
        } else if id.starts_with("TERRAIN_DESERT") {
            if !terrain.hills && wet < 0.3 && rng.chance(0.4) {
                dunes
            } else {
                (!terrain.hills && rng.chance(0.03))
                    .then_some(oasis)
                    .flatten()
            }
        } else if id.starts_with("TERRAIN_SNOW") {
            (latitude > 0.9 && rng.chance(0.5))
                .then_some(glacier)
                .flatten()
        } else if id.starts_with("TERRAIN_TUNDRA") && wet > 0.4 && rng.chance(0.6) {
            taiga
        } else if latitude < 0.35
            && wet > 0.55
            && !id.starts_with("TERRAIN_TUNDRA")
            && rng.chance(0.75)
        {
            jungle
        } else if wet > 0.48 && rng.chance(0.55) {
            forest
        } else if id == "TERRAIN_GRASS" && wet > 0.6 && rng.chance(0.25) {
            marsh
        } else {
            None
        };
        if let Some(chosen) = chosen.filter(|&f| valid_feature(rules, f, map.tile(pos).terrain)) {
            map.tile_mut(pos).feature = Some(chosen);
        }
    }

    place_rivers(rules, &mut map, &shaped, sea_level, rng, floodplains);
    place_natural_wonders(rules, &mut map, rng);
    place_resources(rules, &mut map, rng);
    map
}

fn valid_feature(rules: &Rules, feature: usize, terrain: usize) -> bool {
    let feature = &rules.content.features[feature];
    feature.valid_terrains.is_empty()
        || feature
            .valid_terrains
            .contains(&rules.content.terrains[terrain].id)
}

/// Rivers run downhill from high ground to the sea. Rivers are a tile flag
/// (WayCiv simplifies Civ VI's edge rivers), giving +1 Gold and fresh water.
fn place_rivers(
    rules: &Rules,
    map: &mut Map,
    elevation: &[f64],
    sea_level: f64,
    rng: &mut Rng,
    floodplains: Option<usize>,
) {
    let grid = map.grid;
    let sources: Vec<Pos> = grid
        .all()
        .filter(|&p| {
            let t = &rules.content.terrains[map.tile(p).terrain];
            !t.water && (t.hills || t.mountain)
        })
        .collect();
    let count = (grid.len() / 160).max(4);
    for _ in 0..count {
        let Some(&start) = rng.pick(&sources) else {
            return;
        };
        let mut pos = start;
        for _ in 0..30 {
            if map.is_water(rules, pos) {
                break;
            }
            if !rules.content.terrains[map.tile(pos).terrain].mountain {
                map.tile_mut(pos).river = true;
                let tile = map.tile(pos);
                if let Some(fp) = floodplains {
                    let terrain = &rules.content.terrains[tile.terrain];
                    if tile.feature.is_none()
                        && !terrain.hills
                        && terrain.id == "TERRAIN_DESERT"
                        && valid_feature(rules, fp, tile.terrain)
                    {
                        map.tile_mut(pos).feature = Some(fp);
                    }
                }
                let tile = map.tile(pos);
                let terrain = &rules.content.terrains[tile.terrain];
                let wet_floodplains = match terrain.id.as_str() {
                    "TERRAIN_GRASS" => rules.idx("FEATURE_FLOODPLAINS_GRASSLAND"),
                    "TERRAIN_PLAINS" => rules.idx("FEATURE_FLOODPLAINS_PLAINS"),
                    _ => None,
                };
                if let Some(fp) =
                    wet_floodplains.filter(|_| tile.feature.is_none() && rng.chance(0.4))
                {
                    map.tile_mut(pos).feature = Some(fp);
                }
            }
            let next = grid.neighbors(pos).min_by(|a, b| {
                elevation[grid.index(*a)]
                    .partial_cmp(&elevation[grid.index(*b)])
                    .unwrap()
            });
            match next {
                Some(n)
                    if elevation[grid.index(n)] < elevation[grid.index(pos)]
                        || elevation[grid.index(n)] < sea_level =>
                {
                    pos = n
                }
                _ => break,
            }
        }
    }
}

fn place_natural_wonders(rules: &Rules, map: &mut Map, rng: &mut Rng) {
    let grid = map.grid;
    let mut wonders: Vec<usize> = rules
        .content
        .features
        .iter()
        .enumerate()
        .filter(|(_, f)| f.natural_wonder && !f.valid_terrains.is_empty())
        .map(|(i, _)| i)
        .collect();
    rng.shuffle(&mut wonders);
    let wanted = (grid.len() / 500).clamp(3, 7);
    let mut placed: Vec<Pos> = Vec::new();
    for wonder in wonders {
        if placed.len() >= wanted {
            break;
        }
        let candidates: Vec<Pos> = grid
            .all()
            .filter(|&p| {
                let tile = map.tile(p);
                tile.feature.is_none()
                    && valid_feature(rules, wonder, tile.terrain)
                    && placed.iter().all(|q| grid.distance(*q, p) > 8)
                    && p.y > 2
                    && p.y < grid.height - 3
            })
            .collect();
        if let Some(&pos) = rng.pick(&candidates) {
            map.tile_mut(pos).feature = Some(wonder);
            placed.push(pos);
        }
    }
}

pub(crate) fn place_resources(rules: &Rules, map: &mut Map, rng: &mut Rng) {
    let grid = map.grid;
    for pos in grid.all() {
        let tile = map.tile(pos);
        if tile
            .feature
            .is_some_and(|f| rules.content.features[f].natural_wonder)
        {
            continue;
        }
        let terrain = &rules.content.terrains[tile.terrain];
        let open_sea = terrain.water && !terrain.shallow_water;
        if open_sea
            || tile
                .feature
                .is_some_and(|f| rules.content.features[f].impassable)
            || !rng.chance(if terrain.water { 0.06 } else { 0.11 })
        {
            continue;
        }
        let terrain_id = &terrain.id;
        let feature_id = tile.feature.map(|f| rules.content.features[f].id.as_str());
        let candidates: Vec<(usize, i32)> = rules
            .content
            .resources
            .iter()
            .enumerate()
            .filter(|(_, r)| match feature_id {
                Some(f) => r.valid_features.iter().any(|v| v == f),
                None => r.valid_terrains.iter().any(|v| v == terrain_id),
            })
            .map(|(i, r)| {
                (
                    i,
                    if terrain.water {
                        r.sea_frequency.max(r.frequency)
                    } else {
                        r.frequency
                    }
                    .max(1),
                )
            })
            .collect();
        let total: i32 = candidates.iter().map(|(_, w)| w).sum();
        if total == 0 {
            continue;
        }
        let mut roll = rng.below(total as u32) as i32;
        for (resource, weight) in candidates {
            if roll < weight {
                map.tile_mut(pos).resource = Some(resource);
                break;
            }
            roll -= weight;
        }
    }
}

/// Fertility of a city site: the yields of tiles within two hexes.
pub fn site_score(rules: &Rules, map: &Map, pos: Pos) -> f64 {
    if !map.is_land(rules, pos) || map.is_impassable(rules, pos) {
        return f64::MIN;
    }
    let mut score = 0.0;
    for p in map.grid.within(pos, 2) {
        let y = map.base_yields(rules, p, true);
        score += y[0] as f64 * 1.2 + y[1] as f64 + y[2] as f64 * 0.4 + y[5] as f64 * 0.3;
        if map.tile(p).resource.is_some() {
            score += 1.0;
        }
        if map.is_water(rules, p) {
            score -= 0.3;
        }
    }
    if map.tile(pos).river || map.is_coastal_land(rules, pos) {
        score += 4.0;
    }
    if rules.content.terrains[map.tile(pos).terrain].hills {
        score += 2.0;
    }
    score
}

/// Picks `count` well-spaced, fertile starting tiles.
pub fn starting_positions(
    rules: &Rules,
    map: &Map,
    count: usize,
    min_distance: i32,
    rng: &mut Rng,
) -> Vec<Pos> {
    starting_positions_avoiding(rules, map, count, min_distance, &[], rng)
}

/// Like `starting_positions`, also keeping clear of tiles already taken.
pub fn starting_positions_avoiding(
    rules: &Rules,
    map: &Map,
    count: usize,
    min_distance: i32,
    taken: &[Pos],
    rng: &mut Rng,
) -> Vec<Pos> {
    let grid = map.grid;
    let mut candidates: Vec<(Pos, f64)> = grid
        .all()
        .filter(|&p| p.y > 3 && p.y < grid.height - 4)
        .map(|p| (p, site_score(rules, map, p)))
        .filter(|(_, s)| *s > 12.0)
        .collect();
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let mut chosen: Vec<Pos> = Vec::new();
    let mut distance = min_distance;
    while chosen.len() < count && distance >= 3 {
        let mut pool: Vec<Pos> = candidates
            .iter()
            .take(candidates.len() / 2 + 1)
            .map(|(p, _)| *p)
            .collect();
        rng.shuffle(&mut pool);
        for pos in pool {
            if chosen.len() >= count {
                break;
            }
            if chosen
                .iter()
                .chain(taken)
                .all(|c| grid.distance(*c, pos) >= distance)
            {
                chosen.push(pos);
            }
        }
        distance -= 1;
    }
    chosen
}

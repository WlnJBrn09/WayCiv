//! The Earth map: real landmasses, relief, and biomes, built offline from
//! NOAA ETOPO1 elevation and WorldClim 2.1 climate data by
//! `tools/earth/build_earth.py`. Leaders start at their historical capitals,
//! city-states at their real cities, and natural wonders sit where they are
//! on Earth.
//!
//! Map file: a header `WAYCIV-EARTH <width> <height> <lat top> <lat bottom>
//! <lon west>`, then `height` rows each of terrain, feature, and flag codes.
//! Terrain: `g p d t s` grassland, plains, desert, tundra, snow (uppercase:
//! hills; digits 1-5: the same as mountains), `c o x` coast, sea, deep sea.
//! Feature: `.` none, `f` forest, `T` taiga, `j` rainforest, `m` marsh,
//! `o` oasis, `F` floodplains, `u` dunes, `G` glacier, `i` sea ice.
//! Flags: `.` none, `c` cliffs, `0`-`5` a river flowing toward that
//! neighbor (east, then counter-clockwise as in `hex`), `A`-`F` the same
//! with cliffs.

use crate::hex::{Grid, Pos};
use crate::map::{self, Map, MapSize, Tile};
use crate::rng::Rng;
use crate::rules::Rules;

const SMALL: &str = include_str!("../../../assets/maps/earth_104x72.txt");
const STANDARD: &str = include_str!("../../../assets/maps/earth_128x88.txt");
const LARGE: &str = include_str!("../../../assets/maps/earth_160x110.txt");

/// The projection a map file was built with (Miller cylindrical).
#[derive(Clone, Copy, Debug)]
pub struct Projection {
    pub grid: Grid,
    pub lat_top: f64,
    pub lon_west: f64,
}

fn miller(lat: f64) -> f64 {
    1.25 * (std::f64::consts::FRAC_PI_4 + 0.4 * lat.to_radians()).tan().ln()
}

impl Projection {
    /// The tile containing a latitude/longitude, if it is on the map.
    pub fn tile_of(&self, lat: f64, lon: f64) -> Option<Pos> {
        let width = self.grid.width as f64;
        let hex_w = std::f64::consts::TAU / width;
        let size = hex_w / 3f64.sqrt();
        let row_h = 1.5 * size;
        let px = ((lon - self.lon_west).rem_euclid(360.0)).to_radians() - hex_w / 2.0;
        let py = miller(self.lat_top) - miller(lat.clamp(-89.0, 89.0)) - row_h / 2.0;
        let q = (3f64.sqrt() / 3.0 * px - py / 3.0) / size;
        let r = (2.0 / 3.0 * py) / size;
        let (x, z) = (q, r);
        let y = -x - z;
        let (mut rx, ry, mut rz) = (x.round(), y.round(), z.round());
        let (dx, dy, dz) = ((rx - x).abs(), (ry - y).abs(), (rz - z).abs());
        if dx > dy && dx > dz {
            rx = -ry - rz;
        } else if dy <= dz {
            rz = -rx - ry;
        }
        self.grid.normalize(Pos::from_axial(rx as i32, rz as i32))
    }
}

fn source(size: MapSize) -> &'static str {
    match size {
        MapSize::Small => SMALL,
        MapSize::Standard => STANDARD,
        MapSize::Large => LARGE,
    }
}

pub fn projection(size: MapSize) -> Projection {
    let header: Vec<&str> = source(size).lines().next().unwrap_or_default().split_whitespace().collect();
    let num = |i: usize| header.get(i).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
    Projection { grid: Grid { width: num(1) as i32, height: num(2) as i32 }, lat_top: num(3), lon_west: num(5) }
}

/// Builds the Earth map at `size`: terrain from the map file, then natural
/// wonders at their real places and resources by Civ VI's placement rules.
pub fn generate(rules: &Rules, size: MapSize, rng: &mut Rng) -> Map {
    let text = source(size);
    let proj = projection(size);
    let grid = proj.grid;
    let rows: Vec<&[u8]> = text.lines().skip(1).map(str::as_bytes).collect();
    let (w, h) = (grid.width as usize, grid.height as usize);
    let terrain_id = |code: u8| -> &'static str {
        match code {
            b'g' => "TERRAIN_GRASS",
            b'G' => "TERRAIN_GRASS_HILLS",
            b'1' => "TERRAIN_GRASS_MOUNTAIN",
            b'p' => "TERRAIN_PLAINS",
            b'P' => "TERRAIN_PLAINS_HILLS",
            b'2' => "TERRAIN_PLAINS_MOUNTAIN",
            b'd' => "TERRAIN_DESERT",
            b'D' => "TERRAIN_DESERT_HILLS",
            b'3' => "TERRAIN_DESERT_MOUNTAIN",
            b't' => "TERRAIN_TUNDRA",
            b'T' => "TERRAIN_TUNDRA_HILLS",
            b'4' => "TERRAIN_TUNDRA_MOUNTAIN",
            b's' => "TERRAIN_SNOW",
            b'S' => "TERRAIN_SNOW_HILLS",
            b'5' => "TERRAIN_SNOW_MOUNTAIN",
            b'c' => "TERRAIN_COAST",
            b'x' => "TERRAIN_DEEP_OCEAN",
            _ => "TERRAIN_OCEAN",
        }
    };
    let mut tiles = vec![Tile::default(); grid.len()];
    for y in 0..h {
        for x in 0..w {
            let at = |block: usize| rows.get(block * h + y).and_then(|r| r.get(x)).copied().unwrap_or(b'.');
            let tile = &mut tiles[y * w + x];
            tile.terrain = rules.idx(terrain_id(at(0))).or_else(|| rules.idx("TERRAIN_OCEAN")).unwrap_or(0);
            let base = rules.content.terrains[tile.terrain].id.as_str();
            let feature = match at(1) {
                b'f' => Some("FEATURE_FOREST"),
                b'T' => Some("FEATURE_TAIGA"),
                b'j' => Some("FEATURE_JUNGLE"),
                b'm' => Some("FEATURE_MARSH"),
                b'o' => Some("FEATURE_OASIS"),
                b'u' => Some("FEATURE_DUNES"),
                b'G' => Some("FEATURE_GLACIER"),
                b'i' => Some("FEATURE_ICE"),
                b'F' => Some(match base {
                    "TERRAIN_DESERT" => "FEATURE_FLOODPLAINS",
                    "TERRAIN_GRASS" => "FEATURE_FLOODPLAINS_GRASSLAND",
                    _ => "FEATURE_FLOODPLAINS_PLAINS",
                }),
                _ => None,
            };
            tile.feature = feature.and_then(|f| rules.idx(f));
            match at(2) {
                b'c' => tile.cliffs = true,
                d @ b'0'..=b'5' => tile.river_dir = Some(d - b'0'),
                d @ b'A'..=b'F' => {
                    tile.river_dir = Some(d - b'A');
                    tile.cliffs = true;
                }
                _ => {}
            }
            tile.river = tile.river_dir.is_some();
        }
    }
    let mut map = Map { grid, tiles };
    place_natural_wonders(rules, &mut map, &proj);
    map::place_resources(rules, &mut map, rng);
    map
}

/// Natural wonders at their real locations: (feature, latitude, longitude).
const NATURAL_WONDERS: [(&str, f64, f64); 20] = [
    ("FEATURE_BARRIER_REEF", -18.3, 147.7),
    ("FEATURE_CLIFFS_DOVER", 51.13, 1.0),
    ("FEATURE_CRATER_LAKE", 42.94, -122.1),
    ("FEATURE_DEAD_SEA", 31.5, 35.5),
    ("FEATURE_EVEREST", 27.99, 86.93),
    ("FEATURE_GALAPAGOS", -0.6, -90.5),
    ("FEATURE_KILIMANJARO", -3.07, 37.35),
    ("FEATURE_PANTANAL", -17.5, -57.0),
    ("FEATURE_PIOPIOTAHI", -44.6, 167.9),
    ("FEATURE_TORRES_DEL_PAINE", -51.0, -73.0),
    ("FEATURE_TSINGY", -18.7, 44.7),
    ("FEATURE_YOSEMITE", 37.8, -119.5),
    ("FEATURE_ULURU", -25.3, 131.0),
    ("FEATURE_HA_LONG_BAY", 20.9, 107.1),
    ("FEATURE_EYJAFJALLAJOKULL", 63.6, -19.6),
    ("FEATURE_LYSEFJORDEN", 59.0, 6.2),
    ("FEATURE_GIANTS_CAUSEWAY", 55.24, -6.51),
    ("FEATURE_BERMUDA_TRIANGLE", 25.0, -71.0),
    ("FEATURE_FOUNTAIN_OF_YOUTH", 28.0, -81.5),
    ("FEATURE_PAITITI", -12.5, -71.0),
];

fn place_natural_wonders(rules: &Rules, map: &mut Map, proj: &Projection) {
    for (id, lat, lon) in NATURAL_WONDERS {
        let (Some(feature), Some(target)) = (rules.idx(id), proj.tile_of(lat, lon)) else { continue };
        let Some(f) = rules.content.features.get(feature).filter(|f| f.id == id) else { continue };
        // The nearest tile of a terrain the wonder can sit on; failing that,
        // the target's terrain becomes the wonder's first valid one.
        let valid = |map: &Map, p: Pos| {
            let tile = map.tile(p);
            f.valid_terrains.contains(&rules.content.terrains[tile.terrain].id)
                && tile.feature.is_none_or(|x| !rules.content.features[x].natural_wonder)
        };
        let spot = map.grid.within(target, 2).into_iter().find(|&p| valid(map, p));
        let pos = match spot {
            Some(p) => p,
            None => {
                let Some(terrain) = f.valid_terrains.first().and_then(|t| rules.idx(t)) else { continue };
                map.tile_mut(target).terrain = terrain;
                target
            }
        };
        let tile = map.tile_mut(pos);
        tile.feature = Some(feature);
        tile.river = false;
        tile.river_dir = None;
    }
}

/// Where each leader starts: their historical capital.
const LEADER_STARTS: [(&str, f64, f64); 36] = [
    ("LEADER_REAGAN", 38.9, -77.0),
    ("LEADER_SALADIN", 21.4, 39.8),
    ("LEADER_PEDRO", -22.9, -43.2),
    ("LEADER_QIN", 34.3, 108.9),
    ("LEADER_CLEOPATRA", 30.4, 31.0),
    ("LEADER_VICTORIA", 51.5, -0.1),
    ("LEADER_CATHERINE_DE_MEDICI", 48.9, 2.35),
    ("LEADER_CATHERINE_DE_MEDICI_ALT", 48.9, 2.35),
    ("LEADER_BARBAROSSA", 50.8, 6.1),
    ("LEADER_GORGO", 37.1, 22.4),
    ("LEADER_PERICLES", 38.0, 23.7),
    ("LEADER_HOJO", 35.0, 135.8),
    ("LEADER_MVEMBA", -6.3, 14.2),
    ("LEADER_HARDRADA", 63.4, 10.4),
    ("LEADER_TRAJAN", 41.9, 12.5),
    ("LEADER_PETER_GREAT", 59.9, 30.3),
    ("LEADER_TOMYRIS", 51.0, 55.5),
    ("LEADER_PHILIP_II", 40.4, -3.7),
    ("LEADER_GILGAMESH", 31.3, 45.6),
    ("LEADER_MONTEZUMA", 19.4, -99.1),
    ("LEADER_JADWIGA", 50.06, 19.94),
    ("LEADER_JOHN_CURTIN", -35.3, 149.1),
    ("LEADER_ALEXANDER", 40.76, 22.52),
    ("LEADER_CYRUS", 30.2, 53.2),
    ("LEADER_AMANITORE", 16.9, 33.7),
    ("LEADER_GITARJA", -7.5, 112.4),
    ("LEADER_JAYAVARMAN", 13.4, 103.9),
    ("LEADER_MENELIK", 9.0, 38.7),
    ("LEADER_BASIL", 41.0, 28.98),
    ("LEADER_AMBIORIX", 50.8, 5.5),
    ("LEADER_HAMMURABI", 32.5, 44.4),
    ("LEADER_SIMON_BOLIVAR", 4.7, -74.1),
    ("LEADER_LADY_SIX_SKY", 17.2, -89.6),
    ("LEADER_JOAO_III", 38.7, -9.1),
    ("LEADER_KUBLAI_KHAN_CHINA", 39.9, 116.4),
    ("LEADER_LADY_TRIEU", 21.0, 105.8),
];

/// City-states at their real cities (keyed by Civ VI's civilization type,
/// which for a few city-states is an older name: `CIVILIZATION_LISBON` is
/// Mogadishu, `CIVILIZATION_JAKARTA` is Bandar Brunei, `CIVILIZATION_PALENQUE`
/// is Mitla).
const CITY_STATE_STARTS: [(&str, f64, f64); 42] = [
    ("CIVILIZATION_AMSTERDAM", 52.4, 4.9),
    ("CIVILIZATION_BRUSSELS", 50.85, 4.35),
    ("CIVILIZATION_BUENOS_AIRES", -34.6, -58.4),
    ("CIVILIZATION_CARTHAGE", 36.85, 10.3),
    ("CIVILIZATION_GENEVA", 46.2, 6.15),
    ("CIVILIZATION_HATTUSA", 40.0, 34.6),
    ("CIVILIZATION_HONG_KONG", 22.3, 114.2),
    ("CIVILIZATION_JAKARTA", 4.9, 114.9),
    ("CIVILIZATION_JERUSALEM", 31.8, 35.2),
    ("CIVILIZATION_KABUL", 34.5, 69.2),
    ("CIVILIZATION_KANDY", 7.3, 80.6),
    ("CIVILIZATION_KUMASI", 6.7, -1.6),
    ("CIVILIZATION_LA_VENTA", 18.1, -94.0),
    ("CIVILIZATION_LISBON", 2.0, 45.3),
    ("CIVILIZATION_MOHENJO_DARO", 27.3, 68.1),
    ("CIVILIZATION_NAN_MADOL", 6.8, 158.3),
    ("CIVILIZATION_PRESLAV", 43.15, 26.8),
    ("CIVILIZATION_SEOUL", 37.6, 127.0),
    ("CIVILIZATION_STOCKHOLM", 59.3, 18.1),
    ("CIVILIZATION_TORONTO", 43.7, -79.4),
    ("CIVILIZATION_VALLETTA", 35.9, 14.5),
    ("CIVILIZATION_VILNIUS", 54.7, 25.3),
    ("CIVILIZATION_YEREVAN", 40.2, 44.5),
    ("CIVILIZATION_ZANZIBAR", -6.2, 39.2),
    ("CIVILIZATION_AUCKLAND", -36.85, 174.8),
    ("CIVILIZATION_ANTANANARIVO", -18.9, 47.5),
    ("CIVILIZATION_ARMAGH", 54.35, -6.65),
    ("CIVILIZATION_GRANADA", 37.2, -3.6),
    ("CIVILIZATION_MUSCAT", 23.6, 58.4),
    ("CIVILIZATION_PALENQUE", 16.9, -96.4),
    ("CIVILIZATION_AYUTTHAYA", 14.35, 100.6),
    ("CIVILIZATION_CHINGUETTI", 20.5, -12.4),
    ("CIVILIZATION_JOHANNESBURG", -26.2, 28.0),
    ("CIVILIZATION_NALANDA", 25.1, 85.4),
    ("CIVILIZATION_SAMARKAND", 39.65, 66.96),
    ("CIVILIZATION_WOLIN", 53.85, 14.6),
    ("CIVILIZATION_CAGUANA", 18.3, -66.7),
    ("CIVILIZATION_HUNZA", 36.3, 74.65),
    ("CIVILIZATION_LAHORE", 31.55, 74.35),
    ("CIVILIZATION_SINGAPORE", 1.35, 103.8),
    ("CIVILIZATION_TARUGA", 9.4, 7.6),
    ("CIVILIZATION_VATICAN_CITY", 41.9, 12.45),
];

/// The real-world location of a leader or city-state, by its Civ VI id.
pub fn home(id: &str) -> Option<(f64, f64)> {
    LEADER_STARTS.iter().chain(CITY_STATE_STARTS.iter()).find(|(k, _, _)| *k == id).map(|&(_, lat, lon)| (lat, lon))
}

/// Starting tiles for each player, in order: the nearest good land tile to
/// their historical home that keeps `min_distance` from earlier starts.
/// Players without a known home get a fertile site elsewhere.
pub fn starting_positions(rules: &Rules, map: &Map, size: MapSize, homes: &[Option<(f64, f64)>], min_distance: i32, rng: &mut Rng) -> Vec<Pos> {
    let proj = projection(size);
    let grid = map.grid;
    let usable = |p: Pos, chosen: &[Pos]| {
        map.is_land(rules, p)
            && !map.is_impassable(rules, p)
            && map.tile(p).feature.is_none_or(|f| !rules.content.features[f].natural_wonder)
            && chosen.iter().all(|c| c.x < 0 || grid.distance(*c, p) >= min_distance)
    };
    let mut chosen: Vec<Pos> = Vec::new();
    let mut unplaced = Vec::new();
    for (i, home) in homes.iter().enumerate() {
        let target = home.and_then(|(lat, lon)| proj.tile_of(lat, lon));
        let spot = target.and_then(|t| {
            (0..=8).find_map(|radius| {
                grid.within(t, radius)
                    .into_iter()
                    .filter(|&p| grid.distance(t, p) == radius && usable(p, &chosen))
                    .max_by(|a, b| map::site_score(rules, map, *a).total_cmp(&map::site_score(rules, map, *b)))
            })
        });
        match spot {
            Some(p) => chosen.push(p),
            None => {
                chosen.push(Pos::new(-1, -1));
                unplaced.push(i);
            }
        }
    }
    if !unplaced.is_empty() {
        let placed: Vec<Pos> = chosen.iter().copied().filter(|p| p.x >= 0).collect();
        let extra = map::starting_positions_avoiding(rules, map, unplaced.len(), min_distance, &placed, rng);
        for (slot, pos) in unplaced.into_iter().zip(extra) {
            chosen[slot] = pos;
        }
    }
    chosen.retain(|p| p.x >= 0);
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_files_match_their_headers() {
        for size in [MapSize::Small, MapSize::Standard, MapSize::Large] {
            let proj = projection(size);
            let rows = source(size).lines().skip(1).collect::<Vec<_>>();
            assert_eq!(rows.len(), proj.grid.height as usize * 3, "{size:?}");
            assert!(rows.iter().all(|r| r.len() == proj.grid.width as usize), "{size:?}");
        }
    }

    #[test]
    fn known_places_land_on_the_right_terrain() {
        let proj = projection(MapSize::Standard);
        let text = source(MapSize::Standard);
        let rows: Vec<&str> = text.lines().skip(1).collect();
        let terrain = |lat: f64, lon: f64| {
            let p = proj.tile_of(lat, lon).unwrap();
            rows[p.y as usize].as_bytes()[p.x as usize]
        };
        // Mid-Atlantic and mid-Pacific are water; the Sahara is desert.
        assert!(matches!(terrain(30.0, -40.0), b'o' | b'x'));
        assert!(matches!(terrain(0.0, -140.0), b'o' | b'x'));
        assert!(matches!(terrain(23.0, 10.0), b'd' | b'D' | b'3'));
        // Paris and Beijing are land.
        assert!(!matches!(terrain(48.9, 2.35), b'c' | b'o' | b'x'));
        assert!(!matches!(terrain(39.9, 116.4), b'c' | b'o' | b'x'));
    }
}

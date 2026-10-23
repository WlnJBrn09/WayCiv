//! Colors: biomes, districts, yields, and the UI theme.

use egui::Color32;

pub fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

pub fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// Base color of a terrain type; hills and mountains share their biome's hue.
pub fn biome(terrain_id: &str) -> Color32 {
    let base = terrain_id
        .trim_start_matches("TERRAIN_")
        .trim_end_matches("_HILLS")
        .trim_end_matches("_MOUNTAIN");
    match base {
        "DEEP_OCEAN" => Color32::from_rgb(14, 34, 82),
        "OCEAN" => Color32::from_rgb(28, 62, 122),
        "COAST" => Color32::from_rgb(52, 112, 178),
        "GRASS" => Color32::from_rgb(86, 150, 58),
        "PLAINS" => Color32::from_rgb(170, 168, 78),
        "DESERT" => Color32::from_rgb(226, 200, 124),
        "TUNDRA" => Color32::from_rgb(128, 140, 118),
        "SNOW" => Color32::from_rgb(236, 240, 245),
        _ => Color32::from_rgb(120, 120, 120),
    }
}

pub fn biome_name(terrain_id: &str) -> &'static str {
    let base = terrain_id
        .trim_start_matches("TERRAIN_")
        .trim_end_matches("_HILLS")
        .trim_end_matches("_MOUNTAIN");
    match base {
        "DEEP_OCEAN" => "Deep Sea",
        "OCEAN" => "Sea",
        "COAST" => "Coast",
        "GRASS" => "Grassland",
        "PLAINS" => "Plains",
        "DESERT" => "Desert",
        "TUNDRA" => "Tundra",
        "SNOW" => "Snow",
        _ => "Unknown",
    }
}

pub const LEGEND: [(&str, &str); 8] = [
    ("TERRAIN_GRASS", "Grassland"),
    ("TERRAIN_PLAINS", "Plains"),
    ("TERRAIN_DESERT", "Desert"),
    ("TERRAIN_TUNDRA", "Tundra"),
    ("TERRAIN_SNOW", "Snow"),
    ("TERRAIN_COAST", "Coast"),
    ("TERRAIN_OCEAN", "Sea"),
    ("TERRAIN_DEEP_OCEAN", "Deep Sea"),
];

/// Features shown in the map key: (feature, base terrain to draw it on, label).
pub const LEGEND_FEATURES: [(&str, &str, &str); 10] = [
    ("FEATURE_FOREST", "TERRAIN_GRASS", "Forest"),
    ("FEATURE_TAIGA", "TERRAIN_TUNDRA", "Taiga"),
    ("FEATURE_JUNGLE", "TERRAIN_GRASS", "Rainforest"),
    ("FEATURE_MARSH", "TERRAIN_GRASS", "Marsh"),
    (
        "FEATURE_FLOODPLAINS_GRASSLAND",
        "TERRAIN_GRASS",
        "Floodplains",
    ),
    (
        "FEATURE_FLOODPLAINS",
        "TERRAIN_DESERT",
        "Desert floodplains",
    ),
    ("FEATURE_OASIS", "TERRAIN_DESERT", "Oasis"),
    ("FEATURE_DUNES", "TERRAIN_DESERT", "Dunes"),
    ("FEATURE_GLACIER", "TERRAIN_SNOW", "Glacier"),
    ("FEATURE_ICE", "TERRAIN_OCEAN", "Sea ice"),
];

/// (color, short label) for a district, by the base district it is or replaces.
pub fn district(base_id: &str) -> (Color32, &'static str) {
    match base_id {
        "DISTRICT_CAMPUS" => (Color32::from_rgb(70, 140, 230), "Ca"),
        "DISTRICT_HOLY_SITE" => (Color32::from_rgb(240, 240, 220), "HS"),
        "DISTRICT_THEATER" => (Color32::from_rgb(170, 90, 200), "Th"),
        "DISTRICT_COMMERCIAL_HUB" => (Color32::from_rgb(240, 200, 50), "CH"),
        "DISTRICT_HARBOR" => (Color32::from_rgb(60, 200, 210), "Ha"),
        "DISTRICT_ENCAMPMENT" => (Color32::from_rgb(200, 60, 50), "En"),
        "DISTRICT_INDUSTRIAL_ZONE" => (Color32::from_rgb(230, 130, 40), "IZ"),
        "DISTRICT_ENTERTAINMENT_COMPLEX" => (Color32::from_rgb(120, 210, 90), "EC"),
        "DISTRICT_AQUEDUCT" => (Color32::from_rgb(150, 200, 240), "Aq"),
        "DISTRICT_SPACEPORT" => (Color32::from_rgb(180, 180, 200), "SP"),
        "DISTRICT_NEIGHBORHOOD" => (Color32::from_rgb(200, 170, 140), "Nb"),
        "DISTRICT_AERODROME" => (Color32::from_rgb(160, 160, 160), "Ae"),
        "DISTRICT_DATACENTER" => (Color32::from_rgb(60, 220, 230), "DC"),
        _ => (Color32::from_rgb(190, 190, 190), "D"),
    }
}

pub const YIELD_COLORS: [Color32; 6] = [
    Color32::from_rgb(140, 210, 80),
    Color32::from_rgb(230, 150, 60),
    Color32::from_rgb(250, 210, 60),
    Color32::from_rgb(90, 170, 250),
    Color32::from_rgb(210, 120, 230),
    Color32::from_rgb(230, 230, 230),
];

pub const PANEL: Color32 = Color32::from_rgb(24, 30, 40);
pub const PANEL_LIGHT: Color32 = Color32::from_rgb(38, 46, 60);
pub const GOLD: Color32 = Color32::from_rgb(230, 190, 90);
/// The AI tree and Compute.
pub const COMPUTE: Color32 = Color32::from_rgb(70, 225, 235);

pub fn style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PANEL;
    visuals.window_fill = Color32::from_rgb(28, 34, 46);
    visuals.selection.bg_fill = Color32::from_rgb(70, 110, 170);
    ctx.set_visuals(visuals);
}

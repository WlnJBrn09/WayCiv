//! Colors: biomes, districts, yields, and the UI theme.

use egui::{Color32, FontFamily, FontId};

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

pub const PANEL: Color32 = Color32::from_rgb(14, 22, 36);
pub const GOLD: Color32 = Color32::from_rgb(232, 194, 100);
/// Trim on panels and rings: a darker gold.
pub const GOLD_DIM: Color32 = Color32::from_rgb(150, 120, 58);
/// The lit edge of gold trim and the highlight on gold lettering.
pub const GOLD_HI: Color32 = Color32::from_rgb(255, 229, 158);
/// The shaded edge of gold trim.
pub const GOLD_LO: Color32 = Color32::from_rgb(96, 72, 30);
/// Deep navy behind HUD plates.
pub const NAVY: Color32 = Color32::from_rgb(10, 18, 32);
/// Top and bottom of a panel's gradient.
pub const PLATE_TOP: Color32 = Color32::from_rgb(30, 48, 76);
pub const PLATE_BOTTOM: Color32 = Color32::from_rgb(8, 14, 26);
/// Raised navy: buttons and rows.
pub const NAVY_RAISED: Color32 = Color32::from_rgb(24, 40, 62);
pub const NAVY_HOVER: Color32 = Color32::from_rgb(34, 62, 88);
pub const TEAL: Color32 = Color32::from_rgb(64, 174, 184);
pub const TEAL_DIM: Color32 = Color32::from_rgb(30, 88, 100);
pub const TEXT: Color32 = Color32::from_rgb(234, 238, 242);
pub const TEXT_DIM: Color32 = Color32::from_rgb(158, 172, 186);
pub const GOOD: Color32 = Color32::from_rgb(112, 212, 112);
pub const BAD: Color32 = Color32::from_rgb(242, 94, 84);
/// The AI tree and Compute.
pub const COMPUTE: Color32 = Color32::from_rgb(70, 225, 235);

/// Noto Sans Bold, for names, numbers and button labels.
pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

/// Noto Serif Display Bold, for window titles and the game's name.
pub fn display(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("display".into()))
}

/// Registers the typefaces in `assets/fonts` (Noto Sans for text, Noto Serif
/// Display for titles, both OFL): egui's built-in fonts stay as the
/// fallback, and stand in for the whole set when the folder is missing.
fn install_fonts(ctx: &egui::Context) {
    use egui::{FontData, FontDefinitions};
    use std::sync::Arc;
    let mut fonts = FontDefinitions::default();
    let dir = crate::portraits::assets_dir().join("fonts");
    let fallback = fonts.families[&FontFamily::Proportional].clone();
    let mut load = |name: &str, file: &str| -> Vec<String> {
        let mut list = fallback.clone();
        if let Ok(bytes) = std::fs::read(dir.join(file)) {
            fonts
                .font_data
                .insert(name.to_string(), Arc::new(FontData::from_owned(bytes)));
            list.insert(0, name.to_string());
        }
        list
    };
    let text = load("noto-sans", "NotoSans-Medium.ttf");
    let bold = load("noto-sans-bold", "NotoSans-Bold.ttf");
    let display = load("noto-serif-display", "NotoSerifDisplay-Bold.ttf");
    fonts.families.insert(FontFamily::Proportional, text);
    fonts.families.insert(FontFamily::Name("bold".into()), bold);
    fonts
        .families
        .insert(FontFamily::Name("display".into()), display);
    ctx.set_fonts(fonts);
}

/// Civ VI-on-Switch look: dark navy and teal panels with gold trim, rounded
/// corners, and large readable text.
pub fn style(ctx: &egui::Context) {
    use egui::{CornerRadius, Stroke, TextStyle};
    install_fonts(ctx);
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PANEL;
    visuals.window_fill = Color32::from_rgb(14, 24, 40);
    visuals.extreme_bg_color = NAVY;
    visuals.faint_bg_color = Color32::from_rgb(20, 32, 50);
    visuals.selection.bg_fill = Color32::from_rgb(40, 110, 130);
    visuals.selection.stroke = Stroke::new(1.0, GOLD);
    visuals.window_corner_radius = CornerRadius::same(14);
    visuals.menu_corner_radius = CornerRadius::same(10);
    visuals.window_stroke = Stroke::new(2.0, GOLD_DIM);
    visuals.hyperlink_color = TEAL;
    let round = CornerRadius::same(8);
    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.corner_radius = round;
    }
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(44, 62, 84));
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.inactive.bg_fill = NAVY_RAISED;
    visuals.widgets.inactive.weak_bg_fill = NAVY_RAISED;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, GOLD_DIM);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.hovered.bg_fill = NAVY_HOVER;
    visuals.widgets.hovered.weak_bg_fill = NAVY_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, GOLD);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    visuals.widgets.active.bg_fill = TEAL_DIM;
    visuals.widgets.active.weak_bg_fill = TEAL_DIM;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, GOLD);
    ctx.set_visuals(visuals.clone());
    ctx.set_visuals_of(egui::Theme::Dark, visuals);
    ctx.all_styles_mut(|style| {
        let f = |size: f32| FontId::new(size, FontFamily::Proportional);
        style.text_styles.insert(TextStyle::Body, f(15.0));
        style.text_styles.insert(TextStyle::Button, f(15.0));
        style.text_styles.insert(TextStyle::Small, f(12.5));
        style.text_styles.insert(TextStyle::Heading, display(24.0));
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.interact_size.y = 28.0;
    });
}

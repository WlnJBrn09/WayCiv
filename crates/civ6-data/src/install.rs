//! Locating the user's Civilization VI install and building the gameplay
//! database for the standard ruleset with every installed leader pack.

use crate::{GameplayDatabase, Result, build_base, modinfo};
use std::path::{Path, PathBuf};

/// `UpdateDatabase` actions applied for the standard ruleset, in the order the
/// game's modding framework applies them. Game modes and scenarios are left
/// out. Packs that are not installed are skipped.
pub const STANDARD_RULESET_ACTIONS: &[(&str, &str)] = &[
    ("aztec_montezuma", "AztecMontezumaGameplay"),
    ("poland_jadwiga", "PolandJadwigaGameplay"),
    ("australia", "AustraliaGameplay"),
    ("macedonia_persia", "MacedoniaGameplay"),
    ("nubia_amanitore", "NubiaGameplay"),
    ("indonesia_khmer", "IndonesiaKhmerGameplay"),
    ("vikingslandmarks", "VIKINGSLANDMARKS_DATA"),
    ("catherinedemedici", "CatherineDeMediciGameplay"),
    ("teddyroosevelt", "TeddyRooseveltGameplay"),
    ("ethiopia", "EthiopiaGameplay"),
    ("byzantium_gaul", "ByzantiumGaulGameplay"),
    ("babylon", "BabylonGameplay"),
    ("babylon", "BabylonGameplay_XP1"),
    ("grancolombia_maya", "GranColombiaMayaGameplay"),
    ("portugal", "PortugalGameplay"),
    ("kublaikhan_vietnam", "KublaiKhanVietnamGameplay"),
];

/// The game's `steamassets` directory: `$CIV6_DIR/steamassets` when set,
/// otherwise the default Steam library location.
pub fn steam_assets() -> Result<PathBuf> {
    let root = match std::env::var_os("CIV6_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => {
            let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
            Path::new(&home).join(".local/share/Steam/steamapps/common/Sid Meier's Civilization VI")
        }
    };
    let assets = root.join("steamassets");
    if assets.join("base/assets/gameplay/data").is_dir() {
        Ok(assets)
    } else {
        Err(format!(
            "Civilization VI not found at {} (set CIV6_DIR to the game's install directory)",
            root.display()
        )
        .into())
    }
}

/// Builds the standard-ruleset database with every installed leader pack.
pub fn build_standard(assets: &Path) -> Result<GameplayDatabase> {
    let mut database = build_base(assets)?;
    for (dlc, action) in STANDARD_RULESET_ACTIONS {
        let dir = assets.join("dlc").join(dlc);
        if !dir.is_dir() {
            continue;
        }
        database.apply_action(&modinfo::find_action(&dir, action)?)?;
    }
    Ok(database)
}

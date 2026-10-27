//! WayCiv's game engine: a Civilization-style turn flow with WayCiv's own
//! content (`wayciv-data`) and simplified rules. No UI; see the `wayciv`
//! crate for that.

pub mod actions;
pub mod additions;
pub mod ai;
pub mod air;
pub mod barbarians;
pub mod city;
pub mod combat;
pub mod diplomacy;
pub mod earth;
pub mod effects;
pub mod eras;
pub mod events;
pub mod great_people;
pub mod history;
pub mod hex;
pub mod intelligence;
pub mod length;
pub mod loyalty;
pub mod map;
pub mod movement;
pub mod policy_cards;
pub mod religion;
pub mod research;
pub mod rivalry;
pub mod rng;
pub mod rules;
pub mod setup;
pub mod state;
pub mod trade;
pub mod turn;
pub mod victory;
pub mod villages;

pub use hex::{Grid, Pos};
pub use length::GameLength;
pub use map::{CityId, MapSize, MapType, PlayerId};
pub use rules::Rules;
pub use setup::new_game;
pub use state::{Game, Production, Settings, UnitId};

/// Loads WayCiv's content and builds the rules.
pub fn load_rules() -> wayciv_data::Result<std::sync::Arc<Rules>> {
    Ok(std::sync::Arc::new(Rules::new(wayciv_data::Content::load()?)))
}

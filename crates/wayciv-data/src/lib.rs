//! WayCiv's game content: terrain, resources, civilizations and leaders,
//! units, buildings, wonders, districts, technologies, governments,
//! policies, beliefs, great people, and projects.
//!
//! Everything is WayCiv's own data, written in the TOML files under
//! `data/` and compiled into the game. `Content::load` parses it, fills in
//! the descriptions of effects, and checks that every id it refers to
//! exists.

pub mod content;
pub mod effect;

pub use content::*;
pub use effect::Effect;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

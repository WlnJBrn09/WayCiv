//! The Civ VI-on-Switch style HUD: shared drawing helpers plus one module per
//! piece of screen furniture.
//!
//! * `theme`      plates, rings, round buttons and other shared widgets
//! * `glyph`      vector symbols
//! * `topbar`     yields, population, turn, year, era and the menu buttons
//! * `trackers`   Research, Civic and AI circular trackers
//! * `leaders`    the leader ribbon and the Leaders panel (L)
//! * `notify`     the Notifications panel (N) and transient toasts
//! * `minimap`    the minimap
//! * `turn`       the big circular Next Turn button
//! * `unit_panel` the bottom-left unit panel
//! * `widgets`    labelled action pills and round tool buttons
//! * `city_panel` the right-hand city panel
//! * `cards`      cards, grids and buttons the menus are built from
//! * `menus`      Religion, Government, Great People, City-States, Victory

pub mod cards;
pub mod city_panel;
pub mod glyph;
pub mod keys;
pub mod leaders;
pub mod menus;
pub mod minimap;
pub mod notify;
pub mod theme;
pub mod topbar;
pub mod trackers;
pub mod turn;
pub mod unit_panel;
pub mod widgets;

/// Height of the top bar.
pub const TOP_BAR: f32 = 54.0;
/// Distance from the screen edge to floating HUD elements.
pub const MARGIN: f32 = 12.0;

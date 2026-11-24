//! Keyboard shortcuts.
//!
//! Windows and panels: `N` notifications, `L` leaders, `S` research (tech
//! tree), `A` AI tree, `R` religion, `G` government, `P` great
//! people, `E` city-states and envoys, `V` victory, `T` eras and the
//! timeline, `K` map key. Pressing a
//! window's key again, or `Esc`, closes it; opening one closes the others.
//!
//! Units and turns: `Enter` next turn / resolve the next blocker, `Space`
//! skip, `F` fortify, `Z` sleep, `B` found a city, `U` or `.` next unit.
//! Arrow keys pan the map (see `map_view`); `+` and `-` zoom it.

use crate::hud::Session;
use crate::screens::Panel;
use egui::Key;
use wayciv_core::actions::UnitAction;

/// (key, label, panel) for every window hotkey, in menu order.
pub const PANEL_KEYS: [(Key, &str, Panel); 10] = [
    (Key::N, "N", Panel::Notifications),
    (Key::L, "L", Panel::Leaders),
    (Key::S, "S", Panel::Tech),
    (Key::A, "A", Panel::Ai),
    (Key::R, "R", Panel::Religion),
    (Key::G, "G", Panel::Government),
    (Key::P, "P", Panel::GreatPeople),
    (Key::E, "E", Panel::CityStates),
    (Key::V, "V", Panel::Victory),
    (Key::T, "T", Panel::Eras),
];

pub fn handle(s: &mut Session, ctx: &egui::Context) {
    if ctx.egui_wants_keyboard_input() || crate::screens::modal_up(s) {
        return;
    }
    let pressed = |key: Key| {
        ctx.input(|i| {
            i.key_pressed(key) && !i.modifiers.ctrl && !i.modifiers.alt && !i.modifiers.command
        })
    };
    for (key, _, panel) in PANEL_KEYS {
        if pressed(key) {
            // The AI tree only exists once Artificial Intelligence is known.
            if panel == Panel::Ai && !s.game.ai_tree_unlocked(s.me()) {
                s.toast("Research Artificial Intelligence to open the AI tree.");
                continue;
            }
            s.windows.toggle(panel);
        }
    }
    if pressed(Key::K) {
        s.windows.legend = !s.windows.legend;
    }
    if pressed(Key::F1) {
        s.windows.toggle(Panel::Settings);
    }
    if pressed(Key::Y) {
        s.map_settings.show_yields = !s.map_settings.show_yields;
        s.toast(if s.map_settings.show_yields {
            "Tile yields shown."
        } else {
            "Tile yields hidden."
        });
    }
    if pressed(Key::H) {
        s.map_settings.grid = s.map_settings.grid.next();
        s.toast(format!("Hex grid: {}.", s.map_settings.grid.label()));
    }
    for (key, lens) in [
        (Key::Num4, crate::render::yields::MapLens::Settler),
        (Key::Num5, crate::render::yields::MapLens::Empire),
        (Key::Num6, crate::render::yields::MapLens::Religion),
        (Key::Num7, crate::render::yields::MapLens::Resources),
    ] {
        if pressed(key) {
            s.map_settings.lens = if s.map_settings.lens == lens {
                crate::render::yields::MapLens::None
            } else {
                lens
            };
        }
    }
    if pressed(Key::Enter) {
        s.end_turn_clicked();
    }
    if pressed(Key::Period) || pressed(Key::U) {
        s.next_unit();
    }
    if pressed(Key::Plus) || pressed(Key::Equals) {
        s.cam.target_zoom = (s.cam.target_zoom * 1.3).min(70.0);
        s.cam.zoom_anchor = None;
    }
    if pressed(Key::Minus) {
        s.cam.target_zoom = (s.cam.target_zoom / 1.3).max(8.0);
        s.cam.zoom_anchor = None;
    }
    if pressed(Key::Escape) {
        if s.placing.is_some() || s.city_targeting {
            s.placing = None;
            s.city_targeting = false;
        } else if s.windows.any_open() {
            s.windows.close_all();
        } else if s.map_settings.lens != crate::render::yields::MapLens::None {
            s.map_settings.lens = crate::render::yields::MapLens::None;
        } else {
            s.select_unit(None);
            s.select_city(None);
        }
    }
    if let Some(unit) = s.selected_unit {
        if pressed(Key::Space) {
            s.game.skip(unit);
            s.next_unit();
        }
        if pressed(Key::F) {
            s.game.fortify(unit);
            s.next_unit();
        }
        if pressed(Key::Z) && s.game.unit_actions(unit).contains(&UnitAction::Sleep) {
            let r = s.game.perform(unit, &UnitAction::Sleep);
            s.result(r);
            s.next_unit();
        }
        if pressed(Key::B) && s.game.can_found_city(unit) {
            let r = s.game.found_city(unit);
            s.result(r);
            s.refresh_selection();
        }
    }
}

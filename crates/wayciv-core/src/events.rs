//! Animation events: what just happened on the map, in order, for a front
//! end to play back. The engine only records them (`Game::emit`); nothing in
//! the rules reads them, and a game nobody drains keeps only the newest.

use crate::combat::AttackKind;
use crate::hex::Pos;
use crate::map::{CityId, PlayerId};
use crate::state::{Game, Unit, UnitId};

/// Events kept when nobody takes them (AI-only games, tests).
const MAX_EVENTS: usize = 4096;

/// What finished building, for a sparkle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Built {
    Building,
    District,
    Wonder,
    Project,
}

/// Who is striking: a unit (as it stood before the fight) or a city.
#[derive(Clone, Debug)]
pub enum Striker {
    Unit(Box<Unit>),
    City(CityId, Pos),
}

#[derive(Clone, Debug)]
pub enum Event {
    /// A unit stepped from `from` onto the adjacent tile `to`.
    Moved { unit: UnitId, from: Pos, to: Pos },
    /// An attack on `target`. `defender` is the unit that took it, as it
    /// stood before the fight (none for a city); `dealt` and `taken` are the
    /// damage to the defender and to a melee attacker.
    Attack {
        striker: Striker,
        target: Pos,
        kind: AttackKind,
        defender: Option<Box<Unit>>,
        dealt: i32,
        taken: i32,
    },
    /// A unit was killed; `unit` is its last state.
    Died { unit: Box<Unit> },
    /// A unit appeared (trained, or spawned).
    Spawned { unit: UnitId, pos: Pos },
    CityFounded { city: CityId, owner: PlayerId, pos: Pos },
    CityCaptured { city: CityId, owner: PlayerId, pos: Pos },
    /// Something finished building at `pos`.
    Built { pos: Pos, what: Built },
    /// A builder improved the tile it stands on (`unit` as it was).
    Improved { unit: Box<Unit>, pos: Pos },
}

impl Game {
    /// Records an animation event.
    pub fn emit(&mut self, event: Event) {
        if self.events.len() >= MAX_EVENTS {
            self.events.drain(..MAX_EVENTS / 2);
        }
        self.events.push(event);
    }

    /// The events since the last call.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_events_survive_the_cap() {
        let Ok(rules) = crate::load_rules() else {
            eprintln!("skipped: Civilization VI is not installed");
            return;
        };
        let mut game = crate::new_game(rules, crate::Settings::default());
        assert!(game.events.is_empty(), "setup events are dropped");
        for i in 0..MAX_EVENTS + 10 {
            game.emit(Event::Spawned {
                unit: i as UnitId,
                pos: Pos::default(),
            });
        }
        let events = game.take_events();
        assert!(events.len() <= MAX_EVENTS);
        assert!(matches!(
            events.last(),
            Some(Event::Spawned { unit, .. }) if *unit as usize == MAX_EVENTS + 9
        ));
        assert!(game.events.is_empty());
    }
}

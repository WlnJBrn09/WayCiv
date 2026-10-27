//! Eras, era score, and ages. The world's era follows the calendar. During
//! an era, historic moments (founding cities, meeting civilizations,
//! building wonders, firsts in the world...) earn era score; when the next
//! era arrives, each civilization's score decides its age: Dark, Normal,
//! Golden, or Heroic (a Golden Age right after a Dark one). Every age after
//! the first comes with dedications: in a Golden or Heroic Age their
//! bonuses apply, otherwise each earns +1 era score for its kind of moment.

use crate::map::PlayerId;
use crate::state::Game;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Age {
    #[default]
    Normal,
    Golden,
    Dark,
    Heroic,
}

impl Age {
    pub fn name(self) -> &'static str {
        match self {
            Age::Normal => "Normal Age",
            Age::Golden => "Golden Age",
            Age::Dark => "Dark Age",
            Age::Heroic => "Heroic Age",
        }
    }

    /// Golden and Heroic ages apply their dedications' bonuses.
    pub fn is_golden(self) -> bool {
        matches!(self, Age::Golden | Age::Heroic)
    }

    /// Loyalty per turn in every city.
    pub fn loyalty(self) -> i32 {
        match self {
            Age::Normal => 0,
            Age::Golden | Age::Heroic => 3,
            Age::Dark => -3,
        }
    }
}

/// A historic moment on a civilization's timeline.
#[derive(Clone, Debug)]
pub struct Moment {
    pub turn: u32,
    pub era: usize,
    pub text: String,
    pub score: i32,
}

#[derive(Clone, Debug, Default)]
pub struct EraState {
    /// Era score earned in the current era.
    pub score: i32,
    pub age: Age,
    /// Dedications for the current era (indices into `content.dedications`).
    pub dedications: Vec<usize>,
    /// Dedications still to choose this era.
    pub pending: usize,
    pub moments: Vec<Moment>,
    /// Once-only moments already earned ("pop10:12", "district:3", ...).
    pub firsts: HashSet<String>,
    /// Each finished era: (era, score earned, the age it led to).
    pub past: Vec<(usize, i32, Age)>,
}

/// Era score values for moments.
pub mod score {
    pub const CITY: i32 = 1;
    pub const MEET: i32 = 1;
    pub const NATURAL_WONDER: i32 = 2;
    pub const WONDER: i32 = 4;
    pub const PANTHEON: i32 = 1;
    pub const RELIGION: i32 = 3;
    pub const GREAT_PERSON: i32 = 2;
    pub const CAMP: i32 = 1;
    pub const CAPTURE: i32 = 2;
    pub const CAPTURE_CAPITAL: i32 = 4;
    pub const POP_10: i32 = 1;
    pub const WORLD_FIRST: i32 = 2;
    pub const TRADE_ROUTE: i32 = 1;
    pub const SPACE: i32 = 3;
    pub const VILLAGE: i32 = 1;
    pub const WON_WAR: i32 = 2;
}

impl Game {
    /// The era the calendar is in (index into `content.eras`).
    pub fn world_era_now(&self) -> usize {
        let year = self.year();
        let eras = &self.rules.content.eras;
        eras.iter()
            .rposition(|e| e.start_year <= year)
            .unwrap_or(0)
    }

    pub fn era_name(&self, era: usize) -> &str {
        self.rules
            .content
            .eras
            .get(era)
            .map(|e| e.name.as_str())
            .unwrap_or("")
    }

    /// The current era's first and last turns.
    pub fn era_span(&self) -> (u32, u32) {
        let eras = &self.rules.content.eras;
        let start = self.turn_of_year(eras[self.world_era].start_year);
        let end = eras
            .get(self.world_era + 1)
            .map(|e| self.turn_of_year(e.start_year))
            .unwrap_or_else(|| self.settings.length.pace_turns());
        (start, end.max(start + 1))
    }

    /// Era score needed to avoid a Dark Age and to reach a Golden Age this
    /// era: more for longer eras and bigger empires. A Golden Age is harder
    /// to keep; a Dark Age makes the next Golden Age easier.
    pub fn era_thresholds(&self, player: PlayerId) -> (i32, i32) {
        let (start, end) = self.era_span();
        // The era's length at the 300-turn pace the rules are balanced for.
        let length = (end - start) as f32 / self.speed();
        let cities = self.player_cities(player).count() as i32;
        let dark = (length * 0.25) as i32 + cities / 4;
        let golden = dark + 6 + (length * 0.2) as i32;
        let shift = match self.players[player].era.age {
            Age::Golden | Age::Heroic => 3,
            Age::Dark => -3,
            Age::Normal => 0,
        };
        ((dark + shift).max(1), golden + shift)
    }

    /// The age `player` is heading for with its current era score.
    pub fn projected_age(&self, player: PlayerId) -> Age {
        let (dark, golden) = self.era_thresholds(player);
        let e = &self.players[player].era;
        if e.score >= golden {
            if e.age == Age::Dark {
                Age::Heroic
            } else {
                Age::Golden
            }
        } else if e.score < dark {
            Age::Dark
        } else {
            Age::Normal
        }
    }

    /// Records a historic moment for a major civilization.
    pub(crate) fn moment(&mut self, player: PlayerId, text: impl Into<String>, score: i32) {
        if !self.players.get(player).is_some_and(|p| p.is_major()) {
            return;
        }
        let text = text.into();
        let (turn, era) = (self.turn, self.world_era);
        let e = &mut self.players[player].era;
        e.score += score;
        e.moments.push(crate::eras::Moment {
            turn,
            era,
            text: text.clone(),
            score,
        });
        if score > 0 {
            self.notify(
                player,
                format!("Historic moment: {text} (+{score} era score)"),
                None,
            );
        }
    }

    /// A moment that counts once per civilization (`key`). Returns whether
    /// it was new.
    pub(crate) fn first_moment(
        &mut self,
        player: PlayerId,
        key: &str,
        text: impl Into<String>,
        score: i32,
    ) -> bool {
        if !self.players.get(player).is_some_and(|p| p.is_major())
            || !self.players[player].era.firsts.insert(key.to_string())
        {
            return false;
        }
        self.moment(player, text, score);
        true
    }

    /// Claims a "first in the world" (`key`). Returns whether it was new.
    pub(crate) fn world_first(&mut self, player: PlayerId, key: &str, text: &str) -> bool {
        if !self.players.get(player).is_some_and(|p| p.is_major())
            || !self.world_firsts.insert(key.to_string())
        {
            return false;
        }
        self.moment(player, format!("First in the world: {text}"), score::WORLD_FIRST);
        let civ = self.players[player].civ_name.clone();
        for p in 0..self.players.len() {
            if p != player && self.players[p].is_major() && self.players[p].relations[player].met
            {
                self.notify(p, format!("{civ} is the first in the world: {text}."), None);
            }
        }
        true
    }

    /// +1 era score for a dedication's `kind` of moment (outside a Golden
    /// Age).
    pub(crate) fn dedication_moment(&mut self, player: PlayerId, kind: &str, text: &str) {
        let Some(p) = self.players.get(player) else {
            return;
        };
        if !p.is_major() || p.era.age.is_golden() {
            return;
        }
        let rules = self.rules.clone();
        if let Some(&d) = p
            .era
            .dedications
            .iter()
            .find(|&&d| rules.content.dedications[d].moment == kind)
        {
            let name = &rules.content.dedications[d].name;
            self.moment(player, format!("{text} ({name})"), 1);
        }
    }

    /// When the calendar enters a new era, every civilization's era score
    /// sets its new age and it chooses dedications.
    pub(crate) fn process_era(&mut self) {
        let now = self.world_era_now();
        if now <= self.world_era {
            return;
        }
        let ended = self.world_era;
        let ages: Vec<(PlayerId, Age)> = (0..self.players.len())
            .filter(|&p| self.players[p].alive && self.players[p].is_major())
            .map(|p| (p, self.projected_age(p)))
            .collect();
        self.world_era = now;
        let era_name = self.era_name(now).to_string();
        for &(p, age) in &ages {
            let e = &mut self.players[p].era;
            e.past.push((ended, e.score, age));
            e.score = 0;
            e.age = age;
            e.dedications.clear();
            e.pending = if age == Age::Heroic { 2 } else { 1 };
            self.notify(p, format!("The {era_name} begins: you enter a {}.", age.name()), None);
        }
        // Everyone hears who reached a Golden or fell into a Dark Age.
        for &(p, age) in &ages {
            if age == Age::Normal {
                continue;
            }
            let civ = self.players[p].civ_name.clone();
            for o in 0..self.players.len() {
                if o != p && self.players[o].is_major() && self.players[o].relations[p].met {
                    self.notify(o, format!("{civ} enters a {}.", age.name()), None);
                }
            }
        }
    }

    /// Dedications `player` can choose (all of them, minus ones it has).
    pub fn dedication_options(&self, player: PlayerId) -> Vec<usize> {
        let e = &self.players[player].era;
        (0..self.rules.content.dedications.len())
            .filter(|d| !e.dedications.contains(d))
            .collect()
    }

    pub fn choose_dedication(&mut self, player: PlayerId, dedication: usize) -> Result<(), String> {
        if self.players[player].era.pending == 0 {
            return Err("No dedication to choose now.".into());
        }
        if !self.dedication_options(player).contains(&dedication) {
            return Err("That dedication isn't available.".into());
        }
        let e = &mut self.players[player].era;
        e.dedications.push(dedication);
        e.pending -= 1;
        Ok(())
    }

    /// The AI's dedication: its focus decides.
    pub(crate) fn ai_dedication(&mut self, player: PlayerId) {
        while self.players[player].era.pending > 0 {
            let focus = self.personality(player).focus.clone();
            let wanted = match focus.as_str() {
                "war" => "DEDICATION_TO_ARMS",
                "science" => "DEDICATION_FREE_INQUIRY",
                "culture" => "DEDICATION_PEN_BRUSH_VOICE",
                "faith" => "DEDICATION_EXODUS",
                "trade" => "DEDICATION_REFORM_COINAGE",
                _ => "DEDICATION_MONUMENTALITY",
            };
            let options = self.dedication_options(player);
            let content = &self.rules.content;
            let pick = options
                .iter()
                .copied()
                .find(|&d| content.dedications[d].id == wanted)
                .or_else(|| options.first().copied());
            match pick {
                Some(d) => {
                    let _ = self.choose_dedication(player, d);
                }
                None => break,
            }
        }
    }

    /// Effects of the player's dedications that apply now.
    pub(crate) fn dedication_effects(&self, player: PlayerId) -> Vec<&wayciv_data::Effect> {
        let e = &self.players[player].era;
        if !e.age.is_golden() {
            return Vec::new();
        }
        e.dedications
            .iter()
            .flat_map(|&d| &self.rules.content.dedications[d].effects)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Settings, load_rules, new_game};

    #[test]
    fn eras_bring_ages_and_dedications() {
        let rules = load_rules().unwrap();
        let mut g = new_game(
            rules,
            Settings {
                seed: 3,
                opponents: 2,
                city_states: 0,
                ..Default::default()
            },
        );
        let me = g.human();
        g.players[me].era.score = 1000;
        // Jump the calendar to the next era.
        while g.world_era_now() == 0 {
            g.turn += 1;
        }
        g.process_era();
        assert_eq!(g.world_era, 1);
        let e = &g.players[me].era;
        assert_eq!(e.age, Age::Golden);
        assert_eq!(e.pending, 1);
        assert_eq!(e.score, 0);
        let d = g.dedication_options(me)[0];
        g.choose_dedication(me, d).unwrap();
        assert!(!g.dedication_effects(me).is_empty());
        // A rival with no score falls into a Dark Age.
        let rival = (0..g.players.len())
            .find(|&p| p != me && g.players[p].is_major())
            .unwrap();
        assert_eq!(g.players[rival].era.age, Age::Dark);
    }
}

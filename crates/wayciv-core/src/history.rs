//! The world's story as it unfolds: statistics recorded every turn for the
//! end-of-game graphs, warnings when a rival nears victory, and the race
//! for wonders (announcements, and Gold back for a wonder lost to a rival).

use crate::map::{CityId, PlayerId};
use crate::state::{Game, Production};

/// One major civilization's numbers on one turn.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerStats {
    pub alive: bool,
    pub score: i32,
    pub science: i32,
    pub culture: i32,
    pub military: i32,
    pub cities: i32,
    pub techs: i32,
}

#[derive(Clone, Debug, Default)]
pub struct TurnStats {
    pub turn: u32,
    /// By player id (non-majors stay at their defaults).
    pub players: Vec<PlayerStats>,
}

/// Victory progress (0..1) at which rivals are warned.
const WARNINGS: [f32; 3] = [0.5, 0.75, 0.9];

impl Game {
    pub(crate) fn record_history(&mut self) {
        let mut stats = TurnStats {
            turn: self.turn,
            players: vec![PlayerStats::default(); self.players.len()],
        };
        for p in 0..self.players.len() {
            if !self.players[p].is_major() || !self.players[p].alive {
                continue;
            }
            let income = self.income(p);
            stats.players[p] = PlayerStats {
                alive: true,
                score: self.score(p),
                science: income.yields[crate::rules::SCIENCE].round() as i32,
                culture: income.yields[crate::rules::CULTURE].round() as i32,
                military: self.military_strength(p),
                cities: self.player_cities(p).count() as i32,
                techs: self.players[p].techs.iter().filter(|t| **t).count() as i32,
            };
        }
        self.history.push(stats);
    }

    /// Warns everyone who has met a civilization when it passes 50%, 75%,
    /// and 90% of the way to a victory.
    pub(crate) fn victory_warnings(&mut self) {
        let majors: Vec<PlayerId> = self.alive_majors().map(|p| p.id).collect();
        for &p in &majors {
            for v in self.victory_progress(p) {
                if v.name == "Score" {
                    continue;
                }
                let level = WARNINGS.iter().filter(|&&w| v.frac >= w).count() as u8;
                let warned = self.players[p].warned.get(v.name).copied().unwrap_or(0);
                if level <= warned {
                    continue;
                }
                self.players[p].warned.insert(v.name.to_string(), level);
                let percent = (WARNINGS[level as usize - 1] * 100.0) as i32;
                let civ = self.players[p].civ_name.clone();
                for &o in &majors {
                    if o == p {
                        self.notify(
                            o,
                            format!("You are {percent}% of the way to a {} Victory. Your rivals have noticed.", v.name),
                            None,
                        );
                    } else if self.players[o].relations[p].met {
                        let urgency = match level {
                            3 => "Stop them now!",
                            2 => "Time is running short.",
                            _ => "Keep an eye on them.",
                        };
                        self.notify(
                            o,
                            format!("Warning: {civ} is {percent}% of the way to a {} Victory. {urgency}", v.name),
                            None,
                        );
                        // Other leaders grow wary of a front-runner.
                        if level >= 2 {
                            self.players[o].relations[p].opinion -= 5;
                        }
                    }
                }
            }
        }
    }

    /// Who else is building `wonder`, among civilizations `viewer` has met:
    /// (player, city, fraction done).
    pub fn wonder_rivals(&self, viewer: PlayerId, wonder: usize) -> Vec<(PlayerId, CityId, f32)> {
        let key = Self::production_key(&Production::Wonder(wonder, Default::default()));
        let mut rivals = Vec::new();
        for c in self.cities.values() {
            if c.owner == viewer
                || !self.players[viewer].relations[c.owner].met
                || !matches!(c.production, Some(Production::Wonder(w, _)) if w == wonder)
            {
                continue;
            }
            let cost = self
                .production_cost(c.owner, &Production::Wonder(wonder, c.pos))
                .max(1);
            let done = c.progress.get(&key).copied().unwrap_or(0) as f32 / cost as f32;
            rivals.push((c.owner, c.id, done.min(1.0)));
        }
        rivals
    }

    /// Announces the first time a civilization starts a wonder.
    pub(crate) fn announce_wonder_start(&mut self, city: CityId) {
        let Some(Production::Wonder(w, _)) = self.cities[&city].production else {
            return;
        };
        let owner = self.cities[&city].owner;
        if !self.players[owner].is_major() || !self.wonder_starts.insert((owner, w)) {
            return;
        }
        let civ = self.players[owner].civ_name.clone();
        let wonder = self.rules.content.buildings[w].name.clone();
        for p in 0..self.players.len() {
            if p != owner && self.players[p].is_major() && self.players[p].relations[owner].met {
                self.notify(p, format!("{civ} has begun building {wonder}."), None);
            }
        }
    }

    /// When `wonder` is completed, every other city working on it gets its
    /// progress back as Gold.
    pub(crate) fn wonder_lost(&mut self, wonder: usize, builder: PlayerId) {
        let key = Self::production_key(&Production::Wonder(wonder, Default::default()));
        let ids: Vec<CityId> = self
            .cities
            .values()
            .filter(|c| c.progress.get(&key).copied().unwrap_or(0) > 0)
            .map(|c| c.id)
            .collect();
        let name = self.rules.content.buildings[wonder].name.clone();
        let civ = self.players[builder].civ_name.clone();
        for id in ids {
            let c = self.cities.get_mut(&id).unwrap();
            let progress = c.progress.remove(&key).unwrap_or(0);
            if matches!(c.production, Some(Production::Wonder(w, _)) if w == wonder) {
                c.production = None;
            }
            let (owner, city_name, pos) = (c.owner, c.name.clone(), c.pos);
            self.players[owner].gold += progress;
            self.notify(
                owner,
                format!("{civ} finished {name} first. {city_name}'s {progress} Production became {progress} Gold."),
                Some(pos),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Production, Settings, load_rules, new_game};

    #[test]
    fn losing_a_wonder_race_refunds_gold() {
        let mut g = new_game(
            load_rules().unwrap(),
            Settings {
                seed: 4,
                opponents: 1,
                city_states: 0,
                ..Default::default()
            },
        );
        let (a, b) = (0, 1);
        let pa = g.player_units(a).next().unwrap().pos;
        let pb = g.player_units(b).next().unwrap().pos;
        let ca = g.found_city_at(a, pa, None);
        let cb = g.found_city_at(b, pb, None);
        let w = (0..g.rules.content.buildings.len())
            .find(|&i| g.rules.content.buildings[i].wonder)
            .unwrap();
        let key = crate::Game::production_key(&Production::Wonder(w, pa));
        g.cities.get_mut(&ca).unwrap().progress.insert(key, 120);
        g.cities.get_mut(&ca).unwrap().production = Some(Production::Wonder(w, pa));
        let gold = g.players[a].gold;
        g.wonder_lost(w, b);
        assert_eq!(g.players[a].gold, gold + 120);
        assert!(g.cities[&ca].production.is_none());
        let _ = cb;
    }
}

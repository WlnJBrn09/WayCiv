//! Victory conditions: science, culture, domination, religious, and score.

use crate::map::PlayerId;
use crate::state::{Game, Victory};

impl Game {
    /// Space race projects needed for a science victory.
    pub fn space_projects(&self) -> Vec<usize> {
        self.rules
            .content
            .projects
            .iter()
            .enumerate()
            .filter(|(_, p)| p.space_race)
            .map(|(i, _)| i)
            .collect()
    }

    /// Visiting tourists from `player` to `other` vs `other`'s domestic tourists.
    pub fn tourists(&self, player: PlayerId, other: PlayerId) -> (i64, i64) {
        let visiting = self.players[player].tourism_total / 150;
        let domestic = self.players[other].culture_total / 100;
        (visiting, domestic)
    }

    pub fn victory_progress(&self, player: PlayerId) -> Vec<(String, String)> {
        let p = &self.players[player];
        let space = self.space_projects();
        let done = space
            .iter()
            .filter(|i| p.space_projects.contains(i))
            .count();
        let others: Vec<PlayerId> = self
            .alive_majors()
            .map(|x| x.id)
            .filter(|&x| x != player)
            .collect();
        let culture_won = others.iter().filter(|&&o| {
            let (visiting, domestic) = self.tourists(player, o);
            visiting > domestic
        });
        let capitals = self.original_capitals_held(player);
        let majors = self.players.iter().filter(|x| x.is_major()).count();
        let religious = others
            .iter()
            .filter(|&&o| p.religion.is_some() && self.dominant_religion(o) == p.religion)
            .count();
        vec![
            (
                "Science".into(),
                format!("{done} of {} space projects", space.len()),
            ),
            (
                "Culture".into(),
                format!(
                    "More visiting tourists than {} of {} civilizations' domestic tourists",
                    culture_won.count(),
                    others.len()
                ),
            ),
            (
                "Domination".into(),
                format!("{capitals} of {majors} original capitals held"),
            ),
            (
                "Religion".into(),
                if p.religion.is_some() {
                    format!(
                        "Your religion dominates {religious} of {} other civilizations",
                        others.len()
                    )
                } else {
                    "No religion founded".into()
                },
            ),
            (
                "Score".into(),
                match self.turn_limit() {
                    Some(limit) => format!(
                        "{} points; turn {} of {limit}",
                        self.score(player),
                        self.turn
                    ),
                    None => format!(
                        "{} points; turn {} (no turn limit)",
                        self.score(player),
                        self.turn
                    ),
                },
            ),
        ]
    }

    pub fn original_capitals_held(&self, player: PlayerId) -> usize {
        self.cities
            .values()
            .filter(|c| c.owner == player && c.original_capital_of.is_some())
            .count()
    }

    pub(crate) fn check_victory(&mut self) {
        if self.winner.is_some() {
            return;
        }
        let majors: Vec<PlayerId> = self
            .players
            .iter()
            .filter(|p| p.is_major())
            .map(|p| p.id)
            .collect();
        let space = self.space_projects();
        for &p in &majors {
            if !self.players[p].alive {
                continue;
            }
            let others: Vec<PlayerId> = majors
                .iter()
                .copied()
                .filter(|&o| o != p && self.players[o].alive)
                .collect();
            if !space.is_empty()
                && space
                    .iter()
                    .all(|i| self.players[p].space_projects.contains(i))
            {
                return self.declare_winner(p, Victory::Science);
            }
            if !others.is_empty()
                && others.iter().all(|&o| {
                    let (visiting, domestic) = self.tourists(p, o);
                    visiting > domestic
                })
            {
                return self.declare_winner(p, Victory::Culture);
            }
            if self.original_capitals_held(p) == majors.len() || others.is_empty() && self.turn > 1
            {
                return self.declare_winner(p, Victory::Domination);
            }
            let religion = self.players[p].religion;
            if religion.is_some()
                && self.dominant_religion(p) == religion
                && !others.is_empty()
                && others
                    .iter()
                    .all(|&o| self.dominant_religion(o) == religion)
            {
                return self.declare_winner(p, Victory::Religious);
            }
        }
        if self.turn_limit().is_some_and(|limit| self.turn > limit)
            && let Some(best) = majors
                .iter()
                .copied()
                .filter(|&p| self.players[p].alive)
                .max_by_key(|&p| self.score(p))
        {
            self.declare_winner(best, Victory::Score);
        }
        let human = self.human();
        if !self.players[human].alive
            && self.winner.is_none()
            && let Some(best) = majors
                .iter()
                .copied()
                .filter(|&p| self.players[p].alive)
                .max_by_key(|&p| self.score(p))
        {
            self.declare_winner(best, Victory::Domination);
        }
    }

    fn declare_winner(&mut self, player: PlayerId, victory: Victory) {
        let name = self.players[player].civ_name.clone();
        self.winner = Some((player, victory.clone()));
        let kind = match victory {
            Victory::Science => "Science",
            Victory::Culture => "Culture",
            Victory::Domination => "Domination",
            Victory::Religious => "Religious",
            Victory::Score => "Score",
        };
        for p in 0..self.players.len() {
            self.notify(p, format!("{name} has won a {kind} Victory!"), None);
        }
    }
}

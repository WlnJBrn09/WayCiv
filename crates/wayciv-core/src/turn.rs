//! The turn: AI moves, then every player's end-of-turn processing, then the
//! calendar advances (unlocking governments, policies, and other
//! year-gated content).

use crate::map::{CityId, PlayerId};
use crate::research::Tree;
use crate::rules::{FAITH, GOLD, SCIENCE};
use crate::state::{Game, UnitId};

/// What stops the human from ending the turn (the End Turn button).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Blocker {
    ChooseResearch,
    ChooseProduction(CityId),
    ChoosePantheon,
    ChooseAiResearch,
    UnitNeedsOrders(UnitId),
}

/// Per-turn totals for the top bar.
#[derive(Clone, Debug, Default)]
pub struct Income {
    pub yields: [f32; 6],
    pub maintenance: i32,
    pub net_gold: i32,
    pub tourism: i32,
}

impl Game {
    pub fn income(&self, player: PlayerId) -> Income {
        let mut income = Income::default();
        let content = &self.rules.content;
        for c in self.player_cities(player) {
            let y = self.city_report(c.id).yields;
            for i in 0..6 {
                income.yields[i] += y[i];
            }
            income.maintenance += c
                .buildings
                .iter()
                .map(|&b| content.buildings[b].maintenance)
                .sum::<i32>();
        }
        let units: i32 = self
            .player_units(player)
            .map(|u| self.unit_type(u).maintenance)
            .sum();
        income.maintenance += units;
        income.net_gold = income.yields[GOLD].round() as i32 - income.maintenance;
        income.tourism = self.tourism(player);
        income
    }

    pub fn tourism(&self, player: PlayerId) -> i32 {
        let wonders = self
            .player_cities(player)
            .flat_map(|c| &c.buildings)
            .filter(|&&b| self.rules.content.buildings[b].wonder)
            .count() as i32;
        let flight = if self.players[player].has_tech(&self.rules, "TECH_FLIGHT") {
            2
        } else {
            1
        };
        (self.players[player].great_works * 3 + wonders * 2) * flight
    }

    pub fn blockers(&self, player: PlayerId) -> Vec<Blocker> {
        let p = &self.players[player];
        let mut blockers = Vec::new();
        if p.research.is_none()
            && (0..p.techs.len()).any(|i| self.researchable(player, Tree::Tech, i))
        {
            blockers.push(Blocker::ChooseResearch);
        }
        if p.ai_research.is_none()
            && self.compute(player) > 0.0
            && (0..p.ai_techs.len()).any(|i| self.researchable(player, Tree::Ai, i))
        {
            blockers.push(Blocker::ChooseAiResearch);
        }
        if self.can_choose_pantheon(player)
            && !self.beliefs_of_class("BELIEF_CLASS_PANTHEON").is_empty()
        {
            blockers.push(Blocker::ChoosePantheon);
        }
        for c in self.player_cities(player) {
            if c.production.is_none() {
                blockers.push(Blocker::ChooseProduction(c.id));
            }
        }
        for u in self.player_units(player) {
            if u.moves > 0 && !u.fortified && !u.sleeping && !u.skipped && u.path.is_empty() {
                blockers.push(Blocker::UnitNeedsOrders(u.id));
            }
        }
        blockers
    }

    pub fn blocker_text(&self, blocker: &Blocker) -> String {
        match blocker {
            Blocker::ChooseResearch => "Choose Research".into(),
            Blocker::ChoosePantheon => "Choose Pantheon".into(),
            Blocker::ChooseAiResearch => "Choose AI Research".into(),
            Blocker::ChooseProduction(_) => "Choose Production".into(),
            Blocker::UnitNeedsOrders(_) => "Unit Needs Orders".into(),
        }
    }

    /// Ends the human's turn: AI players act, then the world advances.
    pub fn end_turn(&mut self) {
        if self.winner.is_some() {
            return;
        }
        let players: Vec<PlayerId> = (0..self.players.len())
            .filter(|&p| self.players[p].alive && !self.players[p].human)
            .collect();
        for p in players {
            self.ai_turn(p);
        }
        for p in 0..self.players.len() {
            if self.players[p].alive {
                self.process_player(p);
            }
        }
        self.spread_pressure();
        self.process_loyalty();
        let before = self.year();
        self.turn += 1;
        self.process_year_unlocks(before);
        for p in 0..self.players.len() {
            if self.players[p].alive {
                self.refresh_units(p);
                self.update_visibility(p);
                self.check_boosts(p, crate::research::Trigger::Poll);
            }
        }
        // The human's queued moves continue at the start of their turn.
        let human = self.human();
        let moving: Vec<UnitId> = self
            .player_units(human)
            .filter(|u| !u.path.is_empty())
            .map(|u| u.id)
            .collect();
        for id in moving {
            self.continue_path(id);
        }
        self.check_victory();
        if self.notifications.len() > 400 {
            let drop = self.notifications.len() - 400;
            self.notifications.drain(..drop);
        }
    }

    fn process_player(&mut self, player: PlayerId) {
        let cities: Vec<CityId> = self.player_cities(player).map(|c| c.id).collect();
        let income = self.income(player);
        for id in cities {
            self.process_city(id);
        }
        if !self.players[player].is_major() {
            return;
        }
        self.advance(player, Tree::Tech, income.yields[SCIENCE]);
        self.process_ai(player);
        let p = &mut self.players[player];
        p.gold += income.net_gold;
        p.faith += income.yields[FAITH].round() as i32;
        p.culture_total += income.yields[crate::rules::CULTURE].round() as i64;
        p.tourism_total += income.tourism as i64;
        if p.gold < 0 {
            p.gold = 0;
            // Bankrupt: disband the weakest military unit.
            let weakest = self
                .player_units(player)
                .filter(|u| self.is_military(u))
                .min_by_key(|u| self.unit_type(u).combat)
                .map(|u| u.id);
            if let Some(id) = weakest {
                self.units.remove(&id);
                self.notify(
                    player,
                    "A unit was disbanded: you can't pay its upkeep.",
                    None,
                );
            }
        }
        self.process_great_people(player);
        self.process_influence(player);
        self.process_trade(player);
    }

    fn refresh_units(&mut self, player: PlayerId) {
        let ids: Vec<UnitId> = self.player_units(player).map(|u| u.id).collect();
        for id in ids {
            let (pos, full_moves, untouched) = {
                let u = &self.units[&id];
                (
                    u.pos,
                    self.unit_type(u).moves,
                    u.moves == self.unit_type(u).moves && !u.attacked,
                )
            };
            let in_city = self.city_at(pos).is_some_and(|c| c.owner == player);
            let friendly = self.map.tile(pos).owner == Some(player);
            let u = self.units.get_mut(&id).unwrap();
            if untouched && u.hp < 100 {
                u.hp = (u.hp
                    + if in_city {
                        20
                    } else if friendly {
                        15
                    } else {
                        10
                    })
                .min(100);
            }
            if u.fortified {
                u.fortify_turns += 1;
            }
            if u.sleeping && u.hp >= 100 && untouched && !u.fortified {
                // Sleeping units stay asleep; fortified units keep fortifying.
            }
            u.moves = full_moves;
            u.attacked = false;
            u.skipped = false;
        }
    }

    pub fn score(&self, player: PlayerId) -> i32 {
        let p = &self.players[player];
        let pop: i32 = self.player_cities(player).map(|c| c.population).sum();
        let cities = self.player_cities(player).count() as i32;
        let wonders = self
            .player_cities(player)
            .flat_map(|c| &c.buildings)
            .filter(|&&b| self.rules.content.buildings[b].wonder)
            .count() as i32;
        let techs = p.techs.iter().filter(|&&t| t).count() as i32;
        let great_people = self
            .player_units(player)
            .filter(|u| u.great_person.is_some())
            .count() as i32
            + p.great_works;
        pop + cities * 5
            + wonders * 15
            + techs * 3
            + great_people * 5
            + i32::from(p.religion.is_some()) * 10
    }

    /// Content whose calendar year falls after `before` and by now: new
    /// governments, policies, and year-gated units, buildings, wonders,
    /// districts, and improvements. Each major gets a free policy change
    /// and an envoy when governments or policies unlock.
    fn process_year_unlocks(&mut self, before: i32) {
        let now = self.year();
        if now <= before {
            return;
        }
        let content = &self.rules.content;
        let new = |y: i32| y > before && y <= now;
        let governments: Vec<String> = content
            .governments
            .iter()
            .filter(|g| new(g.unlock_year))
            .map(|g| g.name.clone())
            .collect();
        let policies: Vec<String> = content
            .policies
            .iter()
            .filter(|p| new(p.unlock_year))
            .map(|p| p.name.clone())
            .collect();
        let mut other: Vec<String> = Vec::new();
        let year_of = |y: Option<i32>| y.is_some_and(new);
        other.extend(
            content
                .units
                .iter()
                .filter(|u| year_of(u.unlock_year) && u.unique_to.is_none())
                .map(|u| u.name.clone()),
        );
        other.extend(
            content
                .buildings
                .iter()
                .filter(|b| year_of(b.unlock_year) && b.unique_to.is_none())
                .map(|b| b.name.clone()),
        );
        other.extend(
            content
                .districts
                .iter()
                .filter(|d| year_of(d.unlock_year) && d.unique_to.is_none())
                .map(|d| d.name.clone()),
        );
        if governments.is_empty() && policies.is_empty() && other.is_empty() {
            return;
        }
        let mut parts = Vec::new();
        if !governments.is_empty() {
            parts.push(format!("Government: {}.", governments.join(", ")));
        }
        if !policies.is_empty() {
            parts.push(format!("Policies: {}.", policies.join(", ")));
        }
        if !other.is_empty() {
            parts.push(format!("Now available: {}.", other.join(", ")));
        }
        let text = format!("{}: {}", crate::length::year_label(now), parts.join(" "));
        let policy_news = !governments.is_empty() || !policies.is_empty();
        for p in 0..self.players.len() {
            if !self.players[p].alive || !self.players[p].is_major() {
                continue;
            }
            if policy_news {
                self.players[p].policy_change_free = true;
                self.players[p].envoys_available += 1;
            }
            self.notify(p, text.clone(), None);
        }
    }
}

//! Loyalty, simplified from Civ VI: every city within 9 tiles pulls on a
//! city in proportion to its population (closer pulls harder). Your own
//! cities hold it, foreign cities pull it away. Capitals, garrisons,
//! Monuments, and happy citizens add loyalty; occupation and unhappiness
//! take it away. A city at 0 loyalty defects to the civilization pulling on
//! it hardest. City-states' cities never defect.

use crate::map::{CityId, PlayerId};
use crate::state::Game;
use std::collections::HashMap;

/// How far a city's pull reaches.
pub const PRESSURE_RANGE: i32 = 9;
pub const MAX_LOYALTY: i32 = 100;

#[derive(Clone, Debug, Default)]
pub struct LoyaltyReport {
    /// Change per turn, already clamped.
    pub per_turn: i32,
    /// Named contributions, for the city panel.
    pub lines: Vec<(String, i32)>,
    /// The foreign civilization pulling hardest, if any.
    pub strongest_foreign: Option<PlayerId>,
}

impl Game {
    pub fn loyalty_applies(&self, city: CityId) -> bool {
        self.cities
            .get(&city)
            .is_some_and(|c| self.players[c.owner].is_major())
    }

    pub fn loyalty_report(&self, id: CityId) -> LoyaltyReport {
        let mut report = LoyaltyReport::default();
        let Some(city) = self.cities.get(&id) else {
            return report;
        };
        if !self.loyalty_applies(id) {
            return report;
        }
        let grid = self.map.grid;
        let owner = city.owner;
        let mut own = 0.0f32;
        let mut foreign: HashMap<PlayerId, f32> = HashMap::new();
        for other in self.cities.values() {
            let d = grid.distance(city.pos, other.pos);
            if d > PRESSURE_RANGE || self.players[other.owner].is_barbarian() {
                continue;
            }
            let pull = other.population as f32 * (10 - d) as f32 / 10.0;
            if other.owner == owner {
                own += pull;
            } else {
                *foreign.entry(other.owner).or_insert(0.0) += pull;
            }
        }
        let foreign_total: f32 = foreign.values().sum();
        let pressure = ((own - foreign_total) * 0.5).clamp(-15.0, 10.0).round() as i32;
        report
            .lines
            .push(("Nearby population (yours vs. foreign)".into(), pressure));
        report.strongest_foreign = foreign
            .iter()
            .filter(|(p, _)| self.players[**p].is_major() && self.players[**p].alive)
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(p, _)| *p);
        if city.capital {
            report.lines.push(("Capital".into(), 5));
        }
        let garrisoned = self
            .ground_units_at(city.pos)
            .any(|u| u.owner == owner && self.is_military(u));
        if garrisoned {
            report.lines.push(("Garrison".into(), 2));
        }
        if self
            .rules
            .idx("BUILDING_MONUMENT")
            .is_some_and(|m| city.has_building(m))
        {
            report.lines.push(("Monument".into(), 1));
        }
        let (amenities, needed) = self.amenities(city);
        let mood = (amenities - needed).clamp(-6, 2);
        if mood != 0 {
            report.lines.push((
                if mood > 0 {
                    "Happy citizens"
                } else {
                    "Unhappy citizens"
                }
                .into(),
                mood,
            ));
        }
        let age = self.players[owner].era.age;
        if age.loyalty() != 0 {
            report.lines.push((age.name().into(), age.loyalty()));
        }
        if city.founder != owner {
            report.lines.push(("Occupied".into(), -3));
            if city.original_capital_of.is_some_and(|p| p != owner) {
                report.lines.push(("Occupied capital".into(), -2));
            }
        }
        report.per_turn = report
            .lines
            .iter()
            .map(|(_, v)| v)
            .sum::<i32>()
            .clamp(-20, 20);
        report
    }

    /// End of turn: loyalty moves; disloyal cities defect.
    pub(crate) fn process_loyalty(&mut self) {
        let ids: Vec<CityId> = self.cities.keys().copied().collect();
        for id in ids {
            if !self.loyalty_applies(id) {
                continue;
            }
            let report = self.loyalty_report(id);
            let c = self.cities.get_mut(&id).unwrap();
            let before = c.loyalty;
            c.loyalty = (c.loyalty + report.per_turn).clamp(0, MAX_LOYALTY);
            let (after, owner, name, pos) = (c.loyalty, c.owner, c.name.clone(), c.pos);
            for threshold in [50, 25] {
                if before > threshold && after <= threshold {
                    self.notify(
                        owner,
                        format!(
                            "{name} is losing loyalty ({after}). Garrison it or grow nearby cities."
                        ),
                        Some(pos),
                    );
                }
            }
            if after == 0
                && report.per_turn < 0
                && let Some(to) = report.strongest_foreign
            {
                self.defect(id, to);
            }
        }
    }

    /// A city at 0 loyalty joins `to`. Units of the old owner inside leave
    /// for their nearest city.
    pub(crate) fn defect(&mut self, city: CityId, to: PlayerId) {
        let (old, pos, name, was_capital) = {
            let c = &self.cities[&city];
            (c.owner, c.pos, c.name.clone(), c.capital)
        };
        self.destroy_based_aircraft(city, old);
        self.cancel_routes(|_, r| r.origin != city && r.destination != city);
        let palace = self.rules.idx("BUILDING_PALACE");
        {
            let c = self.cities.get_mut(&city).unwrap();
            c.owner = to;
            c.loyalty = MAX_LOYALTY / 2;
            c.production = None;
            c.capital = false;
            c.buildings.retain(|&b| Some(b) != palace);
        }
        for t in self.map.tiles.iter_mut() {
            if t.owning_city == Some(city) {
                t.owner = Some(to);
            }
        }
        let refuge = self
            .player_cities(old)
            .filter(|c| c.id != city)
            .min_by_key(|c| self.map.grid.distance(c.pos, pos))
            .map(|c| c.pos);
        let stranded: Vec<_> = self
            .ground_units_at(pos)
            .filter(|u| u.owner == old)
            .map(|u| u.id)
            .collect();
        for id in stranded {
            match refuge {
                Some(p) => self.units.get_mut(&id).unwrap().pos = p,
                None => {
                    self.units.remove(&id);
                }
            }
        }
        if was_capital {
            self.players[old].capital = None;
            let next = self
                .player_cities(old)
                .max_by_key(|c| c.population)
                .map(|c| c.id);
            if let Some(next) = next {
                self.cities.get_mut(&next).unwrap().capital = true;
                self.players[old].capital = Some(next);
            }
        }
        let (old_civ, new_civ) = (
            self.players[old].civ_name.clone(),
            self.players[to].civ_name.clone(),
        );
        self.notify(
            old,
            format!("{name} has lost all loyalty and joined {new_civ}!"),
            Some(pos),
        );
        self.notify(
            to,
            format!("{name} has declared loyalty to you, leaving {old_civ}!"),
            Some(pos),
        );
        self.assign_tiles(city);
        self.check_elimination(old);
        self.update_visibility(to);
        self.update_visibility(old);
    }
}

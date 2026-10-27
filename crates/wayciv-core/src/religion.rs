//! Pantheons, religions, missionaries, and religious pressure.

use crate::map::{CityId, PlayerId};
use crate::state::{Game, UnitId};

pub const PANTHEON_COST: i32 = 25;

impl Game {
    pub fn beliefs_of_class(&self, class: &str) -> Vec<usize> {
        let taken: Vec<usize> = self
            .players
            .iter()
            .flat_map(|p| p.beliefs.iter().copied().chain(p.pantheon))
            .collect();
        self.rules
            .content
            .beliefs
            .iter()
            .enumerate()
            .filter(|(i, b)| b.class == class && !taken.contains(i))
            .map(|(i, _)| i)
            .collect()
    }

    pub fn can_choose_pantheon(&self, player: PlayerId) -> bool {
        let p = &self.players[player];
        p.is_major() && p.pantheon.is_none() && p.faith >= PANTHEON_COST
    }

    pub fn choose_pantheon(&mut self, player: PlayerId, belief: usize) -> Result<(), String> {
        if !self.can_choose_pantheon(player)
            || !self
                .beliefs_of_class("BELIEF_CLASS_PANTHEON")
                .contains(&belief)
        {
            return Err("That pantheon can't be chosen.".into());
        }
        let p = &mut self.players[player];
        p.faith -= PANTHEON_COST;
        p.pantheon = Some(belief);
        let name = self.rules.content.beliefs[belief].name.clone();
        self.notify(player, format!("Your people now worship {name}."), None);
        self.moment(player, format!("Adopted the pantheon {name}"), crate::eras::score::PANTHEON);
        self.check_boosts(player, crate::research::Trigger::Pantheon);
        Ok(())
    }

    /// Religions nobody has founded yet.
    pub fn free_religions(&self) -> Vec<usize> {
        self.rules
            .content
            .religions
            .iter()
            .enumerate()
            .filter(|(i, r)| {
                !r.pantheon
                    && !self.religions.contains_key(i)
                    && r.id != "RELIGION_CUSTOM_1"
                    && !r.id.starts_with("RELIGION_CUSTOM")
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn default_beliefs(&self) -> (usize, usize) {
        let founder = self
            .beliefs_of_class("BELIEF_CLASS_FOUNDER")
            .first()
            .copied()
            .unwrap_or(0);
        let follower = self
            .beliefs_of_class("BELIEF_CLASS_FOLLOWER")
            .first()
            .copied()
            .unwrap_or(0);
        (founder, follower)
    }

    /// A Great Prophet on a Holy Site its owner controls, with religions left.
    pub fn can_found_religion(&self, unit: UnitId) -> bool {
        let Some(u) = self.units.get(&unit) else {
            return false;
        };
        let rules = &self.rules;
        let tile = self.map.tile(u.pos);
        self.unit_type(u).found_religion
            && self.players[u.owner].religion.is_none()
            && tile.owner == Some(u.owner)
            && tile.district.is_some_and(|d| {
                let d = &rules.content.districts[d];
                d.id == "DISTRICT_HOLY_SITE" || d.replaces.as_deref() == Some("DISTRICT_HOLY_SITE")
            })
            && !self.free_religions().is_empty()
    }

    pub fn found_religion(
        &mut self,
        unit: UnitId,
        religion: usize,
        founder: usize,
        follower: usize,
    ) -> Result<(), String> {
        if !self.can_found_religion(unit) || !self.free_religions().contains(&religion) {
            return Err("A religion can't be founded here.".into());
        }
        let u = self.units.remove(&unit).unwrap();
        let city = self.map.tile(u.pos).owning_city.ok_or("No city.")?;
        let p = &mut self.players[u.owner];
        p.religion = Some(religion);
        p.beliefs.push(founder);
        p.beliefs.push(follower);
        self.religions.insert(religion, (u.owner, city));
        let c = self.cities.get_mut(&city).unwrap();
        c.followers.clear();
        c.followers.insert(religion, c.population);
        let name = self.rules.content.religions[religion].name.clone();
        let founder_name = self.players[u.owner].civ_name.clone();
        self.moment(u.owner, format!("Founded {name}"), crate::eras::score::RELIGION);
        for p in 0..self.players.len() {
            if self.players[p].is_major() {
                let text = if p == u.owner {
                    format!("You founded {name}!")
                } else {
                    format!("{founder_name} founded {name}.")
                };
                self.notify(p, text, None);
            }
        }
        self.check_boosts(u.owner, crate::research::Trigger::Religion);
        Ok(())
    }

    /// Faith price of a religious unit in a city (needs a founded religion).
    pub fn faith_cost(&self, unit: usize) -> i32 {
        self.scaled((self.rules.content.units[unit].cost.max(50) as f32 * 1.5) as i32)
    }

    pub fn faith_units(&self, city: CityId) -> Vec<usize> {
        let c = &self.cities[&city];
        let p = &self.players[c.owner];
        // As in Civ VI, religious units are bought in cities with a Holy Site.
        let holy_site = c.districts.iter().any(|d| {
            let d = &self.rules.content.districts[d.district];
            d.id == "DISTRICT_HOLY_SITE" || d.replaces.as_deref() == Some("DISTRICT_HOLY_SITE")
        });
        if p.religion.is_none() || !holy_site {
            return Vec::new();
        }
        self.rules
            .content
            .units
            .iter()
            .enumerate()
            .filter(|(_, u)| {
                u.purchase_yield.as_deref() == Some("YIELD_FAITH")
                    && u.can_train
                    && !u.found_religion
                    && !u.id.starts_with("UNIT_GREAT_")
            })
            .filter(|(_, u)| u.religious_strength > 0 || u.spread_charges > 0)
            .filter(|(_, u)| p.meets(&self.rules, u.prereq_tech.as_deref()) && self.reached(u.unlock_year))
            .filter(|(_, u)| u.unique_to.is_none())
            .map(|(i, _)| i)
            .collect()
    }

    pub fn buy_with_faith(&mut self, city: CityId, unit: usize) -> Result<(), String> {
        if !self.faith_units(city).contains(&unit) {
            return Err("Can't buy that here.".into());
        }
        let cost = self.faith_cost(unit);
        let (owner, pos) = (self.cities[&city].owner, self.cities[&city].pos);
        if self.players[owner].faith < cost {
            return Err(format!("Not enough Faith ({cost} needed)."));
        }
        self.players[owner].faith -= cost;
        let id = self.spawn_exact(owner, unit, pos);
        let religion = self.players[owner].religion;
        if let Some(u) = self.units.get_mut(&id) {
            u.religion = religion;
        }
        Ok(())
    }

    /// Spends a charge converting the city the unit stands in or next to.
    pub fn spread_religion(&mut self, unit: UnitId) -> Result<(), String> {
        let u = self.units.get(&unit).ok_or("No such unit.")?.clone();
        let religion = u.religion.ok_or("This unit carries no religion.")?;
        let city = self
            .cities
            .values()
            .filter(|c| self.map.grid.distance(c.pos, u.pos) <= 1)
            .min_by_key(|c| self.map.grid.distance(c.pos, u.pos))
            .map(|c| c.id)
            .ok_or("No city here to convert.")?;
        if self.religions.values().any(|&(_, holy)| holy == city) {
            return Err("Holy cities can't be converted.".into());
        }
        let c = self.cities.get_mut(&city).unwrap();
        let converted = (c.population + 2) / 3;
        let mut left = converted;
        for (r, n) in c.followers.iter_mut() {
            if *r != religion && left > 0 {
                let taken = (*n).min(left);
                *n -= taken;
                left -= taken;
            }
        }
        let total: i32 = c.followers.values().sum();
        let pop = c.population;
        *c.followers.entry(religion).or_insert(0) += converted.min(pop - total + converted).max(0);
        let followers = c.followers.get(&religion).copied().unwrap_or(0);
        if followers > pop {
            c.followers.insert(religion, pop);
        }
        let u = self.units.get_mut(&unit).unwrap();
        u.charges -= 1;
        u.moves = 0;
        if u.charges <= 0 {
            self.units.remove(&unit);
        }
        Ok(())
    }

    /// Each turn: cities near a religion's cities slowly convert.
    pub(crate) fn spread_pressure(&mut self) {
        let grid = self.map.grid;
        let sources: Vec<(crate::hex::Pos, usize, bool)> = self
            .cities
            .values()
            .filter_map(|c| {
                c.majority_religion().map(|r| {
                    (
                        c.pos,
                        r,
                        self.religions
                            .get(&r)
                            .is_some_and(|(_, holy)| *holy == c.id),
                    )
                })
            })
            .collect();
        let ids: Vec<CityId> = self.cities.keys().copied().collect();
        for id in ids {
            let pos = self.cities[&id].pos;
            let mut pressure: Vec<(usize, i32)> = Vec::new();
            for (source, religion, holy) in &sources {
                let d = grid.distance(pos, *source);
                if d <= 10 {
                    let amount = if *holy { 2 } else { 1 } + i32::from(d == 0);
                    match pressure.iter_mut().find(|(r, _)| r == religion) {
                        Some(entry) => entry.1 += amount,
                        None => pressure.push((*religion, amount)),
                    }
                }
            }
            // A founder's own cities lean toward its religion; holy cities
            // never convert.
            let owner = self.cities[&id].owner;
            if let Some(home) = self.players[owner].religion {
                match pressure.iter_mut().find(|(r, _)| *r == home) {
                    Some(entry) => entry.1 += 2,
                    None => pressure.push((home, 2)),
                }
            }
            if self.religions.values().any(|&(_, holy)| holy == id) {
                continue;
            }
            let c = self.cities.get_mut(&id).unwrap();
            for (religion, amount) in pressure {
                let accumulated = c.pressure.entry(religion).or_insert(0);
                *accumulated += amount;
                if *accumulated >= 40 {
                    *accumulated = 0;
                    let total: i32 = c.followers.values().sum();
                    if total < c.population {
                        *c.followers.entry(religion).or_insert(0) += 1;
                    } else if let Some((&other, _)) = c
                        .followers
                        .iter()
                        .filter(|(r, n)| **r != religion && **n > 0)
                        .max_by_key(|(_, n)| **n)
                    {
                        *c.followers.get_mut(&other).unwrap() -= 1;
                        *c.followers.entry(religion).or_insert(0) += 1;
                    }
                }
            }
            // Followers can't exceed population after starvation.
            let pop = c.population;
            let mut total: i32 = c.followers.values().sum();
            while total > pop {
                if let Some((_, n)) = c
                    .followers
                    .iter_mut()
                    .filter(|(_, n)| **n > 0)
                    .min_by_key(|(_, n)| **n)
                {
                    *n -= 1;
                }
                total -= 1;
            }
        }
    }

    /// The religion that is the majority in more than half of `player`'s cities.
    pub fn dominant_religion(&self, player: PlayerId) -> Option<usize> {
        let cities: Vec<_> = self.player_cities(player).collect();
        if cities.is_empty() {
            return None;
        }
        let mut counts: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for c in &cities {
            if let Some(r) = c.majority_religion() {
                *counts.entry(r).or_insert(0) += 1;
            }
        }
        counts
            .into_iter()
            .find(|(_, n)| n * 2 > cities.len())
            .map(|(r, _)| r)
    }
}

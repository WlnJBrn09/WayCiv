//! Great people: points, recruitment, and (simplified) activation effects.

use crate::map::PlayerId;
use crate::research::Tree;
use crate::additions::*;
use crate::state::{Game, Production, Unit, UnitId};

impl Game {
    /// Great person points per turn for each class.
    pub fn great_person_rates(&self, player: PlayerId) -> Vec<(usize, i32)> {
        let rules = &self.rules;
        let content = &rules.content;
        let mut rates: Vec<(usize, i32)> = Vec::new();
        let mut add = |class: usize, n: i32| match rates.iter_mut().find(|(c, _)| *c == class) {
            Some(entry) => entry.1 += n,
            None => rates.push((class, n)),
        };
        for c in self.player_cities(player) {
            for d in &c.districts {
                for (class, n) in &content.districts[d.district].great_person_points {
                    if let Some(i) = rules.idx(class) {
                        add(i, *n);
                    }
                }
            }
            for &b in &c.buildings {
                for (class, n) in &content.buildings[b].great_person_points {
                    if let Some(i) = rules.idx(class) {
                        add(i, *n);
                    }
                }
            }
        }
        for (class, n) in self.bonuses(player).great_person_points {
            add(class, n);
        }
        rates
    }

    /// The next great person available in a class, if any.
    pub fn next_great_person(&self, class: usize) -> Option<usize> {
        let content = &self.rules.content;
        let class_id = &content.great_person_classes[class].id;
        content
            .great_people
            .iter()
            .enumerate()
            .filter(|(i, g)| &g.class == class_id && !self.great_people_taken.contains(i))
            .min_by_key(|(_, g)| self.rules.era_index(&g.era))
            .map(|(i, _)| i)
    }

    pub fn great_person_cost(&self, class: usize) -> i32 {
        let content = &self.rules.content;
        let class_id = &content.great_person_classes[class].id;
        let recruited = self.great_people_taken.iter().filter(|&&g| &content.great_people[g].class == class_id).count() as i32;
        let era = self.next_great_person(class).map(|g| self.rules.era_index(&content.great_people[g].era)).unwrap_or(0);
        let base = content.eras.get(era).map(|e| e.great_person_base_cost).unwrap_or(60).max(30);
        base + recruited * base / 3
    }

    /// Accumulates points; recruits when a player reaches the cost.
    pub(crate) fn process_great_people(&mut self, player: PlayerId) {
        if !self.players[player].is_major() {
            return;
        }
        for (class, rate) in self.great_person_rates(player) {
            *self.players[player].great_person_points.entry(class).or_insert(0) += rate;
            let Some(person) = self.next_great_person(class) else { continue };
            let cost = self.great_person_cost(class);
            if self.players[player].great_person_points[&class] < cost {
                continue;
            }
            *self.players[player].great_person_points.get_mut(&class).unwrap() -= cost;
            self.great_people_taken.push(person);
            let content = &self.rules.content;
            let unit = content.great_person_classes[class].unit.clone();
            let name = content.great_people[person].name.clone();
            let class_name = content.great_person_classes[class].name.clone();
            let Some(capital) = self.players[player].capital.map(|c| self.cities[&c].pos) else { continue };
            if let Some(id) = self.spawn(player, &unit, capital)
                && let Some(u) = self.units.get_mut(&id) {
                    u.great_person = Some(person);
                }
            for p in 0..self.players.len() {
                if !self.players[p].is_major() {
                    continue;
                }
                let text = if p == player {
                    format!("{name} ({class_name}) has joined your civilization!")
                } else {
                    format!("{name} ({class_name}) was recruited by {}.", self.players[player].civ_name)
                };
                self.notify(p, text, Some(capital));
            }
        }
    }

    /// WayCiv's simplified great person effects, by class.
    pub fn great_person_effect(&self, person: usize) -> String {
        let content = &self.rules.content;
        match content.great_people[person].id.as_str() {
            ELON_MUSK => {
                return "Minor boosts to AI, rockets and solar: +25% toward your current AI research (or a Eureka for Artificial Intelligence), a Eureka for Rocketry (or +300 Production toward a space project here), and a free Solar Farm in this city.".into();
            }
            GWYNNE_SHOTWELL => {
                return "Big rocket boost: +1,500 Production toward the space race project this city is building (needs a Spaceport); otherwise Eurekas for Rocketry and Satellites.".into();
            }
            PALMER_LUCKEY => return "Deploys a flight of 3 Anduril Strike Drones here, and adds 50% toward Autonomous Systems in the AI tree.".into(),
            ILYA_SUTSKEVER => {
                return "Big AI boost: completes your current AI research (or the cheapest one available); before Artificial Intelligence, Eurekas for Computers and Artificial Intelligence.".into();
            }
            _ => {}
        }
        let class = content.great_people[person].class.as_str();
        match class {
            "GREAT_PERSON_CLASS_SCIENTIST" => "Eurekas for two technologies you haven't researched.".into(),
            "GREAT_PERSON_CLASS_ENGINEER" => "Adds 250 Production to what this city is building.".into(),
            "GREAT_PERSON_CLASS_MERCHANT" => "Grants 200 Gold.".into(),
            "GREAT_PERSON_CLASS_GENERAL" | "GREAT_PERSON_CLASS_ADMIRAL" => {
                "Every military unit within 3 tiles gains a level and fully heals.".into()
            }
            "GREAT_PERSON_CLASS_PROPHET" => "Founds a religion on a Holy Site.".into(),
            _ => "Creates a Great Work (+Tourism).".into(),
        }
    }

    pub fn activate_great_person(&mut self, unit: UnitId) -> Result<(), String> {
        let u = self.units.get(&unit).ok_or("No such unit.")?.clone();
        let person = u.great_person.ok_or("Not a great person.")?;
        let class = self.rules.content.great_people[person].class.clone();
        let owner = u.owner;
        let id = self.rules.content.great_people[person].id.clone();
        if matches!(id.as_str(), ELON_MUSK | GWYNNE_SHOTWELL | PALMER_LUCKEY | ILYA_SUTSKEVER) {
            self.modern_great_person(&id, &u)?;
            self.units.remove(&unit);
            let name = self.rules.content.great_people[person].name.clone();
            self.notify(owner, format!("{name} has been activated."), Some(u.pos));
            return Ok(());
        }
        match class.as_str() {
            "GREAT_PERSON_CLASS_SCIENTIST" => {
                let mut targets: Vec<usize> = (0..self.rules.content.techs.len())
                    .filter(|&i| !self.players[owner].techs[i] && !self.players[owner].tech_boosted[i])
                    .collect();
                targets.sort_by_key(|&i| self.rules.content.techs[i].cost);
                for i in targets.into_iter().take(2) {
                    self.boost(owner, Tree::Tech, i, 50);
                }
            }
            "GREAT_PERSON_CLASS_ENGINEER" => {
                let city = self.map.tile(u.pos).owning_city.filter(|c| self.cities[c].owner == owner).ok_or("Must be in your territory.")?;
                let item = self.cities[&city].production.clone().ok_or("The city isn't building anything.")?;
                *self.cities.get_mut(&city).unwrap().progress.entry(Self::production_key(&item)).or_insert(0) += 250;
            }
            "GREAT_PERSON_CLASS_MERCHANT" => self.players[owner].gold += 200,
            "GREAT_PERSON_CLASS_GENERAL" | "GREAT_PERSON_CLASS_ADMIRAL" => {
                let nearby: Vec<UnitId> = self
                    .player_units(owner)
                    .filter(|x| x.id != unit && self.is_military(x) && self.map.grid.distance(x.pos, u.pos) <= 3)
                    .map(|x| x.id)
                    .collect();
                for id in nearby {
                    let x = self.units.get_mut(&id).unwrap();
                    x.level += 1;
                    x.hp = 100;
                }
            }
            "GREAT_PERSON_CLASS_PROPHET" => return Err("Great Prophets found religions on a Holy Site.".into()),
            _ => self.players[owner].great_works += 1,
        }
        self.units.remove(&unit);
        let name = self.rules.content.great_people[person].name.clone();
        self.notify(owner, format!("{name} has been activated."), Some(u.pos));
        Ok(())
    }

    /// Elon Musk, Gwynne Shotwell, Palmer Luckey, and Ilya Sutskever.
    fn modern_great_person(&mut self, id: &str, u: &Unit) -> Result<(), String> {
        let owner = u.owner;
        let here = self.map.tile(u.pos).owning_city.filter(|c| self.cities[c].owner == owner);
        let tech = |g: &Game, t: &str| g.rules.tech(t);
        // The cheapest AI tree item available (or the current one).
        let ai_target = |g: &Game| {
            g.current_research(owner, Tree::Ai).or_else(|| {
                (0..g.rules.ai_tree.len()).filter(|&i| g.researchable(owner, Tree::Ai, i)).min_by_key(|&i| g.rules.ai_tree[i].cost)
            })
        };
        let space_project = |g: &Game, city: Option<crate::map::CityId>| {
            city.and_then(|c| match &g.cities[&c].production {
                Some(Production::Project(p)) if g.rules.content.projects[*p].space_race => Some((c, Production::Project(*p))),
                _ => None,
            })
        };
        match id {
            ELON_MUSK => {
                let city = here.ok_or("Must be in one of your cities' territory.")?;
                match ai_target(self) {
                    Some(i) if self.ai_tree_unlocked(owner) => self.boost(owner, Tree::Ai, i, 25),
                    _ => {
                        if let Some(i) = tech(self, TECH_AI) {
                            self.boost(owner, Tree::Tech, i, 40);
                        }
                    }
                }
                match (tech(self, "TECH_ROCKETRY"), space_project(self, Some(city))) {
                    (_, Some((c, item))) => *self.cities.get_mut(&c).unwrap().progress.entry(Self::production_key(&item)).or_insert(0) += 300,
                    (Some(i), None) => self.boost(owner, Tree::Tech, i, 40),
                    _ => {}
                }
                if let Some(solar) = self.rules.idx(SOLAR_FARM)
                    && !self.cities[&city].has_building(solar)
                {
                    self.cities.get_mut(&city).unwrap().buildings.push(solar);
                }
            }
            GWYNNE_SHOTWELL => match space_project(self, here) {
                Some((c, item)) => *self.cities.get_mut(&c).unwrap().progress.entry(Self::production_key(&item)).or_insert(0) += 1500,
                None => {
                    for t in ["TECH_ROCKETRY", "TECH_SATELLITES"] {
                        if let Some(i) = tech(self, t) {
                            self.boost(owner, Tree::Tech, i, 50);
                        }
                    }
                }
            },
            PALMER_LUCKEY => {
                let drone = self.rules.idx(STRIKE_DRONE).ok_or("No drones in this game.")?;
                let spots: Vec<_> = self
                    .map
                    .grid
                    .within(u.pos, 3)
                    .into_iter()
                    .filter(|&p| self.map.is_land(&self.rules, p) && !self.map.is_impassable(&self.rules, p))
                    .filter(|&p| !self.ground_units_at(p).any(|x| x.owner != owner || self.is_military(x)))
                    .take(DRONES_PER_BATCH)
                    .collect();
                for p in spots {
                    self.spawn_exact(owner, drone, p);
                }
                if let Some(i) = self.rules.ai_tech(AI_AUTONOMOUS_SYSTEMS)
                    && !self.players[owner].ai_techs[i]
                {
                    self.players[owner].ai_progress[i] += self.rules.ai_tree[i].cost / 2;
                }
            }
            ILYA_SUTSKEVER => match ai_target(self).filter(|_| self.ai_tree_unlocked(owner)) {
                Some(i) => {
                    let cost = self.research_cost(Tree::Ai, i);
                    self.players[owner].ai_progress[i] = cost;
                    self.complete(owner, Tree::Ai, i);
                    if self.players[owner].ai_research == Some(i) {
                        self.players[owner].ai_research = None;
                    }
                }
                None => {
                    for t in ["TECH_COMPUTERS", TECH_AI] {
                        if let Some(i) = tech(self, t) {
                            self.boost(owner, Tree::Tech, i, 50);
                        }
                    }
                }
            },
            _ => {}
        }
        self.update_visibility(owner);
        Ok(())
    }
}

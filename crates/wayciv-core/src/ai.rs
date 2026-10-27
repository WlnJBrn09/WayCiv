//! Computer players: majors and city-states.

use crate::actions::MIN_CITY_DISTANCE;
use crate::combat::AttackKind;
use crate::hex::Pos;
use crate::map::{CityId, PlayerId, site_score};
use crate::research::Tree;
use crate::rules::FOOD;
use crate::state::{Game, Production, UnitId};

impl Game {
    pub(crate) fn ai_turn(&mut self, player: PlayerId) {
        if self.players[player].is_barbarian() {
            self.barbarian_turn(player);
            return;
        }
        if self.players[player].is_city_state() {
            self.ai_city_state(player);
            return;
        }
        self.ai_research(player);
        self.ai_pick_policy(player);
        self.ai_dedication(player);
        self.ai_government(player);
        self.ai_religion(player);
        self.ai_diplomacy(player);
        self.ai_rivalry(player);
        self.ai_envoys(player);
        let cities: Vec<CityId> = self.player_cities(player).map(|c| c.id).collect();
        for city in &cities {
            self.ai_production(*city);
            self.ai_city_attack(*city);
        }
        let units: Vec<UnitId> = self.player_units(player).map(|u| u.id).collect();
        for unit in units {
            if self.units.contains_key(&unit) {
                self.ai_unit(unit);
            }
        }
        self.ai_spend(player);
    }

    fn ai_research(&mut self, player: PlayerId) {
        for tree in [Tree::Tech, Tree::Ai] {
            if self.current_research(player, tree).is_none()
                && let Some(next) = self.ai_pick_research(player, tree)
            {
                let _ = self.set_research(player, tree, next);
            }
        }
    }

    fn ai_government(&mut self, player: PlayerId) {
        let rules = self.rules.clone();
        let content = &rules.content;
        let tier = |g: usize| {
            content.governments[g]
                .slots
                .iter()
                .map(|(_, n)| n)
                .sum::<i32>()
        };
        // Slots count most; the leader's focus breaks ties and can keep an
        // older government it likes.
        let focus = self.personality(player).focus.clone();
        let liked = |g: usize| -> i32 {
            let id = content.governments[g].id.as_str();
            let likes: &[&str] = match focus.as_str() {
                "war" => &["GOVERNMENT_OLIGARCHY", "GOVERNMENT_MONARCHY", "GOVERNMENT_FASCISM"],
                "science" => &["GOVERNMENT_CLASSICAL_REPUBLIC", "GOVERNMENT_MERCHANT_REPUBLIC", "GOVERNMENT_DEMOCRACY"],
                "culture" => &["GOVERNMENT_CLASSICAL_REPUBLIC", "GOVERNMENT_MONARCHY", "GOVERNMENT_DEMOCRACY"],
                "faith" => &["GOVERNMENT_AUTOCRACY", "GOVERNMENT_THEOCRACY", "GOVERNMENT_FASCISM"],
                "trade" => &["GOVERNMENT_CLASSICAL_REPUBLIC", "GOVERNMENT_MERCHANT_REPUBLIC", "GOVERNMENT_DEMOCRACY"],
                "expansion" => &["GOVERNMENT_AUTOCRACY", "GOVERNMENT_MONARCHY", "GOVERNMENT_COMMUNISM"],
                _ => &[],
            };
            let reagan = content.leaders[self.players[player].leader].id == crate::rules::REAGAN
                && id == "GOVERNMENT_DEMOCRACY";
            i32::from(likes.contains(&id) || reagan) * 3
        };
        let value = |g: usize| tier(g) * 2 + liked(g);
        if let Some(best) = self
            .available_governments(player)
            .into_iter()
            .max_by_key(|&g| (value(g), g))
            && self.players[player]
                .government
                .is_none_or(|g| value(best) > value(g))
        {
            let _ = self.set_government(player, best);
        }
        if self.players[player].policy_change_free {
            let slots = self.policy_slots(player);
            let available = self.available_policies(player);
            let mut chosen = Vec::new();
            for (slot, count) in &slots {
                let wanted = if slot == "SLOT_WILDCARD" {
                    "SLOT_ECONOMIC"
                } else {
                    slot.as_str()
                };
                // Newest policies first: later ones are stronger.
                let mut picks: Vec<usize> = available
                    .iter()
                    .copied()
                    .filter(|&p| content.policies[p].slot == wanted && !chosen.contains(&p))
                    .collect();
                picks.sort_by_key(|&p| std::cmp::Reverse(content.policies[p].unlock_year));
                picks.truncate(*count as usize);
                chosen.extend(picks);
            }
            if self.set_policies(player, chosen).is_ok() {
                self.players[player].policy_change_free = false;
            }
        }
    }

    fn ai_religion(&mut self, player: PlayerId) {
        if self.can_choose_pantheon(player) {
            let options = self.beliefs_of_class("BELIEF_CLASS_PANTHEON");
            if let Some(&belief) = self.rng.pick(&options) {
                let _ = self.choose_pantheon(player, belief);
            }
        }
        if self.players[player].religion.is_some() && self.players[player].faith >= 300 {
            let city = self.player_cities(player).map(|c| c.id).next();
            if let Some(city) = city
                && let Some(&unit) = self.faith_units(city).first()
            {
                let _ = self.buy_with_faith(city, unit);
            }
        }
    }

    fn ai_diplomacy(&mut self, player: PlayerId) {
        let others: Vec<PlayerId> = (0..self.players.len())
            .filter(|&o| {
                o != player && self.players[o].alive && self.players[player].relations[o].met
            })
            .collect();
        for other in others {
            let other_is_cs = self.players[other].is_city_state();
            let at_war = self.players[player].at_war_with(other);
            let mine = self.military_strength(player);
            let theirs = self.military_strength(other).max(1);
            let opinion = self.opinion(player, other);
            if at_war {
                if !other_is_cs
                    && self.can_make_peace(player, other)
                    && self.ai_wants_peace(player, other)
                {
                    if self.players[other].human {
                        // Offers peace; the human answers on the leader screen.
                        let offered = self
                            .popups
                            .iter()
                            .any(|&(p, e)| p == player && e == crate::diplomacy::DiploEvent::MakePeaceFromAi);
                        if !offered && self.rng.chance(0.2) {
                            self.popups
                                .push((player, crate::diplomacy::DiploEvent::MakePeaceFromAi));
                        }
                    } else {
                        self.propose_peace(player, other);
                    }
                }
                continue;
            }
            let _ = (mine, theirs);
            // Wars are planned in `ai_rivalry`.
            let planning = self.players[player]
                .war_plan
                .as_ref()
                .is_some_and(|p| p.target == other);
            if planning {
                continue;
            }
            if !other_is_cs
                && opinion >= 25
                && !self.players[player].relations[other].friendship
                && !self.players[other].human
                && self.rng.chance(0.1)
            {
                self.propose_friendship(player, other);
            } else if !other_is_cs
                && opinion <= -30
                && !self.players[player].relations[other].denounced
                && self.rng.chance(0.05)
            {
                self.denounce(player, other);
                if self.players[other].human {
                    self.popups
                        .push((player, crate::diplomacy::DiploEvent::DenounceFromAi));
                }
            } else if !other_is_cs
                && opinion >= 25
                && self.players[other].human
                && !self.players[player].relations[other].friendship
                && self.rng.chance(0.03)
            {
                self.popups
                    .push((player, crate::diplomacy::DiploEvent::DeclareFriendFromAi));
            }
        }
    }

    fn ai_envoys(&mut self, player: PlayerId) {
        while self.players[player].envoys_available > 0 {
            let target = (0..self.players.len())
                .filter(|&s| {
                    self.players[s].is_city_state()
                        && self.players[s].alive
                        && self.players[player].relations[s].met
                })
                .max_by_key(|&s| {
                    let mine = self.players[player].envoys.get(&s).copied().unwrap_or(0);
                    (mine % 3 == 2) as i32 * 10 + mine
                });
            match target {
                Some(s) => {
                    let _ = self.send_envoy(player, s);
                }
                None => break,
            }
        }
    }

    /// Spends Gold above a reserve: troops when at war or preparing one,
    /// otherwise the current build or a cheap building.
    fn ai_spend(&mut self, player: PlayerId) {
        let cities: Vec<CityId> = self.player_cities(player).map(|c| c.id).collect();
        let reserve = 100 + 20 * cities.len() as i32;
        if self.players[player].gold <= reserve + 40 {
            return;
        }
        let content = self.rules.content.clone();
        let martial = self.major_wars(player) > 0 || self.players[player].war_plan.is_some();
        // No more troops than the war needs.
        let army = self
            .player_units(player)
            .filter(|u| self.is_military(u))
            .count();
        let martial = martial && army < (cities.len() * 2 + 4).min(36);
        // Candidates, gathered once: each city's current build, plus its
        // best land unit (in wartime) or its buildings (in peacetime).
        let mut options: Vec<(CityId, Production)> = Vec::new();
        for &city in &cities {
            if let Some(item) = self.cities[&city].production.clone() {
                options.push((city, item));
            }
            if martial {
                let unit = (0..content.units.len())
                    .filter(|&i| {
                        let u = &content.units[i];
                        u.domain == "DOMAIN_LAND"
                            && (u.combat > 0 || u.ranged_combat > 0)
                            && !u.found_city
                            && self.can_produce(city, &Production::Unit(i))
                    })
                    .max_by_key(|&i| content.units[i].combat.max(content.units[i].ranged_combat));
                if let Some(i) = unit {
                    options.push((city, Production::Unit(i)));
                }
            } else {
                for i in 0..content.buildings.len() {
                    if !content.buildings[i].wonder
                        && self.can_produce(city, &Production::Building(i))
                    {
                        options.push((city, Production::Building(i)));
                    }
                }
            }
        }
        for _ in 0..3 {
            let budget = self.players[player].gold - reserve;
            // Prefer what a city is building, then the cheapest.
            let best = options
                .iter()
                .enumerate()
                .filter_map(|(k, (city, item))| {
                    let price = self.purchase_cost(*city, item)?;
                    let current = self.cities[city].production.as_ref() == Some(item);
                    (price <= budget).then_some((price - i32::from(current) * 1000, k))
                })
                .min();
            let Some((_, k)) = best else {
                return;
            };
            let (city, item) = options.remove(k);
            if self.purchase(city, item).is_err() {
                return;
            }
        }
    }

    fn ai_production(&mut self, city: CityId) {
        if self.cities[&city].production.is_some() {
            return;
        }
        let owner = self.cities[&city].owner;
        if let Some(item) = self.ai_choose_production(city, owner) {
            let _ = self.set_production(city, item);
        }
    }

    fn ai_choose_production(&mut self, city: CityId, owner: PlayerId) -> Option<Production> {
        let content = &self.rules.content;
        let available = self.available_production(city);
        let cities = self.player_cities(owner).count();
        let military = self
            .player_units(owner)
            .filter(|u| self.is_military(u))
            .count();
        let settlers = self.player_units(owner).filter(|u| self.unit_type(u).found_city).count()
            + self.cities.values().filter(|c| c.owner == owner && matches!(c.production, Some(Production::Unit(i)) if content.units[i].found_city)).count();
        let builders = self
            .player_units(owner)
            .filter(|u| self.unit_type(u).build_charges > 0)
            .count();
        let at_war = self.real_wars(owner) > 0 || self.players[owner].war_plan.is_some();
        let target_cities = 6 + (self.turn / 60) as usize;

        let best_unit = |filter: &dyn Fn(&crate::rules::Unit) -> bool| -> Option<Production> {
            available
                .iter()
                .filter_map(|p| match p {
                    Production::Unit(i) if filter(&content.units[*i]) => Some((
                        *i,
                        content.units[*i]
                            .combat
                            .max(content.units[*i].ranged_combat),
                    )),
                    _ => None,
                })
                .max_by_key(|(_, s)| *s)
                .map(|(i, _)| Production::Unit(i))
        };
        let land_military = |u: &crate::rules::Unit| {
            u.domain == "DOMAIN_LAND" && (u.combat > 0 || u.ranged_combat > 0) && !u.found_city
        };
        // In peacetime, don't grow an army the treasury can't pay for.
        let broke = !at_war && self.income(owner).net_gold < 2;
        let wanted_military = if at_war {
            (cities * 2 + 4).min(36)
        } else if broke {
            2
        } else {
            (cities + 2).min(20)
        };

        if (military < 2 || (at_war && military < wanted_military))
            && let Some(item) = best_unit(&land_military)
        {
            return Some(item);
        }
        if cities + settlers < target_cities
            && settlers < 2
            && self.cities[&city].population >= 2
            && self.turn < 200
            && let Some(item) = best_unit(&|u| u.found_city)
        {
            return Some(item);
        }
        if builders < cities.div_ceil(2)
            && let Some(item) = best_unit(&|u| u.build_charges > 0)
        {
            return Some(item);
        }
        let traders = self
            .player_units(owner)
            .filter(|u| self.unit_type(u).make_trade_route)
            .count() as i32;
        if cities >= 2
            && (self.trade_routes_of(owner).count() as i32) + traders < self.trade_capacity(owner)
            && let Some(item) = best_unit(&|u| u.make_trade_route)
        {
            return Some(item);
        }
        if (at_war || self.rng.chance(0.15))
            && self.free_air_base(city).is_some()
            && let Some(item) = best_unit(&|u| u.domain == "DOMAIN_AIR")
        {
            return Some(item);
        }
        if military < wanted_military
            && let Some(item) = best_unit(&land_military)
        {
            return Some(item);
        }
        // Space race when available.
        if let Some(item) = available
            .iter()
            .find(|p| matches!(p, Production::Project(_)))
        {
            return Some(item.clone());
        }
        // A Starbase when the empire has nowhere to launch from: it works as
        // a Spaceport and builds space projects faster.
        let launch_site = self.player_cities(owner).any(|c| {
            c.buildings.iter().any(|&b| content.buildings[b].spaceport)
                || c.districts
                    .iter()
                    .any(|d| content.districts[d.district].id == crate::rules::SPACEPORT)
        });
        if !launch_site
            && let Some((w, site)) = available.iter().find_map(|p| match p {
                Production::Wonder(w, _) if content.buildings[*w].spaceport => {
                    self.wonder_sites(city, *w).first().map(|&site| (*w, site))
                }
                _ => None,
            })
        {
            return Some(Production::Wonder(w, site));
        }
        // A district on its best site.
        let districts: Vec<usize> = available
            .iter()
            .filter_map(|p| {
                if let Production::District(i, _) = p {
                    Some(*i)
                } else {
                    None
                }
            })
            .collect();
        // An Aerodrome once flight is known, for air power.
        let aerodrome = districts.iter().copied().find(|&d| {
            let d = &content.districts[d];
            d.id == "DISTRICT_AERODROME" || d.replaces.as_deref() == Some("DISTRICT_AERODROME")
        });
        if let Some(aerodrome) = aerodrome.filter(|_| self.air_bases(owner).len() < cities / 3 + 1)
        {
            if let Some(site) = self.district_sites(city, aerodrome).first() {
                return Some(Production::District(aerodrome, *site));
            }
        }
        // Campuses first, as a sensible AI would.
        let campus = districts.iter().copied().find(|&d| {
            let d = &content.districts[d];
            d.id == "DISTRICT_CAMPUS" || d.replaces.as_deref() == Some("DISTRICT_CAMPUS")
        });
        if let Some(campus) = campus
            && let Some(site) = self
                .district_sites(city, campus)
                .into_iter()
                .max_by_key(|&p| (self.adjacency(campus, p).iter().sum::<f32>() * 10.0) as i32)
        {
            return Some(Production::District(campus, site));
        }
        // Then a Datacenter to power the AI tree, next to other districts.
        let datacenter = districts.iter().copied().find(|&d| self.is_datacenter(d));
        if let Some(datacenter) = datacenter
            && let Some(site) = self
                .district_sites(city, datacenter)
                .into_iter()
                .max_by_key(|&p| {
                    self.map
                        .grid
                        .neighbors(p)
                        .filter(|&n| self.map.tile(n).district.is_some())
                        .count()
                })
        {
            return Some(Production::District(datacenter, site));
        }
        if !districts.is_empty() && self.rng.chance(0.6) {
            let mut best: Option<(f32, Production)> = None;
            for d in districts {
                for site in self.district_sites(city, d) {
                    let score: f32 =
                        self.adjacency(d, site).iter().sum::<f32>() + self.rng.unit() as f32;
                    if best.as_ref().is_none_or(|(s, _)| score > *s) {
                        best = Some((score, Production::District(d, site)));
                    }
                }
            }
            if let Some((_, item)) = best {
                return Some(item);
            }
        }
        let buildings: Vec<&Production> = available
            .iter()
            .filter(|p| matches!(p, Production::Building(_)))
            .collect();
        // Science buildings first, then the cheapest.
        let science = buildings.iter().find(|p| matches!(p, Production::Building(i) if content.buildings[*i].yields[crate::rules::SCIENCE] > 0));
        if let Some(item) = science {
            return Some((*item).clone());
        }
        if let Some(item) = buildings
            .iter()
            .min_by_key(|p| self.production_cost(owner, p))
        {
            return Some((*item).clone());
        }
        let wonders: Vec<usize> = available
            .iter()
            .filter_map(|p| {
                if let Production::Wonder(i, _) = p {
                    Some(*i)
                } else {
                    None
                }
            })
            .collect();
        if let Some(&w) = self.rng.pick(&wonders)
            && let Some(&site) = self.wonder_sites(city, w).first()
        {
            return Some(Production::Wonder(w, site));
        }
        best_unit(&land_military).or_else(|| {
            available
                .into_iter()
                .find(|p| matches!(p, Production::Unit(_)))
        })
    }

    fn ai_city_attack(&mut self, city: CityId) {
        let pos = self.cities[&city].pos;
        let targets: Vec<Pos> = self.map.grid.within(pos, 2);
        for t in targets {
            if self.city_can_attack(city, t) {
                let _ = self.city_attack(city, t);
                return;
            }
        }
    }

    fn ai_unit(&mut self, id: UnitId) {
        let unit = self.units[&id].clone();
        let ty = self.unit_type(&unit).clone();
        if unit.great_person.is_some() {
            if ty.found_religion {
                self.ai_prophet(id);
            } else {
                let _ = self.activate_great_person(id);
            }
            return;
        }
        if ty.make_trade_route {
            self.ai_trader(id);
        } else if self.is_air(&unit) {
            self.ai_air(id);
        } else if ty.found_city {
            self.ai_settler(id);
        } else if ty.build_charges > 0 {
            self.ai_builder(id);
        } else if unit.religion.is_some() {
            self.ai_missionary(id);
        } else if self.is_military(&unit) {
            self.ai_military(id);
        }
    }

    fn ai_prophet(&mut self, id: UnitId) {
        if self.can_found_religion(id) {
            let religion = *self.free_religions().first().unwrap();
            let (founder, follower) = self.default_beliefs();
            let _ = self.found_religion(id, religion, founder, follower);
            return;
        }
        let owner = self.units[&id].owner;
        let site = self
            .player_cities(owner)
            .flat_map(|c| c.districts.iter())
            .find(|d| {
                let d = &self.rules.content.districts[d.district];
                d.id == "DISTRICT_HOLY_SITE" || d.replaces.as_deref() == Some("DISTRICT_HOLY_SITE")
            })
            .map(|d| d.pos);
        if let Some(site) = site {
            let _ = self.move_unit(id, site);
        }
    }

    fn ai_settler(&mut self, id: UnitId) {
        let unit = self.units[&id].clone();
        let grid = self.map.grid;
        let rules = self.rules.clone();
        let first_city = self.players[unit.owner].cities_founded == 0;
        if self.can_found_city(id) && (first_city || site_score(&rules, &self.map, unit.pos) > 25.0)
        {
            let _ = self.found_city(id);
            return;
        }
        let home = self.players[unit.owner]
            .capital
            .map(|c| self.cities[&c].pos)
            .unwrap_or(unit.pos);
        let best = grid
            .within(unit.pos, if first_city { 2 } else { 10 })
            .into_iter()
            .filter(|&p| {
                self.map.is_land(&rules, p)
                    && self.map.tile(p).owner.is_none_or(|o| o == unit.owner)
                    && self
                        .cities
                        .values()
                        .all(|c| grid.distance(c.pos, p) >= MIN_CITY_DISTANCE)
                    && !self.ground_units_at(p).any(|u| u.owner != unit.owner)
                    && self.players[unit.owner].revealed[grid.index(p)]
            })
            .max_by(|a, b| {
                let score = |p: Pos| {
                    site_score(&rules, &self.map, p)
                        - grid.distance(home, p) as f64 * 1.5
                        - grid.distance(unit.pos, p) as f64
                };
                score(*a).partial_cmp(&score(*b)).unwrap()
            });
        match best {
            Some(site) if site == unit.pos => {
                let _ = self.found_city(id);
            }
            Some(site) => {
                if self.move_unit(id, site).is_err() || first_city {
                    if self.can_found_city(id) {
                        let _ = self.found_city(id);
                    }
                } else if self.units.get(&id).is_some_and(|u| u.pos == site)
                    && self.can_found_city(id)
                {
                    let _ = self.found_city(id);
                }
            }
            None => {
                if self.can_found_city(id) {
                    let _ = self.found_city(id);
                }
            }
        }
    }

    fn ai_builder(&mut self, id: UnitId) {
        let unit = self.units[&id].clone();
        let here = self.valid_improvements(unit.owner, unit.pos);
        if let Some(best) = self.best_improvement(unit.owner, unit.pos, &here) {
            let _ = self.build_improvement(id, best);
            return;
        }
        let grid = self.map.grid;
        let target = grid
            .within(unit.pos, 8)
            .into_iter()
            .filter(|&p| {
                self.map.tile(p).owner == Some(unit.owner) && self.map.tile(p).improvement.is_none()
            })
            .filter(|&p| {
                self.best_improvement(unit.owner, p, &self.valid_improvements(unit.owner, p))
                    .is_some()
            })
            .filter(|&p| {
                !self
                    .ground_units_at(p)
                    .any(|u| u.id != id && !self.is_military(u))
            })
            .min_by_key(|&p| {
                grid.distance(unit.pos, p) * 2 - i32::from(self.map.tile(p).resource.is_some()) * 3
            });
        if let Some(target) = target {
            let _ = self.move_unit(id, target);
            if self
                .units
                .get(&id)
                .is_some_and(|u| u.pos == target && u.moves > 0)
            {
                let options = self.valid_improvements(unit.owner, target);
                if let Some(best) = self.best_improvement(unit.owner, target, &options) {
                    let _ = self.build_improvement(id, best);
                }
            }
        } else {
            self.skip(id);
        }
    }

    /// The improvement that adds the most yield on `pos`.
    fn best_improvement(&self, _owner: PlayerId, pos: Pos, options: &[usize]) -> Option<usize> {
        let content = &self.rules.content;
        if self.map.tile(pos).improvement.is_some() || self.city_at(pos).is_some() {
            return None;
        }
        options
            .iter()
            .copied()
            .max_by_key(|&i| {
                let y = content.improvements[i].yields;
                y[FOOD] * 3
                    + y[1] * 3
                    + y[2] * 2
                    + y[3]
                    + y[4]
                    + y[5]
                    + i32::from(!content.improvements[i].valid_resources.is_empty()) * 2
            })
            .filter(|&i| {
                content.improvements[i].yields.iter().sum::<i32>() > 0
                    || !content.improvements[i].valid_resources.is_empty()
            })
    }

    fn ai_missionary(&mut self, id: UnitId) {
        let unit = self.units[&id].clone();
        let religion = unit.religion.unwrap();
        let target = self
            .cities
            .values()
            .filter(|c| {
                c.majority_religion() != Some(religion)
                    && !self.players[unit.owner].at_war_with(c.owner)
            })
            .min_by_key(|c| self.map.grid.distance(c.pos, unit.pos))
            .map(|c| c.pos);
        if let Some(target) = target {
            if self.map.grid.distance(target, unit.pos) <= 1 {
                let _ = self.spread_religion(id);
            } else {
                let next = self.map.grid.neighbors(target).find(|&n| {
                    self.units
                        .get(&id)
                        .is_some_and(|u| self.passable(u, n) && self.can_stop(u, n))
                });
                if let Some(next) = next {
                    let _ = self.move_unit(id, next);
                }
                if self
                    .units
                    .get(&id)
                    .is_some_and(|u| self.map.grid.distance(target, u.pos) <= 1 && u.moves > 0)
                {
                    let _ = self.spread_religion(id);
                }
            }
        }
    }

    fn ai_military(&mut self, id: UnitId) {
        let unit = self.units[&id].clone();
        let owner = unit.owner;
        let grid = self.map.grid;
        // 1. Attack the best target in reach if the trade is good.
        let range = self.unit_type(&unit).range.max(1);
        let reach = if self.attack_kind(&unit) == Some(AttackKind::Melee) {
            1
        } else {
            range
        };
        let mut best: Option<(i32, Pos)> = None;
        for t in grid.within(unit.pos, reach) {
            if !self.can_attack(&unit, t) {
                continue;
            }
            if let Some(preview) = self.preview(id, t) {
                let value = preview.damage_to_defender * 2 - preview.damage_to_attacker * 2
                    + i32::from(self.city_at(t).is_some()) * 20;
                let safe = preview.damage_to_attacker < unit.hp - 10;
                if safe && value > 0 && best.is_none_or(|(v, _)| value > v) {
                    best = Some((value, t));
                }
            }
        }
        if let Some((_, target)) = best {
            let _ = self.attack(id, target);
            return;
        }
        if unit.hp < 50 {
            self.fortify(id);
            return;
        }
        // 2. At war: march on the nearest enemy city or unit.
        let enemies: Vec<Pos> = self
            .cities
            .values()
            .filter(|c| self.players[owner].at_war_with(c.owner))
            .map(|c| c.pos)
            .chain(
                self.units
                    .values()
                    .filter(|u| self.players[owner].at_war_with(u.owner))
                    .map(|u| u.pos),
            )
            .filter(|&p| self.players[owner].visible[grid.index(p)] || self.city_at(p).is_some())
            .collect();
        // In a real war armies march a long way; barbarians only nearby.
        let reach = if self.major_wars(owner) > 0 { 26 } else { 14 };
        if let Some(&target) = enemies.iter().min_by_key(|&&p| grid.distance(p, unit.pos))
            && grid.distance(target, unit.pos) <= reach
        {
            let approach = grid
                .neighbors(target)
                .filter(|&n| self.passable(&unit, n) && self.can_stop(&unit, n))
                .min_by_key(|&n| grid.distance(n, unit.pos));
            if let Some(approach) = approach
                && self.move_unit(id, approach).is_ok()
            {
                return;
            }
        }
        // 2b. Preparing a war: gather near the border city facing the target,
        // leaving one defender in each city.
        if let Some(plan) = self.players[owner].war_plan.clone() {
            let garrison = self
                .city_at(unit.pos)
                .is_some_and(|c| {
                    c.owner == owner
                        && !self
                            .ground_units_at(c.pos)
                            .any(|u| u.owner == owner && u.id != id && self.is_military(u))
                });
            let staging = self
                .player_cities(owner)
                .flat_map(|mine| {
                    self.player_cities(plan.target)
                        .map(move |theirs| (grid.distance(mine.pos, theirs.pos), mine.pos, theirs.pos))
                })
                .min();
            if !garrison
                && let Some((_, from, toward)) = staging
            {
                let spot = grid
                    .within(from, 3)
                    .into_iter()
                    .filter(|&p| {
                        self.passable(&unit, p)
                            && self.can_stop(&unit, p)
                            && self.map.tile(p).owner == Some(owner)
                    })
                    .min_by_key(|&p| grid.distance(p, toward) * 4 + grid.distance(p, unit.pos));
                if let Some(spot) = spot {
                    if spot == unit.pos {
                        self.fortify(id);
                        return;
                    }
                    if self.move_unit(id, spot).is_ok() {
                        return;
                    }
                }
            }
        }
        // 3. Visit tribal villages the unit knows about.
        let village = self
            .villages
            .iter()
            .copied()
            .filter(|&v| {
                grid.distance(v, unit.pos) <= 8
                    && self.players[owner].revealed[grid.index(v)]
                    && self.passable(&unit, v)
            })
            .min_by_key(|&v| grid.distance(v, unit.pos));
        if let Some(v) = village
            && self.unit_type(&unit).domain == "DOMAIN_LAND"
            && self.move_unit(id, v).is_ok()
        {
            return;
        }
        // 4. Clear barbarian camps nearby.
        let camp = self
            .camps
            .keys()
            .copied()
            .filter(|&c| grid.distance(c, unit.pos) <= 8)
            .min_by_key(|&c| grid.distance(c, unit.pos));
        if let Some(camp) = camp {
            if self.move_unit(id, camp).is_ok() {
                return;
            }
        }
        // 4. Scouts and spare units explore; the rest guard cities.
        let is_recon =
            self.unit_type(&unit).promotion_class.as_deref() == Some("PROMOTION_CLASS_RECON");
        if is_recon || (self.turn < 30 && self.rng.chance(0.3)) {
            let jitter = self.rng.below(1000) as i32;
            let target = grid
                .within(unit.pos, 10)
                .into_iter()
                .filter(|&p| {
                    !self.players[owner].revealed[grid.index(p)] && self.passable(&unit, p)
                })
                .min_by_key(|&p| {
                    grid.distance(p, unit.pos) * 4 + (p.x * 7 + p.y * 13 + jitter) % 3
                });
            if let Some(target) = target
                && self.move_unit(id, target).is_ok()
            {
                return;
            }
        }
        let undefended = self
            .player_cities(owner)
            .filter(|c| {
                !self
                    .ground_units_at(c.pos)
                    .any(|u| u.owner == owner && self.is_military(u) && u.id != id)
            })
            .min_by_key(|c| grid.distance(c.pos, unit.pos))
            .map(|c| c.pos);
        match undefended {
            Some(pos) if pos != unit.pos => {
                if self.move_unit(id, pos).is_err() {
                    self.fortify(id);
                }
            }
            _ => self.fortify(id),
        }
    }

    fn ai_city_state(&mut self, player: PlayerId) {
        let cities: Vec<CityId> = self.player_cities(player).map(|c| c.id).collect();
        for city in cities {
            if self.cities[&city].production.is_none() {
                let military = self
                    .player_units(player)
                    .filter(|u| self.is_military(u))
                    .count();
                let available = self.available_production(city);
                let content = &self.rules.content;
                let item = if military < 3 {
                    available
                        .iter()
                        .filter(|p| matches!(p, Production::Unit(i) if content.units[*i].domain == "DOMAIN_LAND" && content.units[*i].combat > 0))
                        .max_by_key(|p| if let Production::Unit(i) = p { content.units[*i].combat } else { 0 })
                        .cloned()
                } else {
                    available
                        .iter()
                        .filter(|p| matches!(p, Production::Building(_)))
                        .min_by_key(|p| self.production_cost(player, p))
                        .cloned()
                };
                if let Some(item) = item {
                    let _ = self.set_production(city, item);
                }
            }
            self.ai_city_attack(city);
        }
        let units: Vec<UnitId> = self.player_units(player).map(|u| u.id).collect();
        for id in units {
            if !self.units.contains_key(&id) {
                continue;
            }
            let unit = self.units[&id].clone();
            let targets: Vec<Pos> = self.map.grid.within(unit.pos, 1);
            if let Some(t) = targets.into_iter().find(|&t| self.can_attack(&unit, t)) {
                let _ = self.attack(id, t);
            } else {
                self.fortify(id);
            }
        }
    }

    fn ai_trader(&mut self, id: UnitId) {
        let unit = self.units[&id].clone();
        let destinations = self.trade_destinations(id);
        if !destinations.is_empty() {
            let best = destinations.into_iter().max_by_key(|&d| {
                (self.route_yields(unit.owner, d).iter().sum::<f32>() * 10.0) as i32
            });
            if let Some(best) = best {
                if self.establish_trade_route(id, best).is_ok() {
                    return;
                }
            }
        }
        // Not in a city (or no capacity): go to the capital and wait.
        let home = self.players[unit.owner]
            .capital
            .map(|c| self.cities[&c].pos);
        match home {
            Some(home) if home != unit.pos => {
                let _ = self.move_unit(id, home);
            }
            _ => self.sleep(id),
        }
    }

    fn ai_air(&mut self, id: UnitId) {
        let unit = self.units[&id].clone();
        let range = self.unit_type(&unit).range.max(1);
        let target = self
            .map
            .grid
            .within(unit.pos, range)
            .into_iter()
            .filter(|&t| self.can_attack(&unit, t))
            .filter_map(|t| self.preview(id, t).map(|p| (t, p.damage_to_defender)))
            .max_by_key(|(_, d)| *d);
        if let Some((t, _)) = target {
            let _ = self.attack(id, t);
            return;
        }
        // Rebase toward the nearest enemy city when at war.
        let grid = self.map.grid;
        let enemy = self
            .cities
            .values()
            .filter(|c| self.players[unit.owner].at_war_with(c.owner))
            .map(|c| c.pos)
            .min_by_key(|&p| grid.distance(p, unit.pos));
        if let Some(enemy) = enemy {
            if let Some(base) = self
                .rebase_targets(&unit)
                .into_iter()
                .min_by_key(|&b| grid.distance(b, enemy))
            {
                if grid.distance(base, enemy) < grid.distance(unit.pos, enemy) {
                    let _ = self.rebase(id, base);
                    return;
                }
            }
        }
        self.fortify(id);
    }
}

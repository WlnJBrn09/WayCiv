//! Barbarians: camps in unclaimed fog (5+ tiles from any city) that spawn
//! raiders, as in Civ VI.
//! Barbarian units roam toward nearby cities, pillage improvements, and
//! attack weak targets, but never capture cities. Clearing a camp pays
//! 50 Gold.

use crate::hex::Pos;
use crate::map::PlayerId;
use crate::state::{Game, UnitId};

/// Turns between a camp's spawns.
const SPAWN_INTERVAL: u32 = 6;
/// Live barbarian units allowed around one camp.
const UNITS_PER_CAMP: usize = 3;
pub const CAMP_REWARD: i32 = 50;

impl Game {
    fn camp_improvement(&self) -> Option<usize> {
        self.rules.idx("IMPROVEMENT_BARBARIAN_CAMP").filter(|&i| {
            self.rules
                .content
                .improvements
                .get(i)
                .is_some_and(|x| x.id == "IMPROVEMENT_BARBARIAN_CAMP")
        })
    }

    /// A tile no major or city-state can see, far from cities.
    fn camp_site_ok(&self, pos: Pos) -> bool {
        let rules = &self.rules;
        let i = self.map.grid.index(pos);
        let tile = self.map.tile(pos);
        self.map.is_land(rules, pos)
            && !self.map.is_impassable(rules, pos)
            && tile.owner.is_none()
            && tile.improvement.is_none()
            && tile
                .feature
                .is_none_or(|f| !rules.content.features[f].natural_wonder)
            && pos.y > 1
            && pos.y < self.map.grid.height - 2
            && !self
                .players
                .iter()
                .any(|p| !p.is_barbarian() && p.visible[i])
            && self
                .cities
                .values()
                .all(|c| self.map.grid.distance(c.pos, pos) >= 5)
            && self.ground_units_at(pos).next().is_none()
            && self
                .camps
                .keys()
                .all(|c| self.map.grid.distance(*c, pos) >= 6)
    }

    /// Places a new camp somewhere suitable; returns where.
    pub(crate) fn spawn_camp(&mut self) -> Option<Pos> {
        let camp = self.camp_improvement()?;
        let sites: Vec<Pos> = self
            .map
            .grid
            .all()
            .filter(|&p| self.camp_site_ok(p))
            .collect();
        let pos = *self.rng.pick(&sites)?;
        self.map.tile_mut(pos).improvement = Some(camp);
        self.camps.insert(pos, SPAWN_INTERVAL);
        // Every camp starts with a guard, as in Civ VI.
        if let (Some(barbarian), Some(kind)) = (self.barbarian(), self.barbarian_unit()) {
            self.spawn_exact(barbarian, kind, pos);
        }
        Some(pos)
    }

    /// The strongest land unit barbarians can field in the current world era.
    fn barbarian_unit(&mut self) -> Option<usize> {
        let rules = self.rules.clone();
        let majors: Vec<&crate::state::Player> = self
            .players
            .iter()
            .filter(|p| p.is_major() && p.alive)
            .collect();
        let known_by_half = |tech: &Option<String>| -> bool {
            tech.as_deref().is_none_or(|t| {
                majors.iter().filter(|p| p.has_tech(&rules, t)).count() * 2 >= majors.len().max(1)
            })
        };
        let classes = [
            "PROMOTION_CLASS_MELEE",
            "PROMOTION_CLASS_RANGED",
            "PROMOTION_CLASS_ANTI_CAVALRY",
            "PROMOTION_CLASS_LIGHT_CAVALRY",
        ];
        let class = *self.rng.pick(&classes)?;
        let candidates: Vec<(usize, &crate::rules::Unit)> = rules
            .content
            .units
            .iter()
            .enumerate()
            .filter(|(_, u)| {
                u.domain == "DOMAIN_LAND"
                    && u.combat > 0
                    && u.unique_to.is_none()
                    && u.can_train
                    && !u.must_purchase
                    && u.promotion_class
                        .as_deref()
                        .is_some_and(|c| classes.contains(&c))
                    && u.prereq_civic.is_none()
                    && known_by_half(&u.prereq_tech)
            })
            .collect();
        // The chosen class if the world can field it yet, else the best of any.
        let strength = |u: &crate::rules::Unit| u.combat.max(u.ranged_combat);
        candidates
            .iter()
            .filter(|(_, u)| u.promotion_class.as_deref() == Some(class))
            .max_by_key(|(_, u)| strength(u))
            .or_else(|| candidates.iter().max_by_key(|(_, u)| strength(u)))
            .map(|(i, _)| *i)
    }

    /// Seeds the map with one camp per major civilization.
    pub(crate) fn place_starting_camps(&mut self) {
        let majors = self.players.iter().filter(|p| p.is_major()).count();
        for _ in 0..majors {
            self.spawn_camp();
        }
    }

    pub(crate) fn barbarian_turn(&mut self, barbarian: PlayerId) {
        // New camps appear over time while the world has room for them.
        let majors = self
            .players
            .iter()
            .filter(|p| p.is_major() && p.alive)
            .count();
        if self.turn % 8 == 0 && self.camps.len() < majors + 1 {
            self.spawn_camp();
        }
        // Camps spawn raiders.
        let camps: Vec<Pos> = self.camps.keys().copied().collect();
        for camp in camps {
            let timer = self.camps.get_mut(&camp).unwrap();
            *timer = timer.saturating_sub(1);
            if *timer > 0 {
                continue;
            }
            *timer = SPAWN_INTERVAL;
            let nearby = self
                .player_units(barbarian)
                .filter(|u| self.map.grid.distance(u.pos, camp) <= 5)
                .count();
            if nearby >= UNITS_PER_CAMP {
                continue;
            }
            let spot = std::iter::once(camp)
                .chain(self.map.grid.neighbors(camp))
                .find(|&p| {
                    self.map.is_land(&self.rules, p)
                        && !self.map.is_impassable(&self.rules, p)
                        && self.ground_units_at(p).next().is_none()
                });
            if let (Some(spot), Some(kind)) = (spot, self.barbarian_unit()) {
                self.spawn_exact(barbarian, kind, spot);
            }
        }
        let units: Vec<UnitId> = self.player_units(barbarian).map(|u| u.id).collect();
        for id in units {
            if self.units.contains_key(&id) {
                self.barbarian_unit_turn(id);
            }
        }
    }

    fn barbarian_unit_turn(&mut self, id: UnitId) {
        let unit = self.units[&id].clone();
        let grid = self.map.grid;
        // A unit standing on its camp guards it while others are around.
        let guarding = self.camps.contains_key(&unit.pos)
            && !self
                .player_units(unit.owner)
                .any(|u| u.id != id && grid.distance(u.pos, unit.pos) <= 1);
        // Attack anything in reach that won't kill it.
        let reach = if self.attack_kind(&unit) == Some(crate::combat::AttackKind::Melee) {
            1
        } else {
            self.unit_type(&unit).range.max(1)
        };
        let target = grid
            .within(unit.pos, reach)
            .into_iter()
            .filter(|&t| self.can_attack(&unit, t))
            .filter_map(|t| self.preview(id, t).map(|p| (t, p)))
            .filter(|(_, p)| p.damage_to_attacker < unit.hp)
            .max_by_key(|(_, p)| p.damage_to_defender - p.damage_to_attacker);
        if let Some((t, _)) = target {
            let _ = self.attack(id, t);
            return;
        }
        if guarding {
            self.fortify(id);
            return;
        }
        // Pillage the improvement underfoot.
        let tile = self.map.tile(unit.pos);
        if tile.improvement.is_some()
            && !tile.pillaged
            && tile.owner.is_some()
            && !self.camps.contains_key(&unit.pos)
        {
            let owner = tile.owner.unwrap();
            self.map.tile_mut(unit.pos).pillaged = true;
            self.notify(owner, "Barbarians pillaged an improvement!", Some(unit.pos));
            if let Some(u) = self.units.get_mut(&id) {
                u.moves = 0;
            }
            return;
        }
        // Head for the nearest city or improved tile within 10 tiles.
        let goal = self
            .cities
            .values()
            .map(|c| c.pos)
            .chain(grid.within(unit.pos, 6).into_iter().filter(|&p| {
                let t = self.map.tile(p);
                t.owner.is_some() && t.improvement.is_some() && !t.pillaged
            }))
            .filter(|&p| grid.distance(p, unit.pos) <= 10)
            .min_by_key(|&p| grid.distance(p, unit.pos));
        let goal = match goal {
            Some(g) => grid
                .neighbors(g)
                .chain(std::iter::once(g))
                .filter(|&n| self.passable(&unit, n) && self.can_stop(&unit, n))
                .min_by_key(|&n| grid.distance(n, unit.pos)),
            None => {
                let options: Vec<Pos> = grid
                    .within(unit.pos, 2)
                    .into_iter()
                    .filter(|&n| self.passable(&unit, n) && self.can_stop(&unit, n))
                    .collect();
                self.rng.pick(&options).copied()
            }
        };
        if let Some(goal) = goal.filter(|&g| g != unit.pos) {
            let _ = self.move_unit(id, goal);
        }
    }

    /// A non-barbarian military unit entering a camp destroys it.
    pub(crate) fn check_camp_cleared(&mut self, id: UnitId, pos: Pos) {
        if !self.camps.contains_key(&pos) {
            return;
        }
        let Some(unit) = self.units.get(&id) else {
            return;
        };
        let owner = unit.owner;
        if self.players[owner].is_barbarian() || !self.is_military(unit) {
            return;
        }
        self.camps.remove(&pos);
        self.map.tile_mut(pos).improvement = None;
        self.players[owner].gold += CAMP_REWARD;
        self.notify(
            owner,
            format!("Barbarian camp destroyed! +{CAMP_REWARD} Gold."),
            Some(pos),
        );
        self.check_boosts(owner, crate::research::Trigger::ClearedCamp);
    }
}

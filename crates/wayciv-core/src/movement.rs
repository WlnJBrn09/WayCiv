//! Unit movement, pathfinding, visibility, and first contact.

use crate::hex::Pos;
use crate::map::PlayerId;
use crate::state::{Game, Unit, UnitId};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

/// Multi-turn path search radius.
const MAX_SEARCH: i32 = 40;

impl Game {
    pub fn can_embark(&self, player: PlayerId) -> bool {
        self.players[player].has_tech(&self.rules, "TECH_SHIPBUILDING")
    }

    fn can_enter_ocean(&self, player: PlayerId) -> bool {
        self.players[player].has_tech(&self.rules, "TECH_CARTOGRAPHY")
    }

    /// Whether `unit` may stand on `pos` at all (terrain, domain, borders).
    pub fn passable(&self, unit: &Unit, pos: Pos) -> bool {
        let rules = &self.rules;
        let map = &self.map;
        if map.is_impassable(rules, pos) {
            return false;
        }
        let ty = self.unit_type(unit);
        let water = map.is_water(rules, pos);
        let terrain = &rules.content.terrains[map.tile(pos).terrain];
        let ocean = terrain.water && !terrain.shallow_water;
        let ice = map
            .tile(pos)
            .feature
            .is_some_and(|f| rules.content.features[f].id == "FEATURE_ICE");
        if ice {
            return false;
        }
        match ty.domain.as_str() {
            "DOMAIN_SEA" => {
                let own_city = self.city_at(pos).is_some_and(|c| c.owner == unit.owner);
                (water || own_city) && (!ocean || self.can_enter_ocean(unit.owner))
            }
            _ => {
                if water
                    && !(self.can_embark(unit.owner)
                        && (!ocean || self.can_enter_ocean(unit.owner)))
                {
                    return false;
                }
                if let Some(owner) = map.tile(pos).owner {
                    let me = &self.players[unit.owner];
                    let them = &self.players[owner];
                    // Another major's territory is closed unless at war or
                    // declared friends; religious units and great people may
                    // always enter.
                    let exempt = unit.religion.is_some() || unit.great_person.is_some();
                    let closed = owner != unit.owner
                        && them.is_major()
                        && !me.at_war_with(owner)
                        && !me.relations[owner].friendship
                        && !exempt;
                    if closed {
                        return false;
                    }
                }
                true
            }
        }
    }

    /// Movement points to enter `to` (from an adjacent tile).
    pub fn move_cost(&self, unit: &Unit, from: Pos, to: Pos) -> i32 {
        let rules = &self.rules;
        let map = &self.map;
        let ty = self.unit_type(unit);
        if ty.domain == "DOMAIN_SEA" {
            return 1;
        }
        let (from_water, to_water) = (map.is_water(rules, from), map.is_water(rules, to));
        if from_water != to_water && self.city_at(to).is_none() {
            // Embarking or disembarking uses every remaining move.
            return i32::MAX;
        }
        if to_water {
            return 1;
        }
        // Strike drones fly over rough terrain.
        if self.is_drone(unit) {
            return 1;
        }
        // Roads (and cities) make any land tile cost a single move.
        let road = |p: Pos| map.tile(p).road || self.city_at(p).is_some();
        if road(from) && road(to) {
            return 1;
        }
        let tile = map.tile(to);
        let terrain = &rules.content.terrains[tile.terrain];
        let mut cost = terrain.movement_cost.max(1);
        if let Some(f) = tile.feature {
            cost += rules.content.features[f].movement_change.max(0);
        }
        cost
    }

    /// Whether an enemy military unit or city exerts zone of control on `pos`.
    pub fn zone_of_control(&self, player: PlayerId, pos: Pos) -> bool {
        self.map
            .grid
            .neighbors(pos)
            .chain(std::iter::once(pos))
            .any(|n| {
                self.ground_units_at(n)
                    .any(|u| self.players[player].at_war_with(u.owner) && self.is_military(u))
                    || self
                        .city_at(n)
                        .is_some_and(|c| self.players[player].at_war_with(c.owner))
            })
    }

    /// Zones of control stop military units, except flying strike drones.
    pub fn zoc_applies(&self, unit: &Unit) -> bool {
        self.is_military(unit) && !self.is_drone(unit)
    }

    /// True if `unit` could end its move on `pos` (stacking rules).
    pub fn can_stop(&self, unit: &Unit, pos: Pos) -> bool {
        let military = self.is_military(unit);
        !self
            .ground_units_at(pos)
            .any(|u| u.id != unit.id && (u.owner != unit.owner || self.is_military(u) == military))
            && self.city_at(pos).is_none_or(|c| c.owner == unit.owner)
    }

    /// Cheapest path as (turns, moves left), searching up to `MAX_SEARCH`.
    pub fn find_path(&self, unit: &Unit, goal: Pos) -> Option<Vec<Pos>> {
        if unit.pos == goal || self.is_air(unit) || !self.passable(unit, goal) {
            return None;
        }
        let grid = self.map.grid;
        let full = self.unit_type(unit).moves.max(1) + self.bonus_moves(unit);
        // Priority: fewer turns, then more moves left, then distance.
        let mut heap = BinaryHeap::new();
        let mut best: HashMap<Pos, (i32, i32)> = HashMap::new();
        let mut came: HashMap<Pos, Pos> = HashMap::new();
        best.insert(unit.pos, (0, unit.moves));
        heap.push(Reverse((
            0,
            -unit.moves,
            grid.distance(unit.pos, goal),
            unit.pos,
        )));
        while let Some(Reverse((turns, neg_left, _, pos))) = heap.pop() {
            if pos == goal {
                let mut path = vec![goal];
                let mut at = goal;
                while let Some(&prev) = came.get(&at) {
                    if prev == unit.pos {
                        break;
                    }
                    path.push(prev);
                    at = prev;
                }
                path.reverse();
                return Some(path);
            }
            if best
                .get(&pos)
                .is_some_and(|&(t, l)| (t, -l) < (turns, neg_left))
            {
                continue;
            }
            for next in grid.neighbors(pos) {
                if grid.distance(unit.pos, next) > MAX_SEARCH || !self.passable(unit, next) {
                    continue;
                }
                let occupied_by_other = self.ground_units_at(next).any(|u| u.owner != unit.owner)
                    || self.city_at(next).is_some_and(|c| c.owner != unit.owner);
                if occupied_by_other && next != goal {
                    continue;
                }
                let cost = self.move_cost(unit, pos, next);
                let (mut t, mut left) = (turns, -neg_left);
                if left <= 0 {
                    t += 1;
                    left = full;
                }
                if cost > left && left < full {
                    t += 1;
                    left = full;
                }
                left = if cost == i32::MAX {
                    0
                } else {
                    (left - cost).max(0)
                };
                if next != goal && self.zone_of_control(unit.owner, next) && self.zoc_applies(unit)
                {
                    left = 0;
                }
                if best
                    .get(&next)
                    .is_none_or(|&(bt, bl)| (t, -left) < (bt, -bl))
                {
                    best.insert(next, (t, left));
                    came.insert(next, pos);
                    heap.push(Reverse((t, -left, grid.distance(next, goal), next)));
                }
            }
        }
        None
    }

    /// Tiles `unit` can reach with its remaining moves this turn.
    pub fn reachable(&self, unit: &Unit) -> Vec<Pos> {
        if self.is_air(unit) {
            return if unit.moves > 0 {
                self.rebase_targets(unit)
            } else {
                Vec::new()
            };
        }
        let grid = self.map.grid;
        let mut best: HashMap<Pos, i32> = HashMap::new();
        let mut heap = BinaryHeap::new();
        best.insert(unit.pos, unit.moves);
        heap.push((unit.moves, unit.pos));
        let full = self.unit_type(unit).moves;
        while let Some((left, pos)) = heap.pop() {
            if best.get(&pos).is_some_and(|&b| b > left) || left <= 0 {
                continue;
            }
            for next in grid.neighbors(pos) {
                if !self.passable(unit, next)
                    || self.ground_units_at(next).any(|u| u.owner != unit.owner)
                    || self.city_at(next).is_some_and(|c| c.owner != unit.owner)
                {
                    continue;
                }
                let cost = self.move_cost(unit, pos, next);
                if cost > left && left < full {
                    continue;
                }
                let mut remaining = if cost == i32::MAX {
                    0
                } else {
                    (left - cost).max(0)
                };
                if self.zoc_applies(unit) && self.zone_of_control(unit.owner, next) {
                    remaining = 0;
                }
                if best.get(&next).is_none_or(|&b| remaining > b) {
                    best.insert(next, remaining);
                    heap.push((remaining, next));
                }
            }
        }
        best.into_keys()
            .filter(|&p| p != unit.pos && self.can_stop(unit, p))
            .collect()
    }

    fn bonus_moves(&self, _unit: &Unit) -> i32 {
        0
    }

    /// Orders `unit` toward `goal`. Moves as far as possible this turn,
    /// attacking if the goal holds an enemy in reach.
    pub fn move_unit(&mut self, id: UnitId, goal: Pos) -> Result<(), String> {
        let unit = self.units.get(&id).ok_or("No such unit.")?.clone();
        if self.can_attack(&unit, goal) {
            return self.attack(id, goal);
        }
        if self.is_air(&unit) {
            return self.rebase(id, goal);
        }
        let path = self.find_path(&unit, goal).ok_or("No path there.")?;
        let u = self.units.get_mut(&id).unwrap();
        u.path = path;
        u.fortified = false;
        u.sleeping = false;
        self.continue_path(id);
        Ok(())
    }

    /// Advances a unit along its stored path with its remaining moves.
    pub(crate) fn continue_path(&mut self, id: UnitId) {
        loop {
            let Some(unit) = self.units.get(&id).cloned() else {
                return;
            };
            let Some(&next) = unit.path.first() else {
                return;
            };
            if unit.moves <= 0 {
                return;
            }
            if self.map.grid.distance(unit.pos, next) != 1 || !self.passable(&unit, next) {
                self.units.get_mut(&id).unwrap().path.clear();
                return;
            }
            // An enemy in the way: attack only if it is the destination.
            let hostile = self.ground_units_at(next).any(|u| u.owner != unit.owner)
                || self.city_at(next).is_some_and(|c| c.owner != unit.owner);
            if hostile {
                let last = unit.path.len() == 1;
                self.units.get_mut(&id).unwrap().path.clear();
                if last && self.can_attack(&unit, next) {
                    let _ = self.attack(id, next);
                }
                return;
            }
            if unit.path.len() == 1 && !self.can_stop(&unit, next) {
                self.units.get_mut(&id).unwrap().path.clear();
                return;
            }
            let cost = self.move_cost(&unit, unit.pos, next);
            let full = self.unit_type(&unit).moves;
            if cost > unit.moves && unit.moves < full {
                return;
            }
            let zoc = self.zoc_applies(&unit) && self.zone_of_control(unit.owner, next);
            let owner = unit.owner;
            {
                let u = self.units.get_mut(&id).unwrap();
                u.pos = next;
                u.path.remove(0);
                u.moves = if cost == i32::MAX || zoc {
                    0
                } else {
                    (u.moves - cost).max(0)
                };
                u.fortified = false;
                u.fortify_turns = 0;
            }
            self.on_entered(id, next);
            self.update_visibility(owner);
        }
    }

    /// Effects of a unit entering a tile: capturing civilians, discovering
    /// natural wonders, popping into an empty enemy city.
    fn on_entered(&mut self, id: UnitId, pos: Pos) {
        self.check_camp_cleared(id, pos);
        let Some(unit) = self.units.get(&id).cloned() else {
            return;
        };
        // Civilians have no health: an enemy combat unit on their tile kills them.
        let victims: Vec<UnitId> = self
            .units_at(pos)
            .filter(|u| {
                u.owner != unit.owner
                    && !self.is_military(u)
                    && self.players[unit.owner].at_war_with(u.owner)
            })
            .map(|u| u.id)
            .collect();
        for victim in victims {
            if let Some(v) = self.units.remove(&victim) {
                let name = self.unit_type(&v).name.clone();
                self.notify(v.owner, format!("Your {name} was killed."), Some(pos));
                self.check_elimination(v.owner);
            }
        }
    }

    /// Recomputes what `player` can see; records first contact.
    pub fn update_visibility(&mut self, player: PlayerId) {
        let grid = self.map.grid;
        let mut visible = vec![false; grid.len()];
        let mut sources: Vec<(Pos, i32)> = Vec::new();
        for u in self.player_units(player) {
            let hills = self.rules.content.terrains[self.map.tile(u.pos).terrain].hills;
            sources.push((u.pos, self.unit_type(u).sight.max(1) + i32::from(hills)));
        }
        for c in self.player_cities(player) {
            sources.push((c.pos, 3));
        }
        for (i, t) in self.map.tiles.iter().enumerate() {
            if t.owner == Some(player) {
                visible[i] = true;
            }
        }
        for (pos, radius) in sources {
            for p in grid.within(pos, radius) {
                visible[grid.index(p)] = true;
            }
        }
        let p = &mut self.players[player];
        for i in 0..visible.len() {
            if visible[i] {
                p.revealed[i] = true;
            }
        }
        p.visible = visible;
        self.check_contact(player);
    }

    fn check_contact(&mut self, player: PlayerId) {
        let mut met: Vec<PlayerId> = Vec::new();
        let visible = &self.players[player].visible;
        let grid = self.map.grid;
        for u in self.units.values() {
            if u.owner != player && visible[grid.index(u.pos)] {
                met.push(u.owner);
            }
        }
        for c in self.cities.values() {
            if c.owner != player && visible[grid.index(c.pos)] {
                met.push(c.owner);
            }
        }
        met.sort();
        met.dedup();
        for other in met {
            if self.players[player].relations[other].met
                || self.players[other].is_barbarian()
                || self.players[player].is_barbarian()
            {
                continue;
            }
            self.players[player].relations[other].met = true;
            self.players[other].relations[player].met = true;
            let (a, b) = (
                self.players[player].clone_name(),
                self.players[other].clone_name(),
            );
            if self.players[other].is_city_state() {
                if self.players[player].is_major() {
                    // Meeting a city-state first grants an envoy, as in Civ VI.
                    let first = !self
                        .players
                        .iter()
                        .any(|p| p.id != player && p.is_major() && p.relations[other].met);
                    if first {
                        *self.players[player].envoys.entry(other).or_insert(0) += 1;
                    }
                    self.notify(player, format!("You met the city-state of {b}."), None);
                    self.check_boosts(player, crate::research::Trigger::Poll);
                }
            } else if self.players[player].is_city_state() {
                self.notify(other, format!("You met the city-state of {a}."), None);
            } else {
                self.notify(
                    player,
                    format!("You met {b} of {}.", self.players[other].civ_name),
                    None,
                );
                self.notify(
                    other,
                    format!("You met {a} of {}.", self.players[player].civ_name),
                    None,
                );
                if self.players[player].human {
                    self.popups
                        .push((other, crate::diplomacy::DiploEvent::FirstMeet));
                } else if self.players[other].human {
                    self.popups
                        .push((player, crate::diplomacy::DiploEvent::FirstMeet));
                }
                self.check_boosts(player, crate::research::Trigger::MetCiv);
                self.check_boosts(other, crate::research::Trigger::MetCiv);
            }
        }
    }
}

impl crate::state::Player {
    fn clone_name(&self) -> String {
        self.name.clone()
    }
}

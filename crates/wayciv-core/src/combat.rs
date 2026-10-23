//! Combat. The damage formula and wall rules follow Civ VI's own
//! (recovered from `libGameCore_Base.so`):
//! `damage = floor(e^(0.039 * (attack - defense)) * (24 + roll(0..12)) + 0.5)`,
//! clamped to 1..100; walls take 15% of melee, 50% of ranged, and 100% of
//! bombard damage, and shield the city's garrison in proportion to their
//! remaining hit points.

use crate::hex::Pos;
use crate::map::{CityId, PlayerId};
use crate::state::{Game, Unit, UnitId};

/// `COMBAT_POWER_SCALING` 0.04 as the game stores it (10/256).
const POWER_SCALING: f64 = 0.039_062_5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackKind {
    Melee,
    Ranged,
    Bombard,
}

/// Predicted outcome for the UI's combat preview.
#[derive(Clone, Debug)]
pub struct CombatPreview {
    pub attack: i32,
    pub defense: i32,
    pub damage_to_defender: i32,
    pub damage_to_attacker: i32,
    pub kind: AttackKind,
}

fn damage(attack: i32, defense: i32, roll: u32) -> i32 {
    let base = 24.0 + roll as f64;
    let value = ((attack - defense) as f64 * POWER_SCALING).exp() * base + 0.5;
    (value.floor() as i32).clamp(1, 100)
}

impl Game {
    pub fn attack_kind(&self, unit: &Unit) -> Option<AttackKind> {
        let ty = self.unit_type(unit);
        if ty.bombard > 0 {
            Some(AttackKind::Bombard)
        } else if ty.ranged_combat > 0 {
            Some(AttackKind::Ranged)
        } else if ty.combat > 0 {
            Some(AttackKind::Melee)
        } else {
            None
        }
    }

    fn enemy_at(&self, player: PlayerId, pos: Pos) -> bool {
        let at_war = |owner: PlayerId| self.players[player].at_war_with(owner);
        self.ground_units_at(pos).any(|u| at_war(u.owner))
            || self.city_at(pos).is_some_and(|c| at_war(c.owner))
    }

    pub fn can_attack(&self, unit: &Unit, target: Pos) -> bool {
        let Some(kind) = self.attack_kind(unit) else {
            return false;
        };
        if unit.attacked || unit.moves <= 0 || !self.enemy_at(unit.owner, target) {
            return false;
        }
        let distance = self.map.grid.distance(unit.pos, target);
        match kind {
            AttackKind::Melee => {
                distance == 1
                    && self.passable(unit, target)
                    && !self.map.is_water(&self.rules, unit.pos)
                        == !self.map.is_water(&self.rules, target)
            }
            // Aircraft strike anything within their operation range.
            _ if self.is_air(unit) => distance <= self.unit_type(unit).range.max(1),
            _ => distance >= 1 && distance <= self.unit_type(unit).range.max(1),
        }
    }

    /// Strength of a unit in a fight, with every modifier applied.
    pub fn unit_strength(
        &self,
        unit: &Unit,
        attacking: bool,
        kind: AttackKind,
        versus: Option<&Unit>,
        versus_city: bool,
    ) -> i32 {
        let rules = &self.rules;
        let ty = self.unit_type(unit);
        let mut s = match (attacking, kind) {
            (true, AttackKind::Ranged) => ty.ranged_combat,
            (true, AttackKind::Bombard) => ty.bombard,
            _ => ty.combat.max(ty.ranged_combat.min(ty.combat.max(1))),
        };
        if !attacking && ty.combat == 0 {
            s = ty.ranged_combat;
        }
        let embarked = ty.domain == "DOMAIN_LAND" && self.map.is_water(rules, unit.pos);
        if embarked && !attacking {
            return 10 + 5 * self.era_of(unit.owner) as i32;
        }
        let bonuses = self.bonuses(unit.owner);
        s += bonuses.combat + unit.level * 3;
        if self.map.tile(unit.pos).owner == Some(unit.owner) {
            s += bonuses.combat_in_territory;
        }
        s -= (100 - unit.hp) / 10;
        if !attacking {
            let tile = self.map.tile(unit.pos);
            s += rules.content.terrains[tile.terrain].defense_modifier;
            if let Some(f) = tile.feature {
                s += rules.content.features[f].defense_modifier;
            }
            if unit.fortified {
                s += 4;
            }
            if tile.cliffs {
                s += 3;
            }
        }
        if let Some(other) = versus {
            let my_class = ty.promotion_class.as_deref().unwrap_or("");
            let their_class = self
                .unit_type(other)
                .promotion_class
                .as_deref()
                .unwrap_or("");
            if my_class == "PROMOTION_CLASS_ANTI_CAVALRY" && their_class.contains("CAVALRY") {
                s += 10;
            }
        }
        if versus_city && kind == AttackKind::Ranged {
            s -= 17;
        }
        s.max(1)
    }

    /// A city's defensive strength.
    pub fn city_strength(&self, city: CityId) -> i32 {
        let c = &self.cities[&city];
        let rules = &self.rules;
        let p = &self.players[c.owner];
        let best_melee = rules
            .content
            .units
            .iter()
            .filter(|u| {
                u.combat > 0
                    && u.ranged_combat == 0
                    && u.domain == "DOMAIN_LAND"
                    && u.unique_to.is_none()
            })
            .filter(|u| p.meets(rules, u.prereq_tech.as_deref(), u.prereq_civic.as_deref()))
            .map(|u| u.combat)
            .max()
            .unwrap_or(20);
        let walls = c
            .buildings
            .iter()
            .filter(|&&b| rules.content.buildings[b].outer_defense > 0)
            .count() as i32;
        let garrison = self
            .ground_units_at(c.pos)
            .filter(|u| u.owner == c.owner && self.is_military(u))
            .map(|u| self.unit_type(u).combat)
            .max()
            .unwrap_or(0);
        (best_melee - 10).max(12) + walls * 3 + i32::from(c.capital) * 3 + garrison / 10
            - (200 - c.hp) / 20
    }

    pub fn max_walls(&self, city: CityId) -> i32 {
        self.cities[&city]
            .buildings
            .iter()
            .map(|&b| self.rules.content.buildings[b].outer_defense)
            .sum()
    }

    pub fn preview(&self, attacker: UnitId, target: Pos) -> Option<CombatPreview> {
        let unit = self.units.get(&attacker)?;
        let kind = self.attack_kind(unit)?;
        if let Some(city) = self.city_at(target).filter(|c| c.owner != unit.owner) {
            let attack = self.unit_strength(unit, true, kind, None, true);
            let defense = self.city_strength(city.id);
            return Some(CombatPreview {
                attack,
                defense,
                damage_to_defender: damage(attack, defense, 6),
                damage_to_attacker: if kind == AttackKind::Melee {
                    damage(defense, attack, 6)
                } else {
                    0
                },
                kind,
            });
        }
        let defender = self.defender_at(target, unit.owner)?;
        if !self.is_military(defender) {
            // Civilians have no health: one attack kills them.
            return Some(CombatPreview {
                attack: self.unit_strength(unit, true, kind, None, false),
                defense: 0,
                damage_to_defender: 100,
                damage_to_attacker: 0,
                kind,
            });
        }
        let attack = self.unit_strength(unit, true, kind, Some(defender), false);
        let defense = self.unit_strength(defender, false, kind, Some(unit), false);
        Some(CombatPreview {
            attack,
            defense,
            damage_to_defender: damage(attack, defense, 6),
            damage_to_attacker: if kind == AttackKind::Melee {
                damage(defense, attack, 6)
            } else {
                0
            },
            kind,
        })
    }

    /// The unit that defends `pos` against `attacker_owner`: military first.
    fn defender_at(&self, pos: Pos, attacker_owner: PlayerId) -> Option<&Unit> {
        let mut units: Vec<&Unit> = self
            .ground_units_at(pos)
            .filter(|u| self.players[attacker_owner].at_war_with(u.owner))
            .collect();
        units.sort_by_key(|u| std::cmp::Reverse(self.unit_type(u).combat));
        units.into_iter().next()
    }

    pub fn attack(&mut self, attacker: UnitId, target: Pos) -> Result<(), String> {
        let unit = self.units.get(&attacker).ok_or("No such unit.")?.clone();
        if !self.can_attack(&unit, target) {
            return Err("Can't attack there.".into());
        }
        let kind = self.attack_kind(&unit).unwrap();
        let roll = self.rng.below(12);
        let counter_roll = self.rng.below(12);
        let attacker_name = self.unit_type(&unit).name.clone();
        let city = self
            .city_at(target)
            .filter(|c| c.owner != unit.owner)
            .map(|c| c.id);
        if let Some(city) = city {
            let attack = self.unit_strength(&unit, true, kind, None, true);
            let defense = self.city_strength(city);
            let dealt = damage(attack, defense, roll);
            let max_walls = self.max_walls(city);
            let c = self.cities.get_mut(&city).unwrap();
            if c.wall_hp > 0 && max_walls > 0 {
                let wall_share = c.wall_hp as f32 / max_walls as f32;
                let percent = match kind {
                    AttackKind::Melee => 15,
                    AttackKind::Ranged => 50,
                    AttackKind::Bombard => 100,
                };
                c.wall_hp = (c.wall_hp - dealt * percent / 100).max(0);
                let to_garrison = (dealt as f32 * (1.0 - wall_share)).round() as i32;
                c.hp = (c.hp - to_garrison.max(1)).max(0);
            } else {
                c.hp = (c.hp - dealt).max(0);
            }
            c.turns_since_combat = 0;
            let (city_name, owner, hp) = (c.name.clone(), c.owner, c.hp);
            self.combat_log
                .push(format!("{attacker_name} attacked {city_name} for {dealt}."));
            if kind == AttackKind::Melee {
                let taken = damage(defense, attack, counter_roll);
                self.hurt(attacker, taken, Some(owner));
                // Barbarians raid cities but never take them.
                if hp <= 0
                    && self.units.contains_key(&attacker)
                    && !self.players[unit.owner].is_barbarian()
                {
                    self.capture_city(city, unit.owner, attacker);
                }
            }
            self.notify(owner, format!("{city_name} is under attack!"), Some(target));
        } else if self
            .defender_at(target, unit.owner)
            .is_some_and(|d| !self.is_military(d))
        {
            self.kill_civilians(&unit, target, kind);
        } else {
            let defender = self
                .defender_at(target, unit.owner)
                .ok_or("Nothing to attack.")?
                .clone();
            let attack = self.unit_strength(&unit, true, kind, Some(&defender), false);
            let defense = self.unit_strength(&defender, false, kind, Some(&unit), false);
            let dealt = damage(attack, defense, roll);
            let defender_name = self.unit_type(&defender).name.clone();
            self.combat_log.push(format!(
                "{attacker_name} attacked {defender_name} for {dealt}."
            ));
            let killed = self.hurt(defender.id, dealt, Some(unit.owner));
            if killed {
                self.players[unit.owner].units_killed += 1;
                if self.players[defender.owner].is_barbarian() {
                    self.players[unit.owner].barbarians_killed += 1;
                }
                self.check_boosts(
                    unit.owner,
                    crate::research::Trigger::Killed { with: unit.kind },
                );
            }
            if kind == AttackKind::Melee {
                let taken = damage(defense, attack, counter_roll);
                let attacker_died = self.hurt(attacker, taken, Some(defender.owner));
                // A victorious melee attacker advances into the empty tile.
                if killed
                    && !attacker_died
                    && !self.ground_units_at(target).any(|u| self.is_military(u))
                {
                    let u = self.units.get_mut(&attacker).unwrap();
                    u.pos = target;
                    self.check_camp_cleared(attacker, target);
                    self.update_visibility(unit.owner);
                }
            }
        }
        // Aircraft take fire from anti-air and intercepting fighters.
        if self.is_air(&unit) {
            if let Some(defense) = self.air_defense(&unit, target) {
                let roll = self.rng.below(12);
                let attack = self.unit_type(&unit).combat;
                self.hurt(attacker, damage(defense, attack, roll), None);
            }
        }
        if let Some(u) = self.units.get_mut(&attacker) {
            u.attacked = true;
            u.moves = 0;
            u.fortified = false;
            u.xp += if kind == AttackKind::Melee { 5 } else { 3 };
            if u.xp >= 15 * (u.level + 1) {
                u.xp = 0;
                u.level += 1;
                u.hp = (u.hp + 50).min(100);
            }
        }
        Ok(())
    }

    /// Civilians (settlers, builders, ...) have no health: one attack by a
    /// combat unit kills every enemy civilian on the tile. A melee attacker
    /// then moves in.
    fn kill_civilians(&mut self, attacker: &Unit, target: Pos, kind: AttackKind) {
        let victims: Vec<UnitId> = self
            .ground_units_at(target)
            .filter(|u| !self.is_military(u) && self.players[attacker.owner].at_war_with(u.owner))
            .map(|u| u.id)
            .collect();
        for id in victims {
            let dead = self.units.remove(&id).unwrap();
            let name = self.unit_type(&dead).name.clone();
            self.combat_log.push(format!(
                "{} killed a {name}.",
                self.unit_type(attacker).name
            ));
            self.notify(dead.owner, format!("Your {name} was killed."), Some(target));
            self.notify(
                attacker.owner,
                format!("Killed an enemy {name}."),
                Some(target),
            );
            self.check_elimination(dead.owner);
        }
        if kind == AttackKind::Melee
            && self
                .units
                .get(&attacker.id)
                .is_some_and(|u| self.can_stop(u, target))
        {
            self.units.get_mut(&attacker.id).unwrap().pos = target;
            self.check_camp_cleared(attacker.id, target);
            self.update_visibility(attacker.owner);
        }
    }

    /// Damages a unit; returns true if it died.
    fn hurt(&mut self, id: UnitId, amount: i32, by: Option<PlayerId>) -> bool {
        let Some(u) = self.units.get_mut(&id) else {
            return false;
        };
        u.hp -= amount;
        if u.hp > 0 {
            return false;
        }
        let dead = self.units.remove(&id).unwrap();
        let name = self.unit_type(&dead).name.clone();
        self.notify(
            dead.owner,
            format!("Your {name} was killed."),
            Some(dead.pos),
        );
        if let Some(by) = by {
            self.notify(by, format!("Defeated an enemy {name}."), Some(dead.pos));
        }
        self.check_elimination(dead.owner);
        true
    }

    /// City ranged attack (cities with walls, range 2).
    pub fn city_can_attack(&self, city: CityId, target: Pos) -> bool {
        let c = &self.cities[&city];
        !c.has_attacked
            && self.max_walls(city) > 0
            && self.map.grid.distance(c.pos, target) <= 2
            && self
                .ground_units_at(target)
                .any(|u| self.players[c.owner].at_war_with(u.owner))
    }

    pub fn city_attack(&mut self, city: CityId, target: Pos) -> Result<(), String> {
        if !self.city_can_attack(city, target) {
            return Err("The city can't attack there.".into());
        }
        let owner = self.cities[&city].owner;
        let defender = self
            .defender_at(target, owner)
            .ok_or("Nothing to attack.")?
            .clone();
        let attack = self.city_strength(city);
        let defense = self.unit_strength(&defender, false, AttackKind::Ranged, None, false);
        let roll = self.rng.below(12);
        self.hurt(defender.id, damage(attack, defense, roll), Some(owner));
        self.cities.get_mut(&city).unwrap().has_attacked = true;
        Ok(())
    }

    fn capture_city(&mut self, city: CityId, new_owner: PlayerId, unit: UnitId) {
        let (old_owner, pos, name, was_capital) = {
            let c = &self.cities[&city];
            (c.owner, c.pos, c.name.clone(), c.capital)
        };
        self.destroy_based_aircraft(city, old_owner);
        self.cancel_routes(|_, r| r.origin != city && r.destination != city);
        // Civilians in the city are captured or destroyed.
        let civilians: Vec<UnitId> = self
            .ground_units_at(pos)
            .filter(|u| u.owner == old_owner)
            .map(|u| u.id)
            .collect();
        for id in civilians {
            self.units.remove(&id);
        }
        let palace = self.rules.idx("BUILDING_PALACE");
        let rules = self.rules.clone();
        let old_is_cs = self.players[old_owner].is_city_state();
        {
            let c = self.cities.get_mut(&city).unwrap();
            c.owner = new_owner;
            c.loyalty = crate::loyalty::MAX_LOYALTY / 2;
            c.population = (c.population - 1).max(1);
            c.hp = 100;
            c.wall_hp = 0;
            c.production = None;
            c.capital = false;
            c.buildings.retain(|&b| {
                Some(b) != palace && !rules.content.buildings[b].wonder
                    || rules.content.buildings[b].wonder
            });
            c.buildings.retain(|&b| Some(b) != palace);
        }
        for t in self.map.tiles.iter_mut() {
            if t.owning_city == Some(city) {
                t.owner = Some(new_owner);
            }
        }
        if let Some(u) = self.units.get_mut(&unit) {
            u.pos = pos;
        }
        if was_capital {
            self.players[old_owner].capital = None;
            let next = self
                .player_cities(old_owner)
                .max_by_key(|c| c.population)
                .map(|c| c.id);
            if let Some(next) = next {
                self.cities.get_mut(&next).unwrap().capital = true;
                self.players[old_owner].capital = Some(next);
            }
        }
        if old_is_cs {
            self.players[old_owner].alive = false;
        }
        self.notify(old_owner, format!("{name} has been captured!"), Some(pos));
        self.notify(new_owner, format!("You captured {name}!"), Some(pos));
        self.assign_tiles(city);
        self.check_elimination(old_owner);
        self.update_visibility(new_owner);
    }

    /// A player with no cities and no settlers is out of the game.
    pub(crate) fn check_elimination(&mut self, player: PlayerId) {
        if !self.players[player].alive || self.players[player].is_barbarian() {
            return;
        }
        let has_city = self.player_cities(player).next().is_some();
        let has_settler = self
            .player_units(player)
            .any(|u| self.unit_type(u).found_city);
        if !has_city
            && (self.turn > 1 && !has_settler
                || self.players[player].cities_founded > 0 && !has_settler)
        {
            self.players[player].alive = false;
            let units: Vec<UnitId> = self.player_units(player).map(|u| u.id).collect();
            for id in units {
                self.units.remove(&id);
            }
            let name = self.players[player].civ_name.clone();
            for p in 0..self.players.len() {
                if p != player && self.players[p].is_major() {
                    self.notify(p, format!("{name} has been destroyed."), None);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::damage;

    #[test]
    fn matches_civ6_damage() {
        assert_eq!(damage(30, 30, 0), 24);
        assert_eq!(damage(30, 30, 11), 35);
        assert_eq!(damage(40, 30, 6), 44);
        assert_eq!(damage(100, 10, 11), 100);
        assert_eq!(damage(10, 100, 0), 1);
    }
}

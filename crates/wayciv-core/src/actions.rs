//! Unit orders other than moving and attacking.

use crate::events::Event;
use crate::hex::Pos;
use crate::map::PlayerId;
use crate::state::{Game, UnitId};

/// An order the selected unit can be given (for the UI's action bar).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitAction {
    FoundCity,
    Build(usize),
    Fortify,
    Sleep,
    Skip,
    Delete,
    FoundReligion,
    SpreadReligion,
    Activate,
    /// Start a trade route from this city to the given city.
    TradeRoute(crate::map::CityId),
}

/// Cities can't be founded within this many tiles of another city (Civ VI: 3).
pub const MIN_CITY_DISTANCE: i32 = 4;

impl Game {
    pub fn can_found_city(&self, unit: UnitId) -> bool {
        let Some(u) = self.units.get(&unit) else {
            return false;
        };
        let rules = &self.rules;
        self.unit_type(u).found_city
            && u.moves > 0
            && self.map.is_land(rules, u.pos)
            && !self.map.is_impassable(rules, u.pos)
            && self.map.tile(u.pos).owner.is_none_or(|o| o == u.owner)
            && self
                .cities
                .values()
                .all(|c| self.map.grid.distance(c.pos, u.pos) >= MIN_CITY_DISTANCE)
    }

    pub fn found_city(&mut self, unit: UnitId) -> Result<(), String> {
        if !self.can_found_city(unit) {
            return Err("A city can't be founded here.".into());
        }
        let u = self.units.remove(&unit).unwrap();
        let coastal = self.map.is_coastal_land(&self.rules, u.pos);
        let id = self.found_city_at(u.owner, u.pos, None);
        if coastal {
            self.check_boosts(u.owner, crate::research::Trigger::SettledCoast);
        }
        self.update_visibility(u.owner);
        let _ = id;
        Ok(())
    }

    /// Improvements a builder could make on `pos` for `player`.
    pub fn valid_improvements(&self, player: PlayerId, pos: Pos) -> Vec<usize> {
        let rules = &self.rules;
        let content = &rules.content;
        let tile = self.map.tile(pos);
        if tile.owner != Some(player)
            || tile.district.is_some()
            || tile.wonder.is_some()
            || self.map.is_water(rules, pos)
        {
            return Vec::new();
        }
        let p = &self.players[player];
        let terrain = &content.terrains[tile.terrain].id;
        let feature = tile.feature.map(|f| &content.features[f]);
        let resource = tile
            .resource
            .map(|r| &content.resources[r])
            .filter(|r| p.meets(rules, r.prereq_tech.as_deref()));
        (0..content.improvements.len())
            .filter(|&i| {
                let imp = &content.improvements[i];
                if !imp.buildable || Some(i) == tile.improvement && !tile.pillaged {
                    return false;
                }
                if !p.meets(rules, imp.prereq_tech.as_deref()) || !self.reached(imp.unlock_year) {
                    return false;
                }
                if let Some(owner) = &imp.unique_to {
                    let civ = &content.civilizations[p.civ].id;
                    let leader = &content.leaders[p.leader].id;
                    if owner != civ && owner != leader {
                        return false;
                    }
                }
                let on_resource = resource.is_some_and(|r| imp.valid_resources.contains(&r.id));
                let on_feature = feature.is_some_and(|f| imp.valid_features.contains(&f.id));
                let on_terrain = imp.valid_terrains.contains(terrain)
                    && feature.is_none_or(|f| f.removable && !f.natural_wonder);
                if resource.is_some()
                    && !imp.valid_resources.is_empty()
                    && !on_resource
                    && !on_feature
                    && !on_terrain
                {
                    return false;
                }
                on_resource || on_feature || on_terrain
            })
            .collect()
    }

    pub fn build_improvement(&mut self, unit: UnitId, improvement: usize) -> Result<(), String> {
        let u = self.units.get(&unit).ok_or("No such unit.")?.clone();
        if u.charges <= 0 || u.moves <= 0 || self.unit_type(&u).build_charges == 0 {
            return Err("This unit can't build improvements now.".into());
        }
        if !self
            .valid_improvements(u.owner, u.pos)
            .contains(&improvement)
        {
            return Err("That improvement can't go here.".into());
        }
        let rules = self.rules.clone();
        let imp = &rules.content.improvements[improvement];
        let tile = self.map.tile_mut(u.pos);
        if let Some(f) = tile.feature
            && !imp.valid_features.contains(&rules.content.features[f].id)
        {
            tile.feature = None;
        }
        tile.improvement = Some(improvement);
        tile.pillaged = false;
        let owner = u.owner;
        self.emit(Event::Improved {
            unit: Box::new(u.clone()),
            pos: u.pos,
        });
        let unit_ref = self.units.get_mut(&unit).unwrap();
        unit_ref.charges -= 1;
        unit_ref.moves = 0;
        if unit_ref.charges <= 0 {
            self.units.remove(&unit);
        }
        self.check_boosts(owner, crate::research::Trigger::Improved(improvement));
        self.check_boosts(owner, crate::research::Trigger::Poll);
        if let Some(city) = self.map.tile(u.pos).owning_city {
            self.assign_tiles(city);
        }
        Ok(())
    }

    pub fn fortify(&mut self, unit: UnitId) {
        if let Some(u) = self.units.get_mut(&unit) {
            u.fortified = true;
            u.path.clear();
            u.moves = 0;
        }
    }

    pub fn sleep(&mut self, unit: UnitId) {
        if let Some(u) = self.units.get_mut(&unit) {
            u.sleeping = true;
            u.path.clear();
        }
    }

    pub fn skip(&mut self, unit: UnitId) {
        if let Some(u) = self.units.get_mut(&unit) {
            u.skipped = true;
        }
    }

    pub fn delete_unit(&mut self, unit: UnitId) {
        if let Some(u) = self.units.remove(&unit) {
            self.check_elimination(u.owner);
        }
    }

    /// Orders the UI offers for a unit.
    pub fn unit_actions(&self, unit: UnitId) -> Vec<UnitAction> {
        let Some(u) = self.units.get(&unit) else {
            return Vec::new();
        };
        let ty = self.unit_type(u);
        let mut actions = Vec::new();
        if ty.found_city {
            actions.push(UnitAction::FoundCity);
        }
        if ty.build_charges > 0 && u.charges > 0 {
            actions.extend(
                self.valid_improvements(u.owner, u.pos)
                    .into_iter()
                    .map(UnitAction::Build),
            );
        }
        if ty.found_religion && self.can_found_religion(unit) {
            actions.push(UnitAction::FoundReligion);
        }
        if u.religion.is_some() && u.charges > 0 {
            actions.push(UnitAction::SpreadReligion);
        }
        if ty.make_trade_route {
            let used = self.trade_routes_of(u.owner).count() as i32;
            if used < self.trade_capacity(u.owner) {
                actions.extend(
                    self.trade_destinations(unit)
                        .into_iter()
                        .map(UnitAction::TradeRoute),
                );
            }
        }
        if u.great_person.is_some() && !ty.found_religion {
            actions.push(UnitAction::Activate);
        }
        if self.is_military(u) {
            actions.push(UnitAction::Fortify);
        } else {
            actions.push(UnitAction::Sleep);
        }
        actions.push(UnitAction::Skip);
        actions.push(UnitAction::Delete);
        actions
    }

    pub fn perform(&mut self, unit: UnitId, action: &UnitAction) -> Result<(), String> {
        match action {
            UnitAction::FoundCity => self.found_city(unit),
            UnitAction::Build(i) => self.build_improvement(unit, *i),
            UnitAction::Fortify => {
                self.fortify(unit);
                Ok(())
            }
            UnitAction::Sleep => {
                self.sleep(unit);
                Ok(())
            }
            UnitAction::Skip => {
                self.skip(unit);
                Ok(())
            }
            UnitAction::Delete => {
                self.delete_unit(unit);
                Ok(())
            }
            UnitAction::FoundReligion => {
                let religion = self
                    .free_religions()
                    .first()
                    .copied()
                    .ok_or("No religions left to found.")?;
                let (founder, follower) = self.default_beliefs();
                self.found_religion(unit, religion, founder, follower)
            }
            UnitAction::SpreadReligion => self.spread_religion(unit),
            UnitAction::Activate => self.activate_great_person(unit),
            UnitAction::TradeRoute(city) => self.establish_trade_route(unit, *city),
        }
    }

    pub fn action_name(&self, action: &UnitAction) -> String {
        match action {
            UnitAction::FoundCity => "Found City".into(),
            UnitAction::Build(i) => format!("Build {}", self.rules.content.improvements[*i].name),
            UnitAction::Fortify => "Fortify".into(),
            UnitAction::Sleep => "Sleep".into(),
            UnitAction::Skip => "Skip Turn".into(),
            UnitAction::Delete => "Disband".into(),
            UnitAction::FoundReligion => "Found Religion".into(),
            UnitAction::SpreadReligion => "Spread Religion".into(),
            UnitAction::Activate => "Activate".into(),
            UnitAction::TradeRoute(city) => {
                let Some(c) = self.cities.get(city) else {
                    return "Trade Route".into();
                };
                // Only the human's units show action names.
                let y = self.route_yields(self.human(), *city);
                let parts: Vec<String> = (0..6)
                    .filter(|&i| y[i] > 0.0)
                    .map(|i| format!("+{:.0} {}", y[i], crate::rules::YIELD_NAMES[i]))
                    .collect();
                format!("Trade route to {} ({})", c.name, parts.join(", "))
            }
        }
    }
}

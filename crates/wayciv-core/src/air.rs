//! Aircraft: based at Aerodromes, rebased between them, and striking
//! targets within their operation range. Anti-air units near the target and
//! enemy fighters whose range covers it fire back at attacking aircraft.

use crate::hex::Pos;
use crate::map::{CityId, PlayerId};
use crate::state::{Game, Unit, UnitId};

/// Aircraft one Aerodrome can base.
pub const AERODROME_SLOTS: usize = 4;

impl Game {
    /// Each of the player's Aerodromes: (city, tile).
    pub fn air_bases(&self, player: PlayerId) -> Vec<(CityId, Pos)> {
        let content = &self.rules.content;
        self.player_cities(player)
            .flat_map(|c| {
                c.districts.iter().filter_map(move |d| {
                    let district = &content.districts[d.district];
                    let aerodrome = district.id == "DISTRICT_AERODROME" || district.replaces.as_deref() == Some("DISTRICT_AERODROME");
                    aerodrome.then_some((c.id, d.pos))
                })
            })
            .collect()
    }

    pub fn aircraft_at(&self, pos: Pos) -> usize {
        self.units.values().filter(|u| u.pos == pos && self.is_air(u)).count()
    }

    /// The city's Aerodrome tile, if it has a free slot.
    pub fn free_air_base(&self, city: CityId) -> Option<Pos> {
        let owner = self.cities.get(&city)?.owner;
        self.air_bases(owner).into_iter().find(|(c, pos)| *c == city && self.aircraft_at(*pos) < AERODROME_SLOTS).map(|(_, p)| p)
    }

    /// Aerodromes this aircraft can rebase to.
    pub fn rebase_targets(&self, unit: &Unit) -> Vec<Pos> {
        let range = self.unit_type(unit).range.max(1) * 3;
        self.air_bases(unit.owner)
            .into_iter()
            .map(|(_, p)| p)
            .filter(|&p| p != unit.pos && self.aircraft_at(p) < AERODROME_SLOTS && self.map.grid.distance(p, unit.pos) <= range)
            .collect()
    }

    pub fn rebase(&mut self, id: UnitId, to: Pos) -> Result<(), String> {
        let unit = self.units.get(&id).ok_or("No such unit.")?.clone();
        if unit.moves <= 0 || !self.rebase_targets(&unit).contains(&to) {
            return Err("Aircraft can only rebase to an Aerodrome with a free slot within range.".into());
        }
        let u = self.units.get_mut(&id).unwrap();
        u.pos = to;
        u.moves = 0;
        u.fortified = false;
        self.update_visibility(unit.owner);
        Ok(())
    }

    /// Strength of the defenses an aircraft meets striking `target`: the best
    /// anti-air unit on or next to it, or an enemy fighter covering it.
    pub fn air_defense(&self, attacker: &Unit, target: Pos) -> Option<i32> {
        let grid = self.map.grid;
        let hostile = |owner: PlayerId| self.players[attacker.owner].at_war_with(owner);
        let anti_air = grid
            .within(target, 1)
            .into_iter()
            .flat_map(|p| self.ground_units_at(p))
            .filter(|u| hostile(u.owner))
            .map(|u| self.unit_type(u).anti_air)
            .max()
            .filter(|&s| s > 0);
        let fighter = self
            .units
            .values()
            .filter(|u| hostile(u.owner) && self.is_air(u) && self.unit_type(u).ranged_combat > 0)
            .filter(|u| grid.distance(u.pos, target) <= self.unit_type(u).range)
            .map(|u| self.unit_type(u).combat * u.hp / 100)
            .max();
        anti_air.into_iter().chain(fighter).max()
    }

    /// Aircraft based in a city's tiles are lost when the city falls.
    pub(crate) fn destroy_based_aircraft(&mut self, city: CityId, owner: PlayerId) {
        let tiles: Vec<Pos> = self.city_tiles(city);
        let lost: Vec<UnitId> = self.units.values().filter(|u| u.owner == owner && self.is_air(u) && tiles.contains(&u.pos)).map(|u| u.id).collect();
        for id in lost {
            self.units.remove(&id);
        }
    }
}

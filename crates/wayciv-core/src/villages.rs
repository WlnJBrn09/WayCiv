//! Tribal villages: scattered over the unclaimed land at the start. The
//! first major civilization's land unit to enter one gets a gift: Gold,
//! Faith, a Eureka, a new citizen, a map of the region, a free Builder or
//! Scout, or a veteran promotion.

use crate::hex::Pos;
use crate::map::PlayerId;
use crate::research::Tree;
use crate::state::{Game, UnitId};

const VILLAGE: &str = "IMPROVEMENT_GOODY_HUT";

impl Game {
    fn village_improvement(&self) -> Option<usize> {
        self.rules
            .idx(VILLAGE)
            .filter(|&i| self.rules.content.improvements.get(i).is_some_and(|x| x.id == VILLAGE))
    }

    /// Scatters villages over open land, away from the starts and each
    /// other: about one per 110 land tiles.
    pub(crate) fn place_villages(&mut self, starts: &[Pos]) {
        let Some(village) = self.village_improvement() else {
            return;
        };
        let rules = self.rules.clone();
        let grid = self.map.grid;
        let mut sites: Vec<Pos> = (0..grid.len())
            .map(|i| grid.pos(i))
            .filter(|&p| {
                let tile = self.map.tile(p);
                self.map.is_land(&rules, p)
                    && !self.map.is_impassable(&rules, p)
                    && tile.owner.is_none()
                    && tile.improvement.is_none()
                    && tile
                        .feature
                        .is_none_or(|f| !rules.content.features[f].natural_wonder)
                    && p.y > 2
                    && p.y < grid.height - 3
                    && starts.iter().all(|&s| grid.distance(s, p) >= 4)
            })
            .collect();
        let land = sites.len();
        self.rng.shuffle(&mut sites);
        let wanted = land / 110;
        let mut placed: Vec<Pos> = Vec::new();
        for p in sites {
            if placed.len() >= wanted {
                break;
            }
            if placed.iter().all(|&q| grid.distance(p, q) >= 5)
                && self.camps.keys().all(|&c| grid.distance(c, p) >= 3)
            {
                placed.push(p);
            }
        }
        for p in placed {
            self.map.tile_mut(p).improvement = Some(village);
            self.villages.insert(p);
        }
    }

    /// A major's land unit entering a village receives its gift.
    pub(crate) fn check_village(&mut self, id: UnitId, pos: Pos) {
        if !self.villages.contains(&pos) {
            return;
        }
        let Some(unit) = self.units.get(&id).cloned() else {
            return;
        };
        let owner = unit.owner;
        if !self.players[owner].is_major() || self.unit_type(&unit).domain != "DOMAIN_LAND" {
            return;
        }
        self.villages.remove(&pos);
        if self.map.tile(pos).improvement == self.village_improvement() {
            self.map.tile_mut(pos).improvement = None;
        }
        let gift = self.village_gift(owner, id, pos);
        self.notify(owner, format!("Tribal village: {gift}"), Some(pos));
        self.first_moment(
            owner,
            "village",
            "Welcomed by a tribal village",
            crate::eras::score::VILLAGE,
        );
    }

    /// Rolls and grants a gift; returns its description.
    fn village_gift(&mut self, owner: PlayerId, unit: UnitId, pos: Pos) -> String {
        let era = self.world_era as i32;
        let roll = self.rng.below(100);
        // A Eureka for something researchable and not yet boosted.
        if roll < 22 {
            let options: Vec<usize> = (0..self.players[owner].techs.len())
                .filter(|&i| {
                    self.researchable(owner, Tree::Tech, i)
                        && !self.players[owner].tech_boosted[i]
                })
                .collect();
            if let Some(&tech) = self.rng.pick(&options) {
                self.boost(owner, Tree::Tech, tech, 40);
                let name = self.rules.content.techs[tech].name.clone();
                return format!("its elders share their knowledge. Eureka for {name}!");
            }
        }
        if roll < 30 {
            let near = self
                .player_cities(owner)
                .min_by_key(|c| self.map.grid.distance(c.pos, pos))
                .map(|c| c.id);
            if let Some(city) = near {
                let c = self.cities.get_mut(&city).unwrap();
                c.population += 1;
                return format!("villagers settle in {}. +1 Population.", c.name);
            }
        }
        if roll < 42 {
            let faith = 20 + 10 * era + self.rng.below(15) as i32;
            self.players[owner].faith += faith;
            return format!("its shamans bless your people. +{faith} Faith.");
        }
        if roll < 52 {
            let grid = self.map.grid;
            for p in grid.within(pos, 7) {
                self.players[owner].revealed[grid.index(p)] = true;
            }
            return "its hunters map the lands around it.".into();
        }
        if roll < 60 {
            let kind = if self.rng.chance(0.5) {
                "UNIT_BUILDER"
            } else {
                "UNIT_SCOUT"
            };
            if self.spawn(owner, kind, pos).is_some() {
                let name = self
                    .rules
                    .idx(kind)
                    .map(|i| self.rules.content.units[i].name.clone())
                    .unwrap_or_default();
                return format!("a {name} joins you.");
            }
        }
        if roll < 68 && self.is_military(&self.units[&unit]) {
            let u = self.units.get_mut(&unit).unwrap();
            u.level += 1;
            u.hp = 100;
            return "its warriors train your unit. It gains a level.".into();
        }
        let gold = 40 + 25 * era + self.rng.below(40) as i32;
        self.players[owner].gold += gold;
        format!("a gift of {gold} Gold.")
    }
}

#[cfg(test)]
mod tests {
    use crate::{Settings, load_rules, new_game};

    #[test]
    fn villages_are_placed_and_give_gifts() {
        let mut g = new_game(
            load_rules().unwrap(),
            Settings {
                seed: 9,
                opponents: 2,
                city_states: 0,
                ..Default::default()
            },
        );
        assert!(g.villages.len() > 10, "{} villages", g.villages.len());
        let me = g.human();
        let village = *g.villages.iter().next().unwrap();
        let unit = g.player_units(me).next().unwrap().id;
        g.units.get_mut(&unit).unwrap().pos = village;
        let before = (g.players[me].gold, g.players[me].faith);
        g.check_village(unit, village);
        assert!(!g.villages.contains(&village));
        assert!(g.map.tile(village).improvement.is_none());
        let notes = g.notifications.iter().filter(|n| n.text.starts_with("Tribal village")).count();
        assert_eq!(notes, 1);
        let _ = before;
    }
}

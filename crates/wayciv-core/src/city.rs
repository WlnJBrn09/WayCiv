//! Cities: founding, borders, worked tiles, yields, growth, and production.

use crate::hex::Pos;
use crate::map::{CityId, PlayerId};
use crate::rules::{CULTURE, FAITH, FOOD, GOLD, PRODUCTION, SCIENCE};
use crate::state::{City, CityDistrict, Game, Production};
use std::collections::HashMap;

/// Tiles a city can work and place districts on.
pub const CITY_RANGE: i32 = 3;

/// Per-turn yields of a city, with fractional amounts.
pub type CityYields = [f32; 6];

/// A city's breakdown for the UI.
#[derive(Clone, Debug, Default)]
pub struct CityReport {
    pub yields: CityYields,
    pub food_consumed: f32,
    pub growth_threshold: i32,
    pub housing: i32,
    pub amenities: i32,
    pub amenities_needed: i32,
    pub turns_to_grow: Option<i32>,
}

impl Game {
    pub fn found_city_at(&mut self, owner: PlayerId, pos: Pos, name: Option<String>) -> CityId {
        let rules = self.rules.clone();
        let id = self.next_city;
        self.next_city += 1;
        let first = self.players[owner].capital.is_none() && self.players[owner].is_major();
        let name = name.unwrap_or_else(|| self.next_city_name(owner));
        let mut city = City {
            id,
            owner,
            founder: owner,
            name: name.clone(),
            pos,
            population: 1,
            food: 0,
            production: None,
            progress: HashMap::new(),
            buildings: Vec::new(),
            districts: Vec::new(),
            worked: Vec::new(),
            hp: 200,
            wall_hp: 0,
            capital: false,
            original_capital_of: None,
            founded: self.turn,
            border_culture: 0,
            followers: HashMap::new(),
            pressure: HashMap::new(),
            has_attacked: false,
            turns_since_combat: 10,
            loyalty: 100,
        };
        if let Some(center) = rules.idx("DISTRICT_CITY_CENTER") {
            city.districts.push(CityDistrict {
                district: center,
                pos,
            });
            self.map.tile_mut(pos).district = Some(center);
        }
        if first || self.players[owner].is_city_state() && self.players[owner].capital.is_none() {
            city.capital = true;
            if self.players[owner].is_major() {
                city.original_capital_of = Some(owner);
                if let Some(palace) = rules.idx("BUILDING_PALACE") {
                    city.buildings.push(palace);
                }
            }
            self.players[owner].capital = Some(id);
            self.players[owner].original_capital.get_or_insert(pos);
        }
        // Removing a forest/jungle/marsh under the city, as Civ VI does.
        let tile = self.map.tile_mut(pos);
        if tile
            .feature
            .is_some_and(|f| !rules.content.features[f].natural_wonder)
        {
            tile.feature = None;
        }
        tile.improvement = None;
        self.cities.insert(id, city);
        for p in self.map.grid.within(pos, 1) {
            if self.map.tile(p).owner.is_none() {
                self.claim(id, p);
            }
        }
        self.claim(id, pos);
        self.players[owner].cities_founded += 1;
        self.assign_tiles(id);
        if self.players[owner].is_major() {
            self.notify(owner, format!("{name} has been founded."), Some(pos));
        }
        id
    }

    fn next_city_name(&self, owner: PlayerId) -> String {
        let p = &self.players[owner];
        let civ = &self.rules.content.civilizations[p.civ];
        let used: Vec<&str> = self.cities.values().map(|c| c.name.as_str()).collect();
        let leader = &self.rules.content.leaders[p.leader];
        let capital = (!leader.capital.is_empty()
            && !used.contains(&leader.capital.as_str())
            && p.cities_founded == 0)
            .then(|| leader.capital.clone());
        capital
            .or_else(|| {
                civ.city_names
                    .iter()
                    .find(|n| !used.contains(&n.as_str()) && !n.starts_with("LOC_"))
                    .cloned()
            })
            .unwrap_or_else(|| format!("{} {}", civ.adjective, p.cities_founded + 1))
    }

    pub fn claim(&mut self, city: CityId, pos: Pos) {
        let owner = self.cities[&city].owner;
        let tile = self.map.tile_mut(pos);
        tile.owner = Some(owner);
        tile.owning_city = Some(city);
    }

    pub fn city_tiles(&self, city: CityId) -> Vec<Pos> {
        let c = &self.cities[&city];
        self.map
            .grid
            .within(c.pos, CITY_RANGE)
            .into_iter()
            .filter(|&p| self.map.tile(p).owning_city == Some(city))
            .collect()
    }

    /// Yields of a tile for its owner, including improvement and visible resource.
    pub fn tile_yields(&self, player: PlayerId, pos: Pos) -> [i32; 6] {
        let tile = self.map.tile(pos);
        let visible = tile.resource.is_none_or(|r| {
            let res = &self.rules.content.resources[r];
            self.players[player].meets(&self.rules, res.prereq_tech.as_deref(), None)
        });
        let mut y = self.map.base_yields(&self.rules, pos, visible);
        if tile.district.is_some() && self.city_at(pos).is_none() {
            y = [0; 6];
        }
        y
    }

    /// Picks the best tiles for the city's citizens, favoring food.
    pub fn assign_tiles(&mut self, city: CityId) {
        let c = &self.cities[&city];
        let (owner, center, population) = (c.owner, c.pos, c.population);
        let mut candidates: Vec<(Pos, f32)> = self
            .city_tiles(city)
            .into_iter()
            .filter(|&p| {
                p != center
                    && self.map.tile(p).district.is_none()
                    && self.map.tile(p).wonder.is_none()
            })
            .filter(|&p| {
                !self
                    .ground_units_at(p)
                    .any(|u| self.players[owner].at_war_with(u.owner))
            })
            .map(|p| {
                let y = self.tile_yields(owner, p);
                (
                    p,
                    y[FOOD] as f32 * 1.4
                        + y[PRODUCTION] as f32 * 1.2
                        + y[GOLD] as f32 * 0.6
                        + (y[SCIENCE] + y[CULTURE] + y[FAITH]) as f32 * 0.8,
                )
            })
            .collect();
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let worked = candidates
            .into_iter()
            .take(population as usize)
            .map(|(p, _)| p)
            .collect();
        self.cities.get_mut(&city).unwrap().worked = worked;
    }

    pub fn housing(&self, city: &City) -> i32 {
        let rules = &self.rules;
        let fresh_water = self.map.tile(city.pos).river
            || self.map.grid.neighbors(city.pos).any(|n| {
                self.map.tile(n).river
                    || self
                        .map
                        .tile(n)
                        .feature
                        .is_some_and(|f| rules.content.features[f].id == "FEATURE_OASIS")
            });
        let water = if fresh_water {
            5
        } else if self.map.is_coastal_land(rules, city.pos) {
            3
        } else {
            2
        };
        let buildings: i32 = city
            .buildings
            .iter()
            .map(|&b| rules.content.buildings[b].housing)
            .sum();
        let districts: i32 = city
            .districts
            .iter()
            .map(|d| rules.content.districts[d.district].housing)
            .sum();
        let farms = self
            .city_tiles(city.id)
            .iter()
            .filter(|&&p| {
                self.map
                    .tile(p)
                    .improvement
                    .is_some_and(|i| rules.content.improvements[i].id == "IMPROVEMENT_FARM")
            })
            .count() as i32;
        water + buildings + districts + farms / 2 + self.bonuses(city.owner).housing
    }

    /// Luxury resources a player has improved, each worth one amenity in
    /// up to four cities.
    pub fn luxuries(&self, player: PlayerId) -> Vec<usize> {
        let mut found: Vec<usize> = Vec::new();
        for pos in self.map.grid.all() {
            let tile = self.map.tile(pos);
            if tile.owner != Some(player) || tile.improvement.is_none() || tile.pillaged {
                continue;
            }
            if let Some(r) = tile.resource
                && self.rules.content.resources[r].class == "RESOURCECLASS_LUXURY"
                && !found.contains(&r)
            {
                found.push(r);
            }
        }
        found
    }

    /// Whether `player` has an improved tile with `resource` (Civ VI's base
    /// rule for strategic resources).
    pub fn has_resource(&self, player: PlayerId, resource: &str) -> bool {
        let content = &self.rules.content;
        self.map.tiles.iter().any(|t| {
            t.owner == Some(player)
                && t.improvement.is_some()
                && !t.pillaged
                && t.resource
                    .is_some_and(|r| content.resources[r].id == resource)
        })
    }

    pub fn amenities(&self, city: &City) -> (i32, i32) {
        let rules = &self.rules;
        let needed = (city.population - 1).max(0) / 2 + i32::from(city.population > 0);
        let cities = self.player_cities(city.owner).count().max(1) as i32;
        let luxuries = self.luxuries(city.owner).len() as i32 * 4;
        let from_luxuries = (luxuries + cities - 1) / cities;
        let buildings: i32 = city
            .buildings
            .iter()
            .map(|&b| rules.content.buildings[b].amenities)
            .sum();
        let districts: i32 = city
            .districts
            .iter()
            .map(|d| rules.content.districts[d.district].amenities)
            .sum();
        let war_weariness = if self.real_wars(city.owner) > 0 && self.turn > 50 {
            1
        } else {
            0
        };
        (
            from_luxuries.min(4) + buildings + districts + self.bonuses(city.owner).amenities
                - war_weariness,
            needed,
        )
    }

    pub fn growth_threshold(population: i32) -> i32 {
        let p = (population - 1).max(0) as f32;
        (15.0 + 8.0 * p + p.powf(1.5)).floor() as i32
    }

    /// Adjacency bonus yields of a district on `pos`.
    pub fn adjacency(&self, district: usize, pos: Pos) -> [f32; 6] {
        let rules = &self.rules;
        let d = &rules.content.districts[district];
        let mut out = [0f32; 6];
        for adj in &d.adjacencies {
            let mut count = 0;
            for n in self.map.grid.neighbors(pos) {
                let tile = self.map.tile(n);
                let terrain = &rules.content.terrains[tile.terrain].id;
                let feature = tile.feature.map(|f| &rules.content.features[f]);
                let hit = adj.adjacent_terrain.as_ref().is_some_and(|t| t == terrain)
                    || adj
                        .adjacent_feature
                        .as_ref()
                        .is_some_and(|f| feature.is_some_and(|x| &x.id == f))
                    || adj.adjacent_district.as_ref().is_some_and(|ad| {
                        tile.district
                            .is_some_and(|x| &rules.content.districts[x].id == ad)
                    })
                    || (adj.other_district && tile.district.is_some())
                    || (adj.adjacent_wonder && tile.wonder.is_some())
                    || (adj.adjacent_natural_wonder && feature.is_some_and(|f| f.natural_wonder))
                    || (adj.adjacent_resource && tile.resource.is_some());
                if hit {
                    count += 1;
                }
            }
            if adj.adjacent_river && self.map.tile(pos).river {
                count += 1;
            }
            let required = adj.tiles_required.max(1);
            out[adj.yield_index] += (adj.amount * (count / required)) as f32;
        }
        out
    }

    /// The city's yields and growth figures.
    pub fn city_report(&self, id: CityId) -> CityReport {
        let rules = &self.rules;
        let city = &self.cities[&id];
        let owner = city.owner;
        let bonuses = self.bonuses(owner);
        let mut y = [0f32; 6];
        let center = self.tile_yields(owner, city.pos);
        let center = [
            center[0].max(2),
            center[1].max(1),
            center[2],
            center[3],
            center[4],
            center[5],
        ];
        for i in 0..6 {
            y[i] += center[i] as f32;
        }
        // WayCiv rule: the city center adds +2 Food and +2 Production.
        y[FOOD] += 2.0;
        y[PRODUCTION] += 2.0;
        for &p in &city.worked {
            let t = self.tile_yields(owner, p);
            for i in 0..6 {
                y[i] += t[i] as f32;
            }
        }
        for &b in &city.buildings {
            for i in 0..6 {
                y[i] += rules.content.buildings[b].yields[i] as f32;
            }
        }
        for d in &city.districts {
            let adjacency = self.adjacency(d.district, d.pos);
            for i in 0..6 {
                y[i] += adjacency[i] * bonuses.adjacency_multiplier;
            }
        }
        // WayCiv rule: every citizen adds Science and Culture, more in later
        // eras (standing in for Civ VI's specialists and building chains).
        let era = self.era_of(owner) as f32;
        y[SCIENCE] += city.population as f32 * (0.8 + 0.25 * era);
        y[CULTURE] += city.population as f32 * (0.5 + 0.12 * era);
        // Taxes: +1 Gold per 2 citizens.
        y[GOLD] += city.population as f32 * 0.5;
        for i in 0..6 {
            y[i] += bonuses.city_flat[i];
            if city.capital {
                y[i] += bonuses.capital_flat[i];
            }
        }
        for (district, yields) in &bonuses.district_flat {
            if city.has_district(*district) {
                for i in 0..6 {
                    y[i] += yields[i];
                }
            }
        }
        for (building, yields) in &bonuses.building_flat {
            if city.has_building(*building) {
                for i in 0..6 {
                    y[i] += yields[i];
                }
            }
        }
        y = self.city_state_bonus(owner, city, y);
        let trade = self.city_trade_yields(id);
        for i in 0..6 {
            y[i] += trade[i];
        }

        let (amenities, needed) = self.amenities(city);
        let mood = match amenities - needed {
            d if d >= 3 => 0.2,
            d if d >= 1 => 0.1,
            0 => 0.0,
            d if d >= -2 => -0.05,
            d if d >= -4 => -0.15,
            _ => -0.3,
        };
        for i in 1..6 {
            y[i] *= 1.0 + mood + bonuses.city_percent[i] / 100.0;
            if city.capital {
                y[i] *= 1.0 + bonuses.capital_percent[i] / 100.0;
            }
        }
        let food_consumed = city.population as f32 * 2.0;
        let housing = self.housing(city);
        let mut surplus = y[FOOD] - food_consumed;
        if surplus > 0.0 {
            surplus *= 1.0 + bonuses.city_percent[FOOD] / 100.0;
            if city.population >= housing + 4 {
                surplus = 0.0;
            } else if city.population >= housing {
                surplus *= 0.25;
            } else if city.population == housing - 1 {
                surplus *= 0.5;
            }
            if amenities - needed < -4 {
                surplus = 0.0;
            }
        }
        let threshold = self.scaled(Self::growth_threshold(city.population));
        let turns_to_grow = (surplus > 0.0)
            .then(|| ((threshold - city.food) as f32 / surplus).ceil().max(1.0) as i32);
        y[FOOD] = surplus + food_consumed;
        CityReport {
            yields: y,
            food_consumed,
            growth_threshold: threshold,
            housing,
            amenities,
            amenities_needed: needed,
            turns_to_grow,
        }
    }

    /// Key into `City::progress` for an item.
    pub fn production_key(item: &Production) -> String {
        match item {
            Production::Unit(i) => format!("U{i}"),
            Production::Building(i) => format!("B{i}"),
            Production::District(i, _) => format!("D{i}"),
            Production::Wonder(i, _) => format!("W{i}"),
            Production::Project(i) => format!("P{i}"),
        }
    }

    /// Fraction of the tech and civic trees researched (0..1).
    pub fn game_progress(&self, player: PlayerId) -> f32 {
        let p = &self.players[player];
        let techs = p.techs.iter().filter(|&&t| t).count() as f32 / p.techs.len().max(1) as f32;
        let civics = p.civics.iter().filter(|&&t| t).count() as f32 / p.civics.len().max(1) as f32;
        techs.max(civics)
    }

    pub fn production_cost(&self, player: PlayerId, item: &Production) -> i32 {
        let content = &self.rules.content;
        let base = match item {
            Production::Unit(i) => content.units[*i].cost,
            Production::Building(i) | Production::Wonder(i, _) => content.buildings[*i].cost,
            Production::District(i, _) => {
                let base = content.districts[*i].cost as f32;
                (base * (1.0 + 4.0 * self.game_progress(player))).round() as i32
            }
            Production::Project(i) => (content.projects[*i].cost as f32
                * self.project_cost_multiplier(player))
            .round() as i32,
        };
        self.scaled(base)
    }

    /// Production per turn in `city` toward `item`, including bonuses.
    pub fn production_toward(&self, city: CityId, item: &Production) -> f32 {
        let c = &self.cities[&city];
        let base = self.city_report(city).yields[PRODUCTION];
        let bonuses = self.bonuses(c.owner);
        let content = &self.rules.content;
        let percent = match item {
            Production::Unit(i) => {
                let unit = &content.units[*i];
                let era = self.era_of(c.owner);
                bonuses
                    .unit_production
                    .iter()
                    .filter(|b| b.matches(unit, era, &self.rules))
                    .map(|b| b.percent)
                    .sum::<f32>()
            }
            Production::Building(_) => bonuses.building_production,
            Production::Wonder(_, _) => bonuses.wonder_production,
            Production::District(_, _) => bonuses.district_production,
            Production::Project(_) => 0.0,
        };
        base * (1.0 + percent / 100.0)
    }

    pub fn item_name(&self, item: &Production) -> String {
        let content = &self.rules.content;
        match item {
            Production::Unit(i) => content.units[*i].name.clone(),
            Production::Building(i) | Production::Wonder(i, _) => {
                content.buildings[*i].name.clone()
            }
            Production::District(i, _) => content.districts[*i].name.clone(),
            Production::Project(i) => content.projects[*i].name.clone(),
        }
    }

    /// Gold price to buy `item` outright (Civ VI charges about twice the
    /// production cost, minus progress already made).
    pub fn purchase_cost(&self, city: CityId, item: &Production) -> Option<i32> {
        if matches!(
            item,
            Production::District(..) | Production::Wonder(..) | Production::Project(_)
        ) {
            return None;
        }
        let c = &self.cities[&city];
        let cost = self.production_cost(c.owner, item);
        let done = c
            .progress
            .get(&Self::production_key(item))
            .copied()
            .unwrap_or(0);
        Some(
            ((cost - done).max(0) as f32 * 2.0 * self.bonuses(c.owner).purchase_multiplier).ceil()
                as i32,
        )
    }

    pub fn purchase(&mut self, city: CityId, item: Production) -> Result<(), String> {
        let price = self
            .purchase_cost(city, &item)
            .ok_or("This can't be purchased.")?;
        let owner = self.cities[&city].owner;
        if self.players[owner].gold < price {
            return Err(format!("Not enough Gold ({price} needed)."));
        }
        if !self.can_produce(city, &item) {
            return Err("This city can't build that.".into());
        }
        self.players[owner].gold -= price;
        // Buying something other than the current build keeps that build.
        let current = self.cities[&city].production.clone().filter(|c| *c != item);
        self.complete_production(city, item);
        if current.is_some() {
            self.cities.get_mut(&city).unwrap().production = current;
        }
        Ok(())
    }

    /// Everything the city can build now, without tile choices for districts
    /// and wonders (see `district_sites`).
    pub fn available_production(&self, city: CityId) -> Vec<Production> {
        let content = &self.rules.content;
        let mut items = Vec::new();
        for i in 0..content.units.len() {
            let item = Production::Unit(i);
            if self.can_produce(city, &item) {
                items.push(item);
            }
        }
        for i in 0..content.buildings.len() {
            let wonder = content.buildings[i].wonder;
            let item = if wonder {
                Production::Wonder(i, Pos::default())
            } else {
                Production::Building(i)
            };
            if self.can_produce_ignoring_site(city, &item) {
                items.push(item);
            }
        }
        for i in 0..content.districts.len() {
            let item = Production::District(i, Pos::default());
            if self.can_produce_ignoring_site(city, &item) {
                items.push(item);
            }
        }
        for i in 0..content.projects.len() {
            let item = Production::Project(i);
            if self.can_produce(city, &item) {
                items.push(item);
            }
        }
        items
    }

    pub fn can_produce(&self, city: CityId, item: &Production) -> bool {
        match item {
            Production::District(d, pos) => {
                self.can_produce_ignoring_site(city, item)
                    && self.district_sites(city, *d).contains(pos)
            }
            Production::Wonder(w, pos) => {
                self.can_produce_ignoring_site(city, item)
                    && self.wonder_sites(city, *w).contains(pos)
            }
            _ => self.can_produce_ignoring_site(city, item),
        }
    }

    /// Whether `owner_type` (a civ/leader-unique owner) allows `player` to use it.
    pub fn unique_ok(&self, player: PlayerId, unique_to: &Option<String>) -> bool {
        let p = &self.players[player];
        match unique_to {
            None => true,
            Some(owner) => {
                p.is_major()
                    && (self.rules.content.civilizations[p.civ].id == *owner
                        || self.rules.content.leaders[p.leader].id == *owner)
            }
        }
    }

    /// True when the player has a unique item replacing `id`.
    fn replaced_for(&self, player: PlayerId, id: &str, kind: &str) -> bool {
        let content = &self.rules.content;
        match kind {
            "unit" => content.units.iter().any(|u| {
                u.replaces.as_deref() == Some(id)
                    && self.unique_ok(player, &u.unique_to)
                    && u.unique_to.is_some()
            }),
            "building" => content.buildings.iter().any(|b| {
                b.replaces.as_deref() == Some(id)
                    && b.unique_to.is_some()
                    && self.unique_ok(player, &b.unique_to)
            }),
            _ => content.districts.iter().any(|d| {
                d.replaces.as_deref() == Some(id)
                    && d.unique_to.is_some()
                    && self.unique_ok(player, &d.unique_to)
            }),
        }
    }

    /// Districts in `city`, with unique districts counting as what they replace.
    fn city_has_district_type(&self, city: &City, id: &str) -> bool {
        let content = &self.rules.content;
        city.districts.iter().any(|d| {
            let district = &content.districts[d.district];
            district.id == id || district.replaces.as_deref() == Some(id)
        })
    }

    fn city_has_building_type(&self, city: &City, id: &str) -> bool {
        let content = &self.rules.content;
        city.buildings.iter().any(|&b| {
            content.buildings[b].id == id || content.buildings[b].replaces.as_deref() == Some(id)
        })
    }

    pub fn can_produce_ignoring_site(&self, city: CityId, item: &Production) -> bool {
        let rules = &self.rules;
        let content = &rules.content;
        let c = &self.cities[&city];
        let owner = c.owner;
        let p = &self.players[owner];
        match item {
            Production::Unit(i) => {
                let u = &content.units[*i];
                if !self.unique_ok(owner, &u.unique_to) || self.replaced_for(owner, &u.id, "unit") {
                    return false;
                }
                if p.is_city_state() && u.unique_to.is_some() {
                    return false;
                }
                let excluded = u.found_religion
                    || u.religious_strength > 0
                    || u.spread_charges > 0
                    || !u.can_train
                    || u.must_purchase
                    || u.barbarian
                    || u.purchase_yield.as_deref() == Some("YIELD_FAITH")
                    || u.id.starts_with("UNIT_GREAT_")
                    || u.id == "UNIT_SPY"
                    || u.id == "UNIT_ARCHAEOLOGIST"
                    || u.id == "UNIT_BARBARIAN_RAIDER"
                    || u.cost <= 0;
                if excluded {
                    return false;
                }
                if u.found_city && (c.population < 2 || p.is_city_state()) {
                    return false;
                }
                if u.domain == "DOMAIN_SEA" && !self.map.is_coastal_land(rules, c.pos) {
                    return false;
                }
                // Aircraft need a free Aerodrome slot; Traders need a major civ.
                if u.domain == "DOMAIN_AIR" && self.free_air_base(city).is_none() {
                    return false;
                }
                if u.make_trade_route && !p.is_major() {
                    return false;
                }
                if !self.ai_requirement_met(owner, &u.id) {
                    return false;
                }
                if u.obsolete_tech
                    .as_deref()
                    .is_some_and(|t| p.has_tech(rules, t))
                {
                    return false;
                }
                if let Some(resource) = &u.strategic_resource
                    && !self.has_resource(owner, resource)
                {
                    return false;
                }
                p.meets(rules, u.prereq_tech.as_deref(), u.prereq_civic.as_deref())
            }
            Production::Building(i) | Production::Wonder(i, _) => {
                let b = &content.buildings[*i];
                if b.wonder != matches!(item, Production::Wonder(..)) {
                    return false;
                }
                if b.id == "BUILDING_PALACE"
                    || b.enabled_by_religion
                    || c.has_building(*i)
                    || !self.ai_requirement_met(owner, &b.id)
                {
                    return false;
                }
                if !self.unique_ok(owner, &b.unique_to)
                    || self.replaced_for(owner, &b.id, "building")
                {
                    return false;
                }
                if b.replaces
                    .as_deref()
                    .is_some_and(|r| self.city_has_building_type(c, r))
                {
                    return false;
                }
                if b.wonder
                    && (p.is_city_state()
                        || self.world_wonders.contains(i)
                        || self.cities.values().any(|other| {
                            other.production.as_ref() == Some(item)
                                && other.id != city
                                && other.owner == owner
                        }))
                {
                    return false;
                }
                if let Some(district) = &b.prereq_district
                    && !self.city_has_district_type(c, district)
                {
                    return false;
                }
                if !b.prereq_buildings.is_empty()
                    && !b
                        .prereq_buildings
                        .iter()
                        .any(|pb| self.city_has_building_type(c, pb))
                {
                    return false;
                }
                p.meets(rules, b.prereq_tech.as_deref(), b.prereq_civic.as_deref())
            }
            Production::District(i, _) => {
                let d = &content.districts[*i];
                if d.internal_only || d.city_center || p.is_city_state() {
                    return false;
                }
                if !self.unique_ok(owner, &d.unique_to)
                    || self.replaced_for(owner, &d.id, "district")
                {
                    return false;
                }
                let base_id = d.replaces.clone().unwrap_or_else(|| d.id.clone());
                if self.city_has_district_type(c, &base_id)
                    || c.production
                        .as_ref()
                        .is_some_and(|x| matches!(x, Production::District(j, _) if j == i))
                        && false
                {
                    return false;
                }
                // WayCiv rule: Aerodromes don't count toward the population
                // limit on specialty districts (like Spaceports).
                let limited = |d: &crate::rules::District| {
                    d.requires_population
                        && d.id != "DISTRICT_AERODROME"
                        && d.replaces.as_deref() != Some("DISTRICT_AERODROME")
                };
                if limited(d) {
                    let specialty = c
                        .districts
                        .iter()
                        .filter(|x| limited(&content.districts[x.district]))
                        .count() as i32;
                    if specialty >= (c.population + 2) / 3 {
                        return false;
                    }
                }
                if d.coast
                    && !self.map.grid.within(c.pos, CITY_RANGE).iter().any(|&t| {
                        self.map.is_water(rules, t) && self.map.tile(t).owning_city == Some(city)
                    })
                {
                    return false;
                }
                p.meets(rules, d.prereq_tech.as_deref(), d.prereq_civic.as_deref())
            }
            Production::Project(i) => {
                let project = &content.projects[*i];
                if !project.space_race || p.space_projects.contains(i) || p.is_city_state() {
                    return false;
                }
                if let Some(district) = &project.prereq_district
                    && !self.city_has_district_type(c, district)
                {
                    return false;
                }
                // Space projects run in order.
                let order: Vec<usize> = content
                    .projects
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| x.space_race)
                    .map(|(j, _)| j)
                    .collect();
                let position = order.iter().position(|j| j == i).unwrap_or(0);
                if order[..position]
                    .iter()
                    .any(|j| !p.space_projects.contains(j))
                    && position > 0
                    && position < 2
                {
                    return false;
                }
                p.meets(rules, project.prereq_tech.as_deref(), None)
            }
        }
    }

    /// Tiles where `city` may place `district`.
    pub fn district_sites(&self, city: CityId, district: usize) -> Vec<Pos> {
        let rules = &self.rules;
        let d = &rules.content.districts[district];
        let c = &self.cities[&city];
        self.city_tiles(city)
            .into_iter()
            .filter(|&p| p != c.pos)
            .filter(|&p| {
                let tile = self.map.tile(p);
                let water = self.map.is_water(rules, p);
                tile.district.is_none()
                    && tile.wonder.is_none()
                    && !self.map.is_impassable(rules, p)
                    && if d.coast { water && self.map.grid.neighbors(p).any(|n| self.map.is_land(rules, n)) } else { !water }
                    && !self.cities.values().any(|other| other.id != city && other.production.as_ref().is_some_and(|x| matches!(x, Production::District(_, q) | Production::Wonder(_, q) if *q == p)))
            })
            .collect()
    }

    pub fn wonder_sites(&self, city: CityId, _wonder: usize) -> Vec<Pos> {
        let rules = &self.rules;
        let c = &self.cities[&city];
        self.city_tiles(city)
            .into_iter()
            .filter(|&p| {
                p != c.pos && self.map.is_land(rules, p) && !self.map.is_impassable(rules, p)
            })
            .filter(|&p| self.map.tile(p).district.is_none() && self.map.tile(p).wonder.is_none())
            .collect()
    }

    pub fn set_production(&mut self, city: CityId, item: Production) -> Result<(), String> {
        if !self.can_produce(city, &item) {
            return Err("This city can't build that here.".into());
        }
        self.cities.get_mut(&city).unwrap().production = Some(item);
        Ok(())
    }

    /// Adds a turn of production; finishes the item when complete.
    pub(crate) fn progress_production(&mut self, city: CityId) {
        let Some(item) = self.cities[&city].production.clone() else {
            return;
        };
        if !self.can_produce(city, &item) {
            self.cities.get_mut(&city).unwrap().production = None;
            return;
        }
        let amount = self.production_toward(city, &item).round() as i32;
        let cost = self.production_cost(self.cities[&city].owner, &item);
        let key = Self::production_key(&item);
        let c = self.cities.get_mut(&city).unwrap();
        let progress = c.progress.entry(key.clone()).or_insert(0);
        *progress += amount.max(1);
        if *progress >= cost {
            let overflow = *progress - cost;
            c.progress.remove(&key);
            self.complete_production(city, item);
            let c = self.cities.get_mut(&city).unwrap();
            c.border_culture += 0;
            if let Some(next) = c.production.clone() {
                *c.progress.entry(Self::production_key(&next)).or_insert(0) += overflow;
            }
        }
    }

    pub(crate) fn complete_production(&mut self, city: CityId, item: Production) {
        let rules = self.rules.clone();
        let content = &rules.content;
        let (owner, pos, name) = {
            let c = &self.cities[&city];
            (c.owner, c.pos, c.name.clone())
        };
        match &item {
            Production::Unit(i) => {
                let unit = &content.units[*i];
                let military = unit.combat > 0 || unit.ranged_combat > 0 || unit.bombard > 0;
                let occupied =
                    |g: &Game, p: Pos| g.ground_units_at(p).any(|u| g.is_military(u) == military);
                let spot = if unit.domain == "DOMAIN_AIR" {
                    self.free_air_base(city)
                } else if !occupied(self, pos) {
                    Some(pos)
                } else {
                    self.map.grid.within(pos, 2).into_iter().find(|&p| {
                        !occupied(self, p)
                            && self.map.tile(p).owner == Some(owner)
                            && !self.map.is_impassable(&rules, p)
                            && (self.map.is_water(&rules, p) == (unit.domain == "DOMAIN_SEA"))
                    })
                };
                if let Some(spot) = spot {
                    let id = self.spawn_kind(owner, *i, spot);
                    let xp = self.bonuses(owner).unit_xp;
                    if let Some(u) = self.units.get_mut(&id) {
                        u.xp += xp;
                    }
                    if unit.found_city {
                        let c = self.cities.get_mut(&city).unwrap();
                        c.population = (c.population - 1).max(1);
                    }
                    self.on_trained(owner, *i);
                }
                if unit.id == crate::additions::STRIKE_DRONE {
                    // Strike drones come off the line three at a time.
                    let mut made = usize::from(spot.is_some());
                    for p in self.map.grid.within(pos, 3) {
                        if made >= crate::additions::DRONES_PER_BATCH {
                            break;
                        }
                        if !occupied(self, p)
                            && self.map.is_land(&rules, p)
                            && !self.map.is_impassable(&rules, p)
                            && self.map.tile(p).owner == Some(owner)
                        {
                            self.spawn_kind(owner, *i, p);
                            made += 1;
                        }
                    }
                    self.notify(
                        owner,
                        format!("{name} built {made} {}s.", unit.name),
                        Some(pos),
                    );
                } else {
                    self.notify(owner, format!("{name} trained a {}.", unit.name), Some(pos));
                }
            }
            Production::Building(i) => {
                let c = self.cities.get_mut(&city).unwrap();
                c.buildings.push(*i);
                if content.buildings[*i].outer_defense > 0 {
                    c.wall_hp += content.buildings[*i].outer_defense;
                }
                self.on_built(owner, *i);
                self.notify(
                    owner,
                    format!("{name} built the {}.", content.buildings[*i].name),
                    Some(pos),
                );
            }
            Production::Wonder(i, at) => {
                self.cities.get_mut(&city).unwrap().buildings.push(*i);
                self.map.tile_mut(*at).wonder = Some(*i);
                self.world_wonders.push(*i);
                self.players[owner].great_works += 1;
                let wonder = content.buildings[*i].name.clone();
                let builder = self.players[owner].civ_name.clone();
                for p in 0..self.players.len() {
                    let msg = if p == owner {
                        format!("{name} completed {wonder}!")
                    } else {
                        format!("{builder} completed {wonder}.")
                    };
                    if self.players[p].is_major() {
                        self.notify(p, msg, Some(*at));
                    }
                }
                self.on_built(owner, *i);
            }
            Production::District(i, at) => {
                self.cities
                    .get_mut(&city)
                    .unwrap()
                    .districts
                    .push(CityDistrict {
                        district: *i,
                        pos: *at,
                    });
                let tile = self.map.tile_mut(*at);
                tile.district = Some(*i);
                tile.improvement = None;
                if tile
                    .feature
                    .is_some_and(|f| !content.features[f].natural_wonder)
                {
                    tile.feature = None;
                }
                self.on_district(owner, *i);
                self.notify(
                    owner,
                    format!("{name} completed a {}.", content.districts[*i].name),
                    Some(*at),
                );
                self.assign_tiles(city);
            }
            Production::Project(i) => {
                self.players[owner].space_projects.push(*i);
                self.notify(
                    owner,
                    format!("{name} completed {}.", content.projects[*i].name),
                    Some(pos),
                );
            }
        }
        // As in Civ VI, the city then asks what to build next.
        self.cities.get_mut(&city).unwrap().production = None;
    }

    /// Finishes `item` immediately (for tests and scenario setup).
    pub fn complete_production_for_test(&mut self, city: CityId, item: Production) {
        self.complete_production(city, item);
    }

    pub(crate) fn spawn_kind(&mut self, owner: PlayerId, kind: usize, pos: Pos) -> u32 {
        let id = self.rules.content.units[kind].id.clone();
        // `spawn` resolves uniques; production already chose the right unit.
        let unit = self.spawn_exact(owner, kind, pos);
        let _ = id;
        unit
    }

    pub(crate) fn spawn_exact(&mut self, owner: PlayerId, kind: usize, pos: Pos) -> u32 {
        let ty = &self.rules.content.units[kind];
        let id = self.next_unit;
        self.next_unit += 1;
        self.units.insert(
            id,
            crate::state::Unit {
                id,
                owner,
                kind,
                pos,
                moves: ty.moves,
                hp: 100,
                xp: 0,
                level: 0,
                fortified: false,
                fortify_turns: 0,
                sleeping: false,
                charges: ty.build_charges.max(ty.spread_charges),
                religion: None,
                great_person: None,
                path: Vec::new(),
                attacked: false,
                skipped: false,
            },
        );
        id
    }

    /// Grows or starves the city, expands its borders, and builds.
    pub(crate) fn process_city(&mut self, id: CityId) {
        let report = self.city_report(id);
        let surplus = report.yields[FOOD] - report.food_consumed;
        let border_rate = report.yields[CULTURE];
        let speed = self.speed();
        {
            let c = self.cities.get_mut(&id).unwrap();
            c.food += surplus.round() as i32;
            if c.food >= report.growth_threshold {
                c.food -= report.growth_threshold;
                c.population += 1;
            } else if c.food < 0 {
                if c.population > 1 {
                    c.population -= 1;
                    c.food = (Self::growth_threshold(c.population) as f32 * speed / 2.0) as i32;
                } else {
                    c.food = 0;
                }
            }
            c.border_culture += border_rate.round() as i32 + 1;
            c.has_attacked = false;
            c.turns_since_combat += 1;
            if c.turns_since_combat > 1 {
                c.hp = (c.hp + 20).min(200);
                let max_walls: i32 = c
                    .buildings
                    .iter()
                    .map(|&b| self.rules.content.buildings[b].outer_defense)
                    .sum();
                c.wall_hp = (c.wall_hp + 10).min(max_walls);
            }
        }
        self.expand_borders(id);
        self.assign_tiles(id);
        self.progress_production(id);
    }

    fn expand_borders(&mut self, id: CityId) {
        let owned = self.city_tiles(id).len() as f32;
        let cost = self.scaled((10.0 + (6.0 * (owned - 6.0).max(0.0)).powf(1.3)) as i32);
        if self.cities[&id].border_culture < cost {
            return;
        }
        let c = &self.cities[&id];
        let (center, owner) = (c.pos, c.owner);
        let best = self
            .map
            .grid
            .within(center, CITY_RANGE)
            .into_iter()
            .filter(|&p| {
                self.map.tile(p).owner.is_none()
                    && self
                        .map
                        .grid
                        .neighbors(p)
                        .any(|n| self.map.tile(n).owning_city == Some(id))
            })
            .max_by_key(|&p| {
                let y = self.tile_yields(owner, p);
                let resource = i32::from(self.map.tile(p).resource.is_some()) * 3;
                (y[FOOD] * 3
                    + y[PRODUCTION] * 3
                    + y[GOLD]
                    + y[SCIENCE]
                    + y[CULTURE]
                    + y[FAITH]
                    + resource)
                    * 10
                    - self.map.grid.distance(center, p) * 4
            });
        if let Some(pos) = best {
            self.cities.get_mut(&id).unwrap().border_culture -= cost;
            self.claim(id, pos);
        }
    }

    /// Hooks for boosts and one-off effects (see `research.rs`).
    pub(crate) fn on_trained(&mut self, owner: PlayerId, unit: usize) {
        self.check_boosts(owner, crate::research::Trigger::Trained(unit));
    }

    pub(crate) fn on_built(&mut self, owner: PlayerId, building: usize) {
        self.check_boosts(owner, crate::research::Trigger::Built(building));
    }

    pub(crate) fn on_district(&mut self, owner: PlayerId, district: usize) {
        self.check_boosts(owner, crate::research::Trigger::District(district));
    }
}

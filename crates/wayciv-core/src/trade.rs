//! Trade routes and roads.
//!
//! A Trader in one of your cities starts a route to a city within range.
//! The route earns yields for the origin city for `ROUTE_TURNS` turns, then
//! the Trader returns home for a new assignment. Domestic routes bring Food
//! and Production; international routes bring Gold, more for destinations
//! with more districts. Starting a route builds a road between the cities.

use crate::hex::Pos;
use crate::map::{CityId, PlayerId};
use crate::rules::{FOOD, GOLD, PRODUCTION};
use crate::state::{Game, TradeRoute, UnitId};
use std::collections::{HashMap, VecDeque};

pub const ROUTE_RANGE: i32 = 15;
pub const ROUTE_TURNS: u32 = 30;

impl Game {
    /// Trade route capacity: the Palace, Commercial Hubs, Harbors, and
    /// other districts and buildings with capacity, plus government,
    /// policy, ability, and wonder effects.
    pub fn trade_capacity(&self, player: PlayerId) -> i32 {
        let content = &self.rules.content;
        let mut total = self.bonuses(player).trade_capacity;
        for c in self.player_cities(player) {
            for d in &c.districts {
                total += content.districts[d.district].trade_capacity;
            }
            for &b in &c.buildings {
                total += content.buildings[b].trade_capacity;
            }
        }
        total
    }

    pub fn trade_routes_of(&self, player: PlayerId) -> impl Iterator<Item = &TradeRoute> {
        self.trade_routes.iter().filter(move |r| r.owner == player)
    }

    /// Yields a route earns for its origin city each turn.
    pub fn route_yields(&self, owner: PlayerId, destination: CityId) -> [f32; 6] {
        let content = &self.rules.content;
        let dest = &self.cities[&destination];
        let districts = dest
            .districts
            .iter()
            .filter(|d| content.districts[d.district].requires_population)
            .count()
            .min(4) as f32;
        let bonuses = self.bonuses(owner);
        let mut y = [0f32; 6];
        if dest.owner == owner {
            y[FOOD] += 2.0;
            y[PRODUCTION] += 1.0 + (districts / 2.0).floor();
            for i in 0..6 {
                y[i] += bonuses.trade_domestic[i];
            }
        } else {
            y[GOLD] += 3.0
                + districts
                + if self.players[dest.owner].is_city_state() {
                    1.0
                } else {
                    0.0
                };
            for i in 0..6 {
                y[i] += bonuses.trade_international[i];
            }
        }
        y
    }

    /// Cities a Trader standing in one of its owner's cities can reach.
    pub fn trade_destinations(&self, trader: UnitId) -> Vec<CityId> {
        let Some(u) = self.units.get(&trader) else {
            return Vec::new();
        };
        if !self.unit_type(u).make_trade_route {
            return Vec::new();
        }
        let Some(origin) = self.city_at(u.pos).filter(|c| c.owner == u.owner) else {
            return Vec::new();
        };
        let me = &self.players[u.owner];
        self.cities
            .values()
            .filter(|c| c.id != origin.id)
            .filter(|c| {
                c.owner == u.owner
                    || (me.relations[c.owner].met
                        && !me.at_war_with(c.owner)
                        && !self.players[c.owner].is_barbarian())
            })
            .filter(|c| self.map.grid.distance(c.pos, origin.pos) <= ROUTE_RANGE)
            .filter(|c| {
                !self
                    .trade_routes
                    .iter()
                    .any(|r| r.owner == u.owner && r.origin == origin.id && r.destination == c.id)
            })
            .map(|c| c.id)
            .collect()
    }

    pub fn establish_trade_route(
        &mut self,
        trader: UnitId,
        destination: CityId,
    ) -> Result<(), String> {
        if !self.trade_destinations(trader).contains(&destination) {
            return Err("The Trader can't reach that city from here.".into());
        }
        let u = self.units[&trader].clone();
        let used = self.trade_routes_of(u.owner).count() as i32;
        if used >= self.trade_capacity(u.owner) {
            return Err(
                "No trade route capacity left. Commercial Hubs, Harbors and some policies add more."
                    .into(),
            );
        }
        let origin = self.city_at(u.pos).unwrap().id;
        self.units.remove(&trader);
        self.first_moment(u.owner, "trade_route", "Established a first trade route", crate::eras::score::TRADE_ROUTE);
        self.dedication_moment(u.owner, "trade_route", "Started a trade route");
        self.trade_routes.push(TradeRoute {
            owner: u.owner,
            origin,
            destination,
            turns_left: ROUTE_TURNS,
            trader: u.kind,
        });
        let (from, to) = (self.cities[&origin].pos, self.cities[&destination].pos);
        self.build_road(from, to);
        let name = self.cities[&destination].name.clone();
        self.notify(
            u.owner,
            format!("Trade route to {name} established."),
            Some(to),
        );
        self.check_boosts(u.owner, crate::research::Trigger::Poll);
        Ok(())
    }

    /// Lays a road along the shortest land path between two tiles.
    fn build_road(&mut self, from: Pos, to: Pos) {
        let grid = self.map.grid;
        let rules = self.rules.clone();
        let mut came: HashMap<Pos, Pos> = HashMap::new();
        let mut queue = VecDeque::from([from]);
        came.insert(from, from);
        while let Some(pos) = queue.pop_front() {
            if pos == to {
                break;
            }
            for n in grid.neighbors(pos) {
                if !came.contains_key(&n)
                    && self.map.is_land(&rules, n)
                    && !self.map.is_impassable(&rules, n)
                    && grid.distance(n, from) <= ROUTE_RANGE + 2
                {
                    came.insert(n, pos);
                    queue.push_back(n);
                }
            }
        }
        if !came.contains_key(&to) {
            return;
        }
        let mut at = to;
        while at != from {
            self.map.tile_mut(at).road = true;
            at = came[&at];
        }
        self.map.tile_mut(from).road = true;
    }

    /// Ends routes that touch `city` or run between players at war, sending
    /// their Traders home when the origin is still theirs.
    pub(crate) fn cancel_routes(&mut self, keep: impl Fn(&Game, &TradeRoute) -> bool) {
        let routes = std::mem::take(&mut self.trade_routes);
        for route in routes {
            if keep(self, &route) {
                self.trade_routes.push(route);
            } else {
                self.return_trader(&route, "was cancelled");
            }
        }
    }

    fn return_trader(&mut self, route: &TradeRoute, why: &str) {
        let name = self
            .cities
            .get(&route.destination)
            .map(|c| c.name.clone())
            .unwrap_or_default();
        if let Some(origin) = self
            .cities
            .get(&route.origin)
            .filter(|c| c.owner == route.owner)
            .map(|c| c.pos)
        {
            self.spawn_exact(route.owner, route.trader, origin);
            self.notify(
                route.owner,
                format!("The trade route to {name} {why}. Your Trader is ready for a new route."),
                Some(origin),
            );
        }
    }

    /// Counts down routes; finished ones return their Trader.
    pub(crate) fn process_trade(&mut self, player: PlayerId) {
        let mut finished = Vec::new();
        for (i, route) in self.trade_routes.iter_mut().enumerate() {
            if route.owner == player {
                route.turns_left = route.turns_left.saturating_sub(1);
                if route.turns_left == 0 {
                    finished.push(i);
                }
            }
        }
        for i in finished.into_iter().rev() {
            let route = self.trade_routes.remove(i);
            self.return_trader(&route, "has completed");
        }
    }

    /// Route yields earned by a city this turn.
    pub fn city_trade_yields(&self, city: CityId) -> [f32; 6] {
        let mut y = [0f32; 6];
        for route in self.trade_routes.iter().filter(|r| r.origin == city) {
            if self.cities.contains_key(&route.destination) {
                let r = self.route_yields(route.owner, route.destination);
                for i in 0..6 {
                    y[i] += r[i];
                }
            }
        }
        y
    }
}

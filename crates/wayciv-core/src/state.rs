//! Game state: players, cities, units, and the world.

use crate::hex::Pos;
use crate::map::{CityId, Map, PlayerId};
use crate::rng::Rng;
use crate::rules::Rules;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

pub type UnitId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerKind {
    Major,
    CityState,
    /// Barbarians: always at war with everyone, no cities or diplomacy.
    Barbarian,
}

#[derive(Clone, Debug, Default)]
pub struct Relation {
    pub met: bool,
    pub at_war: bool,
    /// Turn the current war or peace began.
    pub since: u32,
    pub friendship: bool,
    pub denounced: bool,
    /// Opinion of the other player, -100..100 (AI only).
    pub opinion: i32,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: PlayerId,
    pub kind: PlayerKind,
    /// Index into `content.leaders` (majors) or `content.city_states`.
    pub leader: usize,
    pub civ: usize,
    pub name: String,
    pub civ_name: String,
    pub human: bool,
    pub alive: bool,
    pub primary: [u8; 3],
    pub secondary: [u8; 3],
    pub gold: i32,
    pub faith: i32,
    pub techs: Vec<bool>,
    pub tech_progress: Vec<i32>,
    pub tech_boosted: Vec<bool>,
    pub research: Option<usize>,
    pub civics: Vec<bool>,
    pub civic_progress: Vec<i32>,
    pub civic_boosted: Vec<bool>,
    pub civic: Option<usize>,
    /// The AI tree (see `additions`): known items, progress, current pick.
    pub ai_techs: Vec<bool>,
    pub ai_progress: Vec<i32>,
    pub ai_research: Option<usize>,
    pub government: Option<usize>,
    pub policies: Vec<usize>,
    /// Policies may be changed for free (after completing a civic).
    pub policy_change_free: bool,
    pub pantheon: Option<usize>,
    pub religion: Option<usize>,
    pub beliefs: Vec<usize>,
    pub great_person_points: HashMap<usize, i32>,
    pub influence: i32,
    pub envoys_available: i32,
    /// Envoys this player has sent to each city-state.
    pub envoys: HashMap<PlayerId, i32>,
    pub relations: Vec<Relation>,
    pub visible: Vec<bool>,
    pub revealed: Vec<bool>,
    pub capital: Option<CityId>,
    pub original_capital: Option<Pos>,
    pub cities_founded: u32,
    pub culture_total: i64,
    pub tourism_total: i64,
    pub great_works: i32,
    pub space_projects: Vec<usize>,
    pub units_killed: u32,
    pub barbarians_killed: u32,
    /// For city-states: `SCIENTIFIC`, `CULTURAL`, ...
    pub city_state_category: String,
}

impl Player {
    pub fn is_city_state(&self) -> bool {
        self.kind == PlayerKind::CityState
    }

    pub fn is_major(&self) -> bool {
        self.kind == PlayerKind::Major
    }

    pub fn is_barbarian(&self) -> bool {
        self.kind == PlayerKind::Barbarian
    }

    pub fn at_war_with(&self, other: PlayerId) -> bool {
        self.relations.get(other).is_some_and(|r| r.at_war)
    }

    pub fn has_tech(&self, rules: &Rules, id: &str) -> bool {
        rules.tech(id).is_some_and(|i| self.techs[i])
    }

    pub fn has_civic(&self, rules: &Rules, id: &str) -> bool {
        rules.civic(id).is_some_and(|i| self.civics[i])
    }

    /// Whether a tech or civic prerequisite (either may be absent) is met.
    pub fn meets(&self, rules: &Rules, tech: Option<&str>, civic: Option<&str>) -> bool {
        tech.is_none_or(|t| self.has_tech(rules, t)) && civic.is_none_or(|c| self.has_civic(rules, c))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Production {
    Unit(usize),
    Building(usize),
    /// A district (index into `content.districts`) on a chosen tile.
    District(usize, Pos),
    /// A wonder (index into `content.buildings`) on a chosen tile.
    Wonder(usize, Pos),
    Project(usize),
}

#[derive(Clone, Debug)]
pub struct CityDistrict {
    pub district: usize,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct City {
    pub id: CityId,
    pub owner: PlayerId,
    pub founder: PlayerId,
    pub name: String,
    pub pos: Pos,
    pub population: i32,
    pub food: i32,
    pub production: Option<Production>,
    /// Progress kept per item, so switching doesn't lose work (as in Civ VI).
    pub progress: HashMap<String, i32>,
    pub buildings: Vec<usize>,
    pub districts: Vec<CityDistrict>,
    pub worked: Vec<Pos>,
    pub hp: i32,
    pub wall_hp: i32,
    pub capital: bool,
    pub original_capital_of: Option<PlayerId>,
    pub founded: u32,
    pub border_culture: i32,
    /// Religion -> number of followers.
    pub followers: HashMap<usize, i32>,
    pub pressure: HashMap<usize, i32>,
    pub has_attacked: bool,
    pub turns_since_combat: u32,
    /// Loyalty 0..=100; at 0 the city flips to the civ pressuring it most.
    pub loyalty: i32,
}

impl City {
    pub fn has_building(&self, building: usize) -> bool {
        self.buildings.contains(&building)
    }

    pub fn has_district(&self, district: usize) -> bool {
        self.districts.iter().any(|d| d.district == district)
    }

    pub fn majority_religion(&self) -> Option<usize> {
        let (religion, count) = self.followers.iter().max_by_key(|(r, c)| (**c, std::cmp::Reverse(**r)))?;
        (*count * 2 > self.population).then_some(*religion)
    }
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub id: UnitId,
    pub owner: PlayerId,
    pub kind: usize,
    pub pos: Pos,
    pub moves: i32,
    pub hp: i32,
    pub xp: i32,
    pub level: i32,
    pub fortified: bool,
    pub fortify_turns: i32,
    pub sleeping: bool,
    pub charges: i32,
    pub religion: Option<usize>,
    pub great_person: Option<usize>,
    /// Remaining multi-turn path.
    pub path: Vec<Pos>,
    pub attacked: bool,
    /// Skipped for this turn (cleared each turn).
    pub skipped: bool,
}

/// An active trade route: the trader travels between two cities, earning
/// yields for the origin city until it completes.
#[derive(Clone, Debug)]
pub struct TradeRoute {
    pub owner: PlayerId,
    pub origin: CityId,
    pub destination: CityId,
    pub turns_left: u32,
    /// The trader's unit type, restored to the origin when the route ends.
    pub trader: usize,
}

#[derive(Clone, Debug)]
pub enum Victory {
    Science,
    Culture,
    Domination,
    Religious,
    Score,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub turn: u32,
    pub player: PlayerId,
    pub text: String,
    pub pos: Option<Pos>,
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub seed: u64,
    pub map_type: crate::map::MapType,
    pub size: crate::map::MapSize,
    pub human_leader: String,
    pub opponents: usize,
    pub city_states: usize,
    pub turn_limit: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            seed: 1,
            map_type: crate::map::MapType::Earth,
            size: crate::map::MapSize::Standard,
            human_leader: crate::rules::REAGAN.to_string(),
            opponents: 5,
            city_states: 6,
            turn_limit: 300,
        }
    }
}

pub struct Game {
    pub rules: Arc<Rules>,
    pub settings: Settings,
    pub map: Map,
    pub players: Vec<Player>,
    pub cities: BTreeMap<CityId, City>,
    pub units: BTreeMap<UnitId, Unit>,
    pub turn: u32,
    pub rng: Rng,
    pub next_city: CityId,
    pub next_unit: UnitId,
    pub notifications: Vec<Notification>,
    pub winner: Option<(PlayerId, Victory)>,
    /// Wonders built anywhere in the world.
    pub world_wonders: Vec<usize>,
    /// Religions founded: religion -> (founder, holy city).
    pub religions: BTreeMap<usize, (PlayerId, CityId)>,
    /// Great people already recruited.
    pub great_people_taken: Vec<usize>,
    pub combat_log: Vec<String>,
    pub trade_routes: Vec<TradeRoute>,
    /// Barbarian camps and turns until each spawns its next unit.
    pub camps: BTreeMap<Pos, u32>,
    /// Leader screens to show the human: (leader's player, what happened).
    pub popups: Vec<(PlayerId, crate::diplomacy::DiploEvent)>,
}

impl Game {
    pub fn human(&self) -> PlayerId {
        self.players.iter().position(|p| p.human).unwrap_or(0)
    }

    pub fn notify(&mut self, player: PlayerId, text: impl Into<String>, pos: Option<Pos>) {
        self.notifications.push(Notification { turn: self.turn, player, text: text.into(), pos });
    }

    pub fn city_at(&self, pos: Pos) -> Option<&City> {
        self.cities.values().find(|c| c.pos == pos)
    }

    pub fn city_id_at(&self, pos: Pos) -> Option<CityId> {
        self.city_at(pos).map(|c| c.id)
    }

    /// Every unit on a tile, including aircraft based there (for display).
    pub fn units_at(&self, pos: Pos) -> impl Iterator<Item = &Unit> {
        self.units.values().filter(move |u| u.pos == pos)
    }

    /// Land and sea units on a tile; aircraft based there don't block
    /// movement, exert zone of control, or defend the tile.
    pub fn ground_units_at(&self, pos: Pos) -> impl Iterator<Item = &Unit> {
        self.units.values().filter(move |u| u.pos == pos && !self.is_air(u))
    }

    pub fn is_air(&self, unit: &Unit) -> bool {
        self.unit_type(unit).domain == "DOMAIN_AIR"
    }

    /// Wars with majors and city-states (barbarians are always hostile and
    /// don't count).
    pub fn real_wars(&self, player: PlayerId) -> usize {
        self.players[player].relations.iter().enumerate().filter(|(o, r)| r.at_war && !self.players[*o].is_barbarian()).count()
    }

    pub fn barbarian(&self) -> Option<PlayerId> {
        self.players.iter().position(|p| p.is_barbarian())
    }

    pub fn player_cities(&self, player: PlayerId) -> impl Iterator<Item = &City> {
        self.cities.values().filter(move |c| c.owner == player)
    }

    pub fn player_units(&self, player: PlayerId) -> impl Iterator<Item = &Unit> {
        self.units.values().filter(move |u| u.owner == player)
    }

    pub fn unit_type(&self, unit: &Unit) -> &crate::rules::Unit {
        &self.rules.content.units[unit.kind]
    }

    /// Military units have combat strength and occupy the military layer of
    /// a tile; civilians (settlers, builders, great people, ...) don't.
    pub fn is_military(&self, unit: &Unit) -> bool {
        let ty = self.unit_type(unit);
        ty.combat > 0 || ty.ranged_combat > 0 || ty.bombard > 0
    }

    pub fn era_of(&self, player: PlayerId) -> usize {
        let p = &self.players[player];
        let content = &self.rules.content;
        let mut era = 0;
        for (i, tech) in content.techs.iter().enumerate() {
            if p.techs[i] {
                era = era.max(self.rules.era_index(&tech.era));
            }
        }
        for (i, civic) in content.civics.iter().enumerate() {
            if p.civics[i] {
                era = era.max(self.rules.era_index(&civic.era));
            }
        }
        era
    }

    pub fn alive_majors(&self) -> impl Iterator<Item = &Player> {
        self.players.iter().filter(|p| p.alive && p.kind == PlayerKind::Major)
    }
}

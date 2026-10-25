//! The content types, and loading them from the TOML files in `data/`.
//!
//! Ids are plain strings (`UNIT_WARRIOR`, `TECH_POTTERY`, ...), unique
//! across every kind of content; art files are named after them. Some
//! fields are written in short form in the data files and expanded here:
//! eras (`ancient` -> `ERA_ANCIENT`), domains, promotion classes, policy
//! slots, belief and resource classes, great person classes, and yields.

use crate::Result;
use crate::effect::{Effect, describe_all, words};
use serde::{Deserialize, Deserializer};
use std::collections::{HashMap, HashSet};

/// Yield order used everywhere: food, production, gold, science, culture, faith.
pub const YIELDS: [&str; 6] = [
    "YIELD_FOOD",
    "YIELD_PRODUCTION",
    "YIELD_GOLD",
    "YIELD_SCIENCE",
    "YIELD_CULTURE",
    "YIELD_FAITH",
];
pub type Yields = [i32; 6];

pub fn yield_index(yield_type: &str) -> Option<usize> {
    crate::effect::yield_index(yield_type)
}

/// `{ food = 2, production = 1 }` in the data files.
fn yields<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Yields, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Y {
        #[serde(default)]
        food: i32,
        #[serde(default)]
        production: i32,
        #[serde(default)]
        gold: i32,
        #[serde(default)]
        science: i32,
        #[serde(default)]
        culture: i32,
        #[serde(default)]
        faith: i32,
    }
    let y = Y::deserialize(d)?;
    Ok([y.food, y.production, y.gold, y.science, y.culture, y.faith])
}

/// `{ scientist = 1 }` in the data files.
fn great_person_points<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<(String, i32)>, D::Error> {
    let map: std::collections::BTreeMap<String, i32> = Deserialize::deserialize(d)?;
    Ok(map
        .into_iter()
        .map(|(k, v)| (prefixed("GREAT_PERSON_CLASS_", &k), v))
        .collect())
}

/// `{ military = 1, economic = 2 }` in the data files, kept in slot order.
fn slots<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<(String, i32)>, D::Error> {
    let map: HashMap<String, i32> = Deserialize::deserialize(d)?;
    let mut out: Vec<(String, i32)> = ["military", "economic", "diplomatic", "wildcard"]
        .iter()
        .filter_map(|k| map.get(*k).map(|n| (prefixed("SLOT_", k), *n)))
        .collect();
    for (k, n) in &map {
        if !["military", "economic", "diplomatic", "wildcard"].contains(&k.as_str()) {
            out.push((prefixed("SLOT_", k), *n));
        }
    }
    Ok(out)
}

/// `ancient` -> `ERA_ANCIENT`; ids already in long form are kept.
pub fn prefixed(prefix: &str, short: &str) -> String {
    if short.is_empty() || short.starts_with(prefix) {
        short.to_string()
    } else {
        format!("{prefix}{}", short.to_ascii_uppercase())
    }
}

fn prefix_opt(prefix: &str, s: &mut Option<String>) {
    if let Some(v) = s {
        *v = prefixed(prefix, v);
    }
}

fn one() -> i32 {
    1
}
fn two() -> i32 {
    2
}
fn yes() -> bool {
    true
}
fn minus_one() -> i32 {
    -1
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terrain {
    pub id: String,
    pub name: String,
    #[serde(default, deserialize_with = "yields")]
    pub yields: Yields,
    #[serde(default)]
    pub hills: bool,
    #[serde(default)]
    pub mountain: bool,
    #[serde(default)]
    pub water: bool,
    #[serde(default)]
    pub shallow_water: bool,
    #[serde(default = "one", alias = "moves")]
    pub movement_cost: i32,
    #[serde(default, alias = "defense")]
    pub defense_modifier: i32,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feature {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, deserialize_with = "yields")]
    pub yields: Yields,
    #[serde(default, alias = "extra_moves")]
    pub movement_change: i32,
    #[serde(default, alias = "defense")]
    pub defense_modifier: i32,
    #[serde(default)]
    pub natural_wonder: bool,
    #[serde(default)]
    pub impassable: bool,
    #[serde(default)]
    pub removable: bool,
    #[serde(default)]
    pub forest: bool,
    #[serde(default, alias = "terrains")]
    pub valid_terrains: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub id: String,
    pub name: String,
    /// `RESOURCECLASS_BONUS`, `_LUXURY`, `_STRATEGIC`, or `_ARTIFACT`.
    pub class: String,
    #[serde(default, deserialize_with = "yields")]
    pub yields: Yields,
    #[serde(default, alias = "tech")]
    pub prereq_tech: Option<String>,
    #[serde(default)]
    pub frequency: i32,
    #[serde(default)]
    pub sea_frequency: i32,
    #[serde(default, alias = "terrains")]
    pub valid_terrains: Vec<String>,
    #[serde(default, alias = "features")]
    pub valid_features: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Improvement {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, deserialize_with = "yields")]
    pub yields: Yields,
    #[serde(default, alias = "tech")]
    pub prereq_tech: Option<String>,
    #[serde(default, alias = "year")]
    pub unlock_year: Option<i32>,
    #[serde(default = "yes")]
    pub buildable: bool,
    #[serde(default, alias = "unique")]
    pub unique_to: Option<String>,
    #[serde(default, alias = "terrains")]
    pub valid_terrains: Vec<String>,
    #[serde(default, alias = "features")]
    pub valid_features: Vec<String>,
    #[serde(default, alias = "resources")]
    pub valid_resources: Vec<String>,
}

/// A leader or civilization ability.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trait {
    pub id: String,
    pub name: String,
    /// The ability's own text; the effects' descriptions are added after it.
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub effects: Vec<Effect>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Civilization {
    pub id: String,
    pub name: String,
    pub adjective: String,
    #[serde(default)]
    pub traits: Vec<Trait>,
    #[serde(default, alias = "cities")]
    pub city_names: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CityState {
    pub id: String,
    pub name: String,
    /// `SCIENTIFIC`, `CULTURAL`, `MILITARISTIC`, `RELIGIOUS`, `TRADE`, `INDUSTRIAL`.
    pub category: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Agenda {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Leader {
    pub id: String,
    pub name: String,
    pub civilization: String,
    pub capital: String,
    #[serde(default)]
    pub traits: Vec<Trait>,
    pub primary_color: [u8; 3],
    pub secondary_color: [u8; 3],
    #[serde(default, rename = "agenda", deserialize_with = "agenda")]
    pub agenda: Option<(String, String)>,
    /// What the leader says, by event (`first_meet`, `greeting`, ...).
    #[serde(default)]
    pub lines: HashMap<String, String>,
}

fn agenda<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<(String, String)>, D::Error> {
    let a = Agenda::deserialize(d)?;
    Ok(Some((a.name, a.description)))
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub cost: i32,
    #[serde(default = "two")]
    pub moves: i32,
    #[serde(default = "two")]
    pub sight: i32,
    #[serde(default)]
    pub combat: i32,
    #[serde(default, alias = "ranged")]
    pub ranged_combat: i32,
    #[serde(default)]
    pub range: i32,
    #[serde(default)]
    pub bombard: i32,
    /// `DOMAIN_LAND`, `DOMAIN_SEA`, or `DOMAIN_AIR`.
    #[serde(default)]
    pub domain: String,
    /// Derived: `FORMATION_CLASS_CIVILIAN`, `_SUPPORT`, `_LAND_COMBAT`,
    /// `_NAVAL`, or `_AIR`.
    #[serde(skip)]
    pub formation_class: String,
    #[serde(default, alias = "class")]
    pub promotion_class: Option<String>,
    #[serde(default, alias = "tech")]
    pub prereq_tech: Option<String>,
    /// Calendar year from which it can be built (former civic unlocks).
    #[serde(default, alias = "year")]
    pub unlock_year: Option<i32>,
    #[serde(default)]
    pub found_city: bool,
    #[serde(default)]
    pub found_religion: bool,
    #[serde(default, alias = "builds")]
    pub build_charges: i32,
    #[serde(default, alias = "spreads")]
    pub spread_charges: i32,
    #[serde(default)]
    pub religious_strength: i32,
    #[serde(skip)]
    pub can_capture: bool,
    #[serde(default, alias = "unique")]
    pub unique_to: Option<String>,
    #[serde(default)]
    pub replaces: Option<String>,
    #[serde(default)]
    pub maintenance: i32,
    #[serde(default, alias = "obsolete")]
    pub obsolete_tech: Option<String>,
    #[serde(default = "yes")]
    pub can_train: bool,
    /// `YIELD_GOLD` (the default for trainable units), `YIELD_FAITH`, or none.
    #[serde(default, alias = "purchase")]
    pub purchase_yield: Option<String>,
    #[serde(default)]
    pub must_purchase: bool,
    /// Strategic resource needed to train it (e.g. Iron for Swordsmen).
    #[serde(default, alias = "resource")]
    pub strategic_resource: Option<String>,
    #[serde(default)]
    pub barbarian: bool,
    #[serde(default)]
    pub anti_air: i32,
    #[serde(default, alias = "trade")]
    pub make_trade_route: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Building {
    pub id: String,
    pub name: String,
    /// The building's own text plus its effects (filled in on load).
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub cost: i32,
    #[serde(default, alias = "tech")]
    pub prereq_tech: Option<String>,
    #[serde(default, alias = "year")]
    pub unlock_year: Option<i32>,
    #[serde(default, alias = "district")]
    pub prereq_district: Option<String>,
    #[serde(default, alias = "requires")]
    pub prereq_buildings: Vec<String>,
    #[serde(default, deserialize_with = "yields")]
    pub yields: Yields,
    #[serde(default)]
    pub housing: i32,
    #[serde(default)]
    pub amenities: i32,
    #[serde(default)]
    pub maintenance: i32,
    #[serde(default)]
    pub wonder: bool,
    #[serde(default, alias = "defense")]
    pub outer_defense: i32,
    #[serde(
        default,
        alias = "great_people",
        deserialize_with = "great_person_points"
    )]
    pub great_person_points: Vec<(String, i32)>,
    #[serde(default, alias = "unique")]
    pub unique_to: Option<String>,
    #[serde(default)]
    pub replaces: Option<String>,
    #[serde(default = "minus_one")]
    pub max_world_instances: i32,
    /// Worship buildings, unlocked by a religion's belief.
    #[serde(default, alias = "worship")]
    pub enabled_by_religion: bool,
    #[serde(default)]
    pub trade_capacity: i32,
    /// Empire-wide effects while the owner has it (mostly wonders).
    #[serde(default)]
    pub effects: Vec<Effect>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adjacency {
    #[serde(skip)]
    pub yield_index: usize,
    #[serde(rename = "yield")]
    pub yield_key: String,
    #[serde(default = "one")]
    pub amount: i32,
    /// Adjacent tiles needed per `amount` (e.g. 2 for "+1 per 2 districts").
    #[serde(default = "one", alias = "per")]
    pub tiles_required: i32,
    #[serde(default, alias = "districts")]
    pub other_district: bool,
    #[serde(default, alias = "terrain")]
    pub adjacent_terrain: Option<String>,
    #[serde(default, alias = "feature")]
    pub adjacent_feature: Option<String>,
    #[serde(default, alias = "district")]
    pub adjacent_district: Option<String>,
    #[serde(default, alias = "mountain")]
    pub adjacent_mountain: bool,
    #[serde(default, alias = "river")]
    pub adjacent_river: bool,
    #[serde(default, alias = "wonder")]
    pub adjacent_wonder: bool,
    #[serde(default, alias = "resource")]
    pub adjacent_resource: bool,
    #[serde(default, alias = "natural_wonder")]
    pub adjacent_natural_wonder: bool,
    #[serde(skip)]
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct District {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub cost: i32,
    #[serde(default, alias = "tech")]
    pub prereq_tech: Option<String>,
    #[serde(default, alias = "year")]
    pub unlock_year: Option<i32>,
    /// Specialty districts: one per 3 population.
    #[serde(default = "yes", alias = "specialty")]
    pub requires_population: bool,
    #[serde(default)]
    pub city_center: bool,
    #[serde(default)]
    pub coast: bool,
    #[serde(default = "yes")]
    pub one_per_city: bool,
    #[serde(default)]
    pub housing: i32,
    #[serde(default)]
    pub amenities: i32,
    #[serde(
        default,
        alias = "great_people",
        deserialize_with = "great_person_points"
    )]
    pub great_person_points: Vec<(String, i32)>,
    #[serde(default)]
    pub adjacencies: Vec<Adjacency>,
    #[serde(default, alias = "unique")]
    pub unique_to: Option<String>,
    #[serde(default)]
    pub replaces: Option<String>,
    #[serde(default)]
    pub internal_only: bool,
    #[serde(default)]
    pub trade_capacity: i32,
}

/// A Eureka: what triggers it. The trigger names match `BOOST_TRIGGER_*`
/// once expanded.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoostDef {
    pub trigger: String,
    pub text: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub building: Option<String>,
    #[serde(default)]
    pub improvement: Option<String>,
    #[serde(default)]
    pub district: Option<String>,
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub tech: Option<String>,
    #[serde(default = "one")]
    pub count: i32,
    #[serde(default)]
    pub percent: Option<i32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Research {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub cost: i32,
    #[serde(default)]
    pub era: String,
    #[serde(default)]
    pub prereqs: Vec<String>,
    /// Row in the tree view.
    #[serde(default, alias = "row")]
    pub tree_row: i32,
    /// Eureka trigger text, when the item has a boost.
    #[serde(skip)]
    pub boost: Option<String>,
    #[serde(default, rename = "boost")]
    pub boost_def: Option<BoostDef>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Government {
    pub id: String,
    pub name: String,
    /// Calendar year from which it can be adopted.
    #[serde(default = "start_year", alias = "year")]
    pub unlock_year: i32,
    /// The government's text plus its effects (filled in on load).
    #[serde(default, alias = "description")]
    pub bonus: String,
    #[serde(default, deserialize_with = "slots")]
    pub slots: Vec<(String, i32)>,
    #[serde(default)]
    pub effects: Vec<Effect>,
}

fn start_year() -> i32 {
    -4000
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "start_year", alias = "year")]
    pub unlock_year: i32,
    /// `SLOT_MILITARY`, `SLOT_ECONOMIC`, `SLOT_DIPLOMATIC`, `SLOT_GREAT_PERSON`.
    pub slot: String,
    #[serde(default)]
    pub obsoleted_by: Option<String>,
    #[serde(default)]
    pub effects: Vec<Effect>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Belief {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// `BELIEF_CLASS_PANTHEON`, `_FOUNDER`, `_FOLLOWER`, `_WORSHIP`, `_ENHANCER`.
    pub class: String,
    #[serde(default)]
    pub effects: Vec<Effect>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Religion {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub pantheon: bool,
    #[serde(default = "grey")]
    pub color: [u8; 3],
}

fn grey() -> [u8; 3] {
    [220, 220, 220]
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Era {
    pub id: String,
    pub name: String,
    pub index: i32,
    #[serde(alias = "great_person_cost")]
    pub great_person_base_cost: i32,
    /// Roughly when the era begins on the calendar, for display.
    #[serde(default)]
    pub start_year: i32,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GreatPersonClass {
    pub id: String,
    pub name: String,
    pub unit: String,
    #[serde(default)]
    pub district: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GreatPerson {
    pub id: String,
    pub name: String,
    pub class: String,
    pub era: String,
    #[serde(default, alias = "note")]
    pub ability: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub cost: i32,
    #[serde(default, alias = "tech")]
    pub prereq_tech: Option<String>,
    #[serde(default, alias = "district")]
    pub prereq_district: Option<String>,
    #[serde(default)]
    pub space_race: bool,
}

/// A Eureka and what triggers it.
#[derive(Clone, Debug, Default)]
pub struct Boost {
    pub tech: Option<String>,
    /// Percent of the cost granted.
    pub percent: i32,
    /// `BOOST_TRIGGER_*`.
    pub class: String,
    pub unit: Option<String>,
    pub building: Option<String>,
    pub improvement: Option<String>,
    pub district: Option<String>,
    pub resource: Option<String>,
    pub boosting_tech: Option<String>,
    pub count: i32,
    pub description: String,
}

/// One data file: any mix of content kinds.
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    terrain: Vec<Terrain>,
    #[serde(default)]
    feature: Vec<Feature>,
    #[serde(default)]
    natural_wonder: Vec<Feature>,
    #[serde(default)]
    resource: Vec<Resource>,
    #[serde(default)]
    improvement: Vec<Improvement>,
    #[serde(default)]
    civilization: Vec<Civilization>,
    #[serde(default)]
    city_state: Vec<CityState>,
    #[serde(default)]
    leader: Vec<Leader>,
    #[serde(default)]
    unit: Vec<Unit>,
    #[serde(default)]
    building: Vec<Building>,
    #[serde(default)]
    wonder: Vec<Building>,
    #[serde(default)]
    district: Vec<District>,
    #[serde(default)]
    tech: Vec<Research>,
    #[serde(default)]
    government: Vec<Government>,
    #[serde(default)]
    policy: Vec<Policy>,
    #[serde(default)]
    belief: Vec<Belief>,
    #[serde(default)]
    religion: Vec<Religion>,
    #[serde(default)]
    era: Vec<Era>,
    #[serde(default)]
    great_person_class: Vec<GreatPersonClass>,
    #[serde(default)]
    great_person: Vec<GreatPerson>,
    #[serde(default)]
    project: Vec<Project>,
}

/// The data files, in load order.
pub const FILES: [(&str, &str); 15] = [
    ("eras.toml", include_str!("../data/eras.toml")),
    ("terrain.toml", include_str!("../data/terrain.toml")),
    ("resources.toml", include_str!("../data/resources.toml")),
    ("techs.toml", include_str!("../data/techs.toml")),
    ("units.toml", include_str!("../data/units.toml")),
    ("buildings.toml", include_str!("../data/buildings.toml")),
    ("wonders.toml", include_str!("../data/wonders.toml")),
    ("districts.toml", include_str!("../data/districts.toml")),
    ("governments.toml", include_str!("../data/governments.toml")),
    ("religion.toml", include_str!("../data/religion.toml")),
    (
        "great_people.toml",
        include_str!("../data/great_people.toml"),
    ),
    ("projects.toml", include_str!("../data/projects.toml")),
    (
        "civilizations.toml",
        include_str!("../data/civilizations.toml"),
    ),
    ("leaders.toml", include_str!("../data/leaders.toml")),
    ("city_states.toml", include_str!("../data/city_states.toml")),
];

#[derive(Clone, Debug, Default)]
pub struct Content {
    pub terrains: Vec<Terrain>,
    pub features: Vec<Feature>,
    pub resources: Vec<Resource>,
    pub improvements: Vec<Improvement>,
    pub civilizations: Vec<Civilization>,
    pub city_states: Vec<CityState>,
    pub leaders: Vec<Leader>,
    pub units: Vec<Unit>,
    pub buildings: Vec<Building>,
    pub districts: Vec<District>,
    pub techs: Vec<Research>,
    pub governments: Vec<Government>,
    pub policies: Vec<Policy>,
    pub beliefs: Vec<Belief>,
    pub religions: Vec<Religion>,
    pub eras: Vec<Era>,
    pub great_person_classes: Vec<GreatPersonClass>,
    pub great_people: Vec<GreatPerson>,
    pub projects: Vec<Project>,
    pub boosts: Vec<Boost>,
}

impl Content {
    /// Loads WayCiv's content from the data files compiled into the game.
    pub fn load() -> Result<Self> {
        let mut content = Content::default();
        for (name, text) in FILES {
            let file: File = toml::from_str(text).map_err(|e| format!("{name}: {e}"))?;
            content.terrains.extend(file.terrain);
            content.features.extend(file.feature);
            content
                .features
                .extend(file.natural_wonder.into_iter().map(|mut f| {
                    f.natural_wonder = true;
                    f
                }));
            content.resources.extend(file.resource);
            content.improvements.extend(file.improvement);
            content.civilizations.extend(file.civilization);
            content.city_states.extend(file.city_state);
            content.leaders.extend(file.leader);
            content.units.extend(file.unit);
            content.buildings.extend(file.building);
            content
                .buildings
                .extend(file.wonder.into_iter().map(|mut w| {
                    w.wonder = true;
                    if w.max_world_instances < 0 {
                        w.max_world_instances = 1;
                    }
                    w
                }));
            content.districts.extend(file.district);
            content.techs.extend(file.tech);
            content.governments.extend(file.government);
            content.policies.extend(file.policy);
            content.beliefs.extend(file.belief);
            content.religions.extend(file.religion);
            content.eras.extend(file.era);
            content.great_person_classes.extend(file.great_person_class);
            content.great_people.extend(file.great_person);
            content.projects.extend(file.project);
        }
        content.expand();
        content.describe();
        let problems = content.validate();
        if !problems.is_empty() {
            return Err(format!("content errors:\n  {}", problems.join("\n  ")).into());
        }
        Ok(content)
    }

    /// Expands short forms and derives fields.
    fn expand(&mut self) {
        for e in &mut self.eras {
            e.id = prefixed("ERA_", &e.id);
        }
        for r in &mut self.resources {
            r.class = prefixed("RESOURCECLASS_", &r.class);
        }
        for c in &mut self.city_states {
            c.category = c.category.to_ascii_uppercase();
        }
        for t in &mut self.techs {
            t.era = prefixed("ERA_", &t.era);
        }
        for g in &mut self.great_person_classes {
            g.id = prefixed("GREAT_PERSON_CLASS_", &g.id);
        }
        for g in &mut self.great_people {
            g.class = prefixed("GREAT_PERSON_CLASS_", &g.class);
            g.era = prefixed("ERA_", &g.era);
        }
        for b in &mut self.beliefs {
            b.class = prefixed("BELIEF_CLASS_", &b.class);
        }
        for p in &mut self.policies {
            p.slot = prefixed("SLOT_", &p.slot);
        }
        for u in &mut self.units {
            if u.domain.is_empty() {
                u.domain = "land".into();
            }
            u.domain = prefixed("DOMAIN_", &u.domain);
            prefix_opt("PROMOTION_CLASS_", &mut u.promotion_class);
            let fights = u.combat > 0 || u.ranged_combat > 0 || u.bombard > 0;
            let support = u.promotion_class.as_deref() == Some("PROMOTION_CLASS_SUPPORT");
            u.formation_class = match (fights, support, u.domain.as_str()) {
                (_, true, _) => "FORMATION_CLASS_SUPPORT",
                (false, _, _) => "FORMATION_CLASS_CIVILIAN",
                (true, _, "DOMAIN_SEA") => "FORMATION_CLASS_NAVAL",
                (true, _, "DOMAIN_AIR") => "FORMATION_CLASS_AIR",
                _ => "FORMATION_CLASS_LAND_COMBAT",
            }
            .into();
            u.can_capture = fights && u.domain != "DOMAIN_AIR";
            u.purchase_yield = match u.purchase_yield.as_deref() {
                Some("none") => None,
                Some(y) => Some(prefixed("YIELD_", y)),
                None if u.can_train => Some("YIELD_GOLD".into()),
                None => None,
            };
        }
        for d in &mut self.districts {
            for a in &mut d.adjacencies {
                a.yield_index = yield_index(&a.yield_key).unwrap_or(0);
            }
        }
        let effects = self
            .policies
            .iter_mut()
            .flat_map(|p| p.effects.iter_mut())
            .chain(
                self.governments
                    .iter_mut()
                    .flat_map(|g| g.effects.iter_mut()),
            )
            .chain(self.beliefs.iter_mut().flat_map(|b| b.effects.iter_mut()))
            .chain(self.buildings.iter_mut().flat_map(|b| b.effects.iter_mut()))
            .chain(
                self.leaders
                    .iter_mut()
                    .flat_map(|l| l.traits.iter_mut())
                    .chain(
                        self.civilizations
                            .iter_mut()
                            .flat_map(|c| c.traits.iter_mut()),
                    )
                    .flat_map(|t| t.effects.iter_mut()),
            );
        for e in effects {
            match e {
                Effect::UnitProduction {
                    class, domain, era, ..
                } => {
                    prefix_opt("PROMOTION_CLASS_", class);
                    prefix_opt("DOMAIN_", domain);
                    prefix_opt("ERA_", era);
                }
                Effect::Combat { class, .. } => prefix_opt("PROMOTION_CLASS_", class),
                Effect::GreatPersonPoints { class, .. } => {
                    *class = prefixed("GREAT_PERSON_CLASS_", class)
                }
                Effect::PolicyYield { slot, .. } => *slot = prefixed("SLOT_", slot),
                _ => {}
            }
        }
        for t in &mut self.techs {
            if let Some(b) = &t.boost_def {
                t.boost = Some(b.text.clone());
                self.boosts.push(Boost {
                    tech: Some(t.id.clone()),
                    percent: b.percent.unwrap_or(40),
                    class: prefixed("BOOST_TRIGGER_", &b.trigger),
                    unit: b.unit.clone(),
                    building: b.building.clone(),
                    improvement: b.improvement.clone(),
                    district: b.district.clone(),
                    resource: b.resource.clone(),
                    boosting_tech: b.tech.clone(),
                    count: b.count,
                    description: b.text.clone(),
                });
            }
        }
    }

    /// The display name of any content id (or a readable form of a class id).
    pub fn name_of(&self, id: &str) -> String {
        macro_rules! find {
            ($($list:ident),*) => {
                $(if let Some(x) = self.$list.iter().find(|x| x.id == id) {
                    return x.name.clone();
                })*
            };
        }
        find!(
            terrains,
            features,
            resources,
            improvements,
            civilizations,
            city_states,
            leaders,
            units,
            buildings,
            districts,
            techs,
            governments,
            policies,
            beliefs,
            religions,
            eras,
            great_person_classes,
            great_people,
            projects
        );
        words(id)
    }

    /// Fills in descriptions from effects and adjacency rules.
    fn describe(&mut self) {
        let names = self.clone();
        let name = |id: &str| names.name_of(id);
        for p in &mut self.policies {
            p.description = describe_all(&p.description, &p.effects, &name);
        }
        for g in &mut self.governments {
            g.bonus = describe_all(&g.bonus, &g.effects, &name);
        }
        for b in &mut self.beliefs {
            b.description = describe_all(&b.description, &b.effects, &name);
        }
        for b in &mut self.buildings {
            b.description = describe_all(&b.description, &b.effects, &name);
        }
        for t in self
            .leaders
            .iter_mut()
            .flat_map(|l| l.traits.iter_mut())
            .chain(
                self.civilizations
                    .iter_mut()
                    .flat_map(|c| c.traits.iter_mut()),
            )
        {
            t.description = describe_all(&t.description, &t.effects, &name);
        }
        for d in &mut self.districts {
            for a in &mut d.adjacencies {
                let yield_word = crate::effect::YIELD_WORDS[a.yield_index];
                let source = if a.other_district {
                    "district".to_string()
                } else if let Some(x) = &a.adjacent_district {
                    name(x)
                } else if let Some(x) = &a.adjacent_terrain {
                    name(x)
                } else if let Some(x) = &a.adjacent_feature {
                    name(x)
                } else if a.adjacent_mountain {
                    "Mountain".into()
                } else if a.adjacent_wonder {
                    "wonder".into()
                } else if a.adjacent_natural_wonder {
                    "natural wonder".into()
                } else if a.adjacent_resource {
                    "resource".into()
                } else {
                    String::new()
                };
                a.description = if a.adjacent_river {
                    format!("+{} {yield_word} next to a river", a.amount)
                } else if a.tiles_required > 1 {
                    format!(
                        "+{} {yield_word} per {} adjacent {source} tiles",
                        a.amount, a.tiles_required
                    )
                } else {
                    format!("+{} {yield_word} per adjacent {source}", a.amount)
                };
            }
        }
    }

    /// Every id that is referred to but not defined, every duplicate id, and
    /// every unknown yield or trigger.
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let mut ids: HashSet<&str> = HashSet::new();
        macro_rules! collect {
            ($($list:ident),*) => {
                $(for x in &self.$list {
                    if !ids.insert(x.id.as_str()) {
                        problems.push(format!("duplicate id {}", x.id));
                    }
                })*
            };
        }
        collect!(
            terrains,
            features,
            resources,
            improvements,
            civilizations,
            city_states,
            leaders,
            units,
            buildings,
            districts,
            techs,
            governments,
            policies,
            beliefs,
            religions,
            eras,
            great_person_classes,
            great_people,
            projects
        );
        for c in &self.civilizations {
            for t in &c.traits {
                ids.insert(t.id.as_str());
            }
        }
        for l in &self.leaders {
            for t in &l.traits {
                ids.insert(t.id.as_str());
            }
        }
        // Owners of unique items that aren't content of their own.
        ids.insert("CIVILIZATION_BARBARIAN");
        let mut check = |what: &str, id: &str| {
            if !id.is_empty() && !ids.contains(id) {
                problems.push(format!("{what} refers to unknown id {id}"));
            }
        };
        let opt = |o: &Option<String>| o.clone().unwrap_or_default();
        let mut trigger_problems = Vec::new();
        for f in &self.features {
            for t in &f.valid_terrains {
                check(&f.id, t);
            }
        }
        for r in &self.resources {
            check(&r.id, &opt(&r.prereq_tech));
            for t in r.valid_terrains.iter().chain(&r.valid_features) {
                check(&r.id, t);
            }
        }
        for i in &self.improvements {
            check(&i.id, &opt(&i.prereq_tech));
            check(&i.id, &opt(&i.unique_to));
            for t in i
                .valid_terrains
                .iter()
                .chain(&i.valid_features)
                .chain(&i.valid_resources)
            {
                check(&i.id, t);
            }
        }
        for l in &self.leaders {
            check(&l.id, &l.civilization);
        }
        for u in &self.units {
            for r in [
                &u.prereq_tech,
                &u.unique_to,
                &u.replaces,
                &u.obsolete_tech,
                &u.strategic_resource,
            ] {
                check(&u.id, &opt(r));
            }
        }
        for b in &self.buildings {
            for r in [
                &b.prereq_tech,
                &b.prereq_district,
                &b.unique_to,
                &b.replaces,
            ] {
                check(&b.id, &opt(r));
            }
            for r in &b.prereq_buildings {
                check(&b.id, r);
            }
            for (c, _) in &b.great_person_points {
                check(&b.id, c);
            }
        }
        for d in &self.districts {
            for r in [&d.prereq_tech, &d.unique_to, &d.replaces] {
                check(&d.id, &opt(r));
            }
            for (c, _) in &d.great_person_points {
                check(&d.id, c);
            }
            for a in &d.adjacencies {
                if yield_index(&a.yield_key).is_none() {
                    check(&d.id, &a.yield_key);
                }
                for r in [
                    &a.adjacent_terrain,
                    &a.adjacent_feature,
                    &a.adjacent_district,
                ] {
                    check(&d.id, &opt(r));
                }
            }
        }
        for t in &self.techs {
            check(&t.id, &t.era);
            for p in &t.prereqs {
                check(&t.id, p);
            }
            if let Some(b) = &t.boost_def {
                for r in [
                    &b.unit,
                    &b.building,
                    &b.improvement,
                    &b.district,
                    &b.resource,
                    &b.tech,
                ] {
                    check(&t.id, &opt(r));
                }
                if !KNOWN_TRIGGERS.contains(&b.trigger.as_str()) {
                    trigger_problems.push(format!("{}: unknown boost trigger {}", t.id, b.trigger));
                }
            }
        }
        for p in &self.policies {
            check(&p.id, &opt(&p.obsoleted_by));
        }
        for g in &self.great_person_classes {
            check(&g.id, &g.unit);
            check(&g.id, &opt(&g.district));
        }
        for g in &self.great_people {
            check(&g.id, &g.class);
            check(&g.id, &g.era);
        }
        for p in &self.projects {
            check(&p.id, &opt(&p.prereq_tech));
            check(&p.id, &opt(&p.prereq_district));
        }
        let owners = self
            .policies
            .iter()
            .map(|p| (&p.id, &p.effects))
            .chain(self.governments.iter().map(|g| (&g.id, &g.effects)))
            .chain(self.beliefs.iter().map(|b| (&b.id, &b.effects)))
            .chain(self.buildings.iter().map(|b| (&b.id, &b.effects)))
            .chain(
                self.leaders
                    .iter()
                    .flat_map(|l| &l.traits)
                    .chain(self.civilizations.iter().flat_map(|c| &c.traits))
                    .map(|t| (&t.id, &t.effects)),
            );
        let mut effect_problems = Vec::new();
        for (owner, effects) in owners {
            for e in effects {
                for r in e.references() {
                    if !ids.contains(r) {
                        effect_problems.push(format!("{owner} effect refers to unknown id {r}"));
                    }
                }
                if let Some(y) = e.yield_key()
                    && yield_index(y).is_none()
                {
                    effect_problems.push(format!("{owner} effect has unknown yield {y}"));
                }
            }
        }
        problems.extend(trigger_problems);
        problems.extend(effect_problems);
        problems
    }

    /// A leader's line for a diplomacy event, if written.
    pub fn line(&self, leader: &str, event: &str) -> Option<&str> {
        self.leaders
            .iter()
            .find(|l| l.id == leader)
            .and_then(|l| l.lines.get(event))
            .map(String::as_str)
    }
}

/// Boost triggers the game understands (`BOOST_TRIGGER_` + upper case).
pub const KNOWN_TRIGGERS: [&str; 26] = [
    "train_unit",
    "construct_building",
    "kill_with",
    "meet_civ",
    "clear_camp",
    "num_barbs_killed",
    "maintain_x_trade_routes",
    "create_pantheon",
    "found_religion",
    "settle_coast",
    "receive_dow",
    "have_x_buildings",
    "have_x_districts",
    "have_x_unique_specialty_districts",
    "own_x_units_of_type",
    "have_x_land_units",
    "have_x_improvements",
    "improve_specific_resource",
    "num_improved_tiles",
    "city_population",
    "empire_population",
    "have_x_wonders",
    "meet_x_city_states",
    "research_tech",
    "have_x_policy_slots",
    "have_x_great_people",
];

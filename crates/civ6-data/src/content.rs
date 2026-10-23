//! Game content read from the installed Civilization VI: terrains, civs,
//! leaders, units, buildings, districts, techs, civics, governments,
//! policies, beliefs, great people, and projects. Names and descriptions are
//! resolved to English. WayCiv's own rules decide what the content does.

use crate::install::{build_standard, steam_assets};
use crate::text::Text;
use crate::{Result, files_in_load_order};
use rusqlite::{Connection, Row};
use std::collections::HashMap;
use std::path::Path;

/// Yield order used everywhere: food, production, gold, science, culture, faith.
pub const YIELDS: [&str; 6] =
    ["YIELD_FOOD", "YIELD_PRODUCTION", "YIELD_GOLD", "YIELD_SCIENCE", "YIELD_CULTURE", "YIELD_FAITH"];
pub type Yields = [i32; 6];

pub fn yield_index(yield_type: &str) -> Option<usize> {
    YIELDS.iter().position(|y| *y == yield_type)
}

#[derive(Clone, Debug)]
pub struct Terrain {
    pub id: String,
    pub name: String,
    pub yields: Yields,
    pub hills: bool,
    pub mountain: bool,
    pub water: bool,
    pub shallow_water: bool,
    pub movement_cost: i32,
    pub defense_modifier: i32,
}

#[derive(Clone, Debug)]
pub struct Feature {
    pub id: String,
    pub name: String,
    pub yields: Yields,
    pub movement_change: i32,
    pub defense_modifier: i32,
    pub natural_wonder: bool,
    pub impassable: bool,
    pub removable: bool,
    pub forest: bool,
    pub valid_terrains: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Resource {
    pub id: String,
    pub name: String,
    /// `RESOURCECLASS_BONUS`, `_LUXURY`, or `_STRATEGIC`.
    pub class: String,
    pub yields: Yields,
    pub prereq_tech: Option<String>,
    pub frequency: i32,
    pub sea_frequency: i32,
    pub valid_terrains: Vec<String>,
    pub valid_features: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Improvement {
    pub id: String,
    pub name: String,
    pub yields: Yields,
    pub prereq_tech: Option<String>,
    pub prereq_civic: Option<String>,
    pub buildable: bool,
    pub unique_to: Option<String>,
    pub valid_terrains: Vec<String>,
    pub valid_features: Vec<String>,
    pub valid_resources: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Trait {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct Civilization {
    pub id: String,
    pub name: String,
    pub adjective: String,
    pub traits: Vec<Trait>,
    pub city_names: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct CityState {
    pub id: String,
    pub name: String,
    /// `SCIENTIFIC`, `CULTURAL`, `MILITARISTIC`, `RELIGIOUS`, `TRADE`, `INDUSTRIAL`.
    pub category: String,
}

#[derive(Clone, Debug)]
pub struct Leader {
    pub id: String,
    pub name: String,
    pub civilization: String,
    pub capital: String,
    pub traits: Vec<Trait>,
    pub primary_color: [u8; 3],
    pub secondary_color: [u8; 3],
    /// Agenda name and description, when the leader has one.
    pub agenda: Option<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: i32,
    pub moves: i32,
    pub sight: i32,
    pub combat: i32,
    pub ranged_combat: i32,
    pub range: i32,
    pub bombard: i32,
    /// `DOMAIN_LAND`, `DOMAIN_SEA`, or `DOMAIN_AIR`.
    pub domain: String,
    pub formation_class: String,
    pub promotion_class: Option<String>,
    pub prereq_tech: Option<String>,
    pub prereq_civic: Option<String>,
    pub found_city: bool,
    pub found_religion: bool,
    pub build_charges: i32,
    pub spread_charges: i32,
    pub religious_strength: i32,
    pub can_capture: bool,
    pub unique_to: Option<String>,
    pub replaces: Option<String>,
    pub maintenance: i32,
    pub obsolete_tech: Option<String>,
    pub can_train: bool,
    /// `YIELD_FAITH` for units bought with Faith.
    pub purchase_yield: Option<String>,
    pub must_purchase: bool,
    /// Strategic resource needed to train it (e.g. Iron for Swordsmen).
    pub strategic_resource: Option<String>,
    pub barbarian: bool,
    pub anti_air: i32,
    pub make_trade_route: bool,
}

#[derive(Clone, Debug)]
pub struct Building {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: i32,
    pub prereq_tech: Option<String>,
    pub prereq_civic: Option<String>,
    pub prereq_district: Option<String>,
    pub prereq_buildings: Vec<String>,
    pub yields: Yields,
    pub housing: i32,
    pub amenities: i32,
    pub maintenance: i32,
    pub wonder: bool,
    pub outer_defense: i32,
    pub great_person_points: Vec<(String, i32)>,
    pub unique_to: Option<String>,
    pub replaces: Option<String>,
    pub max_world_instances: i32,
    /// Worship buildings, unlocked by a religion's belief.
    pub enabled_by_religion: bool,
}

#[derive(Clone, Debug)]
pub struct Adjacency {
    pub yield_index: usize,
    pub amount: i32,
    pub tiles_required: i32,
    pub other_district: bool,
    pub adjacent_terrain: Option<String>,
    pub adjacent_feature: Option<String>,
    pub adjacent_district: Option<String>,
    pub adjacent_river: bool,
    pub adjacent_wonder: bool,
    pub adjacent_resource: bool,
    pub adjacent_natural_wonder: bool,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct District {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: i32,
    pub prereq_tech: Option<String>,
    pub prereq_civic: Option<String>,
    pub requires_population: bool,
    pub city_center: bool,
    pub coast: bool,
    pub one_per_city: bool,
    pub housing: i32,
    pub amenities: i32,
    pub great_person_points: Vec<(String, i32)>,
    pub adjacencies: Vec<Adjacency>,
    pub unique_to: Option<String>,
    pub replaces: Option<String>,
    pub internal_only: bool,
}

#[derive(Clone, Debug)]
pub struct Research {
    pub id: String,
    pub name: String,
    pub cost: i32,
    pub era: String,
    pub prereqs: Vec<String>,
    /// Row in the tree view, from the game's own layout.
    pub tree_row: i32,
    /// "Eureka"/"Inspiration" trigger text, when the item has a boost.
    pub boost: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Government {
    pub id: String,
    pub name: String,
    pub prereq_civic: Option<String>,
    pub bonus: String,
    pub slots: Vec<(String, i32)>,
}

#[derive(Clone, Debug)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prereq_civic: Option<String>,
    /// `SLOT_MILITARY`, `SLOT_ECONOMIC`, `SLOT_DIPLOMATIC`, `SLOT_GREAT_PERSON`.
    pub slot: String,
    pub obsoleted_by: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Belief {
    pub id: String,
    pub name: String,
    pub description: String,
    /// `BELIEF_CLASS_PANTHEON`, `_FOUNDER`, `_FOLLOWER`, `_WORSHIP`, `_ENHANCER`.
    pub class: String,
}

#[derive(Clone, Debug)]
pub struct Religion {
    pub id: String,
    pub name: String,
    pub pantheon: bool,
    pub color: [u8; 3],
}

#[derive(Clone, Debug)]
pub struct Era {
    pub id: String,
    pub name: String,
    pub index: i32,
    pub great_person_base_cost: i32,
}

#[derive(Clone, Debug)]
pub struct GreatPersonClass {
    pub id: String,
    pub name: String,
    pub unit: String,
    pub district: Option<String>,
}

#[derive(Clone, Debug)]
pub struct GreatPerson {
    pub id: String,
    pub name: String,
    pub class: String,
    pub era: String,
    pub ability: String,
}

#[derive(Clone, Debug)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: i32,
    pub prereq_tech: Option<String>,
    pub prereq_district: Option<String>,
    pub space_race: bool,
}

/// A Eureka (tech) or Inspiration (civic) and what triggers it.
#[derive(Clone, Debug)]
pub struct Boost {
    pub tech: Option<String>,
    pub civic: Option<String>,
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
    pub boosting_civic: Option<String>,
    pub count: i32,
    pub description: String,
}

/// One modifier as Civ VI defines it, for WayCiv's simplified effect
/// interpreter: what it does (`effect`), what it applies to (`collection`),
/// and its arguments.
#[derive(Clone, Debug)]
pub struct Modifier {
    pub id: String,
    pub effect: String,
    pub collection: String,
    pub arguments: HashMap<String, String>,
    pub has_requirements: bool,
}

#[derive(Default)]
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
    pub civics: Vec<Research>,
    pub governments: Vec<Government>,
    pub policies: Vec<Policy>,
    pub beliefs: Vec<Belief>,
    pub religions: Vec<Religion>,
    pub eras: Vec<Era>,
    pub great_person_classes: Vec<GreatPersonClass>,
    pub great_people: Vec<GreatPerson>,
    pub projects: Vec<Project>,
    pub boosts: Vec<Boost>,
    /// Modifier id -> modifier.
    pub modifiers: HashMap<String, Modifier>,
    /// Owner type (policy, belief, trait, building, ...) -> its modifier ids.
    pub owned_modifiers: HashMap<String, Vec<String>>,
    pub text: Text,
}

impl Content {
    /// Loads everything from the installed game (`CIV6_DIR` or the default
    /// Steam location).
    pub fn load_installed() -> Result<Self> {
        let assets = steam_assets()?;
        let database = build_standard(&assets)?;
        let text = Text::load(&assets)?;
        let colors = Colors::load(&assets)?;
        Self::from_database(database.connection(), text, &colors)
    }

    pub fn from_database(db: &Connection, text: Text, colors: &Colors) -> Result<Self> {
        let mut content = Content { text, ..Default::default() };

        let yields = |table: &str, key: &str| -> Result<HashMap<String, Yields>> {
            let mut map: HashMap<String, Yields> = HashMap::new();
            let sql = format!("SELECT {key}, YieldType, YieldChange FROM {table}");
            for (id, yield_type, amount) in query(db, &sql, |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i32>(2)?)))? {
                if let Some(i) = yield_index(&yield_type) {
                    map.entry(id).or_default()[i] += amount;
                }
            }
            Ok(map)
        };
        let pairs = |sql: &str| -> Result<HashMap<String, Vec<String>>> {
            let mut map: HashMap<String, Vec<String>> = HashMap::new();
            for (a, b) in query(db, sql, |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))? {
                map.entry(a).or_default().push(b);
            }
            Ok(map)
        };
        let replaces = |table: &str, unique: &str, replaced: &str| -> Result<HashMap<String, String>> {
            Ok(query(db, &format!("SELECT {unique}, {replaced} FROM {table}"), |r| Ok((r.get(0)?, r.get(1)?)))?
                .into_iter()
                .collect())
        };
        // Trait -> civilization or leader that owns it, for unique items.
        let trait_owner: HashMap<String, String> = query(
            db,
            "SELECT TraitType, CivilizationType FROM CivilizationTraits UNION ALL SELECT TraitType, LeaderType FROM LeaderTraits",
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?
        .into_iter()
        .collect();
        let owner = |trait_type: Option<String>| trait_type.and_then(|tr| trait_owner.get(&tr).cloned());
        let text = &content.text;
        let name = |key: String| text.get(&key);
        let opt_name = |key: Option<String>| key.map(|k| text.get(&k)).unwrap_or_default();

        let terrain_yields = yields("Terrain_YieldChanges", "TerrainType")?;
        let terrains = query(
            db,
            "SELECT TerrainType, Name, Hills, Mountain, Water, ShallowWater, MovementCost, DefenseModifier FROM Terrains",
            |r| {
                Ok(Terrain {
                    id: r.get(0)?,
                    name: name(r.get(1)?),
                    yields: Yields::default(),
                    hills: flag(r, 2)?,
                    mountain: flag(r, 3)?,
                    water: flag(r, 4)?,
                    shallow_water: flag(r, 5)?,
                    movement_cost: num(r, 6)?,
                    defense_modifier: num(r, 7)?,
                })
            },
        )?;
        let feature_yields = yields("Feature_YieldChanges", "FeatureType")?;
        let feature_terrains = pairs("SELECT FeatureType, TerrainType FROM Feature_ValidTerrains")?;
        let features = query(
            db,
            "SELECT FeatureType, Name, MovementChange, DefenseModifier, NaturalWonder, Impassable, Removable, Forest FROM Features",
            |r| {
                Ok(Feature {
                    id: r.get(0)?,
                    name: name(r.get(1)?),
                    yields: Yields::default(),
                    movement_change: num(r, 2)?,
                    defense_modifier: num(r, 3)?,
                    natural_wonder: flag(r, 4)?,
                    impassable: flag(r, 5)?,
                    removable: flag(r, 6)?,
                    forest: flag(r, 7)?,
                    valid_terrains: Vec::new(),
                })
            },
        )?;
        let resource_yields = yields("Resource_YieldChanges", "ResourceType")?;
        let resource_terrains = pairs("SELECT ResourceType, TerrainType FROM Resource_ValidTerrains")?;
        let resource_features = pairs("SELECT ResourceType, FeatureType FROM Resource_ValidFeatures")?;
        let resources = query(
            db,
            "SELECT ResourceType, Name, ResourceClassType, PrereqTech, Frequency, SeaFrequency FROM Resources",
            |r| {
                Ok(Resource {
                    id: r.get(0)?,
                    name: name(r.get(1)?),
                    class: r.get(2)?,
                    yields: Yields::default(),
                    prereq_tech: r.get(3)?,
                    frequency: num(r, 4)?,
                    sea_frequency: num(r, 5)?,
                    valid_terrains: Vec::new(),
                    valid_features: Vec::new(),
                })
            },
        )?;
        let improvement_yields = yields("Improvement_YieldChanges", "ImprovementType")?;
        let improvement_terrains = pairs("SELECT ImprovementType, TerrainType FROM Improvement_ValidTerrains")?;
        let improvement_features = pairs("SELECT ImprovementType, FeatureType FROM Improvement_ValidFeatures")?;
        let improvement_resources = pairs("SELECT ImprovementType, ResourceType FROM Improvement_ValidResources")?;
        let improvements = query(
            db,
            "SELECT ImprovementType, Name, PrereqTech, PrereqCivic, Buildable, TraitType FROM Improvements",
            |r| {
                Ok(Improvement {
                    id: r.get(0)?,
                    name: name(r.get(1)?),
                    yields: Yields::default(),
                    prereq_tech: r.get(2)?,
                    prereq_civic: r.get(3)?,
                    buildable: flag(r, 4)?,
                    unique_to: owner(r.get(5)?),
                    valid_terrains: Vec::new(),
                    valid_features: Vec::new(),
                    valid_resources: Vec::new(),
                })
            },
        )?;

        let traits: HashMap<String, Trait> = query(db, "SELECT TraitType, Name, Description FROM Traits", |r| {
            let description: Option<String> = r.get(2)?;
            Ok(Trait {
                id: r.get(0)?,
                name: opt_name(r.get(1)?),
                description: description.map(|k| text.get_or_empty(&k)).unwrap_or_default(),
            })
        })?
        .into_iter()
        .map(|t| (t.id.clone(), t))
        .collect();
        let named_traits = |ids: Option<&Vec<String>>| -> Vec<Trait> {
            ids.into_iter()
                .flatten()
                .filter_map(|id| traits.get(id).cloned())
                .filter(|t| !t.name.is_empty() && !t.name.starts_with("LOC_"))
                .collect()
        };
        let civ_traits = pairs("SELECT CivilizationType, TraitType FROM CivilizationTraits")?;
        let leader_traits = pairs("SELECT LeaderType, TraitType FROM LeaderTraits")?;
        let city_names = pairs("SELECT CivilizationType, CityName FROM CityNames ORDER BY CivilizationType, SortIndex, ID")?;
        let civilizations = query(
            db,
            "SELECT CivilizationType, Name, Adjective FROM Civilizations WHERE StartingCivilizationLevelType = 'CIVILIZATION_LEVEL_FULL_CIV'",
            |r| {
                let id: String = r.get(0)?;
                Ok(Civilization {
                    name: name(r.get(1)?),
                    adjective: name(r.get(2)?),
                    traits: named_traits(civ_traits.get(&id)),
                    city_names: city_names.get(&id).into_iter().flatten().map(|k| text.get(k)).collect(),
                    id,
                })
            },
        )?;
        let city_states = query(
            db,
            "SELECT c.CivilizationType, c.Name, COALESCE(p.Value, '') FROM Civilizations c \
             LEFT JOIN TypeProperties p ON p.Type = c.CivilizationType AND p.Name = 'CityStateCategory' \
             WHERE c.StartingCivilizationLevelType = 'CIVILIZATION_LEVEL_CITY_STATE'",
            |r| Ok(CityState { id: r.get(0)?, name: name(r.get(1)?), category: r.get(2)? }),
        )?;
        let agendas: HashMap<String, (String, String)> = query(
            db,
            "SELECT h.LeaderType, a.Name, a.Description FROM HistoricalAgendas h JOIN Agendas a ON a.AgendaType = h.AgendaType",
            |r| Ok((r.get(0)?, (opt_name(r.get(1)?), opt_name(r.get(2)?)))),
        )?
        .into_iter()
        .collect();
        let leaders = query(
            db,
            "SELECT l.LeaderType, l.Name, cl.CivilizationType, COALESCE(cl.CapitalName, '') FROM Leaders l \
             JOIN CivilizationLeaders cl ON cl.LeaderType = l.LeaderType \
             JOIN Civilizations c ON c.CivilizationType = cl.CivilizationType \
             WHERE c.StartingCivilizationLevelType = 'CIVILIZATION_LEVEL_FULL_CIV'",
            |r| {
                let id: String = r.get(0)?;
                let (primary_color, secondary_color) = colors.player(&id).unwrap_or(([200, 200, 200], [40, 40, 40]));
                Ok(Leader {
                    name: name(r.get(1)?),
                    civilization: r.get(2)?,
                    capital: opt_name(r.get(3)?),
                    traits: named_traits(leader_traits.get(&id)),
                    primary_color,
                    secondary_color,
                    agenda: agendas.get(&id).cloned(),
                    id,
                })
            },
        )?;

        let unit_replaces = replaces("UnitReplaces", "CivUniqueUnitType", "ReplacesUnitType")?;
        let unit_maintenance: HashMap<String, i32> =
            query(db, "SELECT UnitType, Maintenance FROM Units", |r| Ok((r.get(0)?, r.get(1)?)))?.into_iter().collect();
        let units = query(
            db,
            "SELECT UnitType, Name, Description, Cost, BaseMoves, BaseSightRange, Combat, RangedCombat, Range, Bombard, Domain, \
             FormationClass, PromotionClass, PrereqTech, PrereqCivic, FoundCity, FoundReligion, BuildCharges, SpreadCharges, \
             ReligiousStrength, CanCapture, TraitType, ObsoleteTech, CanTrain, PurchaseYield, MustPurchase, StrategicResource, AntiAirCombat, MakeTradeRoute FROM Units",
            |r| {
                let id: String = r.get(0)?;
                Ok(Unit {
                    name: name(r.get(1)?),
                    description: opt_name(r.get(2)?),
                    cost: num(r, 3)?,
                    moves: num(r, 4)?,
                    sight: num(r, 5)?,
                    combat: num(r, 6)?,
                    ranged_combat: num(r, 7)?,
                    range: num(r, 8)?,
                    bombard: num(r, 9)?,
                    domain: r.get(10)?,
                    formation_class: r.get(11)?,
                    promotion_class: r.get(12)?,
                    prereq_tech: r.get(13)?,
                    prereq_civic: r.get(14)?,
                    found_city: flag(r, 15)?,
                    found_religion: flag(r, 16)?,
                    build_charges: num(r, 17)?,
                    spread_charges: num(r, 18)?,
                    religious_strength: num(r, 19)?,
                    can_capture: flag(r, 20)?,
                    unique_to: owner(r.get(21)?),
                    replaces: unit_replaces.get(&id).cloned(),
                    maintenance: unit_maintenance.get(&id).copied().unwrap_or(0),
                    obsolete_tech: r.get(22)?,
                    can_train: flag(r, 23)?,
                    purchase_yield: r.get(24)?,
                    must_purchase: flag(r, 25)?,
                    strategic_resource: r.get(26)?,
                    barbarian: r.get::<_, Option<String>>(21)?.is_some_and(|t| t.contains("BARBARIAN")) || id.contains("BARBARIAN"),
                    anti_air: num(r, 27)?,
                    make_trade_route: flag(r, 28)?,
                    id,
                })
            },
        )?;

        let building_yields = yields("Building_YieldChanges", "BuildingType")?;
        let building_prereqs = pairs("SELECT Building, PrereqBuilding FROM BuildingPrereqs")?;
        let building_replaces = replaces("BuildingReplaces", "CivUniqueBuildingType", "ReplacesBuildingType")?;
        let building_gpp = great_person_points(db, "Building_GreatPersonPoints", "BuildingType")?;
        let buildings = query(
            db,
            "SELECT BuildingType, Name, Description, Cost, PrereqTech, PrereqCivic, PrereqDistrict, Housing, Entertainment, \
             Maintenance, IsWonder, OuterDefenseHitPoints, TraitType, MaxWorldInstances, EnabledByReligion FROM Buildings",
            |r| {
                let id: String = r.get(0)?;
                Ok(Building {
                    name: name(r.get(1)?),
                    description: opt_name(r.get(2)?),
                    cost: num(r, 3)?,
                    prereq_tech: r.get(4)?,
                    prereq_civic: r.get(5)?,
                    prereq_district: r.get(6)?,
                    prereq_buildings: building_prereqs.get(&id).cloned().unwrap_or_default(),
                    yields: building_yields.get(&id).copied().unwrap_or_default(),
                    housing: num(r, 7)?,
                    amenities: num(r, 8)?,
                    maintenance: num(r, 9)?,
                    wonder: flag(r, 10)?,
                    outer_defense: num(r, 11)?,
                    great_person_points: building_gpp.get(&id).cloned().unwrap_or_default(),
                    unique_to: owner(r.get(12)?),
                    replaces: building_replaces.get(&id).cloned(),
                    max_world_instances: num(r, 13)?,
                    enabled_by_religion: flag(r, 14)?,
                    id,
                })
            },
        )?;

        let adjacencies = district_adjacencies(db, text)?;
        let district_replaces = replaces("DistrictReplaces", "CivUniqueDistrictType", "ReplacesDistrictType")?;
        let district_gpp = great_person_points(db, "District_GreatPersonPoints", "DistrictType")?;
        let districts = query(
            db,
            "SELECT DistrictType, Name, Description, Cost, PrereqTech, PrereqCivic, RequiresPopulation, CityCenter, Coast, \
             OnePerCity, Housing, Entertainment, TraitType, InternalOnly FROM Districts",
            |r| {
                let id: String = r.get(0)?;
                Ok(District {
                    name: name(r.get(1)?),
                    description: opt_name(r.get(2)?),
                    cost: num(r, 3)?,
                    prereq_tech: r.get(4)?,
                    prereq_civic: r.get(5)?,
                    requires_population: flag(r, 6)?,
                    city_center: flag(r, 7)?,
                    coast: flag(r, 8)?,
                    one_per_city: flag(r, 9)?,
                    housing: num(r, 10)?,
                    amenities: num(r, 11)?,
                    great_person_points: district_gpp.get(&id).cloned().unwrap_or_default(),
                    adjacencies: adjacencies.get(&id).cloned().unwrap_or_default(),
                    unique_to: owner(r.get(12)?),
                    replaces: district_replaces.get(&id).cloned(),
                    internal_only: flag(r, 13)?,
                    id,
                })
            },
        )?;

        let tech_prereqs = pairs("SELECT Technology, PrereqTech FROM TechnologyPrereqs")?;
        let tech_boosts: HashMap<String, String> = query(
            db,
            "SELECT TechnologyType, TriggerDescription FROM Boosts WHERE TechnologyType IS NOT NULL",
            |r| Ok((r.get(0)?, opt_name(r.get(1)?))),
        )?
        .into_iter()
        .collect();
        let techs = query(db, "SELECT TechnologyType, Name, Cost, EraType, UITreeRow FROM Technologies", |r| {
            let id: String = r.get(0)?;
            Ok(Research {
                name: name(r.get(1)?),
                cost: num(r, 2)?,
                era: r.get(3)?,
                prereqs: tech_prereqs.get(&id).cloned().unwrap_or_default(),
                tree_row: r.get::<_, Option<i32>>(4)?.unwrap_or(0),
                boost: tech_boosts.get(&id).cloned(),
                id,
            })
        })?;
        let civic_prereqs = pairs("SELECT Civic, PrereqCivic FROM CivicPrereqs")?;
        let civic_boosts: HashMap<String, String> = query(
            db,
            "SELECT CivicType, TriggerDescription FROM Boosts WHERE CivicType IS NOT NULL",
            |r| Ok((r.get(0)?, opt_name(r.get(1)?))),
        )?
        .into_iter()
        .collect();
        let civics = query(db, "SELECT CivicType, Name, Cost, EraType, UITreeRow FROM Civics", |r| {
            let id: String = r.get(0)?;
            Ok(Research {
                name: name(r.get(1)?),
                cost: num(r, 2)?,
                era: r.get(3)?,
                prereqs: civic_prereqs.get(&id).cloned().unwrap_or_default(),
                tree_row: r.get::<_, Option<i32>>(4)?.unwrap_or(0),
                boost: civic_boosts.get(&id).cloned(),
                id,
            })
        })?;

        let mut slots: HashMap<String, Vec<(String, i32)>> = HashMap::new();
        for (government, slot, count) in query(
            db,
            "SELECT GovernmentType, GovernmentSlotType, NumSlots FROM Government_SlotCounts",
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i32>(2)?)),
        )? {
            slots.entry(government).or_default().push((slot, count));
        }
        let governments = query(db, "SELECT GovernmentType, Name, PrereqCivic, InherentBonusDesc FROM Governments", |r| {
            let id: String = r.get(0)?;
            Ok(Government {
                name: name(r.get(1)?),
                prereq_civic: r.get(2)?,
                bonus: opt_name(r.get(3)?),
                slots: slots.get(&id).cloned().unwrap_or_default(),
                id,
            })
        })?;
        let obsolete: HashMap<String, String> =
            query(db, "SELECT PolicyType, ObsoletePolicy FROM ObsoletePolicies WHERE ObsoletePolicy IS NOT NULL", |r| Ok((r.get(0)?, r.get(1)?)))?
                .into_iter()
                .collect();
        let policies = query(
            db,
            "SELECT PolicyType, Name, Description, PrereqCivic, GovernmentSlotType FROM Policies",
            |r| {
                let id: String = r.get(0)?;
                Ok(Policy {
                    name: name(r.get(1)?),
                    description: opt_name(r.get(2)?),
                    prereq_civic: r.get(3)?,
                    slot: r.get(4)?,
                    obsoleted_by: obsolete.get(&id).cloned(),
                    id,
                })
            },
        )?;
        let beliefs = query(db, "SELECT BeliefType, Name, Description, BeliefClassType FROM Beliefs", |r| {
            Ok(Belief { id: r.get(0)?, name: name(r.get(1)?), description: opt_name(r.get(2)?), class: r.get(3)? })
        })?;
        let religions = query(db, "SELECT ReligionType, Name, Pantheon, Color FROM Religions", |r| {
            let color: Option<String> = r.get(3)?;
            Ok(Religion {
                id: r.get(0)?,
                name: name(r.get(1)?),
                pantheon: flag(r, 2)?,
                color: color.and_then(|c| colors.named(&c)).unwrap_or([220, 220, 220]),
            })
        })?;
        let eras = query(db, "SELECT EraType, Name, ChronologyIndex, GreatPersonBaseCost FROM Eras ORDER BY ChronologyIndex", |r| {
            Ok(Era { id: r.get(0)?, name: name(r.get(1)?), index: num(r, 2)?, great_person_base_cost: num(r, 3)? })
        })?;
        let great_person_classes = query(
            db,
            "SELECT GreatPersonClassType, Name, UnitType, DistrictType FROM GreatPersonClasses",
            |r| Ok(GreatPersonClass { id: r.get(0)?, name: name(r.get(1)?), unit: r.get(2)?, district: r.get(3)? }),
        )?;
        let great_people = query(
            db,
            "SELECT GreatPersonIndividualType, Name, GreatPersonClassType, EraType, ActionEffectTextOverride FROM GreatPersonIndividuals",
            |r| {
                Ok(GreatPerson {
                    id: r.get(0)?,
                    name: name(r.get(1)?),
                    class: r.get(2)?,
                    era: r.get(3)?,
                    ability: opt_name(r.get(4)?),
                })
            },
        )?;
        let projects = query(
            db,
            "SELECT ProjectType, Name, Description, Cost, PrereqTech, PrereqDistrict, SpaceRace FROM Projects",
            |r| {
                Ok(Project {
                    id: r.get(0)?,
                    name: name(r.get(1)?),
                    description: opt_name(r.get(2)?),
                    cost: num(r, 3)?,
                    prereq_tech: r.get(4)?,
                    prereq_district: r.get(5)?,
                    space_race: flag(r, 6)?,
                })
            },
        )?;
        let boosts = query(
            db,
            "SELECT TechnologyType, CivicType, Boost, BoostClass, Unit1Type, BuildingType, ImprovementType, DistrictType, \
             ResourceType, BoostingTechType, BoostingCivicType, NumItems, TriggerDescription FROM Boosts",
            |r| {
                Ok(Boost {
                    tech: r.get(0)?,
                    civic: r.get(1)?,
                    percent: num(r, 2)?,
                    class: r.get(3)?,
                    unit: r.get(4)?,
                    building: r.get(5)?,
                    improvement: r.get(6)?,
                    district: r.get(7)?,
                    resource: r.get(8)?,
                    boosting_tech: r.get(9)?,
                    boosting_civic: r.get(10)?,
                    count: num(r, 11)?,
                    description: opt_name(r.get(12)?),
                })
            },
        )?;
        let (modifiers, owned_modifiers) = load_modifiers(db)?;

        // Attach yields and placement rules collected above.
        content.terrains = terrains
            .into_iter()
            .map(|mut x| {
                x.yields = terrain_yields.get(&x.id).copied().unwrap_or_default();
                x
            })
            .collect();
        content.features = features
            .into_iter()
            .map(|mut x| {
                x.yields = feature_yields.get(&x.id).copied().unwrap_or_default();
                x.valid_terrains = feature_terrains.get(&x.id).cloned().unwrap_or_default();
                x
            })
            .collect();
        content.resources = resources
            .into_iter()
            .map(|mut x| {
                x.yields = resource_yields.get(&x.id).copied().unwrap_or_default();
                x.valid_terrains = resource_terrains.get(&x.id).cloned().unwrap_or_default();
                x.valid_features = resource_features.get(&x.id).cloned().unwrap_or_default();
                x
            })
            .collect();
        content.improvements = improvements
            .into_iter()
            .map(|mut x| {
                x.yields = improvement_yields.get(&x.id).copied().unwrap_or_default();
                x.valid_terrains = improvement_terrains.get(&x.id).cloned().unwrap_or_default();
                x.valid_features = improvement_features.get(&x.id).cloned().unwrap_or_default();
                x.valid_resources = improvement_resources.get(&x.id).cloned().unwrap_or_default();
                x
            })
            .collect();
        content.civilizations = civilizations;
        content.city_states = city_states;
        content.leaders = leaders;
        content.units = units;
        content.buildings = buildings;
        content.districts = districts;
        content.techs = techs;
        content.civics = civics;
        content.governments = governments;
        content.policies = policies;
        content.beliefs = beliefs;
        content.religions = religions;
        content.eras = eras;
        content.great_person_classes = great_person_classes;
        content.great_people = great_people;
        content.projects = projects;
        content.boosts = boosts;
        content.modifiers = modifiers;
        content.owned_modifiers = owned_modifiers;
        Ok(content)
    }

    /// Modifiers attached to an owner type (policy, belief, trait, building,
    /// district, government, great person, ...).
    pub fn modifiers_of(&self, owner: &str) -> impl Iterator<Item = &Modifier> {
        self.owned_modifiers.get(owner).into_iter().flatten().filter_map(|id| self.modifiers.get(id))
    }
}

/// Integer column that may be NULL (treated as 0).
fn num(r: &Row, i: usize) -> rusqlite::Result<i32> {
    Ok(r.get::<_, Option<i32>>(i)?.unwrap_or(0))
}

/// Boolean column that may be NULL (treated as false).
fn flag(r: &Row, i: usize) -> rusqlite::Result<bool> {
    Ok(r.get::<_, Option<bool>>(i)?.unwrap_or(false))
}

fn query<T>(db: &Connection, sql: &str, map: impl FnMut(&Row) -> rusqlite::Result<T>) -> Result<Vec<T>> {
    let mut statement = db.prepare(sql).map_err(|e| format!("{e}\n  in: {sql}"))?;
    let rows = statement.query_map([], map)?.collect::<rusqlite::Result<Vec<T>>>()?;
    Ok(rows)
}

fn great_person_points(db: &Connection, table: &str, key: &str) -> Result<HashMap<String, Vec<(String, i32)>>> {
    let mut map: HashMap<String, Vec<(String, i32)>> = HashMap::new();
    let sql = format!("SELECT {key}, GreatPersonClassType, PointsPerTurn FROM {table}");
    for (id, class, points) in query(db, &sql, |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i32>(2)?)))? {
        map.entry(id).or_default().push((class, points));
    }
    Ok(map)
}

fn district_adjacencies(db: &Connection, text: &Text) -> Result<HashMap<String, Vec<Adjacency>>> {
    let mut map: HashMap<String, Vec<Adjacency>> = HashMap::new();
    let rows = query(
        db,
        "SELECT d.DistrictType, a.YieldType, a.YieldChange, a.TilesRequired, a.OtherDistrictAdjacent, a.AdjacentTerrain, \
         a.AdjacentFeature, a.AdjacentDistrict, a.AdjacentRiver, a.AdjacentWonder, a.AdjacentResource, \
         a.AdjacentNaturalWonder, a.Description \
         FROM District_Adjacencies d JOIN Adjacency_YieldChanges a ON a.ID = d.YieldChangeId",
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                Adjacency {
                    yield_index: 0,
                    amount: num(r, 2)?,
                    tiles_required: num(r, 3)?,
                    other_district: flag(r, 4)?,
                    adjacent_terrain: r.get(5)?,
                    adjacent_feature: r.get(6)?,
                    adjacent_district: r.get(7)?,
                    adjacent_river: flag(r, 8)?,
                    adjacent_wonder: flag(r, 9)?,
                    adjacent_resource: flag(r, 10)?,
                    adjacent_natural_wonder: flag(r, 11)?,
                    description: r.get::<_, Option<String>>(12)?.map(|k| text.get(&k)).unwrap_or_default(),
                },
            ))
        },
    )?;
    for (district, yield_type, mut adjacency) in rows {
        if let Some(i) = yield_index(&yield_type) {
            adjacency.yield_index = i;
            map.entry(district).or_default().push(adjacency);
        }
    }
    Ok(map)
}

fn load_modifiers(db: &Connection) -> Result<(HashMap<String, Modifier>, HashMap<String, Vec<String>>)> {
    let mut arguments: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (id, name, value) in query(db, "SELECT ModifierId, Name, Value FROM ModifierArguments", |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, rusqlite::types::Value>(2)?))
    })? {
        let value = match value {
            rusqlite::types::Value::Integer(i) => i.to_string(),
            rusqlite::types::Value::Real(f) => f.to_string(),
            rusqlite::types::Value::Text(t) => t,
            _ => String::new(),
        };
        arguments.entry(id).or_default().insert(name, value);
    }
    let modifiers = query(
        db,
        "SELECT m.ModifierId, COALESCE(d.EffectType, ''), COALESCE(d.CollectionType, ''), \
         m.SubjectRequirementSetId IS NOT NULL OR m.OwnerRequirementSetId IS NOT NULL \
         FROM Modifiers m LEFT JOIN DynamicModifiers d ON d.ModifierType = m.ModifierType",
        |r| {
            let id: String = r.get(0)?;
            Ok(Modifier {
                arguments: arguments.remove(&id).unwrap_or_default(),
                effect: r.get(1)?,
                collection: r.get(2)?,
                has_requirements: flag(r, 3)?,
                id,
            })
        },
    )?;
    let modifiers: HashMap<_, _> = modifiers.into_iter().map(|m| (m.id.clone(), m)).collect();

    let mut owned: HashMap<String, Vec<String>> = HashMap::new();
    for (table, key) in [
        ("PolicyModifiers", "PolicyType"),
        ("BeliefModifiers", "BeliefType"),
        ("TraitModifiers", "TraitType"),
        ("BuildingModifiers", "BuildingType"),
        ("DistrictModifiers", "DistrictType"),
        ("GovernmentModifiers", "GovernmentType"),
        ("GreatPersonIndividualActionModifiers", "GreatPersonIndividualType"),
        ("TechnologyModifiers", "TechnologyType"),
        ("CivicModifiers", "CivicType"),
        ("UnitAbilityModifiers", "UnitAbilityType"),
        ("UnitPromotionModifiers", "UnitPromotionType"),
        ("ImprovementModifiers", "ImprovementType"),
    ] {
        let sql = format!("SELECT {key}, ModifierId FROM {table}");
        let Ok(rows) = query(db, &sql, |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) else { continue };
        for (owner, id) in rows {
            owned.entry(owner).or_default().push(id);
        }
    }
    Ok((modifiers, owned))
}

/// Player and named colors from the game's UI color files
/// (`ui/colors/*.xml` and each DLC's `*_colors.xml`). The game loads these into
/// a separate database, so they are read directly here.
#[derive(Default)]
pub struct Colors {
    named: HashMap<String, [u8; 3]>,
    players: HashMap<String, (String, String)>,
}

impl Colors {
    pub fn load(assets: &Path) -> Result<Self> {
        let mut colors = Colors::default();
        for file in files_in_load_order(&assets.join("base/assets/ui/colors"), "xml")? {
            colors.load_file(&file);
        }
        if let Ok(entries) = std::fs::read_dir(assets.join("dlc")) {
            let mut dirs: Vec<_> = entries.flatten().map(|e| e.path().join("data")).filter(|p| p.is_dir()).collect();
            dirs.sort();
            for dir in dirs {
                for file in files_in_load_order(&dir, "xml")? {
                    let name = file.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
                    if name.ends_with("_colors.xml") && !name.contains("scenario") {
                        colors.load_file(&file);
                    }
                }
            }
        }
        Ok(colors)
    }

    fn load_file(&mut self, path: &Path) {
        let Ok(source) = std::fs::read_to_string(path) else { return };
        let Ok(document) = roxmltree::Document::parse(source.trim_start_matches('\u{feff}')) else { return };
        let field = |row: roxmltree::Node, name: &str| -> Option<String> {
            row.attribute(name)
                .map(str::to_owned)
                .or_else(|| row.children().find(|c| c.has_tag_name(name)).and_then(|c| c.text()).map(|t| t.trim().to_owned()))
        };
        for table in document.root_element().children().filter(roxmltree::Node::is_element) {
            for row in table.children().filter(|r| r.has_tag_name("Row") || r.has_tag_name("Replace")) {
                let Some(id) = field(row, "Type") else { continue };
                match table.tag_name().name() {
                    "Colors" => {
                        if let Some(rgb) = field(row, "Color").and_then(|c| parse_rgb(&c)) {
                            self.named.insert(id, rgb);
                        }
                    }
                    "PlayerColors" => {
                        if let (Some(primary), Some(secondary)) = (field(row, "PrimaryColor"), field(row, "SecondaryColor")) {
                            self.players.insert(id, (primary, secondary));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn named(&self, name: &str) -> Option<[u8; 3]> {
        self.named.get(name).copied()
    }

    /// (primary, secondary) RGB for a leader.
    pub fn player(&self, leader: &str) -> Option<([u8; 3], [u8; 3])> {
        let (primary, secondary) = self.players.get(leader)?;
        Some((self.named(primary)?, self.named(secondary)?))
    }
}

fn parse_rgb(value: &str) -> Option<[u8; 3]> {
    let parts: Vec<f32> = value.split(',').filter_map(|p| p.trim().parse().ok()).collect();
    if parts.len() < 3 {
        return None;
    }
    // Some files use 0..1 floats, most use 0..255 integers.
    let scale = if parts[..3].iter().all(|v| *v <= 1.0) && value.contains('.') { 255.0 } else { 1.0 };
    Some([(parts[0] * scale) as u8, (parts[1] * scale) as u8, (parts[2] * scale) as u8])
}

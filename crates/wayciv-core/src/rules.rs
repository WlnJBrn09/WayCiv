//! Game rules: Civ VI content from the user's install, indexed for lookups,
//! plus WayCiv's own changes (such as Ronald Reagan leading America).

use civ6_data::content::{Content, Leader, Trait};
use std::collections::HashMap;

pub use civ6_data::content::{
    Building, CityState, Civilization, District, Feature, Improvement, Research, Resource, Terrain,
    Unit, YIELDS, Yields, yield_index,
};

pub const FOOD: usize = 0;
pub const PRODUCTION: usize = 1;
pub const GOLD: usize = 2;
pub const SCIENCE: usize = 3;
pub const CULTURE: usize = 4;
pub const FAITH: usize = 5;

pub const YIELD_NAMES: [&str; 6] = ["Food", "Production", "Gold", "Science", "Culture", "Faith"];

/// America's leader in WayCiv.
pub const REAGAN: &str = "LEADER_REAGAN";

pub struct Rules {
    pub content: Content,
    /// WayCiv's AI tree (see `additions`), researched with Compute.
    pub ai_tree: Vec<Research>,
    index: HashMap<String, usize>,
}

impl Rules {
    pub fn new(mut content: Content) -> Self {
        apply_overrides(&mut content);
        crate::additions::apply(&mut content);
        let ai_tree = crate::additions::ai_tree();
        let mut index = HashMap::new();
        macro_rules! add {
            ($list:expr) => {
                for (i, item) in $list.iter().enumerate() {
                    index.insert(item.id.clone(), i);
                }
            };
        }
        add!(content.terrains);
        add!(content.features);
        add!(content.resources);
        add!(content.improvements);
        add!(content.civilizations);
        add!(content.city_states);
        add!(content.leaders);
        add!(content.units);
        add!(content.buildings);
        add!(content.districts);
        add!(content.techs);
        add!(content.civics);
        add!(content.governments);
        add!(content.policies);
        add!(content.beliefs);
        add!(content.religions);
        add!(content.eras);
        add!(content.great_person_classes);
        add!(content.great_people);
        add!(content.projects);
        add!(ai_tree);
        Self {
            content,
            ai_tree,
            index,
        }
    }

    /// Index of a content item by its Civ VI type id (`UNIT_WARRIOR`, ...).
    /// Ids are unique across all content kinds.
    pub fn idx(&self, id: &str) -> Option<usize> {
        self.index.get(id).copied()
    }

    pub fn terrain(&self, id: &str) -> Option<&Terrain> {
        self.idx(id)
            .and_then(|i| self.content.terrains.get(i))
            .filter(|t| t.id == id)
    }

    pub fn unit(&self, id: &str) -> Option<&Unit> {
        self.idx(id)
            .and_then(|i| self.content.units.get(i))
            .filter(|u| u.id == id)
    }

    pub fn building(&self, id: &str) -> Option<&Building> {
        self.idx(id)
            .and_then(|i| self.content.buildings.get(i))
            .filter(|b| b.id == id)
    }

    pub fn district(&self, id: &str) -> Option<&District> {
        self.idx(id)
            .and_then(|i| self.content.districts.get(i))
            .filter(|d| d.id == id)
    }

    pub fn tech(&self, id: &str) -> Option<usize> {
        self.idx(id)
            .filter(|&i| self.content.techs.get(i).is_some_and(|t| t.id == id))
    }

    pub fn ai_tech(&self, id: &str) -> Option<usize> {
        self.idx(id)
            .filter(|&i| self.ai_tree.get(i).is_some_and(|t| t.id == id))
    }

    pub fn civic(&self, id: &str) -> Option<usize> {
        self.idx(id)
            .filter(|&i| self.content.civics.get(i).is_some_and(|t| t.id == id))
    }

    pub fn leader(&self, id: &str) -> Option<&Leader> {
        self.idx(id)
            .and_then(|i| self.content.leaders.get(i))
            .filter(|l| l.id == id)
    }

    pub fn civilization(&self, id: &str) -> Option<&Civilization> {
        self.idx(id)
            .and_then(|i| self.content.civilizations.get(i))
            .filter(|c| c.id == id)
    }

    pub fn text(&self, key: &str) -> String {
        self.content.text.get(key)
    }

    pub fn era_index(&self, era: &str) -> usize {
        self.content
            .eras
            .iter()
            .position(|e| e.id == era)
            .unwrap_or(0)
    }

    /// The unit a civilization/leader actually builds for `unit` (its unique
    /// replacement if it has one).
    pub fn unit_for(&self, unit: &str, civ: &str, leader: &str) -> String {
        self.content
            .units
            .iter()
            .find(|u| {
                u.replaces.as_deref() == Some(unit)
                    && u.unique_to
                        .as_deref()
                        .is_some_and(|owner| owner == civ || owner == leader)
            })
            .map(|u| u.id.clone())
            .unwrap_or_else(|| unit.to_string())
    }
}

/// WayCiv's changes to Civ VI's roster: Ronald Reagan leads America in place
/// of both Teddy Roosevelt personas, with his own leader ability and agenda,
/// and Gandhi is left out.
fn apply_overrides(content: &mut Content) {
    // Not in WayCiv: Teddy Roosevelt's alternate persona (Reagan replaces him)
    // and Gandhi.
    content
        .leaders
        .retain(|l| l.id != "LEADER_T_ROOSEVELT_ROUGHRIDER" && l.id != "LEADER_GANDHI");
    if let Some(leader) = content
        .leaders
        .iter_mut()
        .find(|l| l.id == "LEADER_T_ROOSEVELT")
    {
        leader.id = REAGAN.to_string();
        leader.name = "Ronald Reagan".to_string();
        leader.traits = vec![Trait {
            id: "TRAIT_LEADER_PEACE_THROUGH_STRENGTH".to_string(),
            name: "Peace Through Strength".to_string(),
            description: [
                "Defense buildup: +15% Production toward military units, and +5 Combat Strength for units fighting inside your territory.",
                "Reaganomics: +1 Gold in every city for each Economic policy slotted, and Commercial Hubs give +2 Production.",
                "The Great Communicator: +2 Culture in the Capital. Declaring Friendship with a civilization grants an Envoy and 50 Gold.",
            ]
            .join("\n"),
        }];
        leader.agenda = Some((
            "Cold Warrior".to_string(),
            "Likes civilizations running Democracy. Dislikes civilizations running Communism or Fascism.".to_string(),
        ));
    }
}

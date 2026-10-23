//! Player bonuses from policies, governments, beliefs, and leader and civ
//! traits.
//!
//! WayCiv interprets the most common Civ VI modifier effects straight from
//! the game data when they have no requirements: production bonuses for
//! unit classes and eras, city and building yields, district and building
//! production, and great person points. Effects outside that set keep their
//! Civ VI description but have no mechanical effect yet; `is_active` tells
//! the UI which is which. Reagan's leader ability is implemented directly.

use crate::map::PlayerId;
use crate::rules::{self, Rules, Unit, yield_index};
use crate::state::{City, Game};

#[derive(Clone, Debug, Default)]
pub struct UnitProductionBonus {
    pub promotion_class: Option<String>,
    pub unit: Option<String>,
    pub domain: Option<String>,
    pub era: Option<String>,
    pub military_only: bool,
    pub percent: f32,
}

impl UnitProductionBonus {
    pub fn matches(&self, unit: &Unit, _player_era: usize, rules: &Rules) -> bool {
        if self.military_only && unit.combat == 0 && unit.ranged_combat == 0 && unit.bombard == 0 {
            return false;
        }
        if self
            .promotion_class
            .as_ref()
            .is_some_and(|c| unit.promotion_class.as_ref() != Some(c))
        {
            return false;
        }
        if self
            .unit
            .as_ref()
            .is_some_and(|u| *u != unit.id && unit.replaces.as_ref() != Some(u))
        {
            return false;
        }
        if self.domain.as_ref().is_some_and(|d| *d != unit.domain) {
            return false;
        }
        if let Some(era) = &self.era {
            let unit_era = unit
                .prereq_tech
                .as_ref()
                .and_then(|t| rules.content.techs.iter().find(|x| &x.id == t))
                .or_else(|| {
                    unit.prereq_civic
                        .as_ref()
                        .and_then(|c| rules.content.civics.iter().find(|x| &x.id == c))
                })
                .map(|r| r.era.as_str())
                .unwrap_or("ERA_ANCIENT");
            if unit_era != era {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug)]
pub struct Bonuses {
    pub city_flat: [f32; 6],
    pub city_percent: [f32; 6],
    pub capital_flat: [f32; 6],
    pub capital_percent: [f32; 6],
    pub district_flat: Vec<(usize, [f32; 6])>,
    pub building_flat: Vec<(usize, [f32; 6])>,
    pub unit_production: Vec<UnitProductionBonus>,
    pub building_production: f32,
    pub wonder_production: f32,
    pub district_production: f32,
    pub great_person_points: Vec<(usize, i32)>,
    pub housing: i32,
    pub amenities: i32,
    /// Added to every unit's combat strength.
    pub combat: i32,
    /// Added when fighting inside the player's own territory.
    pub combat_in_territory: i32,
    pub unit_xp: i32,
    pub adjacency_multiplier: f32,
    pub purchase_multiplier: f32,
    /// Extra gold per city per slotted economic policy (Reaganomics).
    pub gold_per_economic_policy: f32,
    /// Extra yields per domestic / international trade route.
    pub trade_domestic: [f32; 6],
    pub trade_international: [f32; 6],
}

impl Default for Bonuses {
    fn default() -> Self {
        Self {
            city_flat: [0.0; 6],
            city_percent: [0.0; 6],
            capital_flat: [0.0; 6],
            capital_percent: [0.0; 6],
            district_flat: Vec::new(),
            building_flat: Vec::new(),
            unit_production: Vec::new(),
            building_production: 0.0,
            wonder_production: 0.0,
            district_production: 0.0,
            great_person_points: Vec::new(),
            housing: 0,
            amenities: 0,
            combat: 0,
            combat_in_territory: 0,
            unit_xp: 0,
            adjacency_multiplier: 1.0,
            purchase_multiplier: 1.0,
            gold_per_economic_policy: 0.0,
            trade_domestic: [0.0; 6],
            trade_international: [0.0; 6],
        }
    }
}

/// Whether WayCiv applies `modifier` (see the module docs).
pub fn is_active(modifier: &civ6_data::content::Modifier) -> bool {
    !modifier.has_requirements
        && matches!(
            modifier.effect.as_str(),
            "EFFECT_ADJUST_UNIT_TAG_ERA_PRODUCTION"
                | "EFFECT_ADJUST_UNIT_PRODUCTION"
                | "EFFECT_ADJUST_UNIT_DOMAIN_PRODUCTION"
                | "EFFECT_ADJUST_CITY_YIELD_CHANGE"
                | "EFFECT_ADJUST_CITY_YIELD_MODIFIER"
                | "EFFECT_ADJUST_BUILDING_YIELD_CHANGE"
                | "EFFECT_ADJUST_DISTRICT_PRODUCTION"
                | "EFFECT_ADJUST_BUILDING_PRODUCTION"
                | "EFFECT_ADJUST_GREAT_PERSON_POINTS"
                | "EFFECT_ADJUST_PLAYER_YIELD_MODIFIER"
                | "EFFECT_ADJUST_TRADE_ROUTE_YIELD_FOR_INTERNATIONAL"
                | "EFFECT_ADJUST_TRADE_ROUTE_YIELD_FOR_DOMESTIC"
                | "EFFECT_ADJUST_TRADE_ROUTE_CAPACITY"
        )
}

impl Bonuses {
    fn apply(&mut self, rules: &Rules, modifier: &civ6_data::content::Modifier) {
        if !is_active(modifier) {
            return;
        }
        let arg = |name: &str| modifier.arguments.get(name).map(String::as_str);
        let amount: f32 = arg("Amount").and_then(|a| a.parse().ok()).unwrap_or(0.0);
        let yield_i = arg("YieldType").and_then(yield_index);
        let capital = modifier.collection.contains("CAPITAL");
        match modifier.effect.as_str() {
            "EFFECT_ADJUST_UNIT_TAG_ERA_PRODUCTION" => {
                self.unit_production.push(UnitProductionBonus {
                    promotion_class: arg("UnitPromotionClass").map(str::to_owned),
                    era: arg("EraType").map(str::to_owned),
                    percent: amount,
                    ..Default::default()
                })
            }
            "EFFECT_ADJUST_UNIT_PRODUCTION" => self.unit_production.push(UnitProductionBonus {
                unit: arg("UnitType").map(str::to_owned),
                percent: amount,
                ..Default::default()
            }),
            "EFFECT_ADJUST_UNIT_DOMAIN_PRODUCTION" => {
                self.unit_production.push(UnitProductionBonus {
                    domain: arg("Domain").map(str::to_owned),
                    percent: amount,
                    ..Default::default()
                })
            }
            "EFFECT_ADJUST_CITY_YIELD_CHANGE" => {
                if let Some(i) = yield_i {
                    if capital {
                        self.capital_flat[i] += amount
                    } else {
                        self.city_flat[i] += amount
                    }
                }
            }
            "EFFECT_ADJUST_CITY_YIELD_MODIFIER" | "EFFECT_ADJUST_PLAYER_YIELD_MODIFIER" => {
                if let Some(i) = yield_i {
                    if capital {
                        self.capital_percent[i] += amount
                    } else {
                        self.city_percent[i] += amount
                    }
                }
            }
            "EFFECT_ADJUST_BUILDING_YIELD_CHANGE" => {
                if let (Some(i), Some(building)) =
                    (yield_i, arg("BuildingType").and_then(|b| rules.idx(b)))
                {
                    let mut y = [0.0; 6];
                    y[i] = amount;
                    self.building_flat.push((building, y));
                }
            }
            "EFFECT_ADJUST_TRADE_ROUTE_YIELD_FOR_INTERNATIONAL" => {
                if let Some(i) = yield_i {
                    self.trade_international[i] += amount;
                }
            }
            "EFFECT_ADJUST_TRADE_ROUTE_YIELD_FOR_DOMESTIC" => {
                if let Some(i) = yield_i {
                    self.trade_domestic[i] += amount;
                }
            }
            "EFFECT_ADJUST_DISTRICT_PRODUCTION" => self.district_production += amount,
            "EFFECT_ADJUST_BUILDING_PRODUCTION" => {
                if arg("IsWonder").is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true")) {
                    self.wonder_production += amount;
                } else {
                    self.building_production += amount;
                }
            }
            "EFFECT_ADJUST_GREAT_PERSON_POINTS" => {
                if let Some(class) = arg("GreatPersonClassType").and_then(|c| rules.idx(c)) {
                    self.great_person_points.push((class, amount as i32));
                }
            }
            _ => {}
        }
    }
}

impl Game {
    /// Every bonus currently applying to `player`.
    pub fn bonuses(&self, player: PlayerId) -> Bonuses {
        let rules = &self.rules;
        let content = &rules.content;
        let p = &self.players[player];
        let mut bonuses = Bonuses::default();
        if !p.is_major() {
            return bonuses;
        }
        let mut owners: Vec<&str> = Vec::new();
        if let Some(g) = p.government {
            owners.push(&content.governments[g].id);
        }
        owners.extend(p.policies.iter().map(|&i| content.policies[i].id.as_str()));
        owners.extend(p.beliefs.iter().map(|&i| content.beliefs[i].id.as_str()));
        owners.extend(p.pantheon.iter().map(|&i| content.beliefs[i].id.as_str()));
        owners.extend(
            content.leaders[p.leader]
                .traits
                .iter()
                .map(|t| t.id.as_str()),
        );
        owners.extend(
            content.civilizations[p.civ]
                .traits
                .iter()
                .map(|t| t.id.as_str()),
        );
        for owner in owners {
            for modifier in content.modifiers_of(owner) {
                bonuses.apply(rules, modifier);
            }
        }

        // Ronald Reagan: Peace Through Strength.
        if content.leaders[p.leader].id == rules::REAGAN {
            bonuses.unit_production.push(UnitProductionBonus {
                military_only: true,
                percent: 15.0,
                ..Default::default()
            });
            bonuses.combat_in_territory += 5;
            bonuses.capital_flat[rules::CULTURE] += 2.0;
            let economic = p
                .policies
                .iter()
                .filter(|&&i| content.policies[i].slot == "SLOT_ECONOMIC")
                .count();
            bonuses.city_flat[rules::GOLD] += economic as f32;
            if let Some(hub) = rules.idx("DISTRICT_COMMERCIAL_HUB") {
                let mut y = [0.0; 6];
                y[rules::PRODUCTION] = 2.0;
                bonuses.district_flat.push((hub, y));
            }
        }
        self.ai_tree_bonuses(player, &mut bonuses);
        bonuses
    }

    /// City-state envoy and suzerain bonuses for a city of `owner`.
    pub(crate) fn city_state_bonus(
        &self,
        owner: crate::map::PlayerId,
        city: &City,
        mut y: [f32; 6],
    ) -> [f32; 6] {
        let p = &self.players[owner];
        for (&state, &count) in &p.envoys {
            let cs = &self.players[state];
            if !cs.alive || count == 0 {
                continue;
            }
            let i = match cs.city_state_category.as_str() {
                "SCIENTIFIC" => rules::SCIENCE,
                "CULTURAL" => rules::CULTURE,
                "TRADE" => rules::GOLD,
                "RELIGIOUS" => rules::FAITH,
                _ => rules::PRODUCTION,
            };
            if city.capital {
                y[i] += 2.0;
            }
            if count >= 3 {
                y[i] += 1.0;
            }
            if count >= 6 {
                y[i] += 1.0;
            }
            if self.suzerain(state) == Some(owner) {
                y[i] += 1.0;
            }
        }
        y
    }

    /// The major with the most envoys (at least 3, no tie) at a city-state.
    pub fn suzerain(&self, state: crate::map::PlayerId) -> Option<crate::map::PlayerId> {
        let mut counts: Vec<(crate::map::PlayerId, i32)> = self
            .players
            .iter()
            .filter(|p| p.alive && p.is_major())
            .map(|p| (p.id, p.envoys.get(&state).copied().unwrap_or(0)))
            .collect();
        counts.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
        match counts.as_slice() {
            [(a, x), (_, y), ..] if *x >= 3 && x > y => Some(*a),
            [(a, x)] if *x >= 3 => Some(*a),
            _ => None,
        }
    }
}

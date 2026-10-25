//! Player bonuses from the government, policies, beliefs, leader and
//! civilization abilities, and wonders the player owns. Each source lists
//! its effects in the data files (`wayciv_data::Effect`).

use crate::map::PlayerId;
use crate::rules::{self, Rules, Unit, yield_index};
use crate::state::{City, Game};
use wayciv_data::Effect;

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
                .map(|r| r.era.clone())
                .or_else(|| unit.unlock_year.map(|y| rules.era_of_year(y)))
                .unwrap_or_else(|| "ERA_ANCIENT".to_string());
            if &unit_era != era {
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
    /// Extra tile yields from an improvement (by improvement index).
    pub improvement_flat: Vec<(usize, [i32; 6])>,
    pub unit_production: Vec<UnitProductionBonus>,
    pub building_production: f32,
    pub wonder_production: f32,
    pub district_production: f32,
    pub great_person_points: Vec<(usize, i32)>,
    pub housing: i32,
    pub amenities: i32,
    /// Added to every unit's combat strength.
    pub combat: i32,
    /// Added to the combat strength of one promotion class.
    pub class_combat: Vec<(String, i32)>,
    /// Added when fighting inside the player's own territory.
    pub combat_in_territory: i32,
    pub unit_xp: i32,
    pub adjacency_multiplier: f32,
    pub purchase_multiplier: f32,
    /// Extra yields per domestic / international trade route.
    pub trade_domestic: [f32; 6],
    pub trade_international: [f32; 6],
    pub trade_capacity: i32,
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
            improvement_flat: Vec::new(),
            unit_production: Vec::new(),
            building_production: 0.0,
            wonder_production: 0.0,
            district_production: 0.0,
            great_person_points: Vec::new(),
            housing: 0,
            amenities: 0,
            combat: 0,
            class_combat: Vec::new(),
            combat_in_territory: 0,
            unit_xp: 0,
            adjacency_multiplier: 1.0,
            purchase_multiplier: 1.0,
            trade_domestic: [0.0; 6],
            trade_international: [0.0; 6],
            trade_capacity: 0,
        }
    }
}

impl Bonuses {
    /// Adds one effect. `slotted` counts the player's slotted policies by
    /// slot, for `PolicyYield`.
    fn apply(&mut self, rules: &Rules, effect: &Effect, slotted: &[(String, usize)]) {
        let yield_of = |y: &str| yield_index(y);
        match effect {
            Effect::CityYield {
                yield_type,
                amount,
                capital,
            } => {
                if let Some(i) = yield_of(yield_type) {
                    if *capital {
                        self.capital_flat[i] += amount;
                    } else {
                        self.city_flat[i] += amount;
                    }
                }
            }
            Effect::CityPercent {
                yield_type,
                percent,
                capital,
            } => {
                if let Some(i) = yield_of(yield_type) {
                    if *capital {
                        self.capital_percent[i] += percent;
                    } else {
                        self.city_percent[i] += percent;
                    }
                }
            }
            Effect::DistrictYield {
                district,
                yield_type,
                amount,
            } => {
                if let (Some(i), Some(d)) = (yield_of(yield_type), rules.idx(district)) {
                    let mut y = [0.0; 6];
                    y[i] = *amount;
                    self.district_flat.push((d, y));
                }
            }
            Effect::BuildingYield {
                building,
                yield_type,
                amount,
            } => {
                if let (Some(i), Some(b)) = (yield_of(yield_type), rules.idx(building)) {
                    let mut y = [0.0; 6];
                    y[i] = *amount;
                    self.building_flat.push((b, y));
                }
            }
            Effect::ImprovementYield {
                improvement,
                yield_type,
                amount,
            } => {
                if let (Some(i), Some(imp)) = (yield_of(yield_type), rules.idx(improvement)) {
                    let mut y = [0; 6];
                    y[i] = *amount;
                    self.improvement_flat.push((imp, y));
                }
            }
            Effect::UnitProduction {
                percent,
                class,
                unit,
                domain,
                era,
                military,
            } => self.unit_production.push(UnitProductionBonus {
                promotion_class: class.clone(),
                unit: unit.clone(),
                domain: domain.clone(),
                era: era.clone(),
                military_only: *military,
                percent: *percent,
            }),
            Effect::BuildingProduction { percent } => self.building_production += percent,
            Effect::WonderProduction { percent } => self.wonder_production += percent,
            Effect::DistrictProduction { percent } => self.district_production += percent,
            Effect::GreatPersonPoints { class, amount } => {
                if let Some(c) = rules.idx(class) {
                    self.great_person_points.push((c, *amount));
                }
            }
            Effect::Housing { amount } => self.housing += amount,
            Effect::Amenities { amount } => self.amenities += amount,
            Effect::Combat { amount, class } => match class {
                Some(c) => self.class_combat.push((c.clone(), *amount)),
                None => self.combat += amount,
            },
            Effect::CombatInTerritory { amount } => self.combat_in_territory += amount,
            Effect::UnitXp { amount } => self.unit_xp += amount,
            Effect::AdjacencyPercent { percent } => self.adjacency_multiplier += percent / 100.0,
            Effect::PurchaseDiscount { percent } => {
                self.purchase_multiplier *= 1.0 - percent / 100.0
            }
            Effect::TradeYield {
                routes,
                yield_type,
                amount,
            } => {
                if let Some(i) = yield_of(yield_type) {
                    use wayciv_data::effect::Routes;
                    if *routes != Routes::International {
                        self.trade_domestic[i] += amount;
                    }
                    if *routes != Routes::Domestic {
                        self.trade_international[i] += amount;
                    }
                }
            }
            Effect::TradeCapacity { amount } => self.trade_capacity += amount,
            Effect::PolicyYield {
                slot,
                yield_type,
                amount,
            } => {
                let count = slotted
                    .iter()
                    .find(|(s, _)| s == slot)
                    .map(|(_, n)| *n)
                    .unwrap_or(0);
                if let Some(i) = yield_of(yield_type) {
                    self.city_flat[i] += amount * count as f32;
                }
            }
        }
    }

    /// Combat Strength bonus for a unit of `class`.
    pub fn combat_for(&self, class: Option<&str>) -> i32 {
        self.combat
            + self
                .class_combat
                .iter()
                .filter(|(c, _)| Some(c.as_str()) == class)
                .map(|(_, n)| n)
                .sum::<i32>()
    }

    /// Extra tile yields for an improvement.
    pub fn improvement_yields(&self, improvement: usize) -> [i32; 6] {
        let mut y = [0; 6];
        for (imp, extra) in &self.improvement_flat {
            if *imp == improvement {
                for i in 0..6 {
                    y[i] += extra[i];
                }
            }
        }
        y
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
        let mut effects: Vec<&Effect> = Vec::new();
        if let Some(g) = p.government {
            effects.extend(&content.governments[g].effects);
        }
        for &i in &p.policies {
            effects.extend(&content.policies[i].effects);
        }
        for &i in p.beliefs.iter().chain(p.pantheon.iter()) {
            effects.extend(&content.beliefs[i].effects);
        }
        for t in content.leaders[p.leader]
            .traits
            .iter()
            .chain(&content.civilizations[p.civ].traits)
        {
            effects.extend(&t.effects);
        }
        for c in self.player_cities(player) {
            for &b in &c.buildings {
                effects.extend(&content.buildings[b].effects);
            }
        }
        let mut slotted: Vec<(String, usize)> = Vec::new();
        for &i in &p.policies {
            let slot = &content.policies[i].slot;
            match slotted.iter_mut().find(|(s, _)| s == slot) {
                Some(entry) => entry.1 += 1,
                None => slotted.push((slot.clone(), 1)),
            }
        }
        for effect in effects {
            bonuses.apply(rules, effect, &slotted);
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

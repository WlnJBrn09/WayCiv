//! Artificial intelligence: Datacenters produce Compute, Compute researches
//! the AI tree, and AI tree items give large Science (and other) bonuses.

use crate::additions::*;
use crate::effects::Bonuses;
use crate::map::PlayerId;
use crate::research::Tree;
use crate::rules::{CULTURE, PRODUCTION, SCIENCE};
use crate::state::{City, Game};

/// Percent bonuses from each AI tree item: (id, yield, percent).
const AI_YIELD_PERCENT: [(&str, usize, f32); 10] = [
    (AI_MACHINE_LEARNING, SCIENCE, 15.0),
    (AI_DEEP_LEARNING, SCIENCE, 20.0),
    (AI_ROBOTIC_AUTOMATION, PRODUCTION, 20.0),
    (AI_LARGE_LANGUAGE_MODELS, SCIENCE, 25.0),
    (AI_LARGE_LANGUAGE_MODELS, CULTURE, 15.0),
    (AI_RESEARCH_AGENTS, SCIENCE, 15.0),
    (AI_AUTOMATED_SCIENCE, SCIENCE, 25.0),
    (AI_SUPERINTELLIGENCE, SCIENCE, 50.0),
    (AI_SUPERINTELLIGENCE, PRODUCTION, 20.0),
    ("", 0, 0.0),
];

impl Game {
    pub fn is_datacenter(&self, district: usize) -> bool {
        let d = &self.rules.content.districts[district];
        d.id == DATACENTER || d.replaces.as_deref() == Some(DATACENTER)
    }

    /// Compute a city's Datacenter produces per turn (0 without one).
    pub fn city_compute(&self, city: &City) -> f32 {
        let Some(dc) = city.districts.iter().find(|d| self.is_datacenter(d.district)) else { return 0.0 };
        let map = &self.map;
        let adjacent_districts = map.grid.neighbors(dc.pos).filter(|&n| map.tile(n).district.is_some()).count() as f32;
        let cooled = map.tile(dc.pos).river || map.grid.neighbors(dc.pos).any(|n| map.tile(n).river || map.is_water(&self.rules, n));
        let has = |id: &str| self.rules.idx(id).is_some_and(|b| city.has_building(b));
        let mut compute = 6.0 + 2.0 * adjacent_districts;
        if cooled {
            compute += 3.0;
        }
        if self.has_ai_tech(city.owner, AI_NEURAL_NETWORKS) {
            compute += 2.0;
        }
        if has(GPU_CLUSTER) {
            compute += 8.0;
        }
        if has(SOLAR_FARM) {
            compute += 4.0;
        }
        compute
    }

    /// Compute per turn across the empire.
    pub fn compute(&self, player: PlayerId) -> f32 {
        self.player_cities(player).map(|c| self.city_compute(c)).sum()
    }

    /// AI tree items that unlock something: units and buildings need them
    /// on top of their tech.
    pub(crate) fn ai_requirement(&self, id: &str) -> Option<&'static str> {
        match id {
            STRIKE_DRONE => Some(AI_AUTONOMOUS_SYSTEMS),
            GPU_CLUSTER => Some(AI_DEEP_LEARNING),
            _ => None,
        }
    }

    pub(crate) fn ai_requirement_met(&self, player: PlayerId, id: &str) -> bool {
        self.ai_requirement(id).is_none_or(|req| self.has_ai_tech(player, req))
    }

    pub fn is_drone(&self, unit: &crate::state::Unit) -> bool {
        self.unit_type(unit).id == STRIKE_DRONE
    }

    /// Adds the AI tree's bonuses to a player's bonuses.
    pub(crate) fn ai_tree_bonuses(&self, player: PlayerId, bonuses: &mut Bonuses) {
        for (id, yield_index, percent) in AI_YIELD_PERCENT {
            if !id.is_empty() && self.has_ai_tech(player, id) {
                bonuses.city_percent[yield_index] += percent;
            }
        }
        if self.has_ai_tech(player, AI_COMPUTER_VISION) {
            bonuses.combat += 3;
        }
        if self.has_ai_tech(player, AI_NEURAL_NETWORKS)
            && let Some(campus) = self.rules.idx("DISTRICT_CAMPUS")
        {
            let mut y = [0.0; 6];
            y[SCIENCE] = 3.0;
            bonuses.district_flat.push((campus, y));
        }
    }

    /// Space race projects cost 30% less with Automated Science.
    pub(crate) fn project_cost_multiplier(&self, player: PlayerId) -> f32 {
        if self.has_ai_tech(player, AI_AUTOMATED_SCIENCE) { 0.7 } else { 1.0 }
    }

    /// End of turn: Compute goes into the AI tree; Research Agents hand out
    /// a Eureka every five turns.
    pub(crate) fn process_ai(&mut self, player: PlayerId) {
        if !self.ai_tree_unlocked(player) {
            return;
        }
        let compute = self.compute(player);
        self.advance(player, Tree::Ai, compute);
        if self.has_ai_tech(player, AI_RESEARCH_AGENTS) && self.turn.is_multiple_of(5) {
            let p = &self.players[player];
            let target = (0..self.rules.content.techs.len())
                .filter(|&i| !p.techs[i] && !p.tech_boosted[i])
                .min_by_key(|&i| (!self.researchable(player, Tree::Tech, i), self.rules.content.techs[i].cost));
            if let Some(i) = target {
                self.boost(player, Tree::Tech, i, 50);
            }
        }
    }
}

//! Technologies, civics, Eurekas and Inspirations, governments, and policies.

use crate::map::PlayerId;
use crate::state::Game;

/// Events that can complete a boost; `Poll` re-checks state-based triggers.
#[derive(Clone, Copy, Debug)]
pub enum Trigger {
    Poll,
    Trained(usize),
    Built(usize),
    District(usize),
    Killed { with: usize },
    MetCiv,
    Pantheon,
    Religion,
    SettledCoast,
    Improved(usize),
    DeclaredWarOn,
    ClearedCamp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tree {
    Tech,
    Civic,
    /// WayCiv's AI tree: unlocked by Artificial Intelligence, researched
    /// with Compute from Datacenters.
    Ai,
}

impl Game {
    pub fn tree_items(&self, tree: Tree) -> &[crate::rules::Research] {
        match tree {
            Tree::Tech => &self.rules.content.techs,
            Tree::Civic => &self.rules.content.civics,
            Tree::Ai => &self.rules.ai_tree,
        }
    }

    pub fn known(&self, player: PlayerId, tree: Tree, i: usize) -> bool {
        let p = &self.players[player];
        match tree {
            Tree::Tech => p.techs[i],
            Tree::Civic => p.civics[i],
            Tree::Ai => p.ai_techs[i],
        }
    }

    pub fn has_ai_tech(&self, player: PlayerId, id: &str) -> bool {
        self.rules
            .ai_tech(id)
            .is_some_and(|i| self.players[player].ai_techs[i])
    }

    /// Whether the player has discovered Artificial Intelligence, which
    /// opens the AI tree.
    pub fn ai_tree_unlocked(&self, player: PlayerId) -> bool {
        self.players[player].has_tech(&self.rules, crate::additions::TECH_AI)
    }

    /// Whether every prerequisite of item `i` is known and it isn't yet.
    pub fn researchable(&self, player: PlayerId, tree: Tree, i: usize) -> bool {
        if tree == Tree::Ai && !self.ai_tree_unlocked(player) {
            return false;
        }
        let items = self.tree_items(tree);
        !self.known(player, tree, i)
            && items[i].prereqs.iter().all(|id| {
                items
                    .iter()
                    .position(|x| &x.id == id)
                    .is_some_and(|j| self.known(player, tree, j))
            })
    }

    /// WayCiv charges 60% of Civ VI's costs, so a 300-turn game reaches the
    /// space race despite the systems WayCiv simplifies.
    pub fn research_cost(&self, tree: Tree, i: usize) -> i32 {
        match tree {
            Tree::Ai => self.tree_items(tree)[i].cost,
            _ => self.tree_items(tree)[i].cost * 3 / 5,
        }
    }

    pub fn current_research(&self, player: PlayerId, tree: Tree) -> Option<usize> {
        let p = &self.players[player];
        match tree {
            Tree::Tech => p.research,
            Tree::Civic => p.civic,
            Tree::Ai => p.ai_research,
        }
    }

    fn current_mut(&mut self, player: PlayerId, tree: Tree) -> &mut Option<usize> {
        let p = &mut self.players[player];
        match tree {
            Tree::Tech => &mut p.research,
            Tree::Civic => &mut p.civic,
            Tree::Ai => &mut p.ai_research,
        }
    }

    fn progress_mut(&mut self, player: PlayerId, tree: Tree, i: usize) -> &mut i32 {
        let p = &mut self.players[player];
        match tree {
            Tree::Tech => &mut p.tech_progress[i],
            Tree::Civic => &mut p.civic_progress[i],
            Tree::Ai => &mut p.ai_progress[i],
        }
    }

    pub fn set_research(&mut self, player: PlayerId, tree: Tree, i: usize) -> Result<(), String> {
        if !self.researchable(player, tree, i) {
            return Err("Not available yet.".into());
        }
        *self.current_mut(player, tree) = Some(i);
        Ok(())
    }

    pub fn progress(&self, player: PlayerId, tree: Tree, i: usize) -> i32 {
        let p = &self.players[player];
        match tree {
            Tree::Tech => p.tech_progress[i],
            Tree::Civic => p.civic_progress[i],
            Tree::Ai => p.ai_progress[i],
        }
    }

    /// Adds science, culture, or compute to the current research; completes it.
    pub(crate) fn advance(&mut self, player: PlayerId, tree: Tree, amount: f32) {
        let Some(i) = self.current_research(player, tree) else {
            return;
        };
        let cost = self.research_cost(tree, i);
        let progress = self.progress_mut(player, tree, i);
        *progress += amount.round() as i32;
        if *progress >= cost {
            let overflow = *progress - cost;
            self.complete(player, tree, i);
            // Overflow goes to the next item chosen; keep it on the same tree.
            *self.current_mut(player, tree) = None;
            if let Some(next) = self
                .ai_pick_research(player, tree)
                .filter(|_| !self.players[player].human)
            {
                let _ = self.set_research(player, tree, next);
            }
            if let Some(next) = self.current_research(player, tree) {
                *self.progress_mut(player, tree, next) += overflow;
            }
        }
    }

    pub(crate) fn complete(&mut self, player: PlayerId, tree: Tree, i: usize) {
        let name = self.tree_items(tree)[i].name.clone();
        let p = &mut self.players[player];
        match tree {
            Tree::Tech => p.techs[i] = true,
            Tree::Civic => {
                p.civics[i] = true;
                p.policy_change_free = true;
            }
            Tree::Ai => p.ai_techs[i] = true,
        }
        let what = match tree {
            Tree::Tech => "Technology researched",
            Tree::Civic => "Civic completed",
            Tree::Ai => "AI breakthrough",
        };
        self.notify(player, format!("{what}: {name}."), None);
        if tree == Tree::Tech && self.rules.content.techs[i].id == crate::additions::TECH_AI {
            self.notify(
                player,
                "The AI tree is open. Build Datacenters to produce Compute for it.",
                None,
            );
        }
        if tree == Tree::Civic {
            // Civics that unlock a government or more policies prompt a review.
            let civic_id = self.rules.content.civics[i].id.clone();
            if self
                .rules
                .content
                .governments
                .iter()
                .any(|g| g.prereq_civic.as_deref() == Some(civic_id.as_str()))
            {
                self.notify(player, "A new government is available.", None);
            }
            self.players[player].envoys_available +=
                i32::from(self.rules.content.civics[i].era != "ERA_ANCIENT" || i.is_multiple_of(2));
        }
        self.check_boosts(player, Trigger::Poll);
    }

    /// Grants a boost (Eureka/Inspiration) to `tree` item `i`. AI tree items
    /// have no boost flag: each boost just adds progress.
    pub fn boost(&mut self, player: PlayerId, tree: Tree, i: usize, percent: i32) {
        let cost = self.research_cost(tree, i);
        let name = self.tree_items(tree)[i].name.clone();
        let p = &mut self.players[player];
        let boosted = match tree {
            Tree::Tech => Some(&mut p.tech_boosted[i]),
            Tree::Civic => Some(&mut p.civic_boosted[i]),
            Tree::Ai => None,
        };
        if let Some(boosted) = boosted {
            if *boosted {
                return;
            }
            *boosted = true;
        }
        *self.progress_mut(player, tree, i) += cost * percent / 100;
        let kind = match tree {
            Tree::Tech => "Eureka",
            Tree::Civic => "Inspiration",
            Tree::Ai => "AI boost",
        };
        self.notify(player, format!("{kind}! {name} is boosted."), None);
    }

    /// Checks boosts whose trigger matches `event`, or every state-based
    /// trigger for `Trigger::Poll`.
    pub(crate) fn check_boosts(&mut self, player: PlayerId, event: Trigger) {
        if !self.players[player].is_major() {
            return;
        }
        let rules = self.rules.clone();
        let content = &rules.content;
        for boost in &content.boosts {
            let (tree, i) = match (&boost.tech, &boost.civic) {
                (Some(t), _) => match rules.tech(t) {
                    Some(i) => (Tree::Tech, i),
                    None => continue,
                },
                (_, Some(c)) => match rules.civic(c) {
                    Some(i) => (Tree::Civic, i),
                    None => continue,
                },
                _ => continue,
            };
            let p = &self.players[player];
            let already = match tree {
                Tree::Tech => p.tech_boosted[i] || p.techs[i],
                Tree::Civic => p.civic_boosted[i] || p.civics[i],
                Tree::Ai => true,
            };
            if already || !self.boost_met(player, boost, event) {
                continue;
            }
            self.boost(player, tree, i, boost.percent.max(40));
        }
    }

    fn boost_met(&self, player: PlayerId, b: &civ6_data::content::Boost, event: Trigger) -> bool {
        let rules = &self.rules;
        let content = &rules.content;
        let is = |id: &Option<String>, index: usize, list: &str| -> bool {
            let Some(id) = id else { return false };
            let actual = match list {
                "unit" => &content.units[index].id,
                "building" => &content.buildings[index].id,
                _ => &content.districts[index].id,
            };
            let replaces = match list {
                "unit" => content.units[index].replaces.as_ref(),
                "building" => content.buildings[index].replaces.as_ref(),
                _ => content.districts[index].replaces.as_ref(),
            };
            actual == id || replaces == Some(id)
        };
        let count = b.count.max(1) as usize;
        let cities: Vec<_> = self.player_cities(player).collect();
        match (b.class.as_str(), event) {
            ("BOOST_TRIGGER_TRAIN_UNIT", Trigger::Trained(u)) => is(&b.unit, u, "unit"),
            ("BOOST_TRIGGER_CONSTRUCT_BUILDING", Trigger::Built(x)) => {
                is(&b.building, x, "building")
            }
            ("BOOST_TRIGGER_KILL_WITH", Trigger::Killed { with }) => {
                b.unit.is_none() || is(&b.unit, with, "unit")
            }
            ("BOOST_TRIGGER_MEET_CIV", Trigger::MetCiv) => true,
            ("BOOST_TRIGGER_CLEAR_CAMP", Trigger::ClearedCamp) => true,
            ("BOOST_TRIGGER_NUM_BARBS_KILLED", _) => {
                self.players[player].barbarians_killed as usize >= count
            }
            ("BOOST_TRIGGER_MAINTAIN_X_TRADE_ROUTES", _) => {
                self.trade_routes_of(player).count() >= count
            }
            ("BOOST_TRIGGER_CREATE_PANTHEON", Trigger::Pantheon) => true,
            ("BOOST_TRIGGER_FOUND_RELIGION", Trigger::Religion) => true,
            ("BOOST_TRIGGER_SETTLE_COAST", Trigger::SettledCoast) => true,
            (
                "BOOST_TRIGGER_RECEIVE_DOW" | "BOOST_TRIGGER_DOW_CASUS_BELLI",
                Trigger::DeclaredWarOn,
            ) => true,
            ("BOOST_TRIGGER_HAVE_X_BUILDINGS", _) => {
                cities
                    .iter()
                    .filter(|c| c.buildings.iter().any(|&x| is(&b.building, x, "building")))
                    .count()
                    >= count
            }
            ("BOOST_TRIGGER_HAVE_X_DISTRICTS", _) => {
                cities
                    .iter()
                    .flat_map(|c| &c.districts)
                    .filter(|d| is(&b.district, d.district, "district"))
                    .count()
                    >= count
            }
            ("BOOST_TRIGGER_HAVE_X_UNIQUE_SPECIALTY_DISTRICTS", _) => {
                let mut kinds: Vec<usize> = cities
                    .iter()
                    .flat_map(|c| &c.districts)
                    .filter(|d| content.districts[d.district].requires_population)
                    .map(|d| d.district)
                    .collect();
                kinds.sort();
                kinds.dedup();
                kinds.len() >= count
            }
            ("BOOST_TRIGGER_OWN_X_UNITS_OF_TYPE", _) => {
                self.player_units(player)
                    .filter(|u| is(&b.unit, u.kind, "unit"))
                    .count()
                    >= count
            }
            ("BOOST_TRIGGER_HAVE_X_LAND_UNITS", _) => {
                self.player_units(player)
                    .filter(|u| self.is_military(u) && self.unit_type(u).domain == "DOMAIN_LAND")
                    .count()
                    >= count
            }
            (
                "BOOST_TRIGGER_HAVE_X_IMPROVEMENTS"
                | "BOOST_TRIGGER_IMPROVE_SPECIFIC_RESOURCE"
                | "BOOST_TRIGGER_HAVE_UNIT_AND_IMPROVEMENT",
                _,
            ) => {
                self.map
                    .tiles
                    .iter()
                    .filter(|t| {
                        t.owner == Some(player)
                            && t.improvement.is_some_and(|i| {
                                b.improvement.as_ref() == Some(&content.improvements[i].id)
                            })
                    })
                    .filter(|t| {
                        b.resource.as_ref().is_none_or(|r| {
                            t.resource.is_some_and(|x| &content.resources[x].id == r)
                        })
                    })
                    .count()
                    >= count
            }
            ("BOOST_TRIGGER_NUM_IMPROVED_TILES", _) => {
                self.map
                    .tiles
                    .iter()
                    .filter(|t| t.owner == Some(player) && t.improvement.is_some())
                    .count()
                    >= count
            }
            ("BOOST_TRIGGER_CITY_POPULATION", _) => {
                cities.iter().any(|c| c.population as usize >= count)
            }
            ("BOOST_TRIGGER_EMPIRE_POPULATION", _) => {
                cities.iter().map(|c| c.population as usize).sum::<usize>() >= count
            }
            ("BOOST_TRIGGER_HAVE_X_WONDERS", _) => {
                cities
                    .iter()
                    .flat_map(|c| &c.buildings)
                    .filter(|&&x| content.buildings[x].wonder)
                    .count()
                    >= count
            }
            ("BOOST_TRIGGER_MEET_X_CITY_STATES", _) => {
                self.players[player]
                    .relations
                    .iter()
                    .enumerate()
                    .filter(|(i, r)| r.met && self.players[*i].is_city_state())
                    .count()
                    >= count
            }
            ("BOOST_TRIGGER_RESEARCH_TECH", _) => b
                .boosting_tech
                .as_deref()
                .is_some_and(|t| self.players[player].has_tech(rules, t)),
            ("BOOST_TRIGGER_CULTURVATE_CIVIC", _) => b
                .boosting_civic
                .as_deref()
                .is_some_and(|c| self.players[player].has_civic(rules, c)),
            ("BOOST_TRIGGER_HAVE_X_POLICY_SLOTS", _) => {
                self.policy_slots(player)
                    .iter()
                    .map(|(_, n)| *n)
                    .sum::<i32>() as usize
                    >= count
            }
            ("BOOST_TRIGGER_HAVE_X_GREAT_PEOPLE", _) => {
                self.great_people_taken.len() >= count
                    && self.player_units(player).any(|u| u.great_person.is_some())
            }
            _ => false,
        }
    }

    /// AI's (and auto-pick's) choice of next research: cheapest available,
    /// with a little randomness for variety.
    pub fn ai_pick_research(&self, player: PlayerId, tree: Tree) -> Option<usize> {
        let items = self.tree_items(tree);
        let p = &self.players[player];
        let boosted = |i: usize| match tree {
            Tree::Tech => p.tech_boosted[i],
            Tree::Civic => p.civic_boosted[i],
            Tree::Ai => false,
        };
        (0..items.len())
            .filter(|&i| self.researchable(player, tree, i))
            .min_by_key(|&i| {
                let jitter = ((player * 31 + i * 17) % 7) as i32;
                items[i].cost * 10 - if boosted(i) { items[i].cost * 5 } else { 0 }
                    + jitter * items[i].cost / 4
            })
    }

    /// Policy slots of the current government (`SLOT_*` -> count). Without a
    /// government, Civ VI's Chiefdom-like start gives one military and one
    /// economic slot once Code of Laws is known.
    pub fn policy_slots(&self, player: PlayerId) -> Vec<(String, i32)> {
        let p = &self.players[player];
        match p.government {
            Some(g) => self.rules.content.governments[g].slots.clone(),
            None => Vec::new(),
        }
    }

    pub fn available_governments(&self, player: PlayerId) -> Vec<usize> {
        let p = &self.players[player];
        (0..self.rules.content.governments.len())
            .filter(|&g| {
                p.meets(
                    &self.rules,
                    None,
                    self.rules.content.governments[g].prereq_civic.as_deref(),
                )
            })
            .collect()
    }

    pub fn available_policies(&self, player: PlayerId) -> Vec<usize> {
        let p = &self.players[player];
        let content = &self.rules.content;
        (0..content.policies.len())
            .filter(|&i| {
                let policy = &content.policies[i];
                p.meets(&self.rules, None, policy.prereq_civic.as_deref())
                    && !policy.obsoleted_by.as_deref().is_some_and(|o| {
                        content
                            .policies
                            .iter()
                            .position(|x| x.id == o)
                            .is_some_and(|j| {
                                p.meets(
                                    &self.rules,
                                    None,
                                    content.policies[j].prereq_civic.as_deref(),
                                )
                            })
                    })
            })
            .collect()
    }

    pub fn set_government(&mut self, player: PlayerId, government: usize) -> Result<(), String> {
        if !self.available_governments(player).contains(&government) {
            return Err("That government isn't available.".into());
        }
        let p = &mut self.players[player];
        if p.government == Some(government) {
            return Ok(());
        }
        p.government = Some(government);
        p.policy_change_free = true;
        self.fit_policies(player);
        let name = self.rules.content.governments[government].name.clone();
        self.notify(player, format!("Adopted {name}."), None);
        Ok(())
    }

    /// Drops policies that no longer fit the government's slots.
    fn fit_policies(&mut self, player: PlayerId) {
        let slots = self.policy_slots(player);
        let content = &self.rules.content;
        let mut used: Vec<(String, i32)> = slots.iter().map(|(s, _)| (s.clone(), 0)).collect();
        let mut kept = Vec::new();
        for &policy in &self.players[player].policies {
            if let Some(slot) = self.slot_for(&content.policies[policy].slot, &slots, &used) {
                used[slot].1 += 1;
                kept.push(policy);
            }
        }
        self.players[player].policies = kept;
    }

    /// Which slot a policy of `kind` fits: its own kind, or a wildcard slot.
    fn slot_for(
        &self,
        kind: &str,
        slots: &[(String, i32)],
        used: &[(String, i32)],
    ) -> Option<usize> {
        let fits = |slot: &str| slot == kind || slot == "SLOT_WILDCARD";
        slots
            .iter()
            .enumerate()
            .filter(|(_, (slot, _))| slot == kind)
            .chain(
                slots
                    .iter()
                    .enumerate()
                    .filter(|(_, (slot, _))| slot == "SLOT_WILDCARD"),
            )
            .find(|(i, (slot, n))| fits(slot) && used[*i].1 < *n)
            .map(|(i, _)| i)
    }

    /// Replaces the player's policies; validates slot counts. Changing
    /// policies is free right after a civic completes; otherwise it costs gold.
    pub fn set_policies(&mut self, player: PlayerId, policies: Vec<usize>) -> Result<(), String> {
        let available = self.available_policies(player);
        if policies.iter().any(|p| !available.contains(p)) {
            return Err("A chosen policy isn't available.".into());
        }
        let slots = self.policy_slots(player);
        let content = &self.rules.content;
        let mut used: Vec<(String, i32)> = slots.iter().map(|(s, _)| (s.clone(), 0)).collect();
        for &policy in &policies {
            let slot = self
                .slot_for(&content.policies[policy].slot, &slots, &used)
                .ok_or("Not enough policy slots.")?;
            used[slot].1 += 1;
        }
        let changed = policies
            .iter()
            .filter(|p| !self.players[player].policies.contains(p))
            .count() as i32;
        let cost = if self.players[player].policy_change_free {
            0
        } else {
            changed * 25
        };
        if self.players[player].gold < cost {
            return Err(format!("Changing policies now costs {cost} Gold."));
        }
        let p = &mut self.players[player];
        p.gold -= cost;
        p.policies = policies;
        Ok(())
    }
}

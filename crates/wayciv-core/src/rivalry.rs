//! Rivalry between civilizations: leader personalities, agendas,
//! grievances, war plans, demands for tribute, alliances, joint wars, and
//! peace terms. The AI's side of each is driven from `ai_rivalry`.

use crate::diplomacy::DiploEvent;
use crate::map::PlayerId;
use crate::state::{Game, WarPlan};
use wayciv_data::content::LeaderAi;

/// Grievances for each kind of offense.
pub mod grievance {
    pub const SURPRISE_WAR: i32 = 50;
    pub const FORMAL_WAR: i32 = 25;
    pub const WAR_ON_SOMEONE: i32 = 6;
    pub const CAPTURED_CITY: i32 = 30;
    pub const DENOUNCED: i32 = 10;
    pub const REFUSED_DEMAND: i32 = 20;
    pub const SETTLED_CLOSE: i32 = 15;
    pub const BROKE_ALLIANCE: i32 = 30;
}

/// War score for each blow in a war.
pub const WAR_SCORE_KILL: i32 = 5;
pub const WAR_SCORE_CITY: i32 = 30;

impl Game {
    pub fn personality(&self, player: PlayerId) -> &LeaderAi {
        &self.rules.content.leaders[self.players[player].leader].ai
    }

    fn aggression(&self, player: PlayerId) -> i32 {
        self.personality(player).aggression.clamp(0, 3)
    }

    /// A duration in turns, scaled to the game's length (balanced for 300).
    pub(crate) fn turns(&self, base: u32) -> u32 {
        self.scaled(base as i32) as u32
    }

    pub(crate) fn add_grievance(&mut self, holder: PlayerId, against: PlayerId, amount: i32) {
        if holder == against || !self.players[holder].is_major() {
            return;
        }
        let r = &mut self.players[holder].relations[against];
        r.grievances = (r.grievances + amount).clamp(0, 200);
    }

    /// Each turn grievances fade a little between civilizations at peace.
    pub(crate) fn decay_grievances(&mut self) {
        for p in &mut self.players {
            for r in &mut p.relations {
                if !r.at_war && r.grievances > 0 {
                    r.grievances -= 1;
                }
            }
        }
    }

    pub fn allied(&self, a: PlayerId, b: PlayerId) -> bool {
        self.players[a].relations[b].alliance
    }

    /// What `of`'s agenda makes of `about`: an opinion change and the reason.
    pub fn agenda_opinion(&self, of: PlayerId, about: PlayerId) -> (i32, Option<String>) {
        let rule = self.personality(of).agenda.clone();
        let content = &self.rules.content;
        let majors: Vec<PlayerId> = self.alive_majors().map(|p| p.id).collect();
        let n = majors.len().max(1) as f32;
        // How `about` compares with the world average of `metric`, as -1..1.
        let compare = |metric: &dyn Fn(PlayerId) -> f32| -> f32 {
            let avg = majors.iter().map(|&p| metric(p)).sum::<f32>() / n;
            let mine = metric(about);
            if avg <= 0.0 {
                return if mine > 0.0 { 1.0 } else { 0.0 };
            }
            ((mine - avg) / avg).clamp(-1.0, 1.0)
        };
        let scaled = |x: f32, good: &str, bad: &str| -> (i32, Option<String>) {
            let v = (x * 20.0).round() as i32;
            if v >= 4 {
                (v, Some(good.to_string()))
            } else if v <= -4 {
                (v, Some(bad.to_string()))
            } else {
                (0, None)
            }
        };
        let near = |d: i32| {
            self.player_cities(of).any(|a| {
                self.player_cities(about)
                    .any(|b| self.map.grid.distance(a.pos, b.pos) <= d)
            })
        };
        let gov = self.players[about]
            .government
            .map(|g| content.governments[g].id.as_str());
        match rule.as_str() {
            "democracy" => match gov {
                Some("GOVERNMENT_DEMOCRACY") => (20, Some("Likes your Democracy".into())),
                Some("GOVERNMENT_COMMUNISM" | "GOVERNMENT_FASCISM") => {
                    (-20, Some("Dislikes your government".into()))
                }
                _ => (0, None),
            },
            "military" => scaled(
                compare(&|p| self.military_strength(p) as f32),
                "Respects your army",
                "Scorns your weak army",
            ),
            "navy" => scaled(
                compare(&|p| self.naval_strength(p) as f32),
                "Respects your navy",
                "Scorns your weak fleet",
            ),
            "wonders" => scaled(
                compare(&|p| self.wonder_count(p) as f32),
                "Admires your wonders",
                "Thinks you have too few wonders",
            ),
            "great_people" => scaled(
                compare(&|p| self.players[p].great_people_earned as f32),
                "Admires your great people",
                "Thinks you neglect great people",
            ),
            "culture" => scaled(
                compare(&|p| self.players[p].culture_total as f32),
                "Admires your culture",
                "Thinks your culture is poor",
            ),
            "science" => scaled(
                compare(&|p| self.players[p].techs.iter().filter(|t| **t).count() as f32),
                "Admires your learning",
                "Thinks you are backward",
            ),
            "cities" => scaled(
                compare(&|p| self.player_cities(p).count() as f32),
                "Admires your large empire",
                "Thinks your empire is small",
            ),
            "districts" => scaled(
                compare(&|p| {
                    self.player_cities(p)
                        .map(|c| c.districts.len())
                        .sum::<usize>() as f32
                }),
                "Admires your districts",
                "Thinks you build too few districts",
            ),
            "trade" => {
                let routes = self.trade_routes.iter().any(|r| {
                    let dest = self.cities.get(&r.destination).map(|c| c.owner);
                    (r.owner == about && dest == Some(of)) || (r.owner == of && dest == Some(about))
                });
                if routes {
                    (15, Some("Likes trading with you".into()))
                } else {
                    (-5, Some("Wants you to trade with them".into()))
                }
            }
            "religion" => {
                let theirs = self.dominant_religion(about);
                let mine = self.players[of].religion.or(self.dominant_religion(of));
                match (mine, theirs) {
                    (Some(a), Some(b)) if a == b => (20, Some("Shares your faith".into())),
                    (Some(_), Some(_)) => (-15, Some("Follows another faith".into())),
                    _ => (0, None),
                }
            }
            "warlike" => {
                if self.real_wars(about) > 0 {
                    (15, Some("Admires your wars".into()))
                } else {
                    (-8, Some("Thinks you are soft".into()))
                }
            }
            "peaceful" => {
                let conquered = self
                    .player_cities(about)
                    .filter(|c| c.founder != about && self.players[c.founder].is_major())
                    .count() as i32;
                if conquered > 0 {
                    (-10 * conquered.min(3), Some("Dislikes your conquests".into()))
                } else {
                    (8, Some("Likes that you leave others free".into()))
                }
            }
            "space" => {
                if near(6) {
                    (-20, Some("Hates that you settled near them".into()))
                } else {
                    (5, Some("Likes that you keep your distance".into()))
                }
            }
            "happiness" => {
                let unhappy = self
                    .player_cities(about)
                    .filter(|c| {
                        let (have, need) = self.amenities(c);
                        have < need
                    })
                    .count();
                if unhappy > 0 {
                    (-12, Some("Dislikes your unhappy cities".into()))
                } else {
                    (8, Some("Likes your content people".into()))
                }
            }
            "luxuries" => {
                let mine = self.luxuries(of);
                let lacking = self
                    .luxuries(about)
                    .into_iter()
                    .filter(|l| !mine.contains(l))
                    .count();
                if lacking >= 2 {
                    (-12, Some("Covets your luxuries".into()))
                } else {
                    (0, None)
                }
            }
            "loyal" => {
                let fights_friend = (0..self.players.len()).any(|f| {
                    self.players[of].relations[f].friendship && self.players[about].at_war_with(f)
                });
                if fights_friend {
                    (-25, Some("Hates that you attack their friends".into()))
                } else if self.players[of].relations[about].friendship {
                    (12, Some("Values your friendship".into()))
                } else {
                    (0, None)
                }
            }
            "weak_prey" => {
                let (mine, theirs) = (self.military_strength(of), self.military_strength(about));
                if theirs * 3 < mine * 2 && near(12) {
                    (-15, Some("Sees you as weak".into()))
                } else if theirs > mine {
                    (10, Some("Respects your strength".into()))
                } else {
                    (0, None)
                }
            }
            _ => (0, None),
        }
    }

    pub fn naval_strength(&self, player: PlayerId) -> i32 {
        self.player_units(player)
            .filter(|u| self.unit_type(u).domain == "DOMAIN_SEA" && self.is_military(u))
            .map(|u| self.unit_type(u).combat.max(self.unit_type(u).ranged_combat))
            .sum()
    }

    pub fn wonder_count(&self, player: PlayerId) -> usize {
        self.player_cities(player)
            .flat_map(|c| &c.buildings)
            .filter(|&&b| self.rules.content.buildings[b].wonder)
            .count()
    }

    /// Whether `a` and `b` are close enough to fight over land.
    pub(crate) fn neighbors_within(&self, a: PlayerId, b: PlayerId, d: i32) -> bool {
        self.player_cities(a).any(|x| {
            self.player_cities(b)
                .any(|y| self.map.grid.distance(x.pos, y.pos) <= d)
        })
    }

    /// Strength of `a` against `b` in tenths (10 = even).
    fn strength_ratio(&self, a: PlayerId, b: PlayerId) -> i32 {
        let mine = self.military_strength(a);
        let theirs = self.military_strength(b).max(1);
        mine * 10 / theirs
    }

    /// The war score difference in `a`'s favor in its war with `b`.
    pub fn war_balance(&self, a: PlayerId, b: PlayerId) -> i32 {
        self.players[a].relations[b].war_score - self.players[b].relations[a].war_score
    }

    pub(crate) fn add_war_score(&mut self, by: PlayerId, against: PlayerId, amount: i32) {
        if self.players[by].is_major() && self.players[against].is_major() {
            self.players[by].relations[against].war_score += amount;
        }
    }

    // ---- The AI's rivalry each turn ----

    pub(crate) fn ai_rivalry(&mut self, player: PlayerId) {
        self.ai_plan_war(player);
        self.ai_execute_plan(player);
        self.ai_demand(player);
        self.ai_alliances(player);
    }

    /// Picks a victim and starts a war plan: denounce, build up, gather.
    fn ai_plan_war(&mut self, player: PlayerId) {
        if self.players[player].war_plan.is_some()
            || self.turn < self.turns(25)
            || self.major_wars(player) > 0
            || self.recent_peace(player) < self.turns(20)
        {
            return;
        }
        let aggression = self.aggression(player);
        let rule = self.personality(player).agenda.clone();
        let mut best: Option<(i32, PlayerId)> = None;
        for other in 0..self.players.len() {
            let o = &self.players[other];
            if other == player
                || !o.alive
                || !o.is_major()
                || !self.players[player].relations[other].met
                || self.players[player].relations[other].friendship
                || self.allied(player, other)
                || !self.can_declare_war(player, other)
                || !self.neighbors_within(player, other, 16)
                || self.players[other].cities_founded == 0
                || self.truce(player, other)
            {
                continue;
            }
            let ratio = self.strength_ratio(player, other);
            // Aggressive leaders attack at even odds; peaceful ones want 1.5x.
            if ratio < 15 - aggression * 2 {
                continue;
            }
            let opinion = self.opinion(player, other);
            if opinion > 5 + aggression * 10 {
                continue;
            }
            let grievances = self.players[player].relations[other].grievances;
            let mut score = -opinion + grievances / 2 + aggression * 10 + (ratio - 10) * 2;
            if rule == "weak_prey" && ratio >= 15 {
                score += 25;
            }
            if self.players[other].human {
                // A little extra attention for the human: Civ's AI loves a
                // rival it can see on the scoreboard.
                score += 5;
            }
            if best.is_none_or(|(s, _)| score > s) {
                best = Some((score, other));
            }
        }
        let Some((score, target)) = best else {
            return;
        };
        let base = [0.006, 0.013, 0.022, 0.035][aggression as usize];
        let chance = base * (1.0 + score.max(0) as f64 / 40.0) / self.speed() as f64;
        if !self.rng.chance(chance) {
            return;
        }
        let turn = self.turn;
        let lead = self.turns(5) + self.rng.below(self.turns(4).max(1));
        self.players[player].war_plan = Some(WarPlan {
            target,
            since: turn,
            declare_at: turn + lead,
        });
        // The warning: a denouncement first, as a formal war needs.
        if !self.players[player].relations[target].denounced {
            self.denounce(player, target);
            if self.players[target].human {
                self.popups.push((player, DiploEvent::DenounceFromAi));
            }
        }
    }

    fn ai_execute_plan(&mut self, player: PlayerId) {
        let Some(plan) = self.players[player].war_plan.clone() else {
            return;
        };
        let target = plan.target;
        let cancel = !self.players[target].alive
            || self.players[player].relations[target].friendship
            || self.allied(player, target)
            || self.players[player].at_war_with(target);
        if cancel {
            self.players[player].war_plan = None;
            return;
        }
        // The human notices troops massing near the border.
        if self.players[target].human && self.turn == plan.since + self.turns(3) {
            let civ = self.players[player].civ_name.clone();
            let near = self.player_cities(target).find(|c| {
                self.player_units(player).any(|u| {
                    self.is_military(u) && self.map.grid.distance(u.pos, c.pos) <= 8
                })
            });
            let pos = near.map(|c| c.pos);
            let text = match near {
                Some(c) => format!("{civ} troops are massing near {}.", c.name),
                None => format!("Your envoys warn that {civ} is arming for war."),
            };
            self.notify(target, text, pos);
        }
        if self.turn < plan.declare_at {
            return;
        }
        let aggression = self.aggression(player);
        if self.strength_ratio(player, target) >= 12 - aggression * 2 {
            self.players[player].war_plan = None;
            self.declare_war(player, target);
            if self.players[target].human {
                self.popups.push((player, DiploEvent::DeclareWarFromAi));
            }
            self.invite_to_war(player, target);
        } else if self.turn > plan.declare_at + self.turns(12) {
            self.players[player].war_plan = None;
        }
    }

    /// `aggressor` asks its friends to join its war on `target`.
    fn invite_to_war(&mut self, aggressor: PlayerId, target: PlayerId) {
        for f in 0..self.players.len() {
            if f == target
                || !self.players[f].alive
                || !self.players[aggressor].relations[f].friendship
                || !self.can_declare_war(f, target)
                || !self.players[f].relations[target].met
            {
                continue;
            }
            if self.players[f].human {
                self.popups
                    .push((aggressor, DiploEvent::JoinWar { against: target }));
            } else if self.opinion(f, target) < 0 && self.rng.chance(0.5) {
                self.declare_war(f, target);
                let (a, b, t) = (
                    self.players[f].civ_name.clone(),
                    self.players[aggressor].civ_name.clone(),
                    self.players[target].civ_name.clone(),
                );
                for p in 0..self.players.len() {
                    if self.players[p].is_major() && self.players[p].relations[f].met {
                        self.notify(p, format!("{a} joins {b}'s war on {t}."), None);
                    }
                }
            }
        }
    }

    /// Turns since this player's most recent peace treaty (or forever).
    pub fn recent_peace(&self, player: PlayerId) -> u32 {
        self.players[player]
            .relations
            .iter()
            .filter(|r| !r.at_war && r.made_peace > 0)
            .map(|r| self.turn.saturating_sub(r.made_peace))
            .min()
            .unwrap_or(u32::MAX)
    }

    /// The AI keeps a peace treaty for a while before attacking again.
    pub fn truce(&self, a: PlayerId, b: PlayerId) -> bool {
        let r = &self.players[a].relations[b];
        r.made_peace > 0 && self.turn < r.made_peace + self.turns(35)
    }

    /// Wars with other majors (not barbarians or city-states).
    pub fn major_wars(&self, player: PlayerId) -> usize {
        self.players[player]
            .relations
            .iter()
            .enumerate()
            .filter(|(o, r)| r.at_war && self.players[*o].is_major())
            .count()
    }

    /// A strong, unfriendly AI demands tribute from a weaker neighbor.
    fn ai_demand(&mut self, player: PlayerId) {
        let aggression = self.aggression(player);
        if aggression == 0
            || self.players[player].war_plan.is_some()
            || self.turn < self.turns(30)
            || self.turn < self.players[player].last_demand + self.turns(30)
            || !self.rng.chance(0.012 * aggression as f64 / self.speed() as f64)
        {
            return;
        }
        let target = (0..self.players.len()).find(|&o| {
            o != player
                && self.players[o].alive
                && self.players[o].is_major()
                && self.players[player].relations[o].met
                && !self.players[player].at_war_with(o)
                && !self.players[player].relations[o].friendship
                && self.players[o].gold >= 60
                && self.strength_ratio(player, o) >= 15
                && self.opinion(player, o) < 15
                && self.neighbors_within(player, o, 16)
        });
        let Some(target) = target else {
            return;
        };
        let gold = ((self.players[target].gold / 3).clamp(50, 400) / 10) * 10;
        self.players[player].last_demand = self.turn;
        if self.players[target].human {
            self.popups.push((player, DiploEvent::Demand { gold }));
        } else {
            let pays =
                self.strength_ratio(player, target) >= 20 || self.aggression(target) == 0;
            self.answer_demand(player, target, gold, pays);
        }
    }

    /// Tribute `demander` may ask of `payer` now (None while too soon).
    pub fn tribute_asked(&self, demander: PlayerId, payer: PlayerId) -> Option<i32> {
        let last = self.players[demander].last_demand;
        if last > 0 && self.turn < last + self.turns(30) {
            return None;
        }
        Some(((self.players[payer].gold / 3).clamp(50, 400) / 10) * 10)
    }

    /// The human demands tribute from an AI, which pays only when clearly
    /// outmatched (a peaceful leader gives in sooner).
    pub fn demand_from_ai(&mut self, demander: PlayerId, payer: PlayerId) -> Option<bool> {
        let gold = self.tribute_asked(demander, payer)?;
        self.players[demander].last_demand = self.turn;
        let needed = if self.aggression(payer) == 0 { 15 } else { 20 };
        let pays = self.strength_ratio(demander, payer) >= needed && self.players[payer].gold >= gold;
        self.answer_demand(demander, payer, gold, pays);
        Some(pays)
    }

    /// `payer` answers `demander`'s demand for `gold`.
    pub fn answer_demand(&mut self, demander: PlayerId, payer: PlayerId, gold: i32, pay: bool) {
        let (d, p) = (
            self.players[demander].civ_name.clone(),
            self.players[payer].civ_name.clone(),
        );
        if pay {
            let gold = gold.min(self.players[payer].gold);
            self.players[payer].gold -= gold;
            self.players[demander].gold += gold;
            self.players[demander].relations[payer].opinion += 10;
            self.notify(payer, format!("You paid {d} {gold} Gold in tribute."), None);
            self.notify(demander, format!("{p} paid you {gold} Gold in tribute."), None);
        } else {
            self.add_grievance(demander, payer, grievance::REFUSED_DEMAND);
            self.players[demander].relations[payer].opinion -= 10;
            self.notify(payer, format!("You refused {d}'s demand. They won't forget it."), None);
            // A refusal can be all the excuse a warmonger needs.
            if !self.players[demander].human
                && self.players[demander].war_plan.is_none()
                && self.major_wars(demander) == 0
                && self.rng.chance(0.15 * self.aggression(demander) as f64)
                && self.can_declare_war(demander, payer)
            {
                let turn = self.turn;
                self.players[demander].war_plan = Some(WarPlan {
                    target: payer,
                    since: turn,
                    declare_at: turn + self.turns(4),
                });
                if !self.players[demander].relations[payer].denounced {
                    self.denounce(demander, payer);
                    if self.players[payer].human {
                        self.popups.push((demander, DiploEvent::DenounceFromAi));
                    }
                }
            }
        }
    }

    /// Friends of long standing can ally once the Classical Era arrives.
    pub fn can_ally(&self, a: PlayerId, b: PlayerId) -> bool {
        a != b
            && self.players[a].is_major()
            && self.players[b].is_major()
            && self.players[a].relations[b].friendship
            && !self.players[a].relations[b].alliance
            && !self.players[a].at_war_with(b)
            && self.world_era >= 1
            && self.turn >= self.players[a].relations[b].friend_since + self.turns(10)
    }

    /// Asks `to` to ally with `from`. The human always accepts (the UI asks
    /// first); the AI accepts when it likes `from` enough.
    pub fn propose_alliance(&mut self, from: PlayerId, to: PlayerId) -> bool {
        if !self.can_ally(from, to) {
            return false;
        }
        let accept = self.players[to].human || self.opinion(to, from) >= 30;
        if accept {
            let turn = self.turn;
            for (x, y) in [(from, to), (to, from)] {
                let r = &mut self.players[x].relations[y];
                r.alliance = true;
                r.friend_since = turn;
            }
            let (a, b) = (
                self.players[from].civ_name.clone(),
                self.players[to].civ_name.clone(),
            );
            for p in 0..self.players.len() {
                if !self.players[p].is_major() {
                    continue;
                }
                if p == from || p == to {
                    let other = if p == from { &b } else { &a };
                    self.notify(p, format!("You formed an alliance with {other}."), None);
                    let key = format!("alliance:{}", if p == from { to } else { from });
                    self.first_moment(
                        p,
                        &key,
                        format!("Formed an alliance with {other}"),
                        1,
                    );
                } else if self.players[p].relations[from].met {
                    self.notify(p, format!("{a} and {b} formed an alliance."), None);
                }
            }
        }
        accept
    }

    /// Ends an alliance (and the friendship under it).
    pub fn break_alliance(&mut self, a: PlayerId, b: PlayerId) {
        for (x, y) in [(a, b), (b, a)] {
            let r = &mut self.players[x].relations[y];
            r.alliance = false;
            r.friendship = false;
        }
        self.add_grievance(b, a, grievance::BROKE_ALLIANCE);
        let an = self.players[a].civ_name.clone();
        let bn = self.players[b].civ_name.clone();
        self.notify(a, format!("Your alliance with {bn} has ended."), None);
        self.notify(b, format!("{an} abandoned your alliance."), None);
    }

    fn ai_alliances(&mut self, player: PlayerId) {
        for other in 0..self.players.len() {
            if !self.can_ally(player, other) || self.opinion(player, other) < 30 {
                continue;
            }
            if self.players[other].human {
                if self.rng.chance(0.03) {
                    self.popups.push((player, DiploEvent::AllianceOffer));
                }
            } else if self.rng.chance(0.08) {
                self.propose_alliance(player, other);
            }
        }
    }

    /// Allies of `victim` answer the call when `aggressor` declares war.
    pub(crate) fn call_allies(&mut self, aggressor: PlayerId, victim: PlayerId) {
        for ally in 0..self.players.len() {
            if ally == aggressor
                || !self.players[ally].alive
                || !self.allied(victim, ally)
                || !self.can_declare_war(ally, aggressor)
            {
                continue;
            }
            if self.players[ally].human {
                self.popups
                    .push((victim, DiploEvent::CallToArms { against: aggressor }));
            } else {
                self.declare_war_as(ally, aggressor, false);
                let (a, v, x) = (
                    self.players[ally].civ_name.clone(),
                    self.players[victim].civ_name.clone(),
                    self.players[aggressor].civ_name.clone(),
                );
                for p in 0..self.players.len() {
                    if self.players[p].is_major()
                        && (p == aggressor || self.players[p].relations[ally].met)
                    {
                        self.notify(p, format!("{a} honors its alliance with {v} and declares war on {x}."), None);
                    }
                }
            }
        }
    }

    /// The human answers `asker`'s request to fight `against`: a call to
    /// arms from an ally under attack (`defensive`), or an invitation to
    /// join a friend's war.
    pub fn answer_join_war(
        &mut self,
        human: PlayerId,
        asker: PlayerId,
        against: PlayerId,
        join: bool,
        defensive: bool,
    ) {
        if join {
            if self.can_declare_war(human, against) {
                // Answering an ally is not a surprise war.
                self.declare_war_as(human, against, !defensive);
                self.players[asker].relations[human].opinion += 15;
            }
        } else if defensive && self.allied(human, asker) {
            // Abandoning an ally under attack ends the alliance.
            self.break_alliance(human, asker);
        } else {
            self.players[asker].relations[human].opinion -= 5;
        }
    }

    /// Allies share their maps.
    pub(crate) fn share_maps(&mut self) {
        let n = self.players.len();
        for a in 0..n {
            for b in (a + 1)..n {
                if !self.players[a].relations[b].alliance {
                    continue;
                }
                let merged: Vec<bool> = self.players[a]
                    .revealed
                    .iter()
                    .zip(&self.players[b].revealed)
                    .map(|(x, y)| *x || *y)
                    .collect();
                self.players[a].revealed.clone_from(&merged);
                self.players[b].revealed = merged;
            }
        }
    }

    /// Reparations when a war ends: the side ahead on war score is paid.
    pub(crate) fn settle_war(&mut self, a: PlayerId, b: PlayerId) {
        let balance = self.war_balance(a, b);
        if balance.abs() >= 20 && self.players[a].is_major() && self.players[b].is_major() {
            let (winner, loser) = if balance > 0 { (a, b) } else { (b, a) };
            let gold = (self.players[loser].gold / 2).min(25 + balance.abs() * 2);
            self.players[loser].gold -= gold;
            self.players[winner].gold += gold;
            let (w, l) = (
                self.players[winner].civ_name.clone(),
                self.players[loser].civ_name.clone(),
            );
            if gold > 0 {
                self.notify(loser, format!("The peace costs you {gold} Gold in reparations to {w}."), None);
                self.notify(winner, format!("{l} pays you {gold} Gold in reparations."), None);
            }
            self.moment(
                winner,
                format!("Won a war against {l}"),
                crate::eras::score::WON_WAR,
            );
        }
        self.players[a].relations[b].war_score = 0;
        self.players[b].relations[a].war_score = 0;
    }

    /// Whether the AI `player` wants out of its war with `other`.
    pub(crate) fn ai_wants_peace(&self, player: PlayerId, other: PlayerId) -> bool {
        let r = &self.players[player].relations[other];
        let aggression = self.aggression(player) as u32;
        let tired = self.turn > r.since + self.turns(12 + 5 * aggression);
        let balance = self.war_balance(player, other);
        let losing = self.strength_ratio(player, other) < 8 || balance < -25;
        // Taking a city is enough to call it a victory.
        let satisfied = balance >= WAR_SCORE_CITY && self.turn > r.since + self.turns(6);
        tired || losing || satisfied
    }
}

#[cfg(test)]
mod tests {
    use crate::{Settings, load_rules, new_game};

    fn game() -> crate::Game {
        new_game(
            load_rules().unwrap(),
            Settings {
                seed: 5,
                opponents: 3,
                city_states: 0,
                ..Default::default()
            },
        )
    }

    #[test]
    fn surprise_wars_cause_grievances_and_allies_answer() {
        let mut g = game();
        let (a, b, c) = (1, 2, 3);
        for (x, y) in [(a, b), (a, c), (b, c)] {
            g.players[x].relations[y].met = true;
            g.players[y].relations[x].met = true;
        }
        for (x, y) in [(b, c), (c, b)] {
            g.players[x].relations[y].friendship = true;
            g.players[x].relations[y].alliance = true;
        }
        g.declare_war(a, b);
        assert!(g.players[b].relations[a].grievances >= super::grievance::SURPRISE_WAR);
        // c is b's ally, so it joins the war on a.
        assert!(g.players[c].at_war_with(a));
    }

    #[test]
    fn refused_demands_are_remembered() {
        let mut g = game();
        g.players[2].gold = 300;
        g.answer_demand(1, 2, 100, false);
        assert_eq!(g.players[1].relations[2].grievances, super::grievance::REFUSED_DEMAND);
        g.answer_demand(1, 2, 100, true);
        assert_eq!(g.players[2].gold, 200);
    }
}

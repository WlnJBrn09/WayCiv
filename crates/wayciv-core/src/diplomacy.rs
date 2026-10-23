//! Diplomacy between majors, envoys to city-states, and leader dialogue.

use crate::map::PlayerId;
use crate::rules::REAGAN;
use crate::state::Game;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiploEvent {
    FirstMeet,
    Greeting,
    DeclareWarFromAi,
    DeclareWarFromHuman,
    DenounceFromAi,
    DenounceFromHuman,
    AcceptFriendFromHuman,
    RejectFriendFromHuman,
    DeclareFriendFromAi,
    MakePeaceAccept,
    MakePeaceRefuse,
    MakePeaceFromAi,
    Defeat,
}

impl DiploEvent {
    fn key(self) -> &'static str {
        match self {
            DiploEvent::FirstMeet => "FIRST_MEET",
            DiploEvent::Greeting => "GREETING",
            DiploEvent::DeclareWarFromAi => "DECLARE_WAR_FROM_AI",
            DiploEvent::DeclareWarFromHuman => "DECLARE_WAR_FROM_HUMAN",
            DiploEvent::DenounceFromAi => "DENOUNCE_FROM_AI",
            DiploEvent::DenounceFromHuman => "DENOUNCE_FROM_HUMAN",
            DiploEvent::AcceptFriendFromHuman => "ACCEPT_DECLARE_FRIEND_FROM_HUMAN",
            DiploEvent::RejectFriendFromHuman => "REJECT_DECLARE_FRIEND_FROM_HUMAN",
            DiploEvent::DeclareFriendFromAi => "DECLARE_FRIEND_FROM_AI",
            DiploEvent::MakePeaceAccept => "MAKE_PEACE_AI_ACCEPT_DEAL",
            DiploEvent::MakePeaceRefuse => "MAKE_PEACE_AI_REFUSE_DEAL",
            DiploEvent::MakePeaceFromAi => "MAKE_PEACE_FROM_AI",
            DiploEvent::Defeat => "DEFEAT_FROM_HUMAN",
        }
    }
}

/// Ronald Reagan's lines (WayCiv's own; Civ VI has none for him).
fn reagan_line(event: DiploEvent) -> &'static str {
    match event {
        DiploEvent::FirstMeet => {
            "Well, hello there. I'm Ronald Reagan, and I lead the United States of America. We believe in freedom, and we believe the best is yet to come."
        }
        DiploEvent::Greeting => {
            "It's morning in America, my friend. What can we do for each other today?"
        }
        DiploEvent::DeclareWarFromAi => {
            "We've tried to keep the peace, but peace comes through strength. You leave us no choice."
        }
        DiploEvent::DeclareWarFromHuman => {
            "So it's come to this. History will remember who chose war."
        }
        DiploEvent::DenounceFromAi => {
            "I'm calling it what it is: your conduct is a threat to free people everywhere."
        }
        DiploEvent::DenounceFromHuman => {
            "Words are cheap. We'll judge you by your deeds, and so will the world."
        }
        DiploEvent::AcceptFriendFromHuman => {
            "Trust, but verify. Still, I'm glad to call you a friend."
        }
        DiploEvent::RejectFriendFromHuman => {
            "Not just yet. Friendship has to be earned, and we're not there."
        }
        DiploEvent::DeclareFriendFromAi => {
            "Free nations ought to stand together. What do you say we make it official?"
        }
        DiploEvent::MakePeaceAccept => {
            "Peace it is. Let's put this behind us and get back to building."
        }
        DiploEvent::MakePeaceRefuse => {
            "Peace? Not on those terms. We'll talk again when you mean it."
        }
        DiploEvent::MakePeaceFromAi => {
            "We've proven our resolve. I'm prepared to sit down and talk peace."
        }
        DiploEvent::Defeat => "We gave it everything we had. America will rise again.",
    }
}

impl Game {
    /// What a leader says for `event`, from Civ VI's text when available.
    pub fn dialogue(&self, player: PlayerId, event: DiploEvent) -> String {
        let leader = &self.rules.content.leaders[self.players[player].leader];
        if leader.id == REAGAN {
            return reagan_line(event).to_string();
        }
        for id in [leader.id.as_str(), leader.id.trim_end_matches("_ALT")] {
            let key = format!("LOC_DIPLO_{}_{}_ANY", event.key(), id);
            let text = self.rules.text(&key);
            if text != key {
                return text;
            }
        }
        match event {
            DiploEvent::Greeting | DiploEvent::FirstMeet => format!(
                "Greetings. I am {} of {}.",
                leader.name, self.players[player].civ_name
            ),
            DiploEvent::DeclareWarFromAi => "Our patience has run out. Prepare for war!".into(),
            DiploEvent::MakePeaceAccept => "Very well. Let there be peace.".into(),
            DiploEvent::MakePeaceRefuse => "We are not ready for peace.".into(),
            DiploEvent::AcceptFriendFromHuman => "We accept your friendship.".into(),
            DiploEvent::RejectFriendFromHuman => "We must decline.".into(),
            _ => "...".into(),
        }
    }

    /// City-states can't be attacked: WayCiv has no war with them.
    pub fn can_declare_war(&self, a: PlayerId, b: PlayerId) -> bool {
        a != b
            && !self.players[a].at_war_with(b)
            && self.players[a].is_major()
            && self.players[b].is_major()
    }

    pub fn declare_war(&mut self, a: PlayerId, b: PlayerId) {
        if !self.can_declare_war(a, b) {
            return;
        }
        let turn = self.turn;
        for (x, y) in [(a, b), (b, a)] {
            let r = &mut self.players[x].relations[y];
            r.at_war = true;
            r.since = turn;
            r.friendship = false;
            r.opinion -= 40;
        }
        // Third parties disapprove of warmongers.
        for p in 0..self.players.len() {
            if p != a && p != b && self.players[p].relations[a].met {
                self.players[p].relations[a].opinion -= 10;
            }
        }
        let (an, bn) = (
            self.players[a].civ_name.clone(),
            self.players[b].civ_name.clone(),
        );
        for p in 0..self.players.len() {
            if self.players[p].is_major() && (p == a || p == b || self.players[p].relations[a].met)
            {
                let text = if p == b {
                    format!("{an} has declared war on you!")
                } else {
                    format!("{an} declared war on {bn}.")
                };
                self.notify(p, text, None);
            }
        }
        // Trade between the two stops.
        self.cancel_routes(|g, r| {
            let other = g.cities.get(&r.destination).map(|c| c.owner);
            !((r.owner == a && other == Some(b)) || (r.owner == b && other == Some(a)))
        });
        self.check_boosts(b, crate::research::Trigger::DeclaredWarOn);
        self.check_boosts(a, crate::research::Trigger::DeclaredWarOn);
    }

    pub fn can_make_peace(&self, a: PlayerId, b: PlayerId) -> bool {
        self.players[a].at_war_with(b) && self.turn >= self.players[a].relations[b].since + 10
    }

    pub fn make_peace(&mut self, a: PlayerId, b: PlayerId) {
        let turn = self.turn;
        for (x, y) in [(a, b), (b, a)] {
            let r = &mut self.players[x].relations[y];
            r.at_war = false;
            r.since = turn;
        }
        let (an, bn) = (
            self.players[a].civ_name.clone(),
            self.players[b].civ_name.clone(),
        );
        self.notify(a, format!("Peace with {bn}."), None);
        self.notify(b, format!("Peace with {an}."), None);
    }

    /// Military strength used by AI diplomacy and war decisions.
    pub fn military_strength(&self, player: PlayerId) -> i32 {
        self.player_units(player)
            .filter(|u| self.is_military(u))
            .map(|u| {
                self.unit_type(u)
                    .combat
                    .max(self.unit_type(u).ranged_combat)
                    * u.hp
                    / 100
            })
            .sum()
    }

    /// AI opinion of another player, from its relations and agenda.
    pub fn opinion(&self, of: PlayerId, about: PlayerId) -> i32 {
        let me = &self.players[of];
        let r = &me.relations[about];
        let mut o = r.opinion;
        if r.friendship {
            o += 30;
        }
        if r.denounced || self.players[about].relations[of].denounced {
            o -= 25;
        }
        if r.at_war {
            o -= 50;
        }
        if me.religion.is_some() && self.dominant_religion(about) == me.religion {
            o += 10;
        }
        // Neighbors close by grate on each other.
        let near = self.player_cities(of).any(|a| {
            self.player_cities(about)
                .any(|b| self.map.grid.distance(a.pos, b.pos) <= 8)
        });
        if near {
            o -= 10;
        }
        // Reagan's agenda: Cold Warrior.
        if self.rules.content.leaders[me.leader].id == REAGAN
            && let Some(g) = self.players[about].government
        {
            match self.rules.content.governments[g].id.as_str() {
                "GOVERNMENT_DEMOCRACY" => o += 20,
                "GOVERNMENT_COMMUNISM" | "GOVERNMENT_FASCISM" => o -= 20,
                _ => {}
            }
        }
        o.clamp(-100, 100)
    }

    /// Asks `to` (AI) to declare friendship with `from`. Returns acceptance.
    pub fn propose_friendship(&mut self, from: PlayerId, to: PlayerId) -> bool {
        let accept = !self.players[from].at_war_with(to)
            && (self.players[to].human || self.opinion(to, from) >= 10);
        if accept {
            self.players[from].relations[to].friendship = true;
            self.players[to].relations[from].friendship = true;
            for (who, other) in [(from, to), (to, from)] {
                if self.rules.content.leaders[self.players[who].leader].id == REAGAN {
                    self.players[who].gold += 50;
                    if let Some(cs) = self
                        .players
                        .iter()
                        .find(|p| {
                            p.is_city_state() && p.alive && self.players[who].relations[p.id].met
                        })
                        .map(|p| p.id)
                    {
                        *self.players[who].envoys.entry(cs).or_insert(0) += 1;
                    }
                }
                let name = self.players[other].civ_name.clone();
                self.notify(who, format!("Declared friendship with {name}."), None);
            }
        }
        accept
    }

    pub fn denounce(&mut self, from: PlayerId, to: PlayerId) {
        self.players[from].relations[to].denounced = true;
        self.players[from].relations[to].friendship = false;
        self.players[to].relations[from].friendship = false;
        self.players[to].relations[from].opinion -= 20;
        let name = self.players[from].civ_name.clone();
        self.notify(to, format!("{name} has denounced you."), None);
    }

    /// Asks `to` (AI) for peace. Accepts when losing or tired of the war.
    pub fn propose_peace(&mut self, from: PlayerId, to: PlayerId) -> bool {
        if !self.can_make_peace(from, to) {
            return false;
        }
        let tired = self.turn >= self.players[to].relations[from].since + 25;
        let losing = self.military_strength(to) < self.military_strength(from);
        let accept = self.players[to].human || tired || losing;
        if accept {
            self.make_peace(from, to);
        }
        accept
    }

    pub fn send_envoy(&mut self, player: PlayerId, state: PlayerId) -> Result<(), String> {
        let p = &self.players[player];
        if p.envoys_available <= 0 {
            return Err("No envoys available.".into());
        }
        if self.players[state].is_major() || !p.relations[state].met || !self.players[state].alive {
            return Err("You can't send an envoy there.".into());
        }
        let p = &mut self.players[player];
        p.envoys_available -= 1;
        *p.envoys.entry(state).or_insert(0) += 1;
        Ok(())
    }

    /// Influence points earned toward envoys each turn.
    pub(crate) fn process_influence(&mut self, player: PlayerId) {
        let p = &mut self.players[player];
        if !p.is_major() {
            return;
        }
        p.influence += 1 + i32::from(p.government.is_some());
        let threshold = 30;
        if p.influence >= threshold {
            p.influence -= threshold;
            p.envoys_available += 1;
        }
    }
}

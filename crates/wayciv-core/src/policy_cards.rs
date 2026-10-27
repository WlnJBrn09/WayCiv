//! Policy cards. With no civics, policies unlock by calendar year, but a
//! civilization can only slot the cards it has adopted. Culture fills an
//! inspiration meter; each time it fills, three unlocked cards are offered
//! and the player keeps one. Newer cards are offered more often, and a card
//! that has a successor leaves once the successor is adopted.

use crate::map::PlayerId;
use crate::state::Game;

impl Game {
    /// Culture needed for the next card.
    pub fn inspiration_cost(&self, player: PlayerId) -> i32 {
        let n = self.players[player].cards.count() as i32;
        self.scaled(15 + 14 * n + n * n / 2)
    }

    /// Unlocked cards the player hasn't adopted.
    pub fn draftable_policies(&self, player: PlayerId) -> Vec<usize> {
        let adopted = &self.players[player].cards.adopted;
        self.unlocked_policies(player)
            .into_iter()
            .filter(|&i| !adopted[i])
            .collect()
    }

    /// Offers up to three cards if none are on offer. Newer cards come
    /// first; slot kinds are mixed when possible.
    pub fn offer_policy_draft(&mut self, player: PlayerId) -> bool {
        if !self.players[player].is_major() || !self.players[player].cards.draft.is_empty() {
            return false;
        }
        let content = &self.rules.content;
        let mut pool = self.draftable_policies(player);
        if pool.is_empty() {
            return false;
        }
        // The newest dozen, shuffled, then one per slot kind first.
        pool.sort_by_key(|&p| std::cmp::Reverse(content.policies[p].unlock_year));
        pool.truncate(12);
        self.rng.shuffle(&mut pool);
        let mut draft: Vec<usize> = Vec::new();
        for &p in &pool {
            if draft.len() < 3
                && !draft
                    .iter()
                    .any(|&d| content.policies[d].slot == content.policies[p].slot)
            {
                draft.push(p);
            }
        }
        for &p in &pool {
            if draft.len() < 3 && !draft.contains(&p) {
                draft.push(p);
            }
        }
        let human = self.players[player].human;
        self.players[player].cards.draft = draft;
        if human {
            self.notify(player, "New policy cards are on offer: choose one to keep.", None);
        }
        true
    }

    /// Keeps one of the offered cards. It goes into a free slot if one fits,
    /// replacing the card it supersedes.
    pub fn adopt_policy(&mut self, player: PlayerId, policy: usize) -> Result<(), String> {
        if !self.players[player].cards.draft.contains(&policy) {
            return Err("That card isn't on offer.".into());
        }
        let content = self.rules.content.clone();
        let p = &mut self.players[player];
        p.cards.adopted[policy] = true;
        p.cards.draft.clear();
        // Superseded cards leave the slots.
        let id = &content.policies[policy].id;
        let mut slotted: Vec<usize> = p
            .policies
            .iter()
            .copied()
            .filter(|&q| content.policies[q].obsoleted_by.as_deref() != Some(id.as_str()))
            .collect();
        let mut with = slotted.clone();
        with.push(policy);
        if self.policies_fit(player, &with) {
            slotted = with;
        }
        let p = &mut self.players[player];
        p.policies = slotted;
        // Rearranging slots around the new card is free this turn.
        p.policy_change_free = true;
        let name = content.policies[policy].name.clone();
        self.notify(player, format!("Adopted the {name} policy card."), None);
        // Banked inspiration may already pay for the next offer.
        self.process_inspiration(player, 0.0);
        Ok(())
    }

    /// Adds a turn's Culture to the inspiration meter and offers cards
    /// when it fills.
    pub(crate) fn process_inspiration(&mut self, player: PlayerId, culture: f32) {
        if !self.players[player].is_major() {
            return;
        }
        let cost = self.inspiration_cost(player);
        let cards = &mut self.players[player].cards;
        // While an offer waits, inspiration banks up to one more card.
        cards.inspiration = (cards.inspiration + culture.round() as i32).min(cost * 2);
        if cards.draft.is_empty() && cards.inspiration >= cost && self.offer_policy_draft(player) {
            self.players[player].cards.inspiration -= cost;
        }
    }

    /// The AI keeps the card that suits its focus and its empty slots.
    pub(crate) fn ai_pick_policy(&mut self, player: PlayerId) {
        let draft = self.players[player].cards.draft.clone();
        if draft.is_empty() {
            return;
        }
        let content = &self.rules.content;
        let focus = self.personality(player).focus.clone();
        let wanted_yield = match focus.as_str() {
            "science" => Some("science"),
            "culture" => Some("culture"),
            "faith" => Some("faith"),
            "trade" => Some("gold"),
            "expansion" => Some("food"),
            _ => None,
        };
        let slots = self.policy_slots(player);
        let have = &self.players[player].policies;
        let score = |q: usize| -> i32 {
            let pol = &content.policies[q];
            let mut s = pol.unlock_year / 100;
            let mut with = have.clone();
            with.push(q);
            if self.policies_fit(player, &with) {
                s += 30;
            }
            if slots.iter().any(|(k, _)| *k == pol.slot) {
                s += 10;
            }
            if focus == "war" && pol.slot == "SLOT_MILITARY" {
                s += 15;
            }
            if let Some(y) = wanted_yield
                && pol.effects.iter().any(|e| e.yield_key() == Some(y))
            {
                s += 15;
            }
            s
        };
        if let Some(&best) = draft.iter().max_by_key(|&&q| score(q)) {
            let _ = self.adopt_policy(player, best);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Settings, load_rules, new_game};

    #[test]
    fn culture_buys_policy_cards() {
        let mut g = new_game(
            load_rules().unwrap(),
            Settings {
                seed: 2,
                opponents: 1,
                city_states: 0,
                ..Default::default()
            },
        );
        let me = g.human();
        // A first offer is waiting at the start.
        let draft = g.players[me].cards.draft.clone();
        assert!(!draft.is_empty() && draft.len() <= 3);
        assert!(g.available_policies(me).is_empty());
        g.adopt_policy(me, draft[0]).unwrap();
        assert_eq!(g.available_policies(me), vec![draft[0]]);
        assert!(g.players[me].cards.draft.is_empty());
        let cost = g.inspiration_cost(me);
        g.process_inspiration(me, cost as f32);
        assert!(!g.players[me].cards.draft.is_empty());
        assert_eq!(g.players[me].cards.inspiration, 0);
    }
}

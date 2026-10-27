//! The empire menus: Religion (R), Government (G), Great People (P),
//! City-States (E), Victory (V), Eras (T), and the policy card offer.
//!
//! Each one starts with an instruction strip saying what to do next, shows
//! its choices as cards (click to pick, gold frame = chosen, dimmed = not
//! available yet, with the reason), and closes itself once the choice it
//! exists for has been made. Each returns true when it should close.

use wayciv_core::length::year_label;
use super::cards::{self, Card, State};
use super::glyph::Glyph;
use crate::hud::Session;
use crate::palette::{self, GOLD, GOOD, TEXT, TEXT_DIM, YIELD_COLORS, rgb};
use egui::{Color32, RichText, Sense, Ui, vec2};
use wayciv_core::religion::PANTHEON_COST;
use wayciv_core::{PlayerId, UnitId};

const FAITH: Color32 = YIELD_COLORS[5];

/// The scroll area every menu puts its cards in, leaving `footer` pixels
/// for a button row underneath.
fn body(ui: &mut Ui, footer: f32, add: impl FnOnce(&mut Ui)) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .max_height((ui.available_height() - footer).max(120.0))
        .show(ui, add);
}

fn turns_to(have: f32, need: f32, rate: f32) -> Option<i32> {
    (rate > 0.0).then(|| ((need - have).max(0.0) / rate).ceil() as i32)
}

// ---------------------------------------------------------------- Religion

/// A religion being founded: the Great Prophet and the picks so far.
#[derive(Clone, Copy, Debug)]
pub struct FoundDraft {
    pub unit: UnitId,
    pub religion: Option<usize>,
    pub founder: Option<usize>,
    pub follower: Option<usize>,
}

impl FoundDraft {
    pub fn new(unit: UnitId) -> Self {
        FoundDraft {
            unit,
            religion: None,
            founder: None,
            follower: None,
        }
    }
}

/// A religion's color, or Faith purple when the data gives none (white).
fn religion_color(c: [u8; 3]) -> Color32 {
    if c.iter().all(|&v| v > 200) {
        FAITH
    } else {
        rgb(c)
    }
}

enum ReligionAct {
    Pantheon(usize),
    Draft(FoundDraft),
    Found(FoundDraft),
}

pub fn religion(s: &mut Session, ui: &mut Ui) -> bool {
    let me = s.me();
    let draft = s
        .windows
        .found
        .filter(|d| s.game.can_found_religion(d.unit));
    let game = &s.game;
    let icons = &mut s.icons;
    let content = &game.rules.content;
    let p = &game.players[me];
    let faith_rate = game.income(me).yields[5];
    let can_pantheon = game.can_choose_pantheon(me);
    let mut act = None;

    let wait = turns_to(p.faith as f32, PANTHEON_COST as f32, faith_rate)
        .map(|t| format!(", about {t} turns"))
        .unwrap_or_default();
    let text = if draft.is_some() {
        "Your Great Prophet is ready. Pick a name and two beliefs, then press Found.".to_string()
    } else if p.pantheon.is_none() && can_pantheon {
        "Pick a pantheon belief below. It works in all your cities right away.".to_string()
    } else if p.pantheon.is_none() {
        format!(
            "Save up {PANTHEON_COST} Faith to found a pantheon (you have {}{wait}). Until then you can read the beliefs below.",
            p.faith
        )
    } else if p.religion.is_none() {
        "Next: earn a Great Prophet. Holy Sites make Prophet points; when the Prophet appears, move it onto a Holy Site and press Found Religion.".to_string()
    } else {
        "Spread your religion: buy Missionaries with Faith in your cities and send them to other cities.".to_string()
    };
    cards::instruction(ui, &text);

    if p.pantheon.is_none() {
        cards::progress(
            ui,
            Glyph::Faith,
            FAITH,
            "Faith",
            &format!("{} / {PANTHEON_COST}", p.faith),
            p.faith as f32 / PANTHEON_COST as f32,
            &format!(
                "+{faith_rate:.1} per turn. Shrines, Holy Sites and some natural wonders make Faith."
            ),
        );
    } else {
        ui.label(
            RichText::new(format!("Faith: {}  (+{faith_rate:.1} per turn)", p.faith))
                .size(16.0)
                .color(FAITH),
        );
    }

    body(ui, 60.0, |ui| {
        if let Some(d) = draft {
            let mut next = d;
            cards::section(ui, "1. Name", "which religion your people follow");
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
                for r in game.free_religions() {
                    let rel = &content.religions[r];
                    if cards::choice(
                        ui,
                        &rel.name,
                        religion_color(rel.color),
                        d.religion == Some(r),
                    ) {
                        next.religion = Some(r);
                    }
                }
            });
            for (title, hint, class, slot) in [
                (
                    "2. Founder belief",
                    "helps you, the founder",
                    "BELIEF_CLASS_FOUNDER",
                    0,
                ),
                (
                    "3. Follower belief",
                    "helps every city that follows it",
                    "BELIEF_CLASS_FOLLOWER",
                    1,
                ),
            ] {
                cards::section(ui, title, hint);
                let list = game.beliefs_of_class(class);
                let picked = if slot == 0 { d.founder } else { d.follower };
                cards::grid(ui, 300.0, list.len(), |ui, i, w| {
                    let b = &content.beliefs[list[i]];
                    let chosen = picked == Some(list[i]);
                    let card = Card::new(&b.name, &b.description)
                        .icon("belief", &b.id)
                        .glyph(Glyph::Faith)
                        .accent(FAITH)
                        .state(if chosen { State::Chosen } else { State::Open });
                    if card.show(ui, icons, w).clicked() {
                        if slot == 0 {
                            next.founder = Some(list[i]);
                        } else {
                            next.follower = Some(list[i]);
                        }
                    }
                });
            }
            ui.add_space(6.0);
            let ready =
                next.religion.is_some() && next.founder.is_some() && next.follower.is_some();
            let name = next
                .religion
                .map(|r| content.religions[r].name.as_str())
                .unwrap_or("Religion");
            if cards::action_button(
                ui,
                &format!("Found {name}"),
                ready,
                "Pick a name and both beliefs first.",
            ) {
                act = Some(ReligionAct::Found(next));
            } else if next.religion != d.religion
                || next.founder != d.founder
                || next.follower != d.follower
            {
                act = Some(ReligionAct::Draft(next));
            }
            return;
        }

        match p.pantheon {
            Some(b) => {
                cards::section(ui, "Your pantheon", "");
                let b = &content.beliefs[b];
                Card::new(&b.name, &b.description)
                    .icon("belief", &b.id)
                    .glyph(Glyph::Faith)
                    .accent(FAITH)
                    .state(State::Info)
                    .corner("Active", GOOD)
                    .show(ui, icons, ui.available_width().min(520.0));
            }
            None => {
                cards::section(ui, "Pantheon beliefs", "choose one");
                let list = game.beliefs_of_class("BELIEF_CLASS_PANTHEON");
                cards::grid(ui, 300.0, list.len(), |ui, i, w| {
                    let b = &content.beliefs[list[i]];
                    let card = Card::new(&b.name, &b.description)
                        .icon("belief", &b.id)
                        .glyph(Glyph::Faith)
                        .accent(FAITH)
                        .state(if can_pantheon {
                            State::Open
                        } else {
                            State::Locked
                        });
                    let r = card.show(ui, icons, w);
                    let r = if can_pantheon {
                        r
                    } else {
                        r.on_hover_text(format!(
                            "Needs {PANTHEON_COST} Faith (you have {}).",
                            p.faith
                        ))
                    };
                    if r.clicked() {
                        act = Some(ReligionAct::Pantheon(list[i]));
                    }
                });
            }
        }

        match p.religion {
            Some(r) => {
                cards::section(
                    ui,
                    &format!("Your religion: {}", content.religions[r].name),
                    "",
                );
                let beliefs = p.beliefs.clone();
                cards::grid(ui, 300.0, beliefs.len(), |ui, i, w| {
                    let b = &content.beliefs[beliefs[i]];
                    Card::new(&b.name, &b.description)
                        .icon("belief", &b.id)
                        .glyph(Glyph::Faith)
                        .accent(FAITH)
                        .state(State::Info)
                        .show(ui, icons, w);
                });
            }
            None if p.pantheon.is_some() => {
                cards::section(ui, "Founding a religion", "");
                let left = game.free_religions().len();
                if left == 0 {
                    ui.label(RichText::new("Every religion has been founded. Your cities can still follow another civilization's religion.").color(TEXT_DIM));
                } else if let Some(class) = content
                    .great_person_classes
                    .iter()
                    .position(|c| c.id == "GREAT_PERSON_CLASS_PROPHET")
                {
                    let points = p.great_person_points.get(&class).copied().unwrap_or(0);
                    let cost = game.great_person_cost(class);
                    let rate = game
                        .great_person_rates(me)
                        .into_iter()
                        .find(|(c, _)| *c == class)
                        .map_or(0, |(_, n)| n);
                    let hint = match turns_to(points as f32, cost as f32, rate as f32) {
                        Some(t) => format!(
                            "+{rate} per turn, about {t} turns. {left} religions are still free to found."
                        ),
                        None => format!(
                            "No Prophet points yet: build a Holy Site district in a city. {left} religions are still free to found."
                        ),
                    };
                    cards::progress(
                        ui,
                        Glyph::Star,
                        FAITH,
                        "Great Prophet",
                        &format!("{points} / {cost}"),
                        points as f32 / cost.max(1) as f32,
                        &hint,
                    );
                }
            }
            None => {}
        }

        if !game.religions.is_empty() {
            cards::section(ui, "Religions of the world", "");
            let mut list: Vec<(usize, PlayerId)> =
                game.religions.iter().map(|(&r, &(f, _))| (r, f)).collect();
            list.sort();
            for (r, founder) in list {
                let cities = game
                    .cities
                    .values()
                    .filter(|c| c.majority_religion() == Some(r))
                    .count();
                let rel = &content.religions[r];
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                    ui.painter()
                        .circle_filled(rect.center(), 6.0, religion_color(rel.color));
                    ui.label(RichText::new(&rel.name).size(16.0).color(TEXT));
                    let by = if founder == me {
                        "founded by you".to_string()
                    } else {
                        format!("founded by {}", game.players[founder].civ_name)
                    };
                    ui.label(
                        RichText::new(format!("{by}, followed by {cities} cities")).color(TEXT_DIM),
                    );
                });
            }
        }
    });

    match act {
        Some(ReligionAct::Pantheon(b)) => match s.game.choose_pantheon(me, b) {
            Ok(()) => {
                let name = s.game.rules.content.beliefs[b].name.clone();
                s.toast(format!("Pantheon founded: {name}."));
                return true;
            }
            Err(e) => s.toast(e),
        },
        Some(ReligionAct::Draft(d)) => s.windows.found = Some(d),
        Some(ReligionAct::Found(d)) => {
            let r = s.game.found_religion(
                d.unit,
                d.religion.unwrap(),
                d.founder.unwrap(),
                d.follower.unwrap(),
            );
            match r {
                Ok(()) => {
                    let name = s.game.rules.content.religions[d.religion.unwrap()]
                        .name
                        .clone();
                    s.toast(format!("You founded {name}!"));
                    s.windows.found = None;
                    s.refresh_selection();
                    return true;
                }
                Err(e) => s.toast(e),
            }
        }
        None => {}
    }
    false
}

// -------------------------------------------------------------- Government

pub const SLOTS: [&str; 5] = [
    "SLOT_MILITARY",
    "SLOT_ECONOMIC",
    "SLOT_DIPLOMATIC",
    "SLOT_GREAT_PERSON",
    "SLOT_WILDCARD",
];

pub fn slot_name(slot: &str) -> &'static str {
    match slot {
        "SLOT_MILITARY" => "Military",
        "SLOT_ECONOMIC" => "Economic",
        "SLOT_DIPLOMATIC" => "Diplomatic",
        "SLOT_GREAT_PERSON" => "Great Person",
        "SLOT_WILDCARD" => "Wildcard",
        _ => "Other",
    }
}

/// Civ VI's card colors: red military, yellow economic, green diplomatic,
/// purple great person, white wildcard.
pub fn slot_color(slot: &str) -> Color32 {
    match slot {
        "SLOT_MILITARY" => Color32::from_rgb(222, 86, 70),
        "SLOT_ECONOMIC" => Color32::from_rgb(236, 192, 64),
        "SLOT_DIPLOMATIC" => Color32::from_rgb(84, 184, 120),
        "SLOT_GREAT_PERSON" => Color32::from_rgb(168, 116, 222),
        _ => Color32::from_rgb(210, 214, 224),
    }
}

fn slot_glyph(slot: &str) -> Glyph {
    match slot {
        "SLOT_MILITARY" => Glyph::Swords,
        "SLOT_ECONOMIC" => Glyph::Gold,
        "SLOT_DIPLOMATIC" => Glyph::Crown,
        "SLOT_GREAT_PERSON" => Glyph::Star,
        _ => Glyph::Hex,
    }
}

/// Puts each policy of `draft` into a slot (its own kind first, then a
/// wildcard), returning every slot with what is in it.
fn assign(
    game: &wayciv_core::Game,
    slots: &[(String, i32)],
    draft: &[usize],
) -> Vec<(String, Option<usize>)> {
    let content = &game.rules.content;
    let mut out: Vec<(String, Option<usize>)> = Vec::new();
    for kind in SLOTS {
        for (k, n) in slots {
            if k == kind {
                for _ in 0..*n {
                    out.push((k.clone(), None));
                }
            }
        }
    }
    for &p in draft {
        let kind = &content.policies[p].slot;
        let spot = out
            .iter()
            .position(|(k, x)| x.is_none() && k == kind)
            .or_else(|| {
                out.iter()
                    .position(|(k, x)| x.is_none() && k == "SLOT_WILDCARD")
            });
        if let Some(i) = spot {
            out[i].1 = Some(p);
        }
    }
    out
}

enum GovAct {
    Adopt(usize),
    Remove(usize),
    Add(usize),
    Full(String),
    Confirm,
    Undo,
    Cards,
}

pub fn government(s: &mut Session, ui: &mut Ui) -> bool {
    let me = s.me();
    let game = &s.game;
    let icons = &mut s.icons;
    let content = &game.rules.content;
    let p = &game.players[me];
    let govs = game.available_governments(me);
    if govs.is_empty() {
        cards::instruction(ui, "No government is available yet.");
        return false;
    }
    let draft: Vec<usize> = s
        .windows
        .policy_draft
        .clone()
        .unwrap_or_else(|| p.policies.clone());
    let sorted = |v: &[usize]| {
        let mut v = v.to_vec();
        v.sort();
        v
    };
    let dirty = sorted(&draft) != sorted(&p.policies);
    let cost = game.policy_change_cost(me, &draft);
    let slots = game.policy_slots(me);
    let assigned = assign(game, &slots, &draft);
    let available = game.available_policies(me);
    let fits = |q: usize| {
        let mut v = draft.clone();
        v.push(q);
        game.policies_fit(me, &v)
    };
    let empty = assigned.iter().any(|(_, x)| x.is_none());
    let addable = available.iter().any(|&q| !draft.contains(&q) && fits(q));
    let text = if p.government.is_none() {
        "Pick a government. It gives you policy slots, which you then fill with policy cards."
            .to_string()
    } else if dirty {
        format!(
            "Press Confirm Policies to make your changes official{}.",
            if cost > 0 {
                format!(" (costs {cost} Gold)")
            } else {
                " (free right now)".into()
            }
        )
    } else if empty && addable {
        "Click a policy card to put it in an empty slot. Click a slotted card to take it out."
            .to_string()
    } else {
        "Your slots are full. To swap a policy, click a slotted card to take it out, then pick another.".to_string()
    };
    cards::instruction(ui, &text);
    let mut act = None;
    // Policy cards: only kept cards can be slotted; Culture earns more.
    ui.horizontal(|ui| {
        let cost = game.inspiration_cost(me);
        ui.label(
            RichText::new(format!(
                "You can slot the {} policy cards you have kept. Next card: {} / {cost} Culture.",
                p.cards.count(),
                p.cards.inspiration.min(cost)
            ))
            .color(TEXT_DIM),
        );
        if !p.cards.draft.is_empty()
            && super::theme::pill_button(ui, Some(Glyph::Star), "Choose a new card", palette::TEAL, 0.0)
                .clicked()
        {
            act = Some(GovAct::Cards);
        }
    });

    body(ui, 64.0, |ui| {
        cards::section(ui, "Government", "click another to switch");
        cards::grid(ui, 260.0, govs.len(), |ui, i, w| {
            let g = &content.governments[govs[i]];
            let current = p.government == Some(govs[i]);
            let summary: Vec<String> = g
                .slots
                .iter()
                .map(|(t, n)| format!("{n} {}", slot_name(t)))
                .collect();
            let summary = format!(
                "Slots: {} · since {}",
                summary.join(", "),
                year_label(g.unlock_year)
            );
            let mut card = Card::new(&g.name, &g.bonus)
                .glyph(Glyph::Temple)
                .footer(&summary)
                .state(if current { State::Info } else { State::Open });
            card = if current {
                card.corner("Current", GOLD)
            } else {
                card.corner("Adopt", palette::TEAL)
            };
            if card.show(ui, icons, w).clicked() && !current {
                act = Some(GovAct::Adopt(govs[i]));
            }
        });
        if p.government.is_none() {
            return;
        }

        cards::section(ui, "Your policy slots", "click a card to take it out");
        cards::grid(ui, 250.0, assigned.len(), |ui, i, w| {
            let (kind, policy) = &assigned[i];
            match policy {
                Some(q) => {
                    let pol = &content.policies[*q];
                    let card = Card::new(&pol.name, &pol.description)
                        .glyph(slot_glyph(kind))
                        .accent(slot_color(&pol.slot))
                        .state(State::Chosen);
                    if card
                        .show(ui, icons, w)
                        .on_hover_text("Click to take this card out")
                        .clicked()
                    {
                        act = Some(GovAct::Remove(*q));
                    }
                }
                None => {
                    let title = format!("Empty {} slot", slot_name(kind));
                    let body = if kind == "SLOT_WILDCARD" {
                        "Any kind of policy fits here.".to_string()
                    } else {
                        format!("Pick a {} policy below.", slot_name(kind))
                    };
                    Card::new(&title, &body)
                        .glyph(slot_glyph(kind))
                        .accent(slot_color(kind))
                        .state(State::Info)
                        .show(ui, icons, w);
                }
            }
        });

        for kind in &SLOTS[..4] {
            let list: Vec<usize> = available
                .iter()
                .copied()
                .filter(|&q| content.policies[q].slot == *kind && !draft.contains(&q))
                .collect();
            if list.is_empty() {
                continue;
            }
            cards::section(
                ui,
                &format!("{} policies", slot_name(kind)),
                "click to slot",
            );
            cards::grid(ui, 250.0, list.len(), |ui, i, w| {
                let q = list[i];
                let pol = &content.policies[q];
                let since = format!("Since {}", year_label(pol.unlock_year));
                let card = Card::new(&pol.name, &pol.description)
                    .glyph(slot_glyph(kind))
                    .accent(slot_color(kind))
                    .footer(&since)
                    .state(State::Open);
                let r = card.show(ui, icons, w);
                if r.clicked() {
                    act = Some(if fits(q) {
                        GovAct::Add(q)
                    } else {
                        GovAct::Full(slot_name(kind).to_string())
                    });
                }
            });
        }

        // What the calendar unlocks next.
        if let Some(year) = game.next_unlock_year() {
            let coming: Vec<(String, String, String)> = content
                .governments
                .iter()
                .filter(|g| g.unlock_year == year)
                .map(|g| (g.name.clone(), g.bonus.clone(), "SLOT_WILDCARD".to_string()))
                .chain(
                    content
                        .policies
                        .iter()
                        .filter(|q| q.unlock_year == year)
                        .map(|q| (q.name.clone(), q.description.clone(), q.slot.clone())),
                )
                .collect();
            let when = format!(
                "in {} (turn {})",
                year_label(year),
                game.turn_of_year(year)
            );
            cards::section(ui, "Coming next", &when);
            cards::grid(ui, 250.0, coming.len(), |ui, i, w| {
                let (name, text, kind) = &coming[i];
                Card::new(name, text)
                    .glyph(slot_glyph(kind))
                    .accent(slot_color(kind))
                    .state(State::Info)
                    .show(ui, icons, w);
            });
        }
    });

    if p.government.is_some() {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let label = if cost > 0 {
                format!("Confirm Policies ({cost} Gold)")
            } else {
                "Confirm Policies".to_string()
            };
            let why = if !dirty {
                "Nothing has changed yet."
            } else {
                "Not enough Gold."
            };
            if cards::action_button(ui, &label, dirty && p.gold >= cost, why) {
                act = Some(GovAct::Confirm);
            }
            if dirty && super::theme::pill_button(ui, None, "Undo changes", TEXT, 0.0).clicked() {
                act = Some(GovAct::Undo);
            }
        });
    }

    let mut draft = draft;
    match act {
        Some(GovAct::Adopt(g)) => {
            let r = s.game.set_government(me, g);
            if r.is_ok() {
                let name = s.game.rules.content.governments[g].name.clone();
                s.toast(format!("Adopted {name}. Now fill its policy slots."));
            }
            s.result(r);
            s.windows.policy_draft = None;
            return false;
        }
        Some(GovAct::Remove(q)) => draft.retain(|&x| x != q),
        Some(GovAct::Add(q)) => draft.push(q),
        Some(GovAct::Full(kind)) => s.toast(format!(
            "No free {kind} slot. Click a slotted {kind} card to take it out first."
        )),
        Some(GovAct::Undo) => {
            s.windows.policy_draft = None;
            return false;
        }
        Some(GovAct::Cards) => {
            s.windows.open(crate::screens::Panel::Cards);
            return false;
        }
        Some(GovAct::Confirm) => match s.game.set_policies(me, draft.clone()) {
            Ok(()) => {
                s.game.players[me].policy_change_free = false;
                s.windows.policy_draft = None;
                s.toast("Policies updated.");
                return true;
            }
            Err(e) => s.toast(e),
        },
        None => {}
    }
    s.windows.policy_draft = Some(draft);
    false
}

// ------------------------------------------------------------ Great People

pub fn great_people(s: &mut Session, ui: &mut Ui) -> bool {
    let me = s.me();
    let game = &s.game;
    let icons = &mut s.icons;
    let content = &game.rules.content;
    let p = &game.players[me];
    cards::instruction(
        ui,
        "Great People join whoever fills their bar first. Build the district named on each card to earn points; the person then appears in that city.",
    );
    let rates = game.great_person_rates(me);
    let mut rows: Vec<(usize, i32, i32, i32)> = (0..content.great_person_classes.len())
        .filter(|&c| game.next_great_person(c).is_some())
        // Classes like Gran Colombia's Comandante General belong to one civ.
        .filter(|&c| {
            let unit = &content.great_person_classes[c].unit;
            content
                .units
                .iter()
                .find(|u| &u.id == unit)
                .is_none_or(|u| game.unique_ok(me, &u.unique_to))
        })
        .map(|c| {
            let points = p.great_person_points.get(&c).copied().unwrap_or(0);
            let rate = rates.iter().find(|(x, _)| *x == c).map_or(0, |(_, n)| *n);
            (c, points, game.great_person_cost(c), rate)
        })
        .collect();
    rows.sort_by_key(|&(c, points, cost, rate)| (rate == 0, -(points * 1000 / cost.max(1)), c));
    body(ui, 0.0, |ui| {
        cards::grid(ui, 360.0, rows.len(), |ui, i, w| {
            let (c, points, cost, rate) = rows[i];
            let class = &content.great_person_classes[c];
            let next = game.next_great_person(c).unwrap();
            let person = &content.great_people[next];
            let body = format!("Next: {}. {}", person.name, game.great_person_effect(next));
            let district = class
                .district
                .as_deref()
                .and_then(|d| content.districts.iter().find(|x| x.id == d))
                .map(|d| d.name.as_str());
            let footer = match (turns_to(points as f32, cost as f32, rate as f32), district) {
                (Some(t), _) => {
                    format!("{points} / {cost} points, +{rate} per turn (about {t} turns)")
                }
                (None, Some(d)) => {
                    let a = if d.starts_with(['A', 'E', 'I', 'O', 'U']) {
                        "an"
                    } else {
                        "a"
                    };
                    format!("{points} / {cost} points. Build {a} {d} to earn points.")
                }
                (None, None) => format!("{points} / {cost} points."),
            };
            Card::new(&class.name, &body)
                .icon("unit", &class.unit)
                .glyph(Glyph::Star)
                .accent(if rate > 0 { GOLD } else { TEXT_DIM })
                .state(State::Info)
                .footer(&footer)
                .bar(points as f32 / cost.max(1) as f32, GOLD)
                .show(ui, icons, w);
        });
    });
    false
}

// ------------------------------------------------------------ City-States

fn category_yield(category: &str) -> usize {
    match category {
        "SCIENTIFIC" => 3,
        "CULTURAL" => 4,
        "TRADE" => 2,
        "RELIGIOUS" => 5,
        _ => 1,
    }
}

pub fn city_states(s: &mut Session, ui: &mut Ui) -> bool {
    let me = s.me();
    let game = &s.game;
    let icons = &mut s.icons;
    let p = &game.players[me];
    let envoys = p.envoys_available;
    let per_turn = 1 + i32::from(p.government.is_some());
    let text = if envoys > 0 {
        format!(
            "You have {envoys} envoy{} to send. Click a city-state to send one. 1 envoy: +2 of its yield in your capital. 3 and 6: +1 in every city. Most envoys (at least 3): suzerain, +1 more.",
            if envoys == 1 { "" } else { "s" }
        )
    } else {
        format!(
            "No envoys to send right now. You earn one every 30 influence (now {} / 30, +{per_turn} per turn).",
            p.influence
        )
    };
    cards::instruction(ui, &text);
    let met: Vec<PlayerId> = game
        .players
        .iter()
        .filter(|x| x.is_city_state() && p.relations[x.id].met)
        .map(|x| x.id)
        .collect();
    let unmet = game
        .players
        .iter()
        .filter(|x| x.is_city_state() && !p.relations[x.id].met)
        .count();
    let mut send = None;
    body(ui, 0.0, |ui| {
        if met.is_empty() {
            ui.label(
                RichText::new(
                    "You haven't met any city-states yet. Explore with Scouts to find them.",
                )
                .color(TEXT_DIM),
            );
        }
        cards::grid(ui, 360.0, met.len(), |ui, i, w| {
            let cs = &game.players[met[i]];
            let yi = category_yield(&cs.city_state_category);
            let yname = wayciv_core::rules::YIELD_NAMES[yi];
            let mine = p.envoys.get(&cs.id).copied().unwrap_or(0);
            let next = [1, 3, 6].into_iter().find(|&t| mine < t);
            let category = {
                let c = cs.city_state_category.to_lowercase();
                let mut ch = c.chars();
                ch.next()
                    .map(|f| f.to_uppercase().collect::<String>() + ch.as_str())
                    .unwrap_or_default()
            };
            let body = format!(
                "{category} city-state: gives {yname}.\nYour envoys: {mine}{}",
                next.map(|t| format!(" (next bonus at {t})"))
                    .unwrap_or_default()
            );
            let (corner, corner_color) = match game.suzerain(cs.id) {
                Some(x) if x == me => ("Suzerain: you".to_string(), GOOD),
                Some(x) => (format!("Suzerain: {}", game.players[x].civ_name), TEXT_DIM),
                None => ("No suzerain".to_string(), TEXT_DIM),
            };
            let can = envoys > 0 && cs.alive;
            let footer = if !cs.alive {
                "Conquered"
            } else if can {
                "Click to send an envoy"
            } else {
                ""
            };
            let mut card = Card::new(&cs.name, &body)
                .glyph(super::theme::yield_glyph(yi))
                .accent(YIELD_COLORS[yi])
                .corner(&corner, corner_color)
                .state(if can { State::Open } else { State::Info });
            if !footer.is_empty() {
                card = card.footer(footer);
            }
            if card.show(ui, icons, w).clicked() {
                send = Some(cs.id);
            }
        });
        if unmet > 0 {
            ui.label(
                RichText::new(format!(
                    "{unmet} city-states not met yet. Explore to find them."
                ))
                .color(TEXT_DIM),
            );
        }
    });
    if let Some(cs) = send {
        match s.game.send_envoy(me, cs) {
            Ok(()) => {
                let name = s.game.players[cs].name.clone();
                s.toast(format!("Envoy sent to {name}."));
                // Done once every envoy has been placed.
                return s.game.players[me].envoys_available == 0;
            }
            Err(e) => s.toast(e),
        }
    }
    false
}

// ----------------------------------------------------------------- Victory

pub fn victory(s: &mut Session, ui: &mut Ui) -> bool {
    let me = s.me();
    let game = &s.game;
    cards::instruction(
        ui,
        "Win in any one of these ways. Each bar shows how close you are.",
    );
    for v in game.victory_progress(me) {
        let (g, color) = match v.name {
            "Science" => (Glyph::Science, YIELD_COLORS[3]),
            "Culture" => (Glyph::Tourism, YIELD_COLORS[4]),
            "Domination" => (Glyph::Swords, palette::BAD),
            "Religion" => (Glyph::Faith, YIELD_COLORS[5]),
            _ => (Glyph::Cup, GOLD),
        };
        cards::progress(
            ui,
            g,
            color,
            v.name,
            &format!("{:.0}%", v.frac * 100.0),
            v.frac,
            &v.detail,
        );
    }
    cards::section(ui, "Scores", "civilizations you have met");
    let mut players: Vec<&wayciv_core::state::Player> = game
        .alive_majors()
        .filter(|x| x.id == me || game.players[me].relations[x.id].met)
        .collect();
    players.sort_by_key(|x| std::cmp::Reverse(game.score(x.id)));
    let top = players.first().map_or(1, |x| game.score(x.id).max(1));
    for x in players {
        let score = game.score(x.id);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            ui.painter()
                .circle_filled(rect.center(), 6.0, rgb(x.primary));
            let name = if x.id == me {
                format!("{} (you)", x.civ_name)
            } else {
                format!("{} ({})", x.civ_name, x.name)
            };
            let (r, _) = ui.allocate_exact_size(vec2(260.0, 20.0), Sense::hover());
            let name = super::trackers::fit(ui.painter(), &name, 15.5, 256.0);
            super::theme::text(
                ui.painter(),
                r.left_center(),
                egui::Align2::LEFT_CENTER,
                name,
                15.5,
                TEXT,
            );
            let (bar, _) =
                ui.allocate_exact_size(vec2(ui.available_width() - 60.0, 8.0), Sense::hover());
            let key = egui::Id::unique(("victory_score", x.id));
            super::theme::bar_eased(
                ui.painter(),
                key.with("bar"),
                bar,
                score as f32 / top as f32,
                rgb(x.primary).gamma_multiply(1.3),
            );
            // The score ticks up to its new value.
            let shown = super::anim::smooth(ui.ctx(), key, score as f32, 6.0);
            ui.label(RichText::new((shown.round() as i32).to_string()).color(GOLD));
        });
    }
    false
}

// ------------------------------------------------------------ Policy cards

/// The policy card offer: keep one of three. Closes once a card is kept.
pub fn policy_cards(s: &mut Session, ui: &mut Ui) -> bool {
    let me = s.me();
    let game = &s.game;
    let icons = &mut s.icons;
    let content = &game.rules.content;
    let p = &game.players[me];
    let draft = p.cards.draft.clone();
    let cost = game.inspiration_cost(me);
    let culture = game.income(me).yields[4];
    if draft.is_empty() {
        cards::instruction(
            ui,
            "No cards on offer right now. Culture fills the meter below; when it is full, you choose a new card.",
        );
    } else {
        cards::instruction(
            ui,
            "Choose one card to keep. You can slot kept cards in Government (G); the others may be offered again later.",
        );
    }
    let mut picked = None;
    body(ui, 20.0, |ui| {
        if !draft.is_empty() {
            cards::section(ui, "On offer", "click one to keep it");
            cards::grid(ui, 280.0, draft.len(), |ui, i, w| {
                let q = draft[i];
                let pol = &content.policies[q];
                let replaces: Vec<&str> = content
                    .policies
                    .iter()
                    .enumerate()
                    .filter(|(k, x)| {
                        p.cards.adopted[*k] && x.obsoleted_by.as_deref() == Some(pol.id.as_str())
                    })
                    .map(|(_, x)| x.name.as_str())
                    .collect();
                let footer = if replaces.is_empty() {
                    format!("{} policy · since {}", slot_name(&pol.slot), year_label(pol.unlock_year))
                } else {
                    format!("{} policy · replaces {}", slot_name(&pol.slot), replaces.join(", "))
                };
                let card = Card::new(&pol.name, &pol.description)
                    .glyph(slot_glyph(&pol.slot))
                    .accent(slot_color(&pol.slot))
                    .footer(&footer)
                    .corner("Keep", palette::TEAL)
                    .state(State::Open);
                if card.show(ui, icons, w).clicked() {
                    picked = Some(q);
                }
            });
        }
        cards::section(ui, "Next card", "Culture fills the meter");
        let have = p.cards.inspiration.min(cost);
        let hint = if have >= cost {
            "Full: the next offer comes as soon as you keep a card.".to_string()
        } else {
            match turns_to(have as f32, cost as f32, culture) {
                Some(t) => format!("About {t} turns at {culture:.1} Culture per turn."),
                None => "Earn Culture to fill it.".into(),
            }
        };
        cards::progress(
            ui,
            Glyph::Culture,
            YIELD_COLORS[4],
            "Inspiration",
            &format!("{have} / {cost}"),
            have as f32 / cost.max(1) as f32,
            &hint,
        );
        let kept = p.cards.count();
        let waiting = game.draftable_policies(me).len();
        ui.label(
            RichText::new(format!(
                "You have kept {kept} card{}. {waiting} unlocked card{} you haven't kept can still be offered; the calendar unlocks more.",
                if kept == 1 { "" } else { "s" },
                if waiting == 1 { "" } else { "s" }
            ))
            .color(TEXT_DIM),
        );
    });
    if let Some(q) = picked {
        let r = s.game.adopt_policy(me, q);
        s.result(r);
        return s.game.players[me].cards.draft.is_empty();
    }
    false
}

// -------------------------------------------------------------------- Eras

fn moment_kind(kind: &str) -> &'static str {
    match kind {
        "eureka" => "Eureka",
        "great_person" => "great person recruited",
        "district" => "district completed",
        "trade_route" => "trade route started",
        "kill" => "enemy unit defeated",
        "city" => "city founded",
        _ => "moment",
    }
}

/// What an age means, in a sentence.
fn age_text(age: wayciv_core::eras::Age) -> &'static str {
    use wayciv_core::eras::Age;
    match age {
        Age::Golden => "Your dedication's bonus applies, and every city gains +3 Loyalty per turn. Golden Ages are harder to keep: the next era needs 3 more era score.",
        Age::Heroic => "A comeback from a Dark Age: choose two dedications, whose bonuses apply, and every city gains +3 Loyalty per turn.",
        Age::Dark => "Cities lose 3 Loyalty per turn. Your dedication earns +1 era score for each of its moments, and the next Golden Age needs 3 less.",
        Age::Normal => "Your dedication earns +1 era score for each of its moments. Earn enough era score this era to reach a Golden Age.",
    }
}

/// Eras (T): your age, era score toward the next one, dedications, and the
/// timeline of historic moments. Closes once the last dedication is chosen.
pub fn eras(s: &mut Session, ui: &mut Ui) -> bool {
    let me = s.me();
    let game = &s.game;
    let icons = &mut s.icons;
    let content = &game.rules.content;
    let e = &game.players[me].era;
    let era_name = game.era_name(game.world_era).to_string();
    let next_era = content.eras.get(game.world_era + 1);
    let pending = e.pending;
    let text = if pending > 0 {
        format!(
            "The {era_name} has begun: you are in a {}. Choose {} dedication{} for this era.",
            e.age.name(),
            pending,
            if pending > 1 { "s" } else { "" }
        )
    } else {
        "Historic moments earn era score. Your score this era decides your age in the next one.".to_string()
    };
    cards::instruction(ui, &text);
    let mut chosen = None;
    body(ui, 20.0, |ui| {
        // The age banner.
        ui.horizontal(|ui| {
            super::theme::tag(ui, e.age.name(), palette::age_color(e.age));
            ui.label(RichText::new(&era_name).size(20.0).color(TEXT));
        });
        ui.label(RichText::new(age_text(e.age)).color(TEXT_DIM));
        ui.add_space(6.0);

        // Dedications.
        let golden = e.age.is_golden();
        if pending > 0 {
            cards::section(ui, "Dedications", "click to choose");
            let options = game.dedication_options(me);
            cards::grid(ui, 270.0, options.len(), |ui, i, w| {
                let d = &content.dedications[options[i]];
                let body = if golden {
                    d.description.clone()
                } else {
                    format!(
                        "+1 era score for each {}.\nIn a Golden Age: {}",
                        moment_kind(&d.moment),
                        d.description
                    )
                };
                let card = Card::new(&d.name, &body)
                    .glyph(Glyph::Hourglass)
                    .accent(palette::age_color(e.age))
                    .corner("Dedicate", palette::TEAL)
                    .state(State::Open);
                if card.show(ui, icons, w).clicked() {
                    chosen = Some(options[i]);
                }
            });
        } else if !e.dedications.is_empty() {
            cards::section(ui, "Your dedication", if golden { "its bonus applies" } else { "earns era score" });
            cards::grid(ui, 270.0, e.dedications.len(), |ui, i, w| {
                let d = &content.dedications[e.dedications[i]];
                let body = if golden {
                    d.description.clone()
                } else {
                    format!("+1 era score for each {}.", moment_kind(&d.moment))
                };
                Card::new(&d.name, &body)
                    .glyph(Glyph::Hourglass)
                    .accent(palette::age_color(e.age))
                    .state(State::Chosen)
                    .show(ui, icons, w);
            });
        }

        // Era score toward the next age.
        let (dark, gold_at) = game.era_thresholds(me);
        let projected = game.projected_age(me);
        let when = next_era.map_or("by the end of the game".to_string(), |n| {
            format!("when the {} begins ({}, turn {})", n.name, year_label(n.start_year), game.turn_of_year(n.start_year))
        });
        cards::section(ui, "Era score", &when);
        cards::progress(
            ui,
            Glyph::Star,
            palette::age_color(projected),
            "This era",
            &format!("{} (Dark Age below {dark}, Golden Age at {gold_at})", e.score),
            e.score as f32 / gold_at.max(1) as f32,
            &format!("Heading for a {}.", projected.name()),
        );

        // The timeline.
        if !e.past.is_empty() {
            cards::section(ui, "Past eras", "score earned, and the age it led to");
            ui.horizontal_wrapped(|ui| {
                for &(era, score, age) in &e.past {
                    super::theme::tag(
                        ui,
                        &format!("{}: {score} → {}", game.era_name(era), age.name()),
                        palette::age_color(age),
                    );
                }
            });
        }
        cards::section(ui, "Historic moments", "newest first");
        let mut last_era = usize::MAX;
        for m in e.moments.iter().rev().take(200) {
            if m.era != last_era {
                last_era = m.era;
                ui.label(RichText::new(game.era_name(m.era)).strong().color(GOLD));
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Turn {:>3}", m.turn)).monospace().color(TEXT_DIM));
                let score = if m.score > 0 {
                    RichText::new(format!("+{}", m.score)).color(GOOD)
                } else {
                    RichText::new("   ").color(TEXT_DIM)
                };
                ui.label(score);
                ui.label(RichText::new(&m.text).color(TEXT));
            });
        }
        if e.moments.is_empty() {
            ui.label(RichText::new("No moments yet. Found cities, meet other civilizations, explore.").color(TEXT_DIM));
        }
    });
    if let Some(d) = chosen {
        let r = s.game.choose_dedication(me, d);
        s.result(r);
        return s.game.players[me].era.pending == 0;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, PointerButton, Pos2, RawInput, Rect, pos2};

    fn session() -> Option<Session> {
        let rules = wayciv_core::load_rules().ok()?;
        let game = wayciv_core::new_game(
            rules,
            wayciv_core::Settings {
                seed: 3,
                ..Default::default()
            },
        );
        Some(Session::new(game))
    }

    /// Runs one frame of `menu` on a 1400x3000 screen (tall enough that
    /// nothing scrolls) with `events`;
    /// returns what the menu returned.
    fn frame(
        ctx: &egui::Context,
        s: &mut Session,
        menu: fn(&mut Session, &mut Ui) -> bool,
        events: Vec<Event>,
    ) -> bool {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1400.0, 3000.0))),
            events,
            ..Default::default()
        };
        let mut done = false;
        let mut output = ctx.run_ui(input, |ui| {
            egui::CentralPanel::default().show(ui, |ui| done = menu(s, ui));
        });
        // Icon textures are uploaded through the output; a test has no
        // renderer to hand them to.
        output.textures_delta.clear();
        done
    }

    /// Clicks at `at`: move there, press, release. True if the menu said
    /// it was done on any of those frames.
    fn click(
        ctx: &egui::Context,
        s: &mut Session,
        menu: fn(&mut Session, &mut Ui) -> bool,
        at: Pos2,
    ) -> bool {
        let button = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        frame(ctx, s, menu, vec![Event::PointerMoved(at)])
            | frame(ctx, s, menu, vec![button(true)])
            | frame(ctx, s, menu, vec![button(false)])
    }

    #[test]
    fn picking_a_pantheon_closes_the_religion_menu() {
        let Some(mut s) = session() else {
            eprintln!("skipped: content failed to load");
            return;
        };
        let me = s.me();
        let ctx = egui::Context::default();
        crate::palette::style(&ctx);
        // Without enough Faith the cards are locked: clicks do nothing.
        frame(&ctx, &mut s, religion, vec![]);
        for y in (150..700).step_by(25) {
            assert!(!click(&ctx, &mut s, religion, pos2(200.0, y as f32)));
        }
        assert!(s.game.players[me].pantheon.is_none());
        // With 25 Faith, clicking a card picks it and the menu is done.
        s.game.players[me].faith = 30;
        frame(&ctx, &mut s, religion, vec![]);
        let done = (150..700)
            .step_by(25)
            .any(|y| click(&ctx, &mut s, religion, pos2(200.0, y as f32)));
        assert!(done, "a pantheon card click should close the menu");
        assert!(s.game.players[me].pantheon.is_some());
        assert_eq!(s.game.players[me].faith, 30 - PANTHEON_COST);
    }

    #[test]
    fn founding_a_religion_takes_three_picks() {
        let Some(mut s) = session() else {
            eprintln!("skipped: content failed to load");
            return;
        };
        let me = s.me();
        // Found the capital, give it a Holy Site and a Great Prophet.
        let settler = s.selected_unit.unwrap();
        s.game.found_city(settler).unwrap();
        let capital = s.game.player_cities(me).next().unwrap().pos;
        let site = s
            .game
            .map
            .grid
            .neighbors(capital)
            .find(|&p| s.game.map.tile(p).owner == Some(me))
            .unwrap();
        let holy = s.game.rules.idx("DISTRICT_HOLY_SITE").unwrap();
        s.game.map.tile_mut(site).district = Some(holy);
        let pantheon = s.game.beliefs_of_class("BELIEF_CLASS_PANTHEON")[0];
        s.game.players[me].pantheon = Some(pantheon);
        let prophet = s.game.spawn(me, "UNIT_GREAT_PROPHET", site).unwrap();
        s.windows.found = Some(FoundDraft::new(prophet));

        let ctx = egui::Context::default();
        crate::palette::style(&ctx);
        frame(&ctx, &mut s, religion, vec![]);
        // Click down the left column: a name pill, then a card in each
        // belief grid; the Found button sits below them all.
        let mut done = false;
        for y in (100..3000).step_by(12) {
            if click(&ctx, &mut s, religion, pos2(60.0, y as f32)) {
                done = true;
                break;
            }
        }
        assert!(
            done,
            "the Found button should end the menu: {:?}",
            s.windows.found
        );
        assert!(s.game.players[me].religion.is_some());
        assert_eq!(s.game.players[me].beliefs.len(), 2);
    }

    #[test]
    fn keeping_a_policy_card_closes_the_offer() {
        let Some(mut s) = session() else {
            eprintln!("skipped: content failed to load");
            return;
        };
        let me = s.me();
        assert!(!s.game.players[me].cards.draft.is_empty());
        let ctx = egui::Context::default();
        crate::palette::style(&ctx);
        frame(&ctx, &mut s, policy_cards, vec![]);
        let done = (100..700)
            .step_by(20)
            .any(|y| click(&ctx, &mut s, policy_cards, pos2(150.0, y as f32)));
        assert!(done, "a card click should close the offer");
        assert_eq!(s.game.players[me].cards.count(), 1);
        assert!(s.game.players[me].cards.draft.is_empty());
    }

    #[test]
    fn choosing_a_dedication_closes_the_eras_menu() {
        let Some(mut s) = session() else {
            eprintln!("skipped: content failed to load");
            return;
        };
        let me = s.me();
        s.game.players[me].era.pending = 1;
        let ctx = egui::Context::default();
        crate::palette::style(&ctx);
        frame(&ctx, &mut s, eras, vec![]);
        let done = (100..900)
            .step_by(20)
            .any(|y| click(&ctx, &mut s, eras, pos2(150.0, y as f32)));
        assert!(done, "a dedication click should close the menu");
        assert_eq!(s.game.players[me].era.dedications.len(), 1);
        assert_eq!(s.game.players[me].era.pending, 0);
    }
}

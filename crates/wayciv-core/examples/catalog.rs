//! Prints WayCiv's whole content catalog as Markdown: civilizations and
//! leaders, units, buildings, wonders, districts, technologies,
//! governments, policies, great people, religion, improvements, resources,
//! terrain, city-states, and projects. Year-gated items show the turn their
//! year arrives in a 500-turn game.
//!
//! cargo run -p wayciv-core --example catalog > WayCiv-Content.md

use std::fmt::Write;
use wayciv_core::length::year_label;
use wayciv_core::{GameLength, Settings, load_rules, new_game};
use wayciv_data::content::{Content, Yields};
use wayciv_data::effect::words;

const YIELD_WORDS: [&str; 6] = ["Food", "Production", "Gold", "Science", "Culture", "Faith"];

fn yields(y: &Yields) -> String {
    let parts: Vec<String> = y
        .iter()
        .enumerate()
        .filter(|(_, n)| **n != 0)
        .map(|(i, n)| format!("{:+} {}", n, YIELD_WORDS[i]))
        .collect();
    if parts.is_empty() {
        "—".into()
    } else {
        parts.join(", ")
    }
}

fn amenities(n: i32) -> String {
    format!("+{n} {}", if n == 1 { "Amenity" } else { "Amenities" })
}

/// Table cells can't hold newlines or pipes.
fn cell(s: &str) -> String {
    s.replace('\n', " ").replace('|', "/")
}

fn main() {
    let rules = load_rules().expect("content loads");
    let game = new_game(
        rules.clone(),
        Settings {
            length: GameLength::Turns500,
            ..Settings::default()
        },
    );
    let c: &Content = &rules.content;
    let name = |id: &str| c.name_of(id);
    let when = |year: i32| format!("{} (turn {})", year_label(year), game.turn_of_year(year));
    let unlock = |tech: &Option<String>, year: Option<i32>| match (tech, year) {
        (Some(t), _) => name(t),
        (None, Some(y)) => when(y),
        (None, None) => "Start".into(),
    };
    let era_of_tech = |tech: &Option<String>, year: Option<i32>| -> usize {
        tech.as_deref()
            .and_then(|t| c.techs.iter().find(|x| x.id == t))
            .map(|t| rules.era_index(&t.era))
            .or_else(|| year.map(|y| rules.era_index(&rules.era_of_year(y))))
            .unwrap_or(0)
    };
    let mut out = String::new();
    let o = &mut out;

    let wonders = c.buildings.iter().filter(|b| b.wonder).count();
    let regular = c
        .buildings
        .iter()
        .filter(|b| !b.wonder && b.unique_to.is_none())
        .count();
    writeln!(o, "# WayCiv Content Catalog\n").unwrap();
    writeln!(
        o,
        "Everything in WayCiv is its own data, compiled into the game from `crates/wayciv-data/data/*.toml`; no Civilization VI install is needed. Real historical leaders, nations, places, and generic unit and building names are kept; every ability, description, stat, agenda, and line of dialogue is WayCiv's own.\n"
    )
    .unwrap();
    writeln!(
        o,
        "**There are no civics.** Governments, policies, and everything civics used to unlock (some units, buildings, wonders, districts, and improvements) unlock when the calendar reaches a year. The calendar runs from 4000 BC to about 2150 AD, stretched to the game's length and paced to match typical research; turns below are for a 500-turn game. When governments or policies unlock, every civilization gets a free policy change and an envoy. Culture now grows borders and drives tourism (the culture victory).\n"
    )
    .unwrap();
    writeln!(o, "| | Count |\n|---|---|").unwrap();
    for (what, n) in [
        ("Civilizations", c.civilizations.len()),
        ("Leaders", c.leaders.len()),
        ("Units", c.units.len()),
        ("Buildings (standard)", regular),
        ("Wonders", wonders),
        (
            "Districts",
            c.districts.iter().filter(|d| !d.internal_only).count(),
        ),
        ("Technologies", c.techs.len()),
        ("Governments", c.governments.len()),
        ("Policies", c.policies.len()),
        ("Great people", c.great_people.len()),
        ("Beliefs", c.beliefs.len()),
        ("Improvements", c.improvements.len()),
        ("Resources", c.resources.len()),
        ("City-states", c.city_states.len()),
    ] {
        writeln!(o, "| {what} | {n} |").unwrap();
    }
    writeln!(o, "\n## Contents\n").unwrap();
    for s in [
        "Calendar and eras",
        "Civilizations and leaders",
        "Units",
        "Buildings",
        "Wonders",
        "Districts",
        "Technologies",
        "Governments",
        "Policies",
        "Great people",
        "Religion",
        "Improvements",
        "Resources",
        "Terrain and natural wonders",
        "City-states",
        "Projects",
    ] {
        let anchor = s.to_lowercase().replace(' ', "-");
        writeln!(o, "- [{s}](#{anchor})").unwrap();
    }

    // ------------------------------------------------------------ Calendar
    writeln!(o, "\n## Calendar and eras\n").unwrap();
    writeln!(
        o,
        "| Era | Begins | Turn (500-turn game) | Great person base cost |\n|---|---|---|---|"
    )
    .unwrap();
    for e in &c.eras {
        writeln!(
            o,
            "| {} | {} | {} | {} |",
            e.name,
            year_label(e.start_year),
            game.turn_of_year(e.start_year),
            e.great_person_base_cost
        )
        .unwrap();
    }
    writeln!(o, "\n| Turn | Year |\n|---|---|").unwrap();
    for t in [1, 50, 75, 100, 150, 200, 250, 300, 350, 400, 450, 500] {
        writeln!(o, "| {t} | {} |", year_label(game.calendar_year(t))).unwrap();
    }

    // ------------------------------------------------------- Civilizations
    writeln!(o, "\n## Civilizations and leaders\n").unwrap();
    for civ in &c.civilizations {
        writeln!(o, "### {}\n", civ.name).unwrap();
        for t in &civ.traits {
            writeln!(o, "**{}** — {}\n", t.name, cell(&t.description)).unwrap();
        }
        let leaders: Vec<_> = c
            .leaders
            .iter()
            .filter(|l| l.civilization == civ.id)
            .collect();
        let owners: Vec<&str> = std::iter::once(civ.id.as_str())
            .chain(leaders.iter().map(|l| l.id.as_str()))
            .collect();
        let mut uniques: Vec<String> = Vec::new();
        for u in c
            .units
            .iter()
            .filter(|u| u.unique_to.as_deref().is_some_and(|x| owners.contains(&x)))
        {
            let strength = if u.ranged_combat > 0 {
                format!("{} / {} ranged", u.combat, u.ranged_combat)
            } else if u.bombard > 0 {
                format!("{} / {} bombard", u.combat, u.bombard)
            } else {
                format!("{}", u.combat)
            };
            uniques.push(format!(
                "Unit **{}**{}{}: strength {strength}, {} moves, {} Production; unlocks with {}. {}",
                u.name,
                u.replaces.as_deref().map(|r| format!(" (replaces {})", name(r))).unwrap_or_default(),
                if u.unique_to.as_deref() != Some(civ.id.as_str()) {
                    format!(" — {} only", name(u.unique_to.as_deref().unwrap_or_default()))
                } else {
                    String::new()
                },
                u.moves,
                u.cost,
                unlock(&u.prereq_tech, u.unlock_year),
                u.description
            ));
        }
        for b in c
            .buildings
            .iter()
            .filter(|b| b.unique_to.as_deref().is_some_and(|x| owners.contains(&x)))
        {
            uniques.push(format!(
                "Building **{}**{}: {}{}; {} Production; unlocks with {}. {}",
                b.name,
                b.replaces
                    .as_deref()
                    .map(|r| format!(" (replaces {})", name(r)))
                    .unwrap_or_default(),
                yields(&b.yields),
                if b.housing > 0 {
                    format!(", +{} Housing", b.housing)
                } else {
                    String::new()
                },
                b.cost,
                unlock(&b.prereq_tech, b.unlock_year),
                cell(&b.description)
            ));
        }
        for d in c
            .districts
            .iter()
            .filter(|d| d.unique_to.as_deref().is_some_and(|x| owners.contains(&x)))
        {
            let adj: Vec<String> = d
                .adjacencies
                .iter()
                .map(|a| a.description.clone())
                .collect();
            uniques.push(format!(
                "District **{}**{}: {} Production; unlocks with {}. {}{}",
                d.name,
                d.replaces
                    .as_deref()
                    .map(|r| format!(" (replaces {})", name(r)))
                    .unwrap_or_default(),
                d.cost,
                unlock(&d.prereq_tech, d.unlock_year),
                d.description,
                if adj.is_empty() {
                    String::new()
                } else {
                    format!(" Adjacency: {}.", adj.join("; "))
                }
            ));
        }
        for i in c
            .improvements
            .iter()
            .filter(|i| i.unique_to.as_deref().is_some_and(|x| owners.contains(&x)))
        {
            uniques.push(format!(
                "Improvement **{}**: {}; unlocks with {}. {}",
                i.name,
                yields(&i.yields),
                unlock(&i.prereq_tech, i.unlock_year),
                i.description
            ));
        }
        for u in uniques {
            writeln!(o, "- {u}").unwrap();
        }
        for l in leaders {
            writeln!(o, "\n**Leader: {}** (capital {})\n", l.name, l.capital).unwrap();
            for t in &l.traits {
                writeln!(o, "- Ability — *{}*: {}", t.name, cell(&t.description)).unwrap();
            }
            if let Some((agenda, text)) = &l.agenda {
                writeln!(o, "- Agenda — *{agenda}*: {text}").unwrap();
            }
            if let Some(line) = l.lines.get("first_meet") {
                writeln!(o, "- First meeting: “{line}”").unwrap();
            }
        }
        writeln!(o, "\nCities: {}.\n", civ.city_names.join(", ")).unwrap();
    }

    // --------------------------------------------------------------- Units
    writeln!(o, "## Units\n").unwrap();
    writeln!(
        o,
        "Standard units; unique units are listed with their civilization.\n"
    )
    .unwrap();
    writeln!(o, "| Unit | Era | Unlock | Class | Strength | Ranged / Bombard | Range | Moves | Cost | Needs | Description |\n|---|---|---|---|---|---|---|---|---|---|---|").unwrap();
    let mut units: Vec<_> = c.units.iter().filter(|u| u.unique_to.is_none()).collect();
    units.sort_by_key(|u| (era_of_tech(&u.prereq_tech, u.unlock_year), u.cost));
    for u in units {
        let ranged = if u.ranged_combat > 0 {
            u.ranged_combat.to_string()
        } else if u.bombard > 0 {
            format!("{} (bombard)", u.bombard)
        } else if u.anti_air > 0 {
            format!("{} (anti-air)", u.anti_air)
        } else {
            "—".into()
        };
        writeln!(
            o,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            u.name,
            c.eras[era_of_tech(&u.prereq_tech, u.unlock_year)]
                .name
                .trim_end_matches(" Era"),
            if u.can_train {
                unlock(&u.prereq_tech, u.unlock_year)
            } else {
                "Recruited".into()
            },
            u.promotion_class
                .as_deref()
                .map(words)
                .unwrap_or_else(|| "Civilian".into()),
            if u.combat > 0 {
                u.combat.to_string()
            } else {
                "—".into()
            },
            ranged,
            if u.range > 0 {
                u.range.to_string()
            } else {
                "—".into()
            },
            u.moves,
            if u.can_train {
                u.cost.to_string()
            } else {
                "—".into()
            },
            u.strategic_resource
                .as_deref()
                .map(name)
                .unwrap_or_else(|| "—".into()),
            cell(&u.description)
        )
        .unwrap();
    }

    // ----------------------------------------------------------- Buildings
    writeln!(o, "\n## Buildings\n").unwrap();
    for d in c
        .districts
        .iter()
        .filter(|d| d.unique_to.is_none() && !d.internal_only)
    {
        let list: Vec<_> = c
            .buildings
            .iter()
            .filter(|b| !b.wonder && b.unique_to.is_none())
            .filter(|b| {
                b.prereq_district.as_deref() == Some(d.id.as_str())
                    || (d.city_center && b.prereq_district.is_none())
            })
            .collect();
        if list.is_empty() {
            continue;
        }
        writeln!(o, "### {}\n", d.name).unwrap();
        writeln!(o, "| Building | Unlock | Cost | Yields | Other | Requires | Description |\n|---|---|---|---|---|---|---|").unwrap();
        for b in list {
            let mut other = Vec::new();
            if b.housing != 0 {
                other.push(format!("+{} Housing", b.housing));
            }
            if b.amenities != 0 {
                other.push(amenities(b.amenities));
            }
            if b.outer_defense != 0 {
                other.push(format!("+{} defense", b.outer_defense));
            }
            if b.trade_capacity != 0 {
                other.push(format!("+{} trade route", b.trade_capacity));
            }
            for (g, n) in &b.great_person_points {
                other.push(format!("+{n} {} point", name(g)));
            }
            if b.maintenance != 0 {
                other.push(format!("{} Gold upkeep", b.maintenance));
            }
            writeln!(
                o,
                "| {} | {} | {} | {} | {} | {} | {} |",
                b.name,
                unlock(&b.prereq_tech, b.unlock_year),
                b.cost,
                yields(&b.yields),
                if other.is_empty() {
                    "—".into()
                } else {
                    other.join(", ")
                },
                if b.prereq_buildings.is_empty() {
                    "—".into()
                } else {
                    b.prereq_buildings
                        .iter()
                        .map(|x| name(x))
                        .collect::<Vec<_>>()
                        .join(" or ")
                },
                cell(&b.description)
            )
            .unwrap();
        }
        writeln!(o).unwrap();
    }

    // ------------------------------------------------------------- Wonders
    writeln!(o, "## Wonders\n").unwrap();
    writeln!(
        o,
        "One of each per world. Yields go to the city; effects apply across the builder's empire.\n"
    )
    .unwrap();
    writeln!(
        o,
        "| Wonder | Unlock | Cost | City yields | Needs | Effect |\n|---|---|---|---|---|---|"
    )
    .unwrap();
    let mut wonders: Vec<_> = c.buildings.iter().filter(|b| b.wonder).collect();
    wonders.sort_by_key(|w| (era_of_tech(&w.prereq_tech, w.unlock_year), w.cost));
    for w in wonders {
        let needs: Vec<String> = w
            .prereq_district
            .iter()
            .map(|d| name(d))
            .chain(w.prereq_buildings.iter().map(|b| name(b)))
            .collect();
        writeln!(
            o,
            "| {} | {} | {} | {} | {} | {} |",
            w.name,
            unlock(&w.prereq_tech, w.unlock_year),
            w.cost,
            yields(&w.yields),
            if needs.is_empty() {
                "—".into()
            } else {
                needs.join(", ")
            },
            cell(&w.description)
        )
        .unwrap();
    }

    // ----------------------------------------------------------- Districts
    writeln!(o, "\n## Districts\n").unwrap();
    writeln!(o, "| District | Unlock | Cost | Specialty | Adjacency | Other | Description |\n|---|---|---|---|---|---|---|").unwrap();
    for d in c
        .districts
        .iter()
        .filter(|d| d.unique_to.is_none() && !d.internal_only)
    {
        let adj: Vec<String> = d
            .adjacencies
            .iter()
            .map(|a| a.description.clone())
            .collect();
        let mut other = Vec::new();
        if d.housing != 0 {
            other.push(format!("+{} Housing", d.housing));
        }
        if d.amenities != 0 {
            other.push(amenities(d.amenities));
        }
        if d.trade_capacity != 0 {
            other.push(format!("+{} trade route", d.trade_capacity));
        }
        for (g, n) in &d.great_person_points {
            other.push(format!("+{n} {} point", name(g)));
        }
        writeln!(
            o,
            "| {} | {} | {} | {} | {} | {} | {} |",
            d.name,
            unlock(&d.prereq_tech, d.unlock_year),
            d.cost,
            if d.requires_population { "yes" } else { "no" },
            if adj.is_empty() {
                "—".into()
            } else {
                adj.join("; ")
            },
            if other.is_empty() {
                "—".into()
            } else {
                other.join(", ")
            },
            cell(&d.description)
        )
        .unwrap();
    }

    // -------------------------------------------------------- Technologies
    writeln!(o, "\n## Technologies\n").unwrap();
    writeln!(
        o,
        "Costs are base Science before game-length scaling. A Eureka gives 40% of the cost.\n"
    )
    .unwrap();
    for e in &c.eras {
        let techs: Vec<_> = c.techs.iter().filter(|t| t.era == e.id).collect();
        if techs.is_empty() {
            continue;
        }
        writeln!(o, "### {}\n", e.name).unwrap();
        writeln!(
            o,
            "| Technology | Cost | Requires | Eureka | Unlocks | Notes |\n|---|---|---|---|---|---|"
        )
        .unwrap();
        for t in techs {
            let unlocks: Vec<String> = c
                .units
                .iter()
                .filter(|u| {
                    u.unique_to.is_none() && u.prereq_tech.as_deref() == Some(t.id.as_str())
                })
                .map(|u| u.name.clone())
                .chain(
                    c.buildings
                        .iter()
                        .filter(|b| {
                            b.unique_to.is_none() && b.prereq_tech.as_deref() == Some(t.id.as_str())
                        })
                        .map(|b| b.name.clone()),
                )
                .chain(
                    c.districts
                        .iter()
                        .filter(|d| {
                            d.unique_to.is_none() && d.prereq_tech.as_deref() == Some(t.id.as_str())
                        })
                        .map(|d| d.name.clone()),
                )
                .chain(
                    c.improvements
                        .iter()
                        .filter(|i| {
                            i.unique_to.is_none() && i.prereq_tech.as_deref() == Some(t.id.as_str())
                        })
                        .map(|i| i.name.clone()),
                )
                .chain(
                    c.projects
                        .iter()
                        .filter(|p| p.prereq_tech.as_deref() == Some(t.id.as_str()))
                        .map(|p| p.name.clone()),
                )
                .collect();
            writeln!(
                o,
                "| {} | {} | {} | {} | {} | {} |",
                t.name,
                t.cost,
                if t.prereqs.is_empty() {
                    "—".into()
                } else {
                    t.prereqs
                        .iter()
                        .map(|p| name(p))
                        .collect::<Vec<_>>()
                        .join(", ")
                },
                t.boost.as_deref().unwrap_or("—"),
                if unlocks.is_empty() {
                    "—".into()
                } else {
                    unlocks.join(", ")
                },
                cell(&t.description)
            )
            .unwrap();
        }
        writeln!(o).unwrap();
    }
    writeln!(o, "### Artificial Intelligence tree\n").unwrap();
    writeln!(
        o,
        "Opened by the Artificial Intelligence tech and researched with Compute from Datacenters.\n"
    )
    .unwrap();
    writeln!(
        o,
        "| Research | Compute | Requires | Effect |\n|---|---|---|---|"
    )
    .unwrap();
    for t in wayciv_core::additions::AI_TREE.iter() {
        let reqs: Vec<&str> = t
            .prereqs
            .iter()
            .filter_map(|p| {
                wayciv_core::additions::AI_TREE
                    .iter()
                    .find(|x| x.id == *p)
                    .map(|x| x.name)
            })
            .collect();
        writeln!(
            o,
            "| {} | {} | {} | {} |",
            t.name,
            t.cost,
            if reqs.is_empty() {
                "Artificial Intelligence".into()
            } else {
                reqs.join(", ")
            },
            t.description
        )
        .unwrap();
    }

    // --------------------------------------------------------- Governments
    writeln!(o, "\n## Governments\n").unwrap();
    writeln!(
        o,
        "| Government | Unlocks | Policy slots | Effects |\n|---|---|---|---|"
    )
    .unwrap();
    let mut govs: Vec<_> = c.governments.iter().collect();
    govs.sort_by_key(|g| g.unlock_year);
    for g in govs {
        let slots: Vec<String> = g
            .slots
            .iter()
            .map(|(s, n)| format!("{n} {}", words(s)))
            .collect();
        writeln!(
            o,
            "| {} | {} | {} | {} |",
            g.name,
            when(g.unlock_year),
            slots.join(", "),
            cell(&g.bonus)
        )
        .unwrap();
    }

    // ------------------------------------------------------------ Policies
    writeln!(o, "\n## Policies\n").unwrap();
    writeln!(
        o,
        "A policy leaves the list once the policy that replaces it unlocks.\n"
    )
    .unwrap();
    for slot in [
        "SLOT_MILITARY",
        "SLOT_ECONOMIC",
        "SLOT_DIPLOMATIC",
        "SLOT_GREAT_PERSON",
    ] {
        writeln!(o, "### {}\n", words(slot)).unwrap();
        writeln!(
            o,
            "| Policy | Unlocks | Effects | Replaced by |\n|---|---|---|---|"
        )
        .unwrap();
        let mut list: Vec<_> = c.policies.iter().filter(|p| p.slot == slot).collect();
        list.sort_by_key(|p| p.unlock_year);
        for p in list {
            writeln!(
                o,
                "| {} | {} | {} | {} |",
                p.name,
                when(p.unlock_year),
                cell(&p.description),
                p.obsoleted_by
                    .as_deref()
                    .map(name)
                    .unwrap_or_else(|| "—".into())
            )
            .unwrap();
        }
        writeln!(o).unwrap();
    }

    // -------------------------------------------------------- Great people
    writeln!(o, "## Great people\n").unwrap();
    writeln!(
        o,
        "Each class recruits its people in era order; the class decides what activating one does.\n"
    )
    .unwrap();
    for class in &c.great_person_classes {
        let people: Vec<_> = c
            .great_people
            .iter()
            .filter(|g| g.class == class.id)
            .collect();
        writeln!(
            o,
            "### {} ({}; points from {})\n",
            class.name,
            people.len(),
            class.district.as_deref().map(name).unwrap_or_default()
        )
        .unwrap();
        writeln!(o, "| Name | Era | Known for |\n|---|---|---|").unwrap();
        for g in people {
            writeln!(
                o,
                "| {} | {} | {} |",
                g.name,
                name(&g.era).trim_end_matches(" Era"),
                cell(&g.ability)
            )
            .unwrap();
        }
        writeln!(o).unwrap();
    }

    // ------------------------------------------------------------ Religion
    writeln!(o, "## Religion\n").unwrap();
    let religions: Vec<&str> = c
        .religions
        .iter()
        .filter(|r| !r.pantheon)
        .map(|r| r.name.as_str())
        .collect();
    writeln!(
        o,
        "Religions that can be founded: {}.\n",
        religions.join(", ")
    )
    .unwrap();
    for (class, title) in [
        ("BELIEF_CLASS_PANTHEON", "Pantheons (25 Faith)"),
        ("BELIEF_CLASS_FOUNDER", "Founder beliefs"),
        ("BELIEF_CLASS_FOLLOWER", "Follower beliefs"),
    ] {
        writeln!(o, "### {title}\n").unwrap();
        writeln!(o, "| Belief | Effect |\n|---|---|").unwrap();
        for b in c.beliefs.iter().filter(|b| b.class == class) {
            writeln!(o, "| {} | {} |", b.name, cell(&b.description)).unwrap();
        }
        writeln!(o).unwrap();
    }

    // -------------------------------------------------------- Improvements
    writeln!(o, "## Improvements\n").unwrap();
    writeln!(
        o,
        "| Improvement | Unlock | Yields | Built on | Description |\n|---|---|---|---|---|"
    )
    .unwrap();
    for i in c.improvements.iter().filter(|i| i.unique_to.is_none()) {
        let on: Vec<String> = i
            .valid_resources
            .iter()
            .chain(&i.valid_features)
            .map(|x| name(x))
            .collect();
        let on = if !on.is_empty() {
            on.join(", ")
        } else if i.valid_terrains.is_empty() {
            "—".into()
        } else {
            "open land".into()
        };
        writeln!(
            o,
            "| {} | {} | {} | {} | {} |",
            i.name,
            if i.buildable {
                unlock(&i.prereq_tech, i.unlock_year)
            } else {
                "Placed by the map".into()
            },
            yields(&i.yields),
            on,
            i.description
        )
        .unwrap();
    }

    // ----------------------------------------------------------- Resources
    writeln!(o, "\n## Resources\n").unwrap();
    writeln!(
        o,
        "| Resource | Kind | Yields | Revealed by |\n|---|---|---|---|"
    )
    .unwrap();
    for r in &c.resources {
        writeln!(
            o,
            "| {} | {} | {} | {} |",
            r.name,
            r.class.trim_start_matches("RESOURCECLASS_").to_lowercase(),
            yields(&r.yields),
            r.prereq_tech
                .as_deref()
                .map(name)
                .unwrap_or_else(|| "—".into())
        )
        .unwrap();
    }

    // ------------------------------------------------------------- Terrain
    writeln!(o, "\n## Terrain and natural wonders\n").unwrap();
    writeln!(
        o,
        "| Terrain | Yields | Moves | Defense |\n|---|---|---|---|"
    )
    .unwrap();
    for t in &c.terrains {
        writeln!(
            o,
            "| {} | {} | {} | {} |",
            t.name,
            yields(&t.yields),
            if t.mountain {
                "impassable".into()
            } else {
                t.movement_cost.to_string()
            },
            t.defense_modifier
        )
        .unwrap();
    }
    writeln!(
        o,
        "\n| Feature | Yields | Extra moves | Defense |\n|---|---|---|---|"
    )
    .unwrap();
    for f in c.features.iter().filter(|f| !f.natural_wonder) {
        writeln!(
            o,
            "| {} | {} | {} | {} |",
            f.name,
            yields(&f.yields),
            if f.impassable {
                "impassable".into()
            } else {
                f.movement_change.to_string()
            },
            f.defense_modifier
        )
        .unwrap();
    }
    writeln!(
        o,
        "\n| Natural wonder | Yields | Description |\n|---|---|---|"
    )
    .unwrap();
    for f in c.features.iter().filter(|f| f.natural_wonder) {
        writeln!(
            o,
            "| {} | {} | {} |",
            f.name,
            if f.impassable {
                "impassable".into()
            } else {
                yields(&f.yields)
            },
            f.description
        )
        .unwrap();
    }

    // --------------------------------------------------------- City-states
    writeln!(o, "\n## City-states\n").unwrap();
    for cat in [
        "SCIENTIFIC",
        "CULTURAL",
        "MILITARISTIC",
        "RELIGIOUS",
        "TRADE",
        "INDUSTRIAL",
    ] {
        let list: Vec<&str> = c
            .city_states
            .iter()
            .filter(|s| s.category == cat)
            .map(|s| s.name.as_str())
            .collect();
        writeln!(o, "- **{}**: {}", words(cat), list.join(", ")).unwrap();
    }

    // ------------------------------------------------------------ Projects
    writeln!(o, "\n## Projects\n").unwrap();
    writeln!(o, "Space race projects, built in order in a city with a Spaceport; finishing all of them wins a science victory.\n").unwrap();
    writeln!(
        o,
        "| Project | Tech | Cost | Description |\n|---|---|---|---|"
    )
    .unwrap();
    for p in &c.projects {
        writeln!(
            o,
            "| {} | {} | {} | {} |",
            p.name,
            p.prereq_tech.as_deref().map(name).unwrap_or_default(),
            p.cost,
            p.description
        )
        .unwrap();
    }

    print!("{out}");
}

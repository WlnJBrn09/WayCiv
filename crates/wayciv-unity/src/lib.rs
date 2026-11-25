//! Stable JSON/C ABI for the Unity presentation. The Rust engine owns all rules.
#![recursion_limit = "256"]
use serde_json::{Value, json};
use std::{ffi::{CStr, CString, c_char}, sync::Mutex};
use wayciv_core::{Game, Settings, Pos, Production, MapSize, MapType, GameLength};
use wayciv_core::{actions::UnitAction, research::Tree, turn::Blocker};

static SESSION: Mutex<Option<Game>> = Mutex::new(None);
const VERSION: u32 = 1;

fn number(v: &Value, key: &str) -> Result<usize, String> {
    v[key].as_u64().and_then(|n| usize::try_from(n).ok()).ok_or_else(|| format!("Missing or invalid {key}"))
}

fn pos(v: &Value, g: &Game) -> Result<Pos, String> {
    let p = Pos::new(
        i32::try_from(number(v, "x")?).map_err(|_| "Invalid x")?,
        i32::try_from(number(v, "y")?).map_err(|_| "Invalid y")?,
    );
    if p.x < 0 || p.y < 0 || p.x >= g.map.grid.width || p.y >= g.map.grid.height {
        return Err("Outside map".into());
    }
    Ok(p)
}

fn positions(v: Vec<Pos>) -> Vec<Value> {
    v.into_iter().map(|p| json!({"x": p.x, "y": p.y})).collect()
}

/// Six neighbors counterclockwise from East: 0=E, 1=NE, 2=NW, 3=W, 4=SW, 5=SE.
fn ccw_neighbor(grid: wayciv_core::Grid, p: Pos, edge: usize) -> Option<Pos> {
    let odd = p.y & 1;
    let (nx, ny) = match edge {
        0 => (p.x + 1, p.y),
        1 => (p.x + odd, p.y - 1),
        2 => (p.x + odd - 1, p.y - 1),
        3 => (p.x - 1, p.y),
        4 => (p.x + odd - 1, p.y + 1),
        _ => (p.x + odd, p.y + 1),
    };
    grid.normalize(Pos::new(nx, ny))
}

fn action(a: &UnitAction, g: &Game) -> (String, String) {
    match a {
        UnitAction::Build(i) => (format!("build:{i}"), format!("Build {}", g.rules.content.improvements[*i].name)),
        UnitAction::TradeRoute(i) => (format!("trade:{i}"), format!("Trade with {}", g.cities[i].name)),
        _ => {
            let s = format!("{a:?}");
            (
                s.clone(),
                match a {
                    UnitAction::FoundCity => "Found city".into(),
                    UnitAction::FoundReligion => "Found religion".into(),
                    UnitAction::SpreadReligion => "Spread religion".into(),
                    UnitAction::Activate => "Activate Great Person".into(),
                    _ => s,
                },
            )
        }
    }
}

fn production(p: &Production, g: &Game, city: u32) -> Value {
    let (kind, index, sites) = match p {
        Production::Unit(i) => ("unit", *i, vec![]),
        Production::Building(i) => ("building", *i, vec![]),
        Production::Project(i) => ("project", *i, vec![]),
        Production::District(i, _) => ("district", *i, g.district_sites(city, *i)),
        Production::Wonder(i, _) => ("wonder", *i, g.wonder_sites(city, *i)),
    };
    json!({
        "kind": kind,
        "index": index,
        "name": g.item_name(p),
        "cost": g.production_cost(g.cities[&city].owner, p),
        "sites": positions(sites),
        "purchase": g.purchase_cost(city, p).unwrap_or(-1)
    })
}

fn snapshot(g: &Game) -> Value {
    let h = g.human();
    let p = &g.players[h];
    let c = &g.rules.content;
    let income = g.income(h);
    let grid = g.map.grid;
    let tiles: Vec<_> = grid
        .all()
        .map(|pos| {
            let i = grid.index(pos);
            let t = g.map.tile(pos);
            if !p.revealed[i] {
                return json!({"x": pos.x, "y": pos.y, "known": false});
            }
            let resource = t.resource.filter(|&r| p.meets(&g.rules, c.resources[r].prereq_tech.as_deref()));
            let is_water = g.map.is_water(&g.rules, pos);
            let mut shore_mask = 0u8;
            let mut river_mask = 0u8;
            if !is_water {
                for e in 0..6 {
                    if let Some(np) = ccw_neighbor(grid, pos, e) {
                        if g.map.is_water(&g.rules, np) {
                            shore_mask |= 1 << e;
                        }
                        if t.river {
                            let nt = g.map.tile(np);
                            let connected = match t.river_dir {
                                Some(d) => (d as usize) == e || (nt.river && nt.river_dir == Some(((e + 3) % 6) as u8)),
                                None => nt.river || g.map.is_water(&g.rules, np),
                            };
                            if connected {
                                river_mask |= 1 << e;
                            }
                        }
                    }
                }
                if t.river && river_mask == 0 {
                    river_mask = 1;
                }
            }
            let worked = t
                .owning_city
                .and_then(|cid| g.cities.get(&cid))
                .is_some_and(|city| city.owner == h && city.worked.contains(&pos));
            json!({
                "x": pos.x,
                "y": pos.y,
                "known": true,
                "visible": p.visible[i],
                "terrain": c.terrains[t.terrain].id,
                "feature": t.feature.map(|i| &c.features[i].id),
                "resource": resource.map(|i| &c.resources[i].id),
                "improvement": t.improvement.map(|i| &c.improvements[i].id),
                "district": t.district.map(|i| &c.districts[i].id),
                "wonder": t.wonder.map(|i| &c.buildings[i].id),
                "owner": t.owner.map(|v| v as i32).unwrap_or(-1),
                "river": t.river,
                "riverMask": river_mask,
                "shoreMask": shore_mask,
                "road": t.road,
                "worked": worked,
                "yields": g.tile_yields(h, pos)
            })
        })
        .collect();

    let units: Vec<_> = g
        .units
        .values()
        .filter(|u| u.owner == h || p.visible[grid.index(u.pos)])
        .map(|u| {
            let actions: Vec<_> = if u.owner == h {
                g.unit_actions(u.id)
                    .iter()
                    .map(|a| {
                        let (id, name) = action(a, g);
                        json!({"id": id, "name": name})
                    })
                    .collect()
            } else {
                vec![]
            };
            let rebase = if u.owner == h && g.is_air(u) {
                positions(g.rebase_targets(u))
            } else {
                vec![]
            };
            json!({
                "id": u.id,
                "owner": u.owner,
                "asset": g.unit_type(u).id,
                "name": g.unit_type(u).name,
                "domain": g.unit_type(u).domain,
                "x": u.pos.x,
                "y": u.pos.y,
                "hp": u.hp,
                "moves": u.moves,
                "charges": u.charges,
                "fortified": u.fortified,
                "sleeping": u.sleeping,
                "actions": actions,
                "rebaseTargets": rebase
            })
        })
        .collect();

    let cities: Vec<_> = g
        .cities
        .values()
        .filter(|u| u.owner == h || p.visible[grid.index(u.pos)])
        .map(|u| {
            let choices = if u.owner == h {
                g.available_production(u.id).iter().map(|a| production(a, g, u.id)).collect::<Vec<_>>()
            } else {
                vec![]
            };
            let faith_units = if u.owner == h {
                g.faith_units(u.id)
                    .into_iter()
                    .map(|idx| json!({"index": idx, "name": c.units[idx].name, "cost": g.faith_cost(idx)}))
                    .collect::<Vec<_>>()
            } else {
                vec![]
            };
            let bombard_targets = if u.owner == h {
                grid.within(u.pos, 2)
                    .into_iter()
                    .filter(|&target| g.city_can_attack(u.id, target))
                    .collect::<Vec<_>>()
            } else {
                vec![]
            };
            let buildings: Vec<_> = u.buildings.iter().map(|&i| c.buildings[i].id.clone()).collect();
            let building_names: Vec<_> = u.buildings.iter().map(|&i| c.buildings[i].name.clone()).collect();
            let rep = g.city_report(u.id);
            json!({
                "id": u.id,
                "owner": u.owner,
                "era": g.era_of(u.owner).min(3),
                "name": u.name,
                "x": u.pos.x,
                "y": u.pos.y,
                "population": u.population,
                "hp": u.hp,
                "wallHp": u.wall_hp,
                "maxWalls": g.max_walls(u.id),
                "housing": rep.housing,
                "amenities": rep.amenities,
                "amenitiesNeeded": rep.amenities_needed,
                "turnsToGrow": rep.turns_to_grow.unwrap_or(-1),
                "capital": u.capital,
                "buildings": buildings,
                "buildingNames": building_names,
                "production": u.production.as_ref().map(|a| g.item_name(a)).unwrap_or_default(),
                "choices": choices,
                "faithUnits": faith_units,
                "bombardTargets": positions(bombard_targets),
                "yields": rep.yields
            })
        })
        .collect();

    let research: Vec<_> = [Tree::Tech, Tree::Ai]
        .into_iter()
        .flat_map(|tree| {
            g.tree_items(tree).iter().enumerate().map(move |(i, t)| {
                json!({
                    "index": i,
                    "tree": if matches!(tree, Tree::Tech) { "tech" } else { "ai" },
                    "name": t.name,
                    "era": t.era,
                    "description": t.description,
                    "eureka": t.boost.clone().unwrap_or_default(),
                    "boosted": if matches!(tree, Tree::Tech) { p.tech_boosted.get(i).copied().unwrap_or(false) } else { false },
                    "known": g.known(h, tree, i),
                    "available": g.researchable(h, tree, i),
                    "current": g.current_research(h, tree) == Some(i),
                    "cost": g.research_cost(tree, i),
                    "progress": g.progress(h, tree, i)
                })
            })
        })
        .collect();

    let players: Vec<_> = g
        .players
        .iter()
        .filter(|o| o.is_major() && (o.id == h || p.relations[o.id].met))
        .map(|o| {
            let rel = &p.relations[o.id];
            let (_, reason) = if o.id != h { g.agenda_opinion(o.id, h) } else { (0, None) };
            json!({
                "id": o.id,
                "name": o.name,
                "civilization": o.civ_name,
                "color": o.primary,
                "alive": o.alive,
                "war": rel.at_war,
                "friendship": rel.friendship,
                "alliance": rel.alliance,
                "denounced": rel.denounced,
                "opinion": if o.id == h { 0 } else { g.opinion(o.id, h) },
                "grievances": rel.grievances,
                "agendaNote": reason.unwrap_or_default(),
                "canWar": g.can_declare_war(h, o.id),
                "canPeace": g.can_make_peace(h, o.id),
                "canAlly": g.can_ally(h, o.id),
                "score": g.score(o.id),
                "military": g.military_strength(o.id),
                "cities": g.player_cities(o.id).count()
            })
        })
        .collect();

    let city_states: Vec<_> = g
        .players
        .iter()
        .filter(|o| o.is_city_state() && o.alive && p.relations[o.id].met)
        .map(|o| {
            let suz = g.suzerain(o.id).map(|id| g.players[id].civ_name.clone()).unwrap_or_else(|| "None".into());
            json!({
                "id": o.id,
                "name": o.name,
                "category": o.city_state_category,
                "color": o.primary,
                "envoys": p.envoys.get(&o.id).copied().unwrap_or(0),
                "suzerain": suz,
                "bonus": format!("{} city-state", o.city_state_category)
            })
        })
        .collect();

    let governments: Vec<_> = g
        .available_governments(h)
        .into_iter()
        .map(|idx| {
            let gov = &c.governments[idx];
            json!({
                "index": idx,
                "name": gov.name,
                "description": gov.bonus,
                "current": p.government == Some(idx)
            })
        })
        .collect();

    let policy_slots: Vec<_> = g
        .policy_slots(h)
        .into_iter()
        .map(|(slot, count)| json!({"slot": slot, "count": count}))
        .collect();

    let policy_draft: Vec<_> = p
        .cards
        .draft
        .iter()
        .map(|&idx| {
            let card = &c.policies[idx];
            json!({"index": idx, "name": card.name, "slot": card.slot, "description": card.description})
        })
        .collect();

    let adopted_policies: Vec<_> = g
        .available_policies(h)
        .into_iter()
        .map(|idx| {
            let card = &c.policies[idx];
            json!({
                "index": idx,
                "name": card.name,
                "slot": card.slot,
                "description": card.description,
                "slotted": p.policies.contains(&idx)
            })
        })
        .collect();

    let pantheons: Vec<_> = if g.can_choose_pantheon(h) {
        g.beliefs_of_class("BELIEF_CLASS_PANTHEON")
            .into_iter()
            .map(|idx| {
                let b = &c.beliefs[idx];
                json!({"index": idx, "name": b.name, "description": b.description})
            })
            .collect()
    } else {
        vec![]
    };

    let dedications: Vec<_> = if p.era.pending > 0 {
        g.dedication_options(h)
            .into_iter()
            .map(|idx| {
                let d = &c.dedications[idx];
                json!({"index": idx, "name": d.name, "moment": d.moment, "description": d.description})
            })
            .collect()
    } else {
        vec![]
    };

    let rates = g.great_person_rates(h);
    let great_people: Vec<_> = (0..c.great_person_classes.len())
        .filter_map(|cls| {
            let person = g.next_great_person(cls)?;
            let gp = &c.great_people[person];
            let rate = rates.iter().find(|(k, _)| *k == cls).map(|(_, r)| *r).unwrap_or(0);
            Some(json!({
                "classIndex": cls,
                "className": c.great_person_classes[cls].name,
                "personIndex": person,
                "name": gp.name,
                "era": gp.era,
                "points": p.great_person_points.get(&cls).copied().unwrap_or(0),
                "cost": g.great_person_cost(cls),
                "rate": rate,
                "effect": g.great_person_effect(person)
            }))
        })
        .collect();

    let victory: Vec<_> = g
        .victory_progress(h)
        .into_iter()
        .map(|v| json!({"name": v.name, "detail": v.detail, "frac": v.frac}))
        .collect();

    let blocker_items: Vec<_> = g
        .blockers(h)
        .iter()
        .map(|b| {
            let (kind, unit, city) = match b {
                Blocker::ChooseResearch => ("research", -1, -1),
                Blocker::ChooseAiResearch => ("ai", -1, -1),
                Blocker::ChoosePantheon => ("pantheon", -1, -1),
                Blocker::ChoosePolicyCard => ("policy_draft", -1, -1),
                Blocker::ChooseDedication => ("dedication", -1, -1),
                Blocker::ChooseProduction(cid) => ("production", -1, *cid as i32),
                Blocker::UnitNeedsOrders(uid) => ("unit", *uid as i32, -1),
            };
            json!({"kind": kind, "text": g.blocker_text(b), "unit": unit, "city": city})
        })
        .collect();

    let moments: Vec<_> = p
        .era
        .moments
        .iter()
        .rev()
        .take(16)
        .map(|m| format!("Turn {} (+{}): {}", m.turn, m.score, m.text))
        .collect();

    let notices: Vec<_> = g
        .notifications
        .iter()
        .rev()
        .filter(|n| n.player == h)
        .take(16)
        .map(|n| n.text.clone())
        .collect();

    let (dark_t, gold_t) = g.era_thresholds(h);
    json!({
        "version": VERSION,
        "turn": g.turn,
        "turnLimit": g.turn_limit().map(|t| t as i32).unwrap_or(-1),
        "year": wayciv_core::length::year_label(g.year()),
        "human": h,
        "width": grid.width,
        "height": grid.height,
        "gold": p.gold,
        "faith": p.faith,
        "compute": g.compute(h),
        "aiUnlocked": g.ai_tree_unlocked(h),
        "tourism": income.tourism,
        "envoysAvailable": p.envoys_available,
        "income": income.yields,
        "netGold": income.net_gold,
        "era": g.era_name(g.world_era),
        "age": p.era.age.name(),
        "projectedAge": g.projected_age(h).name(),
        "eraScore": p.era.score,
        "darkThreshold": dark_t,
        "goldThreshold": gold_t,
        "government": p.government.map(|i| c.governments[i].name.clone()).unwrap_or_else(|| "Chiefdom".into()),
        "pantheon": p.pantheon.map(|i| c.beliefs[i].name.clone()).unwrap_or_default(),
        "religion": p.religion.map(|i| c.religions[i].name.clone()).unwrap_or_default(),
        "tiles": tiles,
        "units": units,
        "cities": cities,
        "research": research,
        "players": players,
        "cityStates": city_states,
        "governments": governments,
        "policySlots": policy_slots,
        "policyDraft": policy_draft,
        "adoptedPolicies": adopted_policies,
        "pantheons": pantheons,
        "dedications": dedications,
        "greatPeople": great_people,
        "victory": victory,
        "moments": moments,
        "notifications": notices,
        "blockers": g.blockers(h).iter().map(|b| g.blocker_text(b)).collect::<Vec<_>>(),
        "blockerItems": blocker_items,
        "winner": g.winner.as_ref().map(|(id, v)| format!("{} — {v:?} victory", g.players[*id].name)).unwrap_or_default()
    })
}

fn request(session: &mut Option<Game>, v: &Value) -> Result<Value, String> {
    let command = v["command"].as_str().ok_or("Missing command")?;
    if command == "catalog" {
        let rules = wayciv_core::load_rules().map_err(|e| e.to_string())?;
        return Ok(json!({
            "leaders": rules.content.leaders.iter().map(|l| json!({
                "id": l.id,
                "name": l.name,
                "civilization": l.civilization
            })).collect::<Vec<_>>()
        }));
    }
    if command == "new" {
        let rules = wayciv_core::load_rules().map_err(|e| e.to_string())?;
        let mut s = Settings::default();
        s.seed = v["seed"].as_u64().unwrap_or(42);
        s.map_type = if v["map"] == "earth" { MapType::Earth } else { MapType::Random };
        s.size = match v["size"].as_str() {
            Some("large") => MapSize::Large,
            Some("standard") => MapSize::Standard,
            _ => MapSize::Small,
        };
        s.opponents = v["opponents"].as_u64().unwrap_or(3).min(11) as usize;
        s.city_states = v["cityStates"].as_u64().unwrap_or(4).min(12) as usize;
        if let Some(l) = v["leader"].as_str() && !l.is_empty() {
            if !rules.content.leaders.iter().any(|x| x.id == l) {
                return Err("Unknown leader".into());
            }
            s.human_leader = l.into();
        }
        s.length = GameLength::parse(v["length"].as_str().unwrap_or("500")).ok_or("Unknown game length")?;
        *session = Some(wayciv_core::new_game(rules, s));
    }
    let g = session.as_mut().ok_or("Start a game first")?;
    let h = g.human();
    let unit = |g: &Game| -> Result<u32, String> {
        let id = u32::try_from(number(v, "unit")?).map_err(|_| "Invalid unit")?;
        if g.units.get(&id).is_none_or(|u| u.owner != h) {
            return Err("Select your own unit".into());
        }
        Ok(id)
    };
    let city = |g: &Game| -> Result<u32, String> {
        let id = u32::try_from(number(v, "city")?).map_err(|_| "Invalid city")?;
        if g.cities.get(&id).is_none_or(|u| u.owner != h) {
            return Err("Select your own city".into());
        }
        Ok(id)
    };
    match command {
        "new" | "snapshot" => {}
        "end_turn" => g.end_turn(),
        "move" => {
            let id = unit(g)?;
            let p = pos(v, g)?;
            g.move_unit(id, p)?;
        }
        "attack" => {
            let id = unit(g)?;
            let p = pos(v, g)?;
            g.attack(id, p)?;
        }
        "rebase" => {
            let id = unit(g)?;
            let p = pos(v, g)?;
            g.rebase(id, p)?;
        }
        "city_attack" => {
            let id = city(g)?;
            let p = pos(v, g)?;
            g.city_attack(id, p)?;
        }
        "reachable" => {
            let id = unit(g)?;
            return Ok(json!({"positions": positions(g.reachable(&g.units[&id]))}));
        }
        "action" => {
            let id = unit(g)?;
            let a = g
                .unit_actions(id)
                .into_iter()
                .find(|a| Some(action(a, g).0.as_str()) == v["action"].as_str())
                .ok_or("Order unavailable")?;
            g.perform(id, &a)?;
        }
        "research" => {
            let i = number(v, "index")?;
            let tree = if v["tree"] == "ai" { Tree::Ai } else { Tree::Tech };
            if i >= g.tree_items(tree).len() {
                return Err("Unknown research".into());
            }
            g.set_research(h, tree, i)?;
        }
        "production" | "purchase" => {
            let id = city(g)?;
            let i = number(v, "index")?;
            let choice = g
                .available_production(id)
                .into_iter()
                .find(|a| {
                    let p = production(a, g, id);
                    p["kind"] == v["kind"] && p["index"] == i
                })
                .ok_or("Production unavailable")?;
            let choice = match choice {
                Production::District(i, _) => Production::District(i, pos(v, g)?),
                Production::Wonder(i, _) => Production::Wonder(i, pos(v, g)?),
                p => p,
            };
            if command == "purchase" {
                g.purchase(id, choice)?;
            } else {
                g.set_production(id, choice)?;
            }
        }
        "faith_purchase" => {
            let id = city(g)?;
            let i = number(v, "index")?;
            g.buy_with_faith(id, i)?;
        }
        "pantheon" => {
            let i = number(v, "index")?;
            g.choose_pantheon(h, i)?;
        }
        "dedication" => {
            let i = number(v, "index")?;
            g.choose_dedication(h, i)?;
        }
        "adopt_policy" => {
            let i = number(v, "index")?;
            g.adopt_policy(h, i)?;
        }
        "government" => {
            let i = number(v, "index")?;
            g.set_government(h, i)?;
        }
        "toggle_policy" => {
            let i = number(v, "index")?;
            let mut next = g.players[h].policies.clone();
            if let Some(pos) = next.iter().position(|&x| x == i) {
                next.remove(pos);
            } else {
                next.push(i);
            }
            g.set_policies(h, next)?;
        }
        "diplomacy" => {
            let other = number(v, "player")?;
            if other == h || other >= g.players.len() || !g.players[h].relations[other].met {
                return Err("Unknown foreign leader".into());
            }
            match v["action"].as_str() {
                Some("war") => {
                    if !g.can_declare_war(h, other) {
                        return Err("War cannot be declared".into());
                    }
                    g.declare_war(h, other);
                }
                Some("peace") => {
                    if !g.propose_peace(h, other) {
                        return Err("Peace proposal declined".into());
                    }
                }
                Some("friendship") => {
                    if !g.propose_friendship(h, other) {
                        return Err("Friendship declined".into());
                    }
                }
                Some("alliance") => {
                    if !g.propose_alliance(h, other) {
                        return Err("Alliance declined".into());
                    }
                }
                Some("denounce") => {
                    g.denounce(h, other);
                }
                Some("envoy") => g.send_envoy(h, other)?,
                _ => return Err("Unknown diplomatic action".into()),
            }
        }
        _ => return Err(format!("Unknown command: {command}")),
    }
    g.events.clear();
    g.popups.clear();
    g.reveals.clear();
    Ok(snapshot(g))
}

/// Returns a newly allocated UTF-8 JSON response. Always release with wc_free.
/// # Safety
/// `input` must point to a valid, null-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wc_request(input: *const c_char) -> *mut c_char {
    let response = std::panic::catch_unwind(|| -> Result<Value, String> {
        if input.is_null() {
            return Err("Null request".into());
        }
        let data = unsafe { CStr::from_ptr(input) }.to_str().map_err(|e| e.to_string())?;
        let v: Value = serde_json::from_str(data).map_err(|e| e.to_string())?;
        let mut session = SESSION.lock().map_err(|_| "Engine session failed; restart the application")?;
        request(&mut session, &v)
    });
    let value = match response {
        Ok(Ok(data)) => json!({"ok": true, "data": data}),
        Ok(Err(error)) => json!({"ok": false, "error": error}),
        Err(_) => json!({"ok": false, "error": "Engine panic contained; restart the session"}),
    };
    CString::new(value.to_string()).expect("JSON has no null bytes").into_raw()
}

/// # Safety
/// Pass only a live pointer returned by wc_request, exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wc_free(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gameplay_through_protocol() {
        let mut s = None;
        let initial = request(&mut s, &json!({"command": "new", "seed": 42})).unwrap();
        assert_eq!(initial["tiles"].as_array().unwrap().len(), 52 * 32);
        let g = s.as_ref().unwrap();
        let human = g.human();
        let settler = g.player_units(human).find(|u| g.unit_type(u).found_city).unwrap().id;
        let out = request(&mut s, &json!({"command": "action", "unit": settler, "action": "FoundCity"})).unwrap();
        assert!(out["cities"].as_array().unwrap().iter().any(|c| c["owner"] == human));
        let tech = out["research"].as_array().unwrap().iter().find(|t| t["available"] == true && t["tree"] == "tech").unwrap();
        request(&mut s, &json!({"command": "research", "tree": "tech", "index": tech["index"]})).unwrap();
        let c = out["cities"].as_array().unwrap().iter().find(|c| c["owner"] == human).unwrap();
        let item = c["choices"].as_array().unwrap().iter().find(|i| i["kind"] == "unit").unwrap();
        request(&mut s, &json!({"command": "production", "city": c["id"], "kind": "unit", "index": item["index"]})).unwrap();
        let next = request(&mut s, &json!({"command": "end_turn"})).unwrap();
        assert_eq!(next["turn"].as_u64().unwrap(), initial["turn"].as_u64().unwrap() + 1);
        assert!(next["victory"].as_array().unwrap().len() >= 4);
        assert!(request(&mut s, &json!({"command": "move", "unit": 999999, "x": 0, "y": 0})).is_err());
        assert!(request(&mut s, &json!({"command": "research", "index": 999999})).is_err());
        assert!(next["tiles"].as_array().unwrap().iter().any(|t| t["known"] == false && t.get("terrain").is_none()));
    }
}

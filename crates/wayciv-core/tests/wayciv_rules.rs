//! Scenario tests for WayCiv's own rules: civilians, city-states, loyalty,
//! the AI tree, strike drones, purchases, the Earth map, and modern great
//! people.

use wayciv_core::additions::*;
use wayciv_core::research::Tree;
use wayciv_core::state::{Game, Production};
use wayciv_core::{MapType, Pos, Settings, load_rules, new_game};

fn game() -> Option<Game> {
    let rules = load_rules().ok()?;
    Some(new_game(
        rules,
        Settings {
            seed: 5,
            map_type: MapType::Random,
            ..Settings::default()
        },
    ))
}

fn other_major(g: &Game) -> usize {
    let me = g.human();
    g.players
        .iter()
        .position(|p| p.id != me && p.is_major())
        .unwrap()
}

/// A land tile with land neighbors, far from cities and units.
fn open_land(g: &Game, avoid: &[Pos]) -> Pos {
    let grid = g.map.grid;
    let rules = &g.rules;
    grid.all()
        .find(|&p| {
            g.map.is_land(rules, p)
                && !g.map.is_impassable(rules, p)
                && grid
                    .neighbors(p)
                    .all(|n| g.map.is_land(rules, n) && !g.map.is_impassable(rules, n))
                && g.cities.values().all(|c| grid.distance(c.pos, p) >= 8)
                && g.units.values().all(|u| grid.distance(u.pos, p) >= 3)
                && avoid.iter().all(|a| grid.distance(*a, p) >= 10)
        })
        .expect("open land")
}

#[test]
fn one_attack_kills_a_settler() {
    let Some(mut g) = game() else { return };
    let me = g.human();
    let enemy = other_major(&g);
    g.players[me].relations[enemy].met = true;
    g.players[enemy].relations[me].met = true;
    g.declare_war(me, enemy);
    let pos = open_land(&g, &[]);
    let target = g.map.grid.neighbors(pos).next().unwrap();
    let warrior = g.spawn(me, "UNIT_WARRIOR", pos).unwrap();
    let settler = g.spawn(enemy, "UNIT_SETTLER", target).unwrap();
    let preview = g.preview(warrior, target).unwrap();
    assert_eq!(
        (preview.damage_to_defender, preview.damage_to_attacker),
        (100, 0)
    );
    g.attack(warrior, target).unwrap();
    assert!(
        !g.units.contains_key(&settler),
        "the settler died in one attack"
    );
    assert_eq!(g.units[&warrior].pos, target, "the melee attacker moved in");
    assert_eq!(g.units[&warrior].hp, 100, "civilians don't fight back");
}

#[test]
fn city_states_cannot_be_declared_war_on() {
    let Some(mut g) = game() else { return };
    let me = g.human();
    let cs = g.players.iter().position(|p| p.is_city_state()).unwrap();
    assert!(!g.can_declare_war(me, cs));
    g.declare_war(me, cs);
    assert!(!g.players[me].at_war_with(cs));
    let other = other_major(&g);
    assert!(g.can_declare_war(me, other));
}

#[test]
fn a_city_without_loyalty_defects() {
    let Some(mut g) = game() else { return };
    let me = g.human();
    let enemy = other_major(&g);
    let a = open_land(&g, &[]);
    let mine = g.found_city_at(me, a, Some("Border Town".into()));
    // A big foreign city next door pulls it away.
    let b = g
        .map
        .grid
        .within(a, 4)
        .into_iter()
        .find(|&p| {
            g.map.grid.distance(a, p) == 4
                && g.map.is_land(&g.rules, p)
                && !g.map.is_impassable(&g.rules, p)
        })
        .unwrap();
    let theirs = g.found_city_at(enemy, b, Some("Metropolis".into()));
    g.cities.get_mut(&theirs).unwrap().population = 25;
    let report = g.loyalty_report(mine);
    assert!(report.per_turn < 0, "{report:?}");
    assert_eq!(report.strongest_foreign, Some(enemy));
    g.cities.get_mut(&mine).unwrap().loyalty = 1;
    g.end_turn();
    assert_eq!(g.cities[&mine].owner, enemy, "the city defected");
    assert!(g.cities[&mine].loyalty > 0);
}

#[test]
fn datacenters_power_the_ai_tree() {
    let Some(mut g) = game() else { return };
    let me = g.human();
    let a = open_land(&g, &[]);
    let city = g.found_city_at(me, a, Some("Silicon".into()));
    assert!(
        !g.researchable(me, Tree::Ai, 0),
        "locked before Artificial Intelligence"
    );
    let ai = g.rules.tech(TECH_AI).unwrap();
    g.players[me].techs[ai] = true;
    assert!(g.researchable(me, Tree::Ai, 0));
    let dc = g.rules.idx(DATACENTER).unwrap();
    let site = g.map.grid.neighbors(a).next().unwrap();
    g.complete_production_for_test(city, Production::District(dc, site));
    let compute = g.compute(me);
    assert!(compute >= 6.0, "compute {compute}");
    let science_before = g.city_report(city).yields[3];
    g.set_research(me, Tree::Ai, 0).unwrap();
    for _ in 0..40 {
        if g.players[me].ai_techs[0] {
            break;
        }
        g.players[me].research = None;
        g.end_turn();
    }
    assert!(
        g.has_ai_tech(me, AI_MACHINE_LEARNING),
        "Machine Learning researched with Compute"
    );
    assert!(
        g.city_report(city).yields[3] > science_before,
        "Machine Learning adds Science"
    );
}

#[test]
fn strike_drones_come_three_at_a_time_and_buying_keeps_the_build() {
    let Some(mut g) = game() else { return };
    let me = g.human();
    let a = open_land(&g, &[]);
    let city = g.found_city_at(me, a, Some("Drone Works".into()));
    for p in g.map.grid.within(a, 2) {
        if g.map.tile(p).owner.is_none() {
            g.claim(city, p);
        }
    }
    let drone = g.rules.idx(STRIKE_DRONE).unwrap();
    let ai = g.rules.tech(TECH_AI).unwrap();
    g.players[me].techs[ai] = true;
    assert!(
        !g.can_produce(city, &Production::Unit(drone)),
        "needs Autonomous Systems"
    );
    let autonomy = g.rules.ai_tech(AI_AUTONOMOUS_SYSTEMS).unwrap();
    g.players[me].ai_techs[autonomy] = true;
    assert!(g.can_produce(city, &Production::Unit(drone)));
    let warrior = g.rules.idx("UNIT_WARRIOR").unwrap();
    g.set_production(city, Production::Unit(warrior)).unwrap();
    g.players[me].gold = 100_000;
    let before = g.player_units(me).filter(|u| g.is_drone(u)).count();
    g.purchase(city, Production::Unit(drone)).unwrap();
    let after = g.player_units(me).filter(|u| g.is_drone(u)).count();
    assert_eq!(after - before, DRONES_PER_BATCH);
    assert_eq!(
        g.cities[&city].production,
        Some(Production::Unit(warrior)),
        "buying didn't clear the current build"
    );
}

#[test]
fn modern_great_people_exist_with_their_effects() {
    let Some(g) = game() else { return };
    let names: Vec<&str> = g
        .rules
        .content
        .great_people
        .iter()
        .map(|p| p.name.as_str())
        .collect();
    for name in [
        "Elon Musk",
        "Gwynne Shotwell",
        "Palmer Luckey",
        "Ilya Sutskever",
        "Yanni",
        "NF",
        "Michael Crichton",
        "Iain M. Banks",
        "Douglas Adams",
    ] {
        assert!(names.contains(&name), "{name} missing");
    }
    let elon = g
        .rules
        .content
        .great_people
        .iter()
        .position(|p| p.id == ELON_MUSK)
        .unwrap();
    assert!(g.great_person_effect(elon).contains("Solar"));
}

#[test]
fn earth_starts_leaders_at_home() {
    let Ok(rules) = load_rules() else { return };
    let g = new_game(
        rules,
        Settings {
            seed: 3,
            map_type: MapType::Earth,
            human_leader: "LEADER_VICTORIA".into(),
            ..Settings::default()
        },
    );
    let me = g.human();
    let start = g
        .player_units(me)
        .find(|u| g.unit_type(u).found_city)
        .unwrap()
        .pos;
    let proj = wayciv_core::earth::projection(g.settings.size);
    let london = proj.tile_of(51.5, -0.1).unwrap();
    assert!(
        g.map.grid.distance(start, london) <= 3,
        "Victoria starts near London: {start} vs {london}"
    );
    let everest = g.rules.idx("FEATURE_EVEREST").unwrap();
    let at = g
        .map
        .grid
        .all()
        .find(|&p| g.map.tile(p).feature == Some(everest))
        .expect("Everest placed");
    let target = proj.tile_of(27.99, 86.93).unwrap();
    assert!(g.map.grid.distance(at, target) <= 2);
}

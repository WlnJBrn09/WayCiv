//! Scenario tests for trade, aircraft, and barbarians.

use wayciv_core::state::{Game, Production};
use wayciv_core::{Pos, Settings, load_rules, new_game};

fn game() -> Option<Game> {
    let rules = load_rules().ok()?;
    Some(new_game(
        rules,
        Settings {
            seed: 5,
            ..Settings::default()
        },
    ))
}

fn learn(game: &mut Game, player: usize, tech: &str) {
    let i = game.rules.tech(tech).unwrap();
    game.players[player].techs[i] = true;
}

/// Two land tiles at the given distance, far from any existing city.
fn land_pair(game: &Game, distance: i32) -> (Pos, Pos) {
    let grid = game.map.grid;
    let rules = &game.rules;
    for a in grid.all() {
        if !game.map.is_land(rules, a)
            || game.map.is_impassable(rules, a)
            || game.cities.values().any(|c| grid.distance(c.pos, a) < 6)
        {
            continue;
        }
        for b in grid.within(a, distance) {
            if grid.distance(a, b) == distance
                && game.map.is_land(rules, b)
                && !game.map.is_impassable(rules, b)
                && game.cities.values().all(|c| grid.distance(c.pos, b) >= 4)
            {
                return (a, b);
            }
        }
    }
    panic!("no land pair");
}

#[test]
fn gandhi_is_not_in_the_game_and_spies_are_not_buildable() {
    let Some(game) = game() else { return };
    assert!(
        game.rules
            .content
            .leaders
            .iter()
            .all(|l| l.id != "LEADER_GANDHI")
    );
    assert!(game.players.iter().all(|p| p.name != "Gandhi"));
    let me = game.human();
    let mut g = game;
    for i in 0..g.rules.content.techs.len() {
        g.players[me].techs[i] = true;
    }
    g.turn = g.turn_of_year(2040);
    let settler = g
        .player_units(me)
        .find(|u| g.unit_type(u).found_city)
        .unwrap()
        .id;
    g.found_city(settler).unwrap();
    let city = g.player_cities(me).next().unwrap().id;
    // WayCiv has no espionage: there is no Spy unit to build.
    assert!(g.rules.idx("UNIT_SPY").is_none());
    assert!(
        (0..g.rules.content.units.len()).all(|u| g.rules.content.units[u].id != "UNIT_SPY"
            || !g.can_produce(city, &Production::Unit(u)))
    );
}

#[test]
fn trade_route_earns_yields_builds_road_and_returns_trader() {
    let Some(mut g) = game() else { return };
    let me = g.human();
    let (a, b) = land_pair(&g, 6);
    let origin = g.found_city_at(me, a, Some("Origin".into()));
    let destination = g.found_city_at(me, b, Some("Destination".into()));
    g.turn = g.turn_of_year(-2500);
    assert!(g.trade_capacity(me) >= 1, "the Palace holds a route");
    let trader_kind = g.rules.idx("UNIT_TRADER").unwrap();
    assert!(g.can_produce(origin, &Production::Unit(trader_kind)));
    let trader = g.spawn(me, "UNIT_TRADER", a).unwrap();
    assert!(g.trade_destinations(trader).contains(&destination));
    let before = g.city_report(origin).yields;
    g.establish_trade_route(trader, destination).unwrap();
    let after = g.city_report(origin).yields;
    assert!(
        after[0] > before[0],
        "domestic routes bring food: {before:?} -> {after:?}"
    );
    assert!(
        g.map.tile(a).road && g.map.tile(b).road,
        "a road joins the cities"
    );
    for _ in 0..wayciv_core::trade::ROUTE_TURNS {
        g.end_turn();
    }
    assert!(
        g.trade_routes_of(me).next().is_none(),
        "the route completed"
    );
    assert!(
        g.player_units(me).any(|u| g.unit_type(u).make_trade_route),
        "the trader came home"
    );
}

#[test]
fn aircraft_strike_rebase_and_take_interceptor_fire() {
    let Some(mut g) = game() else { return };
    let me = g.human();
    let enemy = (0..g.players.len())
        .find(|&p| p != me && g.players[p].is_major())
        .unwrap();
    for p in [me, enemy] {
        learn(&mut g, p, "TECH_FLIGHT");
        learn(&mut g, p, "TECH_ADVANCED_FLIGHT");
    }
    let (a, b) = land_pair(&g, 4);
    let city = g.found_city_at(me, a, Some("Airbase".into()));
    let aerodrome = g.rules.idx("DISTRICT_AERODROME").unwrap();
    let site = g.district_sites(city, aerodrome)[0];
    g.complete_production_for_test(city, Production::District(aerodrome, site));
    let biplane = g.rules.idx("UNIT_BIPLANE").unwrap();
    assert!(
        g.can_produce(city, &Production::Unit(biplane)),
        "aerodrome allows aircraft"
    );
    g.complete_production_for_test(city, Production::Unit(biplane));
    let plane = g.player_units(me).find(|u| g.is_air(u)).unwrap().id;
    assert_eq!(
        g.units[&plane].pos, site,
        "aircraft are based on the aerodrome"
    );

    // Strike an enemy warrior in range; an enemy fighter covers it.
    g.declare_war(me, enemy);
    let target = g.spawn(enemy, "UNIT_WARRIOR", b).unwrap();
    assert!(g.can_attack(&g.units[&plane].clone(), b));
    let fighter_base = g
        .map
        .grid
        .neighbors(b)
        .find(|&p| g.map.is_land(&g.rules, p))
        .unwrap();
    g.spawn(enemy, "UNIT_FIGHTER", fighter_base);
    g.attack(plane, b).unwrap();
    assert!(
        !g.units.contains_key(&target) || g.units[&target].hp < 100,
        "the strike hurt the warrior"
    );
    if let Some(p) = g.units.get(&plane) {
        assert!(p.hp < 100, "interceptor fire hurt the biplane");
    }
}

#[test]
fn barbarians_spawn_raid_and_camps_can_be_cleared() {
    let Some(mut g) = game() else { return };
    let barbarian = g.barbarian().expect("barbarian player");
    assert!(!g.camps.is_empty(), "camps placed at start");
    assert!(
        g.players
            .iter()
            .all(|p| p.id == barbarian || p.at_war_with(barbarian))
    );
    let camp = *g.camps.keys().next().unwrap();
    assert!(
        g.ground_units_at(camp).any(|u| u.owner == barbarian),
        "camps start guarded"
    );

    // Clear it: remove the guard, then walk a warrior in.
    let me = g.human();
    let guards: Vec<_> = g.ground_units_at(camp).map(|u| u.id).collect();
    for id in guards {
        g.units.remove(&id);
    }
    let next = g
        .map
        .grid
        .neighbors(camp)
        .find(|&p| g.map.is_land(&g.rules, p) && !g.map.is_impassable(&g.rules, p))
        .unwrap();
    let warrior = g.spawn(me, "UNIT_WARRIOR", next).unwrap();
    let gold = g.players[me].gold;
    g.move_unit(warrior, camp).unwrap();
    assert!(!g.camps.contains_key(&camp), "camp destroyed");
    assert_eq!(
        g.players[me].gold,
        gold + wayciv_core::barbarians::CAMP_REWARD
    );
}

#[test]
fn barbarians_cannot_capture_cities() {
    let Some(mut g) = game() else { return };
    let me = g.human();
    let barbarian = g.barbarian().unwrap();
    let (a, b) = land_pair(&g, 1);
    let city = g.found_city_at(me, a, Some("Target".into()));
    g.cities.get_mut(&city).unwrap().hp = 1;
    let raider = g.spawn(barbarian, "UNIT_WARRIOR", b).unwrap();
    let _ = g.attack(raider, a);
    assert_eq!(g.cities[&city].owner, me);
}

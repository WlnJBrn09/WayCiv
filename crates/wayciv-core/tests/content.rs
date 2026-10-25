//! WayCiv's own content: the calendar, year-gated governments, policies,
//! and buildings, effects that apply, and leader lines.

use wayciv_core::diplomacy::DiploEvent;
use wayciv_core::rules::{FOOD, REAGAN};
use wayciv_core::state::{Game, Production};
use wayciv_core::{GameLength, MapType, Settings, load_rules, new_game};

fn game_with(length: GameLength) -> Game {
    new_game(
        load_rules().expect("content loads"),
        Settings {
            seed: 3,
            map_type: MapType::Random,
            length,
            ..Settings::default()
        },
    )
}

#[test]
fn calendar_runs_from_4000_bc_to_about_2150_for_every_length() {
    for length in GameLength::ALL {
        let Some(last) = length.turns() else { continue };
        let g = game_with(length);
        assert_eq!(g.calendar_year(1), -4000);
        let end = g.calendar_year(last);
        assert!((2140..=2160).contains(&end), "{length:?} ends in {end}");
        let years: Vec<i32> = (1..=last).map(|t| g.calendar_year(t)).collect();
        assert!(
            years.windows(2).all(|w| w[0] <= w[1]),
            "{length:?} goes backward"
        );
        assert!(g.turn_of_year(1000) < last);
        assert!(g.calendar_year(g.turn_of_year(1000)) >= 1000);
    }
}

#[test]
fn governments_and_policies_unlock_by_year() {
    let mut g = game_with(GameLength::Turns500);
    let me = g.human();
    let name = |g: &Game, i: usize| g.rules.content.governments[i].name.clone();
    let at_start: Vec<String> = g
        .available_governments(me)
        .into_iter()
        .map(|i| name(&g, i))
        .collect();
    assert_eq!(at_start, ["Chiefdom"]);
    let democracy = g.rules.idx("GOVERNMENT_DEMOCRACY").unwrap();
    assert!(!g.available_governments(me).contains(&democracy));
    g.turn = g.turn_of_year(1790);
    assert!(g.available_governments(me).contains(&democracy));

    // A successor replaces its predecessor once it unlocks.
    let war_chants = g.rules.idx("POLICY_WAR_CHANTS").unwrap();
    let mut g = game_with(GameLength::Turns500);
    assert!(g.available_policies(me).contains(&war_chants));
    g.turn = g.turn_of_year(-200);
    assert!(!g.available_policies(me).contains(&war_chants));
}

#[test]
fn reaching_an_unlock_year_grants_an_envoy_and_a_free_policy_change() {
    let mut g = game_with(GameLength::Turns500);
    let me = g.human();
    // Jump to the turn before Autocracy (2500 BC) and end it.
    g.turn = g.turn_of_year(-2500) - 1;
    g.players[me].policy_change_free = false;
    let envoys = g.players[me].envoys_available;
    g.end_turn();
    assert!(g.year() >= -2500);
    assert!(g.players[me].policy_change_free);
    assert!(g.players[me].envoys_available > envoys);
    assert!(
        g.notifications.iter().any(|n| n.text.contains("Autocracy")),
        "players hear about the new government"
    );
}

#[test]
fn year_gated_buildings_wait_for_their_year() {
    let mut g = game_with(GameLength::Turns500);
    let me = g.human();
    let settler = g
        .player_units(me)
        .find(|u| g.unit_type(u).found_city)
        .unwrap()
        .id;
    g.found_city(settler).unwrap();
    let city = g.player_cities(me).next().unwrap().id;
    let temple = g.rules.idx("BUILDING_TEMPLE").unwrap();
    let shrine = g.rules.idx("BUILDING_SHRINE").unwrap();
    let holy_site = g.rules.idx("DISTRICT_HOLY_SITE").unwrap();
    let pos = g.cities[&city].pos;
    let c = g.cities.get_mut(&city).unwrap();
    c.districts.push(wayciv_core::state::CityDistrict {
        district: holy_site,
        pos,
    });
    c.buildings.push(shrine);
    assert!(!g.can_produce(city, &Production::Building(temple)));
    g.turn = g.turn_of_year(-800);
    assert!(g.can_produce(city, &Production::Building(temple)));
}

#[test]
fn policies_and_abilities_change_yields() {
    let mut g = game_with(GameLength::Turns500);
    let me = g.human();
    let chiefdom = g.rules.idx("GOVERNMENT_CHIEFDOM").unwrap();
    g.set_government(me, chiefdom).unwrap();
    let before = g.bonuses(me).city_flat[FOOD];
    let granaries = g.rules.idx("POLICY_TRIBAL_GRANARIES").unwrap();
    g.set_policies(me, vec![granaries]).unwrap();
    assert_eq!(g.bonuses(me).city_flat[FOOD], before + 1.0);

    // Reagan leads America with his own ability, now written as data.
    assert_eq!(g.rules.content.leaders[g.players[me].leader].id, REAGAN);
    let b = g.bonuses(me);
    assert_eq!(b.combat_in_territory, 5);
    assert!(
        b.unit_production
            .iter()
            .any(|u| u.military_only && u.percent == 15.0)
    );
}

#[test]
fn leaders_speak_their_own_lines() {
    let g = game_with(GameLength::Turns500);
    let me = g.human();
    assert!(
        g.dialogue(me, DiploEvent::Greeting)
            .contains("morning in America")
    );
    for l in &g.rules.content.leaders {
        assert!(
            l.lines.contains_key("first_meet"),
            "{} has no first_meet",
            l.id
        );
        assert!(l.agenda.is_some(), "{} has no agenda", l.id);
        assert!(!l.traits.is_empty(), "{} has no ability", l.id);
    }
    for c in &g.rules.content.civilizations {
        assert!(c.city_names.len() >= 10, "{} needs more city names", c.id);
    }
}

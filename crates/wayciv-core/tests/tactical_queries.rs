//! Tests for read-only tactical and strategic query helpers used by the HUD.

use wayciv_core::{MapSize, MapType, Settings, new_game};

fn test_game() -> wayciv_core::Game {
    let rules = wayciv_core::load_rules().expect("rules load");
    new_game(
        rules,
        Settings {
            seed: 42,
            map_type: MapType::Random,
            size: MapSize::Small,
            opponents: 3,
            city_states: 2,
            ..Default::default()
        },
    )
}

#[test]
fn combat_breakdown_reports_modifiers_and_hp_deltas() {
    let mut g = test_game();
    let me = g.human();
    let foe = (0..g.players.len())
        .find(|&p| p != me && g.players[p].is_major())
        .unwrap();
    g.players[me].relations[foe].met = true;
    g.players[foe].relations[me].met = true;
    g.players[me].relations[foe].at_war = true;
    g.players[foe].relations[me].at_war = true;

    let grid = g.map.grid;
    let a = grid
        .all()
        .find(|&p| {
            let t = &g.rules.content.terrains[g.map.tile(p).terrain];
            !t.water && !t.mountain && g.units_at(p).next().is_none()
        })
        .unwrap();
    let b = grid
        .neighbors(a)
        .find(|&p| {
            let t = &g.rules.content.terrains[g.map.tile(p).terrain];
            !t.water && !t.mountain && g.units_at(p).next().is_none()
        })
        .unwrap();

    let attacker = g.spawn(me, "UNIT_WARRIOR", a).unwrap();
    let defender = g.spawn(foe, "UNIT_WARRIOR", b).unwrap();
    g.units.get_mut(&defender).unwrap().fortified = true;

    let bd = g
        .combat_breakdown(attacker, b)
        .expect("combat breakdown should exist for adjacent enemy");
    assert!(bd.preview.attack > 0);
    assert!(
        bd.preview.defense > bd.preview.attack,
        "fortified defender should exceed base attack"
    );
    assert!(bd.preview.damage_to_defender > 0);
    assert!(bd.preview.damage_to_attacker > 0);
    assert!(
        bd.defender_mods
            .iter()
            .any(|m| m.label.contains("Fortified") && m.delta > 0),
        "expected Fortified modifier in {:?}",
        bd.defender_mods
    );
}

#[test]
fn path_step_turns_is_monotonic_and_starts_at_turn_one() {
    let g = test_game();
    let me = g.human();
    let unit = g.player_units(me).next().expect("starting unit").clone();
    let grid = g.map.grid;
    let target = grid
        .within(unit.pos, 4)
        .into_iter()
        .filter(|&p| grid.distance(unit.pos, p) >= 3)
        .find(|&p| g.find_path(&unit, p).is_some())
        .expect("reachable multi-turn tile");
    let path = g.find_path(&unit, target).unwrap();
    let turns = g.path_step_turns(&unit, &path);
    assert_eq!(turns.len(), path.len());
    assert_eq!(turns[0], 1);
    assert!(
        *turns.last().unwrap() >= 2,
        "3+ tile walk should take multiple turns"
    );
    for w in turns.windows(2) {
        assert!(
            w[1] >= w[0],
            "turn numbers must be non-decreasing: {turns:?}"
        );
    }
}

#[test]
fn settle_housing_matches_water_availability() {
    let g = test_game();
    let me = g.human();
    let grid = g.map.grid;
    for pos in grid.all() {
        let h = g.settle_water_housing(pos);
        assert!(matches!(h, 2 | 3 | 5), "unexpected water housing {h}");
        if g.is_fresh_water(pos) {
            assert_eq!(h, 5);
        }
        let t = &g.rules.content.terrains[g.map.tile(pos).terrain];
        if t.water || t.mountain {
            assert!(g.can_settle_pos(me, pos).is_err());
        }
    }
}

#[test]
fn adjacency_breakdown_sums_to_tile_adjacency() {
    let g = test_game();
    let campus = g.rules.idx("DISTRICT_CAMPUS").expect("campus exists");
    let grid = g.map.grid;
    for pos in grid.all().take(80) {
        let total = g.adjacency(campus, pos);
        let parts = g.adjacency_breakdown(campus, pos);
        let mut sum = [0.0f32; 6];
        for (yi, v, _) in parts {
            sum[yi] += v as f32;
        }
        for i in 0..6 {
            assert!(
                (total[i] - sum[i]).abs() < 0.01,
                "mismatch on yield {i} at {pos:?}: total={} sum={}",
                total[i],
                sum[i]
            );
        }
    }
}

//! Starting a new game.

use crate::hex::Pos;
use crate::map::{self, PlayerId};
use crate::rng::Rng;
use crate::rules::Rules;
use crate::state::{Game, Player, PlayerKind, Relation, Settings};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// Distinct fallback colors for players whose Civ VI colors clash.
const FALLBACK: [[u8; 3]; 12] = [
    [200, 40, 40],
    [40, 90, 200],
    [240, 200, 40],
    [40, 160, 70],
    [150, 60, 180],
    [240, 130, 30],
    [30, 180, 190],
    [230, 90, 160],
    [120, 80, 40],
    [240, 240, 240],
    [90, 90, 90],
    [160, 200, 60],
];

pub fn new_game(rules: Arc<Rules>, settings: Settings) -> Game {
    let mut rng = Rng::new(settings.seed);
    let map = match settings.map_type {
        map::MapType::Earth => crate::earth::generate(&rules, settings.size, &mut rng),
        map::MapType::Random => map::generate(&rules, settings.size, &mut rng),
    };
    let content = &rules.content;

    // Majors: the human's chosen leader, then distinct civilizations.
    let human = rules.idx(&settings.human_leader).filter(|&i| content.leaders.get(i).is_some_and(|l| l.id == settings.human_leader)).unwrap_or(0);
    let mut leaders = vec![human];
    let mut pool: Vec<usize> = (0..content.leaders.len())
        .filter(|&i| content.leaders[i].civilization != content.leaders[human].civilization)
        .collect();
    rng.shuffle(&mut pool);
    for leader in pool {
        if leaders.len() > settings.opponents {
            break;
        }
        if leaders.iter().all(|&l| content.leaders[l].civilization != content.leaders[leader].civilization) {
            leaders.push(leader);
        }
    }
    let mut states: Vec<usize> = (0..content.city_states.len()).collect();
    rng.shuffle(&mut states);
    states.truncate(settings.city_states);

    // Majors, city-states, and the barbarians.
    let total = leaders.len() + states.len() + 1;
    let starts = match settings.map_type {
        // Everyone starts at home: leaders at their historical capitals,
        // city-states at their real cities.
        map::MapType::Earth => {
            let homes: Vec<Option<(f64, f64)>> = leaders
                .iter()
                .map(|&l| crate::earth::home(&content.leaders[l].id))
                .chain(states.iter().map(|&s| crate::earth::home(&content.city_states[s].id)))
                .collect();
            crate::earth::starting_positions(&rules, &map, settings.size, &homes, crate::actions::MIN_CITY_DISTANCE, &mut rng)
        }
        map::MapType::Random => map::starting_positions(&rules, &map, total - 1, 9, &mut rng),
    };

    let grid = map.grid;
    let tech_count = content.techs.len();
    let civic_count = content.civics.len();
    let ai_count = rules.ai_tree.len();
    let blank = |id: PlayerId, kind: PlayerKind, leader: usize, civ: usize, name: String, civ_name: String| Player {
        id,
        kind,
        leader,
        civ,
        name,
        civ_name,
        human: false,
        alive: true,
        primary: [128; 3],
        secondary: [255; 3],
        gold: 10,
        faith: 0,
        techs: vec![false; tech_count],
        tech_progress: vec![0; tech_count],
        tech_boosted: vec![false; tech_count],
        research: None,
        civics: vec![false; civic_count],
        civic_progress: vec![0; civic_count],
        civic_boosted: vec![false; civic_count],
        civic: None,
        ai_techs: vec![false; ai_count],
        ai_progress: vec![0; ai_count],
        ai_research: None,
        government: None,
        policies: Vec::new(),
        policy_change_free: false,
        pantheon: None,
        religion: None,
        beliefs: Vec::new(),
        great_person_points: HashMap::new(),
        influence: 0,
        envoys_available: 0,
        envoys: HashMap::new(),
        relations: vec![Relation::default(); total],
        visible: vec![false; grid.len()],
        revealed: vec![false; grid.len()],
        capital: None,
        original_capital: None,
        cities_founded: 0,
        culture_total: 0,
        tourism_total: 0,
        great_works: 0,
        space_projects: Vec::new(),
        units_killed: 0,
        barbarians_killed: 0,
        city_state_category: String::new(),
    };

    let mut players = Vec::new();
    for (i, &leader) in leaders.iter().enumerate() {
        let l = &content.leaders[leader];
        let civ = rules.idx(&l.civilization).unwrap_or(0);
        let mut player = blank(i, PlayerKind::Major, leader, civ, l.name.clone(), content.civilizations[civ].name.clone());
        player.human = i == 0;
        player.primary = l.primary_color;
        player.secondary = l.secondary_color;
        players.push(player);
    }
    for &state in &states {
        let cs = &content.city_states[state];
        let id = players.len();
        let mut player = blank(id, PlayerKind::CityState, state, state, cs.name.clone(), cs.name.clone());
        player.city_state_category = cs.category.clone();
        let (primary, secondary) = city_state_colors(&cs.category);
        player.primary = primary;
        player.secondary = secondary;
        players.push(player);
    }
    let barbarian_id = players.len();
    let mut barbarians = blank(barbarian_id, PlayerKind::Barbarian, 0, 0, "Barbarians".into(), "Barbarians".into());
    barbarians.primary = [25, 25, 25];
    barbarians.secondary = [220, 50, 40];
    barbarians.gold = 0;
    players.push(barbarians);
    for p in players.iter_mut() {
        if p.id != barbarian_id {
            p.relations[barbarian_id].at_war = true;
        }
    }
    for r in players[barbarian_id].relations.iter_mut() {
        r.at_war = true;
    }
    distinct_colors(&mut players);

    let mut game = Game {
        rules: rules.clone(),
        settings,
        map,
        players,
        cities: BTreeMap::new(),
        units: BTreeMap::new(),
        turn: 1,
        rng,
        next_city: 1,
        next_unit: 1,
        notifications: Vec::new(),
        winner: None,
        world_wonders: Vec::new(),
        religions: BTreeMap::new(),
        great_people_taken: Vec::new(),
        combat_log: Vec::new(),
        popups: Vec::new(),
        trade_routes: Vec::new(),
        camps: BTreeMap::new(),
    };

    for (player, start) in starts.iter().enumerate().take(game.players.len()) {
        let start = *start;
        if game.players[player].is_city_state() {
            let name = game.players[player].name.clone();
            game.found_city_at(player, start, Some(name));
            game.spawn(player, "UNIT_WARRIOR", start);
        } else {
            game.spawn(player, "UNIT_SETTLER", start);
            let warrior_pos = game.map.grid.neighbors(start).find(|&n| game.map.is_land(&rules, n) && !game.map.is_impassable(&rules, n)).unwrap_or(start);
            game.spawn(player, "UNIT_WARRIOR", warrior_pos);
        }
    }
    for player in 0..game.players.len() {
        game.update_visibility(player);
    }
    game.place_starting_camps();
    game.notify(game.human(), "A new age begins. Found your capital with the Settler.", starts.first().copied());
    game
}

fn city_state_colors(category: &str) -> ([u8; 3], [u8; 3]) {
    match category {
        "SCIENTIFIC" => ([30, 110, 200], [230, 240, 255]),
        "CULTURAL" => ([170, 40, 150], [255, 230, 250]),
        "MILITARISTIC" => ([180, 40, 30], [255, 225, 220]),
        "RELIGIOUS" => ([240, 240, 240], [60, 60, 60]),
        "TRADE" => ([230, 190, 30], [70, 50, 0]),
        "INDUSTRIAL" => ([200, 110, 30], [255, 240, 220]),
        _ => ([120, 120, 120], [255, 255, 255]),
    }
}

/// Keeps every major player's primary color clearly distinct.
fn distinct_colors(players: &mut [Player]) {
    let close = |a: [u8; 3], b: [u8; 3]| {
        let d: i32 = (0..3).map(|i| (a[i] as i32 - b[i] as i32).pow(2)).sum();
        d < 70 * 70
    };
    for i in 0..players.len() {
        if !players[i].is_major() {
            continue;
        }
        let clashes = |c: [u8; 3], players: &[Player]| players[..i].iter().any(|p| p.is_major() && close(p.primary, c));
        if (clashes(players[i].primary, players) || players[i].primary == [200, 200, 200])
            && let Some(c) = FALLBACK.iter().copied().find(|&c| !clashes(c, players)) {
                players[i].primary = c;
                let lum = c[0] as i32 * 3 + c[1] as i32 * 6 + c[2] as i32;
                players[i].secondary = if lum > 1500 { [30, 30, 30] } else { [255, 255, 255] };
            }
    }
}

impl Game {
    pub fn spawn(&mut self, owner: PlayerId, unit_id: &str, pos: Pos) -> Option<u32> {
        let rules = self.rules.clone();
        let p = &self.players[owner];
        let civ_id = &rules.content.civilizations.get(p.civ).map(|c| c.id.clone()).unwrap_or_default();
        let leader_id = rules.content.leaders.get(p.leader).map(|l| l.id.clone()).unwrap_or_default();
        let actual = if !p.is_major() { unit_id.to_string() } else { rules.unit_for(unit_id, civ_id, &leader_id) };
        let kind = rules.idx(&actual).filter(|&i| rules.content.units.get(i).is_some_and(|u| u.id == actual))?;
        let ty = &rules.content.units[kind];
        let id = self.next_unit;
        self.next_unit += 1;
        self.units.insert(
            id,
            crate::state::Unit {
                id,
                owner,
                kind,
                pos,
                moves: ty.moves,
                hp: 100,
                xp: 0,
                level: 0,
                fortified: false,
                fortify_turns: 0,
                sleeping: false,
                charges: ty.build_charges.max(ty.spread_charges),
                religion: None,
                great_person: None,
                path: Vec::new(),
                attacked: false,
                skipped: false,
            },
        );
        Some(id)
    }

    /// Seeds a game's RNG separately (used by tests).
    pub fn reseed(&mut self, seed: u64) {
        self.rng = Rng::new(seed);
    }
}

//! Plays an all-AI game headlessly and prints a summary every 25 turns.
//!     cargo run --release -p wayciv-core --example autoplay -- [turns] [seed]
//! Env: SIZE=0|1|2, OPPONENTS, CITY_STATES, LENGTH=100|250|500|750|1000|infinite.
use wayciv_core::{Settings, load_rules, new_game};

fn main() -> wayciv_data::Result<()> {
    let args: Vec<u64> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let turns = args.first().copied().unwrap_or(150) as u32;
    let rules = load_rules()?;
    let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<usize>().ok());
    let size = match env("SIZE") {
        Some(0) => wayciv_core::MapSize::Small,
        Some(1) => wayciv_core::MapSize::Standard,
        _ => wayciv_core::MapSize::Large,
    };
    let settings = Settings {
        seed: args.get(1).copied().unwrap_or(7),
        size,
        opponents: env("OPPONENTS").unwrap_or(9),
        city_states: env("CITY_STATES").unwrap_or(12),
        length: std::env::var("LENGTH")
            .ok()
            .and_then(|l| wayciv_core::GameLength::parse(&l))
            .unwrap_or_default(),
        ..Settings::default()
    };
    let mut game = new_game(rules, settings);
    let starts: Vec<String> = game
        .players
        .iter()
        .filter(|p| !p.is_barbarian())
        .map(|p| {
            let pos = game
                .player_units(p.id)
                .next()
                .map(|u| u.pos)
                .or_else(|| game.player_cities(p.id).next().map(|c| c.pos))
                .unwrap_or_default();
            format!("{}@{pos}", p.civ_name)
        })
        .collect();
    println!("starts: {}", starts.join(", "));
    // The "human" is played by the AI too.
    let human = game.human();
    game.players[human].human = false;
    let start = std::time::Instant::now();
    // Rivalry counters: wars declared, cities changing hands, alliances.
    let mut wars = 0;
    let mut captures = 0;
    let mut alliances = 0;
    let mut owners: std::collections::BTreeMap<u32, usize> = Default::default();
    let pairs = |g: &wayciv_core::Game, f: &dyn Fn(&wayciv_core::state::Relation) -> bool| {
        let mut n = 0;
        for a in g.players.iter().filter(|p| p.is_major()) {
            for b in g.players.iter().filter(|p| p.is_major() && p.id > a.id) {
                n += usize::from(f(&a.relations[b.id]));
            }
        }
        n
    };
    let mut at_war = 0;
    let mut allied = 0;
    while game.turn <= turns && game.winner.is_none() {
        game.end_turn();
        let now_war = pairs(&game, &|r| r.at_war);
        wars += now_war.saturating_sub(at_war);
        at_war = now_war;
        let now_allied = pairs(&game, &|r| r.alliance);
        alliances += now_allied.saturating_sub(allied);
        allied = now_allied;
        for c in game.cities.values() {
            if let Some(&o) = owners.get(&c.id)
                && o != c.owner
            {
                captures += 1;
            }
            owners.insert(c.id, c.owner);
        }
        if game.turn.is_multiple_of(25) {
            println!(
                "--- turn {} ({:.1}s)",
                game.turn,
                start.elapsed().as_secs_f32()
            );
            let barbarian = game.barbarian().unwrap();
            println!(
                "camps={} barbarian units={} roads={} villages={} era={} | wars declared={wars} at war now={at_war} cities changed hands={captures} alliances={alliances} (now {allied})",
                game.camps.len(),
                game.player_units(barbarian).count(),
                game.map.tiles.iter().filter(|t| t.road).count(),
                game.villages.len(),
                game.era_name(game.world_era),
            );
            for p in game.players.iter().filter(|p| p.is_major()) {
                let air = game.player_units(p.id).filter(|u| game.is_air(u)).count();
                let income = game.income(p.id);
                let ai = p.ai_techs.iter().filter(|t| **t).count();
                let drones = game.player_units(p.id).filter(|u| game.is_drone(u)).count();
                let loyalty_low = game.player_cities(p.id).filter(|c| c.loyalty < 50).count();
                println!(
                    "    {} science {:.0}/turn culture {:.0}/turn compute {:.0} ai-tree {ai} drones {drones} low-loyalty {loyalty_low}, trade routes {}/{}, aircraft {}, barbarians killed {}",
                    p.civ_name,
                    income.yields[3],
                    income.yields[4],
                    game.compute(p.id),
                    game.trade_routes_of(p.id).count(),
                    game.trade_capacity(p.id),
                    air,
                    p.barbarians_killed
                );
            }
            for p in game.players.iter().filter(|p| p.is_major()) {
                let cities: Vec<_> = game.player_cities(p.id).collect();
                let pop: i32 = cities.iter().map(|c| c.population).sum();
                let techs = p.techs.iter().filter(|t| **t).count();
                let policies = p.policies.len();
                let units = game.player_units(p.id).count();
                let wars = game.real_wars(p.id);
                let ages: String = p
                    .era
                    .past
                    .iter()
                    .map(|(_, s, a)| format!("{}{s}", &a.name()[..1]))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!(
                    "{:<24} {:<12} alive={} cities={:>2} pop={:>3} techs={:>2} cards={:>2} slotted={} era score={:>3} ages=[{ages}] units={:>3} gold={:>5} wars={} gov={} score={}",
                    p.name,
                    p.civ_name,
                    p.alive,
                    cities.len(),
                    pop,
                    techs,
                    p.cards.count(),
                    policies,
                    p.era.score,
                    units,
                    p.gold,
                    wars,
                    p.government
                        .map(|g| game.rules.content.governments[g].name.clone())
                        .unwrap_or_default(),
                    game.score(p.id)
                );
            }
        }
    }
    println!(
        "winner: {:?} at turn {}",
        game.winner
            .as_ref()
            .map(|(p, v)| (game.players[*p].name.clone(), v.clone())),
        game.turn
    );
    Ok(())
}

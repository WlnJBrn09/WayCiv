use wayciv_core::{Settings, load_rules, new_game};
fn main() -> civ6_data::Result<()> {
    let rules = load_rules()?;
    let mut game = new_game(rules, Settings { seed: 7, ..Settings::default() });
    let human = game.human();
    game.players[human].human = false;
    let b = game.barbarian().unwrap();
    for _ in 0..40 {
        let before: Vec<_> = game.camps.keys().copied().collect();
        game.end_turn();
        let units: Vec<_> = game.player_units(b).map(|u| (u.pos, u.hp)).collect();
        let gone: Vec<_> = before.iter().filter(|c| !game.camps.contains_key(c)).collect();
        println!("T{} camps={} gone={:?} units={:?} alive={}", game.turn, game.camps.len(), gone, units, game.players[b].alive);
    }
    Ok(())
}

use wayciv_core::{Settings, load_rules, new_game};
fn main() -> wayciv_data::Result<()> {
    let rules = load_rules()?;
    let mut game = new_game(
        rules,
        Settings {
            seed: 7,
            ..Settings::default()
        },
    );
    let human = game.human();
    game.players[human].human = false;
    for _ in 0..150 {
        game.end_turn();
    }
    for c in game
        .cities
        .values()
        .filter(|c| game.players[c.owner].is_major())
        .take(12)
    {
        let r = game.city_report(c.id);
        println!(
            "{:<14} pop {:>2} food {:>5.1}-{:>4.1} housing {:>2} amen {}/{} prod {:>4.1} sci {:>4.1} buildings {} districts {} worked {}",
            c.name,
            c.population,
            r.yields[0],
            r.food_consumed,
            r.housing,
            r.amenities,
            r.amenities_needed,
            r.yields[1],
            r.yields[3],
            c.buildings.len(),
            c.districts.len(),
            c.worked.len()
        );
    }
    Ok(())
}

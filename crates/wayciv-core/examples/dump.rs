fn main() {
    let rules = wayciv_core::load_rules().unwrap();
    let c = &rules.content;
    let what = std::env::args().nth(1).unwrap_or_default();
    match what.as_str() {
        "terrain" => {
            for t in &c.terrains {
                println!(
                    "{} {:?} mv{} def{} hills{} mtn{} water{}",
                    t.id,
                    t.yields,
                    t.movement_cost,
                    t.defense_modifier,
                    t.hills,
                    t.mountain,
                    t.water
                )
            }
        }
        "feature" => {
            for f in &c.features {
                println!(
                    "{} [{}] {:?} nw{} imp{} {:?}",
                    f.id, f.name, f.yields, f.natural_wonder, f.impassable, f.valid_terrains
                )
            }
        }
        "resource" => {
            for r in &c.resources {
                println!("{} [{}] {} {:?}", r.id, r.name, r.class, r.valid_terrains)
            }
        }
        "unit" => {
            for u in &c.units {
                println!(
                    "{} [{}] {} c{} r{} {:?} {:?} uniq{:?} cost{}",
                    u.id,
                    u.name,
                    u.domain,
                    u.combat,
                    u.ranged_combat,
                    u.prereq_tech,
                    u.unlock_year,
                    u.unique_to,
                    u.cost
                )
            }
        }
        "tech" => {
            for t in &c.techs {
                println!(
                    "{} [{}] {} {} row{} {:?}",
                    t.id, t.name, t.era, t.cost, t.tree_row, t.prereqs
                )
            }
        }
        "district" => {
            for d in &c.districts {
                println!(
                    "{} [{}] {:?} {:?} cost{} uniq{:?}",
                    d.id, d.name, d.prereq_tech, d.unlock_year, d.cost, d.unique_to
                )
            }
        }
        "gp" => {
            for g in &c.great_person_classes {
                println!("{:?}", g)
            }
            for g in &c.great_people {
                println!(
                    "{} [{}] {} {} :: {}",
                    g.id, g.name, g.class, g.era, g.ability
                )
            }
        }
        "belief" => {
            for b in &c.beliefs {
                println!("{} [{}] {}", b.id, b.name, b.class)
            }
        }
        "leader" => {
            for l in &c.leaders {
                println!("{} [{}] {} cap={}", l.id, l.name, l.civilization, l.capital)
            }
        }
        "cs" => {
            for s in &c.city_states {
                println!("{} [{}] {}", s.id, s.name, s.category)
            }
        }
        "project" => {
            for p in &c.projects {
                println!(
                    "{} [{}] {:?} {:?} {}",
                    p.id, p.name, p.prereq_tech, p.prereq_district, p.cost
                )
            }
        }
        "building" => {
            for b in &c.buildings {
                println!(
                    "{} [{}] {:?} {:?} {:?} w{}",
                    b.id, b.name, b.prereq_tech, b.unlock_year, b.prereq_district, b.wonder
                )
            }
        }
        "improvement" => {
            for b in &c.improvements {
                println!("{} [{}] {:?}", b.id, b.name, b.prereq_tech)
            }
        }
        _ => {}
    }
}

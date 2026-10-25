//! Prints the icons WayCiv wants, one per line: kind, id, prompt subject.
//! Used by tools/icons/generate.sh.
fn main() {
    let rules = wayciv_core::load_rules().unwrap();
    let c = &rules.content;
    let clean = |s: &str| {
        s.replace(['\t', '\n'], " ")
            .chars()
            .take(220)
            .collect::<String>()
    };
    let era = |e: &str| {
        rules
            .content
            .eras
            .iter()
            .find(|x| x.id == e)
            .map(|x| x.name.trim_end_matches(" Era").to_string())
            .unwrap_or_default()
    };
    for r in &c.resources {
        let class = r.class.trim_start_matches("RESOURCECLASS_").to_lowercase();
        println!(
            "resource\t{}\tthe {} natural resource \"{}\" shown as the raw resource itself",
            r.id, class, r.name
        );
    }
    for u in &c.units {
        let tech_era = u
            .prereq_tech
            .as_deref()
            .and_then(|t| rules.tech(t))
            .map(|i| c.techs[i].era.clone())
            .or_else(|| u.unlock_year.map(|y| rules.era_of_year(y)))
            .unwrap_or_else(|| "ERA_ANCIENT".into());
        let owner = u
            .unique_to
            .as_deref()
            .map(|o| {
                let civ = rules.civilization(o).map(|x| x.name.clone()).or_else(|| {
                    rules.leader(o).map(|l| {
                        format!(
                            "{} of {}",
                            l.name,
                            rules
                                .civilization(&l.civilization)
                                .map(|x| x.name.clone())
                                .unwrap_or_default()
                        )
                    })
                });
                format!(", the unique unit of {}", civ.unwrap_or_default())
            })
            .unwrap_or_default();
        println!(
            "unit\t{}\tthe \"{}\" unit ({} era{}): {} Show one representative soldier, vehicle, ship or person",
            u.id,
            u.name,
            era(&tech_era),
            owner,
            clean(&u.description)
        );
    }
    for t in &c.techs {
        println!(
            "tech\t{}\tthe technology \"{}\" ({} era) as a symbolic emblem",
            t.id,
            t.name,
            era(&t.era)
        );
    }
    for t in wayciv_core::additions::AI_TREE.iter() {
        println!(
            "ai\t{}\tthe artificial intelligence research \"{}\" as a symbolic emblem, glowing cyan",
            t.id, t.name
        );
    }
    for (id, subject) in [
        (
            "TREE_TECH",
            "the Technology Tree: a glowing gear and lightbulb emblem, blue tones",
        ),
        (
            "TREE_RELIGION",
            "the Religion Tree: a radiant sunburst and candle flame emblem, white and gold tones",
        ),
        (
            "TREE_AI",
            "the Artificial Intelligence Tree: a glowing microchip brain emblem, cyan tones",
        ),
    ] {
        println!("tree\t{id}\t{subject}");
    }
    for b in &c.beliefs {
        let class = b.class.trim_start_matches("BELIEF_CLASS_").to_lowercase();
        println!(
            "belief\t{}\tthe {} religious belief \"{}\": {} Symbolic, respectful, no specific real deity's face",
            b.id,
            class,
            b.name,
            clean(&b.description)
        );
    }
}

//! Prints a summary of the content loaded from the installed game.
fn main() -> civ6_data::Result<()> {
    let content = civ6_data::content::Content::load_installed()?;
    println!(
        "terrains {} features {} resources {} improvements {} civs {} city-states {} leaders {} units {} buildings {} districts {} techs {} civics {} governments {} policies {} beliefs {} religions {} great people {} projects {} modifiers {} text {}",
        content.terrains.len(),
        content.features.len(),
        content.resources.len(),
        content.improvements.len(),
        content.civilizations.len(),
        content.city_states.len(),
        content.leaders.len(),
        content.units.len(),
        content.buildings.len(),
        content.districts.len(),
        content.techs.len(),
        content.civics.len(),
        content.governments.len(),
        content.policies.len(),
        content.beliefs.len(),
        content.religions.len(),
        content.great_people.len(),
        content.projects.len(),
        content.modifiers.len(),
        content.text.len()
    );
    for leader in &content.leaders {
        let civ = content
            .civilizations
            .iter()
            .find(|c| c.id == leader.civilization)
            .map(|c| c.name.as_str())
            .unwrap_or("?");
        println!(
            "{:<34} {:<26} {:<12} {:?} {:?} traits={}",
            leader.id,
            leader.name,
            civ,
            leader.primary_color,
            leader.secondary_color,
            leader
                .traits
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>()
                .join(" / ")
        );
    }
    Ok(())
}

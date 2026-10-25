//! The data files parse, every reference resolves, and descriptions are filled in.

#[test]
fn content_loads_and_validates() {
    let content = wayciv_data::Content::load().unwrap_or_else(|e| panic!("{e}"));
    assert!(content.leaders.len() >= 30);
    assert!(content.techs.len() >= 60);
    assert!(content.policies.len() >= 80);
    assert!(content.buildings.iter().filter(|b| b.wonder).count() >= 30);
    for p in &content.policies {
        assert!(!p.description.is_empty(), "{} has no description", p.id);
    }
}

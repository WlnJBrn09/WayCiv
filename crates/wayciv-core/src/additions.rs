//! WayCiv's own content on top of Civ VI's: more biomes (Taiga, Dunes,
//! Glacier, grassland and plains Floodplains, Deep Sea), the Artificial
//! Intelligence tech with its own AI tree, Datacenter districts, Anduril
//! Strike Drones, and modern great people.

use civ6_data::content::{Adjacency, Building, Content, District, Feature, GreatPerson, Research, Terrain};

pub const TECH_AI: &str = "TECH_ARTIFICIAL_INTELLIGENCE";
pub const DATACENTER: &str = "DISTRICT_DATACENTER";
pub const GPU_CLUSTER: &str = "BUILDING_GPU_CLUSTER";
pub const SOLAR_FARM: &str = "BUILDING_SOLAR_FARM";
pub const STRIKE_DRONE: &str = "UNIT_ANDURIL_STRIKE_DRONE";
/// Strike drones roll off the line three at a time.
pub const DRONES_PER_BATCH: usize = 3;

pub const ELON_MUSK: &str = "GREAT_PERSON_INDIVIDUAL_ELON_MUSK";
pub const GWYNNE_SHOTWELL: &str = "GREAT_PERSON_INDIVIDUAL_GWYNNE_SHOTWELL";
pub const PALMER_LUCKEY: &str = "GREAT_PERSON_INDIVIDUAL_PALMER_LUCKEY";
pub const ILYA_SUTSKEVER: &str = "GREAT_PERSON_INDIVIDUAL_ILYA_SUTSKEVER";

/// One research in the AI tree: researched with Compute from Datacenters
/// once Artificial Intelligence is known.
pub struct AiTech {
    pub id: &'static str,
    pub name: &'static str,
    pub cost: i32,
    pub prereqs: &'static [&'static str],
    pub row: i32,
    pub description: &'static str,
}

pub const AI_MACHINE_LEARNING: &str = "AI_MACHINE_LEARNING";
pub const AI_COMPUTER_VISION: &str = "AI_COMPUTER_VISION";
pub const AI_NEURAL_NETWORKS: &str = "AI_NEURAL_NETWORKS";
pub const AI_AUTONOMOUS_SYSTEMS: &str = "AI_AUTONOMOUS_SYSTEMS";
pub const AI_DEEP_LEARNING: &str = "AI_DEEP_LEARNING";
pub const AI_ROBOTIC_AUTOMATION: &str = "AI_ROBOTIC_AUTOMATION";
pub const AI_LARGE_LANGUAGE_MODELS: &str = "AI_LARGE_LANGUAGE_MODELS";
pub const AI_RESEARCH_AGENTS: &str = "AI_RESEARCH_AGENTS";
pub const AI_AUTOMATED_SCIENCE: &str = "AI_AUTOMATED_SCIENCE";
pub const AI_SUPERINTELLIGENCE: &str = "AI_SUPERINTELLIGENCE";

pub const AI_TREE: [AiTech; 10] = [
    AiTech { id: AI_MACHINE_LEARNING, name: "Machine Learning", cost: 120, prereqs: &[], row: 0, description: "+15% Science in every city." },
    AiTech { id: AI_COMPUTER_VISION, name: "Computer Vision", cost: 220, prereqs: &[AI_MACHINE_LEARNING], row: -1, description: "+3 Combat Strength for all units." },
    AiTech { id: AI_NEURAL_NETWORKS, name: "Neural Networks", cost: 220, prereqs: &[AI_MACHINE_LEARNING], row: 1, description: "Campuses give +3 Science. +2 Compute from every Datacenter." },
    AiTech { id: AI_AUTONOMOUS_SYSTEMS, name: "Autonomous Systems", cost: 360, prereqs: &[AI_COMPUTER_VISION], row: -1, description: "Unlocks Anduril Strike Drones, built three at a time." },
    AiTech { id: AI_DEEP_LEARNING, name: "Deep Learning", cost: 360, prereqs: &[AI_NEURAL_NETWORKS], row: 1, description: "+20% Science. Unlocks the GPU Cluster (+8 Compute) in Datacenters." },
    AiTech { id: AI_ROBOTIC_AUTOMATION, name: "Robotic Automation", cost: 520, prereqs: &[AI_AUTONOMOUS_SYSTEMS], row: -1, description: "+20% Production in every city." },
    AiTech { id: AI_LARGE_LANGUAGE_MODELS, name: "Large Language Models", cost: 520, prereqs: &[AI_DEEP_LEARNING], row: 1, description: "+25% Science and +15% Culture." },
    AiTech { id: AI_RESEARCH_AGENTS, name: "AI Research Agents", cost: 720, prereqs: &[AI_LARGE_LANGUAGE_MODELS], row: 1, description: "+15% Science, and a free Eureka every 5 turns." },
    AiTech { id: AI_AUTOMATED_SCIENCE, name: "Automated Science", cost: 950, prereqs: &[AI_RESEARCH_AGENTS, AI_ROBOTIC_AUTOMATION], row: 0, description: "+25% Science. Space race projects cost 30% less." },
    AiTech { id: AI_SUPERINTELLIGENCE, name: "Superintelligence", cost: 1300, prereqs: &[AI_AUTOMATED_SCIENCE], row: 0, description: "+50% Science and +20% Production." },
];

/// The AI tree as tree items, for the shared research code and tree view.
pub fn ai_tree() -> Vec<Research> {
    AI_TREE
        .iter()
        .map(|t| Research {
            id: t.id.to_string(),
            name: t.name.to_string(),
            cost: t.cost,
            era: "ERA_INFORMATION".to_string(),
            prereqs: t.prereqs.iter().map(|p| p.to_string()).collect(),
            tree_row: t.row,
            boost: None,
        })
        .collect()
}

pub fn apply(content: &mut Content) {
    biomes(content);
    artificial_intelligence(content);
    great_people(content);
}

fn biomes(content: &mut Content) {
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    if let Some(t) = content.terrains.iter_mut().find(|t| t.id == "TERRAIN_OCEAN") {
        t.name = "Sea".into();
    }
    if let Some(ocean) = content.terrains.iter().find(|t| t.id == "TERRAIN_OCEAN").cloned() {
        content.terrains.push(Terrain { id: "TERRAIN_DEEP_OCEAN".into(), name: "Deep Sea".into(), ..ocean });
    }
    let mut floodplains = None;
    let mut forest = None;
    for f in content.features.iter_mut() {
        match f.id.as_str() {
            "FEATURE_FOREST" => {
                f.name = "Forest".into();
                forest = Some(f.clone());
            }
            "FEATURE_FLOODPLAINS" => {
                f.name = "Desert Floodplains".into();
                floodplains = Some(f.clone());
            }
            "FEATURE_MARSH" => {
                for t in ["TERRAIN_PLAINS", "TERRAIN_TUNDRA"] {
                    if !f.valid_terrains.iter().any(|v| v == t) {
                        f.valid_terrains.push(t.into());
                    }
                }
            }
            _ => {}
        }
    }
    let blank = Feature {
        id: String::new(),
        name: String::new(),
        yields: [0; 6],
        movement_change: 0,
        defense_modifier: 0,
        natural_wonder: false,
        impassable: false,
        removable: false,
        forest: false,
        valid_terrains: Vec::new(),
    };
    let flood = floodplains.unwrap_or_else(|| Feature { yields: [3, 0, 0, 0, 0, 0], removable: false, ..blank.clone() });
    content.features.push(Feature {
        id: "FEATURE_FLOODPLAINS_GRASSLAND".into(),
        name: "Floodplains".into(),
        yields: [1, 0, 0, 0, 0, 0],
        valid_terrains: s(&["TERRAIN_GRASS"]),
        ..flood.clone()
    });
    content.features.push(Feature {
        id: "FEATURE_FLOODPLAINS_PLAINS".into(),
        name: "Floodplains".into(),
        yields: [1, 0, 0, 0, 0, 0],
        valid_terrains: s(&["TERRAIN_PLAINS"]),
        ..flood
    });
    let woods = forest.unwrap_or_else(|| Feature { yields: [0, 1, 0, 0, 0, 0], movement_change: 1, defense_modifier: 3, removable: true, forest: true, ..blank.clone() });
    content.features.push(Feature {
        id: "FEATURE_TAIGA".into(),
        name: "Taiga".into(),
        valid_terrains: s(&["TERRAIN_TUNDRA", "TERRAIN_TUNDRA_HILLS", "TERRAIN_PLAINS", "TERRAIN_PLAINS_HILLS"]),
        ..woods
    });
    content.features.push(Feature {
        id: "FEATURE_DUNES".into(),
        name: "Dunes".into(),
        movement_change: 1,
        valid_terrains: s(&["TERRAIN_DESERT"]),
        ..blank.clone()
    });
    content.features.push(Feature {
        id: "FEATURE_GLACIER".into(),
        name: "Glacier".into(),
        impassable: true,
        valid_terrains: s(&["TERRAIN_SNOW", "TERRAIN_SNOW_HILLS"]),
        ..blank
    });

    // What works on Woods works on Taiga; what works on desert Floodplains
    // works on the others.
    let twins = [
        ("FEATURE_FOREST", &["FEATURE_TAIGA"][..]),
        ("FEATURE_FLOODPLAINS", &["FEATURE_FLOODPLAINS_GRASSLAND", "FEATURE_FLOODPLAINS_PLAINS"][..]),
    ];
    for (original, copies) in twins {
        let add = |list: &mut Vec<String>| {
            if list.iter().any(|v| v == original) {
                for c in copies {
                    list.push(c.to_string());
                }
            }
        };
        for r in content.resources.iter_mut() {
            add(&mut r.valid_features);
        }
        for i in content.improvements.iter_mut() {
            add(&mut i.valid_features);
        }
        for d in content.districts.iter_mut() {
            let extra: Vec<Adjacency> = d
                .adjacencies
                .iter()
                .filter(|a| a.adjacent_feature.as_deref() == Some(original))
                .flat_map(|a| copies.iter().map(move |c| Adjacency { adjacent_feature: Some(c.to_string()), ..a.clone() }))
                .collect();
            d.adjacencies.extend(extra);
        }
    }
}

fn artificial_intelligence(content: &mut Content) {
    content.techs.push(Research {
        id: TECH_AI.into(),
        name: "Artificial Intelligence".into(),
        cost: 1950,
        era: "ERA_INFORMATION".into(),
        prereqs: vec!["TECH_TELECOMMUNICATIONS".into()],
        tree_row: -3,
        boost: Some("Build a Research Lab.".into()),
    });
    content.boosts.push(civ6_data::content::Boost {
        tech: Some(TECH_AI.into()),
        civic: None,
        percent: 40,
        class: "BOOST_TRIGGER_HAVE_X_BUILDINGS".into(),
        unit: None,
        building: Some("BUILDING_RESEARCH_LAB".into()),
        improvement: None,
        district: None,
        resource: None,
        boosting_tech: None,
        boosting_civic: None,
        count: 1,
        description: "Build a Research Lab.".into(),
    });

    content.districts.push(District {
        id: DATACENTER.into(),
        name: "Datacenter".into(),
        description: "Powers your artificial intelligence: produces Compute for the AI tree (+6, +2 per adjacent district, +3 next to a river or the coast for cooling). +1 Science per adjacent district.".into(),
        cost: 54,
        prereq_tech: Some(TECH_AI.into()),
        prereq_civic: None,
        requires_population: true,
        city_center: false,
        coast: false,
        one_per_city: true,
        housing: 0,
        amenities: 0,
        great_person_points: vec![("GREAT_PERSON_CLASS_SCIENTIST".into(), 1)],
        adjacencies: vec![Adjacency {
            yield_index: crate::rules::SCIENCE,
            amount: 1,
            tiles_required: 1,
            other_district: true,
            adjacent_terrain: None,
            adjacent_feature: None,
            adjacent_district: None,
            adjacent_river: false,
            adjacent_wonder: false,
            adjacent_resource: false,
            adjacent_natural_wonder: false,
            description: "+1 Science per adjacent district".into(),
        }],
        unique_to: None,
        replaces: None,
        internal_only: false,
    });

    let building = |id: &str, name: &str, description: &str, cost: i32, tech: &str, district: Option<&str>, yields: [i32; 6]| Building {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        cost,
        prereq_tech: Some(tech.into()),
        prereq_civic: None,
        prereq_district: district.map(Into::into),
        prereq_buildings: Vec::new(),
        yields,
        housing: 0,
        amenities: 0,
        maintenance: 2,
        wonder: false,
        outer_defense: 0,
        great_person_points: Vec::new(),
        unique_to: None,
        replaces: None,
        max_world_instances: -1,
        enabled_by_religion: false,
    };
    content.buildings.push(building(
        GPU_CLUSTER,
        "GPU Cluster",
        "+8 Compute and +2 Science. Requires Deep Learning from the AI tree.",
        420,
        TECH_AI,
        Some(DATACENTER),
        [0, 0, 0, 2, 0, 0],
    ));
    content.buildings.push(building(
        SOLAR_FARM,
        "Solar Farm",
        "+3 Production. Powers a Datacenter in this city for +4 Compute.",
        260,
        "TECH_COMPUTERS",
        None,
        [0, 3, 0, 0, 0, 0],
    ));

    let template = content
        .units
        .iter()
        .find(|u| u.id == "UNIT_MACHINE_GUN")
        .or_else(|| content.units.iter().find(|u| u.ranged_combat > 0 && u.domain == "DOMAIN_LAND" && u.unique_to.is_none()))
        .cloned();
    if let Some(template) = template {
        content.units.push(civ6_data::content::Unit {
            id: STRIKE_DRONE.into(),
            name: "Anduril Strike Drone".into(),
            description: "Autonomous loitering strike drone. Built three at a time. Flies over rough terrain (every land tile costs 1 move) and ignores zones of control. Requires Autonomous Systems from the AI tree.".into(),
            cost: 360,
            moves: 5,
            sight: 3,
            combat: 45,
            ranged_combat: 80,
            range: 3,
            bombard: 0,
            prereq_tech: Some(TECH_AI.into()),
            prereq_civic: None,
            unique_to: None,
            replaces: None,
            maintenance: 2,
            obsolete_tech: None,
            strategic_resource: None,
            can_train: true,
            must_purchase: false,
            purchase_yield: None,
            barbarian: false,
            anti_air: 0,
            ..template
        });
    }
}

fn great_people(content: &mut Content) {
    let people = [
        (ELON_MUSK, "Elon Musk", "GREAT_PERSON_CLASS_ENGINEER", "Minor boosts to AI, rockets and solar."),
        (GWYNNE_SHOTWELL, "Gwynne Shotwell", "GREAT_PERSON_CLASS_ENGINEER", "A big boost to rockets."),
        (PALMER_LUCKEY, "Palmer Luckey", "GREAT_PERSON_CLASS_ENGINEER", "Deploys Anduril Strike Drones."),
        (ILYA_SUTSKEVER, "Ilya Sutskever", "GREAT_PERSON_CLASS_SCIENTIST", "A big boost to artificial intelligence."),
        ("GREAT_PERSON_INDIVIDUAL_YANNI", "Yanni", "GREAT_PERSON_CLASS_MUSICIAN", "Creates a Great Work of Music."),
        ("GREAT_PERSON_INDIVIDUAL_NF", "NF", "GREAT_PERSON_CLASS_MUSICIAN", "Creates a Great Work of Music."),
        ("GREAT_PERSON_INDIVIDUAL_MICHAEL_CRICHTON", "Michael Crichton", "GREAT_PERSON_CLASS_WRITER", "Creates a Great Work of Writing."),
        ("GREAT_PERSON_INDIVIDUAL_IAIN_BANKS", "Iain M. Banks", "GREAT_PERSON_CLASS_WRITER", "Creates a Great Work of Writing."),
        ("GREAT_PERSON_INDIVIDUAL_DOUGLAS_ADAMS", "Douglas Adams", "GREAT_PERSON_CLASS_WRITER", "Creates a Great Work of Writing."),
    ];
    for (id, name, class, ability) in people {
        content.great_people.push(GreatPerson {
            id: id.into(),
            name: name.into(),
            class: class.into(),
            era: "ERA_INFORMATION".into(),
            ability: ability.into(),
        });
    }
}

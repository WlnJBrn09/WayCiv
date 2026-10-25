//! The AI tree: researched with Compute from Datacenters once Artificial
//! Intelligence is known. Ids of the content it ties into (the tech,
//! Datacenter, GPU Cluster, Solar Farm, Strike Drones, and the great people
//! with AI abilities) live in the data files.

use wayciv_data::content::Research;

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
    AiTech {
        id: AI_MACHINE_LEARNING,
        name: "Machine Learning",
        cost: 120,
        prereqs: &[],
        row: 0,
        description: "+15% Science in every city.",
    },
    AiTech {
        id: AI_COMPUTER_VISION,
        name: "Computer Vision",
        cost: 220,
        prereqs: &[AI_MACHINE_LEARNING],
        row: -1,
        description: "+3 Combat Strength for all units.",
    },
    AiTech {
        id: AI_NEURAL_NETWORKS,
        name: "Neural Networks",
        cost: 220,
        prereqs: &[AI_MACHINE_LEARNING],
        row: 1,
        description: "Campuses give +3 Science. +2 Compute from every Datacenter.",
    },
    AiTech {
        id: AI_AUTONOMOUS_SYSTEMS,
        name: "Autonomous Systems",
        cost: 360,
        prereqs: &[AI_COMPUTER_VISION],
        row: -1,
        description: "Unlocks Anduril Strike Drones, built three at a time.",
    },
    AiTech {
        id: AI_DEEP_LEARNING,
        name: "Deep Learning",
        cost: 360,
        prereqs: &[AI_NEURAL_NETWORKS],
        row: 1,
        description: "+20% Science. Unlocks the GPU Cluster (+8 Compute) in Datacenters.",
    },
    AiTech {
        id: AI_ROBOTIC_AUTOMATION,
        name: "Robotic Automation",
        cost: 520,
        prereqs: &[AI_AUTONOMOUS_SYSTEMS],
        row: -1,
        description: "+20% Production in every city.",
    },
    AiTech {
        id: AI_LARGE_LANGUAGE_MODELS,
        name: "Large Language Models",
        cost: 520,
        prereqs: &[AI_DEEP_LEARNING],
        row: 1,
        description: "+25% Science and +15% Culture.",
    },
    AiTech {
        id: AI_RESEARCH_AGENTS,
        name: "AI Research Agents",
        cost: 720,
        prereqs: &[AI_LARGE_LANGUAGE_MODELS],
        row: 1,
        description: "+15% Science, and a free Eureka every 5 turns.",
    },
    AiTech {
        id: AI_AUTOMATED_SCIENCE,
        name: "Automated Science",
        cost: 950,
        prereqs: &[AI_RESEARCH_AGENTS, AI_ROBOTIC_AUTOMATION],
        row: 0,
        description: "+25% Science. Space race projects cost 30% less.",
    },
    AiTech {
        id: AI_SUPERINTELLIGENCE,
        name: "Superintelligence",
        cost: 1300,
        prereqs: &[AI_AUTOMATED_SCIENCE],
        row: 0,
        description: "+50% Science and +20% Production.",
    },
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
            ..Default::default()
        })
        .collect()
}

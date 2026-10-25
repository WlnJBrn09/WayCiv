//! What policies, governments, beliefs, leader and civilization abilities,
//! and wonders do. Each effect is applied by the game (`wayciv-core`'s
//! `effects` module) and described in words by `describe`.

use serde::Deserialize;

/// Yield names as written in the data files, in yield order.
pub const YIELD_KEYS: [&str; 6] = ["food", "production", "gold", "science", "culture", "faith"];
pub const YIELD_WORDS: [&str; 6] = ["Food", "Production", "Gold", "Science", "Culture", "Faith"];

/// Which trade routes a trade effect applies to.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Routes {
    #[default]
    All,
    Domestic,
    International,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    /// Flat yield in every city (or only the capital).
    CityYield {
        #[serde(rename = "yield")]
        yield_type: String,
        amount: f32,
        #[serde(default)]
        capital: bool,
    },
    /// Percent yield in every city (or only the capital).
    CityPercent {
        #[serde(rename = "yield")]
        yield_type: String,
        percent: f32,
        #[serde(default)]
        capital: bool,
    },
    DistrictYield {
        district: String,
        #[serde(rename = "yield")]
        yield_type: String,
        amount: f32,
    },
    BuildingYield {
        building: String,
        #[serde(rename = "yield")]
        yield_type: String,
        amount: f32,
    },
    ImprovementYield {
        improvement: String,
        #[serde(rename = "yield")]
        yield_type: String,
        amount: i32,
    },
    /// Production toward units; optionally only one promotion class, unit,
    /// domain, or era of units, or only military units.
    UnitProduction {
        percent: f32,
        #[serde(default)]
        class: Option<String>,
        #[serde(default)]
        unit: Option<String>,
        #[serde(default)]
        domain: Option<String>,
        #[serde(default)]
        era: Option<String>,
        #[serde(default)]
        military: bool,
    },
    BuildingProduction {
        percent: f32,
    },
    WonderProduction {
        percent: f32,
    },
    DistrictProduction {
        percent: f32,
    },
    GreatPersonPoints {
        class: String,
        amount: i32,
    },
    Housing {
        amount: i32,
    },
    Amenities {
        amount: i32,
    },
    /// Combat Strength for all units, or one promotion class.
    Combat {
        amount: i32,
        #[serde(default)]
        class: Option<String>,
    },
    CombatInTerritory {
        amount: i32,
    },
    UnitXp {
        amount: i32,
    },
    AdjacencyPercent {
        percent: f32,
    },
    /// Cheaper Gold purchases.
    PurchaseDiscount {
        percent: f32,
    },
    TradeYield {
        #[serde(default)]
        routes: Routes,
        #[serde(rename = "yield")]
        yield_type: String,
        amount: f32,
    },
    TradeCapacity {
        amount: i32,
    },
    /// Yield in every city for each slotted policy of one kind.
    PolicyYield {
        slot: String,
        #[serde(rename = "yield")]
        yield_type: String,
        amount: f32,
    },
}

pub fn yield_index(key: &str) -> Option<usize> {
    let key = key.trim_start_matches("YIELD_").to_ascii_lowercase();
    YIELD_KEYS.iter().position(|y| *y == key)
}

fn signed(n: f32) -> String {
    let n = if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    };
    if n.starts_with('-') {
        n
    } else {
        format!("+{n}")
    }
}

fn yield_word(key: &str) -> &'static str {
    yield_index(key).map(|i| YIELD_WORDS[i]).unwrap_or("?")
}

/// Turns `SLOT_MILITARY` or `PROMOTION_CLASS_LIGHT_CAVALRY` into "Military"
/// or "Light Cavalry".
pub fn words(id: &str) -> String {
    let tail = [
        "PROMOTION_CLASS_",
        "SLOT_",
        "DOMAIN_",
        "ERA_",
        "GREAT_PERSON_CLASS_",
    ]
    .iter()
    .find_map(|p| id.strip_prefix(p))
    .unwrap_or(id);
    tail.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f
                    .to_uppercase()
                    .chain(c.flat_map(char::to_lowercase))
                    .collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
        .replace("Anti Cavalry", "Anti-Cavalry")
}

impl Effect {
    /// One sentence describing the effect. `name` resolves a content id to
    /// its display name.
    pub fn describe(&self, name: &dyn Fn(&str) -> String) -> String {
        let place = |capital: bool| if capital { "the Capital" } else { "every city" };
        match self {
            Effect::CityYield {
                yield_type,
                amount,
                capital,
            } => format!(
                "{} {} in {}.",
                signed(*amount),
                yield_word(yield_type),
                place(*capital)
            ),
            Effect::CityPercent {
                yield_type,
                percent,
                capital,
            } => format!(
                "{}% {} in {}.",
                signed(*percent),
                yield_word(yield_type),
                place(*capital)
            ),
            Effect::DistrictYield {
                district,
                yield_type,
                amount,
            } => format!(
                "{} {} from each {}.",
                signed(*amount),
                yield_word(yield_type),
                name(district)
            ),
            Effect::BuildingYield {
                building,
                yield_type,
                amount,
            } => format!(
                "{} {} from each {}.",
                signed(*amount),
                yield_word(yield_type),
                name(building)
            ),
            Effect::ImprovementYield {
                improvement,
                yield_type,
                amount,
            } => format!(
                "{} {} from each {}.",
                signed(*amount as f32),
                yield_word(yield_type),
                name(improvement)
            ),
            Effect::UnitProduction {
                percent,
                class,
                unit,
                domain,
                era,
                military,
            } => {
                let mut what = String::new();
                if let Some(era) = era {
                    what.push_str(&format!("{} era ", words(era)));
                }
                if let Some(unit) = unit {
                    what.push_str(&format!("{}s", name(unit)));
                } else if let Some(class) = class {
                    what.push_str(&format!("{} units", words(class)));
                } else if let Some(domain) = domain {
                    what.push_str(&format!("{} units", words(domain).to_lowercase()));
                } else if *military {
                    what.push_str("military units");
                } else {
                    what.push_str("units");
                }
                format!("{}% Production toward {what}.", signed(*percent))
            }
            Effect::BuildingProduction { percent } => {
                format!("{}% Production toward buildings.", signed(*percent))
            }
            Effect::WonderProduction { percent } => {
                format!("{}% Production toward wonders.", signed(*percent))
            }
            Effect::DistrictProduction { percent } => {
                format!("{}% Production toward districts.", signed(*percent))
            }
            Effect::GreatPersonPoints { class, amount } => format!(
                "{} {} point{} per turn.",
                signed(*amount as f32),
                name(class),
                if *amount == 1 { "" } else { "s" }
            ),
            Effect::Housing { amount } => {
                format!("{} Housing in every city.", signed(*amount as f32))
            }
            Effect::Amenities { amount } => format!(
                "{} {} in every city.",
                signed(*amount as f32),
                if *amount == 1 { "Amenity" } else { "Amenities" }
            ),
            Effect::Combat { amount, class } => match class {
                Some(c) => format!(
                    "{} Combat Strength for {} units.",
                    signed(*amount as f32),
                    words(c)
                ),
                None => format!("{} Combat Strength for all units.", signed(*amount as f32)),
            },
            Effect::CombatInTerritory { amount } => format!(
                "{} Combat Strength for units fighting inside your territory.",
                signed(*amount as f32)
            ),
            Effect::UnitXp { amount } => format!(
                "New units start with {} experience.",
                signed(*amount as f32).trim_start_matches('+')
            ),
            Effect::AdjacencyPercent { percent } => {
                format!("{}% district adjacency bonuses.", signed(*percent))
            }
            Effect::PurchaseDiscount { percent } => {
                format!("Gold purchases cost {}% less.", percent)
            }
            Effect::TradeYield {
                routes,
                yield_type,
                amount,
            } => {
                let which = match routes {
                    Routes::All => "trade route",
                    Routes::Domestic => "domestic trade route",
                    Routes::International => "international trade route",
                };
                format!(
                    "{} {} per {which}.",
                    signed(*amount),
                    yield_word(yield_type)
                )
            }
            Effect::TradeCapacity { amount } => {
                format!("{} trade route capacity.", signed(*amount as f32))
            }
            Effect::PolicyYield {
                slot,
                yield_type,
                amount,
            } => format!(
                "{} {} in every city for each {} policy slotted.",
                signed(*amount),
                yield_word(yield_type),
                words(slot)
            ),
        }
    }

    /// Content ids the effect refers to, for validation.
    pub fn references(&self) -> Vec<&str> {
        match self {
            Effect::DistrictYield { district, .. } => vec![district],
            Effect::BuildingYield { building, .. } => vec![building],
            Effect::ImprovementYield { improvement, .. } => vec![improvement],
            Effect::UnitProduction { unit, era, .. } => {
                unit.iter().chain(era.iter()).map(String::as_str).collect()
            }
            Effect::GreatPersonPoints { class, .. } => vec![class],
            _ => Vec::new(),
        }
    }

    /// Yield keys the effect uses, for validation.
    pub fn yield_key(&self) -> Option<&str> {
        match self {
            Effect::CityYield { yield_type, .. }
            | Effect::CityPercent { yield_type, .. }
            | Effect::DistrictYield { yield_type, .. }
            | Effect::BuildingYield { yield_type, .. }
            | Effect::ImprovementYield { yield_type, .. }
            | Effect::TradeYield { yield_type, .. }
            | Effect::PolicyYield { yield_type, .. } => Some(yield_type),
            _ => None,
        }
    }
}

/// The description of a list of effects, one sentence each, after an
/// optional line of the item's own text.
pub fn describe_all(text: &str, effects: &[Effect], name: &dyn Fn(&str) -> String) -> String {
    let mut lines: Vec<String> = Vec::new();
    if !text.trim().is_empty() {
        lines.push(text.trim().to_string());
    }
    let mechanics: Vec<String> = effects.iter().map(|e| e.describe(name)).collect();
    if !mechanics.is_empty() {
        lines.push(mechanics.join(" "));
    }
    lines.join("\n")
}

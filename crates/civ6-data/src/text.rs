//! English (`en_US`) text for `LOC_*` keys, from the base game and DLC.

use crate::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Default)]
pub struct Text {
    strings: HashMap<String, String>,
}

impl Text {
    /// Loads every `text/en_us/*.xml` under `base/assets` and each DLC.
    /// Later files replace earlier ones, as `<Replace>` rows do in the game.
    pub fn load(assets: &Path) -> Result<Self> {
        let mut text = Self::default();
        let mut dirs = vec![assets.join("base/assets/text/en_us")];
        if let Ok(entries) = fs::read_dir(assets.join("dlc")) {
            // Scenarios and game modes rename base-game text (a scenario calls
            // the Settler "Civilian"), so only standard content is loaded.
            let mut dlc: Vec<_> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name().is_some_and(|n| {
                        let n = n.to_string_lossy();
                        !n.contains("scenario") && !n.ends_with("mode") && n != "treerandomizer"
                    })
                })
                .map(|p| p.join("text/en_us"))
                .collect();
            dlc.sort();
            dirs.extend(dlc);
        }
        for dir in dirs.iter().filter(|d| d.is_dir()) {
            for file in crate::files_in_load_order(dir, "xml")? {
                let name = file
                    .file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if name.contains("scenario") || name.contains("_mode") {
                    continue;
                }
                text.load_file(&file);
            }
        }
        Ok(text)
    }

    fn load_file(&mut self, path: &Path) {
        let Ok(source) = fs::read_to_string(path) else {
            return;
        };
        let Ok(document) = roxmltree::Document::parse(source.trim_start_matches('\u{feff}')) else {
            return;
        };
        for row in document
            .descendants()
            .filter(|n| n.has_tag_name("Row") || n.has_tag_name("Replace"))
        {
            let language = row.attribute("Language").unwrap_or("en_US");
            if !language.eq_ignore_ascii_case("en_US") {
                continue;
            }
            let Some(tag) = row.attribute("Tag") else {
                continue;
            };
            let value = row.attribute("Text").map(str::to_owned).or_else(|| {
                row.children()
                    .find(|c| c.has_tag_name("Text"))
                    .and_then(|c| c.text())
                    .map(str::to_owned)
            });
            if let Some(value) = value {
                self.strings.insert(tag.to_string(), value);
            }
        }
    }

    /// The English text for `key`, with Civ VI's inline markup removed, or
    /// the key itself when no text exists.
    pub fn get(&self, key: &str) -> String {
        match self.strings.get(key) {
            Some(value) => clean(value),
            None => key.to_string(),
        }
    }

    /// Like `get`, but empty when the key has no text.
    pub fn get_or_empty(&self, key: &str) -> String {
        self.strings.get(key).map(|v| clean(v)).unwrap_or_default()
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.strings.insert(key.to_string(), value.to_string());
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
}

/// Removes icon tags like `[ICON_Strength]` and `[NEWLINE]`, and the
/// `{1_Amount}`-style placeholders' braces.
fn clean(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '[' {
            let tag: String = chars.by_ref().take_while(|&c| c != ']').collect();
            if tag.eq_ignore_ascii_case("NEWLINE") {
                out.push('\n');
            }
            continue;
        }
        out.push(c);
    }
    out.split(' ')
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" \n", "\n")
        .replace("\n ", "\n")
}

#[cfg(test)]
mod tests {
    use super::clean;

    #[test]
    fn strips_markup() {
        assert_eq!(
            clean("+5 [ICON_Strength] Combat Strength."),
            "+5 Combat Strength."
        );
        assert_eq!(clean("Line one.[NEWLINE]Line two."), "Line one.\nLine two.");
    }
}

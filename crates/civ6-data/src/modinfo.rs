//! `.modinfo` files: the `UpdateDatabase` actions that DLC and mods apply to
//! the gameplay database.
//!
//! Which actions apply in a given game (criteria such as `RuleSetInUse` and
//! `LeaderPlayable`) and their order are decided by the engine's modding
//! framework, which is not ported yet. Callers pass the applied actions in
//! order, as the game's `Modding.log` lists them under "Applying Component".

use crate::{GameplayDatabase, Result, resolve_case_insensitive};
use std::fs;
use std::path::{Path, PathBuf};

/// One `<UpdateDatabase id=...>` action and its files, resolved on disk.
#[derive(Debug, Clone)]
pub struct UpdateDatabaseAction {
    pub id: String,
    pub files: Vec<PathBuf>,
}

/// Finds `UpdateDatabase` action `id` in any `.modinfo` under `dlc_dir`.
pub fn find_action(dlc_dir: &Path, id: &str) -> Result<UpdateDatabaseAction> {
    for entry in fs::read_dir(dlc_dir)?.flatten() {
        let path = entry.path();
        if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("modinfo")) {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        let document = roxmltree::Document::parse(text.trim_start_matches('\u{feff}'))?;
        let action = document.descendants().find(|n| {
            n.has_tag_name("UpdateDatabase")
                && n.attribute("id") == Some(id)
                && n.ancestors().any(|a| a.has_tag_name("InGameActions"))
        });
        let Some(action) = action else { continue };
        let mut files = Vec::new();
        for file in action.children().filter(|c| c.has_tag_name("File")) {
            let relative = file.text().unwrap_or_default().trim();
            // Some shipped .modinfo files list files that are not installed;
            // the game skips those.
            if let Some(resolved) = resolve_case_insensitive(dlc_dir, relative) {
                files.push(resolved);
            }
        }
        return Ok(UpdateDatabaseAction { id: id.to_string(), files });
    }
    Err(format!("no InGameActions/UpdateDatabase id={id} in {}", dlc_dir.display()).into())
}

impl GameplayDatabase {
    /// Applies an action's files in order (`.xml` and `.sql`).
    pub fn apply_action(&mut self, action: &UpdateDatabaseAction) -> Result<()> {
        for file in &action.files {
            if file.extension().is_some_and(|e| e.eq_ignore_ascii_case("sql")) {
                self.execute_sql_file(file)?;
            } else {
                self.load_xml_file(file)?;
            }
        }
        Ok(())
    }
}

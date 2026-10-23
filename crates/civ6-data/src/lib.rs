//! Builds the Civilization VI gameplay database the way the game does:
//! schema SQL, then the base game's XML in file-name order, then GameCore's
//! own registrations, then each enabled DLC/mod `UpdateDatabase` action.
//!
//! The engine's XML loader is reproduced from its observable behavior, as
//! verified against the game's own `DebugGameplay.sqlite` in the
//! civ6-decompile project. WayCiv reads the user's installed copy at
//! runtime; nothing from the game is stored in this repository.

pub mod content;
pub mod hash;
pub mod install;
pub mod modinfo;
pub mod text;

use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, params_from_iter};
use std::fs;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// A gameplay database under construction.
pub struct GameplayDatabase {
    connection: Connection,
    /// Errors the engine would log to `Database.log` and continue past.
    pub log: Vec<String>,
}

impl GameplayDatabase {
    /// An empty in-memory database with the engine's `Make_Hash` SQL function.
    pub fn new() -> Result<Self> {
        let connection = Connection::open_in_memory()?;
        // Foreign keys are checked once after loading (`validate`), not per row.
        connection.pragma_update(None, "foreign_keys", false)?;
        connection.create_scalar_function(
            "Make_Hash",
            1,
            FunctionFlags::SQLITE_DETERMINISTIC,
            |ctx| {
                let name: String = ctx.get(0)?;
                // Stored as a signed 32-bit integer.
                Ok(i64::from(hash::make_hash(&name) as i32))
            },
        )?;
        Ok(Self {
            connection,
            log: Vec::new(),
        })
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn execute_sql_file(&mut self, path: &Path) -> Result<()> {
        let text = fs::read_to_string(path)?;
        // Some shipped SQL files start with a UTF-8 byte-order mark.
        self.connection
            .execute_batch(text.trim_start_matches('\u{feff}'))?;
        Ok(())
    }

    /// Applies one `GameInfo`/`GameData`/`Database` XML file.
    pub fn load_xml_file(&mut self, path: &Path) -> Result<()> {
        let text = fs::read_to_string(path)?;
        let document = roxmltree::Document::parse(text.trim_start_matches('\u{feff}'))?;
        let file = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let root = document.root_element();
        if !matches!(root.tag_name().name(), "GameInfo" | "GameData" | "Database") {
            self.log.push(format!("{file}: Input XML does not contain database entry tags. GameData, GameInfo or Database"));
            return Ok(());
        }
        for table in root.children().filter(roxmltree::Node::is_element) {
            if table.tag_name().name() == "Table" {
                self.create_table(&file, table);
                continue;
            }
            let name = table.tag_name().name();
            for operation in table.children().filter(roxmltree::Node::is_element) {
                if let Err(error) = self.apply(name, operation) {
                    self.log.push(format!(
                        "ERROR: {error}\nIn XMLSerializer while updating table {name} from file {file}."
                    ));
                }
            }
        }
        Ok(())
    }

    /// `<Table name=...><Column .../></Table>` definitions in `GameData` files.
    /// The SQL text matches what the engine writes into `sqlite_master`:
    /// `CREATE TABLE Colors ('Type' text  not null , 'Color' text , PRIMARY KEY (Type))`.
    /// An empty `default=""` adds no default.
    fn create_table(&mut self, file: &str, table: roxmltree::Node) {
        let name = table.attribute("name").unwrap_or_default();
        let mut columns = Vec::new();
        let mut keys = Vec::new();
        for column in table.children().filter(|c| c.has_tag_name("Column")) {
            let column_name = column.attribute("name").unwrap_or_default();
            let mut definition = format!(
                "'{column_name}' {}",
                column.attribute("type").unwrap_or("text")
            );
            if column.attribute("notnull") == Some("true") {
                definition.push_str("  not null");
            }
            if column.attribute("unique") == Some("true") {
                definition.push_str(" unique");
            }
            if let Some(default) = column.attribute("default").filter(|d| !d.is_empty()) {
                definition.push_str(&format!(" default {}", sql_literal(default)));
            }
            definition.push(' ');
            if column.attribute("primarykey") == Some("true") {
                keys.push(column_name.to_string());
            }
            columns.push(definition);
        }
        if !keys.is_empty() {
            columns.push(format!("PRIMARY KEY ({})", keys.join(", ")));
        }
        let sql = format!("CREATE TABLE {name} ({})", columns.join(", "));
        if let Err(error) = self.connection.execute(&sql, []) {
            self.log.push(format!(
                "ERROR: {error}\nIn XMLSerializer while building table {name} from file {file}."
            ));
        }
    }

    fn apply(&self, table: &str, operation: roxmltree::Node) -> rusqlite::Result<()> {
        let table = quote(table);
        match operation.tag_name().name() {
            verb @ ("Row" | "Replace" | "InsertOrIgnore") => {
                let values = fields(operation);
                if values.is_empty() {
                    return Ok(());
                }
                let insert = match verb {
                    "Replace" => "INSERT OR REPLACE",
                    "InsertOrIgnore" => "INSERT OR IGNORE",
                    _ => "INSERT",
                };
                let columns: Vec<_> = values.iter().map(|(c, _)| quote(c)).collect();
                let sql = format!(
                    "{insert} INTO {table} ({}) VALUES ({})",
                    columns.join(", "),
                    vec!["?"; values.len()].join(", ")
                );
                self.connection
                    .execute(&sql, params_from_iter(values.iter().map(|(_, v)| v)))?;
            }
            "Update" => {
                let set = operation
                    .children()
                    .find(|c| c.has_tag_name("Set"))
                    .map(fields)
                    .unwrap_or_default();
                let filter = operation
                    .children()
                    .find(|c| c.has_tag_name("Where"))
                    .map(fields)
                    .unwrap_or_default();
                if set.is_empty() {
                    return Ok(());
                }
                let assignments: Vec<_> = set
                    .iter()
                    .map(|(c, _)| format!("{} = ?", quote(c)))
                    .collect();
                let sql = format!(
                    "UPDATE {table} SET {}{}",
                    assignments.join(", "),
                    where_clause(&filter)
                );
                let parameters = set.iter().chain(filter.iter()).map(|(_, v)| v);
                self.connection
                    .execute(&sql, params_from_iter(parameters))?;
            }
            "Delete" => {
                let filter = fields(operation);
                let sql = format!("DELETE FROM {table}{}", where_clause(&filter));
                self.connection
                    .execute(&sql, params_from_iter(filter.iter().map(|(_, v)| v)))?;
            }
            _ => {}
        }
        Ok(())
    }

    /// "Validating Foreign Key Constraints...": every violation, as
    /// `(table, rowid, parent table)`.
    pub fn validate(&self) -> Result<Vec<(String, i64, String)>> {
        let mut statement = self.connection.prepare("PRAGMA foreign_key_check")?;
        let violations = statement
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get::<_, Option<i64>>(1)?.unwrap_or(-1),
                    r.get(2)?,
                ))
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(violations)
    }

    /// Writes the database to `path`, replacing any existing file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if path.exists() {
            fs::remove_file(path)?;
        }
        self.connection
            .execute("VACUUM INTO ?1", [path.to_string_lossy()])?;
        Ok(())
    }
}

/// A value as the engine binds it, observed from `DebugGameplay.sqlite`:
/// surrounding whitespace is trimmed, an empty value is NULL, `true`/`false`
/// (any case) are 1/0, numeric text is bound as a number (so `"01"` stores
/// `1` and `".25"` stores `0.25` even in TEXT columns), and anything else
/// is text.
#[derive(Debug)]
enum XmlValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
}

impl rusqlite::ToSql for XmlValue {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            XmlValue::Null => rusqlite::types::Null.to_sql(),
            XmlValue::Integer(v) => v.to_sql(),
            XmlValue::Real(v) => v.to_sql(),
            XmlValue::Text(v) => v.to_sql(),
        }
    }
}

fn xml_value(raw: &str) -> XmlValue {
    let text = raw.trim();
    let numeric = !text.is_empty()
        && text.bytes().any(|b| b.is_ascii_digit())
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'.' | b'-' | b'+' | b'e' | b'E'));
    if text.is_empty() {
        XmlValue::Null
    } else if text.eq_ignore_ascii_case("true") {
        XmlValue::Integer(1)
    } else if text.eq_ignore_ascii_case("false") {
        XmlValue::Integer(0)
    } else if let Some(value) = numeric.then(|| text.parse::<i64>().ok()).flatten() {
        XmlValue::Integer(value)
    } else if let Some(value) = numeric.then(|| text.parse::<f64>().ok()).flatten() {
        XmlValue::Real(value)
    } else {
        XmlValue::Text(text.to_string())
    }
}

/// Column/value pairs from attributes and `<Column>value</Column>` children.
fn fields(node: roxmltree::Node) -> Vec<(String, XmlValue)> {
    let mut fields: Vec<_> = node
        .attributes()
        .map(|a| (a.name().to_string(), xml_value(a.value())))
        .collect();
    for child in node.children().filter(roxmltree::Node::is_element) {
        fields.push((
            child.tag_name().name().to_string(),
            xml_value(child.text().unwrap_or_default()),
        ));
    }
    fields
}

fn where_clause(filter: &[(String, XmlValue)]) -> String {
    if filter.is_empty() {
        return String::new();
    }
    let conditions: Vec<_> = filter
        .iter()
        .map(|(c, _)| format!("{} = ?", quote(c)))
        .collect();
    format!(" WHERE {}", conditions.join(" AND "))
}

fn quote(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn sql_literal(value: &str) -> String {
    if value.parse::<f64>().is_ok() {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "''"))
    }
}

/// Files in `dir` with extension `ext`, in the engine's load order
/// (case-insensitive byte order of the file name).
pub fn files_in_load_order(dir: &Path, ext: &str) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e.eq_ignore_ascii_case(ext)))
        .collect();
    files.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
    Ok(files)
}

/// Resolves a path from a `.modinfo` (e.g. `Data/Foo.xml`) case-insensitively,
/// since the Linux install stores lower-cased names.
pub fn resolve_case_insensitive(base: &Path, relative: &str) -> Option<PathBuf> {
    let mut current = base.to_path_buf();
    for part in relative.split(['/', '\\']).filter(|p| !p.is_empty()) {
        let exact = current.join(part);
        current = if exact.exists() {
            exact
        } else {
            fs::read_dir(&current)
                .ok()?
                .filter_map(|e| e.ok())
                .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(part))?
                .path()
        };
    }
    Some(current)
}

/// The base game's gameplay database before GameCore registrations and mods:
/// `<assets>/base/assets/gameplay/data/schema/*.sql` and `*_tables.xml`, then
/// `<assets>/base/assets/gameplay/data/*.xml`.
pub fn build_base(steam_assets: &Path) -> Result<GameplayDatabase> {
    let data = steam_assets.join("base/assets/gameplay/data");
    let schema = data.join("schema");
    let mut database = GameplayDatabase::new()?;
    for file in files_in_load_order(&schema, "sql")? {
        database.execute_sql_file(&file)?;
    }
    for file in files_in_load_order(&schema, "xml")? {
        database.load_xml_file(&file)?;
    }
    for file in files_in_load_order(&data, "xml")? {
        database.load_xml_file(&file)?;
    }
    Ok(database)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored(raw: &str) -> rusqlite::types::Value {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute(
                "CREATE TABLE t (text_column TEXT, integer_column INTEGER)",
                [],
            )
            .unwrap();
        let value = xml_value(raw);
        connection
            .execute("INSERT INTO t VALUES (?1, ?1)", [&value])
            .unwrap();
        connection
            .query_row("SELECT text_column FROM t", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn values_are_normalized_like_the_engine() {
        use rusqlite::types::Value::*;
        assert_eq!(stored("false"), Text("0".into()));
        assert_eq!(stored("True"), Text("1".into()));
        assert_eq!(stored("TRUE"), Text("1".into()));
        assert_eq!(stored("01"), Text("1".into()));
        assert_eq!(stored(".25"), Text("0.25".into()));
        assert_eq!(stored("-3"), Text("-3".into()));
        assert_eq!(stored(""), Null);
        assert_eq!(stored("   "), Null);
        assert_eq!(
            stored("raid or invasion. "),
            Text("raid or invasion.".into())
        );
        assert_eq!(stored("UNIT_WARRIOR"), Text("UNIT_WARRIOR".into()));
        assert_eq!(
            stored("inf"),
            Text("inf".into()),
            "only digit-bearing text is numeric"
        );
    }

    #[test]
    fn make_hash_is_signed_32_bit() {
        let database = GameplayDatabase::new().unwrap();
        let hash: i64 = database
            .connection()
            .query_row("SELECT Make_Hash('UNIT_SETTLER')", [], |r| r.get(0))
            .unwrap();
        assert_eq!(hash, -798_712_536);
    }

    #[test]
    fn xml_table_definitions_match_engine_sql_text() {
        let dir = std::env::temp_dir().join(format!("civ6-gamedata-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("tables.xml");
        fs::write(
            &file,
            r#"<GameData><Table name="T">
                <Column name="Type" type="text" notnull="true" unique="true" primarykey="true"/>
                <Column name="Sort" type="integer" default="0"/>
                <Column name="Tooltip" type="text" default=""/>
            </Table><T><Row Type="A"/><Row><Type>B</Type><Sort>2</Sort></Row></T></GameData>"#,
        )
        .unwrap();
        let mut database = GameplayDatabase::new().unwrap();
        database.load_xml_file(&file).unwrap();
        let sql: String = database
            .connection()
            .query_row("SELECT sql FROM sqlite_master WHERE name = 'T'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            sql,
            "CREATE TABLE T ('Type' text  not null unique , 'Sort' integer default 0 , 'Tooltip' text , PRIMARY KEY (Type))"
        );
        let rows: Vec<(String, i64)> = database
            .connection()
            .prepare("SELECT Type, Sort FROM T ORDER BY rowid")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(rows, [("A".to_string(), 0), ("B".to_string(), 2)]);
        fs::remove_dir_all(&dir).unwrap();
    }
}

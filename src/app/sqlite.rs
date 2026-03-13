use std::fs;
use std::path::Path;
use std::path::PathBuf;

use rusqlite::{Connection, params};

use super::{SavedCraft, SavedItemPrice};

const DB_FILE_NAME: &str = "mdcraft.sqlite3";

fn resolve_app_data_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "linux")]
    {
        if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
            return Ok(PathBuf::from(path).join("mdcraft"));
        }

        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home).join(".local/share/mdcraft"));
        }

        return Err("Nao foi possivel resolver o diretorio de dados no Linux.".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(path) = std::env::var_os("APPDATA") {
            return Ok(PathBuf::from(path).join("mdcraft"));
        }

        if let Some(profile) = std::env::var_os("USERPROFILE") {
            return Ok(PathBuf::from(profile).join("AppData/Roaming/mdcraft"));
        }

        return Err("Nao foi possivel resolver o diretorio de dados no Windows.".to_string());
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        std::env::current_dir()
            .map(|dir| dir.join("mdcraft-data"))
            .map_err(|err| format!("Nao foi possivel resolver diretorio de dados: {err}"))
    }
}

fn sqlite_db_path() -> Result<PathBuf, String> {
    let app_dir = resolve_app_data_dir()?;
    fs::create_dir_all(&app_dir)
        .map_err(|err| format!("Nao foi possivel criar diretorio de dados: {err}"))?;
    Ok(app_dir.join(DB_FILE_NAME))
}

fn ensure_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS saved_crafts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            position INTEGER NOT NULL,
            name TEXT NOT NULL,
            recipe_text TEXT NOT NULL,
            sell_price_input TEXT NOT NULL,
            item_prices_json TEXT NOT NULL
        );
        ",
    )
    .map_err(|err| format!("Falha ao criar schema SQLite: {err}"))
}

pub(super) fn load_saved_crafts_from_sqlite() -> Result<Vec<SavedCraft>, String> {
    let db_path = sqlite_db_path()?;
    load_saved_crafts_from_path(&db_path)
}

fn load_saved_crafts_from_path(db_path: &Path) -> Result<Vec<SavedCraft>, String> {
    let conn = Connection::open(db_path)
        .map_err(|err| format!("Falha ao abrir banco SQLite para leitura: {err}"))?;

    ensure_schema(&conn)?;

    let mut stmt = conn
        .prepare(
            "
            SELECT name, recipe_text, sell_price_input, item_prices_json
            FROM saved_crafts
            ORDER BY position ASC, id ASC
            ",
        )
        .map_err(|err| format!("Falha ao preparar consulta de receitas: {err}"))?;

    let rows = stmt
        .query_map([], |row| {
            let item_prices_json: String = row.get(3)?;
            let item_prices = serde_json::from_str::<Vec<SavedItemPrice>>(&item_prices_json)
                .unwrap_or_default();

            Ok(SavedCraft {
                name: row.get(0)?,
                recipe_text: row.get(1)?,
                sell_price_input: row.get(2)?,
                item_prices,
            })
        })
        .map_err(|err| format!("Falha ao consultar receitas no SQLite: {err}"))?;

    let mut crafts = Vec::new();
    for row in rows {
        crafts.push(row.map_err(|err| format!("Falha ao mapear linha de receita: {err}"))?);
    }

    Ok(crafts)
}

pub(super) fn save_saved_crafts_to_sqlite(saved_crafts: &[SavedCraft]) -> Result<(), String> {
    let db_path = sqlite_db_path()?;
    save_saved_crafts_to_path(&db_path, saved_crafts)
}

fn save_saved_crafts_to_path(db_path: &Path, saved_crafts: &[SavedCraft]) -> Result<(), String> {
    let mut conn = Connection::open(db_path)
        .map_err(|err| format!("Falha ao abrir banco SQLite para escrita: {err}"))?;

    ensure_schema(&conn)?;

    let tx = conn
        .transaction()
        .map_err(|err| format!("Falha ao iniciar transacao SQLite: {err}"))?;

    tx.execute("DELETE FROM saved_crafts", [])
        .map_err(|err| format!("Falha ao limpar receitas antigas no SQLite: {err}"))?;

    for (idx, craft) in saved_crafts.iter().enumerate() {
        let item_prices_json = serde_json::to_string(&craft.item_prices)
            .map_err(|err| format!("Falha ao serializar itens de receita para SQLite: {err}"))?;

        tx.execute(
            "
            INSERT INTO saved_crafts (position, name, recipe_text, sell_price_input, item_prices_json)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                idx as i64,
                craft.name,
                craft.recipe_text,
                craft.sell_price_input,
                item_prices_json,
            ],
        )
        .map_err(|err| format!("Falha ao inserir receita no SQLite: {err}"))?;
    }

    tx.commit()
        .map_err(|err| format!("Falha ao finalizar transacao SQLite: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{load_saved_crafts_from_path, save_saved_crafts_to_path};
    use crate::app::{SavedCraft, SavedItemPrice};
    use std::path::PathBuf;

    fn unique_temp_db_path(test_name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "mdcraft-{test_name}-{}-{nonce}.sqlite3",
            std::process::id()
        ))
    }

    #[test]
    fn sqlite_roundtrip_preserves_saved_crafts_and_item_prices() {
        let db_path = unique_temp_db_path("sqlite-roundtrip");

        let expected = vec![
            SavedCraft {
                name: "Receita A".to_string(),
                recipe_text: "1 Iron Ore, 2 Screw".to_string(),
                sell_price_input: "12k".to_string(),
                item_prices: vec![
                    SavedItemPrice {
                        item_name: "Iron Ore".to_string(),
                        price_input: "100".to_string(),
                    },
                    SavedItemPrice {
                        item_name: "Screw".to_string(),
                        price_input: "250".to_string(),
                    },
                ],
            },
            SavedCraft {
                name: "Receita B".to_string(),
                recipe_text: "3 Rubber Ball".to_string(),
                sell_price_input: "4k".to_string(),
                item_prices: vec![SavedItemPrice {
                    item_name: "Rubber Ball".to_string(),
                    price_input: "1k".to_string(),
                }],
            },
        ];

        save_saved_crafts_to_path(&db_path, &expected)
            .expect("saving sqlite fixtures should succeed");
        let loaded = load_saved_crafts_from_path(&db_path)
            .expect("loading sqlite fixtures should succeed");

        assert_eq!(loaded.len(), expected.len());
        assert_eq!(loaded[0].name, expected[0].name);
        assert_eq!(loaded[0].recipe_text, expected[0].recipe_text);
        assert_eq!(loaded[0].sell_price_input, expected[0].sell_price_input);
        assert_eq!(loaded[0].item_prices.len(), expected[0].item_prices.len());
        assert_eq!(loaded[0].item_prices[0].item_name, "Iron Ore");
        assert_eq!(loaded[0].item_prices[0].price_input, "100");
        assert_eq!(loaded[1].name, expected[1].name);
        assert_eq!(loaded[1].item_prices[0].item_name, "Rubber Ball");

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn sqlite_load_returns_empty_when_database_has_no_rows() {
        let db_path = unique_temp_db_path("sqlite-empty");

        save_saved_crafts_to_path(&db_path, &[]).expect("saving empty list should succeed");
        let loaded = load_saved_crafts_from_path(&db_path)
            .expect("loading empty sqlite list should succeed");

        assert!(loaded.is_empty());

        let _ = std::fs::remove_file(db_path);
    }
}

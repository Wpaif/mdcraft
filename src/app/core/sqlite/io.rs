use std::path::Path;

use rusqlite::{Connection, params};

use crate::app::{SavedCraft, SavedItemPrice};

use super::paths::sqlite_db_path;
use super::schema::ensure_schema;

pub(super) fn load_saved_crafts_from_sqlite() -> Result<Vec<SavedCraft>, String> {
    let db_path = sqlite_db_path()?;
    load_saved_crafts_from_path(&db_path)
}

pub(super) fn load_saved_crafts_from_path(db_path: &Path) -> Result<Vec<SavedCraft>, String> {
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

pub(super) fn save_saved_crafts_to_path(
    db_path: &Path,
    saved_crafts: &[SavedCraft],
) -> Result<(), String> {
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

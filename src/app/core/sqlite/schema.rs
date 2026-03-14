use rusqlite::Connection;

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), String> {
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

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
            sell_price_is_per_item INTEGER NOT NULL DEFAULT 0,
            item_prices_json TEXT NOT NULL
        );
        ",
    )
    .map_err(|err| format!("Falha ao criar schema SQLite: {err}"))
    .and_then(|_| {
        // Migração leve: adiciona coluna em bancos antigos.
        match conn.execute(
            "ALTER TABLE saved_crafts ADD COLUMN sell_price_is_per_item INTEGER NOT NULL DEFAULT 0",
            [],
        ) {
            Ok(_) => Ok(()),
            Err(err) => {
                let msg = err.to_string().to_lowercase();
                if msg.contains("duplicate column") {
                    Ok(())
                } else {
                    Err(format!("Falha ao atualizar schema SQLite: {err}"))
                }
            }
        }
    })
}

use std::path::PathBuf;

use crate::app::{SavedCraft, SavedItemPrice};

use super::{load_saved_crafts_from_path, save_saved_crafts_to_path};

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
            sell_price_is_per_item: true,
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
            sell_price_is_per_item: false,
            item_prices: vec![SavedItemPrice {
                item_name: "Rubber Ball".to_string(),
                price_input: "1k".to_string(),
            }],
        },
    ];

    save_saved_crafts_to_path(&db_path, &expected).expect("saving sqlite fixtures should succeed");
    let loaded = load_saved_crafts_from_path(&db_path).expect("loading sqlite fixtures should succeed");

    assert_eq!(loaded.len(), expected.len());
    assert_eq!(loaded[0].name, expected[0].name);
    assert_eq!(loaded[0].recipe_text, expected[0].recipe_text);
    assert_eq!(loaded[0].sell_price_input, expected[0].sell_price_input);
    assert_eq!(loaded[0].sell_price_is_per_item, expected[0].sell_price_is_per_item);
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
    let loaded = load_saved_crafts_from_path(&db_path).expect("loading empty sqlite list should succeed");

    assert!(loaded.is_empty());

    let _ = std::fs::remove_file(db_path);
}

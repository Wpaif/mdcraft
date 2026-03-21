use crate::app::{MdcraftApp, SavedCraft, SavedItemPrice};

use super::{apply_pending_sidebar_actions, load_saved_craft_for_edit, set_pending_action};

fn make_saved_craft(name: &str, recipe_text: &str, sell_price_input: &str) -> SavedCraft {
    SavedCraft {
        name: name.to_string(),
        recipe_text: recipe_text.to_string(),
        sell_price_input: sell_price_input.to_string(),
        sell_price_is_per_item: false,
        item_prices: vec![],
    }
}

#[test]
fn load_saved_craft_for_edit_updates_active_data() {
    let mut app = MdcraftApp::default();
    app.saved_crafts.push(SavedCraft {
        name: "teste".to_string(),
        recipe_text: "2 Iron Ore, 3 Screw".to_string(),
        sell_price_input: "12k".to_string(),
        sell_price_is_per_item: true,
        item_prices: vec![SavedItemPrice {
            item_name: "Screw".to_string(),
            price_input: "250".to_string(),
        }],
    });
    // Adiciona receita ao cache para reconstrução correta
    app.craft_recipes_cache
        .push(crate::data::wiki_scraper::ScrapedCraftRecipe {
            profession: crate::data::wiki_scraper::CraftProfession::Engineer,
            rank: crate::data::wiki_scraper::CraftRank::E,
            name: "teste".to_string(),
            ingredients: vec![
                crate::data::wiki_scraper::CraftIngredient {
                    name: "Iron Ore".to_string(),
                    quantity: 2.0,
                },
                crate::data::wiki_scraper::CraftIngredient {
                    name: "Screw".to_string(),
                    quantity: 3.0,
                },
            ],
        });
    load_saved_craft_for_edit(&mut app, 0);
    assert_eq!(app.active_saved_craft_index, Some(0));
    assert_eq!(app.sell_price_input, "12k");
    assert!(app.sell_price_is_per_item);
    assert!(!app.items.is_empty());
    let screw = app
        .items
        .iter()
        .find(|i| i.nome == "Screw")
        .expect("Screw should exist after loading saved craft");
    assert_eq!(screw.preco_input, "250");
    assert_eq!(screw.preco_unitario, 250.0);
}

#[test]
fn load_saved_craft_for_edit_restores_craft_quantity_from_recipe_text() {
    let mut app = MdcraftApp::default();
    app.saved_crafts.push(SavedCraft {
        name: "teste".to_string(),
        recipe_text: "200 Iron Ore, 300 Screw".to_string(),
        sell_price_input: "12k".to_string(),
        sell_price_is_per_item: true,
        item_prices: vec![],
    });
    app.craft_recipes_cache
        .push(crate::data::wiki_scraper::ScrapedCraftRecipe {
            profession: crate::data::wiki_scraper::CraftProfession::Engineer,
            rank: crate::data::wiki_scraper::CraftRank::E,
            name: "teste".to_string(),
            ingredients: vec![
                crate::data::wiki_scraper::CraftIngredient {
                    name: "Iron Ore".to_string(),
                    quantity: 2.0,
                },
                crate::data::wiki_scraper::CraftIngredient {
                    name: "Screw".to_string(),
                    quantity: 3.0,
                },
            ],
        });

    load_saved_craft_for_edit(&mut app, 0);

    assert_eq!(app.craft_search_qty, 100);
    assert_eq!(app.craft_search_qty_input, "100");
    let iron = app
        .items
        .iter()
        .find(|i| i.nome == "Iron Ore")
        .expect("Iron Ore should exist after loading saved craft");
    assert_eq!(iron.quantidade, 200);
    let screw = app
        .items
        .iter()
        .find(|i| i.nome == "Screw")
        .expect("Screw should exist after loading saved craft");
    assert_eq!(screw.quantidade, 300);
}

#[test]
fn load_saved_craft_for_edit_ignores_out_of_bounds_index() {
    let mut app = MdcraftApp::default();
    app.saved_crafts
        .push(make_saved_craft("teste", "1 Iron Ore", "1k"));

    load_saved_craft_for_edit(&mut app, 9);

    assert_eq!(app.active_saved_craft_index, None);
}

#[test]
fn apply_pending_sidebar_actions_sets_delete_and_selects_recipe() {
    let mut app = MdcraftApp::default();
    app.saved_crafts
        .push(make_saved_craft("receita a", "1 Iron Ore", "3k"));

    apply_pending_sidebar_actions(&mut app, Some(0), Some(0));

    assert_eq!(app.pending_delete_index, Some(0));
    assert_eq!(app.active_saved_craft_index, Some(0));
}

#[test]
fn set_pending_action_respects_clicked_flag() {
    let mut slot = None;

    set_pending_action(&mut slot, 1, false);
    assert_eq!(slot, None);

    set_pending_action(&mut slot, 3, true);
    assert_eq!(slot, Some(3));
}


use crate::app::{MdcraftApp, SavedCraft};
use crate::data::wiki_scraper::{ScrapedItem, WikiSource};
use crate::model::Item;

use super::logic::{
    apply_cached_npc_price_if_available,
};

#[test]
fn rebuild_items_from_input_parses_recipe_and_preserves_prices() {
    let mut app = MdcraftApp::default();
    app.items = vec![
        Item {
            nome: "Screw".to_string(),
            quantidade: 1,
            preco_unitario: 250.0,
            valor_total: 250.0,
            is_resource: false,
            preco_input: "250".to_string(),
        },
        Item {
            nome: "Outro".to_string(),
            quantidade: 1,
            preco_unitario: 10.0,
            valor_total: 10.0,
            is_resource: false,
            preco_input: "10".to_string(),
        },
    ];

    rebuild_items_from_input(&mut app);

    assert_eq!(app.items.len(), 2);
    let screw = app
        .items
        .iter()
        .find(|i| i.nome == "Screw")
        .expect("Screw should exist after parsing");
    assert_eq!(screw.quantidade, 3);
    assert_eq!(screw.preco_input, "250");
    assert_eq!(screw.preco_unitario, 250.0);
    assert_eq!(screw.valor_total, 750.0);
}

#[test]
fn rebuild_items_from_input_does_not_autosave_active_craft() {
    let mut app = MdcraftApp::default();
    app.sell_price_input = "5k".to_string();
    app.saved_crafts.push(SavedCraft {
        name: "A".to_string(),
        recipe_text: "original".to_string(),
        sell_price_input: "1k".to_string(),
        item_prices: vec![],
    });
    app.active_saved_craft_index = Some(0);

    rebuild_items_from_input(&mut app);

    assert_eq!(app.saved_crafts[0].recipe_text, "original");
    assert_eq!(app.saved_crafts[0].sell_price_input, "1k");
}

#[test]
fn apply_cached_npc_price_if_available_sets_item_values() {
    let mut app = MdcraftApp::default();
    app.wiki_cached_items.push(ScrapedItem {
        name: "Test Item Beta".to_string(),
        npc_price: Some("2k".to_string()),
        sources: vec![WikiSource::Loot],
    });

    let mut item = Item {
        nome: "Test Item Beta".to_string(),
        quantidade: 3,
        preco_unitario: 0.0,
        valor_total: 0.0,
        is_resource: false,
        preco_input: String::new(),
    };

    apply_cached_npc_price_if_available(&app, &mut item);

    assert_eq!(item.preco_input, "2k");
    assert_eq!(item.preco_unitario, 2000.0);
    assert_eq!(item.valor_total, 6000.0);
}

#[test]
fn rebuild_items_from_input_prefills_npc_price_for_new_item() {
    let mut app = MdcraftApp::default();
    app.input_text = "2 Test Item Gamma".to_string();
    app.wiki_cached_items.push(ScrapedItem {
        name: "Test Item Gamma".to_string(),
        npc_price: Some("1k".to_string()),
        sources: vec![WikiSource::Nightmare],
    });

    rebuild_items_from_input(&mut app);

    assert_eq!(app.items.len(), 1);
    assert_eq!(app.items[0].preco_input, "1k");
    assert_eq!(app.items[0].preco_unitario, 1000.0);
    assert_eq!(app.items[0].valor_total, 2000.0);
}

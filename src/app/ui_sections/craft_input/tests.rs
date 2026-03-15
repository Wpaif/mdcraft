use crate::app::MdcraftApp;
use crate::data::wiki_scraper::{ScrapedItem, WikiSource};
use crate::model::Item;

use super::logic::apply_cached_npc_price_if_available;

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
        quantidade_base: 1,
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

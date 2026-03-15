use eframe::egui;
use std::collections::HashMap;

use crate::app::MdcraftApp;
use crate::data::wiki_scraper::{ScrapedItem, WikiSource};
use crate::model::Item;

use super::build_npc_price_lookup;
use super::compare_item_price_with_npc;
use super::npc_price_for_item;
use super::price_input_fill_color;
use super::price_input_stroke;
use super::should_show_npc_price_icon;
use super::NpcPriceComparison;

fn make_item(nome: &str, quantidade: u64, preco_input: &str) -> Item {
    Item {
        nome: nome.to_string(),
        quantidade_base: quantidade,
        quantidade,
        preco_unitario: 0.0,
        valor_total: 0.0,
        is_resource: false,
        preco_input: preco_input.to_string(),
    }
}

#[test]
fn compare_item_price_with_npc_covers_equal_cheaper_and_expensive() {
    let mut app = MdcraftApp::default();
    app.wiki_cached_items.push(ScrapedItem {
        name: "Screw".to_string(),
        npc_price: Some("1k".to_string()),
        sources: vec![WikiSource::Loot],
    });

    let lookup = build_npc_price_lookup(&app);

    let equal = make_item("Screw", 1, "1k");
    assert_eq!(
        compare_item_price_with_npc(&equal, &lookup),
        Some(NpcPriceComparison::Equal)
    );

    let cheaper = make_item("Screw", 1, "800");
    assert_eq!(
        compare_item_price_with_npc(&cheaper, &lookup),
        Some(NpcPriceComparison::LowerThanNpc)
    );

    let expensive = make_item("Screw", 1, "2k");
    assert_eq!(
        compare_item_price_with_npc(&expensive, &lookup),
        Some(NpcPriceComparison::HigherThanNpc)
    );
}

#[test]
fn build_npc_price_lookup_includes_fixed_compressed_nightmare_gems() {
    let app = MdcraftApp::default();
    let lookup = build_npc_price_lookup(&app);
    assert_eq!(lookup.get("compressed nightmare gems").copied(), Some(25_000.0));
}

#[test]
fn build_npc_price_lookup_includes_fixed_neutral_essence() {
    let app = MdcraftApp::default();
    let lookup = build_npc_price_lookup(&app);
    assert_eq!(lookup.get("neutral essence").copied(), Some(1000.0));
}

#[test]
fn price_input_stroke_highlights_missing_value() {
    egui::__run_test_ui(|ui| {
        let item = make_item("Screw", 1, "");
        let lookup = HashMap::new();
        let stroke = price_input_stroke(ui, &item, &lookup);
        assert_eq!(
            stroke,
            egui::Stroke::new(1.4, egui::Color32::from_rgb(235, 188, 90))
        );
    });
}

#[test]
fn price_input_fill_color_marks_only_missing_value() {
    let missing = make_item("Screw", 1, "");
    assert_eq!(
        price_input_fill_color(&missing),
        egui::Color32::from_rgba_unmultiplied(235, 188, 90, 22)
    );

    let present = make_item("Screw", 1, "1k");
    assert_eq!(price_input_fill_color(&present), egui::Color32::TRANSPARENT);
}

#[test]
fn npc_price_for_item_returns_lookup_value() {
    let mut app = MdcraftApp::default();
    app.wiki_cached_items.push(ScrapedItem {
        name: "Screw".to_string(),
        npc_price: Some("1k".to_string()),
        sources: vec![WikiSource::Loot],
    });

    let lookup = build_npc_price_lookup(&app);
    let item = make_item("Screw", 1, "");
    assert_eq!(npc_price_for_item(&item, &lookup), Some(1000.0));
}

#[test]
fn should_show_npc_price_icon_hides_only_for_diamond() {
    assert!(!should_show_npc_price_icon("Diamond"));
    assert!(!should_show_npc_price_icon(" diamond "));
    assert!(should_show_npc_price_icon("Screw"));
}

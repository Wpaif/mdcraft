use eframe::egui;

use crate::app::MdcraftApp;
use crate::app::price::PriceStatus;
use crate::model::Item;

use super::layout::render_empty_item_cells;
use super::price_logic::{
    apply_item_price_from_input, apply_item_price_if_changed, item_price_status, item_status_hover,
};
use super::render_items_and_values;

fn make_item(nome: &str, quantidade: u64, preco_input: &str, is_resource: bool) -> Item {
    Item {
        nome: nome.to_string(),
        quantidade_base: quantidade,
        quantidade,
        preco_unitario: 0.0,
        valor_total: 0.0,
        is_resource,
        preco_input: preco_input.to_string(),
    }
}

#[test]
fn apply_item_price_from_input_parses_and_updates_total() {
    let mut item = make_item("Screw", 3, "2k", false);
    apply_item_price_from_input(&mut item);
    assert_eq!(item.preco_unitario, 2000.0);
    assert_eq!(item.valor_total, 6000.0);
}

#[test]
fn apply_item_price_if_changed_respects_flag() {
    let mut item = make_item("Screw", 2, "3k", false);

    apply_item_price_if_changed(&mut item, false);
    assert_eq!(item.preco_unitario, 0.0);
    assert_eq!(item.valor_total, 0.0);

    apply_item_price_if_changed(&mut item, true);
    assert_eq!(item.preco_unitario, 3000.0);
    assert_eq!(item.valor_total, 6000.0);
}

#[test]
fn item_price_status_covers_invalid_ok_and_none() {
    let mut invalid = make_item("A", 1, "x", false);
    apply_item_price_from_input(&mut invalid);
    assert_eq!(item_price_status(&invalid), PriceStatus::Invalid);
    assert_eq!(item_status_hover(PriceStatus::Invalid), Some("Valor Inválido"));

    let mut ok = make_item("B", 2, "1k", false);
    apply_item_price_from_input(&mut ok);
    assert_eq!(item_price_status(&ok), PriceStatus::Ok);
    assert_eq!(item_status_hover(PriceStatus::Ok), Some("OK"));

    let none = make_item("C", 1, "", false);
    assert_eq!(item_price_status(&none), PriceStatus::None);
    assert_eq!(item_status_hover(PriceStatus::None), None);
}

#[test]
fn render_items_and_values_with_only_resources_keeps_total_zero() {
    let mut app = MdcraftApp::default();
    app.items = vec![
        make_item("Iron Ore", 2, "", true),
        make_item("Copper Ore", 3, "", true),
    ];

    let mut total_cost = 0.0;
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_items_and_values(ui, &mut app, 720.0, &mut total_cost);
        });
    });

    assert_eq!(total_cost, 0.0);
}

#[test]
fn render_items_and_values_handles_uneven_grid_with_blank_cells() {
    let mut app = MdcraftApp::default();
    app.items = vec![
        make_item("A", 1, "1k", false),
        make_item("B", 1, "", false),
        make_item("C", 1, "x", false),
    ];

    let mut total_cost = 0.0;
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_items_and_values(ui, &mut app, 900.0, &mut total_cost);
        });
    });

    assert!(total_cost >= 0.0);
}

#[test]
fn render_empty_item_cells_runs_without_panicking() {
    egui::__run_test_ui(|ui| {
        render_empty_item_cells(ui, 120.0, 46.0, 96.0, 78.0, 56.0);
    });
}

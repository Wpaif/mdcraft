use super::MdcraftApp;
use crate::app::capture_saved_item_prices;

mod closing;
pub mod craft_input;
mod items_grid;
mod npc_price;

pub(super) use closing::render_closing;
pub(super) use craft_input::render_craft_input;
pub(super) use items_grid::render_items_and_values;

pub(super) use super::{capitalize_display_name, placeholder};


#[allow(dead_code)]
pub(super) fn autosave_active_craft(app: &mut MdcraftApp) {
    let Some(idx) = app.active_saved_craft_index else {
        return;
    };

    if let Some(craft) = app.saved_crafts.get_mut(idx) {
        // craft.recipe_text = app.input_text.clone();
        craft.sell_price_input = app.sell_price_input.clone();
        craft.item_prices = capture_saved_item_prices(&app.items);
    }
}

pub(super) fn collect_found_resources(app: &MdcraftApp) -> Vec<(String, u64)> {
    app.items
        .iter()
        .filter(|item| item.is_resource)
        .map(|item| (item.nome.clone(), item.quantidade))
        .collect()
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::model::Item;

    use crate::app::SavedCraft;

    use super::{
        MdcraftApp, autosave_active_craft, capitalize_display_name, collect_found_resources,
        render_closing, render_craft_input, render_items_and_values,
    };

    fn run_ui_frame(mut f: impl FnMut(&egui::Context)) {
        egui::__run_test_ctx(|ctx| f(ctx));
    }

    #[test]
    fn capitalize_display_name_normalizes_words() {
        assert_eq!(capitalize_display_name("iron ORE"), "Iron Ore");
        assert_eq!(capitalize_display_name("  pure   grass  "), "Pure Grass");
        assert_eq!(capitalize_display_name("   "), "");
    }

    #[test]
    fn collect_found_resources_returns_only_resource_items() {
        let mut app = MdcraftApp::default();
        app.items = vec![
            Item {
                nome: "iron ore".to_string(),
                quantidade: 12,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: true,
                preco_input: String::new(),
            },
            Item {
                nome: "screw".to_string(),
                quantidade: 5,
                preco_unitario: 100.0,
                valor_total: 500.0,
                is_resource: false,
                preco_input: "100".to_string(),
            },
        ];

        let found = collect_found_resources(&app);
        assert_eq!(found, vec![("iron ore".to_string(), 12)]);
    }

    #[test]
    fn render_items_and_values_sums_only_priced_items() {
        let mut app = MdcraftApp::default();
        app.items = vec![
            Item {
                nome: "iron ore".to_string(),
                quantidade: 10,
                preco_unitario: 1.0,
                valor_total: 10.0,
                is_resource: true,
                preco_input: "1".to_string(),
            },
            Item {
                nome: "screw".to_string(),
                quantidade: 4,
                preco_unitario: 250.0,
                valor_total: 1_000.0,
                is_resource: false,
                preco_input: "250".to_string(),
            },
            Item {
                nome: "rubber ball".to_string(),
                quantidade: 2,
                preco_unitario: 500.0,
                valor_total: 1_000.0,
                is_resource: false,
                preco_input: "500".to_string(),
            },
        ];

        let mut total_cost = 0.0;
        run_ui_frame(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_items_and_values(ui, &mut app, 900.0, &mut total_cost);
            });
        });

        let expected_per_pass = 2_000.0;
        let passes = total_cost / expected_per_pass;
        assert!(passes >= 1.0);
        assert!((passes - passes.round()).abs() < 1e-9);
    }

    #[test]
    fn render_craft_input_and_closing_render_without_panicking() {
        let mut app = MdcraftApp::default();
        app.input_text = "1 Iron Ore, 2 Screw".to_string();
        app.sell_price_input = "10k".to_string();
        app.items = vec![
            Item {
                nome: "Iron Ore".to_string(),
                quantidade: 1,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: true,
                preco_input: String::new(),
            },
            Item {
                nome: "Screw".to_string(),
                quantidade: 2,
                preco_unitario: 100.5,
                valor_total: 201.0,
                is_resource: false,
                preco_input: "100.5".to_string(),
            },
        ];

        let found_resources = collect_found_resources(&app);

        run_ui_frame(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_craft_input(ui, &mut app, 900.0);
                render_closing(ui, &mut app, 900.0, 201.0, &found_resources);
            });
        });

        assert_eq!(app.sell_price_input, "10k");
    }

    #[test]
    fn autosave_active_craft_updates_selected_entry() {
        let mut app = MdcraftApp::default();
        app.input_text = "1 Iron Ore".to_string();
        app.sell_price_input = "7k".to_string();
        app.items = vec![Item {
            nome: "Iron Ore".to_string(),
            quantidade: 1,
            preco_unitario: 120.0,
            valor_total: 120.0,
            is_resource: true,
            preco_input: "120".to_string(),
        }];
        app.saved_crafts.push(SavedCraft {
            name: "A".to_string(),
            recipe_text: String::new(),
            sell_price_input: String::new(),
            item_prices: vec![],
        });
        app.active_saved_craft_index = Some(0);

        autosave_active_craft(&mut app);

        assert_eq!(app.saved_crafts[0].recipe_text, "1 Iron Ore");
        assert_eq!(app.saved_crafts[0].sell_price_input, "7k");
        assert_eq!(app.saved_crafts[0].item_prices.len(), 1);
        assert_eq!(app.saved_crafts[0].item_prices[0].item_name, "Iron Ore");
        assert_eq!(app.saved_crafts[0].item_prices[0].price_input, "120");
    }

    #[test]
    fn autosave_active_craft_noop_without_active_index() {
        let mut app = MdcraftApp::default();
        app.input_text = "2 Screw".to_string();
        app.sell_price_input = "3k".to_string();
        app.saved_crafts.push(SavedCraft {
            name: "A".to_string(),
            recipe_text: "old".to_string(),
            sell_price_input: "old".to_string(),
            item_prices: vec![],
        });

        autosave_active_craft(&mut app);

        assert_eq!(app.saved_crafts[0].recipe_text, "old");
        assert_eq!(app.saved_crafts[0].sell_price_input, "old");
    }
}

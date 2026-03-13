use eframe::egui;

use crate::parse::parse_clipboard;
use crate::parse::parse_price_flag;

use super::{MdcraftApp, autosave_active_craft, placeholder};

fn lookup_cached_npc_price_input(app: &MdcraftApp, item_name: &str) -> Option<String> {
    let normalized = item_name.trim().to_lowercase();
    app.wiki_cached_items
        .iter()
        .find(|entry| entry.name.trim().to_lowercase() == normalized)
        .and_then(|entry| entry.npc_price.clone())
}

fn apply_cached_npc_price_if_available(app: &MdcraftApp, item: &mut crate::model::Item) {
    let Some(npc_input) = lookup_cached_npc_price_input(app, &item.nome) else {
        return;
    };

    if parse_price_flag(&npc_input).is_err() {
        return;
    }

    item.preco_input = npc_input;
    item.preco_unitario = parse_price_flag(&item.preco_input).unwrap_or(0.0);
    item.valor_total = item.preco_unitario * item.quantidade as f64;
}

fn rebuild_items_from_input(app: &mut MdcraftApp) {
    let resources: Vec<&str> = app.resource_list.iter().map(AsRef::as_ref).collect();
    let old_items = std::mem::take(&mut app.items);
    let mut new_items = parse_clipboard(&app.input_text, &resources);

    for new_item in &mut new_items {
        if let Some(old_item) = old_items.iter().find(|o| o.nome == new_item.nome) {
            new_item.preco_input = old_item.preco_input.clone();
            new_item.preco_unitario = old_item.preco_unitario;
            new_item.valor_total = new_item.preco_unitario * new_item.quantidade as f64;
        } else {
            apply_cached_npc_price_if_available(app, new_item);
        }
    }

    app.items = new_items;
    autosave_active_craft(app);
}

fn apply_input_change(app: &mut MdcraftApp, changed: bool) {
    if changed {
        rebuild_items_from_input(app);
    }
}

pub(crate) fn render_craft_input(ui: &mut egui::Ui, app: &mut MdcraftApp, content_width: f32) {
    ui.group(|ui| {
        ui.set_width(content_width);
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(5))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Digite a receita...")
                        .strong()
                        .size(16.0),
                );
                ui.add_space(5.0);

                let response = ui.add(
                    egui::TextEdit::multiline(&mut app.input_text)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace)
                        .hint_text(placeholder(
                            ui,
                            "1 Appricorn, 80 Screw, 80 Rubber Ball, 10 Iron Ore",
                        ))
                        .margin(egui::vec2(10.0, 10.0)),
                );

                apply_input_change(app, response.changed());
            });
    });
}

#[cfg(test)]
mod tests {
    use crate::data::wiki_scraper::{ScrapedItem, WikiSource};
    use crate::app::{MdcraftApp, SavedCraft};
    use crate::model::Item;

    use super::{
        apply_cached_npc_price_if_available, apply_input_change, lookup_cached_npc_price_input,
        rebuild_items_from_input,
    };

    #[test]
    fn rebuild_items_from_input_parses_recipe_and_preserves_prices() {
        let mut app = MdcraftApp::default();
        app.input_text = "2 Iron Ore, 3 Screw".to_string();
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
    fn rebuild_items_from_input_autosaves_active_craft() {
        let mut app = MdcraftApp::default();
        app.input_text = "1 Iron Ore".to_string();
        app.sell_price_input = "5k".to_string();
        app.saved_crafts.push(SavedCraft {
            name: "A".to_string(),
            recipe_text: String::new(),
            sell_price_input: String::new(),
        });
        app.active_saved_craft_index = Some(0);

        rebuild_items_from_input(&mut app);

        assert_eq!(app.saved_crafts[0].recipe_text, "1 Iron Ore");
        assert_eq!(app.saved_crafts[0].sell_price_input, "5k");
    }

    #[test]
    fn apply_input_change_runs_only_when_changed() {
        let mut app = MdcraftApp::default();
        app.input_text = "1 Iron Ore".to_string();

        apply_input_change(&mut app, false);
        assert!(app.items.is_empty());

        apply_input_change(&mut app, true);
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].nome, "Iron Ore");
    }

    #[test]
    fn lookup_cached_npc_price_input_matches_by_normalized_name() {
        let mut app = MdcraftApp::default();
        app.wiki_cached_items.push(ScrapedItem {
            name: "Test Item Alpha".to_string(),
            npc_price: Some("12k".to_string()),
            sources: vec![WikiSource::Loot],
        });

        let found = lookup_cached_npc_price_input(&app, " test item alpha ");
        assert_eq!(found.as_deref(), Some("12k"));
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
}

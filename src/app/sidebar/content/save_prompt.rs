use eframe::egui;

use crate::app::{MdcraftApp, SavedCraft, capture_saved_item_prices};

use super::super::{capitalize_display_name, placeholder};

pub(super) fn infer_craft_name_from_items(app: &MdcraftApp) -> Option<String> {
    crate::app::infer_craft_name_from_items(
        &app.items,
        &app.craft_recipes_cache,
        &app.craft_recipe_name_by_signature,
    )
    .map(|name| capitalize_display_name(&name))
}

pub(super) fn start_save_recipe_prompt(app: &mut MdcraftApp, save_clicked: bool) {
    if save_clicked {
        app.awaiting_craft_name = true;
        app.pending_craft_name = infer_craft_name_from_items(app).unwrap_or_default();
        app.focus_craft_name_input = true;
    }
}

pub(super) fn update_current_recipe(app: &mut MdcraftApp) {
    let Some(idx) = app.active_saved_craft_index else {
        return;
    };
    if let Some(craft) = app.saved_crafts.get_mut(idx) {
        // craft.recipe_text = app.input_text.clone();
        craft.sell_price_input = app.sell_price_input.clone();
        craft.item_prices = capture_saved_item_prices(&app.items);
    }
    app.persist_saved_crafts_to_sqlite();
}

pub(super) fn render_save_name_prompt(ui: &mut egui::Ui, app: &mut MdcraftApp, content_w: f32) {
    if !app.awaiting_craft_name {
        return;
    }

    ui.add_space(10.0);
    ui.label(egui::RichText::new("Nome da receita:").strong());

    let mut name_resp_opt: Option<egui::Response> = None;
    let accent = ui.visuals().hyperlink_color;
    egui::Frame::NONE
        .fill(egui::Color32::from_rgba_unmultiplied(
            accent.r(),
            accent.g(),
            accent.b(),
            18,
        ))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(
                accent.r(),
                accent.g(),
                accent.b(),
                96,
            ),
        ))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(6, 4))
        .show(ui, |ui| {
            let input_width = (content_w - 12.0).max(80.0);
            let name_resp = ui.add_sized(
                [input_width, 30.0],
                    egui::TextEdit::singleline(&mut app.pending_craft_name)
                        .font(egui::TextStyle::Button)
                        .horizontal_align(egui::Align::Center)
                        .vertical_align(egui::Align::Center)
                        .hint_text("Nome da receita"),
            );
            if name_resp.changed() {
                app.pending_craft_name = app.pending_craft_name.chars()
                    .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '(' || *c == ')')
                    .collect();
            }
            name_resp_opt = Some(name_resp);
        });

    let name_resp = name_resp_opt.expect("name input response should exist");

    if app.focus_craft_name_input {
        name_resp.request_focus();
        app.focus_craft_name_input = false;
    }

    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.awaiting_craft_name = false;
        app.pending_craft_name.clear();
        app.focus_craft_name_input = false;
    }

    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
    if enter_pressed {
        let fallback_name = format!("Receita {}", app.saved_crafts.len() + 1);
        let raw_name = if app.pending_craft_name.trim().is_empty() {
            fallback_name
        } else {
            app.pending_craft_name.clone()
        };
        let normalized_name = capitalize_display_name(&raw_name);
        app.saved_crafts.insert(
            0,
            SavedCraft {
                name: normalized_name,
                recipe_text: String::new(),
                sell_price_input: app.sell_price_input.clone(),
                item_prices: capture_saved_item_prices(&app.items),
            },
        );
        app.active_saved_craft_index = Some(0);
        app.persist_saved_crafts_to_sqlite();
        app.awaiting_craft_name = false;
        app.pending_craft_name.clear();
        app.focus_craft_name_input = false;
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::{MdcraftApp, SavedCraft};
    use crate::data::wiki_scraper::{
        CraftIngredient, CraftProfession, CraftRank, ScrapedCraftRecipe,
    };

    use super::{
        infer_craft_name_from_items, render_save_name_prompt, start_save_recipe_prompt,
        update_current_recipe,
    };

    fn run_with_events(app: &mut MdcraftApp, events: Vec<egui::Event>) {
        let ctx = egui::Context::default();
        let mut input = egui::RawInput::default();
        input.events = events;
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_save_name_prompt(ui, app, 260.0);
            });
        });
    }

    #[test]
    fn start_save_recipe_prompt_only_changes_state_on_click() {
        let mut app = MdcraftApp::default();
        app.pending_craft_name = "keep".to_string();

        start_save_recipe_prompt(&mut app, false);
        assert!(!app.awaiting_craft_name);
        assert_eq!(app.pending_craft_name, "keep");

        start_save_recipe_prompt(&mut app, true);
        assert!(app.awaiting_craft_name);
        assert_eq!(app.pending_craft_name, "");
        assert!(app.focus_craft_name_input);
    }

    #[test]
    fn render_save_name_prompt_escape_cancels_name_prompt() {
        let mut app = MdcraftApp::default();
        app.awaiting_craft_name = true;
        app.pending_craft_name = "Tmp".to_string();
        app.focus_craft_name_input = true;

        run_with_events(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );

        assert!(!app.awaiting_craft_name);
        assert!(app.pending_craft_name.is_empty());
        assert!(!app.focus_craft_name_input);
    }

    #[test]
    fn render_save_name_prompt_enter_saves_pending_recipe_name() {
        let mut app = MdcraftApp::default();
        app.awaiting_craft_name = true;
        app.pending_craft_name = "nova receita".to_string();
        app.input_text = "1 Iron Ore".to_string();
        app.sell_price_input = "9k".to_string();
        app.active_saved_craft_index = Some(1);

        run_with_events(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );

        assert!(!app.awaiting_craft_name);
        assert_eq!(app.saved_crafts.len(), 1);
        assert_eq!(app.saved_crafts[0].name, "Nova Receita");
        assert_eq!(app.saved_crafts[0].sell_price_input, "9k");
        assert_eq!(app.active_saved_craft_index, Some(0));
    }

    #[test]
    fn infer_craft_name_from_items_returns_exact_match() {
        let mut app = MdcraftApp::default();
        app.items = vec![
            crate::model::Item {
                nome: "Apricorn".to_string(),
                quantidade: 1,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: false,
                preco_input: String::new(),
            },
            crate::model::Item {
                nome: "Screw".to_string(),
                quantidade: 80,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: false,
                preco_input: String::new(),
            },
        ];

        app.craft_recipes_cache = vec![ScrapedCraftRecipe {
            profession: CraftProfession::Engineer,
            rank: CraftRank::E,
            name: "poke ball (100x)".to_string(),
            ingredients: vec![
                CraftIngredient {
                    name: "Apricorn".to_string(),
                    quantity: 1.0,
                },
                CraftIngredient {
                    name: "Screw".to_string(),
                    quantity: 80.0,
                },
            ],
        }];
        app.rebuild_craft_recipe_name_index();

        let inferred = infer_craft_name_from_items(&app);
        assert_eq!(inferred.as_deref(), Some("Poke Ball (100x)"));
    }

    #[test]
    fn update_current_recipe_updates_active_craft_in_place() {
        let mut app = MdcraftApp::default();
        app.saved_crafts.push(SavedCraft {
            name: "Receita A".to_string(),
            recipe_text: "1 Ore".to_string(),
            sell_price_input: "1k".to_string(),
            item_prices: vec![],
        });
        app.active_saved_craft_index = Some(0);
        app.input_text = "2 Iron Ore".to_string();
        app.sell_price_input = "9k".to_string();

        update_current_recipe(&mut app);

        assert_eq!(app.saved_crafts[0].name, "Receita A");
        assert_eq!(app.saved_crafts[0].recipe_text, "2 Iron Ore");
        assert_eq!(app.saved_crafts[0].sell_price_input, "9k");
    }

    #[test]
    fn update_current_recipe_noop_without_active_index() {
        let mut app = MdcraftApp::default();
        app.saved_crafts.push(SavedCraft {
            name: "A".to_string(),
            recipe_text: "old".to_string(),
            sell_price_input: "1k".to_string(),
            item_prices: vec![],
        });
        app.active_saved_craft_index = None;
        app.input_text = "new".to_string();

        update_current_recipe(&mut app);

        assert_eq!(app.saved_crafts[0].recipe_text, "old");
    }
}

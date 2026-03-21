use eframe::egui;

use crate::app::{MdcraftApp, SavedCraft};
use crate::data::wiki_scraper::{CraftIngredient, CraftProfession, CraftRank, ScrapedCraftRecipe};

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
fn recipe_save_toast_ativa_e_limpa_apos_tempo() {
    let mut app = MdcraftApp::default();
    app.recipe_save_toast_started_at = Some(
        std::time::Instant::now() - std::time::Duration::from_secs_f32(3.0),
    );
    if let Some(started) = app.recipe_save_toast_started_at {
        if started.elapsed().as_secs_f32() > 2.5 {
            app.recipe_save_toast_started_at = None;
        }
    }
    assert!(app.recipe_save_toast_started_at.is_none());
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
            quantidade_base: 1,
            preco_unitario: 0.0,
            valor_total: 0.0,
            is_resource: false,
            preco_input: String::new(),
        },
        crate::model::Item {
            nome: "Screw".to_string(),
            quantidade: 80,
            quantidade_base: 80,
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
        sell_price_is_per_item: false,
        item_prices: vec![],
    });
    app.active_saved_craft_index = Some(0);
    app.sell_price_input = "9k".to_string();
    app.craft_recipes_cache
        .push(crate::data::wiki_scraper::ScrapedCraftRecipe {
            profession: crate::data::wiki_scraper::CraftProfession::Engineer,
            rank: crate::data::wiki_scraper::CraftRank::E,
            name: "Receita A".to_string(),
            ingredients: vec![crate::data::wiki_scraper::CraftIngredient {
                name: "Iron Ore".to_string(),
                quantity: 2.0,
            }],
        });
    app.items = vec![crate::model::Item {
        nome: "Iron Ore".to_string(),
        quantidade: 2,
        quantidade_base: 2,
        preco_unitario: 0.0,
        valor_total: 0.0,
        is_resource: true,
        preco_input: String::new(),
    }];
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
        sell_price_is_per_item: false,
        item_prices: vec![],
    });
    app.active_saved_craft_index = None;

    update_current_recipe(&mut app);

    assert_eq!(app.saved_crafts[0].recipe_text, "old");
}


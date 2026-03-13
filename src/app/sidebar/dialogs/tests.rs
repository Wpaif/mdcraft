use eframe::egui;

use crate::app::{MdcraftApp, SavedCraft};

use super::delete_logic::{
    apply_delete_recipe, handle_cancel_delete_click, handle_confirm_delete_click,
};
use super::delete_popup::render_delete_confirmation_popup;

#[test]
fn delete_popup_returns_early_when_no_pending_index() {
    let mut app = MdcraftApp::default();
    egui::__run_test_ctx(|ctx| {
        render_delete_confirmation_popup(ctx, &mut app);
    });
    assert_eq!(app.pending_delete_index, None);
}

#[test]
fn delete_popup_clears_invalid_pending_index() {
    let mut app = MdcraftApp::default();
    app.pending_delete_index = Some(0);

    egui::__run_test_ctx(|ctx| {
        render_delete_confirmation_popup(ctx, &mut app);
    });

    assert_eq!(app.pending_delete_index, None);
}

#[test]
fn delete_popup_keeps_state_when_waiting_user_action() {
    let mut app = MdcraftApp::default();
    app.saved_crafts.push(SavedCraft {
        name: "Receita X".to_string(),
        recipe_text: "1 Iron Ore".to_string(),
        sell_price_input: "2k".to_string(),
        item_prices: vec![],
    });
    app.pending_delete_index = Some(0);

    egui::__run_test_ctx(|ctx| {
        render_delete_confirmation_popup(ctx, &mut app);
    });

    assert_eq!(app.saved_crafts.len(), 1);
    assert_eq!(app.pending_delete_index, Some(0));
}

#[test]
fn delete_popup_closes_on_escape() {
    let mut app = MdcraftApp::default();
    app.saved_crafts.push(SavedCraft {
        name: "Receita X".to_string(),
        recipe_text: "1 Iron Ore".to_string(),
        sell_price_input: "2k".to_string(),
        item_prices: vec![],
    });
    app.pending_delete_index = Some(0);

    let ctx = egui::Context::default();
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });

    let _ = ctx.run(input, |ctx| {
        render_delete_confirmation_popup(ctx, &mut app);
    });

    assert_eq!(app.pending_delete_index, None);
    assert_eq!(app.saved_crafts.len(), 1);
}

#[test]
fn apply_delete_recipe_clears_active_when_deleting_active_item() {
    let mut app = MdcraftApp::default();
    app.saved_crafts.push(SavedCraft {
        name: "A".to_string(),
        recipe_text: String::new(),
        sell_price_input: String::new(),
        item_prices: vec![],
    });
    app.active_saved_craft_index = Some(0);
    app.pending_delete_index = Some(0);

    apply_delete_recipe(&mut app, 0);

    assert!(app.saved_crafts.is_empty());
    assert_eq!(app.active_saved_craft_index, None);
    assert_eq!(app.pending_delete_index, None);
}

#[test]
fn apply_delete_recipe_shifts_active_index_when_needed() {
    let mut app = MdcraftApp::default();
    app.saved_crafts.push(SavedCraft {
        name: "A".to_string(),
        recipe_text: String::new(),
        sell_price_input: String::new(),
        item_prices: vec![],
    });
    app.saved_crafts.push(SavedCraft {
        name: "B".to_string(),
        recipe_text: String::new(),
        sell_price_input: String::new(),
        item_prices: vec![],
    });
    app.active_saved_craft_index = Some(1);
    app.pending_delete_index = Some(0);

    apply_delete_recipe(&mut app, 0);

    assert_eq!(app.saved_crafts.len(), 1);
    assert_eq!(app.active_saved_craft_index, Some(0));
    assert_eq!(app.pending_delete_index, None);
}

#[test]
fn apply_delete_recipe_keeps_active_index_when_before_deleted_item() {
    let mut app = MdcraftApp::default();
    app.saved_crafts.push(SavedCraft {
        name: "A".to_string(),
        recipe_text: String::new(),
        sell_price_input: String::new(),
        item_prices: vec![],
    });
    app.saved_crafts.push(SavedCraft {
        name: "B".to_string(),
        recipe_text: String::new(),
        sell_price_input: String::new(),
        item_prices: vec![],
    });
    app.active_saved_craft_index = Some(0);
    app.pending_delete_index = Some(1);

    apply_delete_recipe(&mut app, 1);

    assert_eq!(app.saved_crafts.len(), 1);
    assert_eq!(app.active_saved_craft_index, Some(0));
    assert_eq!(app.pending_delete_index, None);
}

#[test]
fn handle_cancel_delete_click_only_clears_when_clicked() {
    let mut app = MdcraftApp::default();
    app.pending_delete_index = Some(2);

    handle_cancel_delete_click(&mut app, false);
    assert_eq!(app.pending_delete_index, Some(2));

    handle_cancel_delete_click(&mut app, true);
    assert_eq!(app.pending_delete_index, None);
}

#[test]
fn handle_confirm_delete_click_only_deletes_when_clicked() {
    let mut app = MdcraftApp::default();
    app.saved_crafts.push(SavedCraft {
        name: "A".to_string(),
        recipe_text: String::new(),
        sell_price_input: String::new(),
        item_prices: vec![],
    });
    app.pending_delete_index = Some(0);

    handle_confirm_delete_click(&mut app, 0, false);
    assert_eq!(app.saved_crafts.len(), 1);
    assert_eq!(app.pending_delete_index, Some(0));

    handle_confirm_delete_click(&mut app, 0, true);
    assert!(app.saved_crafts.is_empty());
    assert_eq!(app.pending_delete_index, None);
}

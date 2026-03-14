use eframe::egui;

use crate::app::{MdcraftApp, SavedCraft};

use super::handlers::{
    handle_export_close_click, handle_export_copy_click, handle_import_cancel_click,
    handle_import_confirm_click, handle_import_format_click, handle_sidebar_export_click,
    handle_sidebar_import_click, insert_imported_crafts,
};
use super::json_codec::{build_export_json, format_json_pretty, parse_imported_saved_crafts};
use super::state::{
    apply_export_popup_result, close_export_popup, close_import_popup, mark_export_copied,
    open_export_popup, open_import_popup,
};

pub(super) fn sample_craft(name: &str) -> SavedCraft {
    SavedCraft {
        name: name.to_string(),
        recipe_text: "1 Iron Ore".to_string(),
        sell_price_input: "10k".to_string(),
        item_prices: vec![],
    }
}

#[test]
fn build_export_json_outputs_saved_crafts_object() {
    let json = build_export_json(&[sample_craft("Receita A")]).expect("export should work");
    assert!(json.contains("saved_crafts"));
    assert!(json.contains("Receita A"));
}

#[test]
fn parse_imported_saved_crafts_accepts_direct_list() {
    let raw = r#"[
        {"name":"A","recipe_text":"1 X","sell_price_input":"2k"}
    ]"#;
    let crafts = parse_imported_saved_crafts(raw).expect("list payload should parse");
    assert_eq!(crafts.len(), 1);
    assert_eq!(crafts[0].name, "A");
}

#[test]
fn parse_imported_saved_crafts_accepts_saved_crafts_object() {
    let raw = r#"{
        "saved_crafts": [
            {"name":"B","recipe_text":"1 Y","sell_price_input":"3k"}
        ]
    }"#;
    let crafts = parse_imported_saved_crafts(raw).expect("saved_crafts payload should parse");
    assert_eq!(crafts.len(), 1);
    assert_eq!(crafts[0].name, "B");
}

#[test]
fn parse_imported_saved_crafts_accepts_recipes_object() {
    let raw = r#"{
        "recipes": [
            {"name":"C","recipe_text":"1 Z","sell_price_input":"4k"}
        ]
    }"#;
    let crafts = parse_imported_saved_crafts(raw).expect("recipes payload should parse");
    assert_eq!(crafts.len(), 1);
    assert_eq!(crafts[0].name, "C");
}

#[test]
fn parse_imported_saved_crafts_rejects_invalid_json() {
    let err = parse_imported_saved_crafts("{invalid").expect_err("invalid JSON must fail");
    assert!(err.contains("JSON inválido"));
}

#[test]
fn parse_imported_saved_crafts_rejects_too_many_crafts() {
    let mut entries = Vec::new();
    for i in 0..1_001 {
        entries.push(format!(
            "{{\"name\":\"R{i}\",\"recipe_text\":\"1 X\",\"sell_price_input\":\"2k\"}}"
        ));
    }

    let raw = format!("[{}]", entries.join(","));
    let err = parse_imported_saved_crafts(&raw).expect_err("payload must be rejected");
    assert!(err.contains("receitas demais"));
}

#[test]
fn parse_imported_saved_crafts_rejects_oversized_json_payload() {
    let oversized = " ".repeat(2_000_001);
    let err = parse_imported_saved_crafts(&oversized).expect_err("payload must be rejected");
    assert!(err.contains("JSON muito grande"));
}

#[test]
fn parse_imported_saved_crafts_sanitizes_control_characters() {
    let raw = r#"[
        {
            "name":"A\u0000\n\t",
            "recipe_text":"1 Iron Ore\r\n2 Screw\u0007",
            "sell_price_input":" 9k\u0000 ",
            "item_prices":[{"item_name":"Screw\u0000","price_input":"2k\u0000"}]
        }
    ]"#;

    let crafts = parse_imported_saved_crafts(raw).expect("payload should parse and sanitize");
    assert_eq!(crafts.len(), 1);
    assert_eq!(crafts[0].name, "A");
    assert_eq!(crafts[0].recipe_text, "1 Iron Ore\n2 Screw");
    assert_eq!(crafts[0].sell_price_input, "9k");
    assert_eq!(crafts[0].item_prices[0].item_name, "Screw");
    assert_eq!(crafts[0].item_prices[0].price_input, "2k");
}

#[test]
fn format_json_pretty_formats_valid_json() {
    let formatted = format_json_pretty("{\"a\":1}").expect("valid JSON should format");
    assert!(formatted.contains("\n"));
    assert!(formatted.contains("\"a\""));
}

#[test]
fn format_json_pretty_rejects_invalid_json() {
    let err = format_json_pretty("{invalid").expect_err("invalid JSON must fail");
    assert!(err.contains("JSON inválido para formatação"));
}

#[test]
fn insert_imported_crafts_normalizes_and_offsets_active_index() {
    let mut app = MdcraftApp::default();
    app.active_saved_craft_index = Some(1);
    app.saved_crafts.push(sample_craft("existente"));

    let imported = insert_imported_crafts(
        &mut app,
        vec![
            SavedCraft {
                name: "nova receita".to_string(),
                recipe_text: "1 X".to_string(),
                sell_price_input: "2k".to_string(),
                item_prices: vec![],
            },
            SavedCraft {
                name: " ".to_string(),
                recipe_text: "1 Y".to_string(),
                sell_price_input: "3k".to_string(),
                item_prices: vec![],
            },
        ],
    );

    assert_eq!(imported, 2);
    assert_eq!(app.active_saved_craft_index, Some(3));
    assert_eq!(app.saved_crafts[0].name, "Nova Receita");
    assert!(app.saved_crafts[1].name.starts_with("Receita "));
}

#[test]
fn insert_imported_crafts_with_empty_input_keeps_active_index() {
    let mut app = MdcraftApp::default();
    app.active_saved_craft_index = Some(2);

    let imported = insert_imported_crafts(&mut app, vec![]);

    assert_eq!(imported, 0);
    assert_eq!(app.active_saved_craft_index, Some(2));
}

#[test]
fn popup_state_helpers_update_expected_flags() {
    let mut app = MdcraftApp::default();
    app.saved_crafts.push(sample_craft("A"));
    app.import_feedback = Some("x".to_string());
    app.export_feedback = Some("y".to_string());

    open_import_popup(&mut app);
    assert!(app.awaiting_import_json);
    assert_eq!(app.import_feedback, None);

    close_import_popup(&mut app);
    assert!(!app.awaiting_import_json);
    assert_eq!(app.import_feedback, None);

    open_export_popup(&mut app).expect("export popup should open");
    assert!(app.awaiting_export_json);
    assert!(app.export_json_output.contains("saved_crafts"));

    mark_export_copied(&mut app);
    assert_eq!(
        app.export_feedback.as_deref(),
        Some("JSON copiado para a area de transferencia.")
    );

    close_export_popup(&mut app);
    assert!(!app.awaiting_export_json);
    assert_eq!(app.export_feedback, None);
}

#[test]
fn apply_export_popup_result_sets_feedback_on_error() {
    let mut app = MdcraftApp::default();
    apply_export_popup_result(&mut app, Err("erro de teste".to_string()));

    assert_eq!(app.export_feedback.as_deref(), Some("erro de teste"));
    assert!(app.awaiting_export_json);
}

#[test]
fn sidebar_click_handlers_toggle_expected_popup_flags() {
    let mut app = MdcraftApp::default();

    handle_sidebar_import_click(&mut app, true);
    assert!(app.awaiting_import_json);

    app.saved_crafts.push(sample_craft("A"));
    handle_sidebar_export_click(&mut app, true);
    assert!(app.awaiting_export_json);
    assert!(app.export_json_output.contains("saved_crafts"));
}

#[test]
fn import_format_click_handler_covers_empty_invalid_and_valid_paths() {
    let mut app = MdcraftApp::default();

    handle_import_format_click(&mut app, true);
    assert_eq!(
        app.import_feedback.as_deref(),
        Some("Cole um JSON antes de formatar.")
    );

    app.import_json_input = "{invalid".to_string();
    handle_import_format_click(&mut app, true);
    assert!(
        app.import_feedback
            .as_deref()
            .expect("feedback should exist")
            .contains("JSON inválido para formatação")
    );

    app.import_json_input = "{\"a\":1}".to_string();
    handle_import_format_click(&mut app, true);
    assert_eq!(
        app.import_feedback.as_deref(),
        Some("JSON formatado com sucesso.")
    );
    assert!(app.import_json_input.contains('\n'));
}

#[test]
fn import_confirm_click_handler_covers_branches() {
    let mut app = MdcraftApp::default();
    app.awaiting_import_json = true;

    handle_import_confirm_click(&mut app, true);
    assert_eq!(
        app.import_feedback.as_deref(),
        Some("Cole um JSON antes de importar.")
    );

    app.import_json_input = "{invalid".to_string();
    handle_import_confirm_click(&mut app, true);
    assert!(
        app.import_feedback
            .as_deref()
            .expect("feedback should exist")
            .contains("JSON inválido para importação")
    );

    app.import_json_input = "[]".to_string();
    handle_import_confirm_click(&mut app, true);
    assert_eq!(
        app.import_feedback.as_deref(),
        Some("Nenhuma receita encontrada no JSON.")
    );

    app.import_json_input =
        "[{\"name\":\"R\",\"recipe_text\":\"1 X\",\"sell_price_input\":\"2k\"}]".to_string();
    handle_import_confirm_click(&mut app, true);
    assert_eq!(
        app.import_feedback.as_deref(),
        Some("1 receita(s) importada(s) com sucesso.")
    );
    assert!(!app.awaiting_import_json);
    assert!(app.import_json_input.is_empty());
}

#[test]
fn cancel_close_and_copy_handlers_apply_state_changes() {
    let mut app = MdcraftApp::default();
    app.awaiting_import_json = true;
    app.import_feedback = Some("keep".to_string());

    handle_import_cancel_click(&mut app, true);
    assert!(!app.awaiting_import_json);
    assert_eq!(app.import_feedback, None);

    app.awaiting_export_json = true;
    app.export_feedback = Some("old".to_string());
    app.export_json_output = "{}".to_string();

    let ctx = egui::Context::default();
    handle_export_copy_click(&ctx, &mut app, true);
    assert_eq!(
        app.export_feedback.as_deref(),
        Some("JSON copiado para a area de transferencia.")
    );

    handle_export_close_click(&mut app, true);
    assert!(!app.awaiting_export_json);
    assert_eq!(app.export_feedback, None);
}

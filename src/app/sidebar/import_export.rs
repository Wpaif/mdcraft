use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::app::{MdcraftApp, SavedCraft};

use super::normalize_craft_name;

#[derive(Serialize)]
pub(super) struct ExportPayload<'a> {
    saved_crafts: &'a [SavedCraft],
}

pub(super) fn build_export_json(saved_crafts: &[SavedCraft]) -> Result<String, String> {
    let payload = ExportPayload { saved_crafts };
    serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Erro ao gerar JSON de exportação: {err}"))
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum ImportPayload {
    List(Vec<SavedCraft>),
    SavedCrafts { saved_crafts: Vec<SavedCraft> },
    Recipes { recipes: Vec<SavedCraft> },
}

pub(super) fn parse_imported_saved_crafts(raw_json: &str) -> Result<Vec<SavedCraft>, String> {
    let payload: ImportPayload = serde_json::from_str(raw_json)
        .map_err(|err| format!("JSON inválido para importação: {err}"))?;

    let crafts = match payload {
        ImportPayload::List(items) => items,
        ImportPayload::SavedCrafts { saved_crafts } => saved_crafts,
        ImportPayload::Recipes { recipes } => recipes,
    };

    Ok(crafts)
}

pub(super) fn format_json_pretty(raw_json: &str) -> Result<String, String> {
    let value = serde_json::from_str::<serde_json::Value>(raw_json)
        .map_err(|err| format!("JSON inválido para formatação: {err}"))?;

    serde_json::to_string_pretty(&value).map_err(|err| format!("Erro ao formatar JSON: {err}"))
}

pub(super) fn insert_imported_crafts(app: &mut MdcraftApp, crafts: Vec<SavedCraft>) -> usize {
    let mut imported = 0usize;

    for craft in crafts.into_iter().rev() {
        let fallback_name = format!("Receita {}", app.saved_crafts.len() + 1);
        let name = if craft.name.trim().is_empty() {
            fallback_name
        } else {
            normalize_craft_name(&craft.name)
        };

        app.saved_crafts.insert(
            0,
            SavedCraft {
                name,
                recipe_text: craft.recipe_text,
                sell_price_input: craft.sell_price_input,
                item_prices: craft.item_prices,
            },
        );
        imported += 1;
    }

    if imported > 0 {
        app.active_saved_craft_index = app.active_saved_craft_index.map(|idx| idx + imported);
    }

    imported
}

pub(super) fn open_import_popup(app: &mut MdcraftApp) {
    app.awaiting_import_json = true;
    app.import_feedback = None;
}

pub(super) fn close_import_popup(app: &mut MdcraftApp) {
    app.awaiting_import_json = false;
    app.import_feedback = None;
}

pub(super) fn open_export_popup(app: &mut MdcraftApp) -> Result<(), String> {
    let json = build_export_json(&app.saved_crafts)?;
    app.export_json_output = json;
    app.export_feedback = None;
    app.awaiting_export_json = true;
    Ok(())
}

pub(super) fn close_export_popup(app: &mut MdcraftApp) {
    app.awaiting_export_json = false;
    app.export_feedback = None;
}

pub(super) fn mark_export_copied(app: &mut MdcraftApp) {
    app.export_feedback = Some("JSON copiado para a area de transferencia.".to_string());
}

pub(super) fn handle_sidebar_import_click(app: &mut MdcraftApp, import_clicked: bool) {
    if import_clicked {
        open_import_popup(app);
    }
}

pub(super) fn handle_sidebar_export_click(app: &mut MdcraftApp, export_clicked: bool) {
    if export_clicked {
        let result = open_export_popup(app);
        apply_export_popup_result(app, result);
    }
}

pub(super) fn apply_export_popup_result(app: &mut MdcraftApp, result: Result<(), String>) {
    if let Err(err) = result {
        app.export_feedback = Some(err);
        app.awaiting_export_json = true;
    }
}

pub(super) fn handle_import_format_click(app: &mut MdcraftApp, format_clicked: bool) {
    if !format_clicked {
        return;
    }

    let raw_json = app.import_json_input.trim();
    if raw_json.is_empty() {
        app.import_feedback = Some("Cole um JSON antes de formatar.".to_string());
    } else {
        match format_json_pretty(raw_json) {
            Ok(pretty) => {
                app.import_json_input = pretty;
                app.import_feedback = Some("JSON formatado com sucesso.".to_string());
            }
            Err(err) => {
                app.import_feedback = Some(err);
            }
        }
    }
}

pub(super) fn handle_import_confirm_click(app: &mut MdcraftApp, import_clicked: bool) {
    if !import_clicked {
        return;
    }

    let raw_json = app.import_json_input.trim();
    if raw_json.is_empty() {
        app.import_feedback = Some("Cole um JSON antes de importar.".to_string());
        return;
    }

    match parse_imported_saved_crafts(raw_json) {
        Ok(crafts) => {
            if crafts.is_empty() {
                app.import_feedback = Some("Nenhuma receita encontrada no JSON.".to_string());
                return;
            }

            let imported = insert_imported_crafts(app, crafts);

            app.persist_saved_crafts_to_sqlite();

            app.import_feedback =
                Some(format!("{} receita(s) importada(s) com sucesso.", imported));
            app.awaiting_import_json = false;
            app.import_json_input.clear();
        }
        Err(err) => {
            app.import_feedback = Some(err);
        }
    }
}

pub(super) fn handle_export_copy_click(ctx: &egui::Context, app: &mut MdcraftApp, copied: bool) {
    if copied {
        ctx.copy_text(app.export_json_output.clone());
        mark_export_copied(app);
    }
}

pub(super) fn handle_import_cancel_click(app: &mut MdcraftApp, cancel_clicked: bool) {
    if cancel_clicked {
        close_import_popup(app);
    }
}

pub(super) fn handle_export_close_click(app: &mut MdcraftApp, close_clicked: bool) {
    if close_clicked {
        close_export_popup(app);
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use super::{
        apply_export_popup_result, build_export_json, close_export_popup, close_import_popup,
        format_json_pretty, handle_export_close_click, handle_export_copy_click,
        handle_import_cancel_click, handle_import_confirm_click, handle_import_format_click,
        handle_sidebar_export_click, handle_sidebar_import_click, insert_imported_crafts,
        mark_export_copied, open_export_popup, open_import_popup, parse_imported_saved_crafts,
    };
    use crate::app::{MdcraftApp, SavedCraft};

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
}

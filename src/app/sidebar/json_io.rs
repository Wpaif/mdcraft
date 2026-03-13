use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::app::{MdcraftApp, SavedCraft};

use super::{normalize_craft_name, placeholder};

fn action_button_colors(ui: &egui::Ui) -> (egui::Color32, egui::Stroke, egui::Color32) {
    let accent = ui.visuals().hyperlink_color;
    let fill = egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 56);
    let stroke = egui::Stroke::new(1.0, accent.gamma_multiply(0.9));
    let text = ui.visuals().text_color();
    (fill, stroke, text)
}

#[derive(Clone, Copy)]
enum JsonContainer {
    Object { expecting_key: bool },
    Array,
}

fn push_json_text(job: &mut egui::text::LayoutJob, text: &str, color: egui::Color32) {
    if text.is_empty() {
        return;
    }

    job.append(
        text,
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(13.0),
            color,
            ..Default::default()
        },
    );
}

fn json_layout_job(ui: &egui::Ui, text: &str, wrap_width: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let default_color = ui.visuals().text_color();
    let punct_color = default_color.gamma_multiply(0.9);
    let key_color = ui.visuals().hyperlink_color;
    let string_color = default_color.gamma_multiply(0.95);
    let number_color = ui.visuals().warn_fg_color;
    let bool_color = egui::Color32::from_rgb(96, 197, 139);
    let null_color = ui.visuals().error_fg_color;

    let mut stack: Vec<JsonContainer> = Vec::new();
    let mut i = 0usize;
    let bytes = text.as_bytes();

    while i < bytes.len() {
        let ch = bytes[i] as char;

        if ch.is_whitespace() {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i] as char).is_whitespace() {
                i += 1;
            }
            push_json_text(&mut job, &text[start..i], default_color);
            continue;
        }

        match ch {
            '{' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                stack.push(JsonContainer::Object {
                    expecting_key: true,
                });
                i += 1;
            }
            '}' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                stack.pop();
                i += 1;
            }
            '[' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                stack.push(JsonContainer::Array);
                i += 1;
            }
            ']' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                stack.pop();
                i += 1;
            }
            ':' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                if let Some(JsonContainer::Object { expecting_key }) = stack.last_mut() {
                    *expecting_key = false;
                }
                i += 1;
            }
            ',' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                if let Some(JsonContainer::Object { expecting_key }) = stack.last_mut() {
                    *expecting_key = true;
                }
                i += 1;
            }
            '"' => {
                let start = i;
                i += 1;
                let mut escaped = false;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }

                let is_key = matches!(
                    stack.last(),
                    Some(JsonContainer::Object {
                        expecting_key: true
                    })
                );
                let color = if is_key { key_color } else { string_color };
                push_json_text(&mut job, &text[start..i], color);
            }
            '-' | '0'..='9' => {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-') {
                        i += 1;
                    } else {
                        break;
                    }
                }
                push_json_text(&mut job, &text[start..i], number_color);
            }
            't' if text[i..].starts_with("true") => {
                push_json_text(&mut job, "true", bool_color);
                i += 4;
            }
            'f' if text[i..].starts_with("false") => {
                push_json_text(&mut job, "false", bool_color);
                i += 5;
            }
            'n' if text[i..].starts_with("null") => {
                push_json_text(&mut job, "null", null_color);
                i += 4;
            }
            _ => {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if c.is_whitespace() || matches!(c, '{' | '}' | '[' | ']' | ':' | ',' | '"') {
                        break;
                    }
                    i += 1;
                }
                push_json_text(&mut job, &text[start..i], default_color);
            }
        }
    }

    job
}

pub(super) fn render_sidebar_json_actions(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
    content_w: f32,
    has_saved_crafts: bool,
) {
    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    let (action_fill, action_stroke, action_text) = action_button_colors(ui);

    let import_clicked = ui
        .add_sized(
            [content_w, 34.0],
            egui::Button::new(
                egui::RichText::new("Importar receitas (JSON)")
                    .strong()
                    .color(action_text),
            )
            .fill(action_fill)
            .stroke(action_stroke),
        )
        .on_hover_text("Cole um JSON com receitas salvas para importar em lote")
        .clicked();

    handle_sidebar_import_click(app, import_clicked);

    if has_saved_crafts {
        ui.add_space(8.0);

        let export_clicked = ui
            .add_sized(
                [content_w, 34.0],
                egui::Button::new(
                    egui::RichText::new("Exportar receitas (JSON)")
                        .strong()
                        .color(action_text),
                )
                .fill(action_fill)
                .stroke(action_stroke),
            )
            .on_hover_text("Gerar JSON com todas as receitas salvas")
            .clicked();

        handle_sidebar_export_click(app, export_clicked);
    }
}

#[derive(Serialize)]
struct ExportPayload<'a> {
    saved_crafts: &'a [SavedCraft],
}

fn build_export_json(saved_crafts: &[SavedCraft]) -> Result<String, String> {
    let payload = ExportPayload { saved_crafts };
    serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Erro ao gerar JSON de exportação: {err}"))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ImportPayload {
    List(Vec<SavedCraft>),
    SavedCrafts { saved_crafts: Vec<SavedCraft> },
    Recipes { recipes: Vec<SavedCraft> },
}

fn parse_imported_saved_crafts(raw_json: &str) -> Result<Vec<SavedCraft>, String> {
    let payload: ImportPayload = serde_json::from_str(raw_json)
        .map_err(|err| format!("JSON inválido para importação: {err}"))?;

    let crafts = match payload {
        ImportPayload::List(items) => items,
        ImportPayload::SavedCrafts { saved_crafts } => saved_crafts,
        ImportPayload::Recipes { recipes } => recipes,
    };

    Ok(crafts)
}

fn format_json_pretty(raw_json: &str) -> Result<String, String> {
    let value =
        serde_json::from_str::<serde_json::Value>(raw_json).map_err(|err| {
            format!("JSON inválido para formatação: {err}")
        })?;

    serde_json::to_string_pretty(&value).map_err(|err| format!("Erro ao formatar JSON: {err}"))
}

fn insert_imported_crafts(app: &mut MdcraftApp, crafts: Vec<SavedCraft>) -> usize {
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
            },
        );
        imported += 1;
    }

    if imported > 0 {
        app.active_saved_craft_index = app.active_saved_craft_index.map(|idx| idx + imported);
    }

    imported
}

fn open_import_popup(app: &mut MdcraftApp) {
    app.awaiting_import_json = true;
    app.import_feedback = None;
}

fn close_import_popup(app: &mut MdcraftApp) {
    app.awaiting_import_json = false;
    app.import_feedback = None;
}

fn open_export_popup(app: &mut MdcraftApp) -> Result<(), String> {
    let json = build_export_json(&app.saved_crafts)?;
    app.export_json_output = json;
    app.export_feedback = None;
    app.awaiting_export_json = true;
    Ok(())
}

fn close_export_popup(app: &mut MdcraftApp) {
    app.awaiting_export_json = false;
    app.export_feedback = None;
}

fn mark_export_copied(app: &mut MdcraftApp) {
    app.export_feedback = Some("JSON copiado para a area de transferencia.".to_string());
}

fn handle_sidebar_import_click(app: &mut MdcraftApp, import_clicked: bool) {
    if import_clicked {
        open_import_popup(app);
    }
}

fn handle_sidebar_export_click(app: &mut MdcraftApp, export_clicked: bool) {
    if export_clicked {
        let result = open_export_popup(app);
        apply_export_popup_result(app, result);
    }
}

fn apply_export_popup_result(app: &mut MdcraftApp, result: Result<(), String>) {
    if let Err(err) = result {
        app.export_feedback = Some(err);
        app.awaiting_export_json = true;
    }
}

fn handle_import_format_click(app: &mut MdcraftApp, format_clicked: bool) {
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

fn handle_import_confirm_click(app: &mut MdcraftApp, import_clicked: bool) {
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

            app.import_feedback = Some(format!("{} receita(s) importada(s) com sucesso.", imported));
            app.awaiting_import_json = false;
            app.import_json_input.clear();
        }
        Err(err) => {
            app.import_feedback = Some(err);
        }
    }
}

fn handle_export_copy_click(ctx: &egui::Context, app: &mut MdcraftApp, copied: bool) {
    if copied {
        ctx.copy_text(app.export_json_output.clone());
        mark_export_copied(app);
    }
}

fn handle_import_cancel_click(app: &mut MdcraftApp, cancel_clicked: bool) {
    if cancel_clicked {
        close_import_popup(app);
    }
}

fn handle_export_close_click(app: &mut MdcraftApp, close_clicked: bool) {
    if close_clicked {
        close_export_popup(app);
    }
}

pub(super) fn render_import_recipes_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
    if !app.awaiting_import_json {
        return;
    }

    egui::Window::new("Importar Receitas")
        .id(egui::Id::new("import_saved_recipes_json"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(560.0, 390.0))
        .show(ctx, |ui| {
            let mut json_layouter =
                |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                    ui.ctx().fonts_mut(|fonts| {
                        fonts.layout_job(json_layout_job(ui, text.as_str(), wrap_width))
                    })
                };

            let (action_fill, action_stroke, action_text) = action_button_colors(ui);

            ui.label(
                egui::RichText::new("Cole aqui o JSON no formato de receitas salvas")
                    .strong()
                    .size(15.0),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(
                    "Aceita: lista direta (`[{...}]`) ou objeto com `saved_crafts`.",
                )
                .weak(),
            );

            ui.add_space(8.0);
            ui.add_sized(
                [ui.available_width(), 240.0],
                egui::TextEdit::multiline(&mut app.import_json_input)
                    .font(egui::TextStyle::Monospace)
                    .hint_text(placeholder(ui, "[{\"name\":\"Receita X\", ...}]"))
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(10.0, 10.0))
                    .layouter(&mut json_layouter),
            );

            if let Some(feedback) = &app.import_feedback {
                ui.add_space(6.0);
                ui.label(feedback);
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let format_clicked = ui
                    .add_sized(
                        [140.0, 32.0],
                        egui::Button::new(
                            egui::RichText::new("Formatar JSON")
                                .strong()
                                .color(action_text),
                        )
                        .fill(action_fill)
                        .stroke(action_stroke),
                    )
                    .on_hover_text("Organiza e indenta o JSON colado")
                    .clicked();

                handle_import_format_click(app, format_clicked);

                let cancel_clicked = ui
                    .add_sized([120.0, 32.0], egui::Button::new("Cancelar"))
                    .clicked();
                handle_import_cancel_click(app, cancel_clicked);

                let import_clicked = ui
                    .add_sized(
                        [140.0, 32.0],
                        egui::Button::new(egui::RichText::new("Importar").strong()),
                    )
                    .clicked();

                handle_import_confirm_click(app, import_clicked);
            });
        });
}

pub(super) fn render_export_recipes_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
    if !app.awaiting_export_json {
        return;
    }

    egui::Window::new("Exportar Receitas")
        .id(egui::Id::new("export_saved_recipes_json"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(560.0, 390.0))
        .show(ctx, |ui| {
            let mut json_layouter =
                |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                    ui.ctx().fonts_mut(|fonts| {
                        fonts.layout_job(json_layout_job(ui, text.as_str(), wrap_width))
                    })
                };

            let (action_fill, action_stroke, action_text) = action_button_colors(ui);

            ui.label(
                egui::RichText::new("JSON de exportação das receitas salvas")
                    .strong()
                    .size(15.0),
            );
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Copie o conteúdo abaixo.").weak());
            ui.add_space(8.0);

            if let Some(feedback) = &app.export_feedback {
                ui.label(feedback);
            }

            ui.add_sized(
                [ui.available_width(), 270.0],
                egui::TextEdit::multiline(&mut app.export_json_output)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(10.0, 10.0))
                    .layouter(&mut json_layouter)
                    .interactive(false),
            );

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let copied = ui
                    .add_sized(
                        [120.0, 32.0],
                        egui::Button::new(
                            egui::RichText::new("Copiar").strong().color(action_text),
                        )
                        .fill(action_fill)
                        .stroke(action_stroke),
                    )
                    .on_hover_text("Copiar JSON para a area de transferencia")
                    .clicked();

                handle_export_copy_click(ui.ctx(), app, copied);

                let close_clicked = ui
                    .add_sized([120.0, 32.0], egui::Button::new("Fechar"))
                    .clicked();
                handle_export_close_click(app, close_clicked);
            });
        });
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::MdcraftApp;
    use super::{
        action_button_colors, apply_export_popup_result, build_export_json, close_export_popup,
        close_import_popup, format_json_pretty, handle_export_close_click, handle_export_copy_click,
        handle_import_cancel_click, handle_import_confirm_click, handle_import_format_click,
        handle_sidebar_export_click, handle_sidebar_import_click, insert_imported_crafts,
        json_layout_job, mark_export_copied, open_export_popup, open_import_popup,
        parse_imported_saved_crafts, push_json_text,
    };
    use crate::app::SavedCraft;

    fn sample_craft(name: &str) -> SavedCraft {
        SavedCraft {
            name: name.to_string(),
            recipe_text: "1 Iron Ore".to_string(),
            sell_price_input: "10k".to_string(),
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
    fn render_sidebar_json_actions_runs_for_both_empty_and_non_empty_state() {
        let mut empty_app = MdcraftApp::default();
        let mut app_with_crafts = MdcraftApp::default();
        app_with_crafts.saved_crafts.push(sample_craft("Com dados"));

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                super::render_sidebar_json_actions(ui, &mut empty_app, 220.0, false);
            });
        });

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                super::render_sidebar_json_actions(ui, &mut app_with_crafts, 220.0, true);
            });
        });
    }

    #[test]
    fn render_import_popup_returns_early_when_closed() {
        let mut app = MdcraftApp::default();
        app.awaiting_import_json = false;

        egui::__run_test_ctx(|ctx| {
            super::render_import_recipes_popup(ctx, &mut app);
        });

        assert!(!app.awaiting_import_json);
    }

    #[test]
    fn render_import_popup_open_state_runs_without_panicking() {
        let mut app = MdcraftApp::default();
        app.awaiting_import_json = true;
        app.import_json_input = "[{\"name\":\"A\",\"recipe_text\":\"1 X\",\"sell_price_input\":\"2k\"}]".to_string();

        egui::__run_test_ctx(|ctx| {
            super::render_import_recipes_popup(ctx, &mut app);
        });

        assert!(app.awaiting_import_json);
    }

    #[test]
    fn render_export_popup_open_state_runs_without_panicking() {
        let mut app = MdcraftApp::default();
        app.awaiting_export_json = true;
        app.export_json_output = build_export_json(&[sample_craft("A")]).expect("export should work");

        egui::__run_test_ctx(|ctx| {
            super::render_export_recipes_popup(ctx, &mut app);
        });

        assert!(app.awaiting_export_json);
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
                },
                SavedCraft {
                    name: " ".to_string(),
                    recipe_text: "1 Y".to_string(),
                    sell_price_input: "3k".to_string(),
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
    fn json_layout_job_and_push_json_text_generate_sections() {
        egui::__run_test_ui(|ui| {
            let job = json_layout_job(ui, "{\"a\":1,true:false,null:[1,2]}", 300.0);
            assert!(!job.sections.is_empty());

            let escaped = json_layout_job(ui, "{\"k\":\"a\\\\\"b\",x:[1,2],u:foo}", 300.0);
            assert!(!escaped.sections.is_empty());

            let punctuation_without_object = json_layout_job(ui, "[:,]", 300.0);
            assert!(!punctuation_without_object.sections.is_empty());

            let scientific_number = json_layout_job(ui, "{\"n\":1.2e-3}", 300.0);
            assert!(!scientific_number.sections.is_empty());

            let bare_identifier = json_layout_job(ui, "foobar", 300.0);
            assert!(!bare_identifier.sections.is_empty());

            let comma_outside = json_layout_job(ui, ",", 300.0);
            assert!(!comma_outside.sections.is_empty());
        });

        let mut job = egui::text::LayoutJob::default();
        push_json_text(&mut job, "", egui::Color32::WHITE);
        assert!(job.sections.is_empty());
        push_json_text(&mut job, "abc", egui::Color32::WHITE);
        assert_eq!(job.sections.len(), 1);
    }

    #[test]
    fn action_button_colors_returns_non_zero_stroke_and_alpha() {
        egui::__run_test_ui(|ui| {
            let (fill, stroke, _text) = action_button_colors(ui);
            assert!(fill.a() > 0);
            assert!(stroke.width > 0.0);
        });
    }

    #[test]
    fn apply_export_popup_result_sets_feedback_on_error() {
        let mut app = MdcraftApp::default();
        apply_export_popup_result(&mut app, Err("erro de teste".to_string()));

        assert_eq!(app.export_feedback.as_deref(), Some("erro de teste"));
        assert!(app.awaiting_export_json);
    }

    #[test]
    fn render_popups_show_feedback_labels_when_present() {
        let mut app = MdcraftApp::default();
        app.awaiting_import_json = true;
        app.import_feedback = Some("import feedback".to_string());
        app.import_json_input = "[]".to_string();

        egui::__run_test_ctx(|ctx| {
            super::render_import_recipes_popup(ctx, &mut app);
        });

        app.awaiting_export_json = true;
        app.export_feedback = Some("export feedback".to_string());
        app.export_json_output = "{}".to_string();

        egui::__run_test_ctx(|ctx| {
            super::render_export_recipes_popup(ctx, &mut app);
        });
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
        assert_eq!(app.import_feedback.as_deref(), Some("Cole um JSON antes de formatar."));

        app.import_json_input = "{invalid".to_string();
        handle_import_format_click(&mut app, true);
        assert!(app
            .import_feedback
            .as_deref()
            .expect("feedback should exist")
            .contains("JSON inválido para formatação"));

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
        assert_eq!(app.import_feedback.as_deref(), Some("Cole um JSON antes de importar."));

        app.import_json_input = "{invalid".to_string();
        handle_import_confirm_click(&mut app, true);
        assert!(app
            .import_feedback
            .as_deref()
            .expect("feedback should exist")
            .contains("JSON inválido para importação"));

        app.import_json_input = "[]".to_string();
        handle_import_confirm_click(&mut app, true);
        assert_eq!(
            app.import_feedback.as_deref(),
            Some("Nenhuma receita encontrada no JSON.")
        );

        app.import_json_input =
            "[{\"name\":\"R\",\"recipe_text\":\"1 X\",\"sell_price_input\":\"2k\"}]"
                .to_string();
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

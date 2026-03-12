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

    if import_clicked {
        app.awaiting_import_json = true;
        app.import_feedback = None;
    }

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

        if export_clicked {
            match build_export_json(&app.saved_crafts) {
                Ok(json) => {
                    app.export_json_output = json;
                    app.export_feedback = None;
                    app.awaiting_export_json = true;
                }
                Err(err) => {
                    app.export_feedback = Some(err);
                    app.awaiting_export_json = true;
                }
            }
        }
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

                if format_clicked {
                    let raw_json = app.import_json_input.trim();
                    if raw_json.is_empty() {
                        app.import_feedback = Some("Cole um JSON antes de formatar.".to_string());
                    } else {
                        match serde_json::from_str::<serde_json::Value>(raw_json) {
                            Ok(value) => match serde_json::to_string_pretty(&value) {
                                Ok(pretty) => {
                                    app.import_json_input = pretty;
                                    app.import_feedback =
                                        Some("JSON formatado com sucesso.".to_string());
                                }
                                Err(err) => {
                                    app.import_feedback =
                                        Some(format!("Erro ao formatar JSON: {err}"));
                                }
                            },
                            Err(err) => {
                                app.import_feedback =
                                    Some(format!("JSON inválido para formatação: {err}"));
                            }
                        }
                    }
                }

                if ui
                    .add_sized([120.0, 32.0], egui::Button::new("Cancelar"))
                    .clicked()
                {
                    app.awaiting_import_json = false;
                    app.import_feedback = None;
                }

                let import_clicked = ui
                    .add_sized(
                        [140.0, 32.0],
                        egui::Button::new(egui::RichText::new("Importar").strong()),
                    )
                    .clicked();

                if import_clicked {
                    let raw_json = app.import_json_input.trim();
                    if raw_json.is_empty() {
                        app.import_feedback = Some("Cole um JSON antes de importar.".to_string());
                        return;
                    }

                    match parse_imported_saved_crafts(raw_json) {
                        Ok(crafts) => {
                            if crafts.is_empty() {
                                app.import_feedback =
                                    Some("Nenhuma receita encontrada no JSON.".to_string());
                                return;
                            }

                            let mut imported = 0usize;
                            for craft in crafts.into_iter().rev() {
                                let fallback_name =
                                    format!("Receita {}", app.saved_crafts.len() + 1);
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
                                app.active_saved_craft_index =
                                    app.active_saved_craft_index.map(|idx| idx + imported);
                            }

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

                if copied {
                    ui.ctx().copy_text(app.export_json_output.clone());
                    app.export_feedback =
                        Some("JSON copiado para a area de transferencia.".to_string());
                }

                if ui
                    .add_sized([120.0, 32.0], egui::Button::new("Fechar"))
                    .clicked()
                {
                    app.awaiting_export_json = false;
                    app.export_feedback = None;
                }
            });
        });
}

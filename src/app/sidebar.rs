use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::parse::parse_clipboard;

use super::{MdcraftApp, SavedCraft};

const SIDEBAR_WIDTH_EXPANDED: f32 = 260.0;
const SIDEBAR_WIDTH_COLLAPSED: f32 = 56.0;

fn placeholder(ui: &egui::Ui, text: &str) -> egui::RichText {
    egui::RichText::new(text).color(ui.visuals().text_color().gamma_multiply(0.7))
}

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
                    if c.is_ascii_digit()
                        || matches!(c, '.' | 'e' | 'E' | '+' | '-')
                    {
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
                    if c.is_whitespace()
                        || matches!(c, '{' | '}' | '[' | ']' | ':' | ',' | '"')
                    {
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

pub(super) fn render_sidebar(ctx: &egui::Context, app: &mut MdcraftApp) {
    let width = if app.sidebar_open {
        SIDEBAR_WIDTH_EXPANDED
    } else {
        SIDEBAR_WIDTH_COLLAPSED
    };

    egui::SidePanel::left(egui::Id::new("sidebar_panel"))
        .resizable(false)
        .exact_width(width)
        .show_separator_line(false)
        .show(ctx, |ui| {
            let panel_fill = ui.visuals().panel_fill;

            egui::Frame::NONE
                .fill(panel_fill)
                .inner_margin(egui::Margin::symmetric(10, 10))
                .show(ui, |ui| {
                    let content_w = ui.available_width();
                    render_sidebar_header(ui, app);

                    if app.sidebar_open {
                        render_sidebar_content(ui, app, content_w);
                    }
                });
        });

    render_delete_confirmation_popup(ctx, app);
    render_import_recipes_popup(ctx, app);
    render_export_recipes_popup(ctx, app);
}

fn render_sidebar_content(ui: &mut egui::Ui, app: &mut MdcraftApp, content_w: f32) {
    let content_w = content_w.max(120.0);
    let has_saved_crafts = !app.saved_crafts.is_empty();

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(10.0);

    let footer_h = if has_saved_crafts { 126.0 } else { 86.0 };
    let scroll_h = (ui.available_height() - footer_h).max(120.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(scroll_h)
        .show(ui, |ui| {
            let has_recipe = !app.input_text.trim().is_empty() && !app.items.is_empty();
            if has_recipe {
                let save_clicked = ui
                    .add_sized([content_w, 32.0], egui::Button::new("Salvar receita atual"))
                    .clicked();

                if save_clicked {
                    app.awaiting_craft_name = true;
                    app.pending_craft_name.clear();
                    app.focus_craft_name_input = true;
                }
            } else {
                ui.label(egui::RichText::new("Adicione uma receita para salvar").weak());
            }

            if app.awaiting_craft_name {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Nome da receita:").strong());

                let mut name_resp_opt: Option<egui::Response> = None;
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui| {
                        let input_width = (content_w - 12.0).max(80.0);
                        let name_resp = ui.add_sized(
                            [input_width, 30.0],
                            egui::TextEdit::singleline(&mut app.pending_craft_name)
                                .hint_text(placeholder(ui, "Digite um nome ou pressione Enter")),
                        );
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
                let save_by_enter = enter_pressed;

                if save_by_enter {
                    let fallback_name = format!("Receita {}", app.saved_crafts.len() + 1);
                    let raw_name = if app.pending_craft_name.trim().is_empty() {
                        fallback_name
                    } else {
                        app.pending_craft_name.clone()
                    };
                    let normalized_name = normalize_craft_name(&raw_name);
                    app.saved_crafts.insert(
                        0,
                        SavedCraft {
                            name: normalized_name,
                            recipe_text: app.input_text.clone(),
                            sell_price_input: app.sell_price_input.clone(),
                        },
                    );
                    app.active_saved_craft_index = app.active_saved_craft_index.map(|idx| idx + 1);
                    app.awaiting_craft_name = false;
                    app.pending_craft_name.clear();
                    app.focus_craft_name_input = false;
                }
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Receitas salvas").strong());
            ui.add_space(6.0);

            if app.saved_crafts.is_empty() {
                ui.label(egui::RichText::new("Nenhuma receita salva ainda.").weak());
            } else {
                let mut pending_click_delete: Option<usize> = None;
                let mut pending_click_select: Option<usize> = None;

                for (idx, craft) in app.saved_crafts.iter().enumerate() {
                    ui.group(|ui| {
                        ui.set_width(content_w);
                        let is_active = app.active_saved_craft_index == Some(idx);
                        let name_text = normalize_craft_name(&craft.name);
                        let saved_lines = craft
                            .recipe_text
                            .lines()
                            .filter(|line| !line.trim().is_empty())
                            .count();
                        let has_saved_price = !craft.sell_price_input.trim().is_empty();
                        let hover_details = if has_saved_price {
                            format!("{} linhas salvas | com preco final", saved_lines)
                        } else {
                            format!("{} linhas salvas", saved_lines)
                        };
                        let row_height = 26.0;
                        let icon_size = 22.0;
                        ui.allocate_ui_with_layout(
                            egui::vec2(content_w, row_height),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                let text_width =
                                    (content_w - icon_size - ui.spacing().item_spacing.x - 8.0)
                                        .max(80.0);

                                let name_fill = if is_active {
                                    ui.visuals().faint_bg_color
                                } else {
                                    ui.visuals().widgets.inactive.bg_fill
                                };
                                let name_stroke = if is_active {
                                    ui.visuals().widgets.active.bg_stroke
                                } else {
                                    ui.visuals().widgets.inactive.bg_stroke
                                };

                                let name_btn = egui::Button::new(
                                    egui::RichText::new(name_text)
                                        .size(16.0)
                                        .color(ui.visuals().text_color()),
                                )
                                .fill(name_fill)
                                .stroke(name_stroke);

                                let name_resp = ui
                                    .add_sized([text_width, icon_size], name_btn)
                                    .on_hover_text(hover_details);

                                if name_resp.clicked() {
                                    pending_click_select = Some(idx);
                                }

                                let delete_btn = egui::Button::new(
                                    egui::RichText::new("🗑")
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(220, 98, 98)),
                                )
                                .fill(egui::Color32::from_rgba_unmultiplied(220, 98, 98, 32))
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    egui::Color32::from_rgb(180, 72, 72),
                                ));

                                let delete_clicked = ui
                                    .add_sized([icon_size, icon_size], delete_btn)
                                    .on_hover_text("Excluir receita")
                                    .clicked();

                                if delete_clicked {
                                    pending_click_delete = Some(idx);
                                }
                            },
                        );
                    });
                    ui.add_space(6.0);
                }

                if let Some(idx) = pending_click_delete {
                    app.pending_delete_index = Some(idx);
                }

                if let Some(idx) = pending_click_select {
                    load_saved_craft_for_edit(app, idx);
                }
            }
        });

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

fn render_import_recipes_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
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
            let mut json_layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                ui.ctx().fonts_mut(|fonts| fonts.layout_job(json_layout_job(ui, text.as_str(), wrap_width)))
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
                        app.import_feedback =
                            Some("Cole um JSON antes de formatar.".to_string());
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

fn render_export_recipes_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
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
            let mut json_layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                ui.ctx().fonts_mut(|fonts| fonts.layout_job(json_layout_job(ui, text.as_str(), wrap_width)))
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
                    app.export_feedback = Some("JSON copiado para a area de transferencia.".to_string());
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

fn load_saved_craft_for_edit(app: &mut MdcraftApp, idx: usize) {
    let Some(craft) = app.saved_crafts.get(idx) else {
        return;
    };

    app.input_text = craft.recipe_text.clone();
    app.sell_price_input = craft.sell_price_input.clone();

    let resources: Vec<&str> = app.resource_list.iter().map(AsRef::as_ref).collect();
    app.items = parse_clipboard(&app.input_text, &resources);
    app.active_saved_craft_index = Some(idx);
}

fn normalize_craft_name(raw_name: &str) -> String {
    raw_name
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let first = first.to_uppercase().collect::<String>();
                    let rest = chars.as_str().to_lowercase();
                    format!("{}{}", first, rest)
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_delete_confirmation_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
    let Some(idx) = app.pending_delete_index else {
        return;
    };

    if idx >= app.saved_crafts.len() {
        app.pending_delete_index = None;
        return;
    }

    let recipe_name = app.saved_crafts[idx].name.clone();

    egui::Window::new("Confirmar Exclusão")
        .id(egui::Id::new("confirm_delete_saved_recipe"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(400.0, 190.0))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Deseja realmente apagar esta receita?")
                        .strong()
                        .size(16.0),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("'{}'", recipe_name))
                        .weak()
                        .size(14.0),
                );
            });

            ui.add_space(12.0);
            let cancel_fill = ui.visuals().widgets.inactive.bg_fill;
            let delete_fill = egui::Color32::from_rgb(181, 61, 61);
            let row_height = 32.0;
            let button_width = 120.0;
            let spacing = ui.spacing().item_spacing.x;
            let total_buttons_width = (button_width * 2.0) + spacing;
            let left_pad = ((ui.available_width() - total_buttons_width) * 0.5).max(0.0);

            ui.horizontal(|ui| {
                ui.add_space(left_pad);

                if ui
                    .add_sized(
                        [button_width, row_height],
                        egui::Button::new("Cancelar").fill(cancel_fill),
                    )
                    .clicked()
                {
                    app.pending_delete_index = None;
                }

                if ui
                    .add_sized(
                        [button_width, row_height],
                        egui::Button::new(
                            egui::RichText::new("Apagar")
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(delete_fill),
                    )
                    .clicked()
                {
                    app.saved_crafts.remove(idx);

                    if let Some(active_idx) = app.active_saved_craft_index {
                        app.active_saved_craft_index = if active_idx == idx {
                            None
                        } else if active_idx > idx {
                            Some(active_idx - 1)
                        } else {
                            Some(active_idx)
                        };
                    }

                    app.pending_delete_index = None;
                }
            });
        });
}

fn render_sidebar_header(ui: &mut egui::Ui, app: &mut MdcraftApp) {
    ui.horizontal(|ui| {
        let toggle_icon = if app.sidebar_open { "◀" } else { "▶" };
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());

        let bg = if resp.hovered() {
            ui.visuals().widgets.hovered.bg_fill
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(6), bg);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            toggle_icon,
            egui::TextStyle::Button.resolve(ui.style()),
            ui.visuals().text_color(),
        );

        if resp.clicked() {
            app.sidebar_open = !app.sidebar_open;
        }

        if app.sidebar_open {
            ui.label(egui::RichText::new("RECEITAS").strong());
        }
    });
}

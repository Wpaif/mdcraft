use super::MdcraftApp;
use eframe::egui;
pub use logic::apply_cached_npc_price_if_available;
pub mod local_search_thread;
mod logic;
#[cfg(test)]
mod tests;
use local_search_thread::LocalSearchResult;

pub(crate) fn render_craft_input(ui: &mut egui::Ui, app: &mut MdcraftApp, content_width: f32) {
    ui.group(|ui| {
        ui.set_width(content_width);
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(5))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Buscar item para craftar")
                        .strong()
                        .size(16.0),
                );
                ui.add_space(8.0);

                let mut should_update_grid = false;

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Nome do item:");
                        let search_response = ui.add_sized([
                            220.0, 32.0
                        ],
                            egui::TextEdit::singleline(&mut app.craft_search_query)
                                .hint_text("Digite o nome do item")
                                .margin(egui::vec2(8.0, 8.0))
                        );
                        if search_response.changed() {
                            app.craft_search_query = app.craft_search_query.chars()
                                .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '(' || *c == ')')
                                .collect();
                        }
                        if search_response.changed() {
                            should_update_grid = true;
                            if let Some(tx) = &app.es_query_tx {
                                let _ = tx.send(crate::app::ui_sections::craft_input::local_search_thread::LocalSearchMsg::Query(app.craft_search_query.clone()));
                            }
                        }
                    });
                    ui.add_space(32.0);
                    ui.vertical(|ui| {
                        ui.label("Quantidade:");
                        let qty_response = ui.add_sized([
                            70.0, 32.0
                        ],
                            egui::TextEdit::singleline(&mut app.craft_search_qty_input)
                                .hint_text("1")
                                .margin(egui::vec2(8.0, 8.0))
                                .char_limit(4)
                        );
                        // Normaliza input: permite apagar tudo com backspace enquanto edita,
                        // mas só aplica quando for um inteiro válido (1..=9999).
                        if qty_response.changed() {
                            app.craft_search_qty_input.retain(|c| c.is_ascii_digit());
                            if let Ok(val) = app.craft_search_qty_input.parse::<u64>() {
                                if (1..=9999).contains(&val) && val != app.craft_search_qty {
                                    app.craft_search_qty = val;
                                    should_update_grid = true;
                                    for item in &mut app.items {
                                        item.quantidade = item.quantidade_base * app.craft_search_qty;
                                        crate::app::ui_sections::items_grid::apply_item_price_from_input(item);
                                    }
                                }
                            }
                        }

                        let enter_pressed =
                            qty_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        let commit = qty_response.lost_focus() || enter_pressed;
                        if commit {
                            let trimmed = app.craft_search_qty_input.trim();
                            if trimmed.is_empty() {
                                // Reverte para o valor atual (mantém a UX de "campo normal").
                                app.craft_search_qty_input = app.craft_search_qty.to_string();
                            } else if let Ok(val) = trimmed.parse::<u64>() {
                                let clamped = val.clamp(1, 9999);
                                if clamped != app.craft_search_qty {
                                    app.craft_search_qty = clamped;
                                    should_update_grid = true;
                                    for item in &mut app.items {
                                        item.quantidade = item.quantidade_base * app.craft_search_qty;
                                        crate::app::ui_sections::items_grid::apply_item_price_from_input(item);
                                    }
                                }
                                // Normaliza string (remove zeros à esquerda, aplica clamp).
                                app.craft_search_qty_input = app.craft_search_qty.to_string();
                            } else {
                                app.craft_search_qty_input = app.craft_search_qty.to_string();
                            }
                        }
                    });
                });

                if let Some(rx) = &app.es_result_rx {
                    while let Ok(result) = rx.try_recv() {
                        match result {
                            LocalSearchResult::Suggestions(suggestions) => {
                                app.es_suggestions = suggestions;
                                app.es_error = None;
                            }
                        }
                    }
                }

                if !app.es_suggestions.is_empty() {
                    ui.label("Sugestões:");
                    let mut clicked_suggestion: Option<String> = None;
                    let button_width = 220.0;
                    let button_height = 32.0;
                    ui.horizontal_wrapped(|ui| {
                        for suggestion in &app.es_suggestions {
                            let button = egui::Button::new(
                                egui::RichText::new(suggestion)
                                    .strong()
                                    .color(egui::Color32::from_rgb(220, 220, 220))
                                    .size(16.0)
                            )
                                .min_size(egui::vec2(button_width, button_height))
                                .wrap()
                                .fill(egui::Color32::from_rgb(40, 40, 40))
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)));
                            if ui.add(button).clicked() {
                                clicked_suggestion = Some(suggestion.clone());
                            }
                            ui.add_space(8.0);
                        }
                    });
                    if let Some(suggestion) = clicked_suggestion {
                        if let Some(recipe) = app.craft_recipes_cache.iter().find(|r| r.name == suggestion) {
                            app.items.clear();
                            for ing in &recipe.ingredients {
                                let is_resource = app.resource_list.iter().any(|res| res.eq_ignore_ascii_case(&ing.name));
                                let mut item = crate::model::Item {
                                    nome: ing.name.clone(),
                                    quantidade: (ing.quantity as u64) * app.craft_search_qty.max(1),
                                    quantidade_base: ing.quantity as u64,
                                    preco_unitario: 0.0,
                                    valor_total: 0.0,
                                    is_resource,
                                    preco_input: String::new(),
                                };
                                crate::app::ui_sections::craft_input::logic::apply_cached_npc_price_if_available(app, &mut item);
                                app.items.push(item);
                            }
                            app.selected_craft_name = recipe.name.clone();
                            app.craft_search_query.clear();
                            app.es_suggestions.clear();
                            should_update_grid = false;
                                        if !app.selected_craft_name.is_empty() {
                                            ui.add_space(10.0);
                                            ui.label("Nome do craft selecionado (pode editar):");
                                            let name_resp = ui.add(
                                                egui::TextEdit::singleline(&mut app.selected_craft_name)
                                                    .desired_width(300.0)
                                                    .font(egui::TextStyle::Heading)
                                            );
                                            if name_resp.changed() {
                                                app.selected_craft_name = app.selected_craft_name.chars()
                                                    .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '(' || *c == ')')
                                                    .collect();
                                            }
                                        }
                        } else {
                            app.craft_search_query = suggestion;
                            should_update_grid = true;
                        }
                    }
                }
                if let Some(err) = &app.es_error {
                    ui.colored_label(egui::Color32::RED, err);
                }

                if should_update_grid {
                    let query = app.craft_search_query.trim();
                    if !query.is_empty() {
                        if let Some(wiki_item) = app.wiki_cached_items.iter().find(|entry| entry.name.eq_ignore_ascii_case(query)) {
                            let nome = wiki_item.name.clone();
                            let quantidade = app.craft_search_qty.max(1);
                            let mut found = false;
                            for item in &mut app.items {
                                if item.nome.eq_ignore_ascii_case(&nome) {
                                    item.quantidade = quantidade;
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                app.items.push(crate::model::Item {
                                    nome,
                                    quantidade: quantidade,
                                    quantidade_base: 1,
                                    preco_unitario: 0.0,
                                    valor_total: 0.0,
                                    is_resource: false,
                                    preco_input: String::new(),
                                });
                            }
                        }
                    }
                }
            });
    });
}

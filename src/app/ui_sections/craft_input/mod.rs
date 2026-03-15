pub use logic::apply_cached_npc_price_if_available;
use eframe::egui;

use super::MdcraftApp;
pub mod local_search_thread;

mod logic;
#[cfg(test)]
mod tests;

use local_search_thread::LocalSearchResult;
// poll_elasticsearch removido: integração ElasticSearch não existe mais

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

                // Inputs lado a lado, alinhados pela base, sem centralização vertical
                // Simula duas "divs" lado a lado, cada uma com label acima do input
                ui.horizontal(|ui| {
                    // Primeira div: Nome do item
                    ui.vertical(|ui| {
                        ui.label("Nome do item:");
                        let search_response = ui.add_sized([
                            220.0, 32.0
                        ],
                            egui::TextEdit::singleline(&mut app.craft_search_query)
                                .hint_text("Digite o nome do item")
                                .margin(egui::vec2(8.0, 8.0))
                        );
                        // Filtro: só permite caracteres alfanuméricos e parênteses
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
                    // Segunda div: Quantidade
                    ui.vertical(|ui| {
                        ui.label("Quantidade:");
                        // Simula <input type='number'>
                        let mut qty_str = app.craft_search_qty.to_string();
                        let qty_response = ui.add_sized([
                            70.0, 32.0
                        ],
                            egui::TextEdit::singleline(&mut qty_str)
                                .hint_text("1")
                                .margin(egui::vec2(8.0, 8.0))
                                .char_limit(4)
                        );
                        if qty_response.changed() {
                            if let Ok(val) = qty_str.parse::<u32>() {
                                app.craft_search_qty = (val.clamp(1, 9999)) as u64;
                            }
                            should_update_grid = true;
                            // Atualiza a quantidade de todos os itens do grid e recalcula o valor_total
                            let nova_qtd = app.craft_search_qty.max(1);
                            for item in &mut app.items {
                                item.quantidade = nova_qtd;
                                crate::app::ui_sections::items_grid::apply_item_price_from_input(item);
                            }
                        }
                    });
                });

                // poll_elasticsearch removido: integração ElasticSearch não existe mais
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
                                    quantidade: ing.quantity as u64,
                                    preco_unitario: 0.0,
                                    valor_total: 0.0,
                                    is_resource,
                                    preco_input: String::new(),
                                };
                                crate::app::ui_sections::craft_input::logic::apply_cached_npc_price_if_available(app, &mut item);
                                app.items.push(item);
                            }
                            // Salva o nome do craft selecionado para edição final
                            app.selected_craft_name = recipe.name.clone();
                            app.craft_search_query.clear();
                            app.es_suggestions.clear();
                            should_update_grid = false;
                                        // Campo editável para o nome do craft selecionado
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

                // Integração com ElasticSearch (placeholder: chamada síncrona não é possível, mas mostra como integrar)
                // Em produção, use um runtime async para buscar sugestões e atualizar a UI
                // Exemplo:
                // let client = elasticsearch::Elasticsearch::default();
                // if let Ok(suggestions) = futures::executor::block_on(elasticsearch::search_items(&client, &app.craft_search_query)) {
                //     // Renderize sugestões/autocomplete na UI
                // }

                // Atualiza o grid automaticamente se houver match válido local
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
                                    quantidade,
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

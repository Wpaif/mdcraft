use eframe::egui;

use crate::parse::{parse_clipboard, parse_price_flag};
use crate::units::format_game_units;

use super::MdcraftApp;
use super::price::{PriceStatus, paint_price_status};

fn placeholder(ui: &egui::Ui, text: &str) -> egui::RichText {
    egui::RichText::new(text).color(ui.visuals().text_color().gamma_multiply(0.7))
}

fn capitalize_display_name(raw_name: &str) -> String {
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

pub(super) fn render_craft_input(ui: &mut egui::Ui, app: &mut MdcraftApp, content_width: f32) {
    ui.group(|ui| {
        ui.set_width(content_width);
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(5))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Digite a receita...")
                        .strong()
                        .size(16.0),
                );
                ui.add_space(5.0);

                let response = ui.add(
                    egui::TextEdit::multiline(&mut app.input_text)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace)
                        .hint_text(placeholder(
                            ui,
                            "1 Appricorn, 80 Screw, 80 Rubber Ball, 10 Iron Ore",
                        ))
                        .margin(egui::vec2(10.0, 10.0)),
                );

                if response.changed() {
                    let resources: Vec<&str> =
                        app.resource_list.iter().map(AsRef::as_ref).collect();
                    let old_items = std::mem::take(&mut app.items);
                    let mut new_items = parse_clipboard(&app.input_text, &resources);

                    for new_item in &mut new_items {
                        if let Some(old_item) = old_items.iter().find(|o| o.nome == new_item.nome) {
                            new_item.preco_input = old_item.preco_input.clone();
                            new_item.preco_unitario = old_item.preco_unitario;
                            new_item.valor_total = new_item.preco_unitario * new_item.quantidade;
                        }
                    }

                    app.items = new_items;
                }
            });
    });
}

pub(super) fn collect_found_resources(app: &MdcraftApp) -> Vec<(String, u64)> {
    app.items
        .iter()
        .filter(|item| item.is_resource)
        .map(|item| (item.nome.clone(), item.quantidade))
        .collect()
}

pub(super) fn render_items_and_values(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
    content_width: f32,
    total_cost: &mut u64,
) {
    if app.items.is_empty() {
        return;
    }

    ui.group(|ui| {
        ui.set_width(content_width);
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(5))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Itens e Valores")
                        .strong()
                        .size(16.0),
                );
                ui.add_space(10.0);

                let available_space_for_cols = (content_width - 10.0).max(300.0);
                let field_gap = 8.0;
                let indices_precificaveis: Vec<usize> = app
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(_, res)| !res.is_resource)
                    .map(|(i, _)| i)
                    .collect();

                let min_qty_w = 46.0;
                let min_price_w = 96.0;
                let min_total_w = 78.0;
                let min_status_w = 56.0;
                let longest_name_chars = indices_precificaveis
                    .iter()
                    .map(|&idx| app.items[idx].nome.chars().count())
                    .max()
                    .unwrap_or(10);
                let preferred_item_w = (longest_name_chars as f32 * 8.0 + 20.0).clamp(120.0, 360.0);
                let min_col_width = preferred_item_w
                    + min_qty_w
                    + min_price_w
                    + min_total_w
                    + min_status_w
                    + (field_gap * 4.0);
                let max_columns = (((available_space_for_cols + field_gap)
                    / (min_col_width + field_gap))
                    .floor() as usize)
                    .max(1);

                let num_items = indices_precificaveis.len();
                let column_count = if num_items == 0 {
                    1
                } else {
                    max_columns.clamp(1, num_items)
                };
                let rows = (num_items + column_count - 1) / column_count;

                let total_gaps = ((column_count * 5).saturating_sub(1)) as f32 * field_gap;
                let per_col_width = ((available_space_for_cols - total_gaps) / column_count as f32)
                    .max(min_col_width - (field_gap * 4.0));
                let qty_w = min_qty_w;
                let price_w = min_price_w;
                let total_w = min_total_w;
                let status_w = min_status_w;
                let item_w = (per_col_width - qty_w - price_w - total_w - status_w).max(120.0);

                egui::ScrollArea::vertical()
                    .max_height(350.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        egui::Frame::NONE
                            .inner_margin(egui::Margin::same(5))
                            .show(ui, |ui| {
                                egui::Grid::new("items_grid_multi")
                                    .num_columns(column_count * 5)
                                    .spacing([field_gap, 10.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for _ in 0..column_count {
                                            ui.add_sized(
                                                [item_w, 20.0],
                                                egui::Label::new(
                                                    egui::RichText::new("Item").size(14.0),
                                                ),
                                            );
                                            ui.add_sized(
                                                [qty_w, 20.0],
                                                egui::Label::new(
                                                    egui::RichText::new("Qtd").size(14.0),
                                                ),
                                            );
                                            ui.add_sized(
                                                [price_w, 20.0],
                                                egui::Label::new(
                                                    egui::RichText::new("Preço").size(14.0),
                                                ),
                                            );
                                            ui.add_sized(
                                                [total_w, 20.0],
                                                egui::Label::new(
                                                    egui::RichText::new("Total").size(14.0),
                                                ),
                                            );
                                            ui.add_sized(
                                                [status_w, 20.0],
                                                egui::Label::new(
                                                    egui::RichText::new("Status").size(14.0),
                                                ),
                                            );
                                        }
                                        ui.end_row();

                                        for row in 0..rows {
                                            for col in 0..column_count {
                                                let idx = col * rows + row;
                                                if idx < indices_precificaveis.len() {
                                                    let real_idx = indices_precificaveis[idx];
                                                    let item = &mut app.items[real_idx];
                                                    let nome_exibido =
                                                        capitalize_display_name(&item.nome);

                                                    let name_resp = ui.allocate_ui_with_layout(
                                                        egui::vec2(item_w, 22.0),
                                                        egui::Layout::left_to_right(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            ui.add_sized(
                                                                [item_w, 22.0],
                                                                egui::Label::new(
                                                                    egui::RichText::new(
                                                                        &nome_exibido,
                                                                    )
                                                                    .strong(),
                                                                )
                                                                .wrap(),
                                                            )
                                                        },
                                                    );
                                                    name_resp.response.on_hover_text(&nome_exibido);

                                                    ui.add_sized(
                                                        [qty_w, 22.0],
                                                        egui::Label::new(
                                                            item.quantidade.to_string(),
                                                        ),
                                                    );

                                                    let text_edit = egui::TextEdit::singleline(
                                                        &mut item.preco_input,
                                                    )
                                                    .hint_text(placeholder(ui, "0"))
                                                    .desired_width(price_w - 8.0)
                                                    .margin(egui::vec2(8.0, 8.0));

                                                    if ui
                                                        .add_sized([price_w, 24.0], text_edit)
                                                        .changed()
                                                    {
                                                        item.preco_unitario =
                                                            parse_price_flag(&item.preco_input)
                                                                .unwrap_or(0);
                                                        item.valor_total =
                                                            item.preco_unitario * item.quantidade;
                                                    }

                                                    ui.add_sized(
                                                        [total_w, 22.0],
                                                        egui::Label::new(egui::RichText::new(
                                                            format_game_units(
                                                                item.valor_total as f64,
                                                            ),
                                                        )),
                                                    );

                                                    let status = if !item.preco_input.is_empty()
                                                        && parse_price_flag(&item.preco_input)
                                                            .is_err()
                                                    {
                                                        PriceStatus::Invalid
                                                    } else if item.valor_total > 0 {
                                                        PriceStatus::Ok
                                                    } else {
                                                        PriceStatus::None
                                                    };

                                                    let hover = match status {
                                                        PriceStatus::Invalid => {
                                                            Some("Valor Inválido")
                                                        }
                                                        PriceStatus::Ok => Some("OK"),
                                                        PriceStatus::None => None,
                                                    };

                                                    ui.allocate_ui_with_layout(
                                                        egui::vec2(status_w, 22.0),
                                                        egui::Layout::left_to_right(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            let resp =
                                                                paint_price_status(ui, status);
                                                            if let Some(text) = hover {
                                                                resp.on_hover_text(text);
                                                            }
                                                        },
                                                    );

                                                    *total_cost += item.valor_total;
                                                } else {
                                                    ui.add_sized(
                                                        [item_w, 22.0],
                                                        egui::Label::new(" "),
                                                    );
                                                    ui.add_sized(
                                                        [qty_w, 22.0],
                                                        egui::Label::new(" "),
                                                    );
                                                    ui.add_sized(
                                                        [price_w, 22.0],
                                                        egui::Label::new(" "),
                                                    );
                                                    ui.add_sized(
                                                        [total_w, 22.0],
                                                        egui::Label::new(" "),
                                                    );
                                                    ui.add_sized(
                                                        [status_w, 22.0],
                                                        egui::Label::new(" "),
                                                    );
                                                }
                                            }
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
            });
    });
}

pub(super) fn render_closing(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
    content_width: f32,
    total_cost: u64,
    found_resources: &[(String, u64)],
) {
    ui.group(|ui| {
        ui.set_width(content_width);
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(5))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Fechamento")
                        .strong()
                        .size(16.0),
                );
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_sized([150.0, 32.0], egui::Label::new("Preço de Venda Final:"));
                    ui.add(
                        egui::TextEdit::singleline(&mut app.sell_price_input)
                            .hint_text(placeholder(ui, "100k"))
                            .desired_width(180.0)
                            .margin(egui::vec2(12.0, 10.0)),
                    );
                });

                ui.add_space(15.0);

                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.label("CUSTO TOTAL");
                        ui.heading(egui::RichText::new(format_game_units(total_cost as f64)));
                    });

                    ui.add_space(40.0);

                    let sell_price = parse_price_flag(&app.sell_price_input).unwrap_or(0);
                    if sell_price > 0 {
                        let lucro_total = sell_price.saturating_sub(total_cost);
                        let is_profit = sell_price >= total_cost;
                        let color = if is_profit {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        };

                        ui.vertical(|ui| {
                            ui.label("RECEITA TOTAL");
                            ui.heading(egui::RichText::new(format_game_units(sell_price as f64)));
                        });

                        ui.add_space(40.0);

                        ui.vertical(|ui| {
                            ui.label("LUCRO LÍQUIDO");
                            ui.heading(
                                egui::RichText::new(format_game_units(lucro_total as f64))
                                    .color(color),
                            );
                        });

                        ui.add_space(40.0);

                        ui.vertical(|ui| {
                            let margem = if total_cost > 0 {
                                lucro_total as f64 / total_cost as f64 * 100.0
                            } else {
                                0.0
                            };

                            ui.label(
                                egui::RichText::new(format!("MARGEM: {:.1}%", margem))
                                    .strong()
                                    .color(color),
                            );

                            if !found_resources.is_empty() {
                                ui.add_space(5.0);

                                egui::Frame::NONE
                                    .inner_margin(egui::Margin::symmetric(8, 6))
                                    .show(ui, |ui| {
                                        egui::Grid::new("resources_cost_grid")
                                            .spacing([10.0, 2.0])
                                            .show(ui, |ui| {
                                                for (res_name, res_qtd) in found_resources {
                                                    if *res_qtd > 0 {
                                                        let custo_por_ponto =
                                                            lucro_total as f64 / *res_qtd as f64;

                                                        ui.label(format!(
                                                            "{} {}",
                                                            res_qtd, res_name
                                                        ));
                                                        ui.label("-");
                                                        ui.label(format!(
                                                            "{:.1} por pt",
                                                            custo_por_ponto
                                                        ));
                                                        ui.end_row();
                                                    }
                                                }
                                            });
                                    });
                            }
                        });
                    }
                });
            });
    });
}

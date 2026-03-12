use eframe::egui;

use crate::parse::{parse_clipboard, parse_price_flag};
use crate::units::format_game_units;

use super::price::{PriceStatus, paint_price_status};
use super::styles::{setup_custom_styles, setup_emoji_support};
use super::theme_state::theme_toggle_button;

impl eframe::App for super::MdcraftApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.fonts_loaded {
            setup_custom_styles(ctx);
            setup_emoji_support(ctx);
            ctx.set_visuals(self.theme.visuals());
            self.fonts_loaded = true;
        }

        // Floating theme-toggle button in the top right corner.
        egui::Area::new(egui::Id::new("theme_toggle_area"))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                if theme_toggle_button(ui, self.theme)
                    .on_hover_text("Alternar tema")
                    .clicked()
                {
                    self.theme = self.theme.toggle();
                    ctx.set_visuals(self.theme.visuals());
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_width = ui.available_width();
            let max_width = available_width.min(1600.0);
            let padding = ((available_width - max_width) / 2.0).max(10.0) as i8;

            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(padding, 20))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("Mdcraft Calculator").strong());
                    });

                    ui.add_space(20.0);

                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.label(egui::RichText::new("📋 Cole a lista do craft do jogo:").strong());
                        ui.add_space(5.0);

                        let response = ui.add(
                            egui::TextEdit::multiline(&mut self.input_text)
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Monospace)
                                .margin(egui::vec2(10.0, 10.0)),
                        );

                        if response.changed() {
                            let resources: Vec<&str> =
                                self.resource_list.iter().map(AsRef::as_ref).collect();

                            let old_items = std::mem::take(&mut self.items);
                            let mut new_items = parse_clipboard(&self.input_text, &resources);

                            for new_item in new_items.iter_mut() {
                                if let Some(old_item) =
                                    old_items.iter().find(|o| o.nome == new_item.nome)
                                {
                                    new_item.preco_input = old_item.preco_input.clone();
                                    new_item.preco_unitario = old_item.preco_unitario;
                                    new_item.valor_total =
                                        new_item.preco_unitario * new_item.quantidade;
                                }
                            }

                            self.items = new_items;
                        }
                    });

                    ui.add_space(20.0);

                    let mut total_cost: u64 = 0;
                    let mut found_resources: Vec<(String, u64)> = Vec::new();

                    for item in &self.items {
                        if item.is_resource {
                            found_resources.push((item.nome.clone(), item.quantidade));
                        }
                    }

                    if !self.items.is_empty() {
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.label(egui::RichText::new("🛒 Itens e Valores").strong());
                            ui.add_space(10.0);

                            // Newspaper layout: compute columns and rows, fill top-to-bottom.
                            let available_space_for_cols = ui.available_width();
                            let min_col_width = 360.0;
                            let max_columns = (available_space_for_cols / min_col_width).floor() as usize;

                            let indices_precificaveis: Vec<usize> = self
                                .items
                                .iter()
                                .enumerate()
                                .filter(|(_, res)| !res.is_resource)
                                .map(|(i, _)| i)
                                .collect();

                            let num_items = indices_precificaveis.len();
                            let column_count = if num_items == 0 { 1 } else { max_columns.clamp(1, num_items) };
                            let rows = (num_items + column_count - 1) / column_count; // ceil

                            egui::ScrollArea::vertical()
                                .max_height(350.0)
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    // Each item uses 5 logical cells: Item, Qtd, Preço Unit., Total, Status
                                    egui::Grid::new("items_grid_multi")
                                        .num_columns((column_count * 5) as usize)
                                        .spacing([15.0, 12.0])
                                        .striped(true)
                                        .show(ui, |ui| {
                                            // Header repeated per column
                                            for _col in 0..column_count {
                                                ui.heading(egui::RichText::new("Item").size(14.0));
                                                ui.heading(egui::RichText::new("Qtd").size(14.0));
                                                ui.heading(egui::RichText::new("Preço Unit.").size(14.0));
                                                ui.heading(egui::RichText::new("Total").size(14.0));
                                                ui.heading(egui::RichText::new("Status").size(14.0));
                                            }
                                            ui.end_row();

                                            for row in 0..rows {
                                                for col in 0..column_count {
                                                    let idx = col * rows + row;
                                                    if idx < indices_precificaveis.len() {
                                                        let real_idx = indices_precificaveis[idx];
                                                        let item = &mut self.items[real_idx];

                                                        let nome_truncado = if item.nome.len() > 25 {
                                                            format!("{}...", &item.nome[..22])
                                                        } else {
                                                            item.nome.clone()
                                                        };

                                                        ui.label(egui::RichText::new(nome_truncado).strong())
                                                            .on_hover_text(&item.nome);

                                                        ui.label(item.quantidade.to_string());

                                                        let text_edit = egui::TextEdit::singleline(&mut item.preco_input)
                                                            .desired_width(140.0)
                                                            .margin(egui::vec2(8.0, 8.0));

                                                        if ui.add(text_edit).changed() {
                                                            item.preco_unitario = parse_price_flag(&item.preco_input).unwrap_or(0);
                                                            item.valor_total = item.preco_unitario * item.quantidade;
                                                        }

                                                        ui.label(egui::RichText::new(format_game_units(item.valor_total as f64)));

                                                        let status = if !item.preco_input.is_empty() && parse_price_flag(&item.preco_input).is_err() {
                                                            PriceStatus::Invalid
                                                        } else if item.valor_total > 0 {
                                                            PriceStatus::Ok
                                                        } else {
                                                            PriceStatus::None
                                                        };

                                                        let hover = match status {
                                                            PriceStatus::Invalid => Some("Valor Inválido"),
                                                            PriceStatus::Ok => Some("OK"),
                                                            PriceStatus::None => None,
                                                        };

                                                        let resp = paint_price_status(ui, status);
                                                        if let Some(text) = hover {
                                                            resp.on_hover_text(text);
                                                        }

                                                        total_cost += item.valor_total;
                                                    } else {
                                                        // filler cells to keep grid alignment
                                                        ui.label(" ");
                                                        ui.label(" ");
                                                        ui.label(" ");
                                                        ui.label(" ");
                                                        ui.label(" ");
                                                    }
                                                }
                                                ui.end_row();
                                            }
                                        });
                                });
                        });
                    }

                    ui.add_space(20.0);

                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.label(egui::RichText::new("💰 Fechamento").strong());
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label("Preço de Venda Final:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.sell_price_input)
                                    .desired_width(180.0)
                                    .margin(egui::vec2(12.0, 10.0)),
                            );
                        });

                        ui.add_space(15.0);

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label("CUSTO TOTAL");
                                ui.heading(egui::RichText::new(format_game_units(total_cost as f64)));
                            });

                            ui.add_space(40.0);

                            let sell_price = parse_price_flag(&self.sell_price_input).unwrap_or(0);
                            if sell_price > 0 {
                                let lucro_total = sell_price.saturating_sub(total_cost);
                                let is_profit = sell_price >= total_cost;
                                let color = if is_profit { egui::Color32::GREEN } else { egui::Color32::RED };

                                ui.vertical(|ui| {
                                    ui.label("RECEITA TOTAL");
                                    ui.heading(egui::RichText::new(format_game_units(
                                        sell_price as f64,
                                    )));
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

                                        egui::Grid::new("resources_cost_grid")
                                            .spacing([10.0, 2.0])
                                            .show(ui, |ui| {
                                                for (res_name, res_qtd) in &found_resources {
                                                    if *res_qtd > 0 {
                                                        let custo_por_ponto = lucro_total as f64 / *res_qtd as f64;

                                                        ui.label(format!("{} {}", res_qtd, res_name));
                                                        ui.label("-");
                                                        ui.label(format!(
                                                            "{:.1} por pt",
                                                            custo_por_ponto
                                                        ));
                                                        ui.end_row();
                                                    }
                                                }
                                            });
                                    }
                                });
                            }
                        });
                    });
                });
        });
    }
}

use eframe::egui;

use crate::parse::parse_price_flag;
use crate::units::format_game_units;

use super::super::price::{PriceStatus, paint_price_status};
use super::MdcraftApp;
use super::capitalize_display_name;
use super::placeholder;

pub(crate) fn render_items_and_values(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
    content_width: f32,
    total_cost: &mut f64,
) {
    if app.items.is_empty() {
        return;
    }

    ui.group(|ui| {
        ui.set_width(content_width);
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(5))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Itens e Valores").strong().size(16.0));
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
                                                    .desired_width(price_w - 8.0);

                                                    let price_changed = ui
                                                        .allocate_ui_with_layout(
                                                            egui::vec2(price_w, 22.0),
                                                            egui::Layout::left_to_right(
                                                                egui::Align::Center,
                                                            ),
                                                            |ui| {
                                                                ui.add_sized(
                                                                    [price_w, 24.0],
                                                                    text_edit,
                                                                )
                                                                .changed()
                                                            },
                                                        )
                                                        .inner;

                                                    if price_changed {
                                                        item.preco_unitario =
                                                            parse_price_flag(&item.preco_input)
                                                                .unwrap_or(0.0);
                                                        item.valor_total =
                                                            item.preco_unitario
                                                                * item.quantidade as f64;
                                                    }

                                                    ui.add_sized(
                                                        [total_w, 22.0],
                                                        egui::Label::new(egui::RichText::new(
                                                            format_game_units(item.valor_total),
                                                        )),
                                                    );

                                                    let status = if !item.preco_input.is_empty()
                                                        && parse_price_flag(&item.preco_input)
                                                            .is_err()
                                                    {
                                                        PriceStatus::Invalid
                                                    } else if item.valor_total > 0.0 {
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

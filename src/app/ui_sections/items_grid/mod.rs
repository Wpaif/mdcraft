use eframe::egui;

use crate::units::format_game_units;

use super::super::price::paint_price_status;
use super::MdcraftApp;
use super::capitalize_display_name;
use super::npc_price::{
    NpcPriceComparison, build_npc_price_lookup, compare_item_price_with_npc, npc_price_for_item,
    paint_npc_price_icon, price_input_fill_color, price_input_stroke, should_show_npc_price_icon,
};
use super::placeholder;

mod layout;
mod price_logic;
pub use price_logic::apply_item_price_from_input;
#[cfg(test)]
mod tests;

use layout::render_empty_item_cells;
use price_logic::{
    apply_item_price_if_changed, item_price_status, item_status_hover,
};

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
                ui.label(egui::RichText::new("Itens e Valores").strong().size(20.0));
                ui.add_space(10.0);

                let available_space_for_cols = (ui.available_width() - 10.0).max(300.0);
                let field_gap = 8.0;
                let indices_precificaveis: Vec<usize> = app
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(_, res)| !res.is_resource)
                    .map(|(i, _)| i)
                    .collect();
                let npc_lookup = build_npc_price_lookup(app);

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
                let max_columns =
                    (((available_space_for_cols + field_gap) / (min_col_width + field_gap)).floor()
                        as usize)
                        .max(1);

                let num_items = indices_precificaveis.len();
                let column_count = if num_items == 0 {
                    1
                } else {
                    max_columns.clamp(1, num_items)
                };
                let rows = num_items.div_ceil(column_count);

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
                                                    egui::RichText::new("Item").size(13.0).strong(),
                                                ),
                                            );
                                            ui.add_sized(
                                                [qty_w, 20.0],
                                                egui::Label::new(
                                                    egui::RichText::new("Qtd").size(13.0).strong(),
                                                ),
                                            );
                                            ui.add_sized(
                                                [price_w, 20.0],
                                                egui::Label::new(
                                                    egui::RichText::new("Preço").size(13.0).strong(),
                                                ),
                                            );
                                            ui.add_sized(
                                                [total_w, 20.0],
                                                egui::Label::new(
                                                    egui::RichText::new("Total").size(13.0).strong(),
                                                ),
                                            );
                                            ui.add_sized(
                                                [status_w, 20.0],
                                                egui::Label::new(
                                                    egui::RichText::new("Status").size(13.0).strong(),
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
                                                        egui::Label::new(item.quantidade.to_string()),
                                                    );

                                                    let stroke = price_input_stroke(ui, item, &npc_lookup);
                                                    let fill = price_input_fill_color(item);


                                                    let text_edit = egui::TextEdit::singleline(&mut item.preco_input)
                                                        .hint_text(placeholder(ui, "0"))
                                                        .frame(false)
                                                        .desired_width(price_w - 8.0);


                                                    let price_changed = ui
                                                        .allocate_ui_with_layout(
                                                            egui::vec2(price_w, 22.0),
                                                            egui::Layout::left_to_right(
                                                                egui::Align::Center,
                                                            ),
                                                            |ui| {
                                                                egui::Frame::NONE
                                                                    .fill(fill)
                                                                    .stroke(stroke)
                                                                    .corner_radius(
                                                                        egui::CornerRadius::same(4),
                                                                    )
                                                                    .inner_margin(
                                                                        egui::Margin::symmetric(4, 2),
                                                                    )
                                                                    .show(ui, |ui| {
                                                                        ui.add_sized(
                                                                            [price_w - 8.0, 20.0],
                                                                            text_edit,
                                                                        )
                                                                        .changed()
                                                                    })
                                                                    .inner
                                                            },
                                                        )
                                                        .inner;

                                                    // Filtro manual: só permite dígitos, vírgula, ponto, 'k'/'K' no final e apenas uma vez
                                                    if price_changed {
                                                        let mut filtered = String::new();
                                                        let mut k_count = 0;
                                                        let mut last_was_k = false;
                                                        for c in item.preco_input.chars() {
                                                            if c.is_ascii_digit() || c == ',' || c == '.' {
                                                                if k_count == 0 {
                                                                    filtered.push(c);
                                                                }
                                                            } else if c == 'k' || c == 'K' {
                                                                if k_count < 2 && !filtered.is_empty() {
                                                                    filtered.push('k');
                                                                    k_count += 1;
                                                                    last_was_k = true;
                                                                } else {
                                                                    break;
                                                                }
                                                            } else {
                                                                break;
                                                            }
                                                        }
                                                        // Só permite 'k' ou 'kk' no final
                                                        if k_count > 0 {
                                                            // Remove qualquer coisa após o(s) 'k'
                                                            let pos = filtered.find('k').unwrap();
                                                            filtered.truncate(pos + k_count);
                                                        }
                                                        item.preco_input = filtered;
                                                    }

                                                    apply_item_price_if_changed(item, price_changed);

                                                    ui.add_sized(
                                                        [total_w, 22.0],
                                                        egui::Label::new(egui::RichText::new(
                                                            format_game_units(item.valor_total),
                                                        )),
                                                    );

                                                    let status = item_price_status(item);
                                                    let hover = item_status_hover(status);
                                                    let npc_price = npc_price_for_item(item, &npc_lookup);
                                                    let npc_equal = matches!(
                                                        compare_item_price_with_npc(item, &npc_lookup),
                                                        Some(NpcPriceComparison::Equal)
                                                    );

                                                    ui.allocate_ui_with_layout(
                                                        egui::vec2(status_w, 22.0),
                                                        egui::Layout::left_to_right(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            if should_show_npc_price_icon(&item.nome) {
                                                                let npc_resp = paint_npc_price_icon(
                                                                    ui,
                                                                    npc_price.is_some(),
                                                                    npc_equal,
                                                                );
                                                                let npc_clicked = npc_resp.clicked();

                                                                if let Some(npc_value) = npc_price {
                                                                    let npc_text =
                                                                        format_game_units(npc_value);
                                                                    let hover_text = if npc_equal {
                                                                        format!(
                                                                            "Preço NPC aplicado ({npc_text}). Clique para reaplicar."
                                                                        )
                                                                    } else {
                                                                        format!(
                                                                            "Preço NPC: {npc_text}. Clique para usar no campo."
                                                                        )
                                                                    };
                                                                    npc_resp.on_hover_text(hover_text);

                                                                    if npc_clicked {
                                                                        item.preco_input = npc_text;
                                                                        apply_item_price_from_input(item);
                                                                    }
                                                                } else {
                                                                    npc_resp.on_hover_text(
                                                                        "Sem preço NPC para este item",
                                                                    );
                                                                }
                                                            }

                                                            let resp = paint_price_status(ui, status);
                                                            if let Some(text) = hover {
                                                                resp.on_hover_text(text);
                                                            }
                                                        },
                                                    );

                                                    *total_cost += item.valor_total;
                                                } else {
                                                    render_empty_item_cells(
                                                        ui, item_w, qty_w, price_w, total_w, status_w,
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

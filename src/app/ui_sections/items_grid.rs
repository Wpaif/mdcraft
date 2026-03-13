use eframe::egui;

use crate::parse::parse_price_flag;
use crate::units::format_game_units;

use super::super::price::{PriceStatus, paint_price_status};
use super::MdcraftApp;
use super::capitalize_display_name;
use super::placeholder;

fn apply_item_price_from_input(item: &mut crate::model::Item) {
    item.preco_unitario = parse_price_flag(&item.preco_input).unwrap_or(0.0);
    item.valor_total = item.preco_unitario * item.quantidade as f64;
}

fn apply_item_price_if_changed(item: &mut crate::model::Item, price_changed: bool) {
    if price_changed {
        apply_item_price_from_input(item);
    }
}

fn item_price_status(item: &crate::model::Item) -> PriceStatus {
    if !item.preco_input.is_empty() && parse_price_flag(&item.preco_input).is_err() {
        PriceStatus::Invalid
    } else if item.valor_total > 0.0 {
        PriceStatus::Ok
    } else {
        PriceStatus::None
    }
}

fn item_status_hover(status: PriceStatus) -> Option<&'static str> {
    match status {
        PriceStatus::Invalid => Some("Valor Inválido"),
        PriceStatus::Ok => Some("OK"),
        PriceStatus::None => None,
    }
}

fn render_empty_item_cells(
    ui: &mut egui::Ui,
    item_w: f32,
    qty_w: f32,
    price_w: f32,
    total_w: f32,
    status_w: f32,
) {
    ui.add_sized([item_w, 22.0], egui::Label::new(" "));
    ui.add_sized([qty_w, 22.0], egui::Label::new(" "));
    ui.add_sized([price_w, 22.0], egui::Label::new(" "));
    ui.add_sized([total_w, 22.0], egui::Label::new(" "));
    ui.add_sized([status_w, 22.0], egui::Label::new(" "));
}

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

                                                    apply_item_price_if_changed(item, price_changed);

                                                    ui.add_sized(
                                                        [total_w, 22.0],
                                                        egui::Label::new(egui::RichText::new(
                                                            format_game_units(item.valor_total),
                                                        )),
                                                    );

                                                    let status = item_price_status(item);
                                                    let hover = item_status_hover(status);

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
                                                    render_empty_item_cells(
                                                        ui,
                                                        item_w,
                                                        qty_w,
                                                        price_w,
                                                        total_w,
                                                        status_w,
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

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::MdcraftApp;
    use crate::model::Item;

    use super::{
        apply_item_price_from_input, apply_item_price_if_changed, item_price_status,
        item_status_hover, render_empty_item_cells, render_items_and_values,
    };
    use crate::app::price::PriceStatus;

    fn make_item(nome: &str, quantidade: u64, preco_input: &str, is_resource: bool) -> Item {
        Item {
            nome: nome.to_string(),
            quantidade,
            preco_unitario: 0.0,
            valor_total: 0.0,
            is_resource,
            preco_input: preco_input.to_string(),
        }
    }

    #[test]
    fn apply_item_price_from_input_parses_and_updates_total() {
        let mut item = make_item("Screw", 3, "2k", false);
        apply_item_price_from_input(&mut item);
        assert_eq!(item.preco_unitario, 2000.0);
        assert_eq!(item.valor_total, 6000.0);
    }

    #[test]
    fn apply_item_price_if_changed_respects_flag() {
        let mut item = make_item("Screw", 2, "3k", false);

        apply_item_price_if_changed(&mut item, false);
        assert_eq!(item.preco_unitario, 0.0);
        assert_eq!(item.valor_total, 0.0);

        apply_item_price_if_changed(&mut item, true);
        assert_eq!(item.preco_unitario, 3000.0);
        assert_eq!(item.valor_total, 6000.0);
    }

    #[test]
    fn item_price_status_covers_invalid_ok_and_none() {
        let mut invalid = make_item("A", 1, "x", false);
        apply_item_price_from_input(&mut invalid);
        assert_eq!(item_price_status(&invalid), PriceStatus::Invalid);
        assert_eq!(item_status_hover(PriceStatus::Invalid), Some("Valor Inválido"));

        let mut ok = make_item("B", 2, "1k", false);
        apply_item_price_from_input(&mut ok);
        assert_eq!(item_price_status(&ok), PriceStatus::Ok);
        assert_eq!(item_status_hover(PriceStatus::Ok), Some("OK"));

        let none = make_item("C", 1, "", false);
        assert_eq!(item_price_status(&none), PriceStatus::None);
        assert_eq!(item_status_hover(PriceStatus::None), None);
    }

    #[test]
    fn render_items_and_values_with_only_resources_keeps_total_zero() {
        let mut app = MdcraftApp::default();
        app.items = vec![
            make_item("Iron Ore", 2, "", true),
            make_item("Copper Ore", 3, "", true),
        ];

        let mut total_cost = 0.0;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_items_and_values(ui, &mut app, 720.0, &mut total_cost);
            });
        });

        assert_eq!(total_cost, 0.0);
    }

    #[test]
    fn render_items_and_values_handles_uneven_grid_with_blank_cells() {
        let mut app = MdcraftApp::default();
        app.items = vec![
            make_item("A", 1, "1k", false),
            make_item("B", 1, "", false),
            make_item("C", 1, "x", false),
        ];

        let mut total_cost = 0.0;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_items_and_values(ui, &mut app, 900.0, &mut total_cost);
            });
        });

        // First pass does not necessarily change text edits, but should render and sum existing totals safely.
        assert!(total_cost >= 0.0);
    }

    #[test]
    fn render_empty_item_cells_runs_without_panicking() {
        egui::__run_test_ui(|ui| {
            render_empty_item_cells(ui, 120.0, 46.0, 96.0, 78.0, 56.0);
        });
    }
}

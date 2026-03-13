use eframe::egui;
use std::collections::HashMap;

use crate::app::fixed_npc_price_input;
use crate::parse::parse_price_flag;
use crate::units::format_game_units;

use super::super::price::{PriceStatus, paint_price_status};
use super::MdcraftApp;
use super::autosave_active_craft;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NpcPriceComparison {
    Equal,
    HigherThanNpc,
    LowerThanNpc,
}

fn build_npc_price_lookup(app: &MdcraftApp) -> HashMap<String, f64> {
    let mut lookup = HashMap::new();

    for entry in &app.wiki_cached_items {
        let Some(raw_price) = &entry.npc_price else {
            continue;
        };

        let Ok(parsed) = parse_price_flag(raw_price) else {
            continue;
        };

        lookup.insert(entry.name.trim().to_lowercase(), parsed);
    }

    for fixed_name in ["Compressed Nightmare Gems", "Neutral Essence"] {
        if let Some(raw_price) = fixed_npc_price_input(fixed_name)
            && let Ok(parsed) = parse_price_flag(raw_price)
        {
            lookup.insert(fixed_name.trim().to_lowercase(), parsed);
        }
    }

    lookup
}

fn compare_item_price_with_npc(
    item: &crate::model::Item,
    npc_lookup: &HashMap<String, f64>,
) -> Option<NpcPriceComparison> {
    let entered = parse_price_flag(&item.preco_input).ok()?;
    let npc_price = npc_lookup.get(&item.nome.trim().to_lowercase()).copied()?;

    let eps = 1e-9;
    if (entered - npc_price).abs() < eps {
        Some(NpcPriceComparison::Equal)
    } else if entered > npc_price {
        Some(NpcPriceComparison::HigherThanNpc)
    } else {
        Some(NpcPriceComparison::LowerThanNpc)
    }
}

fn price_input_stroke(
    ui: &egui::Ui,
    item: &crate::model::Item,
    npc_lookup: &HashMap<String, f64>,
) -> egui::Stroke {
    if item.preco_input.trim().is_empty() {
        // Missing price gets a warm border to draw attention without looking like an error.
        return egui::Stroke::new(1.4, egui::Color32::from_rgb(235, 188, 90));
    }

    let default = ui.visuals().widgets.inactive.bg_stroke;

    match compare_item_price_with_npc(item, npc_lookup) {
        Some(NpcPriceComparison::HigherThanNpc) => {
            egui::Stroke::new(1.4, egui::Color32::from_rgb(74, 201, 126))
        }
        Some(NpcPriceComparison::LowerThanNpc) => {
            egui::Stroke::new(1.4, egui::Color32::from_rgb(220, 98, 98))
        }
        _ => default,
    }
}

fn price_input_fill_color(item: &crate::model::Item) -> egui::Color32 {
    if item.preco_input.trim().is_empty() {
        egui::Color32::from_rgba_unmultiplied(235, 188, 90, 22)
    } else {
        egui::Color32::TRANSPARENT
    }
}

fn npc_price_for_item(item: &crate::model::Item, npc_lookup: &HashMap<String, f64>) -> Option<f64> {
    npc_lookup.get(&item.nome.trim().to_lowercase()).copied()
}

fn should_show_npc_price_icon(item_name: &str) -> bool {
    !item_name.trim().eq_ignore_ascii_case("diamond")
}

fn paint_npc_price_icon(
    ui: &mut egui::Ui,
    has_npc_price: bool,
    is_equal_to_npc: bool,
) -> egui::Response {
    let size = egui::vec2(18.0, 18.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let painter = ui.painter();
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.45;

    let (fill, stroke, text_color) = if !has_npc_price {
        (
            egui::Color32::from_rgba_unmultiplied(120, 120, 120, 24),
            egui::Stroke::new(1.0, egui::Color32::from_rgb(130, 130, 130)),
            egui::Color32::from_rgb(130, 130, 130),
        )
    } else if is_equal_to_npc {
        (
            egui::Color32::from_rgb(29, 155, 240),
            egui::Stroke::new(1.2, egui::Color32::from_rgb(180, 225, 255)),
            egui::Color32::WHITE,
        )
    } else {
        (
            egui::Color32::from_rgba_unmultiplied(29, 155, 240, 36),
            egui::Stroke::new(1.2, egui::Color32::from_rgb(29, 155, 240)),
            egui::Color32::from_rgb(29, 155, 240),
        )
    };

    painter.circle(center, radius, fill, stroke);
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        "N",
        egui::FontId::proportional(10.0),
        text_color,
    );

    response
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
                                        let mut should_autosave_prices = false;
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
                                                        egui::Label::new(
                                                            item.quantidade.to_string(),
                                                        ),
                                                    );

                                                    let stroke = price_input_stroke(
                                                        ui,
                                                        item,
                                                        &npc_lookup,
                                                    );
                                                    let fill = price_input_fill_color(item);

                                                    let text_edit = egui::TextEdit::singleline(
                                                        &mut item.preco_input,
                                                    )
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
                                                                        egui::CornerRadius::same(
                                                                            4,
                                                                        ),
                                                                    )
                                                                    .inner_margin(
                                                                        egui::Margin::symmetric(
                                                                            4,
                                                                            2,
                                                                        ),
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

                                                    apply_item_price_if_changed(item, price_changed);
                                                    if price_changed {
                                                        should_autosave_prices = true;
                                                    }

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
                                                        compare_item_price_with_npc(
                                                            item,
                                                            &npc_lookup,
                                                        ),
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
                                                                    let npc_text = format_game_units(
                                                                        npc_value,
                                                                    );
                                                                    let hover_text = if npc_equal {
                                                                        format!(
                                                                            "Preco NPC aplicado ({npc_text}). Clique para reaplicar."
                                                                        )
                                                                    } else {
                                                                        format!(
                                                                            "Preco NPC: {npc_text}. Clique para usar no campo."
                                                                        )
                                                                    };
                                                                    npc_resp.on_hover_text(
                                                                        hover_text,
                                                                    );

                                                                    if npc_clicked {
                                                                        item.preco_input = npc_text;
                                                                        apply_item_price_from_input(
                                                                            item,
                                                                        );
                                                                        should_autosave_prices = true;
                                                                    }
                                                                } else {
                                                                    npc_resp.on_hover_text(
                                                                        "Sem preco NPC para este item",
                                                                    );
                                                                }
                                                            }

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

                                        if should_autosave_prices {
                                            autosave_active_craft(app);
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
    use std::collections::HashMap;

    use crate::app::MdcraftApp;
    use crate::model::Item;

    use super::{
        NpcPriceComparison, apply_item_price_from_input, apply_item_price_if_changed,
        build_npc_price_lookup, compare_item_price_with_npc, item_price_status, item_status_hover,
        npc_price_for_item, price_input_fill_color, price_input_stroke, render_empty_item_cells,
        render_items_and_values, should_show_npc_price_icon,
    };
    use crate::app::price::PriceStatus;
    use crate::data::wiki_scraper::{ScrapedItem, WikiSource};

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
        assert_eq!(
            item_status_hover(PriceStatus::Invalid),
            Some("Valor Inválido")
        );

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

    #[test]
    fn compare_item_price_with_npc_covers_equal_cheaper_and_expensive() {
        let mut app = MdcraftApp::default();
        app.wiki_cached_items.push(ScrapedItem {
            name: "Screw".to_string(),
            npc_price: Some("1k".to_string()),
            sources: vec![WikiSource::Loot],
        });

        let lookup = build_npc_price_lookup(&app);

        let equal = make_item("Screw", 1, "1k", false);
        assert_eq!(
            compare_item_price_with_npc(&equal, &lookup),
            Some(NpcPriceComparison::Equal)
        );

        let cheaper = make_item("Screw", 1, "800", false);
        assert_eq!(
            compare_item_price_with_npc(&cheaper, &lookup),
            Some(NpcPriceComparison::LowerThanNpc)
        );

        let expensive = make_item("Screw", 1, "2k", false);
        assert_eq!(
            compare_item_price_with_npc(&expensive, &lookup),
            Some(NpcPriceComparison::HigherThanNpc)
        );
    }

    #[test]
    fn build_npc_price_lookup_includes_fixed_compressed_nightmare_gems() {
        let app = MdcraftApp::default();
        let lookup = build_npc_price_lookup(&app);
        assert_eq!(
            lookup.get("compressed nightmare gems").copied(),
            Some(25_000.0)
        );
    }

    #[test]
    fn build_npc_price_lookup_includes_fixed_neutral_essence() {
        let app = MdcraftApp::default();
        let lookup = build_npc_price_lookup(&app);
        assert_eq!(lookup.get("neutral essence").copied(), Some(1000.0));
    }

    #[test]
    fn price_input_stroke_highlights_missing_value() {
        egui::__run_test_ui(|ui| {
            let item = make_item("Screw", 1, "", false);
            let lookup = HashMap::new();
            let stroke = price_input_stroke(ui, &item, &lookup);
            assert_eq!(
                stroke,
                egui::Stroke::new(1.4, egui::Color32::from_rgb(235, 188, 90))
            );
        });
    }

    #[test]
    fn price_input_fill_color_marks_only_missing_value() {
        let missing = make_item("Screw", 1, "", false);
        assert_eq!(
            price_input_fill_color(&missing),
            egui::Color32::from_rgba_unmultiplied(235, 188, 90, 22)
        );

        let present = make_item("Screw", 1, "1k", false);
        assert_eq!(price_input_fill_color(&present), egui::Color32::TRANSPARENT);
    }

    #[test]
    fn npc_price_for_item_returns_lookup_value() {
        let mut app = MdcraftApp::default();
        app.wiki_cached_items.push(ScrapedItem {
            name: "Screw".to_string(),
            npc_price: Some("1k".to_string()),
            sources: vec![WikiSource::Loot],
        });

        let lookup = build_npc_price_lookup(&app);
        let item = make_item("Screw", 1, "", false);
        assert_eq!(npc_price_for_item(&item, &lookup), Some(1000.0));
    }

    #[test]
    fn should_show_npc_price_icon_hides_only_for_diamond() {
        assert!(!should_show_npc_price_icon("Diamond"));
        assert!(!should_show_npc_price_icon(" diamond "));
        assert!(should_show_npc_price_icon("Screw"));
    }
}

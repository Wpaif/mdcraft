use eframe::egui;

use crate::model::Item;
use crate::parse::{parse_clipboard, parse_price_flag};
use crate::theme;
use crate::units::format_game_units;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Theme {
    Light,
    Dark,
}

impl Theme {
    fn visuals(self) -> egui::Visuals {
        match self {
            Theme::Light => theme::github_light(),
            Theme::Dark => theme::doki_dark(),
        }
    }

    fn toggle(self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }

}

pub struct MdcraftApp {
    input_text: String,
    items: Vec<Item>,
    sell_price_input: String,
    resource_list: Vec<String>,
    fonts_loaded: bool,
    theme: Theme,
}

impl Default for MdcraftApp {
    fn default() -> Self {
        Self {
            input_text: String::new(),
            items: Vec::new(),
            sell_price_input: String::new(),
            resource_list: vec![
                "tech data".to_string(),
                "iron ore".to_string(),
                "iron bar".to_string(),
                "platinum bar".to_string(),
                "platinum ore".to_string(),
                "pure grass".to_string(),
                "minor seed bag".to_string(),
                "condensed grass".to_string(),
                "nature energy".to_string(),
                "major seed bag".to_string(),
                "pure strong grass".to_string(),
                "condensed strong grass".to_string(),
                "strong nature energy".to_string(),
                "darkrai essence".to_string(),
                "dew becker".to_string(),
                "study note".to_string(),
                "log".to_string(),
                "style point".to_string(),
                "refined style point".to_string(),
                "planks".to_string(),
                "refined fashion point".to_string(),
                "oak planks".to_string(),
                "fashion point".to_string(),
                "purpleheart log".to_string(),
                "nightmare style point".to_string(),
                "drawing clipboard".to_string(),
                "Gold Coins".to_string(),
                "Gold Bar".to_string(),
                "Cooking Token".to_string(),
                "Hidden Relic".to_string(),
                "Corrupted Gold Bar".to_string(),
                "Food Bag".to_string(),
                "Strange Gold Bar".to_string(),
            ],
            fonts_loaded: false,
            theme: Theme::Dark,
        }
    }
}

fn setup_custom_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(26.0));
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(16.0));

    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(8.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(20);
    style.spacing.interact_size = egui::vec2(40.0, 24.0);
    style.spacing.text_edit_width = 150.0;

    ctx.set_style(style);
}

fn setup_emoji_support(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "emoji".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/NotoEmoji-Regular.ttf")).into(),
    );

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .push("emoji".to_owned());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("emoji".to_owned());

    ctx.set_fonts(fonts);
}

fn paint_theme_icon(p: &egui::Painter, rect: egui::Rect, theme: Theme, color: egui::Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.26;

    match theme {
        Theme::Light => {
            // Sun: filled circle + rays
            p.circle_filled(center, r, color);
            let ray_color = color;
            let ray_len = r * 1.9;
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let dir = egui::vec2(a.cos(), a.sin());
                let a0 = center + dir * (r * 1.2);
                let a1 = center + dir * (ray_len * 0.65);
                p.line_segment([a0, a1], egui::Stroke::new(1.5, ray_color));
            }
        }
        Theme::Dark => {
            // Moon: circle - offset circle (crescent)
            p.circle_filled(center, r, color);
            let cut = center + egui::vec2(r * 0.55, -r * 0.15);
            p.circle_filled(cut, r * 0.95, egui::Color32::TRANSPARENT);
            // "cut" with background color by painting a second circle with the button fill later
            // (caller paints it using the same fill color).
        }
    }
}

fn theme_toggle_button(ui: &mut egui::Ui, theme: Theme) -> egui::Response {
    let size = egui::vec2(36.0, 36.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let rounding = egui::CornerRadius::same(18);
        let fill = visuals.bg_fill;
        let stroke = visuals.bg_stroke;

        let p = ui.painter();
        p.rect(rect, rounding, fill, stroke, egui::StrokeKind::Inside);

        // Icon color: use fg stroke color for maximum contrast.
        let icon_color = visuals.fg_stroke.color;
        let icon_rect = rect.shrink(6.0);

        match theme {
            Theme::Light => {
                paint_theme_icon(p, icon_rect, Theme::Light, icon_color);
            }
            Theme::Dark => {
                // Moon crescent: paint moon, then "cut" with fill color.
                let center = icon_rect.center();
                let r = icon_rect.width().min(icon_rect.height()) * 0.26;
                p.circle_filled(center, r, icon_color);
                let cut = center + egui::vec2(r * 0.55, -r * 0.15);
                p.circle_filled(cut, r * 0.95, fill);
            }
        }
    }

    response
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PriceStatus {
    None,
    Ok,
    Invalid,
}

fn paint_price_status(ui: &mut egui::Ui, status: PriceStatus) -> egui::Response {
    let size = egui::vec2(18.0, 18.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());

    if !ui.is_rect_visible(rect) || status == PriceStatus::None {
        return response;
    }

    let p = ui.painter();
    match status {
        PriceStatus::Ok => {
            let center = rect.center();
            let r = rect.width().min(rect.height()) * 0.45;
            p.circle_filled(center, r, egui::Color32::from_rgb(26, 127, 55));

            // Check mark (two segments).
            let a = center + egui::vec2(-r * 0.45, r * 0.05);
            let b = center + egui::vec2(-r * 0.10, r * 0.38);
            let c = center + egui::vec2(r * 0.55, -r * 0.35);
            p.line_segment([a, b], egui::Stroke::new(2.0, egui::Color32::WHITE));
            p.line_segment([b, c], egui::Stroke::new(2.0, egui::Color32::WHITE));
        }
        PriceStatus::Invalid => {
            let center = rect.center();
            let r = rect.width().min(rect.height()) * 0.5;

            // Triangle warning sign.
            let top = center + egui::vec2(0.0, -r * 0.95);
            let bl = center + egui::vec2(-r * 0.9, r * 0.85);
            let br = center + egui::vec2(r * 0.9, r * 0.85);
            let yellow = egui::Color32::from_rgb(241, 250, 140);
            let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(154, 103, 0));
            p.add(egui::Shape::convex_polygon(vec![top, br, bl], yellow, stroke));

            // Exclamation mark.
            let bar_top = center + egui::vec2(0.0, -r * 0.35);
            let bar_bot = center + egui::vec2(0.0, r * 0.25);
            p.line_segment(
                [bar_top, bar_bot],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(36, 41, 47)),
            );
            p.circle_filled(
                center + egui::vec2(0.0, r * 0.48),
                1.6,
                egui::Color32::from_rgb(36, 41, 47),
            );
        }
        PriceStatus::None => {}
    }

    response
}

impl eframe::App for MdcraftApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.fonts_loaded {
            setup_custom_styles(ctx);
            setup_emoji_support(ctx);
            ctx.set_visuals(self.theme.visuals());
            self.fonts_loaded = true;
        }

        // Botão flutuante no canto superior direito (não interfere no layout/usabilidade).
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

                            let available_space_for_cols = ui.available_width();
                            let min_col_width = 380.0;
                            let max_columns =
                                (available_space_for_cols / min_col_width).floor() as usize;

                            let indices_precificaveis: Vec<usize> = self
                                .items
                                .iter()
                                .enumerate()
                                .filter(|(_, res)| !res.is_resource)
                                .map(|(i, _)| i)
                                .collect();

                            let num_items = indices_precificaveis.len();
                            let column_count = if num_items == 0 {
                                1
                            } else {
                                max_columns.clamp(1, num_items)
                            };

                            let items_per_col =
                                (num_items as f32 / column_count as f32).ceil() as usize;

                            egui::ScrollArea::vertical()
                                .max_height(350.0)
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    ui.horizontal_top(|ui| {
                                        for col_idx in 0..column_count {
                                            let start_idx = col_idx * items_per_col;
                                            let end_idx = ((col_idx + 1) * items_per_col)
                                                .min(indices_precificaveis.len());

                                            if start_idx >= indices_precificaveis.len() {
                                                break;
                                            }

                                            let col_width = (available_space_for_cols
                                                / column_count as f32)
                                                - 20.0;

                                            ui.allocate_ui_with_layout(
                                                egui::vec2(col_width, ui.available_height()),
                                                egui::Layout::top_down(egui::Align::Min),
                                                |ui| {
                                                    egui::Grid::new(format!(
                                                        "items_grid_{}",
                                                        col_idx
                                                    ))
                                                    .num_columns(5)
                                                    .spacing([15.0, 15.0])
                                                    .striped(true)
                                                    .show(ui, |ui| {
                                                        ui.heading(
                                                            egui::RichText::new("Item").size(14.0),
                                                        );
                                                        ui.heading(
                                                            egui::RichText::new("Qtd").size(14.0),
                                                        );
                                                        ui.heading(
                                                            egui::RichText::new("Preço Unit.")
                                                                .size(14.0),
                                                        );
                                                        ui.heading(
                                                            egui::RichText::new("Total").size(14.0),
                                                        );
                                                        ui.heading(
                                                            egui::RichText::new("Status")
                                                                .size(14.0),
                                                        );
                                                        ui.end_row();

                                                        for idx_loop in start_idx..end_idx {
                                                            let real_idx =
                                                                indices_precificaveis[idx_loop];
                                                            let item = &mut self.items[real_idx];

                                                            let nome_truncado = if item.nome.len()
                                                                > 25
                                                            {
                                                                format!("{}...", &item.nome[..22])
                                                            } else {
                                                                item.nome.clone()
                                                            };

                                                            ui.label(
                                                                egui::RichText::new(nome_truncado)
                                                                    .strong(),
                                                            )
                                                            .on_hover_text(&item.nome);

                                                            ui.label(item.quantidade.to_string());

                                                            let text_edit =
                                                                egui::TextEdit::singleline(
                                                                    &mut item.preco_input,
                                                                )
                                                                .desired_width(150.0)
                                                                .margin(egui::vec2(12.0, 10.0));

                                                            if ui.add(text_edit).changed() {
                                                                item.preco_unitario =
                                                                    parse_price_flag(
                                                                        &item.preco_input,
                                                                    )
                                                                    .unwrap_or(0);

                                                                item.valor_total = item
                                                                    .preco_unitario
                                                                    * item.quantidade;
                                                            }

                                                            ui.label(egui::RichText::new(
                                                                format_game_units(
                                                                    item.valor_total as f64,
                                                                ),
                                                            ));

                                                            let status = if !item.preco_input
                                                                .is_empty()
                                                                && parse_price_flag(
                                                                    &item.preco_input,
                                                                )
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

                                                            let resp =
                                                                paint_price_status(ui, status);
                                                            if let Some(text) = hover {
                                                                resp.on_hover_text(text);
                                                            }

                                                            total_cost += item.valor_total;
                                                            ui.end_row();
                                                        }
                                                    });
                                                },
                                            );

                                            if col_idx < column_count - 1 {
                                                ui.add_space(10.0);
                                                ui.separator();
                                                ui.add_space(10.0);
                                            }
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
                                let color =
                                    if is_profit { egui::Color32::GREEN } else { egui::Color32::RED };

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
                                                        let custo_por_ponto = lucro_total as f64
                                                            / *res_qtd as f64;

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


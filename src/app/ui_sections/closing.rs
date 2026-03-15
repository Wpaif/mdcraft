use eframe::egui;

use crate::parse::parse_price_flag;
use crate::units::format_game_units;

use super::{MdcraftApp, placeholder};

pub(crate) fn render_closing(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
    content_width: f32,
    total_cost: f64,
    found_resources: &[(String, u64)],
) {
    ui.group(|ui| {
        ui.set_width(content_width);
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(5))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Fechamento").strong().size(20.0));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_sized([150.0, 32.0], egui::Label::new(egui::RichText::new("Preço de Venda Final:").size(14.0)));
                    let sell_resp = ui.add(
                        egui::TextEdit::singleline(&mut app.sell_price_input)
                            .hint_text(placeholder(ui, "100k"))
                            .desired_width(180.0)
                            .margin(egui::vec2(12.0, 10.0)),
                    );
                    // Filtro manual: só permite dígitos, vírgula, ponto, 'k'/'kk' no final
                    if sell_resp.changed() {
                        let mut filtered = String::new();
                        let mut k_count = 0;
                        for c in app.sell_price_input.chars() {
                            if c.is_ascii_digit() || c == ',' || c == '.' {
                                if k_count == 0 {
                                    filtered.push(c);
                                }
                            } else if c == 'k' || c == 'K' {
                                if k_count < 2 && !filtered.is_empty() {
                                    filtered.push('k');
                                    k_count += 1;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        // Só permite 'k' ou 'kk' no final
                        if k_count > 0 {
                            let pos = filtered.find('k').unwrap();
                            filtered.truncate(pos + k_count);
                        }
                        app.sell_price_input = filtered;
                    }

                });

                ui.add_space(15.0);

                let caption = |text: &str, ui: &egui::Ui| -> egui::RichText {
                    egui::RichText::new(text)
                        .size(11.0)
                        .strong()
                        .color(ui.visuals().weak_text_color())
                };

                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.label(caption("CUSTO TOTAL", ui));
                        ui.label(egui::RichText::new(format_game_units(total_cost)).strong().size(22.0));
                    });

                    ui.add_space(40.0);

                    let sell_price = parse_price_flag(&app.sell_price_input).unwrap_or(0.0);
                    if sell_price > 0.0 {
                        let lucro_total = sell_price - total_cost;
                        let is_profit = lucro_total >= 0.0;
                        let color = if is_profit {
                            ui.visuals().widgets.active.bg_stroke.color
                        } else {
                            ui.visuals().error_fg_color
                        };

                        ui.vertical(|ui| {
                            ui.label(caption("RECEITA TOTAL", ui));
                            ui.label(egui::RichText::new(format_game_units(sell_price)).strong().size(22.0));
                        });

                        ui.add_space(40.0);

                        ui.vertical(|ui| {
                            ui.label(caption("LUCRO LÍQUIDO", ui));
                            ui.label(
                                egui::RichText::new(format_game_units(lucro_total)).strong().size(22.0).color(color),
                            );
                        });

                        ui.add_space(40.0);

                        ui.vertical(|ui| {
                            let margem = if total_cost > 0.0 {
                                lucro_total / total_cost * 100.0
                            } else {
                                0.0
                            };

                            ui.label(
                                egui::RichText::new(format!("MARGEM: {:.1}%", margem))
                                    .size(13.0)
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
                                                            lucro_total / *res_qtd as f64;

                                                        ui.label(format!("{} {}", res_qtd, res_name));
                                                        ui.label("-");
                                                        ui.label(format!("{:.1} por pt", custo_por_ponto));
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

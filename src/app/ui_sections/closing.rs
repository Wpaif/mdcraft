use eframe::egui;

use crate::parse::parse_price_flag;
use crate::units::format_game_units;

use super::{MdcraftApp, autosave_active_craft, placeholder};

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
                ui.label(egui::RichText::new("Fechamento").strong().size(16.0));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_sized([150.0, 32.0], egui::Label::new("Preço de Venda Final:"));
                    let sell_resp = ui.add(
                        egui::TextEdit::singleline(&mut app.sell_price_input)
                            .hint_text(placeholder(ui, "100k"))
                            .desired_width(180.0)
                            .margin(egui::vec2(12.0, 10.0)),
                    );

                    if sell_resp.changed() {
                        autosave_active_craft(app);
                    }
                });

                ui.add_space(15.0);

                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.label("CUSTO TOTAL");
                        ui.heading(egui::RichText::new(format_game_units(total_cost)));
                    });

                    ui.add_space(40.0);

                    let sell_price = parse_price_flag(&app.sell_price_input).unwrap_or(0.0);
                    if sell_price > 0.0 {
                        let lucro_total = sell_price - total_cost;
                        let is_profit = lucro_total >= 0.0;
                        let color = if is_profit {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        };

                        ui.vertical(|ui| {
                            ui.label("RECEITA TOTAL");
                            ui.heading(egui::RichText::new(format_game_units(sell_price)));
                        });

                        ui.add_space(40.0);

                        ui.vertical(|ui| {
                            ui.label("LUCRO LÍQUIDO");
                            ui.heading(
                                egui::RichText::new(format_game_units(lucro_total)).color(color),
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

use eframe::egui;

use crate::parse::parse_clipboard;

use super::{MdcraftApp, autosave_active_craft, placeholder};

pub(crate) fn render_craft_input(ui: &mut egui::Ui, app: &mut MdcraftApp, content_width: f32) {
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
                            new_item.valor_total =
                                new_item.preco_unitario * new_item.quantidade as f64;
                        }
                    }

                    app.items = new_items;
                    autosave_active_craft(app);
                }
            });
    });
}

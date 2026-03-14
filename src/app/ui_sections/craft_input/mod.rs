use eframe::egui;

use super::{MdcraftApp, placeholder};

mod logic;
#[cfg(test)]
mod tests;

use logic::apply_input_change;

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

                apply_input_change(app, response.changed());
            });
    });
}

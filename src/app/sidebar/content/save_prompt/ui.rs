use eframe::egui;

use crate::app::MdcraftApp;

use super::logic::commit_new_saved_craft;

pub(in crate::app::sidebar::content) fn render_save_name_prompt(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
    content_w: f32,
) {
    if !app.awaiting_craft_name {
        return;
    }

    ui.add_space(10.0);
    ui.label(egui::RichText::new("Nome da receita:").strong());

    let mut name_resp_opt: Option<egui::Response> = None;
    let accent = ui.visuals().hyperlink_color;
    egui::Frame::NONE
        .fill(egui::Color32::from_rgba_unmultiplied(
            accent.r(),
            accent.g(),
            accent.b(),
            18,
        ))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(
                accent.r(),
                accent.g(),
                accent.b(),
                96,
            ),
        ))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(6, 4))
        .show(ui, |ui| {
            let input_width = (content_w - 12.0).max(80.0);
            let name_resp = ui.add_sized(
                [input_width, 30.0],
                egui::TextEdit::singleline(&mut app.pending_craft_name)
                    .font(egui::TextStyle::Button)
                    .horizontal_align(egui::Align::Center)
                    .vertical_align(egui::Align::Center)
                    .hint_text("Nome da receita"),
            );
            if name_resp.changed() {
                app.pending_craft_name = app
                    .pending_craft_name
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '(' || *c == ')')
                    .collect();
            }
            name_resp_opt = Some(name_resp);
        });

    let name_resp = name_resp_opt.expect("name input response should exist");

    if app.focus_craft_name_input {
        name_resp.request_focus();
        app.focus_craft_name_input = false;
    }

    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.awaiting_craft_name = false;
        app.pending_craft_name.clear();
        app.focus_craft_name_input = false;
    }

    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        commit_new_saved_craft(app);
    }
}

use eframe::egui;

use crate::app::MdcraftApp;

use super::delete_logic::{handle_cancel_delete_click, handle_confirm_delete_click};

pub(super) fn render_delete_confirmation_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
    let Some(idx) = app.pending_delete_index else {
        return;
    };

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.pending_delete_index = None;
        return;
    }

    if idx >= app.saved_crafts.len() {
        app.pending_delete_index = None;
        return;
    }

    let recipe_name = app.saved_crafts[idx].name.clone();

    egui::Window::new("Confirmar Exclusão")
        .id(egui::Id::new("confirm_delete_saved_recipe"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(400.0, 190.0))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Deseja realmente apagar esta receita?")
                        .strong()
                        .size(16.0),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("'{}'", recipe_name))
                        .weak()
                        .size(14.0),
                );
            });

            ui.add_space(12.0);
            let cancel_fill = ui.visuals().widgets.inactive.bg_fill;
            let delete_fill = egui::Color32::from_rgb(181, 61, 61);
            let row_height = 32.0;
            let button_width = 120.0;
            let spacing = ui.spacing().item_spacing.x;
            let total_buttons_width = (button_width * 2.0) + spacing;
            let left_pad = ((ui.available_width() - total_buttons_width) * 0.5).max(0.0);

            ui.horizontal(|ui| {
                ui.add_space(left_pad);

                let cancel_clicked = ui
                    .add_sized(
                        [button_width, row_height],
                        egui::Button::new("Cancelar").fill(cancel_fill),
                    )
                    .clicked();
                handle_cancel_delete_click(app, cancel_clicked);

                let delete_clicked = ui
                    .add_sized(
                        [button_width, row_height],
                        egui::Button::new(
                            egui::RichText::new("Apagar")
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(delete_fill),
                    )
                    .clicked();
                handle_confirm_delete_click(app, idx, delete_clicked);
            });
        });
}

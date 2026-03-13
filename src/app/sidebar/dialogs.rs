use eframe::egui;

use crate::app::MdcraftApp;

pub(super) fn render_delete_confirmation_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
    let Some(idx) = app.pending_delete_index else {
        return;
    };

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

                if ui
                    .add_sized(
                        [button_width, row_height],
                        egui::Button::new("Cancelar").fill(cancel_fill),
                    )
                    .clicked()
                {
                    app.pending_delete_index = None;
                }

                if ui
                    .add_sized(
                        [button_width, row_height],
                        egui::Button::new(
                            egui::RichText::new("Apagar")
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(delete_fill),
                    )
                    .clicked()
                {
                    app.saved_crafts.remove(idx);

                    if let Some(active_idx) = app.active_saved_craft_index {
                        app.active_saved_craft_index = if active_idx == idx {
                            None
                        } else if active_idx > idx {
                            Some(active_idx - 1)
                        } else {
                            Some(active_idx)
                        };
                    }

                    app.pending_delete_index = None;
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::{MdcraftApp, SavedCraft};

    use super::render_delete_confirmation_popup;

    #[test]
    fn delete_popup_returns_early_when_no_pending_index() {
        let mut app = MdcraftApp::default();
        egui::__run_test_ctx(|ctx| {
            render_delete_confirmation_popup(ctx, &mut app);
        });
        assert_eq!(app.pending_delete_index, None);
    }

    #[test]
    fn delete_popup_clears_invalid_pending_index() {
        let mut app = MdcraftApp::default();
        app.pending_delete_index = Some(0);

        egui::__run_test_ctx(|ctx| {
            render_delete_confirmation_popup(ctx, &mut app);
        });

        assert_eq!(app.pending_delete_index, None);
    }

    #[test]
    fn delete_popup_keeps_state_when_waiting_user_action() {
        let mut app = MdcraftApp::default();
        app.saved_crafts.push(SavedCraft {
            name: "Receita X".to_string(),
            recipe_text: "1 Iron Ore".to_string(),
            sell_price_input: "2k".to_string(),
        });
        app.pending_delete_index = Some(0);

        egui::__run_test_ctx(|ctx| {
            render_delete_confirmation_popup(ctx, &mut app);
        });

        assert_eq!(app.saved_crafts.len(), 1);
        assert_eq!(app.pending_delete_index, Some(0));
    }
}

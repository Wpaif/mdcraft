use eframe::egui;

use crate::app::MdcraftApp;

fn apply_delete_recipe(app: &mut MdcraftApp, idx: usize) {
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

fn handle_cancel_delete_click(app: &mut MdcraftApp, clicked: bool) {
    if clicked {
        app.pending_delete_index = None;
    }
}

fn handle_confirm_delete_click(app: &mut MdcraftApp, idx: usize, clicked: bool) {
    if clicked {
        apply_delete_recipe(app, idx);
    }
}

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

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::{MdcraftApp, SavedCraft};

    use super::{
        apply_delete_recipe, handle_cancel_delete_click, handle_confirm_delete_click,
        render_delete_confirmation_popup,
    };

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
            item_prices: vec![],
        });
        app.pending_delete_index = Some(0);

        egui::__run_test_ctx(|ctx| {
            render_delete_confirmation_popup(ctx, &mut app);
        });

        assert_eq!(app.saved_crafts.len(), 1);
        assert_eq!(app.pending_delete_index, Some(0));
    }

    #[test]
    fn delete_popup_closes_on_escape() {
        let mut app = MdcraftApp::default();
        app.saved_crafts.push(SavedCraft {
            name: "Receita X".to_string(),
            recipe_text: "1 Iron Ore".to_string(),
            sell_price_input: "2k".to_string(),
            item_prices: vec![],
        });
        app.pending_delete_index = Some(0);

        let ctx = egui::Context::default();
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });

        let _ = ctx.run(input, |ctx| {
            render_delete_confirmation_popup(ctx, &mut app);
        });

        assert_eq!(app.pending_delete_index, None);
        assert_eq!(app.saved_crafts.len(), 1);
    }

    #[test]
    fn apply_delete_recipe_clears_active_when_deleting_active_item() {
        let mut app = MdcraftApp::default();
        app.saved_crafts.push(SavedCraft {
            name: "A".to_string(),
            recipe_text: String::new(),
            sell_price_input: String::new(),
            item_prices: vec![],
        });
        app.active_saved_craft_index = Some(0);
        app.pending_delete_index = Some(0);

        apply_delete_recipe(&mut app, 0);

        assert!(app.saved_crafts.is_empty());
        assert_eq!(app.active_saved_craft_index, None);
        assert_eq!(app.pending_delete_index, None);
    }

    #[test]
    fn apply_delete_recipe_shifts_active_index_when_needed() {
        let mut app = MdcraftApp::default();
        app.saved_crafts.push(SavedCraft {
            name: "A".to_string(),
            recipe_text: String::new(),
            sell_price_input: String::new(),
            item_prices: vec![],
        });
        app.saved_crafts.push(SavedCraft {
            name: "B".to_string(),
            recipe_text: String::new(),
            sell_price_input: String::new(),
            item_prices: vec![],
        });
        app.active_saved_craft_index = Some(1);
        app.pending_delete_index = Some(0);

        apply_delete_recipe(&mut app, 0);

        assert_eq!(app.saved_crafts.len(), 1);
        assert_eq!(app.active_saved_craft_index, Some(0));
        assert_eq!(app.pending_delete_index, None);
    }

    #[test]
    fn apply_delete_recipe_keeps_active_index_when_before_deleted_item() {
        let mut app = MdcraftApp::default();
        app.saved_crafts.push(SavedCraft {
            name: "A".to_string(),
            recipe_text: String::new(),
            sell_price_input: String::new(),
            item_prices: vec![],
        });
        app.saved_crafts.push(SavedCraft {
            name: "B".to_string(),
            recipe_text: String::new(),
            sell_price_input: String::new(),
            item_prices: vec![],
        });
        app.active_saved_craft_index = Some(0);
        app.pending_delete_index = Some(1);

        apply_delete_recipe(&mut app, 1);

        assert_eq!(app.saved_crafts.len(), 1);
        assert_eq!(app.active_saved_craft_index, Some(0));
        assert_eq!(app.pending_delete_index, None);
    }

    #[test]
    fn handle_cancel_delete_click_only_clears_when_clicked() {
        let mut app = MdcraftApp::default();
        app.pending_delete_index = Some(2);

        handle_cancel_delete_click(&mut app, false);
        assert_eq!(app.pending_delete_index, Some(2));

        handle_cancel_delete_click(&mut app, true);
        assert_eq!(app.pending_delete_index, None);
    }

    #[test]
    fn handle_confirm_delete_click_only_deletes_when_clicked() {
        let mut app = MdcraftApp::default();
        app.saved_crafts.push(SavedCraft {
            name: "A".to_string(),
            recipe_text: String::new(),
            sell_price_input: String::new(),
            item_prices: vec![],
        });
        app.pending_delete_index = Some(0);

        handle_confirm_delete_click(&mut app, 0, false);
        assert_eq!(app.saved_crafts.len(), 1);
        assert_eq!(app.pending_delete_index, Some(0));

        handle_confirm_delete_click(&mut app, 0, true);
        assert!(app.saved_crafts.is_empty());
        assert_eq!(app.pending_delete_index, None);
    }
}

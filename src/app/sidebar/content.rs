use eframe::egui;

use crate::app::MdcraftApp;

use super::json_io;

mod actions;
mod header;
mod save_prompt;
mod saved_recipes;

pub(super) fn render_sidebar_header(ui: &mut egui::Ui, app: &mut MdcraftApp) {
    header::render_sidebar_header(ui, app);
}

pub(super) fn render_sidebar_content(ui: &mut egui::Ui, app: &mut MdcraftApp, content_w: f32) {
    let content_w = content_w.max(120.0);
    let has_saved_crafts = !app.saved_crafts.is_empty();

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(10.0);

    egui::TopBottomPanel::bottom(egui::Id::new("sidebar_json_actions_bottom"))
        .show_separator_line(false)
        .resizable(false)
        .show_inside(ui, |ui| {
            json_io::render_sidebar_json_actions(ui, app, content_w, has_saved_crafts);
        });

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(ui.available_height().max(120.0))
        .show(ui, |ui| {
            let has_recipe = !app.input_text.trim().is_empty() && !app.items.is_empty();
            if has_recipe {
                let save_button = egui::Button::new(
                    egui::RichText::new("Salvar receita atual")
                        .strong()
                        .color(egui::Color32::from_rgb(245, 251, 244)),
                )
                .fill(egui::Color32::from_rgb(48, 118, 78))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(86, 168, 120),
                ))
                .corner_radius(egui::CornerRadius::same(8));

                let save_clicked = ui
                    .add_sized([content_w, 34.0], save_button)
                    .on_hover_text("Salvar a receita atual com nome automático ou manual")
                    .clicked();
                save_prompt::start_save_recipe_prompt(app, save_clicked);
            } else {
                ui.label(egui::RichText::new("Adicione uma receita para salvar").weak());
            }

            save_prompt::render_save_name_prompt(ui, app, content_w);
            saved_recipes::render_saved_recipes_list(ui, app, content_w);
        });
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::MdcraftApp;

    use super::{render_sidebar_content, render_sidebar_header};

    #[test]
    fn render_sidebar_header_and_content_do_not_panic() {
        let mut app = MdcraftApp::default();
        app.input_text = "1 Iron Ore".to_string();
        app.items = vec![];

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_sidebar_header(ui, &mut app);
                render_sidebar_content(ui, &mut app, 220.0);
            });
        });
    }

    #[test]
    fn render_sidebar_content_handles_pending_name_input_state() {
        let mut app = MdcraftApp::default();
        app.sidebar_open = true;
        app.awaiting_craft_name = true;
        app.focus_craft_name_input = true;
        app.pending_craft_name = "Minha Receita".to_string();
        app.input_text = "1 Iron Ore".to_string();
        app.items = vec![crate::model::Item {
            nome: "Iron Ore".to_string(),
            quantidade: 1,
            preco_unitario: 0.0,
            valor_total: 0.0,
            is_resource: true,
            preco_input: String::new(),
        }];

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_sidebar_content(ui, &mut app, 260.0);
            });
        });

        assert!(app.awaiting_craft_name);
    }
}

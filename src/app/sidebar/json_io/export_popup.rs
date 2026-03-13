use eframe::egui;

use crate::app::MdcraftApp;

use super::colors::action_button_colors;
use super::super::import_export::{
    close_export_popup, handle_export_close_click, handle_export_copy_click,
};
use super::super::json_viewer::json_layout_job;

pub(super) fn render_export_recipes_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
    if !app.awaiting_export_json {
        return;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close_export_popup(app);
        return;
    }

    let screen_size = ctx.content_rect().size();
    let popup_size = egui::vec2(
        (screen_size.x * 0.9).clamp(420.0, 760.0),
        (screen_size.y * 0.9).clamp(320.0, 620.0),
    );

    egui::Window::new("Exportar Receitas")
        .id(egui::Id::new("export_saved_recipes_json"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .fixed_size(popup_size)
        .show(ctx, |ui| {
            let mut json_layouter =
                |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                    ui.ctx().fonts_mut(|fonts| {
                        fonts.layout_job(json_layout_job(ui, text.as_str(), wrap_width))
                    })
                };

            let (action_fill, action_stroke, action_text) = action_button_colors(ui);

            egui::TopBottomPanel::top(egui::Id::new("export_popup_header"))
                .show_separator_line(false)
                .resizable(false)
                .show_inside(ui, |ui| {
                    ui.label(
                        egui::RichText::new("JSON de exportacao das receitas salvas")
                            .strong()
                            .size(15.0),
                    );
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Copie o conteudo abaixo.").weak());

                    if let Some(feedback) = &app.export_feedback {
                        ui.add_space(6.0);
                        ui.label(feedback);
                    }

                    ui.add_space(8.0);
                });

            egui::TopBottomPanel::bottom(egui::Id::new("export_popup_actions"))
                .show_separator_line(false)
                .resizable(false)
                .show_inside(ui, |ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let copied = ui
                            .add_sized(
                                [120.0, 32.0],
                                egui::Button::new(
                                    egui::RichText::new("Copiar").strong().color(action_text),
                                )
                                .fill(action_fill)
                                .stroke(action_stroke),
                            )
                            .on_hover_text("Copiar JSON para a area de transferencia")
                            .clicked();

                        handle_export_copy_click(ui.ctx(), app, copied);

                        let close_clicked = ui
                            .add_sized([120.0, 32.0], egui::Button::new("Fechar"))
                            .clicked();
                        handle_export_close_click(app, close_clicked);
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_sized(
                            [
                                ui.available_width().max(240.0),
                                ui.available_height().max(160.0),
                            ],
                            egui::TextEdit::multiline(&mut app.export_json_output)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .margin(egui::vec2(10.0, 10.0))
                                .layouter(&mut json_layouter)
                                .interactive(false),
                        );
                    });
            });
        });
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::MdcraftApp;

    use super::render_export_recipes_popup;

    #[test]
    fn render_export_popup_open_state_runs_without_panicking() {
        let mut app = MdcraftApp::default();
        app.awaiting_export_json = true;
        app.export_json_output = "{\"saved_crafts\":[{\"name\":\"A\",\"recipe_text\":\"1 X\",\"sell_price_input\":\"2k\",\"item_prices\":[]}]}".to_string();

        egui::__run_test_ctx(|ctx| {
            render_export_recipes_popup(ctx, &mut app);
        });

        assert!(app.awaiting_export_json);
    }

    #[test]
    fn render_export_popup_closes_on_escape() {
        let mut app = MdcraftApp::default();
        app.awaiting_export_json = true;

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
            render_export_recipes_popup(ctx, &mut app);
        });

        assert!(!app.awaiting_export_json);
    }

    #[test]
    fn render_export_popup_shows_feedback_when_present() {
        let mut app = MdcraftApp::default();
        app.awaiting_export_json = true;
        app.export_feedback = Some("export feedback".to_string());
        app.export_json_output = "{}".to_string();

        egui::__run_test_ctx(|ctx| {
            render_export_recipes_popup(ctx, &mut app);
        });
    }
}

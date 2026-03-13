use eframe::egui;

use crate::app::MdcraftApp;

use super::super::import_export::{
    close_import_popup, handle_import_cancel_click, handle_import_confirm_click,
    handle_import_format_click,
};
use super::super::json_viewer::json_layout_job;
use super::super::placeholder;
use super::colors::action_button_colors;

pub(super) fn render_import_recipes_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
    if !app.awaiting_import_json {
        return;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close_import_popup(app);
        return;
    }

    egui::Window::new("Importar Receitas")
        .id(egui::Id::new("import_saved_recipes_json"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(560.0, 390.0))
        .show(ctx, |ui| {
            let mut json_layouter =
                |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                    ui.ctx().fonts_mut(|fonts| {
                        fonts.layout_job(json_layout_job(ui, text.as_str(), wrap_width))
                    })
                };

            let (action_fill, action_stroke, action_text) = action_button_colors(ui);

            ui.label(
                egui::RichText::new("Cole aqui o JSON no formato de receitas salvas")
                    .strong()
                    .size(15.0),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(
                    "Aceita: lista direta (`[{...}]`) ou objeto com `saved_crafts`.",
                )
                .weak(),
            );

            ui.add_space(8.0);
            ui.add_sized(
                [ui.available_width(), 240.0],
                egui::TextEdit::multiline(&mut app.import_json_input)
                    .font(egui::TextStyle::Monospace)
                    .hint_text(placeholder(ui, "[{\"name\":\"Receita X\", ...}]"))
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(10.0, 10.0))
                    .layouter(&mut json_layouter),
            );

            if let Some(feedback) = &app.import_feedback {
                ui.add_space(6.0);
                ui.label(feedback);
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let format_clicked = ui
                    .add_sized(
                        [140.0, 32.0],
                        egui::Button::new(
                            egui::RichText::new("Formatar JSON")
                                .strong()
                                .color(action_text),
                        )
                        .fill(action_fill)
                        .stroke(action_stroke),
                    )
                    .on_hover_text("Organiza e indenta o JSON colado")
                    .clicked();

                handle_import_format_click(app, format_clicked);

                let cancel_clicked = ui
                    .add_sized([120.0, 32.0], egui::Button::new("Cancelar"))
                    .clicked();
                handle_import_cancel_click(app, cancel_clicked);

                let import_clicked = ui
                    .add_sized(
                        [140.0, 32.0],
                        egui::Button::new(egui::RichText::new("Importar").strong()),
                    )
                    .clicked();

                handle_import_confirm_click(app, import_clicked);
            });
        });
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::MdcraftApp;

    use super::render_import_recipes_popup;

    #[test]
    fn render_import_popup_returns_early_when_closed() {
        let mut app = MdcraftApp::default();
        app.awaiting_import_json = false;

        egui::__run_test_ctx(|ctx| {
            render_import_recipes_popup(ctx, &mut app);
        });

        assert!(!app.awaiting_import_json);
    }

    #[test]
    fn render_import_popup_open_state_runs_without_panicking() {
        let mut app = MdcraftApp::default();
        app.awaiting_import_json = true;
        app.import_json_input =
            "[{\"name\":\"A\",\"recipe_text\":\"1 X\",\"sell_price_input\":\"2k\"}]".to_string();

        egui::__run_test_ctx(|ctx| {
            render_import_recipes_popup(ctx, &mut app);
        });

        assert!(app.awaiting_import_json);
    }

    #[test]
    fn render_import_popup_closes_on_escape() {
        let mut app = MdcraftApp::default();
        app.awaiting_import_json = true;

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
            render_import_recipes_popup(ctx, &mut app);
        });

        assert!(!app.awaiting_import_json);
    }

    #[test]
    fn render_import_popup_shows_feedback_when_present() {
        let mut app = MdcraftApp::default();
        app.awaiting_import_json = true;
        app.import_feedback = Some("import feedback".to_string());
        app.import_json_input = "[]".to_string();

        egui::__run_test_ctx(|ctx| {
            render_import_recipes_popup(ctx, &mut app);
        });
    }
}

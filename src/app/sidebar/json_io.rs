use eframe::egui;

use crate::app::MdcraftApp;

use super::import_export::{
    close_export_popup, close_import_popup, handle_export_close_click, handle_export_copy_click,
    handle_import_cancel_click, handle_import_confirm_click, handle_import_format_click,
    handle_sidebar_export_click, handle_sidebar_import_click,
};
use super::json_viewer::json_layout_job;
use super::wiki_sync::{handle_sidebar_wiki_refresh_click, poll_wiki_refresh_result};
use super::placeholder;

fn action_button_colors(ui: &egui::Ui) -> (egui::Color32, egui::Stroke, egui::Color32) {
    let is_dark = ui.visuals().dark_mode;
    let fill = if is_dark {
        egui::Color32::from_rgb(56, 98, 74)
    } else {
        egui::Color32::from_rgb(101, 144, 116)
    };
    let stroke = if is_dark {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(110, 173, 138))
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(78, 120, 95))
    };
    let text = if is_dark {
        egui::Color32::from_rgb(242, 248, 241)
    } else {
        egui::Color32::from_rgb(245, 250, 244)
    };
    (fill, stroke, text)
}

pub(super) fn render_sidebar_json_actions(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
    content_w: f32,
    has_saved_crafts: bool,
) {
    poll_wiki_refresh_result(app);

    let action_w = content_w.min(ui.available_width()).max(1.0);

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    let (action_fill, action_stroke, action_text) = action_button_colors(ui);

    let refresh_label = if app.wiki_refresh_in_progress {
        "Sincronizando..."
    } else {
        "Sincronizar Preços NPC"
    };

    let refresh_clicked = ui
        .scope(|inner| {
            inner.set_enabled(!app.wiki_refresh_in_progress);
            inner
                .add_sized(
                    [action_w, 34.0],
                    egui::Button::new(
                        egui::RichText::new(refresh_label)
                            .strong()
                            .color(action_text),
                    )
                    .fill(action_fill)
                    .stroke(action_stroke),
                )
                .on_hover_text(
                    "Consulta o wiki e atualiza os preços NPC usados como referência",
                )
        })
        .inner
        .clicked();

    handle_sidebar_wiki_refresh_click(app, refresh_clicked);

    if let Some(feedback) = &app.wiki_sync_feedback {
        ui.add_space(6.0);
        ui.label(feedback);
    }

    ui.add_space(8.0);

    let import_clicked = ui
        .add_sized(
            [action_w, 34.0],
            egui::Button::new(
                egui::RichText::new("Importar Receitas (JSON)")
                    .strong()
                    .color(action_text),
            )
            .fill(action_fill)
            .stroke(action_stroke),
        )
        .on_hover_text("Cole um JSON com receitas salvas para importar em lote")
        .clicked();

    handle_sidebar_import_click(app, import_clicked);

    if has_saved_crafts {
        ui.add_space(8.0);

        let export_clicked = ui
            .add_sized(
                [action_w, 34.0],
                egui::Button::new(
                    egui::RichText::new("Exportar Receitas (JSON)")
                        .strong()
                        .color(action_text),
                )
                .fill(action_fill)
                .stroke(action_stroke),
            )
            .on_hover_text("Gera um JSON com todas as receitas salvas")
            .clicked();

        handle_sidebar_export_click(app, export_clicked);
    }
}

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
                        egui::RichText::new("JSON de exportação das receitas salvas")
                            .strong()
                            .size(15.0),
                    );
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Copie o conteúdo abaixo.").weak());

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

    use crate::app::{MdcraftApp, SavedCraft};

    fn sample_craft(name: &str) -> SavedCraft {
        SavedCraft {
            name: name.to_string(),
            recipe_text: "1 Iron Ore".to_string(),
            sell_price_input: "10k".to_string(),
            item_prices: vec![],
        }
    }

    #[test]
    fn render_sidebar_json_actions_runs_for_both_empty_and_non_empty_state() {
        let mut empty_app = MdcraftApp::default();
        let mut app_with_crafts = MdcraftApp::default();
        app_with_crafts.saved_crafts.push(sample_craft("Com dados"));

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                super::render_sidebar_json_actions(ui, &mut empty_app, 220.0, false);
            });
        });

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                super::render_sidebar_json_actions(ui, &mut app_with_crafts, 220.0, true);
            });
        });
    }

    #[test]
    fn render_import_popup_returns_early_when_closed() {
        let mut app = MdcraftApp::default();
        app.awaiting_import_json = false;

        egui::__run_test_ctx(|ctx| {
            super::render_import_recipes_popup(ctx, &mut app);
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
            super::render_import_recipes_popup(ctx, &mut app);
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
            super::render_import_recipes_popup(ctx, &mut app);
        });

        assert!(!app.awaiting_import_json);
    }

    #[test]
    fn render_export_popup_open_state_runs_without_panicking() {
        let mut app = MdcraftApp::default();
        app.awaiting_export_json = true;
        app.export_json_output = "{\"saved_crafts\":[{\"name\":\"A\",\"recipe_text\":\"1 X\",\"sell_price_input\":\"2k\",\"item_prices\":[]}]}".to_string();

        egui::__run_test_ctx(|ctx| {
            super::render_export_recipes_popup(ctx, &mut app);
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
            super::render_export_recipes_popup(ctx, &mut app);
        });

        assert!(!app.awaiting_export_json);
    }

    #[test]
    fn action_button_colors_returns_non_zero_stroke_and_alpha() {
        egui::__run_test_ui(|ui| {
            let (fill, stroke, _text) = super::action_button_colors(ui);
            assert!(fill.a() > 0);
            assert!(stroke.width > 0.0);
        });
    }

    #[test]
    fn render_popups_show_feedback_labels_when_present() {
        let mut app = MdcraftApp::default();
        app.awaiting_import_json = true;
        app.import_feedback = Some("import feedback".to_string());
        app.import_json_input = "[]".to_string();

        egui::__run_test_ctx(|ctx| {
            super::render_import_recipes_popup(ctx, &mut app);
        });

        app.awaiting_export_json = true;
        app.export_feedback = Some("export feedback".to_string());
        app.export_json_output = "{}".to_string();

        egui::__run_test_ctx(|ctx| {
            super::render_export_recipes_popup(ctx, &mut app);
        });
    }
}


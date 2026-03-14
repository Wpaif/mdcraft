use eframe::egui;

use crate::app::MdcraftApp;

use super::super::import_export::{handle_sidebar_export_click, handle_sidebar_import_click};
use super::super::wiki_sync::{handle_sidebar_wiki_refresh_click, poll_wiki_refresh_result};
use super::colors::action_button_colors;

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
        "Sincronizar Precos NPC"
    };

    let refresh_clicked = ui
        .add_enabled_ui(!app.wiki_refresh_in_progress, |ui| {
            ui.add_sized(
                [action_w, 34.0],
                egui::Button::new(
                    egui::RichText::new(refresh_label)
                        .strong()
                        .color(action_text),
                )
                .fill(action_fill)
                .stroke(action_stroke),
            )
            .on_hover_text("Consulta o wiki e atualiza os precos NPC usados como referencia")
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

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::{MdcraftApp, SavedCraft};

    use super::render_sidebar_json_actions;

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
                render_sidebar_json_actions(ui, &mut empty_app, 220.0, false);
            });
        });

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_sidebar_json_actions(ui, &mut app_with_crafts, 220.0, true);
            });
        });
    }
}

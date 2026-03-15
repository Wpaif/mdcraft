use eframe::egui;

use crate::app::MdcraftApp;

use super::super::import_export::{handle_sidebar_export_click, handle_sidebar_import_click};
use super::super::wiki_sync::{poll_wiki_refresh_result};
use super::colors::action_button_colors;

fn clear_feedback_after_timeout(started: &mut Option<std::time::Instant>, feedback: &mut Option<String>, timeout: f32) {
    if let (Some(s), Some(_)) = (started.as_ref(), feedback.as_ref()) {
        if s.elapsed().as_secs_f32() > timeout {
            *feedback = None;
            *started = None;
        }
    }
}

pub(super) fn render_sidebar_json_actions(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
    content_w: f32,
    has_saved_crafts: bool,
) {
    clear_feedback_after_timeout(&mut app.wiki_sync_success_anim_started_at, &mut app.wiki_sync_feedback, 2.5);
    clear_feedback_after_timeout(&mut app.wiki_sync_error_anim_started_at, &mut app.wiki_sync_feedback, 2.5);
    poll_wiki_refresh_result(app);

    let action_w = content_w.min(ui.available_width()).max(1.0);

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    let (action_fill, action_stroke, action_text) = action_button_colors(ui);

    // Bloqueia botão de sync durante erro/interrupção cooldown
    let error_cooldown = app.wiki_sync_error_anim_started_at
        .map(|t| t.elapsed().as_secs_f32() < 2.5)
        .unwrap_or(false);


    // Botão de sincronização removido: sincronização agora é automática/assíncrona


    // Espaço e campo de mensagem de feedback removidos: não são mais necessários

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

        #[test]
        fn wiki_sync_feedback_sucesso_limpa_apos_cooldown() {
            let mut app = MdcraftApp::default();
            app.wiki_sync_feedback = Some("Sincronizado com sucesso".to_string());
            app.wiki_sync_success_anim_started_at = Some(std::time::Instant::now() - std::time::Duration::from_secs_f32(3.0));
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render_sidebar_json_actions(ui, &mut app, 220.0, false);
                });
            });
            assert!(app.wiki_sync_feedback.is_none());
            assert!(app.wiki_sync_success_anim_started_at.is_none());
        }

        #[test]
        fn wiki_sync_feedback_erro_limpa_apos_cooldown() {
            let mut app = MdcraftApp::default();
            app.wiki_sync_feedback = Some("Erro de sincronização".to_string());
            app.wiki_sync_error_anim_started_at = Some(std::time::Instant::now() - std::time::Duration::from_secs_f32(3.0));
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render_sidebar_json_actions(ui, &mut app, 220.0, false);
                });
            });
            assert!(app.wiki_sync_feedback.is_none());
            assert!(app.wiki_sync_error_anim_started_at.is_none());
        }

        #[test]
        fn wiki_sync_feedback_mantem_durante_cooldown() {
            let mut app = MdcraftApp::default();
            app.wiki_sync_feedback = Some("Sincronizando...".to_string());
            app.wiki_sync_success_anim_started_at = Some(std::time::Instant::now());
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render_sidebar_json_actions(ui, &mut app, 220.0, false);
                });
            });
            assert!(app.wiki_sync_feedback.is_some());
        }
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

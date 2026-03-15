use crate::app::state::RecipeSavePopupType;
use eframe::egui;

use super::detect_system_theme;
use super::sidebar::{poll_sidebar_background_tasks, render_sidebar};
use super::styles::{setup_custom_styles, setup_emoji_support};
use super::theme_toggle::render_theme_toggle_area;
use super::ui_sections::{
    collect_found_resources, render_closing, render_craft_input, render_items_and_values,
};

#[path = "ui/layout.rs"]
mod layout;
#[path = "ui/shortcuts.rs"]
mod shortcuts;
#[cfg(test)]
#[path = "ui/tests.rs"]
mod tests;
#[path = "ui/toast.rs"]
pub mod toast;

use layout::content_padding;
use shortcuts::apply_sidebar_toggle_shortcut;
use toast::render_wiki_sync_success_toast;

impl super::MdcraftApp {
    fn render_main_ui(&mut self, ctx: &egui::Context) {
        // Sempre aplicar o tema antes de qualquer renderização
        if !self.fonts_loaded {
            setup_custom_styles(ctx);
            setup_emoji_support(ctx);
            self.fonts_loaded = true;
        }
        if self.follow_system_theme {
            let system_theme = detect_system_theme();
            if self.theme != system_theme {
                self.theme = system_theme;
            }
        }
        ctx.set_visuals(self.theme.visuals());

        apply_sidebar_toggle_shortcut(self, ctx);
        poll_sidebar_background_tasks(self);
        render_sidebar(ctx, self);

        render_theme_toggle_area(self, ctx);
        render_wiki_sync_success_toast(self, ctx);

        // Toast de erro na sincronização da wiki
        if let (Some(started_at), Some(msg)) = (self.wiki_sync_error_anim_started_at, self.wiki_sync_feedback.as_ref()) {
            use crate::app::ui::toast::render_toast_area;
            // Cores modernas para erro
            let bg = egui::Color32::from_rgb(180, 40, 40); // vermelho escuro
            let border = egui::Color32::from_rgb(255, 120, 120); // vermelho claro
            let sub_color = egui::Color32::from_rgb(255, 220, 220); // pastel
            let finished = render_toast_area(
                ctx,
                egui::Id::new("wiki_sync_error_toast"),
                "Erro ao sincronizar com a wiki",
                Some(msg.as_str()),
                bg,
                border,
                sub_color,
                started_at,
                std::time::Duration::from_millis(3200),
            );
            if finished {
                self.wiki_sync_error_anim_started_at = None;
                self.wiki_sync_feedback = None;
                self.wiki_refresh_in_progress = false;
            }
        }

        // Toast de sucesso ao salvar/atualizar receita
        if let Some(kind) = self.show_recipe_save_popup {
            use crate::app::ui::toast::render_toast_area;
            let (main_text, sub_text, bg, border, sub_color) = match kind {
                RecipeSavePopupType::Save => {
                    let name = self.last_saved_recipe_name.as_deref().unwrap_or("");
                    (
                        "Receita salva!",
                        Some(format!("{} salvo com sucesso.", name)),
                        egui::Color32::from_rgb(26, 127, 55), // verde vibrante
                        egui::Color32::from_rgb(193, 255, 214), // borda clara
                        egui::Color32::from_rgb(228, 255, 237),
                    )
                },
                RecipeSavePopupType::Update => {
                    let name = self.last_saved_recipe_name.as_deref().unwrap_or("");
                    (
                        "Receita atualizada!",
                        Some(format!("{} atualizado com sucesso.", name)),
                        egui::Color32::from_rgb(70, 90, 120), // azul escuro
                        egui::Color32::from_rgb(180, 200, 255),
                        egui::Color32::from_rgb(220, 230, 255),
                    )
                },
            };
            let finished = if let Some(started_at) = self.recipe_save_toast_started_at {
                render_toast_area(
                    ctx,
                    egui::Id::new("recipe_save_toast"),
                    main_text,
                    sub_text.as_deref(),
                    bg,
                    border,
                    sub_color,
                    started_at,
                    std::time::Duration::from_millis(2600),
                )
            } else {
                false
            };
            if finished {
                self.show_recipe_save_popup = None;
                self.recipe_save_toast_started_at = None;
                self.last_saved_recipe_name = None;
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
            let available_width = ui.available_width();
            let padding = content_padding(available_width);

            egui::Frame::NONE
                .fill(ui.visuals().window_fill())
                .inner_margin(egui::Margin::symmetric(padding, 20))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading(egui::RichText::new("Mdcraft Calculator").strong());
                            });

                            ui.add_space(20.0);
                            let content_width = ui.available_width();

                            render_craft_input(ui, self, content_width);

                            ui.add_space(20.0);

                            let mut total_cost: f64 = 0.0;
                            let found_resources = collect_found_resources(self);

                            render_items_and_values(ui, self, content_width, &mut total_cost);

                            ui.add_space(20.0);

                            render_closing(ui, self, content_width, total_cost, &found_resources);
                        });
                });
        });
    }
}

impl eframe::App for super::MdcraftApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_main_ui(ctx);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.save_app_settings(storage);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.persist_saved_crafts_to_sqlite();
    }
}

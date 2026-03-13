use eframe::egui;

use super::detect_system_theme;
use super::sidebar::render_sidebar;
use super::styles::{setup_custom_styles, setup_emoji_support};
use super::ui_sections::{
    collect_found_resources, render_closing, render_craft_input, render_items_and_values,
};

impl eframe::App for super::MdcraftApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        render_sidebar(ctx, self);

        if !self.fonts_loaded {
            setup_custom_styles(ctx);
            setup_emoji_support(ctx);
            ctx.set_visuals(self.theme.visuals());
            self.fonts_loaded = true;
        }

        if self.follow_system_theme {
            let system_theme = detect_system_theme();
            if self.theme != system_theme {
                self.theme = system_theme;
                ctx.set_visuals(self.theme.visuals());
            }
        }

        egui::Area::new(egui::Id::new("theme_toggle_area"))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.menu_button(egui::RichText::new("⚙").size(18.0), |ui| {
                    ui.label(egui::RichText::new("Tema").strong());
                    ui.add_space(4.0);

                    let manual_label = if self.theme == super::Theme::Dark {
                        "☀ Alternar para claro"
                    } else {
                        "🌙 Alternar para escuro"
                    };

                    let manual_toggle_clicked = ui
                        .add_sized(
                            [190.0, 32.0],
                            egui::Button::new(egui::RichText::new(manual_label).strong()),
                        )
                        .on_hover_text("Alternar tema manualmente")
                        .clicked();

                    if manual_toggle_clicked {
                        // Manual toggle turns off automatic OS sync.
                        self.follow_system_theme = false;
                        self.theme = self.theme.toggle();
                        ctx.set_visuals(self.theme.visuals());
                        ui.close();
                    }

                    ui.separator();

                    let follow_resp = ui
                        .checkbox(&mut self.follow_system_theme, "Seguir sistema")
                        .on_hover_text("Usa o tema claro/escuro do sistema operacional");

                    if follow_resp.changed() && self.follow_system_theme {
                        self.theme = detect_system_theme();
                        ctx.set_visuals(self.theme.visuals());
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_width = ui.available_width();
            let max_width = available_width.min(1600.0);
            let padding = ((available_width - max_width) / 2.0).max(10.0) as i8;

            egui::Frame::NONE
                .fill(ui.visuals().panel_fill)
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

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.save_app_settings(storage);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use eframe::{App, Storage};

    use super::super::{APP_SETTINGS_KEY, MdcraftApp};

    #[derive(Default)]
    struct MemoryStorage {
        values: HashMap<String, String>,
    }

    impl Storage for MemoryStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.values.get(key).cloned()
        }

        fn set_string(&mut self, key: &str, value: String) {
            self.values.insert(key.to_string(), value);
        }

        fn flush(&mut self) {}
    }

    #[test]
    fn save_persists_app_settings_key() {
        let mut app = MdcraftApp::default();
        let mut storage = MemoryStorage::default();

        App::save(&mut app, &mut storage);

        assert!(storage.get_string(APP_SETTINGS_KEY).is_some());
    }
}

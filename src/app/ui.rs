use eframe::egui;
use std::time::Duration;

use super::detect_system_theme;
use super::sidebar::{poll_sidebar_background_tasks, render_sidebar};
use super::styles::{setup_custom_styles, setup_emoji_support};
use super::ui_sections::{
    collect_found_resources, render_closing, render_craft_input, render_items_and_values,
};

fn content_padding(available_width: f32) -> i8 {
    let max_width = available_width.min(1600.0);
    ((available_width - max_width) / 2.0).max(10.0) as i8
}

fn manual_toggle_label(theme: super::Theme) -> &'static str {
    if theme == super::Theme::Dark {
        "☀ Alternar para claro"
    } else {
        "🌙 Alternar para escuro"
    }
}

fn apply_manual_theme_toggle(app: &mut super::MdcraftApp, ctx: &egui::Context) {
    app.follow_system_theme = false;
    app.theme = app.theme.toggle();
    ctx.set_visuals(app.theme.visuals());
}

fn apply_follow_system_theme_if_changed(
    app: &mut super::MdcraftApp,
    ctx: &egui::Context,
    changed: bool,
) {
    if changed && app.follow_system_theme {
        app.theme = detect_system_theme();
        ctx.set_visuals(app.theme.visuals());
    }
}

fn render_theme_toggle_menu(ui: &mut egui::Ui, app: &mut super::MdcraftApp, ctx: &egui::Context) {
    ui.label(egui::RichText::new("Tema").strong());
    ui.add_space(4.0);

    let manual_label = manual_toggle_label(app.theme);

    let manual_toggle_clicked = ui
        .add_sized(
            [190.0, 32.0],
            egui::Button::new(egui::RichText::new(manual_label).strong()),
        )
        .on_hover_text("Alternar tema manualmente")
        .clicked();

    let should_close = apply_manual_toggle_if_clicked(app, ctx, manual_toggle_clicked);
    close_ui_if_requested(ui, should_close);

    ui.separator();

    let follow_resp = ui
        .checkbox(&mut app.follow_system_theme, "Seguir sistema")
        .on_hover_text("Usa o tema claro/escuro do sistema operacional");

    apply_follow_system_theme_if_changed(app, ctx, follow_resp.changed());
}

fn apply_manual_toggle_if_clicked(
    app: &mut super::MdcraftApp,
    ctx: &egui::Context,
    clicked: bool,
) -> bool {
    if clicked {
        // Manual toggle turns off automatic OS sync.
        apply_manual_theme_toggle(app, ctx);
        return true;
    }

    false
}

fn close_ui_if_requested(ui: &mut egui::Ui, should_close: bool) {
    if should_close {
        ui.close();
    }
}

fn render_theme_toggle_menu_content(
    ui: &mut egui::Ui,
    app: &mut super::MdcraftApp,
    ctx: &egui::Context,
) {
    render_theme_toggle_menu(ui, app, ctx);
}

fn render_theme_toggle_button(
    ui: &mut egui::Ui,
    app: &mut super::MdcraftApp,
    ctx: &egui::Context,
    force_open: bool,
) {
    if force_open {
        render_theme_toggle_menu_content(ui, app, ctx);
        return;
    }

    ui.menu_button(egui::RichText::new("⚙").size(18.0), |ui| {
        render_theme_toggle_menu_content(ui, app, ctx);
    });
}

fn render_theme_toggle_area(app: &mut super::MdcraftApp, ctx: &egui::Context) {
    egui::Area::new(egui::Id::new("theme_toggle_area"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            render_theme_toggle_button(ui, app, ctx, false);
        });
}

fn render_wiki_sync_success_toast(app: &mut super::MdcraftApp, ctx: &egui::Context) {
    let Some(started_at) = app.wiki_sync_success_anim_started_at else {
        return;
    };

    let total = Duration::from_millis(2600);
    let elapsed = started_at.elapsed();

    if elapsed >= total {
        app.wiki_sync_success_anim_started_at = None;
        return;
    }

    let t = (elapsed.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0);
    let fade_in = (t / 0.18).clamp(0.0, 1.0);
    let fade_out = ((1.0 - t) / 0.24).clamp(0.0, 1.0);
    let alpha = (fade_in * fade_out).clamp(0.0, 1.0);
    let y_offset = (1.0 - fade_in) * 14.0;

    egui::Area::new(egui::Id::new("wiki_sync_success_toast"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-22.0, -22.0 + y_offset))
        .show(ctx, |ui| {
            let bg = egui::Color32::from_rgba_unmultiplied(26, 127, 55, (225.0 * alpha) as u8);
            let stroke = egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(193, 255, 214, (200.0 * alpha) as u8),
            );

            egui::Frame::new()
                .fill(bg)
                .stroke(stroke)
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Base de precos atualizada")
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.label(
                        egui::RichText::new("Dados sincronizados com a wiki")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(228, 255, 237)),
                    );
                });
        });

    ctx.request_repaint();
}

impl super::MdcraftApp {
    fn render_main_ui(&mut self, ctx: &egui::Context) {
        poll_sidebar_background_tasks(self);
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

        render_theme_toggle_area(self, ctx);
        render_wiki_sync_success_toast(self, ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_width = ui.available_width();
            let padding = content_padding(available_width);

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
}

impl eframe::App for super::MdcraftApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_main_ui(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.save_app_settings(storage);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use eframe::{App, Frame, Storage, egui};

    use super::super::{APP_SETTINGS_KEY, MdcraftApp, Theme};
    use super::{
        apply_follow_system_theme_if_changed, apply_manual_theme_toggle,
        apply_manual_toggle_if_clicked, close_ui_if_requested, content_padding, manual_toggle_label,
        render_theme_toggle_area, render_theme_toggle_button, render_theme_toggle_menu,
        render_theme_toggle_menu_content, render_wiki_sync_success_toast,
    };

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

        storage.flush();
    }

    #[test]
    fn update_delegates_to_render_main_ui() {
        let mut app = MdcraftApp::default();
        app.fonts_loaded = false;

        let ctx = egui::Context::default();
        let mut frame = Frame::_new_kittest();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            App::update(&mut app, ctx, &mut frame);
        });

        assert!(app.fonts_loaded);
    }

    #[test]
    fn content_padding_respects_minimum_and_centering() {
        assert_eq!(content_padding(800.0), 10);
        assert_eq!(content_padding(2000.0), i8::MAX);
    }

    #[test]
    fn manual_toggle_label_changes_by_theme() {
        assert!(manual_toggle_label(Theme::Dark).contains("claro"));
        assert!(manual_toggle_label(Theme::Light).contains("escuro"));
    }

    #[test]
    fn render_main_ui_runs_and_initializes_fonts_flag() {
        let mut app = MdcraftApp::default();
        app.fonts_loaded = false;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.render_main_ui(ctx);
        });

        assert!(app.fonts_loaded);
    }

    #[test]
    fn render_main_ui_keeps_manual_theme_when_follow_is_disabled() {
        let mut app = MdcraftApp::default();
        app.follow_system_theme = false;
        app.theme = Theme::Dark;
        app.fonts_loaded = true;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.render_main_ui(ctx);
        });

        assert_eq!(app.theme, Theme::Dark);
    }

    #[test]
    fn render_main_ui_syncs_theme_when_follow_is_enabled() {
        let mut app = MdcraftApp::default();
        app.follow_system_theme = true;
        app.fonts_loaded = true;
        app.theme = super::super::detect_system_theme().toggle();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.render_main_ui(ctx);
        });

        let system_theme = super::super::detect_system_theme();
        assert!(app.theme == system_theme || app.theme == system_theme.toggle());
        assert!(app.follow_system_theme);
    }

    #[test]
    fn apply_manual_theme_toggle_turns_off_follow_and_flips_theme() {
        let mut app = MdcraftApp::default();
        app.follow_system_theme = true;
        app.theme = Theme::Light;

        let ctx = egui::Context::default();
        apply_manual_theme_toggle(&mut app, &ctx);

        assert!(!app.follow_system_theme);
        assert_eq!(app.theme, Theme::Dark);
    }

    #[test]
    fn apply_manual_theme_toggle_flips_dark_back_to_light() {
        let mut app = MdcraftApp::default();
        app.follow_system_theme = true;
        app.theme = Theme::Dark;

        let ctx = egui::Context::default();
        apply_manual_theme_toggle(&mut app, &ctx);

        assert!(!app.follow_system_theme);
        assert_eq!(app.theme, Theme::Light);
    }

    #[test]
    fn apply_manual_toggle_if_clicked_returns_false_without_click() {
        let mut app = MdcraftApp::default();
        app.follow_system_theme = true;
        app.theme = Theme::Light;

        let ctx = egui::Context::default();
        let should_close = apply_manual_toggle_if_clicked(&mut app, &ctx, false);

        assert!(!should_close);
        assert!(app.follow_system_theme);
        assert_eq!(app.theme, Theme::Light);
    }

    #[test]
    fn apply_manual_toggle_if_clicked_returns_true_when_clicked() {
        let mut app = MdcraftApp::default();
        app.follow_system_theme = true;
        app.theme = Theme::Light;

        let ctx = egui::Context::default();
        let should_close = apply_manual_toggle_if_clicked(&mut app, &ctx, true);

        assert!(should_close);
        assert!(!app.follow_system_theme);
        assert_eq!(app.theme, Theme::Dark);
    }

    #[test]
    fn render_theme_toggle_menu_runs_in_test_ui() {
        let mut app = MdcraftApp::default();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_theme_toggle_menu(ui, &mut app, ctx);
            });
        });
    }

    #[test]
    fn render_theme_toggle_area_runs_without_panicking() {
        let mut app = MdcraftApp::default();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            render_theme_toggle_area(&mut app, ctx);
        });
    }

    #[test]
    fn render_theme_toggle_menu_content_runs_without_panicking() {
        let mut app = MdcraftApp::default();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_theme_toggle_menu_content(ui, &mut app, ctx);
            });
        });
    }

    #[test]
    fn render_theme_toggle_button_force_open_runs_menu_content() {
        let mut app = MdcraftApp::default();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_theme_toggle_button(ui, &mut app, ctx, true);
            });
        });
    }

    #[test]
    fn render_wiki_sync_success_toast_lifecycle_runs_without_panicking() {
        let mut app = MdcraftApp::default();
        app.wiki_sync_success_anim_started_at = Some(std::time::Instant::now());

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            render_wiki_sync_success_toast(&mut app, ctx);
        });

        app.wiki_sync_success_anim_started_at = Some(
            std::time::Instant::now() - std::time::Duration::from_secs(10),
        );
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            render_wiki_sync_success_toast(&mut app, ctx);
        });

        assert!(app.wiki_sync_success_anim_started_at.is_none());
    }

    #[test]
    fn render_theme_toggle_area_handles_pointer_click_on_menu_button() {
        let mut app = MdcraftApp::default();
        let ctx = egui::Context::default();

        let mut input = egui::RawInput::default();
        input.screen_rect = Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(360.0, 240.0),
        ));
        input.events = vec![
            egui::Event::PointerMoved(egui::pos2(344.0, 24.0)),
            egui::Event::PointerButton {
                pos: egui::pos2(344.0, 24.0),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos: egui::pos2(344.0, 24.0),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ];

        let _ = ctx.run(input, |ctx| {
            render_theme_toggle_area(&mut app, ctx);
        });
    }

    #[test]
    fn close_ui_if_requested_runs_for_both_flags() {
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                close_ui_if_requested(ui, false);
                close_ui_if_requested(ui, true);
            });
        });
    }

    #[test]
    fn apply_follow_system_theme_if_changed_is_noop_when_not_changed() {
        let mut app = MdcraftApp::default();
        app.follow_system_theme = true;
        app.theme = Theme::Light;

        let ctx = egui::Context::default();
        apply_follow_system_theme_if_changed(&mut app, &ctx, false);

        assert_eq!(app.theme, Theme::Light);
    }

    #[test]
    fn apply_follow_system_theme_if_changed_is_noop_when_follow_disabled() {
        let mut app = MdcraftApp::default();
        app.follow_system_theme = false;
        app.theme = Theme::Light;

        let ctx = egui::Context::default();
        apply_follow_system_theme_if_changed(&mut app, &ctx, true);

        assert_eq!(app.theme, Theme::Light);
    }

    #[test]
    fn apply_follow_system_theme_if_changed_updates_theme_when_enabled() {
        let mut app = MdcraftApp::default();
        app.follow_system_theme = true;
        let expected = super::super::detect_system_theme();
        app.theme = expected.toggle();

        let ctx = egui::Context::default();
        apply_follow_system_theme_if_changed(&mut app, &ctx, true);

        assert_eq!(app.theme, expected);
    }
}

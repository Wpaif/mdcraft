use eframe::egui;

use super::{MdcraftApp, Theme, detect_system_theme};

pub(super) fn manual_toggle_label(theme: Theme) -> &'static str {
    if theme == Theme::Dark {
        "☀ Alternar para claro"
    } else {
        "🌙 Alternar para escuro"
    }
}

pub(super) fn apply_manual_theme_toggle(app: &mut MdcraftApp, ctx: &egui::Context) {
    app.follow_system_theme = false;
    app.theme = app.theme.toggle();
    ctx.set_visuals(app.theme.visuals());
}

pub(super) fn apply_follow_system_theme_if_changed(
    app: &mut MdcraftApp,
    ctx: &egui::Context,
    changed: bool,
) {
    if changed && app.follow_system_theme {
        app.theme = detect_system_theme();
        ctx.set_visuals(app.theme.visuals());
    }
}

pub(super) fn apply_manual_toggle_if_clicked(
    app: &mut MdcraftApp,
    ctx: &egui::Context,
    clicked: bool,
) -> bool {
    if clicked {
        apply_manual_theme_toggle(app, ctx);
        return true;
    }

    false
}

pub(super) fn close_ui_if_requested(ui: &mut egui::Ui, should_close: bool) {
    if should_close {
        ui.close();
    }
}

fn render_theme_toggle_menu_content(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
    ctx: &egui::Context,
) {
    render_theme_toggle_menu(ui, app, ctx);
}

fn render_theme_toggle_menu(ui: &mut egui::Ui, app: &mut MdcraftApp, ctx: &egui::Context) {
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

fn render_theme_toggle_button(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
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

pub(super) fn render_theme_toggle_area(app: &mut MdcraftApp, ctx: &egui::Context) {
    egui::Area::new(egui::Id::new("theme_toggle_area"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            render_theme_toggle_button(ui, app, ctx, false);
        });
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use super::{
        apply_follow_system_theme_if_changed, apply_manual_theme_toggle,
        apply_manual_toggle_if_clicked, close_ui_if_requested, manual_toggle_label,
        render_theme_toggle_area, render_theme_toggle_button, render_theme_toggle_menu,
        render_theme_toggle_menu_content,
    };
    use crate::app::{MdcraftApp, Theme};

    #[test]
    fn manual_toggle_label_changes_by_theme() {
        assert!(manual_toggle_label(Theme::Dark).contains("claro"));
        assert!(manual_toggle_label(Theme::Light).contains("escuro"));
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
        let expected = crate::app::detect_system_theme();
        app.theme = expected.toggle();

        let ctx = egui::Context::default();
        apply_follow_system_theme_if_changed(&mut app, &ctx, true);

        assert_eq!(app.theme, expected);
    }
}

use eframe::egui;

use super::logic::{
    apply_follow_system_theme_if_changed, apply_follow_system_theme_if_changed_with,
    apply_manual_theme_toggle, apply_manual_toggle_if_clicked, close_ui_if_requested,
    manual_toggle_label,
};
use super::menu::{
    render_theme_toggle_button, render_theme_toggle_menu, render_theme_toggle_menu_content,
};
use super::render_theme_toggle_area;
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
    app.theme = Theme::Light;

    let ctx = egui::Context::default();
    apply_follow_system_theme_if_changed_with(&mut app, &ctx, true, || Theme::Dark);

    assert_eq!(app.theme, Theme::Dark);
}

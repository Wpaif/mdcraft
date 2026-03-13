use std::collections::HashMap;

use eframe::{App, Frame, Storage, egui};

use super::super::{APP_SETTINGS_KEY, MdcraftApp, Theme};
use super::{apply_sidebar_toggle_shortcut, content_padding, render_wiki_sync_success_toast};

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
fn render_wiki_sync_success_toast_lifecycle_runs_without_panicking() {
    let mut app = MdcraftApp::default();
    app.wiki_sync_success_anim_started_at = Some(std::time::Instant::now());

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        render_wiki_sync_success_toast(&mut app, ctx);
    });

    app.wiki_sync_success_anim_started_at =
        Some(std::time::Instant::now() - std::time::Duration::from_secs(10));
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        render_wiki_sync_success_toast(&mut app, ctx);
    });

    assert!(app.wiki_sync_success_anim_started_at.is_none());
}

#[test]
fn apply_sidebar_toggle_shortcut_toggles_with_ctrl_e() {
    let mut app = MdcraftApp::default();
    app.sidebar_open = true;

    let ctx = egui::Context::default();
    let mut input = egui::RawInput::default();
    input.modifiers = egui::Modifiers {
        ctrl: true,
        ..Default::default()
    };
    input.events.push(egui::Event::Key {
        key: egui::Key::E,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
    });

    let _ = ctx.run(input, |ctx| {
        apply_sidebar_toggle_shortcut(&mut app, ctx);
    });

    assert!(!app.sidebar_open);
}

#[test]
fn apply_sidebar_toggle_shortcut_ignores_plain_e() {
    let mut app = MdcraftApp::default();
    app.sidebar_open = true;

    let ctx = egui::Context::default();
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::Key {
        key: egui::Key::E,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });

    let _ = ctx.run(input, |ctx| {
        apply_sidebar_toggle_shortcut(&mut app, ctx);
    });

    assert!(app.sidebar_open);
}

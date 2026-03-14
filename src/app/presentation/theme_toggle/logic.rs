use eframe::egui;

use crate::app::{MdcraftApp, Theme, detect_system_theme};

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
    apply_follow_system_theme_if_changed_with(app, ctx, changed, detect_system_theme);
}

pub(super) fn apply_follow_system_theme_if_changed_with(
    app: &mut MdcraftApp,
    ctx: &egui::Context,
    changed: bool,
    detect_theme: impl FnOnce() -> Theme,
) {
    if changed && app.follow_system_theme {
        app.theme = detect_theme();
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

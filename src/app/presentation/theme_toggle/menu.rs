use eframe::egui;

use crate::app::MdcraftApp;

use super::logic::{
    apply_follow_system_theme_if_changed, apply_manual_toggle_if_clicked, close_ui_if_requested,
    manual_toggle_label,
};

pub(super) fn render_theme_toggle_menu_content(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
    ctx: &egui::Context,
) {
    render_theme_toggle_menu(ui, app, ctx);
}

pub(super) fn render_theme_toggle_menu(
    ui: &mut egui::Ui,
    app: &mut MdcraftApp,
    ctx: &egui::Context,
) {
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

pub(super) fn render_theme_toggle_button(
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

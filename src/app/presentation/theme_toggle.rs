use eframe::egui;

use super::MdcraftApp;

#[path = "theme_toggle/logic.rs"]
mod logic;
#[path = "theme_toggle/menu.rs"]
mod menu;
#[cfg(test)]
#[path = "theme_toggle/tests.rs"]
mod tests;

use menu::render_theme_toggle_button;

pub(super) fn render_theme_toggle_area(app: &mut MdcraftApp, ctx: &egui::Context) {
    egui::Area::new(egui::Id::new("theme_toggle_area"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            render_theme_toggle_button(ui, app, ctx, false);
        });
}

use eframe::egui;

use crate::app::MdcraftApp;

mod delete_logic;
mod delete_popup;

pub(super) fn render_delete_confirmation_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
    delete_popup::render_delete_confirmation_popup(ctx, app);
}

#[cfg(test)]
mod tests;

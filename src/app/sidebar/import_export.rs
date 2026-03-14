use eframe::egui;

use crate::app::MdcraftApp;

mod handlers;
mod json_codec;
mod state;

pub(super) fn close_import_popup(app: &mut MdcraftApp) {
    state::close_import_popup(app);
}

pub(super) fn close_export_popup(app: &mut MdcraftApp) {
    state::close_export_popup(app);
}

pub(super) fn handle_sidebar_import_click(app: &mut MdcraftApp, import_clicked: bool) {
    handlers::handle_sidebar_import_click(app, import_clicked);
}

pub(super) fn handle_sidebar_export_click(app: &mut MdcraftApp, export_clicked: bool) {
    handlers::handle_sidebar_export_click(app, export_clicked);
}

pub(super) fn handle_import_format_click(app: &mut MdcraftApp, format_clicked: bool) {
    handlers::handle_import_format_click(app, format_clicked);
}

pub(super) fn handle_import_confirm_click(app: &mut MdcraftApp, import_clicked: bool) {
    handlers::handle_import_confirm_click(app, import_clicked);
}

pub(super) fn handle_export_copy_click(ctx: &egui::Context, app: &mut MdcraftApp, copied: bool) {
    handlers::handle_export_copy_click(ctx, app, copied);
}

pub(super) fn handle_import_cancel_click(app: &mut MdcraftApp, cancel_clicked: bool) {
    handlers::handle_import_cancel_click(app, cancel_clicked);
}

pub(super) fn handle_export_close_click(app: &mut MdcraftApp, close_clicked: bool) {
    handlers::handle_export_close_click(app, close_clicked);
}

#[cfg(test)]
mod tests;

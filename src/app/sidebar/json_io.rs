mod actions;
mod colors;
mod export_popup;
mod import_popup;

use eframe::egui;

use crate::app::MdcraftApp;

pub(super) fn render_sidebar_json_actions(
	ui: &mut egui::Ui,
	app: &mut MdcraftApp,
	content_w: f32,
	has_saved_crafts: bool,
) {
	actions::render_sidebar_json_actions(ui, app, content_w, has_saved_crafts);
}

pub(super) fn render_import_recipes_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
	import_popup::render_import_recipes_popup(ctx, app);
}

pub(super) fn render_export_recipes_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
	export_popup::render_export_recipes_popup(ctx, app);
}

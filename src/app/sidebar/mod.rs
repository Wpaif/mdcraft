use eframe::egui;

use super::MdcraftApp;

mod content;
mod dialogs;
mod import_export;
mod json_io;
mod json_viewer;
mod wiki_sync;

const SIDEBAR_WIDTH_EXPANDED: f32 = 260.0;
const SIDEBAR_WIDTH_COLLAPSED: f32 = 56.0;

pub(super) use super::{capitalize_display_name, placeholder};

pub(super) fn render_sidebar(ctx: &egui::Context, app: &mut MdcraftApp) {
    let width = if app.sidebar_open {
        SIDEBAR_WIDTH_EXPANDED
    } else {
        SIDEBAR_WIDTH_COLLAPSED
    };

    egui::SidePanel::left(egui::Id::new("sidebar_panel"))
        .resizable(false)
        .exact_width(width)
        .show_separator_line(false)
        .show(ctx, |ui| {
            let panel_fill = ui.visuals().panel_fill;

            egui::Frame::NONE
                .fill(panel_fill)
                .inner_margin(egui::Margin::symmetric(10, 10))
                .show(ui, |ui| {
                    let content_w = ui.available_width();
                    content::render_sidebar_header(ui, app);

                    if app.sidebar_open {
                        content::render_sidebar_content(ui, app, content_w);
                    }
                });
        });

    dialogs::render_delete_confirmation_popup(ctx, app);
    json_io::render_import_recipes_popup(ctx, app);
    json_io::render_export_recipes_popup(ctx, app);
}

pub(super) fn poll_sidebar_background_tasks(app: &mut MdcraftApp) {
    wiki_sync::ensure_wiki_refresh_started(app);
    wiki_sync::poll_wiki_refresh_result(app);
    wiki_sync::poll_craft_refresh_result(app);
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use super::{capitalize_display_name, placeholder};

    #[test]
    fn capitalize_display_name_capitalizes_words_and_trims_spaces() {
        assert_eq!(capitalize_display_name("  iron  ore  "), "Iron Ore");
        assert_eq!(capitalize_display_name("sUPER   PoTiOn"), "Super Potion");
    }

    #[test]
    fn placeholder_can_be_used_in_ui_without_panicking() {
        egui::__run_test_ui(|ui| {
            let hint = placeholder(ui, "abc");
            ui.label(hint);
        });
    }
}

use eframe::egui;

use crate::app::MdcraftApp;

fn toggle_sidebar(app: &mut MdcraftApp, clicked: bool) {
    if clicked {
        app.sidebar_open = !app.sidebar_open;
    }
}

pub(super) fn sidebar_header_bg_color(
    hovered: bool,
    hovered_bg: egui::Color32,
    inactive_bg: egui::Color32,
) -> egui::Color32 {
    if hovered { hovered_bg } else { inactive_bg }
}

pub(super) fn render_sidebar_header(ui: &mut egui::Ui, app: &mut MdcraftApp) {
    ui.horizontal(|ui| {
        let toggle_icon = if app.sidebar_open { "◀" } else { "▶" };
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());

        let bg = sidebar_header_bg_color(
            resp.hovered(),
            ui.visuals().widgets.hovered.bg_fill,
            ui.visuals().widgets.inactive.bg_fill,
        );
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(6), bg);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            toggle_icon,
            egui::TextStyle::Button.resolve(ui.style()),
            ui.visuals().text_color(),
        );

        toggle_sidebar(app, resp.clicked());

        if app.sidebar_open {
            ui.label(egui::RichText::new("RECEITAS").strong());
        }
    });
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::MdcraftApp;

    use super::{render_sidebar_header, sidebar_header_bg_color};

    #[test]
    fn sidebar_header_bg_color_selects_hover_or_inactive_color() {
        let hovered = sidebar_header_bg_color(
            true,
            egui::Color32::from_rgb(10, 20, 30),
            egui::Color32::from_rgb(40, 50, 60),
        );
        assert_eq!(hovered, egui::Color32::from_rgb(10, 20, 30));

        let inactive = sidebar_header_bg_color(
            false,
            egui::Color32::from_rgb(10, 20, 30),
            egui::Color32::from_rgb(40, 50, 60),
        );
        assert_eq!(inactive, egui::Color32::from_rgb(40, 50, 60));
    }

    #[test]
    fn render_sidebar_header_runs_without_panicking() {
        let mut app = MdcraftApp::default();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_sidebar_header(ui, &mut app);
            });
        });
    }
}

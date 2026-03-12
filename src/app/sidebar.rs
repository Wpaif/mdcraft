use eframe::egui;

use super::MdcraftApp;

const SIDEBAR_WIDTH_EXPANDED: f32 = 260.0;
const SIDEBAR_WIDTH_COLLAPSED: f32 = 56.0;

pub(super) fn render_sidebar(ctx: &egui::Context, app: &mut MdcraftApp) {
    let width = if app.sidebar_open {
        SIDEBAR_WIDTH_EXPANDED
    } else {
        SIDEBAR_WIDTH_COLLAPSED
    };

    egui::SidePanel::left(egui::Id::new("sidebar_panel"))
        .resizable(false)
        .exact_width(width)
        .show(ctx, |ui| {
            let panel_fill = ui.visuals().panel_fill;
            let stroke_color = ui.visuals().widgets.noninteractive.bg_stroke.color;

            egui::Frame::NONE
                .fill(panel_fill)
                .stroke(egui::Stroke::new(1.0, stroke_color))
                .inner_margin(egui::Margin::symmetric(10, 10))
                .show(ui, |ui| {
                    render_sidebar_header(ui, app);

                    if app.sidebar_open {
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(10.0);

                        let has_recipe = !app.input_text.trim().is_empty() && !app.items.is_empty();
                        if has_recipe {
                            let _ = ui.add_sized(
                                [SIDEBAR_WIDTH_EXPANDED - 28.0, 32.0],
                                egui::Button::new("Salvar receita atual"),
                            );
                        } else {
                            ui.label(egui::RichText::new("Adicione uma receita para salvar").weak());
                        }
                    }
                });
        });
}

fn render_sidebar_header(ui: &mut egui::Ui, app: &mut MdcraftApp) {
    ui.horizontal(|ui| {
        let toggle_icon = if app.sidebar_open { "◀" } else { "▶" };
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());

        let bg = if resp.hovered() {
            ui.visuals().widgets.hovered.bg_fill
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(6), bg);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            toggle_icon,
            egui::TextStyle::Button.resolve(ui.style()),
            ui.visuals().text_color(),
        );

        if resp.clicked() {
            app.sidebar_open = !app.sidebar_open;
        }

        if app.sidebar_open {
            ui.label(egui::RichText::new("Menu lateral").strong());
        }
    });
}

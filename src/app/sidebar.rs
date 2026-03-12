use eframe::egui;

use super::{MdcraftApp, SavedCraft};

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

            egui::Frame::NONE
                .fill(panel_fill)
                .inner_margin(egui::Margin::symmetric(10, 10))
                .show(ui, |ui| {
                    let content_w = ui.available_width();
                    render_sidebar_header(ui, app);

                    if app.sidebar_open {
                        render_sidebar_content(ui, app, content_w);
                    }
                });
        });
}

fn render_sidebar_content(ui: &mut egui::Ui, app: &mut MdcraftApp, content_w: f32) {
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(10.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let has_recipe = !app.input_text.trim().is_empty() && !app.items.is_empty();
            if has_recipe {
                let save_clicked = ui
                    .add_sized([content_w, 32.0], egui::Button::new("Salvar receita atual"))
                    .clicked();

                if save_clicked {
                    app.awaiting_craft_name = true;
                    if app.pending_craft_name.trim().is_empty() {
                        app.pending_craft_name = format!("Receita {}", app.saved_crafts.len() + 1);
                    }
                }
            } else {
                ui.label(egui::RichText::new("Adicione uma receita para salvar").weak());
            }

            if app.awaiting_craft_name {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Nome da receita:").strong());

                let name_resp = ui.add_sized(
                    [content_w, 30.0],
                    egui::TextEdit::singleline(&mut app.pending_craft_name)
                        .hint_text("Pressione Enter para salvar"),
                );

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    app.awaiting_craft_name = false;
                    app.pending_craft_name.clear();
                }

                let save_by_enter = name_resp.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !app.pending_craft_name.trim().is_empty();

                if save_by_enter {
                    app.saved_crafts.insert(
                        0,
                        SavedCraft {
                            name: app.pending_craft_name.trim().to_string(),
                            recipe_text: app.input_text.clone(),
                            sell_price_input: app.sell_price_input.clone(),
                        },
                    );
                    app.awaiting_craft_name = false;
                    app.pending_craft_name.clear();
                }
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Receitas salvas").strong());
            ui.add_space(6.0);

            if app.saved_crafts.is_empty() {
                ui.label(egui::RichText::new("Nenhuma receita salva ainda.").weak());
            } else {
                for craft in &app.saved_crafts {
                    ui.group(|ui| {
                        ui.set_width(content_w);
                        ui.label(egui::RichText::new(&craft.name).strong());

                        let info = format!(
                            "{} caracteres{}",
                            craft.recipe_text.chars().count(),
                            if craft.sell_price_input.trim().is_empty() {
                                ""
                            } else {
                                " | com preco final"
                            }
                        );
                        ui.label(egui::RichText::new(info).small().weak());
                    });
                    ui.add_space(6.0);
                }
            }
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

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
        .show_separator_line(false)
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

    render_delete_confirmation_popup(ctx, app);
}

fn render_sidebar_content(ui: &mut egui::Ui, app: &mut MdcraftApp, content_w: f32) {
    let content_w = content_w.max(120.0);

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
                    app.pending_craft_name.clear();
                    app.focus_craft_name_input = true;
                }
            } else {
                ui.label(egui::RichText::new("Adicione uma receita para salvar").weak());
            }

            if app.awaiting_craft_name {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Nome da receita:").strong());

                let mut name_resp_opt: Option<egui::Response> = None;
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui| {
                        let input_width = (content_w - 12.0).max(80.0);
                        let name_resp = ui.add_sized(
                            [input_width, 30.0],
                            egui::TextEdit::singleline(&mut app.pending_craft_name)
                                .hint_text("Digite um nome ou pressione Enter"),
                        );
                        name_resp_opt = Some(name_resp);
                    });

                let name_resp = name_resp_opt.expect("name input response should exist");

                if app.focus_craft_name_input {
                    name_resp.request_focus();
                    app.focus_craft_name_input = false;
                }

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    app.awaiting_craft_name = false;
                    app.pending_craft_name.clear();
                    app.focus_craft_name_input = false;
                }

                let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let save_by_enter = enter_pressed;

                if save_by_enter {
                    let fallback_name = format!("Receita {}", app.saved_crafts.len() + 1);
                    let raw_name = if app.pending_craft_name.trim().is_empty() {
                        fallback_name
                    } else {
                        app.pending_craft_name.clone()
                    };
                    let normalized_name = normalize_craft_name(&raw_name);
                    app.saved_crafts.insert(
                        0,
                        SavedCraft {
                            name: normalized_name,
                            recipe_text: app.input_text.clone(),
                            sell_price_input: app.sell_price_input.clone(),
                        },
                    );
                    app.awaiting_craft_name = false;
                    app.pending_craft_name.clear();
                    app.focus_craft_name_input = false;
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
                let mut pending_click_delete: Option<usize> = None;

                for (idx, craft) in app.saved_crafts.iter().enumerate() {
                    ui.group(|ui| {
                        ui.set_width(content_w);
                        let name_text = normalize_craft_name(&craft.name);
                        let saved_lines = craft
                            .recipe_text
                            .lines()
                            .filter(|line| !line.trim().is_empty())
                            .count();
                        let has_saved_price = !craft.sell_price_input.trim().is_empty();
                        let hover_details = if has_saved_price {
                            format!("{} linhas salvas | com preco final", saved_lines)
                        } else {
                            format!("{} linhas salvas", saved_lines)
                        };
                        let row_height = 24.0;
                        let icon_size = 22.0;
                        ui.allocate_ui(egui::vec2(content_w, row_height), |ui| {
                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let text_width =
                                        (content_w - icon_size - ui.spacing().item_spacing.x - 8.0)
                                            .max(80.0);
                                    let (name_rect, name_resp) = ui.allocate_exact_size(
                                        egui::vec2(text_width, row_height),
                                        egui::Sense::hover(),
                                    );
                                    name_resp.on_hover_text(hover_details);
                                    ui.painter().text(
                                        egui::pos2(name_rect.left(), name_rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        name_text,
                                        egui::FontId::proportional(16.0),
                                        ui.visuals().text_color(),
                                    );

                                    let delete_btn = egui::Button::new(
                                        egui::RichText::new("🗑")
                                            .size(13.0)
                                            .color(egui::Color32::from_rgb(220, 98, 98)),
                                    )
                                    .fill(egui::Color32::from_rgba_unmultiplied(220, 98, 98, 32))
                                    .stroke(egui::Stroke::new(
                                        1.0,
                                        egui::Color32::from_rgb(180, 72, 72),
                                    ));

                                    let delete_clicked = ui
                                        .add_sized([icon_size, icon_size], delete_btn)
                                        .on_hover_text("Excluir receita")
                                        .clicked();

                                    if delete_clicked {
                                        pending_click_delete = Some(idx);
                                    }
                                },
                            );
                        });
                    });
                    ui.add_space(6.0);
                }

                if let Some(idx) = pending_click_delete {
                    app.pending_delete_index = Some(idx);
                }
            }
        });
}

fn normalize_craft_name(raw_name: &str) -> String {
    raw_name
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let first = first.to_uppercase().collect::<String>();
                    let rest = chars.as_str().to_lowercase();
                    format!("{}{}", first, rest)
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_delete_confirmation_popup(ctx: &egui::Context, app: &mut MdcraftApp) {
    let Some(idx) = app.pending_delete_index else {
        return;
    };

    if idx >= app.saved_crafts.len() {
        app.pending_delete_index = None;
        return;
    }

    let recipe_name = app.saved_crafts[idx].name.clone();

    egui::Window::new("Confirmar Exclusão")
        .id(egui::Id::new("confirm_delete_saved_recipe"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(400.0, 190.0))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Deseja realmente apagar esta receita?")
                        .strong()
                        .size(12.0),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("'{}'", recipe_name))
                        .weak()
                        .size(11.0),
                );
            });

            ui.add_space(12.0);
            let cancel_fill = ui.visuals().widgets.inactive.bg_fill;
            let delete_fill = egui::Color32::from_rgb(181, 61, 61);
            let row_height = 32.0;
            let button_width = 120.0;
            let spacing = ui.spacing().item_spacing.x;
            let total_buttons_width = (button_width * 2.0) + spacing;
            let left_pad = ((ui.available_width() - total_buttons_width) * 0.5).max(0.0);

            ui.horizontal(|ui| {
                ui.add_space(left_pad);

                if ui
                    .add_sized(
                        [button_width, row_height],
                        egui::Button::new("Cancelar").fill(cancel_fill),
                    )
                    .clicked()
                {
                    app.pending_delete_index = None;
                }

                if ui
                    .add_sized(
                        [button_width, row_height],
                        egui::Button::new("Apagar").fill(delete_fill),
                    )
                    .clicked()
                {
                    app.saved_crafts.remove(idx);
                    app.pending_delete_index = None;
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

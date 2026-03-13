use eframe::egui;

use crate::app::{MdcraftApp, SavedCraft};
use crate::parse::parse_clipboard;

use super::{json_io, normalize_craft_name, placeholder};

pub(super) fn render_sidebar_content(ui: &mut egui::Ui, app: &mut MdcraftApp, content_w: f32) {
    let content_w = content_w.max(120.0);
    let has_saved_crafts = !app.saved_crafts.is_empty();

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(10.0);

    let footer_h = if has_saved_crafts { 126.0 } else { 86.0 };
    let scroll_h = (ui.available_height() - footer_h).max(120.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(scroll_h)
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
                                .hint_text(placeholder(ui, "Digite um nome ou pressione Enter")),
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
                    app.active_saved_craft_index = app.active_saved_craft_index.map(|idx| idx + 1);
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
                let mut pending_click_select: Option<usize> = None;

                for (idx, craft) in app.saved_crafts.iter().enumerate() {
                    ui.group(|ui| {
                        ui.set_width(content_w);
                        let is_active = app.active_saved_craft_index == Some(idx);
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
                        let row_height = 26.0;
                        let icon_size = 22.0;
                        ui.allocate_ui_with_layout(
                            egui::vec2(content_w, row_height),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                let text_width =
                                    (content_w - icon_size - ui.spacing().item_spacing.x - 8.0)
                                        .max(80.0);

                                let name_fill = if is_active {
                                    ui.visuals().faint_bg_color
                                } else {
                                    ui.visuals().widgets.inactive.bg_fill
                                };
                                let name_stroke = if is_active {
                                    ui.visuals().widgets.active.bg_stroke
                                } else {
                                    ui.visuals().widgets.inactive.bg_stroke
                                };

                                let name_btn = egui::Button::new(
                                    egui::RichText::new(name_text)
                                        .size(16.0)
                                        .color(ui.visuals().text_color()),
                                )
                                .fill(name_fill)
                                .stroke(name_stroke);

                                let name_resp = ui
                                    .add_sized([text_width, icon_size], name_btn)
                                    .on_hover_text(hover_details);

                                if name_resp.clicked() {
                                    pending_click_select = Some(idx);
                                }

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
                    ui.add_space(6.0);
                }

                if let Some(idx) = pending_click_delete {
                    app.pending_delete_index = Some(idx);
                }

                if let Some(idx) = pending_click_select {
                    load_saved_craft_for_edit(app, idx);
                }
            }
        });

    json_io::render_sidebar_json_actions(ui, app, content_w, has_saved_crafts);
}

fn load_saved_craft_for_edit(app: &mut MdcraftApp, idx: usize) {
    let Some(craft) = app.saved_crafts.get(idx) else {
        return;
    };

    app.input_text = craft.recipe_text.clone();
    app.sell_price_input = craft.sell_price_input.clone();

    let resources: Vec<&str> = app.resource_list.iter().map(AsRef::as_ref).collect();
    app.items = parse_clipboard(&app.input_text, &resources);
    app.active_saved_craft_index = Some(idx);
}

pub(super) fn render_sidebar_header(ui: &mut egui::Ui, app: &mut MdcraftApp) {
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
            ui.label(egui::RichText::new("RECEITAS").strong());
        }
    });
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::{MdcraftApp, SavedCraft};

    use super::{load_saved_craft_for_edit, render_sidebar_content, render_sidebar_header};

    fn make_saved_craft(name: &str, recipe_text: &str, sell_price_input: &str) -> SavedCraft {
        SavedCraft {
            name: name.to_string(),
            recipe_text: recipe_text.to_string(),
            sell_price_input: sell_price_input.to_string(),
        }
    }

    #[test]
    fn load_saved_craft_for_edit_updates_active_data() {
        let mut app = MdcraftApp::default();
        app.saved_crafts.push(make_saved_craft(
            "teste",
            "2 Iron Ore, 3 Screw",
            "12k",
        ));

        load_saved_craft_for_edit(&mut app, 0);

        assert_eq!(app.active_saved_craft_index, Some(0));
        assert_eq!(app.sell_price_input, "12k");
        assert!(!app.items.is_empty());
    }

    #[test]
    fn load_saved_craft_for_edit_ignores_out_of_bounds_index() {
        let mut app = MdcraftApp::default();
        app.input_text = "unchanged".to_string();
        app.saved_crafts
            .push(make_saved_craft("teste", "1 Iron Ore", "1k"));

        load_saved_craft_for_edit(&mut app, 9);

        assert_eq!(app.input_text, "unchanged");
        assert_eq!(app.active_saved_craft_index, None);
    }

    #[test]
    fn render_sidebar_header_and_content_do_not_panic() {
        let mut app = MdcraftApp::default();
        app.input_text = "1 Iron Ore".to_string();
        app.items = vec![];

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_sidebar_header(ui, &mut app);
                render_sidebar_content(ui, &mut app, 220.0);
            });
        });
    }
}

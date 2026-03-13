use eframe::egui;

use crate::app::MdcraftApp;

use super::actions::{apply_pending_sidebar_actions, set_pending_action};
use super::super::capitalize_display_name;

pub(super) fn render_saved_recipes_list(ui: &mut egui::Ui, app: &mut MdcraftApp, content_w: f32) {
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Receitas salvas").strong());
    ui.add_space(6.0);

    if app.saved_crafts.is_empty() {
        ui.label(egui::RichText::new("Nenhuma receita salva ainda.").weak());
        return;
    }

    let mut pending_click_delete: Option<usize> = None;
    let mut pending_click_select: Option<usize> = None;

    for (idx, craft) in app.saved_crafts.iter().enumerate() {
        let is_active = app.active_saved_craft_index == Some(idx);
        let name_text = capitalize_display_name(&craft.name);
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

        let item_fill = if is_active {
            ui.visuals().faint_bg_color
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };
        let item_stroke = if is_active {
            ui.visuals().widgets.active.bg_stroke
        } else {
            ui.visuals().widgets.inactive.bg_stroke
        };

        egui::Frame::new()
            .fill(item_fill)
            .stroke(item_stroke)
            .corner_radius(egui::CornerRadius::same(4))
            .inner_margin(egui::Margin::symmetric(8, 5))
            .show(ui, |ui| {
                ui.set_width(content_w);
                let row_height = 22.0;
                let icon_size = 20.0;
                ui.allocate_ui_with_layout(
                    egui::vec2(content_w, row_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        let text_width =
                            (content_w - icon_size - ui.spacing().item_spacing.x - 8.0)
                                .max(80.0);

                        let name_btn = egui::Button::new(
                            egui::RichText::new(&name_text)
                                .size(14.0)
                                .color(ui.visuals().text_color()),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE);

                        let name_resp = ui
                            .add_sized([text_width, icon_size], name_btn)
                            .on_hover_text(hover_details);
                        set_pending_action(
                            &mut pending_click_select,
                            idx,
                            name_resp.clicked(),
                        );

                        let (delete_rect, delete_resp) = ui.allocate_exact_size(
                            egui::vec2(icon_size, icon_size),
                            egui::Sense::click(),
                        );
                        let delete_fill = if delete_resp.hovered() {
                            egui::Color32::from_rgba_unmultiplied(220, 98, 98, 44)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(220, 98, 98, 32)
                        };
                        ui.painter().rect(
                            delete_rect,
                            egui::CornerRadius::same(4),
                            delete_fill,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 72, 72)),
                            egui::StrokeKind::Middle,
                        );
                        ui.painter().text(
                            delete_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "🗑",
                            egui::FontId::proportional(13.0),
                            egui::Color32::from_rgb(220, 98, 98),
                        );

                        let delete_clicked =
                            delete_resp.on_hover_text("Excluir receita").clicked();
                        set_pending_action(
                            &mut pending_click_delete,
                            idx,
                            delete_clicked,
                        );
                    },
                );
            });
        ui.add_space(4.0);
    }

    apply_pending_sidebar_actions(app, pending_click_delete, pending_click_select);
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::{MdcraftApp, SavedCraft};

    use super::render_saved_recipes_list;

    fn make_saved_craft(name: &str, recipe_text: &str, sell_price_input: &str) -> SavedCraft {
        SavedCraft {
            name: name.to_string(),
            recipe_text: recipe_text.to_string(),
            sell_price_input: sell_price_input.to_string(),
            item_prices: vec![],
        }
    }

    #[test]
    fn render_saved_recipes_list_handles_saved_recipes_list() {
        let mut app = MdcraftApp::default();
        app.saved_crafts
            .push(make_saved_craft("receita a", "1 Iron Ore", "3k"));
        app.saved_crafts
            .push(make_saved_craft("receita b", "2 Screw", ""));
        app.active_saved_craft_index = Some(0);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_saved_recipes_list(ui, &mut app, 260.0);
            });
        });

        assert_eq!(app.saved_crafts.len(), 2);
    }
}

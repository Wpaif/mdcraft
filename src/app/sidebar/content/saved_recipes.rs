use eframe::egui;

use crate::app::MdcraftApp;

use super::actions::{apply_pending_sidebar_actions, set_pending_action};
use super::super::capitalize_display_name;

fn paint_centered_trash_icon(ui: &egui::Ui, button_rect: egui::Rect, color: egui::Color32) {
    let painter = ui.painter();
    let icon_rect = button_rect.shrink2(egui::vec2(5.5, 4.5));
    let stroke = egui::Stroke::new(1.35, color);

    // Body
    let body_top = icon_rect.top() + icon_rect.height() * 0.24;
    let body_rect = egui::Rect::from_min_max(
        egui::pos2(icon_rect.left() + icon_rect.width() * 0.2, body_top),
        egui::pos2(icon_rect.right() - icon_rect.width() * 0.2, icon_rect.bottom()),
    );
    painter.rect_stroke(
        body_rect,
        egui::CornerRadius::same(1),
        stroke,
        egui::StrokeKind::Middle,
    );

    // Lid and handle
    let lid_y = body_rect.top() - icon_rect.height() * 0.16;
    painter.line_segment(
        [
            egui::pos2(body_rect.left() - 0.6, lid_y),
            egui::pos2(body_rect.right() + 0.6, lid_y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(icon_rect.center().x - icon_rect.width() * 0.12, lid_y - 1.8),
            egui::pos2(icon_rect.center().x + icon_rect.width() * 0.12, lid_y - 1.8),
        ],
        stroke,
    );

    // Inner slots
    for frac in [0.35_f32, 0.5, 0.65] {
        let x = egui::lerp(body_rect.left()..=body_rect.right(), frac);
        painter.line_segment(
            [
                egui::pos2(x, body_rect.top() + 1.2),
                egui::pos2(x, body_rect.bottom() - 1.2),
            ],
            stroke,
        );
    }
}

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
                        paint_centered_trash_icon(
                            ui,
                            delete_rect,
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

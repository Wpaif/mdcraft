use eframe::egui;

pub(super) fn paint_npc_price_icon(
    ui: &mut egui::Ui,
    has_npc_price: bool,
    is_equal_to_npc: bool,
) -> egui::Response {
    let size = egui::vec2(18.0, 18.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let painter = ui.painter();
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.45;

    let (fill, stroke, text_color) = if !has_npc_price {
        (
            egui::Color32::from_rgba_unmultiplied(120, 120, 120, 24),
            egui::Stroke::new(1.0, egui::Color32::from_rgb(130, 130, 130)),
            egui::Color32::from_rgb(130, 130, 130),
        )
    } else if is_equal_to_npc {
        (
            egui::Color32::from_rgb(29, 155, 240),
            egui::Stroke::new(1.2, egui::Color32::from_rgb(180, 225, 255)),
            egui::Color32::WHITE,
        )
    } else {
        (
            egui::Color32::from_rgba_unmultiplied(29, 155, 240, 36),
            egui::Stroke::new(1.2, egui::Color32::from_rgb(29, 155, 240)),
            egui::Color32::from_rgb(29, 155, 240),
        )
    };

    painter.circle(center, radius, fill, stroke);
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        "N",
        egui::FontId::proportional(10.0),
        text_color,
    );

    response
}

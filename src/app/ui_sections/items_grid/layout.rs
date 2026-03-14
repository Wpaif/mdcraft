use eframe::egui;

pub(super) fn render_empty_item_cells(
    ui: &mut egui::Ui,
    item_w: f32,
    qty_w: f32,
    price_w: f32,
    total_w: f32,
    status_w: f32,
) {
    ui.add_sized([item_w, 22.0], egui::Label::new(" "));
    ui.add_sized([qty_w, 22.0], egui::Label::new(" "));
    ui.add_sized([price_w, 22.0], egui::Label::new(" "));
    ui.add_sized([total_w, 22.0], egui::Label::new(" "));
    ui.add_sized([status_w, 22.0], egui::Label::new(" "));
}

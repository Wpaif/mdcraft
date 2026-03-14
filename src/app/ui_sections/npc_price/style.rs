use std::collections::HashMap;

use eframe::egui;

use super::lookup::compare_item_price_with_npc;
use super::NpcPriceComparison;

pub(super) fn price_input_stroke(
    ui: &egui::Ui,
    item: &crate::model::Item,
    npc_lookup: &HashMap<String, f64>,
) -> egui::Stroke {
    if item.preco_input.trim().is_empty() {
        return egui::Stroke::new(1.4, egui::Color32::from_rgb(235, 188, 90));
    }

    let default = ui.visuals().widgets.inactive.bg_stroke;

    match compare_item_price_with_npc(item, npc_lookup) {
        Some(NpcPriceComparison::HigherThanNpc) => {
            egui::Stroke::new(1.4, egui::Color32::from_rgb(74, 201, 126))
        }
        Some(NpcPriceComparison::LowerThanNpc) => {
            egui::Stroke::new(1.4, egui::Color32::from_rgb(220, 98, 98))
        }
        _ => default,
    }
}

pub(super) fn price_input_fill_color(item: &crate::model::Item) -> egui::Color32 {
    if item.preco_input.trim().is_empty() {
        egui::Color32::from_rgba_unmultiplied(235, 188, 90, 22)
    } else {
        egui::Color32::TRANSPARENT
    }
}

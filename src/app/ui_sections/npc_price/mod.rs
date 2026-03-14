use std::collections::HashMap;

use eframe::egui;

use crate::app::MdcraftApp;

mod icon;
mod lookup;
mod style;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NpcPriceComparison {
    Equal,
    HigherThanNpc,
    LowerThanNpc,
}

pub(super) fn build_npc_price_lookup(app: &MdcraftApp) -> HashMap<String, f64> {
    lookup::build_npc_price_lookup(app)
}

pub(super) fn compare_item_price_with_npc(
    item: &crate::model::Item,
    npc_lookup: &HashMap<String, f64>,
) -> Option<NpcPriceComparison> {
    lookup::compare_item_price_with_npc(item, npc_lookup)
}

pub(super) fn price_input_stroke(
    ui: &egui::Ui,
    item: &crate::model::Item,
    npc_lookup: &HashMap<String, f64>,
) -> egui::Stroke {
    style::price_input_stroke(ui, item, npc_lookup)
}

pub(super) fn price_input_fill_color(item: &crate::model::Item) -> egui::Color32 {
    style::price_input_fill_color(item)
}

pub(super) fn npc_price_for_item(
    item: &crate::model::Item,
    npc_lookup: &HashMap<String, f64>,
) -> Option<f64> {
    lookup::npc_price_for_item(item, npc_lookup)
}

pub(super) fn should_show_npc_price_icon(item_name: &str) -> bool {
    lookup::should_show_npc_price_icon(item_name)
}

pub(super) fn paint_npc_price_icon(
    ui: &mut egui::Ui,
    has_npc_price: bool,
    is_equal_to_npc: bool,
) -> egui::Response {
    icon::paint_npc_price_icon(ui, has_npc_price, is_equal_to_npc)
}

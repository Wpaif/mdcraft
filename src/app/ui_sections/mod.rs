use eframe::egui;

use super::MdcraftApp;

mod closing;
mod craft_input;
mod items_grid;

pub(super) use closing::render_closing;
pub(super) use craft_input::render_craft_input;
pub(super) use items_grid::render_items_and_values;

pub(super) fn placeholder(ui: &egui::Ui, text: &str) -> egui::RichText {
    egui::RichText::new(text).color(ui.visuals().text_color().gamma_multiply(0.7))
}

pub(super) fn capitalize_display_name(raw_name: &str) -> String {
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

pub(super) fn autosave_active_craft(app: &mut MdcraftApp) {
    let Some(idx) = app.active_saved_craft_index else {
        return;
    };

    if let Some(craft) = app.saved_crafts.get_mut(idx) {
        craft.recipe_text = app.input_text.clone();
        craft.sell_price_input = app.sell_price_input.clone();
    }
}

pub(super) fn collect_found_resources(app: &MdcraftApp) -> Vec<(String, u64)> {
    app.items
        .iter()
        .filter(|item| item.is_resource)
        .map(|item| (item.nome.clone(), item.quantidade))
        .collect()
}
